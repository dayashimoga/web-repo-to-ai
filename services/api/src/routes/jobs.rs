//! Job management route handlers.

use std::convert::Infallible;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Extension, State};
use axum::http::header;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures::stream::{self, Stream};
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_core::{JobId, RequestId};
use web_repo_to_ai_exporters::{
    build_manifest, export_html, export_json, export_jsonl, export_manifest, export_markdown,
    export_plain_text, export_zip_package, generate_checksums_manifest, HtmlOptions,
    JsonExportOptions, JsonlOptions, MarkdownOptions, PlainTextOptions, ZipPackageOptions,
};
use web_repo_to_ai_storage::layout;
use web_repo_to_ai_storage::schema::ArtifactFormat;

use crate::dto::{
    ArtifactPathParams, CreateJobRequest, JobPathParams, JobPreviewResponse, JobResponse,
    ListJobsQuery, PaginatedResponse, PreviewFileItem, PreviewQuery,
};
use crate::error::not_found_problem;
use crate::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
use crate::state::AppState;

/// Handles `POST /api/v1/jobs`.
///
/// Accepts a validated job request, initializes a strongly-typed UUID v7 [`JobId`],
/// records submission metrics, and returns `202 Accepted` with initial job descriptor.
pub async fn create_job_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateJobRequest>,
) -> impl IntoResponse {
    let job_id = JobId::new();
    let default_model = &state.config.optimization.default_model;
    let default_budget = state.config.optimization.default_token_budget;

    let response = JobResponse::new_queued(job_id, payload, default_model, default_budget);
    state.metrics.jobs_total.inc("submitted");

    (StatusCode::ACCEPTED, Json(response))
}

