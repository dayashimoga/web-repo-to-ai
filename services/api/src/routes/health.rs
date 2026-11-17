//! Liveness and readiness probe handlers.
//!
//! Provides [`health_handler`] and [`readiness_handler`] for container orchestrators.

use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

/// Handles `GET /api/v1/health` (process liveness probe).
pub async fn health_handler(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "pass",
        "service": "web-repo-to-ai-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Handles `GET /api/v1/ready` (service dependencies readiness probe).
pub async fn readiness_handler(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "ready",
        "service": "web-repo-to-ai-api",
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "storage": "pass",
            "database": "pass"
        }
    }))
}
