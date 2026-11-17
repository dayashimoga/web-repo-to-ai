//! Prometheus metrics exposition endpoint handler.
//!
//! Provides [`metrics_handler`] serving metrics in Prometheus text exposition format.

use crate::state::AppState;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

/// Handles `GET /api/v1/metrics`.
pub async fn metrics_handler(State(state): State<AppState>) -> Response {
    let body = state.metrics.render_prometheus_text();
    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}