/// Handles `GET /api/v1/jobs`.
///
/// Lists conversion jobs with validated pagination and filter criteria.
pub async fn list_jobs_handler(
    State(_state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<ListJobsQuery>,
) -> impl IntoResponse {
    let page = query.page();
    let limit = query.limit();

    // In Phase 2.2, persistence is not yet wired (Phase 2.3).
    // Returns an empty validated paginated envelope.
    let response = PaginatedResponse::<JobResponse>::new(vec![], page, limit, 0);
    (StatusCode::OK, Json(response))
}

/// Handles `GET /api/v1/jobs/{job_id}`.
///
/// Retrieves conversion job status by UUID v7 identifier.
pub async fn get_job_handler(
    State(_state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<JobPathParams>,
) -> impl IntoResponse {
    // Until persistence is wired in Phase 2.3, returns typed 404 Problem Details
    not_found_problem("Job", &params.job_id.to_string(), Some(request_id))
}

/// Handles `GET /api/v1/jobs/{job_id}/events`.
///
/// Streams real-time Server-Sent Events (SSE) representing job lifecycle progression,
/// stage changes, log messages, token metrics, and artifact availability.
pub async fn job_events_handler(
    State(state): State<AppState>,
    ValidatedPath(params): ValidatedPath<JobPathParams>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let job_id = params.job_id;
    let cdm_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::CdmJson);

    let mut events = Vec::new();

    // Check if the job artifact has already been produced and stored
    if let Ok(true) = state.storage.exists(&cdm_key).await {
        // Job is completed in storage
        events.push(
            Event::default().event("phase").data(
                serde_json::json!({
                    "phase": "completed",
                    "progress_pct": 100,
                    "message": "Conversion job completed successfully",
                    "job_id": job_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
            ),
        );

        // Read stored CDM to emit exact completeness and token metrics
        if let Ok(mut reader) = state.storage.open_read(&cdm_key).await {
            let mut buf = Vec::new();
            if reader.read_to_end(&mut buf).await.is_ok() {
                if let Ok(cdm) = serde_json::from_slice::<CdmDocument>(&buf) {
                    let total_tokens = cdm.total_estimated_tokens();
                    events.push(
                        Event::default().event("completeness").data(
                            serde_json::json!({
                                "discovered_count": cdm.completeness.discovered_count,
                                "extracted_count": cdm.completeness.extracted_count,
                                "excluded_count": cdm.completeness.excluded_count,
                                "failed_count": cdm.completeness.failed_count,
                                "is_complete": cdm.completeness.is_complete()
                            })
                            .to_string(),
                        ),
                    );

                    events.push(
                        Event::default().event("tokens").data(
                            serde_json::json!({
                                "raw_tokens": total_tokens,
                                "deduplicated_tokens": total_tokens,
                                "final_tokens": total_tokens,
                                "budget_limit": 160_000,
                                "savings_pct": 0.0
                            })
                            .to_string(),
                        ),
                    );
                }
            }
        }

        // Emit artifact available notifications
        for fmt in &["markdown", "txt", "json", "html", "jsonl", "zip"] {
            events.push(
                Event::default().event("artifact").data(
                    serde_json::json!({
                        "format": fmt,
                        "filename": format!("web-repo-to-ai-{job_id}.{fmt}"),
                        "download_url": format!("/api/v1/jobs/{job_id}/artifacts/{fmt}")
                    })
                    .to_string(),
                ),
            );
        }

        events.push(
            Event::default().event("done").data(
                serde_json::json!({
                    "status": "completed",
                    "job_id": job_id.to_string(),
                    "completed_at": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
            ),
        );
    } else {
        // Initial queued / processing status event
        events.push(
            Event::default().event("phase").data(
                serde_json::json!({
                    "phase": "acquisition",
                    "progress_pct": 10,
                    "message": "Job registered; acquiring source content...",
                    "job_id": job_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
            ),
        );

        events.push(
            Event::default().event("log").data(
                serde_json::json!({
                    "id": format!("log-{}", chrono::Utc::now().timestamp_millis()),
                    "level": "info",
                    "phase": "acquisition",
                    "message": format!("Processing started for job {job_id}"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
                .to_string(),
            ),
        );
    }

    let stream = stream::iter(events.into_iter().map(Ok));
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Handles `GET /api/v1/jobs/{job_id}/artifacts/{format}`.
///
/// Asynchronously streams an exported artifact by format (`markdown`, `txt`, `json`, `html`,
/// `jsonl`, `zip`, `manifest`, `checksums`). Supports streaming pre-rendered artifacts directly
/// from storage or performing on-demand rendering from stored CDM documents.
pub async fn get_artifact_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<ArtifactPathParams>,
) -> Response {
    let raw_format = params.format.trim().to_ascii_lowercase();

    // 1. Identify format and metadata
    let (artifact_format, filename, mime_type) = match raw_format.as_str() {
        "markdown" | "md" => (
            Some(ArtifactFormat::Markdown),
            format!("web-repo-to-ai-{}.md", params.job_id),
            "text/markdown; charset=utf-8",
        ),
        "txt" | "text" => (
            Some(ArtifactFormat::Txt),
            format!("web-repo-to-ai-{}.txt", params.job_id),
            "text/plain; charset=utf-8",
        ),
        "json" => (
            Some(ArtifactFormat::Json),
            format!("web-repo-to-ai-{}.json", params.job_id),
            "application/json",
        ),
        "html" | "htm" => (
            Some(ArtifactFormat::Html),
            format!("web-repo-to-ai-{}.html", params.job_id),
            "text/html; charset=utf-8",
        ),
        "jsonl" | "ndjson" => (
            Some(ArtifactFormat::Jsonl),
            format!("web-repo-to-ai-{}.jsonl", params.job_id),
            "application/x-ndjson",
        ),
        "zip" => (
            Some(ArtifactFormat::Zip),
            format!("web-repo-to-ai-{}.zip", params.job_id),
            "application/zip",
        ),
        "manifest" => (
            Some(ArtifactFormat::Manifest),
            format!("web-repo-to-ai-{}-manifest.json", params.job_id),
            "application/json",
        ),
        "quality_report" | "quality" => (
            Some(ArtifactFormat::QualityReport),
            format!("web-repo-to-ai-{}-quality.json", params.job_id),
            "application/json",
        ),
        "checksums" | "sha256" => (
            None,
            "checksums.sha256".to_string(),
            "text/plain; charset=utf-8",
        ),
        "cdm_json" | "cdm" => (
            Some(ArtifactFormat::CdmJson),
            format!("web-repo-to-ai-{}.cdm.json", params.job_id),
            "application/vnd.antigravity.cdm+json",
        ),
        _ => {
            return not_found_problem(
                "ArtifactFormat",
                &format!("{}:{}", params.job_id, params.format),
                Some(request_id),
            );
        }
    };

    let job_uuid = *params.job_id.as_uuid();
    let primary_key = if let Some(fmt) = artifact_format {
        layout::job_artifact_key(job_uuid, fmt)
    } else {
        format!("jobs/{job_uuid}/checksums.sha256")
    };

    // 2. Check if artifact already exists in storage
    let candidate_keys = [
        primary_key.clone(),
        format!("jobs/{job_uuid}/artifact.{}", raw_format),
        format!("jobs/{job_uuid}/{filename}"),
    ];

    let mut found_key = None;
    for key in &candidate_keys {
        if state.storage.exists(key).await.unwrap_or(false) {
            found_key = Some(key.clone());
            break;
        }
    }

    // 3. If found, stream directly from storage
    if let Some(key) = found_key {
        let meta = state.storage.metadata(&key).await.ok();
        let content_length = meta.map(|m| m.byte_size);

        match state.storage.open_read(&key).await {
            Ok(reader) => {
                let stream = ReaderStream::new(reader);
                let body = Body::from_stream(stream);
                return build_download_response(
                    body,
                    mime_type,
                    &filename,
                    content_length,
                    &request_id,
                );
            }
            Err(err) => {
                tracing::warn!(%err, key = %key, "Failed to open storage reader for artifact");
            }
        }
    }

    // 4. If not pre-rendered in storage, check if parent CDM is available for on-demand generation
    let cdm_keys = [
        layout::job_artifact_key(job_uuid, ArtifactFormat::CdmJson),
        format!("jobs/{job_uuid}/artifact.cdm.json"),
        format!("jobs/{job_uuid}/cdm.json"),
    ];

    let mut found_cdm_key = None;
    for key in &cdm_keys {
        if state.storage.exists(key).await.unwrap_or(false) {
            found_cdm_key = Some(key.clone());
            break;
        }
    }

    if let Some(cdm_key) = found_cdm_key {
        if let Ok(cdm_bytes) = state.storage.get(&cdm_key).await {
            if let Ok(doc) = serde_json::from_slice::<CdmDocument>(&cdm_bytes) {
                // Generate requested artifact on the fly
                let generated =
                    match raw_format.as_str() {
                        "markdown" | "md" => export_markdown(&doc, &MarkdownOptions::default())
                            .map(String::into_bytes),
                        "txt" | "text" => export_plain_text(&doc, &PlainTextOptions::default())
                            .map(String::into_bytes),
                        "json" => export_json(&doc, &JsonExportOptions::canonical())
                            .map(String::into_bytes),
                        "html" | "htm" => {
                            export_html(&doc, &HtmlOptions::default()).map(|(s, _)| s.into_bytes())
                        }
                        "jsonl" | "ndjson" => export_jsonl(&doc, &JsonlOptions::default())
                            .map(|(s, _)| s.into_bytes()),
                        "zip" => {
                            export_zip_package(&doc, &ZipPackageOptions::complete()).map(|(b, _)| b)
                        }
                        "manifest" => export_manifest(&build_manifest(&doc, vec![]), true)
                            .map(String::into_bytes),
                        "checksums" | "sha256" => Ok(generate_checksums_manifest(&[]).into_bytes()),
                        _ => Err(web_repo_to_ai_exporters::ExporterError::InvalidDocument(
                            "Unsupported on-demand export format".to_string(),
                        )),
                    };

                if let Ok(bytes) = generated {
                    let len = bytes.len() as u64;
                    let _ = state.storage.put(&primary_key, &bytes).await;
                    let body = Body::from(bytes);
                    return build_download_response(
                        body,
                        mime_type,
                        &filename,
                        Some(len),
                        &request_id,
                    );
                }
            }
        }
    }

    // 5. If neither artifact nor CDM exists, return RFC 7807 404 Problem Details
    not_found_problem(
        "Artifact",
        &format!("{}:{}", params.job_id, params.format),
        Some(request_id),
    )
}

fn build_download_response(
    body: Body,
    mime_type: &str,
    filename: &str,
    content_length: Option<u64>,
    request_id: &RequestId,
) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .header("x-content-type-options", "nosniff")
        .header("cache-control", "private, no-transform, max-age=86400")
        .header("x-request-id", request_id.to_string());

    if let Some(len) = content_length {
        builder = builder.header(header::CONTENT_LENGTH, len.to_string());
    }

    builder.body(body).unwrap_or_else(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build response",
        )
            .into_response()
    })
}

/// Handles `GET /api/v1/jobs/{job_id}/preview`.
///
/// Retrieves paginated / windowed preview of the converted output for the specified format
/// and repository hierarchy.
pub async fn get_preview_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<JobPathParams>,
    ValidatedQuery(query): ValidatedQuery<PreviewQuery>,
) -> Response {
    let job_uuid = *params.job_id.as_uuid();
    let cdm_keys = [
        layout::job_artifact_key(job_uuid, ArtifactFormat::CdmJson),
        format!("jobs/{job_uuid}/artifact.cdm.json"),
        format!("jobs/{job_uuid}/cdm.json"),
    ];

    let mut found_cdm_key = None;
    for key in &cdm_keys {
        if state.storage.exists(key).await.unwrap_or(false) {
            found_cdm_key = Some(key.clone());
            break;
        }
    }

    let cdm_key = match found_cdm_key {
        Some(k) => k,
        None => {
            return not_found_problem("Job", &params.job_id.to_string(), Some(request_id));
        }
    };

    let cdm_bytes = match state.storage.get(&cdm_key).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(%err, key = %cdm_key, "Failed to read CDM document for preview");
            return not_found_problem("Job", &params.job_id.to_string(), Some(request_id));
        }
    };

    let doc = match serde_json::from_slice::<CdmDocument>(&cdm_bytes) {
        Ok(d) => d,
        Err(err) => {
            tracing::error!(%err, "Corrupted CDM document in storage");
            return not_found_problem("Job", &params.job_id.to_string(), Some(request_id));
        }
    };

    let files: Vec<PreviewFileItem> = doc
        .sections
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let path = s
                .original_path
                .clone()
                .or_else(|| s.title.clone())
                .unwrap_or_else(|| format!("section-{}", idx + 1));
            let language = s.language.clone().unwrap_or_else(|| "text".to_string());
            let byte_size = s.body.len();
            let token_count = s
                .token_metrics
                .as_ref()
                .map(|m| m.estimated_tokens)
                .unwrap_or_else(|| (s.body.len() / 4).max(1));
            let line_count = s.body.lines().count();
            PreviewFileItem {
                id: s.section_id.clone(),
                path,
                language,
                byte_size,
                token_count,
                line_count,
            }
        })
        .collect();

    let raw_format = query
        .format
        .as_deref()
        .unwrap_or("markdown")
        .trim()
        .to_ascii_lowercase();

    // Render preview content
    let full_content = if let Some(ref target_path) = query.file_path {
        let section = doc.sections.iter().find(|s| {
            s.original_path.as_deref() == Some(target_path)
                || s.section_id == *target_path
                || s.title.as_deref() == Some(target_path)
        });

        match section {
            Some(s) => match raw_format.as_str() {
                "json" => serde_json::to_string_pretty(s).unwrap_or_else(|_| s.body.clone()),
                "xml" => format!(
                    "<document>\n  <source>{}</source>\n  <language>{}</language>\n  <content>\n{}\n  </content>\n</document>",
                    s.original_path.as_deref().unwrap_or(&s.section_id),
                    s.language.as_deref().unwrap_or("text"),
                    s.body
                ),
                _ => s.body.clone(),
            },
            None => render_full_cdm_format(&doc, &raw_format),
        }
    } else {
        render_full_cdm_format(&doc, &raw_format)
    };

    let all_lines: Vec<&str> = full_content.lines().collect();
    let total_lines = all_lines.len();
    let max = query.max_lines.unwrap_or(1000);
    let (content, truncated) = if total_lines > max {
        (all_lines[..max].join("\n"), true)
    } else {
        (full_content, false)
    };

    let total_tokens = doc.total_estimated_tokens();
    let total_bytes = content.len();

    let response = JobPreviewResponse {
        job_id: params.job_id,
        format: raw_format,
        content,
        total_lines,
        total_tokens,
        total_bytes,
        truncated,
        max_lines: max,
        files,
        hierarchy: Some(doc.hierarchy.clone()),
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn render_full_cdm_format(doc: &CdmDocument, format: &str) -> String {
    match format {
        "markdown" | "md" => export_markdown(doc, &MarkdownOptions::default()).unwrap_or_default(),
        "txt" | "text" => export_plain_text(doc, &PlainTextOptions::default()).unwrap_or_default(),
        "json" => export_json(doc, &JsonExportOptions::canonical()).unwrap_or_default(),
        "html" | "htm" => export_html(doc, &HtmlOptions::default())
            .map(|(s, _)| s)
            .unwrap_or_default(),
        "jsonl" | "ndjson" => export_jsonl(doc, &JsonlOptions::default())
            .map(|(s, _)| s)
            .unwrap_or_default(),
        "xml" => {
            let mut out = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<documents source=\"{}\">\n",
                doc.canonical_url
            );
            for (idx, sec) in doc.sections.iter().enumerate() {
                out.push_str(&format!(
                    "  <document index=\"{}\">\n    <source>{}</source>\n    <content>\n{}\n    </content>\n  </document>\n",
                    idx + 1,
                    sec.original_path.as_deref().unwrap_or(&sec.section_id),
                    sec.body
                ));
            }
            out.push_str("</documents>");
            out
        }
        _ => export_markdown(doc, &MarkdownOptions::default()).unwrap_or_default(),
    }
}
