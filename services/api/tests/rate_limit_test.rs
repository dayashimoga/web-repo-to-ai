//! Integration tests for API rate limiting: token bucket replenishment, sliding window evaluation,
//! tiered quotas, HTTP 429 Too Many Requests responses, Retry-After headers, and path exemptions.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::collections::HashSet;
use tower::ServiceExt;
use uuid::Uuid;
use web_repo_to_ai_api::build_app;
use web_repo_to_ai_api::middleware::{
    HEADER_RATELIMIT_LIMIT, HEADER_RATELIMIT_REMAINING, HEADER_RATELIMIT_RESET,
};
use web_repo_to_ai_api::state::AppState;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_core::config::RateLimitConfig;
use web_repo_to_ai_core::AppConfig;

fn setup_rate_limit_test_app(rate_limit: RateLimitConfig) -> (axum::Router, AppState) {
    let config = AppConfig {
        rate_limit,
        ..Default::default()
    };
    let state = AppState::new(config);
    let app = build_app(state.clone());
    (app, state)
}

async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_rate_limit_headers_injected_on_allowed_requests() {
    let (app, _) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: true,
        requests_per_minute: 120,
        burst_capacity: 30,
        anonymous_requests_per_minute: 60,
        anonymous_burst_capacity: 15,
        window_seconds: 60,
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify rate limit headers
    let limit = res
        .headers()
        .get(&HEADER_RATELIMIT_LIMIT)
        .expect("x-ratelimit-limit must be present")
        .to_str()
        .unwrap();
    assert_eq!(limit, "15");

    let remaining = res
        .headers()
        .get(&HEADER_RATELIMIT_REMAINING)
        .expect("x-ratelimit-remaining must be present")
        .to_str()
        .unwrap();
    assert_eq!(remaining, "14");

    let reset = res
        .headers()
        .get(&HEADER_RATELIMIT_RESET)
        .expect("x-ratelimit-reset must be present")
        .to_str()
        .unwrap();
    let reset_secs: u64 = reset.parse().expect("reset must be a valid integer");
    assert!(reset_secs >= 1);
}

#[tokio::test]
async fn test_burst_exhaustion_triggers_429_with_retry_after() {
    let (app, _) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: true,
        requests_per_minute: 60,
        burst_capacity: 10,
        anonymous_requests_per_minute: 60,
        anonymous_burst_capacity: 3,
        window_seconds: 60,
    });

    // Make 3 requests to exhaust the anonymous burst capacity (capacity = 3)
    for i in 0..3 {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/sources/detect")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-forwarded-for", "203.0.113.10")
            .body(Body::from(
                json!({
                    "urls": ["https://github.com/rust-lang/rust"]
                })
                .to_string(),
            ))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let rem: u32 = res
            .headers()
            .get(&HEADER_RATELIMIT_REMAINING)
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(rem, 2 - i);
    }

    // 4th request must be throttled with HTTP 429 Too Many Requests
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "203.0.113.10")
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

    // Verify Retry-After header
    let retry_after = res
        .headers()
        .get(header::RETRY_AFTER)
        .expect("Retry-After header must be present on 429 response")
        .to_str()
        .unwrap();
    let retry_secs: u64 = retry_after
        .parse()
        .expect("Retry-After must parse to a positive integer");
    assert!(retry_secs >= 1);

    // Verify rate limit exhaustion headers
    assert_eq!(
        res.headers()
            .get(&HEADER_RATELIMIT_LIMIT)
            .unwrap()
            .to_str()
            .unwrap(),
        "3"
    );
    assert_eq!(
        res.headers()
            .get(&HEADER_RATELIMIT_REMAINING)
            .unwrap()
            .to_str()
            .unwrap(),
        "0"
    );

    // Verify RFC 7807 problem details JSON payload
    let json_body = body_to_json(res.into_body()).await;
    assert_eq!(json_body["error"]["code"], "RATE_LIMIT_EXCEEDED");
    assert!(json_body["error"]["details"]["retry_after_seconds"].is_number());
}

