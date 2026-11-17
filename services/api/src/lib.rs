//! # Web API Service Library (`web-repo-to-ai-api`)
//!
//! Provides the core HTTP router, middleware stack, Prometheus metrics endpoints,
//! and server lifecycle for the `web-repo-to-ai` REST and SSE API.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod middleware;
pub mod routes;
pub mod server;
pub mod state;

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
}
