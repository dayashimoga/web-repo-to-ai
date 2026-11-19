//! Comprehensive end-to-end API integration tests for `web-repo-to-ai-api`.
//!
//! Exercises single-job lifecycle (`/jobs`), preview generation (`/jobs/:id/preview`),
//! SSE event streaming (`/jobs/:id/events`, `/jobs/:id/stream`), multi-format artifact downloads
//! (`/jobs/:id/artifacts/:format`), multi-URL batch workflows (`/jobs/batch`), batch cancellation,
//! source detection (`/sources/detect`), telemetry & health probes (`/health`, `/ready`, `/metrics`),
//! and cross-tenant isolation enforcement.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;
use web_repo_to_ai_api::build_app;
use web_repo_to_ai_api::dto::{
    BatchArtifactsResponse, BatchItemsListResponse, BatchJobResponse, BatchStatus,
    CreateBatchJobResponse, DetectSourcesResponse, JobPreviewResponse, JobResponse, JobStatus,
    ListBatchesResponse, SourceClassification,
};
use web_repo_to_ai_api::state::AppState;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{CdmDocument, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_core::AppConfig;
use web_repo_to_ai_storage::batch::BatchItemStatusKind;
use web_repo_to_ai_storage::layout;
use web_repo_to_ai_storage::schema::ArtifactFormat;

fn setup_test_app() -> (axum::Router, AppState) {
    let config = AppConfig::default();
    let state = AppState::new(config);
    let app = build_app(state.clone());
    (app, state)
}

async fn body_to_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn body_to_bytes(body: Body) -> Vec<u8> {
    body.collect().await.unwrap().to_bytes().to_vec()
}

async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn create_test_cdm(raw_url: &str) -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: raw_url.to_string(),
        canonical_url: raw_url.to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("integration-test".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let sec1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/integration-test/blob/main/src/main.rs",
        SectionKind::CodeFile,
        "fn main() {\n    println!(\"Hello Integration Tests!\");\n}\n",
    )
    .title("src/main.rs")
    .original_path("src/main.rs")
    .language("rust")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/integration-test/blob/main/README.md",
        SectionKind::Prose,
        "# Integration Test Project\n\nComprehensive end-to-end API verification suite.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .build();

    CdmDocumentBuilder::new(identity, SourceType::GitHubPublic, raw_url)
        .add_section(sec1)
        .add_section(sec2)
        .build()
}