#[tokio::test]
async fn test_exempt_endpoints_never_throttled() {
    let (app, _) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: true,
        requests_per_minute: 60,
        burst_capacity: 5,
        anonymous_requests_per_minute: 60,
        anonymous_burst_capacity: 2, // very small capacity
        window_seconds: 60,
    });

    // 1. Health check - 10 successive calls must all return 200
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 2. Readiness check - 10 successive calls must succeed
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/ready")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 3. Prometheus metrics - 10 successive calls must succeed
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/metrics")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 4. OpenAPI JSON specification - 10 successive calls must succeed
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/openapi.json")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 5. Root service metadata - 10 successive calls must succeed
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_client_ip_isolation() {
    let (app, _) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: true,
        requests_per_minute: 60,
        burst_capacity: 5,
        anonymous_requests_per_minute: 60,
        anonymous_burst_capacity: 2,
        window_seconds: 60,
    });

    let ip_a = "192.0.2.1";
    let ip_b = "192.0.2.2";

    // Exhaust quota for IP A
    for _ in 0..2 {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/sources/detect")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-forwarded-for", ip_a)
            .body(Body::from(
                json!({
                    "urls": ["https://github.com/rust-lang/rust"]
                })
                .to_string(),
            ))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // IP A's 3rd request is throttled
    let req_a_throttled = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", ip_a)
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();
    let res_a = app.clone().oneshot(req_a_throttled).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::TOO_MANY_REQUESTS);

    // IP B's request succeeds independently
    let req_b = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", ip_b)
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();
    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    assert_eq!(
        res_b
            .headers()
            .get(&HEADER_RATELIMIT_REMAINING)
            .unwrap()
            .to_str()
            .unwrap(),
        "1"
    );
}

#[tokio::test]
async fn test_authenticated_caller_tiered_quota_and_isolation() {
    let (app, state) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: true,
        requests_per_minute: 120,
        burst_capacity: 10,
        anonymous_requests_per_minute: 60,
        anonymous_burst_capacity: 2,
        window_seconds: 60,
    });

    // Create an active API key in storage
    let mut scopes = HashSet::new();
    scopes.insert(ApiKeyScope::Read);
    scopes.insert(ApiKeyScope::Write);
    let gen_key =
        GeneratedApiKey::generate(Uuid::new_v4(), "Active Tenant Tier Test Key", scopes, None)
            .unwrap();
    state
        .api_key_storage
        .save_key(&gen_key.record)
        .await
        .unwrap();
    let raw_token = gen_key.plaintext;

    // 1. Anonymous caller exhausts their burst quota of 2
    for _ in 0..2 {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/sources/detect")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-forwarded-for", "198.51.100.99")
            .body(Body::from(
                json!({
                    "urls": ["https://github.com/rust-lang/rust"]
                })
                .to_string(),
            ))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Anonymous 3rd request is rejected
    let req_anon_throttled = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "198.51.100.99")
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();
    let res_anon = app.clone().oneshot(req_anon_throttled).await.unwrap();
    assert_eq!(res_anon.status(), StatusCode::TOO_MANY_REQUESTS);

    // 2. Authenticated caller from the same IP passes and receives the higher burst limit (10)
    let auth_req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/sources/detect")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "198.51.100.99")
        .header(header::AUTHORIZATION, format!("Bearer {raw_token}"))
        .body(Body::from(
            json!({
                "urls": ["https://github.com/rust-lang/rust"]
            })
            .to_string(),
        ))
        .unwrap();

    let res_auth = app.clone().oneshot(auth_req).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);
    assert_eq!(
        res_auth
            .headers()
            .get(&HEADER_RATELIMIT_LIMIT)
            .unwrap()
            .to_str()
            .unwrap(),
        "10"
    );
    assert_eq!(
        res_auth
            .headers()
            .get(&HEADER_RATELIMIT_REMAINING)
            .unwrap()
            .to_str()
            .unwrap(),
        "9"
    );
}

#[tokio::test]
async fn test_rate_limiting_disabled_allows_unbounded_requests() {
    let (app, _) = setup_rate_limit_test_app(RateLimitConfig {
        enabled: false,
        requests_per_minute: 10,
        burst_capacity: 1,
        anonymous_requests_per_minute: 10,
        anonymous_burst_capacity: 1,
        window_seconds: 60,
    });

    // Make 10 requests even though burst capacity is configured to 1
    for _ in 0..10 {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/sources/detect")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "urls": ["https://github.com/rust-lang/rust"]
                })
                .to_string(),
            ))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
