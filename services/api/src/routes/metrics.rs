//! Prometheus metrics exposition endpoint handler.
//!
//! Provides [`metrics_handler`] serving metrics in Prometheus text exposition format.

use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use crate::state::AppState;

/// Standard Prometheus exposition media type.
pub const PROMETHEUS_MEDIA_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

/// Handles `GET /api/v1/metrics`.
///
/// Serves application, job, HTTP, and resource metrics in Prometheus text exposition format.
pub async fn metrics_handler(State(state): State<AppState>) -> Response {
    let mut body = state.metrics.render_prometheus_text();

    // Append dynamic process uptime metric
    body.push_str("# HELP web_repo_to_ai_uptime_seconds Process uptime in seconds.\n");
    body.push_str("# TYPE web_repo_to_ai_uptime_seconds counter\n");
    body.push_str(&format!(
        "web_repo_to_ai_uptime_seconds {}\n",
        state.uptime_seconds()
    ));

    // Append build info metric
    body.push_str("# HELP web_repo_to_ai_build_info Build and version metadata.\n");
    body.push_str("# TYPE web_repo_to_ai_build_info gauge\n");
    body.push_str(&format!(
        "web_repo_to_ai_build_info{{version=\"{}\"}} 1\n",
        env!("CARGO_PKG_VERSION")
    ));

    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static(PROMETHEUS_MEDIA_TYPE),
    );
    response
}
