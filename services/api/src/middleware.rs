//! HTTP middleware layers: CORS, request tracing, rate limiting, and timeout controls.
//!
//! Enforces security headers, W3C TraceContext propagation, token-bucket rate limiting, and request timeout bounds.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use web_repo_to_ai_core::rate_limit::{RateLimitDecision, RateLimitTier};
use web_repo_to_ai_core::{AppConfig, CoreError, RequestId};
use web_repo_to_ai_observability::TraceContext;

use crate::error::to_problem_response;
use crate::state::AppState;

/// Header name constant for request correlation identifier.
pub static HEADER_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
/// Header name constant for W3C distributed traceparent.
pub static HEADER_TRACEPARENT: HeaderName = HeaderName::from_static("traceparent");
/// Header name constant for rate limit quota ceiling.
pub static HEADER_RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
/// Header name constant for remaining rate limit tokens.
pub static HEADER_RATELIMIT_REMAINING: HeaderName =
    HeaderName::from_static("x-ratelimit-remaining");
/// Header name constant for seconds until rate limit quota resets.
pub static HEADER_RATELIMIT_RESET: HeaderName = HeaderName::from_static("x-ratelimit-reset");

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
        HEADER_RATELIMIT_LIMIT.clone(),
        HEADER_RATELIMIT_REMAINING.clone(),
        HEADER_RATELIMIT_RESET.clone(),
    ])
    .expose_headers([
        HEADER_REQUEST_ID.clone(),
        HEADER_TRACEPARENT.clone(),
        HEADER_RATELIMIT_LIMIT.clone(),
        HEADER_RATELIMIT_REMAINING.clone(),
        HEADER_RATELIMIT_RESET.clone(),
        axum::http::header::RETRY_AFTER,
    ])
}

/// Builds the global request timeout middleware layer.
#[must_use]
pub fn build_timeout_layer(config: &AppConfig) -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(config.server.request_timeout_secs))
}

/// Checks whether a request path should bypass rate limiting (probes, documentation, metrics).
#[must_use]
pub fn is_rate_limit_exempt(path: &str) -> bool {
    let trimmed = path.trim_end_matches('/');
    let normalized = if trimmed.is_empty() { "/" } else { trimmed };
    matches!(
        normalized,
        "/" | "/docs"
            | "/api/v1/health"
            | "/api/v1/ready"
            | "/api/v1/metrics"
            | "/api/v1/openapi.json"
            | "/api/v1/openapi.yaml"
            | "/api/v1/docs"
    )
}

/// Resolves client identification key and applicable rate limit tier from request headers.
#[must_use]
pub fn resolve_rate_limit_client(req: &Request) -> (String, RateLimitTier) {
    // 1. Check for Authorization header
    if let Some(auth_val) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_val.to_str() {
            let auth_str = auth_str.trim();
            let token = auth_str
                .strip_prefix("Bearer ")
                .or_else(|| auth_str.strip_prefix("bearer "))
                .or_else(|| auth_str.strip_prefix("ApiKey "))
                .or_else(|| auth_str.strip_prefix("apikey "))
                .map(str::trim);

            if let Some(tok) = token {
                if !tok.is_empty() {
                    let hash = blake3::hash(tok.as_bytes()).to_hex().to_string();
                    return (format!("key:{hash}"), RateLimitTier::Authenticated);
                }
            }
        }
    }

    // 2. Check for X-Session-ID header
    if let Some(session_val) = req.headers().get("x-session-id") {
        if let Ok(session_str) = session_val.to_str() {
            let session_str = session_str.trim();
            if !session_str.is_empty() {
                return (
                    format!("session:{session_str}"),
                    RateLimitTier::Authenticated,
                );
            }
        }
    }

    // 3. Fallback to client IP from forwarding headers or loopback
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(client_ip) = xff_str.split(',').next() {
                let client_ip = client_ip.trim();
                if !client_ip.is_empty() {
                    return (format!("ip:{client_ip}"), RateLimitTier::Anonymous);
                }
            }
        }
    }

    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            let ip_str = ip_str.trim();
            if !ip_str.is_empty() {
                return (format!("ip:{ip_str}"), RateLimitTier::Anonymous);
            }
        }
    }

    ("ip:127.0.0.1".to_string(), RateLimitTier::Anonymous)
}

/// Middleware enforcing token-bucket rate limits per client identity and tier.
///
/// Injects `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers on outgoing responses.
/// Returns HTTP 429 Too Many Requests with RFC 7807 problem details and `Retry-After` header when quota is exhausted.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Bypass if disabled or OPTIONS preflight
    if !state.config.rate_limit.enabled || req.method() == Method::OPTIONS {
        return next.run(req).await;
    }

    // Bypass exempt system/documentation paths
    if is_rate_limit_exempt(req.uri().path()) {
        return next.run(req).await;
    }

    let (client_key, tier) = resolve_rate_limit_client(&req);
    let decision = state.rate_limiter.check(&client_key, tier, 1);

    match decision {
        RateLimitDecision::Allowed {
            limit,
            remaining,
            reset_seconds,
        } => {
            let mut response = next.run(req).await;

            if let Ok(val) = HeaderValue::from_str(&limit.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_LIMIT.clone(), val);
            }
            if let Ok(val) = HeaderValue::from_str(&remaining.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_REMAINING.clone(), val);
            }
            if let Ok(val) = HeaderValue::from_str(&reset_seconds.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_RESET.clone(), val);
            }

            response
        }
        RateLimitDecision::Exceeded {
            limit,
            retry_after_seconds,
            reset_seconds,
        } => {
            let request_id = req.extensions().get::<RequestId>().copied();
            let err = CoreError::RateLimitExceeded {
                retry_after_seconds,
                message: format!(
                    "Rate limit exceeded. Quota resets in {reset_seconds} seconds (retry after {retry_after_seconds} seconds)."
                ),
            };

            let mut response = to_problem_response(err, request_id);

            if let Ok(val) = HeaderValue::from_str(&limit.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_LIMIT.clone(), val);
            }
            if let Ok(val) = HeaderValue::from_str("0") {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_REMAINING.clone(), val);
            }
            if let Ok(val) = HeaderValue::from_str(&reset_seconds.to_string()) {
                response
                    .headers_mut()
                    .insert(HEADER_RATELIMIT_RESET.clone(), val);
            }
            if let Ok(val) = HeaderValue::from_str(&retry_after_seconds.to_string()) {
                response
                    .headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, val);
            }

            response
        }
    }
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
