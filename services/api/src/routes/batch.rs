//! Multi-URL batch conversion route handlers.

use axum::body::Body;
use axum::extract::{Extension, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_core::{BatchId, JobId, RequestId};
use web_repo_to_ai_exporters::batch::{
    export_batch_consolidated_json, export_batch_consolidated_jsonl,
    export_batch_consolidated_markdown, export_batch_consolidated_plain_text,
    export_batch_zip_package, BatchExportOptions, BatchItemExportInput,
};
use web_repo_to_ai_storage::batch::{
    BatchItemRecord, BatchItemStatusKind, BatchJobStatus, BatchRecord,
};
use web_repo_to_ai_storage::layout;

use crate::dto::{
    BatchArtifactItemDescriptor, BatchArtifactPathParams, BatchArtifactsResponse,
    BatchItemPathParams, BatchItemStatus, BatchItemsListResponse, BatchJobResponse,
    BatchPathParams, BatchProgress, BatchStatus, BatchSummaryItem, CreateBatchJobRequest,
    CreateBatchJobResponse, JobStatus, ListBatchesResponse, DEFAULT_BATCH_PARALLELISM,
};
use crate::error::{internal_problem, not_found_problem};
use crate::extractors::{ValidatedJson, ValidatedPath};
use crate::state::AppState;

/// Helper to convert a storage [`BatchRecord`] and slice of [`BatchItemRecord`]s into [`BatchJobResponse`].
fn build_batch_job_response(record: BatchRecord, items: Vec<BatchItemRecord>) -> BatchJobResponse {
    let item_statuses: Vec<BatchItemStatus> = items.into_iter().map(Into::into).collect();
    let status = match record.status {
        BatchJobStatus::Queued => BatchStatus::Queued,
        BatchJobStatus::Processing => BatchStatus::Processing,
        BatchJobStatus::Completed => BatchStatus::Completed,
        BatchJobStatus::PartialFailure => BatchStatus::PartialFailure,
        BatchJobStatus::Failed => BatchStatus::Failed,
        BatchJobStatus::Cancelled => BatchStatus::Cancelled,
    };
    let progress = BatchProgress::from_items(&item_statuses);

    BatchJobResponse {
        batch_id: record.id,
        name: record.name,
        status,
        progress,
        max_parallelism: record.max_parallelism,
        items: item_statuses,
        created_at: record.created_at,
        updated_at: record.updated_at,
        completed_at: record.completed_at,
    }
}

/// Handles `POST /api/v1/jobs/batch`.
///
/// Accepts a validated multi-URL batch request, initializes a strongly-typed [`BatchId`],
/// allocates strongly-typed UUID v7 [`JobId`]s for each distinct URL member,
/// persists initial batch and item records to storage, records submission metrics,
/// and returns `202 Accepted` with initial batch descriptor.
pub async fn create_batch_job_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateBatchJobRequest>,
) -> impl IntoResponse {
    let batch_id = BatchId::new();
    let effective_items = payload.effective_items();
    let now = Utc::now();
    let total_items = effective_items.len();
    let max_parallelism = payload.max_parallelism.unwrap_or(DEFAULT_BATCH_PARALLELISM);

    let mut item_records = Vec::with_capacity(total_items);
    let mut item_statuses = Vec::with_capacity(total_items);

    for (idx, item) in effective_items.iter().enumerate() {
        let job_id = JobId::new();
        item_records.push(BatchItemRecord {
            batch_id,
            item_index: idx,
            job_id,
            url: item.url.clone(),
            custom_id: item.custom_id.clone(),
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        });

        item_statuses.push(BatchItemStatus {
            item_index: idx,
            job_id,
            url: item.url.clone(),
            custom_id: item.custom_id.clone(),
            status: JobStatus::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            created_at: now,
            completed_at: None,
        });
    }

    let batch_record = BatchRecord {
        id: batch_id,
        name: payload.name.clone(),
        status: BatchJobStatus::Queued,
        total_items,
        queued_items: total_items,
        processing_items: 0,
        completed_items: 0,
        failed_items: 0,
        max_parallelism,
        owner_session_id: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
    };

    if let Err(e) = state
        .batch_storage
        .create_batch(batch_record, item_records)
        .await
    {
        tracing::error!("Failed to persist batch {batch_id}: {e}");
    }

    let response = CreateBatchJobResponse {
        batch_id,
        name: payload.name,
        status: BatchStatus::Queued,
        total_items,
        queued_items: total_items,
        max_parallelism,
        items: item_statuses,
        created_at: now,
    };

    state.metrics.jobs_total.inc("batch_submitted");

    (StatusCode::ACCEPTED, Json(response))
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}`.
///
/// Retrieves multi-URL batch job execution progress and itemized statuses.
pub async fn get_batch_job_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    let batch = match state.batch_storage.get_batch(params.batch_id).await {
        Ok(Some(b)) => b,
        Ok(None) => {
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to query batch {}: {e}", params.batch_id);
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
    };

    let items = state
        .batch_storage
        .get_batch_items(params.batch_id)
        .await
        .unwrap_or_default();

    let response = build_batch_job_response(batch, items);
    (StatusCode::OK, Json(response)).into_response()
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}/items`.
///
/// Retrieves all individual item statuses for a given batch conversion job.
pub async fn get_batch_items_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    let exists = state.batch_storage.get_batch(params.batch_id).await;
    match exists {
        Ok(Some(_)) => {
            let items = state
                .batch_storage
                .get_batch_items(params.batch_id)
                .await
                .unwrap_or_default();
            let total_items = items.len();
            let item_statuses = items.into_iter().map(Into::into).collect();

            (
                StatusCode::OK,
                Json(BatchItemsListResponse {
                    batch_id: params.batch_id,
                    total_items,
                    items: item_statuses,
                }),
            )
                .into_response()
        }
        _ => not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
            .into_response(),
    }
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}/items/{item_index}`.
///
/// Retrieves detailed execution status of a single URL item within a batch.
pub async fn get_batch_item_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchItemPathParams>,
) -> impl IntoResponse {
    match state
        .batch_storage
        .get_batch_item(params.batch_id, params.item_index)
        .await
    {
        Ok(Some(item)) => {
            let status: BatchItemStatus = item.into();
            (StatusCode::OK, Json(status)).into_response()
        }
        _ => not_found_problem(
            "BatchItem",
            &format!("{}:{}", params.batch_id, params.item_index),
            Some(request_id),
        )
        .into_response(),
    }
}

/// Handles `DELETE /api/v1/jobs/batch/{batch_id}` and `POST /api/v1/jobs/batch/{batch_id}/cancel`.
///
/// Cancels all remaining queued and processing items in a batch.
pub async fn cancel_batch_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    match state.batch_storage.cancel_batch(params.batch_id).await {
        Ok(Some(cancelled_batch)) => {
            let items = state
                .batch_storage
                .get_batch_items(params.batch_id)
                .await
                .unwrap_or_default();
            let response = build_batch_job_response(cancelled_batch, items);
            (StatusCode::OK, Json(response)).into_response()
        }
        _ => not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
            .into_response(),
    }
}

/// Handles `GET /api/v1/jobs/batch`.
///
/// Lists multi-URL batch conversion jobs.
pub async fn list_batches_handler(State(state): State<AppState>) -> impl IntoResponse {
    let list = state
        .batch_storage
        .list_batches(None, 50, 0)
        .await
        .unwrap_or_default();

    let count = list.len();
    let batches = list.into_iter().map(BatchSummaryItem::from).collect();

    (StatusCode::OK, Json(ListBatchesResponse { batches, count })).into_response()
}

/// Helper building a standardized binary/text download attachment response.
fn build_batch_download_response(
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

/// Handles `GET /api/v1/jobs/batch/{batch_id}/export/{format}`.
///
/// Exports all items in a batch into the requested consolidated distribution format.
pub async fn export_batch_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchArtifactPathParams>,
) -> Response {
    let batch = match state.batch_storage.get_batch(params.batch_id).await {
        Ok(Some(b)) => b,
        _ => {
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
    };

    let items = state
        .batch_storage
        .get_batch_items(params.batch_id)
        .await
        .unwrap_or_default();

    let mut export_inputs = Vec::with_capacity(items.len());
    for item in items {
        let job_uuid = *item.job_id.as_uuid();
        let cdm_keys = [
            layout::job_artifact_key(
                job_uuid,
                web_repo_to_ai_storage::schema::ArtifactFormat::CdmJson,
            ),
            format!("jobs/{job_uuid}/artifact.cdm.json"),
            format!("jobs/{job_uuid}/cdm.json"),
        ];

        let mut doc = None;
        for key in &cdm_keys {
            if let Ok(bytes) = state.storage.get(key).await {
                if let Ok(parsed) = serde_json::from_slice::<CdmDocument>(&bytes) {
                    doc = Some(parsed);
                    break;
                }
            }
        }

        if let Some(cdm) = doc {
            export_inputs.push(BatchItemExportInput::success(
                item.item_index,
                item.job_id,
                item.url,
                item.custom_id,
                cdm,
            ));
        } else {
            let err = item.error_message.unwrap_or_else(|| {
                if item.status == BatchItemStatusKind::Completed {
                    "Artifact CDM document missing from storage".to_string()
                } else {
                    format!("Item status is {}", item.status.as_str())
                }
            });
            export_inputs.push(BatchItemExportInput::failure(
                item.item_index,
                item.job_id,
                item.url,
                item.custom_id,
                err,
            ));
        }
    }

    let raw_format = params.format.trim().to_ascii_lowercase();
    let options = BatchExportOptions {
        title: batch.name,
        ..Default::default()
    };
    let batch_id_str = params.batch_id.to_string();

    let (bytes, filename, mime_type) = match raw_format.as_str() {
        "zip" => match export_batch_zip_package(&batch_id_str, &export_inputs, &options) {
            Ok((b, _)) => (b, format!("batch-{batch_id_str}.zip"), "application/zip"),
            Err(e) => {
                return internal_problem(&e.to_string(), Some(request_id)).into_response();
            }
        },
        "markdown" | "md" => {
            match export_batch_consolidated_markdown(&batch_id_str, &export_inputs, &options) {
                Ok(s) => (
                    s.into_bytes(),
                    format!("batch-{batch_id_str}.md"),
                    "text/markdown; charset=utf-8",
                ),
                Err(e) => {
                    return internal_problem(&e.to_string(), Some(request_id)).into_response();
                }
            }
        }
        "jsonl" | "ndjson" => {
            match export_batch_consolidated_jsonl(&batch_id_str, &export_inputs, &options) {
                Ok(s) => (
                    s.into_bytes(),
                    format!("batch-{batch_id_str}.jsonl"),
                    "application/x-ndjson",
                ),
                Err(e) => {
                    return internal_problem(&e.to_string(), Some(request_id)).into_response();
                }
            }
        }
        "json" => match export_batch_consolidated_json(&batch_id_str, &export_inputs, &options) {
            Ok(s) => (
                s.into_bytes(),
                format!("batch-{batch_id_str}.json"),
                "application/json",
            ),
            Err(e) => {
                return internal_problem(&e.to_string(), Some(request_id)).into_response();
            }
        },
        "txt" | "text" => {
            match export_batch_consolidated_plain_text(&batch_id_str, &export_inputs, &options) {
                Ok(s) => (
                    s.into_bytes(),
                    format!("batch-{batch_id_str}.txt"),
                    "text/plain; charset=utf-8",
                ),
                Err(e) => {
                    return internal_problem(&e.to_string(), Some(request_id)).into_response();
                }
            }
        }
        _ => {
            return not_found_problem(
                "ArtifactFormat",
                &format!("{}:{}", params.batch_id, params.format),
                Some(request_id),
            )
            .into_response();
        }
    };

    let len = bytes.len() as u64;
    let body = Body::from(bytes);
    build_batch_download_response(body, mime_type, &filename, Some(len), &request_id)
}

/// Handles `GET /api/v1/jobs/batch/{batch_id}/artifacts`.
///
/// Lists all available downloadable export formats for the batch.
pub async fn list_batch_artifacts_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    ValidatedPath(params): ValidatedPath<BatchPathParams>,
) -> impl IntoResponse {
    let batch = match state.batch_storage.get_batch(params.batch_id).await {
        Ok(Some(b)) => b,
        _ => {
            return not_found_problem("BatchJob", &params.batch_id.to_string(), Some(request_id))
                .into_response();
        }
    };

    let status = match batch.status {
        BatchJobStatus::Queued => BatchStatus::Queued,
        BatchJobStatus::Processing => BatchStatus::Processing,
        BatchJobStatus::Completed => BatchStatus::Completed,
        BatchJobStatus::PartialFailure => BatchStatus::PartialFailure,
        BatchJobStatus::Failed => BatchStatus::Failed,
        BatchJobStatus::Cancelled => BatchStatus::Cancelled,
    };

    let base = format!("/api/v1/jobs/batch/{}", params.batch_id);
    let id_str = params.batch_id.to_string();

    let artifacts = vec![
        BatchArtifactItemDescriptor {
            format: "zip".to_string(),
            filename: format!("batch-{id_str}.zip"),
            mime_type: "application/zip".to_string(),
            download_url: format!("{base}/export/zip"),
        },
        BatchArtifactItemDescriptor {
            format: "markdown".to_string(),
            filename: format!("batch-{id_str}.md"),
            mime_type: "text/markdown; charset=utf-8".to_string(),
            download_url: format!("{base}/export/markdown"),
        },
        BatchArtifactItemDescriptor {
            format: "jsonl".to_string(),
            filename: format!("batch-{id_str}.jsonl"),
            mime_type: "application/x-ndjson".to_string(),
            download_url: format!("{base}/export/jsonl"),
        },
        BatchArtifactItemDescriptor {
            format: "json".to_string(),
            filename: format!("batch-{id_str}.json"),
            mime_type: "application/json".to_string(),
            download_url: format!("{base}/export/json"),
        },
        BatchArtifactItemDescriptor {
            format: "txt".to_string(),
            filename: format!("batch-{id_str}.txt"),
            mime_type: "text/plain; charset=utf-8".to_string(),
            download_url: format!("{base}/export/txt"),
        },
    ];

    (
        StatusCode::OK,
        Json(BatchArtifactsResponse {
            batch_id: params.batch_id,
            status,
            artifacts,
        }),
    )
        .into_response()
}
