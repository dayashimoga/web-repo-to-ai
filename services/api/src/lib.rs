//! # Web API Service Library (`web-repo-to-ai-api`)
//!
//! Provides the core HTTP router, middleware stack, Prometheus metrics endpoints,
//! and server lifecycle for the `web-repo-to-ai` REST and SSE API.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod dto;
pub mod error;
pub mod extractors;
pub mod middleware;
pub mod routes;
pub mod server;
pub mod state;

pub use dto::{
    classify_source_url, ArtifactPathParams, CreateJobRequest, DetectSourcesRequest,
    DetectSourcesResponse, DetectedSource, JobPathParams, JobResponse, JobStatus, ListJobsQuery,
    PaginatedResponse, SourceClassification, VALID_EXPORT_FORMATS, VALID_SORT_ORDERS,
    VALID_STATUS_FILTERS,
};
pub use error::*;
pub use extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
pub use server::{build_app, run_server};
pub use state::AppState;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use web_repo_to_ai_core::AppConfig;
    use web_repo_to_ai_storage::{StorageBackend, TokenStorageRepository};

    fn setup_test_app() -> axum::Router {
        let config = AppConfig::default();
        let state = AppState::new(config);
        build_app(state)
    }

    async fn body_to_string(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_root_endpoint_metadata() {
        let app = setup_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("web-repo-to-ai-api"));
        assert!(body.contains("/api/v1"));
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("\"status\":\"pass\""));
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("\"status\":\"ready\""));
        assert!(body.contains("\"checks\""));
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/plain"));
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("web_repo_to_ai_jobs_total"));
    }

    #[tokio::test]
    async fn test_request_id_and_traceparent_injection() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-request-id"));
        assert!(response.headers().contains_key("traceparent"));
    }

    #[tokio::test]
    async fn test_client_request_id_preservation() {
        let app = setup_test_app();
        let custom_id = "019335c4-7c20-7000-8000-000000000042";
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header("x-request-id", custom_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let returned_id = response
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(returned_id, custom_id);
    }

    #[tokio::test]
    async fn test_fallback_404_problem_details() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/unknown-endpoint")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("RESOURCE_NOT_FOUND"));
        assert!(body.contains("/api/v1/unknown-endpoint"));
    }

    #[tokio::test]
    async fn test_cors_options_preflight() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/api/v1/health")
                    .header(header::ORIGIN, "http://localhost:5173")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let allow_origin = response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(allow_origin, "http://localhost:5173");
    }

    #[tokio::test]
    async fn test_detect_sources_endpoint_valid() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "urls": [
                "https://github.com/tokio-rs/tokio",
                "https://docs.rs/axum/latest/axum/"
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("public_github"));
        assert!(body.contains("web_page"));
        assert!(body.contains("\"total\":2"));
    }

    #[tokio::test]
    async fn test_detect_sources_empty_urls_rejected() {
        let app = setup_test_app();
        let payload = serde_json::json!({ "urls": [] });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("urls"));
    }

    #[tokio::test]
    async fn test_detect_sources_invalid_scheme_rejected() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "urls": ["ftp://archive.example.com/file.zip"]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("UNSUPPORTED_URL_SCHEME"));
    }

    #[tokio::test]
    async fn test_detect_sources_malformed_json_rejected() {
        let app = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{ malformed json }"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("INVALID_REQUEST_SCHEMA"));
    }

    #[tokio::test]
    async fn test_create_job_valid() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "sources": ["https://github.com/rust-lang/rust"],
            "git_ref": "master",
            "token_budget": 50000,
            "export_formats": ["markdown", "json"]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/jobs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("\"status\":\"queued\""));
        assert!(body.contains("\"job_id\""));
    }

    #[tokio::test]
    async fn test_create_job_invalid_token_budget_rejected() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "sources": ["https://github.com/rust-lang/rust"],
            "token_budget": 500
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/jobs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("token_budget"));
    }

    #[tokio::test]
    async fn test_create_job_invalid_git_ref_rejected() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "sources": ["https://github.com/rust-lang/rust"],
            "git_ref": "feature/../forbidden"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/jobs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("git_ref"));
    }

    #[tokio::test]
    async fn test_create_job_invalid_export_format_rejected() {
        let app = setup_test_app();
        let payload = serde_json::json!({
            "sources": ["https://github.com/rust-lang/rust"],
            "export_formats": ["markdown", "tar.gz", "exe"]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/jobs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("export_formats"));
    }

    #[tokio::test]
    async fn test_list_jobs_valid_query() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs?page=2&limit=50&status=completed&sort_order=asc")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("\"page\":2"));
        assert!(body.contains("\"limit\":50"));
    }

    #[tokio::test]
    async fn test_list_jobs_invalid_limit_rejected() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs?limit=500")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("limit"));
    }

    #[tokio::test]
    async fn test_list_jobs_invalid_page_rejected() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs?page=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("page"));
    }

    #[tokio::test]
    async fn test_get_job_invalid_path_uuid() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs/not-a-valid-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
    }

    #[tokio::test]
    async fn test_get_job_valid_path_not_found() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs/019335c4-7c20-7000-8000-000000000042")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("RESOURCE_NOT_FOUND"));
    }

    #[tokio::test]
    async fn test_get_artifact_invalid_format_rejected() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs/019335c4-7c20-7000-8000-000000000042/artifacts/bad_format")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("format"));
    }

    #[tokio::test]
    async fn test_get_job_events_sse_stream() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/jobs/019335c4-7c20-7000-8000-000000000001/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/event-stream"));
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("event: phase"));
        assert!(body.contains("event: log"));
    }

    #[tokio::test]
    async fn test_rfc7807_content_type_header() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/nonexistent-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "application/problem+json");
    }

    #[tokio::test]
    async fn test_rfc7807_request_id_correlation_integrity() {
        let app = setup_test_app();
        let custom_id = "019335c4-7c20-7000-8000-000000000099";

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header("x-request-id", custom_id)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{ malformed json }"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Header check
        let header_id = response
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(header_id, custom_id);

        // Body check
        let body_str = body_to_string(response.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(v["error"]["code"], "INVALID_REQUEST_SCHEMA");
        assert_eq!(v["error"]["request_id"], custom_id);
    }

    #[tokio::test]
    async fn test_payload_too_large_rejection() {
        let app = setup_test_app();
        // Create an oversized body exceeding 10 MB limit (10 MB + 1 KB)
        let oversized = vec![b'a'; 10 * 1024 * 1024 + 1024];

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/sources/detect")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(oversized))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "application/problem+json");
        let body_str = body_to_string(response.into_body()).await;
        assert!(body_str.contains("PAYLOAD_TOO_LARGE"));
    }

    #[tokio::test]
    async fn test_traceparent_propagation_on_error() {
        let app = setup_test_app();
        let custom_trace = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/unknown")
                    .header("traceparent", custom_trace)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let resp_trace = response
            .headers()
            .get("traceparent")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(resp_trace.starts_with("00-4bf92f3577b34da6a3ce929d0e0e4736-"));
    }

    #[tokio::test]
    async fn test_health_liveness_structured_fields() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["status"], "pass");
        assert_eq!(v["service"], "web-repo-to-ai-api");
        assert!(v.get("uptime_seconds").is_some());
        assert!(v.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_readiness_subsystem_checks_structure() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["status"], "ready");
        assert_eq!(v["checks"]["storage"]["status"], "pass");
        assert_eq!(v["checks"]["database"]["status"], "pass");
        assert_eq!(v["checks"]["workers"]["status"], "pass");
    }

    #[tokio::test]
    async fn test_readiness_unconfigured_database_failure() {
        let mut config = AppConfig::default();
        config.database.url = String::new(); // Empty DB URL
        let state = AppState::new(config);
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = body_to_string(response.into_body()).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["status"], "not_ready");
        assert_eq!(v["checks"]["database"]["status"], "fail");
    }

    #[tokio::test]
    async fn test_metrics_prometheus_exposition_content() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("web_repo_to_ai_uptime_seconds"));
        assert!(body.contains("web_repo_to_ai_build_info"));
        assert!(body.contains("web_repo_to_ai_http_requests_total"));
        assert!(body.contains("web_repo_to_ai_http_request_duration_ms"));
    }

    fn create_test_cdm() -> web_repo_to_ai_cdm::model::CdmDocument {
        use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
        use web_repo_to_ai_cdm::model::{SourceIdentity, SourceType};
        use web_repo_to_ai_cdm::section::SectionKind;

        let identity = SourceIdentity {
            raw_input: "https://github.com/example/stream-test".to_string(),
            canonical_url: "https://github.com/example/stream-test".to_string(),
            domain: Some("github.com".to_string()),
            repo_owner: Some("example".to_string()),
            repo_name: Some("stream-test".to_string()),
            git_ref: Some("main".to_string()),
            subpath: None,
        };

        let sec = CdmSectionBuilder::new(
            "sec-1",
            "https://github.com/example/stream-test/blob/main/src/main.rs",
            SectionKind::CodeFile,
            "fn main() {\n    println!(\"Hello Stream!\");\n}\n",
        )
        .title("src/main.rs")
        .original_path("src/main.rs")
        .language("rust")
        .build();

        CdmDocumentBuilder::new(
            identity,
            SourceType::GitHubPublic,
            "https://github.com/example/stream-test",
        )
        .add_section(sec)
        .build()
    }

    #[tokio::test]
    async fn test_get_artifact_streaming_markdown_success() {
        use web_repo_to_ai_storage::layout;
        use web_repo_to_ai_storage::schema::ArtifactFormat;
        use web_repo_to_ai_storage::InMemoryStorage;

        let storage = std::sync::Arc::new(InMemoryStorage::new());
        let job_id = web_repo_to_ai_core::JobId::new();
        let key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::Markdown);
        let content = b"# Markdown Content\n\nStreamed successfully!";
        storage.put(&key, content).await.unwrap();

        let config = AppConfig::default();
        let state = AppState::with_storage(config, storage);
        let app = build_app(state);

        let uri = format!("/api/v1/jobs/{job_id}/artifacts/markdown");
        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/markdown; charset=utf-8"
        );
        let disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(disposition.contains(&format!("web-repo-to-ai-{job_id}.md")));
        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        assert_eq!(
            response.headers().get(header::CONTENT_LENGTH).unwrap(),
            &content.len().to_string()
        );

        let body = body_to_string(response.into_body()).await;
        assert_eq!(body, String::from_utf8(content.to_vec()).unwrap());
    }

    #[tokio::test]
    async fn test_get_artifact_streaming_zip_success() {
        use web_repo_to_ai_storage::layout;
        use web_repo_to_ai_storage::schema::ArtifactFormat;
        use web_repo_to_ai_storage::InMemoryStorage;

        let storage = std::sync::Arc::new(InMemoryStorage::new());
        let job_id = web_repo_to_ai_core::JobId::new();
        let key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::Zip);
        let zip_bytes = b"PK\x03\x04mock_zip_archive_data";
        storage.put(&key, zip_bytes).await.unwrap();

        let config = AppConfig::default();
        let state = AppState::with_storage(config, storage);
        let app = build_app(state);

        let uri = format!("/api/v1/jobs/{job_id}/artifacts/zip");
        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/zip"
        );
        let disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(disposition.contains(&format!("web-repo-to-ai-{job_id}.zip")));

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), zip_bytes);
    }

    #[tokio::test]
    async fn test_get_artifact_streaming_manifest_and_checksums() {
        use web_repo_to_ai_storage::layout;
        use web_repo_to_ai_storage::schema::ArtifactFormat;
        use web_repo_to_ai_storage::InMemoryStorage;

        let storage = std::sync::Arc::new(InMemoryStorage::new());
        let job_id = web_repo_to_ai_core::JobId::new();
        let m_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::Manifest);
        storage
            .put(&m_key, b"{\"manifest_version\":\"1.0.0\"}")
            .await
            .unwrap();

        let cs_key = format!("jobs/{}/checksums.sha256", job_id.as_uuid());
        storage.put(&cs_key, b"e3b0c442  file.txt\n").await.unwrap();

        let config = AppConfig::default();
        let state = AppState::with_storage(config, storage);
        let app = build_app(state);

        // Manifest
        let uri_m = format!("/api/v1/jobs/{job_id}/artifacts/manifest");
        let res_m = app
            .clone()
            .oneshot(Request::builder().uri(&uri_m).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res_m.status(), StatusCode::OK);
        assert_eq!(
            res_m.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        // Checksums
        let uri_cs = format!("/api/v1/jobs/{job_id}/artifacts/checksums");
        let res_cs = app
            .oneshot(Request::builder().uri(&uri_cs).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res_cs.status(), StatusCode::OK);
        assert_eq!(
            res_cs.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );
        let body_cs = body_to_string(res_cs.into_body()).await;
        assert_eq!(body_cs, "e3b0c442  file.txt\n");
    }

    #[tokio::test]
    async fn test_get_artifact_on_demand_rendering_from_cdm() {
        use web_repo_to_ai_storage::layout;
        use web_repo_to_ai_storage::schema::ArtifactFormat;
        use web_repo_to_ai_storage::InMemoryStorage;

        let storage = std::sync::Arc::new(InMemoryStorage::new());
        let job_id = web_repo_to_ai_core::JobId::new();
        let cdm_doc = create_test_cdm();
        let cdm_bytes = serde_json::to_vec(&cdm_doc).unwrap();

        let cdm_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::CdmJson);
        storage.put(&cdm_key, &cdm_bytes).await.unwrap();

        let config = AppConfig::default();
        let state = AppState::with_storage(config, storage.clone());
        let app = build_app(state);

        // Request Markdown (not pre-rendered in storage)
        let uri_md = format!("/api/v1/jobs/{job_id}/artifacts/markdown");
        let res_md = app
            .clone()
            .oneshot(Request::builder().uri(&uri_md).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res_md.status(), StatusCode::OK);
        let body_md = body_to_string(res_md.into_body()).await;
        assert!(body_md.contains("Hello Stream!"));

        // Check that on-demand generation cached the markdown artifact into storage
        let cached_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::Markdown);
        assert!(storage.exists(&cached_key).await.unwrap());

        // Request HTML on demand
        let uri_html = format!("/api/v1/jobs/{job_id}/artifacts/html");
        let res_html = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri_html)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_html.status(), StatusCode::OK);
        assert_eq!(
            res_html.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        // Request JSON on demand
        let uri_json = format!("/api/v1/jobs/{job_id}/artifacts/json");
        let res_json = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri_json)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_json.status(), StatusCode::OK);
        assert_eq!(
            res_json.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        // Request JSONL on demand
        let uri_jsonl = format!("/api/v1/jobs/{job_id}/artifacts/jsonl");
        let res_jsonl = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri_jsonl)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_jsonl.status(), StatusCode::OK);
        assert_eq!(
            res_jsonl.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/x-ndjson"
        );

        // Request ZIP on demand
        let uri_zip = format!("/api/v1/jobs/{job_id}/artifacts/zip");
        let res_zip = app
            .oneshot(
                Request::builder()
                    .uri(&uri_zip)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_zip.status(), StatusCode::OK);
        assert_eq!(
            res_zip.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/zip"
        );
    }

    #[tokio::test]
    async fn test_get_preview_not_found() {
        let app = setup_test_app();
        let job_id = web_repo_to_ai_core::JobId::new();
        let uri = format!("/api/v1/jobs/{job_id}/preview");

        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("RESOURCE_NOT_FOUND"));
    }

    #[tokio::test]
    async fn test_get_preview_invalid_format_rejected() {
        let app = setup_test_app();
        let job_id = web_repo_to_ai_core::JobId::new();
        let uri = format!("/api/v1/jobs/{job_id}/preview?format=invalid_fmt");

        let response = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("format"));
    }

    #[tokio::test]
    async fn test_get_preview_success_from_cdm() {
        use web_repo_to_ai_storage::layout;
        use web_repo_to_ai_storage::schema::ArtifactFormat;
        use web_repo_to_ai_storage::InMemoryStorage;

        let storage = std::sync::Arc::new(InMemoryStorage::new());
        let job_id = web_repo_to_ai_core::JobId::new();
        let cdm_doc = create_test_cdm();
        let cdm_bytes = serde_json::to_vec(&cdm_doc).unwrap();

        let cdm_key = layout::job_artifact_key(*job_id.as_uuid(), ArtifactFormat::CdmJson);
        storage.put(&cdm_key, &cdm_bytes).await.unwrap();

        let config = AppConfig::default();
        let state = AppState::with_storage(config, storage);
        let app = build_app(state);

        // 1. Default preview (markdown)
        let uri = format!("/api/v1/jobs/{job_id}/preview");
        let res = app
            .clone()
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = body_to_string(res.into_body()).await;
        let preview: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert_eq!(preview["job_id"], job_id.to_string());
        assert_eq!(preview["format"], "markdown");
        assert_eq!(preview["truncated"], false);
        assert!(preview["content"]
            .as_str()
            .unwrap()
            .contains("Hello Stream!"));
        assert_eq!(preview["files"].as_array().unwrap().len(), 1);
        assert_eq!(preview["files"][0]["path"], "src/main.rs");
        assert_eq!(preview["files"][0]["language"], "rust");

        // 2. Truncated preview with max_lines = 1
        let uri_trunc = format!("/api/v1/jobs/{job_id}/preview?max_lines=1");
        let res_trunc = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri_trunc)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res_trunc.status(), StatusCode::OK);
        let body_trunc = body_to_string(res_trunc.into_body()).await;
        let preview_trunc: serde_json::Value = serde_json::from_str(&body_trunc).unwrap();
        assert_eq!(preview_trunc["truncated"], true);
        assert_eq!(preview_trunc["max_lines"], 1);

        // 3. JSON format preview
        let uri_json = format!("/api/v1/jobs/{job_id}/preview?format=json");
        let res_json = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri_json)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res_json.status(), StatusCode::OK);
        let body_json = body_to_string(res_json.into_body()).await;
        let preview_json: serde_json::Value = serde_json::from_str(&body_json).unwrap();
        assert_eq!(preview_json["format"], "json");

        // 4. File-specific preview
        let uri_file = format!("/api/v1/jobs/{job_id}/preview?file_path=src/main.rs");
        let res_file = app
            .oneshot(
                Request::builder()
                    .uri(&uri_file)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res_file.status(), StatusCode::OK);
        let body_file = body_to_string(res_file.into_body()).await;
        let preview_file: serde_json::Value = serde_json::from_str(&body_file).unwrap();
        assert!(preview_file["content"]
            .as_str()
            .unwrap()
            .contains("println!"));
    }

    #[tokio::test]
    async fn test_list_installations_missing_session_rejected() {
        let app = setup_test_app();
        let req = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(res.into_body()).await;
        assert!(body.contains("VALIDATION_ERROR"));
        assert!(body.contains("X-Session-ID"));
    }

    #[tokio::test]
    async fn test_list_installations_success() {
        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));

        let session_id = uuid::Uuid::new_v4();
        let install_id = 12345678i64;
        let expires = chrono::Utc::now() + chrono::Duration::hours(2);

        token_storage
            .save_installation_token(session_id, install_id, "octo-org", "ghs_Secret123", expires)
            .await
            .unwrap();

        let config = AppConfig::default();
        let state = AppState::new(config).with_token_storage(token_storage);
        let app = build_app(state);

        let req = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", session_id.to_string())
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_to_string(res.into_body()).await;
        let list: serde_json::Value = serde_json::from_str(&body).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["installation_id"], install_id);
        assert_eq!(arr[0]["account_login"], "octo-org");
        assert_eq!(arr[0]["is_revoked"], false);
        // Zero secret leakage check
        assert!(!body.contains("ghs_Secret123"));
    }

    #[tokio::test]
    async fn test_revoke_installation_success() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let upstream_called = std::sync::Arc::new(AtomicUsize::new(0));
        let count_clone = upstream_called.clone();

        // Mock GitHub DELETE /installation/token endpoint
        let mock_gh = axum::Router::new().route(
            "/installation/token",
            axum::routing::delete(move || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    StatusCode::NO_CONTENT
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, mock_gh).await.unwrap();
        });

        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));
        let session_id = uuid::Uuid::new_v4();
        let install_id = 998877i64;
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);

        token_storage
            .save_installation_token(
                session_id,
                install_id,
                "revoke-target",
                "ghs_ToRevoke",
                expires,
            )
            .await
            .unwrap();

        let gh_client = std::sync::Arc::new(
            web_repo_to_ai_github_extractor::client::GitHubApiClient::new()
                .with_base_url_override(format!("http://{addr}")),
        );

        let config = AppConfig::default();
        let state = AppState::new(config)
            .with_token_storage(token_storage.clone())
            .with_github_client(gh_client);
        let app = build_app(state);

        let req = Request::builder()
            .uri(format!("/api/v1/auth/github/installations/{install_id}"))
            .method("DELETE")
            .header("X-Session-ID", session_id.to_string())
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_to_string(res.into_body()).await;
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["status"], "revoked");
        assert_eq!(parsed["installation_id"], install_id);
        assert_eq!(parsed["upstream_revoked"], true);
        assert_eq!(upstream_called.load(Ordering::SeqCst), 1);

        // Verify active token is now None
        assert!(token_storage
            .get_active_token(session_id, install_id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_revoke_installation_not_found() {
        let app = setup_test_app();
        let session_id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .uri("/api/v1/auth/github/installations/99999999")
            .method("DELETE")
            .header("X-Session-ID", session_id.to_string())
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = body_to_string(res.into_body()).await;
        assert!(body.contains("RESOURCE_NOT_FOUND"));
    }

    #[tokio::test]
    async fn test_revoke_installation_cross_session_isolation() {
        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));

        let victim_session = uuid::Uuid::new_v4();
        let attacker_session = uuid::Uuid::new_v4();
        let install_id = 554433i64;
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);

        token_storage
            .save_installation_token(
                victim_session,
                install_id,
                "victim",
                "ghs_VictimToken",
                expires,
            )
            .await
            .unwrap();

        let config = AppConfig::default();
        let state = AppState::new(config).with_token_storage(token_storage.clone());
        let app = build_app(state);

        // Attacker attempts to delete victim's installation
        let req = Request::builder()
            .uri(format!("/api/v1/auth/github/installations/{install_id}"))
            .method("DELETE")
            .header("X-Session-ID", attacker_session.to_string())
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        // Victim token must remain active and unaffected
        assert!(token_storage
            .get_active_token(victim_session, install_id)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn test_github_webhook_missing_signature_rejected() {
        let config = AppConfig::default();
        let state = AppState::new(config).with_webhook_secret("test_secret");
        let app = build_app(state);

        let req = Request::builder()
            .uri("/api/v1/auth/github/webhook")
            .method("POST")
            .header("X-GitHub-Event", "ping")
            .body(Body::from(r#"{"zen":"test"}"#))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = body_to_string(res.into_body()).await;
        assert!(body.contains("X-Hub-Signature-256"));
    }

    #[tokio::test]
    async fn test_github_webhook_invalid_signature_rejected() {
        let config = AppConfig::default();
        let state = AppState::new(config).with_webhook_secret("super_secret_key");
        let app = build_app(state);

        let req = Request::builder()
            .uri("/api/v1/auth/github/webhook")
            .method("POST")
            .header(
                "X-Hub-Signature-256",
                "sha256=0000000000000000000000000000000000000000000000000000000000000000",
            )
            .header("X-GitHub-Event", "ping")
            .body(Body::from(r#"{"zen":"test"}"#))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body = body_to_string(res.into_body()).await;
        assert!(
            body.contains("SSRF_ADDRESS_BLOCKED")
                || body.contains("Security")
                || body.contains("Webhook verification failed")
        );
    }

    #[tokio::test]
    async fn test_github_webhook_installation_deleted_synchronization() {
        let secret = "test_webhook_sync_secret";
        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));

        let session1 = uuid::Uuid::new_v4();
        let session2 = uuid::Uuid::new_v4();
        let install_id = 887766i64;
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);

        token_storage
            .save_installation_token(session1, install_id, "deleted-account", "token_a", expires)
            .await
            .unwrap();
        token_storage
            .save_installation_token(session2, install_id, "deleted-account", "token_b", expires)
            .await
            .unwrap();

        let config = AppConfig::default();
        let state = AppState::new(config)
            .with_token_storage(token_storage.clone())
            .with_webhook_secret(secret);
        let app = build_app(state);

        let installation = web_repo_to_ai_github_extractor::installation::GitHubInstallation {
            id: install_id as u64,
            account: web_repo_to_ai_github_extractor::installation::GitHubAccountSummary {
                id: 1122,
                login: "deleted-account".to_string(),
                account_type: "User".to_string(),
                avatar_url: None,
                html_url: None,
            },
            repository_selection:
                web_repo_to_ai_github_extractor::installation::RepositorySelection::All,
            access_tokens_url: "".to_string(),
            repositories_url: "".to_string(),
            html_url: "".to_string(),
            app_id: 42,
            app_slug: Some("web-repo-to-ai".to_string()),
            target_id: 1122,
            target_type: "User".to_string(),
            permissions: Default::default(),
            events: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            suspended_at: None,
            suspended_by: None,
        };
        let payload = serde_json::json!({
            "action": "deleted",
            "installation": installation,
        });
        let body_bytes = serde_json::to_vec(&payload).unwrap();
        let sig = web_repo_to_ai_github_extractor::callback::WebhookSignatureVerifier::compute_signature_header(
            secret.as_bytes(),
            &body_bytes,
        );

        let req = Request::builder()
            .uri("/api/v1/auth/github/webhook")
            .method("POST")
            .header("X-Hub-Signature-256", sig)
            .header("X-GitHub-Event", "installation")
            .header("X-GitHub-Delivery", uuid::Uuid::new_v4().to_string())
            .body(Body::from(body_bytes))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_to_string(res.into_body()).await;
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["status"], "processed");
        assert_eq!(parsed["installation_id"], install_id);
        assert_eq!(parsed["revoked_count"], 2);

        // Verify tokens are now wiped for both sessions
        assert!(token_storage
            .get_active_token(session1, install_id)
            .await
            .unwrap()
            .is_none());
        assert!(token_storage
            .get_active_token(session2, install_id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_github_webhook_replay_rejected() {
        let secret = "replay_defense_secret";
        let config = AppConfig::default();
        let state = AppState::new(config).with_webhook_secret(secret);
        let app = build_app(state);

        let payload = serde_json::json!({
            "zen": "Replay test",
            "hook_id": 9988
        });
        let body_bytes = serde_json::to_vec(&payload).unwrap();
        let sig = web_repo_to_ai_github_extractor::callback::WebhookSignatureVerifier::compute_signature_header(
            secret.as_bytes(),
            &body_bytes,
        );
        let delivery_id = uuid::Uuid::new_v4().to_string();

        // First delivery: OK
        let req1 = Request::builder()
            .uri("/api/v1/auth/github/webhook")
            .method("POST")
            .header("X-Hub-Signature-256", &sig)
            .header("X-GitHub-Event", "ping")
            .header("X-GitHub-Delivery", &delivery_id)
            .body(Body::from(body_bytes.clone()))
            .unwrap();

        let res1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(res1.status(), StatusCode::OK);

        // Second delivery (replay): Forbidden
        let req2 = Request::builder()
            .uri("/api/v1/auth/github/webhook")
            .method("POST")
            .header("X-Hub-Signature-256", &sig)
            .header("X-GitHub-Event", "ping")
            .header("X-GitHub-Delivery", &delivery_id)
            .body(Body::from(body_bytes))
            .unwrap();

        let res2 = app.oneshot(req2).await.unwrap();
        assert_eq!(res2.status(), StatusCode::FORBIDDEN);
        let body2 = body_to_string(res2.into_body()).await;
        assert!(body2.contains("replay") || body2.contains("Duplicate"));
    }

    #[tokio::test]
    async fn test_api_list_installations_cross_tenant_isolation() {
        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));

        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);

        // Tenant A has 2 installations
        token_storage
            .save_installation_token(tenant_a, 1001, "tenant-a-org1", "ghs_SecretA1", expires)
            .await
            .unwrap();
        token_storage
            .save_installation_token(tenant_a, 1002, "tenant-a-org2", "ghs_SecretA2", expires)
            .await
            .unwrap();

        let config = AppConfig::default();
        let state = AppState::new(config).with_token_storage(token_storage.clone());
        let app = build_app(state);

        // Tenant B lists installations -> must be empty []
        let req_b = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", tenant_b.to_string())
            .body(Body::empty())
            .unwrap();

        let res_b = app.clone().oneshot(req_b).await.unwrap();
        assert_eq!(res_b.status(), StatusCode::OK);
        let body_b = body_to_string(res_b.into_body()).await;
        let list_b: Vec<serde_json::Value> = serde_json::from_str(&body_b).unwrap();
        assert!(
            list_b.is_empty(),
            "Tenant B must see 0 installations from Tenant A"
        );
        assert!(!body_b.contains("tenant-a-org"));
        assert!(!body_b.contains("ghs_Secret"));

        // Tenant B adds its own installation
        token_storage
            .save_installation_token(tenant_b, 2002, "tenant-b-corp", "ghs_SecretB1", expires)
            .await
            .unwrap();

        // Tenant B lists installations -> sees only installation 2002
        let req_b2 = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", tenant_b.to_string())
            .body(Body::empty())
            .unwrap();
        let res_b2 = app.clone().oneshot(req_b2).await.unwrap();
        assert_eq!(res_b2.status(), StatusCode::OK);
        let body_b2 = body_to_string(res_b2.into_body()).await;
        let list_b2: Vec<serde_json::Value> = serde_json::from_str(&body_b2).unwrap();
        assert_eq!(list_b2.len(), 1);
        assert_eq!(list_b2[0]["installation_id"], 2002);
        assert_eq!(list_b2[0]["account_login"], "tenant-b-corp");

        // Tenant A lists installations -> sees only 1001 and 1002
        let req_a = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", tenant_a.to_string())
            .body(Body::empty())
            .unwrap();
        let res_a = app.oneshot(req_a).await.unwrap();
        assert_eq!(res_a.status(), StatusCode::OK);
        let body_a = body_to_string(res_a.into_body()).await;
        let list_a: Vec<serde_json::Value> = serde_json::from_str(&body_a).unwrap();
        assert_eq!(list_a.len(), 2);
        assert!(list_a.iter().all(|i| i["installation_id"] != 2002));
        assert!(!body_a.contains("tenant-b-corp"));
    }

    #[tokio::test]
    async fn test_api_session_header_validation_errors() {
        let app = setup_test_app();

        // 1. Missing header on GET installations -> 400
        let req_missing = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res1 = app.clone().oneshot(req_missing).await.unwrap();
        assert_eq!(res1.status(), StatusCode::BAD_REQUEST);
        let body1 = body_to_string(res1.into_body()).await;
        assert!(body1.contains("VALIDATION_ERROR"));
        assert!(body1.contains("X-Session-ID"));

        // 2. Malformed UUID string on GET installations -> 400
        let req_malformed = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", "not-a-valid-uuid-format")
            .body(Body::empty())
            .unwrap();
        let res2 = app.clone().oneshot(req_malformed).await.unwrap();
        assert_eq!(res2.status(), StatusCode::BAD_REQUEST);
        let body2 = body_to_string(res2.into_body()).await;
        assert!(body2.contains("VALIDATION_ERROR"));
        assert!(body2.contains("Invalid UUID formatted string"));

        // 3. Malformed UUID string on DELETE installation -> 400
        let req_del_malformed = Request::builder()
            .uri("/api/v1/auth/github/installations/12345")
            .method("DELETE")
            .header("X-Session-ID", "invalid-guid")
            .body(Body::empty())
            .unwrap();
        let res3 = app.oneshot(req_del_malformed).await.unwrap();
        assert_eq!(res3.status(), StatusCode::BAD_REQUEST);
        let body3 = body_to_string(res3.into_body()).await;
        assert!(body3.contains("VALIDATION_ERROR"));
    }

    #[tokio::test]
    async fn test_api_concurrent_multi_tenant_revocation_isolation() {
        let key = web_repo_to_ai_core::crypto::EncryptionKey::generate().unwrap();
        let token_storage =
            std::sync::Arc::new(web_repo_to_ai_storage::InMemoryTokenStorage::new(key));

        let tenant_a = uuid::Uuid::new_v4();
        let tenant_b = uuid::Uuid::new_v4();
        let install_a = 5001i64;
        let install_b = 6002i64;
        let expires = chrono::Utc::now() + chrono::Duration::hours(2);

        token_storage
            .save_installation_token(tenant_a, install_a, "alpha-org", "ghs_AlphaToken", expires)
            .await
            .unwrap();
        token_storage
            .save_installation_token(tenant_b, install_b, "beta-org", "ghs_BetaToken", expires)
            .await
            .unwrap();

        let config = AppConfig::default();
        let state = AppState::new(config).with_token_storage(token_storage.clone());
        let app = build_app(state);

        // Tenant A revokes its own installation 5001 -> 200 OK
        let req_del = Request::builder()
            .uri(format!("/api/v1/auth/github/installations/{install_a}"))
            .method("DELETE")
            .header("X-Session-ID", tenant_a.to_string())
            .body(Body::empty())
            .unwrap();
        let res_del = app.clone().oneshot(req_del).await.unwrap();
        assert_eq!(res_del.status(), StatusCode::OK);

        // Verify Tenant A now has 0 active installations
        let req_list_a = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", tenant_a.to_string())
            .body(Body::empty())
            .unwrap();
        let res_list_a = app.clone().oneshot(req_list_a).await.unwrap();
        assert_eq!(res_list_a.status(), StatusCode::OK);
        let body_list_a = body_to_string(res_list_a.into_body()).await;
        let list_a: Vec<serde_json::Value> = serde_json::from_str(&body_list_a).unwrap();
        assert!(list_a.is_empty());

        // Verify Tenant B's installation 6002 is completely unaffected and active
        let req_list_b = Request::builder()
            .uri("/api/v1/auth/github/installations")
            .method("GET")
            .header("X-Session-ID", tenant_b.to_string())
            .body(Body::empty())
            .unwrap();
        let res_list_b = app.oneshot(req_list_b).await.unwrap();
        assert_eq!(res_list_b.status(), StatusCode::OK);
        let body_list_b = body_to_string(res_list_b.into_body()).await;
        let list_b: Vec<serde_json::Value> = serde_json::from_str(&body_list_b).unwrap();
        assert_eq!(list_b.len(), 1);
        assert_eq!(list_b[0]["installation_id"], install_b);
        assert_eq!(list_b[0]["account_login"], "beta-org");
    }
}
