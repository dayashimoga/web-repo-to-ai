//! HTTP middleware layers: CORS, request tracing, and timeout controls.
//!
//! Enforces security headers, W3C TraceContext propagation, and request timeout bounds.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use web_repo_to_ai_core::{AppConfig, RequestId};
use web_repo_to_ai_observability::TraceContext;

use crate::state::AppState;

/// Header name constant for request correlation identifier.
pub static HEADER_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
/// Header name constant for W3C distributed traceparent.
pub static HEADER_TRACEPARENT: HeaderName = HeaderName::from_static("traceparent");

/// Builds the CORS middleware layer based on server configuration origins.
pub fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let mut origins = Vec::new();
    for origin in &config.server.cors_allowed_origins {
        if let Ok(hv) = HeaderValue::from_str(origin) {
            origins.push(hv);
        }
    }

    let cors = if origins.is_empty() {
        CorsLayer::permissive()
    } else {
        CorsLayer::new().allow_origin(origins)
    };

    cors.allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
        Method::HEAD,
    ])
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::AUTHORIZATION,
        axum::http::header::ACCEPT,
        HEADER_REQUEST_ID.clone(),
        HEADER_TRACEPARENT.clone(),
    ])
}

/// Builds the global request timeout middleware layer.
#[must_use]
pub fn build_timeout_layer(config: &AppConfig) -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(config.server.request_timeout_secs))
}

/// Middleware that extracts or generates correlation identifiers (`x-request-id`, `traceparent`),
/// records latency and HTTP request counts in Prometheus metrics, and attaches headers to outgoing responses.
pub async fn trace_id_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();

    // 1. Resolve or generate RequestId
    let request_id = req
        .headers()
        .get(&HEADER_REQUEST_ID)
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<RequestId>().ok())
        .unwrap_or_else(RequestId::new);

    // 2. Resolve or generate TraceContext
    let trace_context = req
        .headers()
        .get(&HEADER_TRACEPARENT)
        .and_then(|val| val.to_str().ok())
        .and_then(|s| TraceContext::from_traceparent(s).ok())
        .unwrap_or_else(TraceContext::new_root);

    // Store in request extensions for downstream extractors
    req.extensions_mut().insert(request_id);
    req.extensions_mut().insert(trace_context.clone());

    // Execute downstream pipeline
    let mut response = next.run(req).await;

    // Record Prometheus request metrics
    let elapsed_ms = (start.elapsed().as_micros() as f64) / 1000.0;
    state.metrics.http_request_duration_ms.observe(elapsed_ms);
    let status_str = response.status().as_u16().to_string();
    let label = format!("method=\"{method}\",status=\"{status_str}\"");
    state.metrics.http_requests_total.inc(&label);

    // Attach correlation headers to outgoing HTTP response
    if let Ok(val) = HeaderValue::from_str(&request_id.to_string()) {
        response
            .headers_mut()
            .insert(HEADER_REQUEST_ID.clone(), val);
    }

    if let Ok(val) = HeaderValue::from_str(&trace_context.to_traceparent()) {
        response
            .headers_mut()
            .insert(HEADER_TRACEPARENT.clone(), val);
    }

    response
}
