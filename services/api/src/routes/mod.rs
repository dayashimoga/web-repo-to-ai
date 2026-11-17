//! REST API route definitions and request dispatching.
//!
//! Exposes the root service metadata, health/readiness endpoints, Prometheus metrics,
//! and RFC 7807 fallback handlers.

pub mod health;
pub mod metrics;

use crate::state::AppState;
use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

/// Root landing handler returning service descriptor and API version info.
async fn root_handler(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "service": "web-repo-to-ai-api",
        "description": "Universal URL & GitHub Repository to AI-Ready Content Converter API",
        "version": env!("CARGO_PKG_VERSION"),
        "base_path": "/api/v1",
        "endpoints": {
            "health": "/api/v1/health",
            "ready": "/api/v1/ready",
            "metrics": "/api/v1/metrics",
            "sources_detect": "/api/v1/sources/detect",
            "jobs": "/api/v1/jobs",
        }
    }))
}

/// Fallback handler returning RFC 7807 compliant problem details JSON for unmatched routes.
async fn fallback_handler(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "code": "RESOURCE_NOT_FOUND",
                "message": format!("Route '{}' was not found on this server", uri),
                "details": {
                    "path": uri.path()
                }
            }
        })),
    )
}

/// Constructs the primary Axum application router with all registered routes and state.
pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new()
        .route("/health", get(health::health_handler))
        .route("/ready", get(health::readiness_handler))
        .route("/metrics", get(metrics::metrics_handler));

    Router::new()
        .route("/", get(root_handler))
        .nest("/api/v1", api_v1)
        .fallback(fallback_handler)
        .with_state(state)
}
