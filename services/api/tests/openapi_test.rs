//! Integration tests for OpenAPI 3.1 specifications and interactive documentation endpoints.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use web_repo_to_ai_api::{build_app, AppState};
use web_repo_to_ai_core::AppConfig;

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
async fn test_openapi_json_endpoint() {
    let app = setup_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
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
    assert!(content_type.contains("application/json"));

    let body = body_to_string(response.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");

    assert_eq!(v["openapi"], "3.1.0");
    assert_eq!(v["info"]["title"], "web-repo-to-ai REST API");
    assert_eq!(v["info"]["version"], "0.1.0");
    assert!(v["paths"].is_object());
    assert!(v["components"]["schemas"].is_object());
    assert!(v["components"]["securitySchemes"].is_object());
}

#[tokio::test]
async fn test_openapi_yaml_endpoint() {
    let app = setup_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.yaml")
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
    assert!(content_type.contains("yaml"));

    let body = body_to_string(response.into_body()).await;
    assert!(body.starts_with("openapi: 3.1.0"));
    assert!(body.contains("info:"));
    assert!(body.contains("paths:"));
    assert!(body.contains("/sources/detect:"));
    assert!(body.contains("/jobs/batch:"));
}

#[tokio::test]
async fn test_openapi_docs_endpoint_and_root_alias() {
    let app = setup_test_app();

    // 1. /api/v1/docs
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs")
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
    assert!(content_type.contains("text/html"));

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("<title>web-repo-to-ai REST API Reference</title>"));
    assert!(body.contains("/api/v1/openapi.json"));

    // 2. /docs top-level alias
    let response_root_alias = app
        .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response_root_alias.status(), StatusCode::OK);
    let body_root_alias = body_to_string(response_root_alias.into_body()).await;
    assert!(body_root_alias.contains("<title>web-repo-to-ai REST API Reference</title>"));
}

#[tokio::test]
async fn test_root_endpoint_contains_openapi_links() {
    let app = setup_test_app();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();

    let endpoints = &v["endpoints"];
    assert_eq!(endpoints["openapi_json"], "/api/v1/openapi.json");
    assert_eq!(endpoints["openapi_yaml"], "/api/v1/openapi.yaml");
    assert_eq!(endpoints["docs"], "/api/v1/docs");
}

#[tokio::test]
async fn test_openapi_route_coverage() {
    let app = setup_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let paths = v["paths"].as_object().expect("paths is object");

    // All registered operational endpoints must be documented in OpenAPI specification
    let required_paths = [
        "/health",
        "/ready",
        "/metrics",
        "/openapi.json",
        "/openapi.yaml",
        "/docs",
        "/sources/detect",
        "/jobs",
        "/jobs/{job_id}",
        "/jobs/{job_id}/preview",
        "/jobs/{job_id}/events",
        "/jobs/{job_id}/stream",
        "/jobs/{job_id}/artifacts/{format}",
        "/jobs/batch",
        "/jobs/batch/{batch_id}",
        "/jobs/batch/{batch_id}/cancel",
        "/jobs/batch/{batch_id}/items",
        "/jobs/batch/{batch_id}/items/{item_index}",
        "/jobs/batch/{batch_id}/export/{format}",
        "/jobs/batch/{batch_id}/artifacts",
        "/auth/github/installations",
        "/auth/github/installations/{installation_id}",
        "/auth/github/webhook",
    ];

    for path in &required_paths {
        assert!(
            paths.contains_key(*path),
            "OpenAPI specification is missing path definition for: {path}"
        );
    }
}

#[tokio::test]
async fn test_openapi_schemas_presence_and_validity() {
    let app = setup_test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let schemas = v["components"]["schemas"]
        .as_object()
        .expect("schemas is object");

    let required_schemas = [
        "ApiProblemEnvelope",
        "ApiErrorDetail",
        "HealthResponse",
        "ReadinessResponse",
        "DetectSourcesRequest",
        "DetectSourcesResponse",
        "DetectedSource",
        "CreateJobRequest",
        "JobResponse",
        "PaginatedJobsResponse",
        "JobPreviewResponse",
        "PreviewFileItem",
        "CreateBatchJobRequest",
        "CreateBatchJobResponse",
        "BatchJobResponse",
        "BatchProgress",
        "ListBatchesResponse",
        "BatchItemsListResponse",
        "BatchSummaryItem",
        "BatchArtifactsResponse",
        "BatchArtifactItemDescriptor",
        "InstallationSummaryResponse",
        "RevokeInstallationResponse",
        "WebhookReceiptResponse",
    ];

    for schema_name in &required_schemas {
        assert!(
            schemas.contains_key(*schema_name),
            "Missing schema definition for: {schema_name}"
        );
        let schema = &schemas[*schema_name];
        assert_eq!(
            schema["type"], "object",
            "Schema {schema_name} should be an object"
        );
    }

    let security_schemes = v["components"]["securitySchemes"]
        .as_object()
        .expect("securitySchemes is object");
    assert!(security_schemes.contains_key("SessionAuth"));
    assert!(security_schemes.contains_key("BearerAuth"));
    assert!(security_schemes.contains_key("GitHubWebhookSignature"));
}