#[tokio::test]
async fn test_e2e_single_job_lifecycle_preview_events_and_artifacts() {
    let (app, state) = setup_test_app();

    // 1. Submit Single Job via POST /api/v1/jobs
    let create_payload = json!({
        "sources": ["https://github.com/example/integration-test"],
        "export_formats": ["markdown"],
        "token_budget": 80000
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/jobs")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    let job_resp: JobResponse =
        serde_json::from_str(&body_to_string(res.into_body()).await).unwrap();
    assert_eq!(job_resp.status, JobStatus::Queued);
    assert_eq!(
        job_resp.sources,
        vec!["https://github.com/example/integration-test"]
    );
    let job_id = job_resp.job_id;

    // 2. Query Preview before CDM is available -> 404 Problem Details
    let preview_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/{job_id}/preview"))
        .body(Body::empty())
        .unwrap();
    let preview_res = app.clone().oneshot(preview_req).await.unwrap();
    assert_eq!(preview_res.status(), StatusCode::NOT_FOUND);

    // 3. Seed CDM Document in Storage (simulating extraction worker completion)
    let cdm_doc = create_test_cdm("https://github.com/example/integration-test");
    let cdm_bytes = serde_json::to_vec_pretty(&cdm_doc).unwrap();
    let cdm_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::CdmJson);
    state.storage.put(&cdm_key, &cdm_bytes).await.unwrap();

    // 4. Query Preview with default markdown format -> 200 OK
    let preview_req2 = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/{job_id}/preview"))
        .body(Body::empty())
        .unwrap();
    let preview_res2 = app.clone().oneshot(preview_req2).await.unwrap();
    assert_eq!(preview_res2.status(), StatusCode::OK);

    let preview_body: JobPreviewResponse =
        serde_json::from_str(&body_to_string(preview_res2.into_body()).await).unwrap();
    assert_eq!(preview_body.job_id, job_id);
    assert_eq!(preview_body.format, "markdown");
    assert_eq!(preview_body.files.len(), 2);
    assert!(preview_body.content.contains("Hello Integration Tests!"));
    assert!(preview_body.content.contains("Integration Test Project"));

    // 5. Query Preview with JSON format override and file filtering
    let preview_json_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/api/v1/jobs/{job_id}/preview?format=json&file_path=src/main.rs"
        ))
        .body(Body::empty())
        .unwrap();
    let preview_json_res = app.clone().oneshot(preview_json_req).await.unwrap();
    assert_eq!(preview_json_res.status(), StatusCode::OK);
    let preview_json_body = body_to_string(preview_json_res.into_body()).await;
    assert!(preview_json_body.contains("src/main.rs"));

    // 6. Query SSE Events Stream (/events and /stream alias)
    for endpoint in &["events", "stream"] {
        let sse_req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/jobs/{job_id}/{endpoint}"))
            .body(Body::empty())
            .unwrap();
        let sse_res = app.clone().oneshot(sse_req).await.unwrap();
        assert_eq!(sse_res.status(), StatusCode::OK);

        let ct = sse_res
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("text/event-stream"));

        let sse_body = body_to_string(sse_res.into_body()).await;
        assert!(sse_body.contains("event: phase"));
        assert!(sse_body.contains("event: completeness"));
        assert!(sse_body.contains("event: tokens"));
        assert!(sse_body.contains("event: artifact"));
        assert!(sse_body.contains("event: done"));
    }

    // 7. Verify Artifact Streaming across all supported formats
    let formats = [
        ("markdown", "text/markdown; charset=utf-8"),
        ("txt", "text/plain; charset=utf-8"),
        ("json", "application/json"),
        ("html", "text/html; charset=utf-8"),
        ("jsonl", "application/x-ndjson"),
        ("zip", "application/zip"),
        ("manifest", "application/json"),
        ("checksums", "text/plain; charset=utf-8"),
    ];

    for (fmt, expected_mime) in formats {
        let art_req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/jobs/{job_id}/artifacts/{fmt}"))
            .body(Body::empty())
            .unwrap();
        let art_res = app.clone().oneshot(art_req).await.unwrap();
        assert_eq!(art_res.status(), StatusCode::OK, "Format '{fmt}' failed");

        let content_type = art_res
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            content_type.contains(expected_mime),
            "Expected MIME '{expected_mime}' for format '{fmt}', got '{content_type}'"
        );

        let bytes = body_to_bytes(art_res.into_body()).await;
        if fmt != "checksums" {
            assert!(!bytes.is_empty(), "Artifact for '{fmt}' must not be empty");
        }

        if fmt == "zip" {
            // Validate ZIP archive signature (PK\x03\x04)
            assert!(bytes.len() >= 4);
            assert_eq!(&bytes[0..4], b"PK\x03\x04");
        }
    }
}

