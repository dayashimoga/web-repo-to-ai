//! REST API route definitions and request dispatching.
//!
//! Exposes the root service metadata, health/readiness endpoints, Prometheus metrics,
//! and RFC 7807 fallback handlers.

pub mod auth_github;
pub mod batch;
pub mod health;
pub mod jobs;
pub mod metrics;
pub mod sources;

use crate::error::not_found_problem;
use crate::state::AppState;
use axum::extract::{Extension, OriginalUri, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde_json::json;
use web_repo_to_ai_core::RequestId;

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
            "jobs_batch": "/api/v1/jobs/batch",
            "auth_github_installations": "/api/v1/auth/github/installations",
            "auth_github_webhook": "/api/v1/auth/github/webhook",
        }
    }))
}

/// Fallback handler returning RFC 7807 compliant problem details JSON for unmatched routes.
async fn fallback_handler(
    OriginalUri(uri): OriginalUri,
    req_ext: Option<Extension<RequestId>>,
) -> impl IntoResponse {
    let request_id = req_ext.map(|Extension(r)| r);
    not_found_problem("Route", uri.path(), request_id)
}

/// Constructs the primary Axum application router with all registered routes and state.
pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new()
        .route("/health", get(health::health_handler))
        .route("/ready", get(health::readiness_handler))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/sources/detect", post(sources::detect_sources_handler))
        .route(
            "/jobs/batch",
            post(batch::create_batch_job_handler).get(batch::list_batches_handler),
        )
        .route(
            "/jobs/batch/:batch_id",
            get(batch::get_batch_job_handler).delete(batch::cancel_batch_handler),
        )
        .route(
            "/jobs/batch/:batch_id/cancel",
            post(batch::cancel_batch_handler),
        )
        .route(
            "/jobs/batch/:batch_id/items",
            get(batch::get_batch_items_handler),
        )
        .route(
            "/jobs/batch/:batch_id/items/:item_index",
            get(batch::get_batch_item_handler),
        )
        .route(
            "/jobs/batch/:batch_id/export/:format",
            get(batch::export_batch_handler),
        )
        .route(
            "/jobs/batch/:batch_id/artifacts",
            get(batch::list_batch_artifacts_handler),
        )
        .route(
            "/jobs",
            post(jobs::create_job_handler).get(jobs::list_jobs_handler),
        )
        .route("/jobs/:job_id", get(jobs::get_job_handler))
        .route("/jobs/:job_id/preview", get(jobs::get_preview_handler))
        .route("/jobs/:job_id/events", get(jobs::job_events_handler))
        .route("/jobs/:job_id/stream", get(jobs::job_events_handler))
        .route(
            "/jobs/:job_id/artifacts/:format",
            get(jobs::get_artifact_handler),
        )
        .route(
            "/auth/github/installations",
            get(auth_github::list_installations_handler),
        )
        .route(
            "/auth/github/installations/:installation_id",
            delete(auth_github::revoke_installation_handler),
        )
        .route(
            "/auth/github/webhook",
            post(auth_github::github_webhook_handler),
        );

    Router::new()
        .route("/", get(root_handler))
        .nest("/api/v1", api_v1)
        .fallback(fallback_handler)
        .with_state(state)
}