#[tokio::test]
async fn test_e2e_batch_job_submission_tracking_export_and_cancellation() {
    let (app, state) = setup_test_app();

    // 1. Submit Batch via POST /api/v1/jobs/batch
    let batch_req_body = json!({
        "name": "E2E Integration Test Batch",
        "urls": [
            "https://example.com/page-1",
            "https://example.com/page-2",
            "https://github.com/example/repo-3"
        ],
        "max_parallelism": 3,
        "token_budget": 50000
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/jobs/batch")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(batch_req_body.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    let batch_create_resp: CreateBatchJobResponse =
        serde_json::from_str(&body_to_string(res.into_body()).await).unwrap();
    assert_eq!(batch_create_resp.total_items, 3);
    assert_eq!(batch_create_resp.status, BatchStatus::Queued);
    let batch_id = batch_create_resp.batch_id;

    // 2. Query Batch Status via GET /api/v1/jobs/batch/{batch_id}
    let get_batch_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/batch/{batch_id}"))
        .body(Body::empty())
        .unwrap();
    let get_batch_res = app.clone().oneshot(get_batch_req).await.unwrap();
    assert_eq!(get_batch_res.status(), StatusCode::OK);

    let batch_info: BatchJobResponse =
        serde_json::from_str(&body_to_string(get_batch_res.into_body()).await).unwrap();
    assert_eq!(batch_info.batch_id, batch_id);
    assert_eq!(
        batch_info.name.as_deref(),
        Some("E2E Integration Test Batch")
    );
    assert_eq!(batch_info.progress.total_items, 3);
    assert_eq!(batch_info.progress.completed_items, 0);

    // 3. Query Batch Items via GET /api/v1/jobs/batch/{batch_id}/items
    let items_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/batch/{batch_id}/items"))
        .body(Body::empty())
        .unwrap();
    let items_res = app.clone().oneshot(items_req).await.unwrap();
    assert_eq!(items_res.status(), StatusCode::OK);

    let items_list: BatchItemsListResponse =
        serde_json::from_str(&body_to_string(items_res.into_body()).await).unwrap();
    assert_eq!(items_list.total_items, 3);
    assert_eq!(items_list.items.len(), 3);

    // 4. Query Individual Batch Item via GET /api/v1/jobs/batch/{batch_id}/items/{index}
    let item_0_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/batch/{batch_id}/items/0"))
        .body(Body::empty())
        .unwrap();
    let item_0_res = app.clone().oneshot(item_0_req).await.unwrap();
    assert_eq!(item_0_res.status(), StatusCode::OK);

    // 5. Complete Item 0 and seed its CDM in storage
    let item_0_job_id = items_list.items[0].job_id;
    state
        .batch_storage
        .update_item_status(
            batch_id,
            0,
            BatchItemStatusKind::Completed,
            None,
            None,
            Some(120),
        )
        .await
        .unwrap();

    let cdm_item_0 = create_test_cdm("https://example.com/page-1");
    let cdm_bytes = serde_json::to_vec_pretty(&cdm_item_0).unwrap();
    let cdm_key = layout::job_artifact_key(*item_0_job_id.as_uuid(), ArtifactFormat::CdmJson);
    state.storage.put(&cdm_key, &cdm_bytes).await.unwrap();

    // 6. Test Batch Multi-Format Exports
    let batch_formats = ["markdown", "json", "jsonl", "txt", "zip"];
    for fmt in batch_formats {
        let export_req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/jobs/batch/{batch_id}/export/{fmt}"))
            .body(Body::empty())
            .unwrap();
        let export_res = app.clone().oneshot(export_req).await.unwrap();
        assert_eq!(
            export_res.status(),
            StatusCode::OK,
            "Batch export failed for {fmt}"
        );

        let export_bytes = body_to_bytes(export_res.into_body()).await;
        assert!(
            !export_bytes.is_empty(),
            "Batch export output empty for {fmt}"
        );
    }

    // 7. Test Batch Artifacts Listing
    let artifacts_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/batch/{batch_id}/artifacts"))
        .body(Body::empty())
        .unwrap();
    let artifacts_res = app.clone().oneshot(artifacts_req).await.unwrap();
    assert_eq!(artifacts_res.status(), StatusCode::OK);

    let artifacts_list: BatchArtifactsResponse =
        serde_json::from_str(&body_to_string(artifacts_res.into_body()).await).unwrap();
    assert_eq!(artifacts_list.batch_id, batch_id);
    assert_eq!(artifacts_list.artifacts.len(), 5);

    // 8. Cancel Batch via POST /api/v1/jobs/batch/{batch_id}/cancel
    let cancel_req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1/jobs/batch/{batch_id}/cancel"))
        .body(Body::empty())
        .unwrap();
    let cancel_res = app.clone().oneshot(cancel_req).await.unwrap();
    assert_eq!(cancel_res.status(), StatusCode::OK);

    let cancelled_resp: BatchJobResponse =
        serde_json::from_str(&body_to_string(cancel_res.into_body()).await).unwrap();
    assert_eq!(cancelled_resp.status, BatchStatus::Cancelled);

    // 9. List All Batches via GET /api/v1/jobs/batch
    let list_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/jobs/batch")
        .body(Body::empty())
        .unwrap();
    let list_res = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    let batches: ListBatchesResponse =
        serde_json::from_str(&body_to_string(list_res.into_body()).await).unwrap();
    assert!(batches.batches.iter().any(|b| b.batch_id == batch_id));
}

#[tokio::test]
async fn test_source_detection_and_classification_endpoints() {
    let (app, _) = setup_test_app();

    let detect_payload = json!({
        "urls": [
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "https://github.com/rust-lang/rust",
            "https://github.com/rust-lang/rust/tree/master/src/tools/cargo",
            "https://github.com/rust-lang/rust/issues/12345"
        ]
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(detect_payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let detect_resp: DetectSourcesResponse =
        serde_json::from_str(&body_to_string(res.into_body()).await).unwrap();
    assert_eq!(detect_resp.total, 4);
    assert_eq!(detect_resp.sources.len(), 4);

    // Verify classification details
    assert_eq!(
        detect_resp.sources[0].classification,
        SourceClassification::WebPage
    );
    assert_eq!(
        detect_resp.sources[1].classification,
        SourceClassification::PublicGitHub
    );
    assert_eq!(
        detect_resp.sources[2].classification,
        SourceClassification::PublicGitHub
    );
    assert!(detect_resp.sources[2].repo_owner.is_some());
}

#[tokio::test]
async fn test_system_telemetry_health_and_root_endpoints() {
    let (app, _) = setup_test_app();

    // 1. GET / Root Service Descriptor
    let root_req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let root_res = app.clone().oneshot(root_req).await.unwrap();
    assert_eq!(root_res.status(), StatusCode::OK);
    let root_body: Value = body_to_json(root_res.into_body()).await;
    assert_eq!(root_body["service"], "web-repo-to-ai-api");
    assert_eq!(root_body["base_path"], "/api/v1");

    // 2. GET /api/v1/health Liveness Probe
    let health_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();
    let health_res = app.clone().oneshot(health_req).await.unwrap();
    assert_eq!(health_res.status(), StatusCode::OK);
    let health_body: Value = body_to_json(health_res.into_body()).await;
    assert_eq!(health_body["status"], "pass");
    assert_eq!(health_body["service"], "web-repo-to-ai-api");

    // 3. GET /api/v1/ready Readiness Probe
    let ready_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/ready")
        .body(Body::empty())
        .unwrap();
    let ready_res = app.clone().oneshot(ready_req).await.unwrap();
    // In local ephemeral in-memory configuration without Postgres connected,
    // returns structured status evaluation (200 or 503) with subsystem checks
    let ready_status = ready_res.status();
    assert!(ready_status == StatusCode::OK || ready_status == StatusCode::SERVICE_UNAVAILABLE);
    let ready_body: Value = body_to_json(ready_res.into_body()).await;
    assert!(ready_body["checks"]["storage"].is_object());

    // 4. GET /api/v1/metrics Prometheus Exposition
    let metrics_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/metrics")
        .body(Body::empty())
        .unwrap();
    let metrics_res = app.oneshot(metrics_req).await.unwrap();
    assert_eq!(metrics_res.status(), StatusCode::OK);
    let metrics_text = body_to_string(metrics_res.into_body()).await;
    assert!(metrics_text.contains("web_repo_to_ai_uptime_seconds"));
    assert!(metrics_text.contains("web_repo_to_ai_build_info"));
}

#[tokio::test]
async fn test_tenant_isolation_boundary_anti_enumeration() {
    let (app, state) = setup_test_app();

    let tenant_a_session = Uuid::new_v4();
    let tenant_b_session = Uuid::new_v4();

    // 1. Tenant A creates an API key
    let mut scopes = std::collections::HashSet::new();
    scopes.insert(ApiKeyScope::Read);
    scopes.insert(ApiKeyScope::Write);
    let gen_key =
        GeneratedApiKey::generate(tenant_a_session, "Tenant A Programmatic Key", scopes, None)
            .unwrap();
    state
        .api_key_storage
        .save_key(&gen_key.record)
        .await
        .unwrap();

    // 2. Tenant B queries keys with X-Session-ID -> does NOT see Tenant A's key
    let list_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header("X-Session-ID", tenant_b_session.to_string())
        .body(Body::empty())
        .unwrap();
    let list_res = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);
    let list_json: Value = body_to_json(list_res.into_body()).await;
    assert_eq!(list_json["total"], 0);

    // 3. Tenant B attempts to revoke Tenant A's key -> returns 404 (not 403, anti-enumeration)
    let revoke_req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{}", gen_key.record.id))
        .header("X-Session-ID", tenant_b_session.to_string())
        .body(Body::empty())
        .unwrap();
    let revoke_res = app.oneshot(revoke_req).await.unwrap();
    assert_eq!(revoke_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_rfc7807_error_fidelity_and_validation() {
    let (app, _) = setup_test_app();

    // 1. Malformed UUID in route parameter -> 400 Validation Problem
    let bad_uuid_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/jobs/not-a-valid-uuid/preview")
        .body(Body::empty())
        .unwrap();
    let bad_uuid_res = app.clone().oneshot(bad_uuid_req).await.unwrap();
    assert_eq!(bad_uuid_res.status(), StatusCode::BAD_REQUEST);
    let bad_uuid_json = body_to_json(bad_uuid_res.into_body()).await;
    assert_eq!(bad_uuid_json["error"]["code"], "VALIDATION_ERROR");

    // 2. Unsupported export format in route parameter -> 400 Validation Problem
    let bad_fmt_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/api/v1/jobs/{}/artifacts/unsupported_format",
            Uuid::new_v4()
        ))
        .body(Body::empty())
        .unwrap();
    let bad_fmt_res = app.clone().oneshot(bad_fmt_req).await.unwrap();
    assert_eq!(bad_fmt_res.status(), StatusCode::BAD_REQUEST);
    let bad_fmt_json = body_to_json(bad_fmt_res.into_body()).await;
    assert_eq!(bad_fmt_json["error"]["code"], "VALIDATION_ERROR");

    // 3. Unmatched route -> 404 Not Found Problem Details
    let fallback_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/nonexistent/path/for/testing")
        .body(Body::empty())
        .unwrap();
    let fallback_res = app.oneshot(fallback_req).await.unwrap();
    assert_eq!(fallback_res.status(), StatusCode::NOT_FOUND);
    let fallback_json = body_to_json(fallback_res.into_body()).await;
    assert_eq!(fallback_json["error"]["code"], "RESOURCE_NOT_FOUND");
}
