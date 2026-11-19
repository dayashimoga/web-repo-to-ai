//! Integration tests for API key authentication, permission scopes, and tenant isolation.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;
use web_repo_to_ai_api::build_app;
use web_repo_to_ai_api::dto::{CreateApiKeyResponse, ListApiKeysResponse, RevokeApiKeyResponse};
use web_repo_to_ai_api::state::AppState;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_core::AppConfig;

fn setup_test_app() -> (axum::Router, AppState) {
    let config = AppConfig::default();
    let state = AppState::new(config);
    let app = build_app(state.clone());
    (app, state)
}

async fn body_to_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn test_anonymous_requests_on_key_management_rejected_with_401() {
    let (app, _) = setup_test_app();

    // 1. POST /api/v1/auth/keys without auth
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Test Key",
                "scopes": ["read", "write"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("UNAUTHORIZED"));

    // 2. GET /api/v1/auth/keys without auth
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. DELETE /api/v1/auth/keys/:id without auth
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{}", Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_malformed_authorization_headers_rejected() {
    let (app, _) = setup_test_app();

    // 1. Invalid scheme
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("Invalid Authorization header format"));

    // 2. Empty bearer token
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, "Bearer    ")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. Invalid API key token
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, "Bearer wr_live_invalid_nonexistent")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_key_lifecycle_creation_listing_and_revocation() {
    let (app, _) = setup_test_app();
    let tenant_id = Uuid::new_v4();

    // 1. Create API key using session header
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Session-ID", tenant_id.to_string())
        .body(Body::from(
            json!({
                "name": "Production Agent 1",
                "scopes": ["read", "write"],
                "expires_in_days": 30
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = body_to_string(res.into_body()).await;
    let create_resp: CreateApiKeyResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(create_resp.name, "Production Agent 1");
    assert_eq!(create_resp.tenant_id, tenant_id);
    assert!(create_resp.key.starts_with("wr_live_"));
    assert_eq!(create_resp.scopes, vec!["read", "write"]);
    assert!(create_resp.expires_at.is_some());

    let key_id = create_resp.id;
    let raw_key = create_resp.key;

    // 2. List API keys using session header
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header("X-Session-ID", tenant_id.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body_to_string(res.into_body()).await;
    let list_resp: ListApiKeysResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(list_resp.total, 1);
    assert_eq!(list_resp.keys[0].id, key_id);
    assert_eq!(list_resp.keys[0].name, "Production Agent 1");
    assert!(list_resp.keys[0].is_active);

    // 3. Try to use key without admin scope to manage keys (fails with 403 Forbidden)
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Subkey",
                "scopes": ["read"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("FORBIDDEN"));

    // 4. Revoke key using session header
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{key_id}"))
        .header("X-Session-ID", tenant_id.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_to_string(res.into_body()).await;
    let revoke_resp: RevokeApiKeyResponse = serde_json::from_str(&body).unwrap();
    assert!(revoke_resp.revoked);

    // 5. Subsequent request with revoked key fails with 401
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("revoked"));
}

#[tokio::test]
async fn test_tenant_isolation_in_key_listing_and_revocation() {
    let (app, _) = setup_test_app();
    let tenant_1 = Uuid::new_v4();
    let tenant_2 = Uuid::new_v4();

    // 1. Create key for Tenant 1
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Session-ID", tenant_1.to_string())
        .body(Body::from(
            json!({
                "name": "Tenant 1 Key",
                "scopes": ["admin"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_to_string(res.into_body()).await;
    let tenant_1_key: CreateApiKeyResponse = serde_json::from_str(&body).unwrap();

    // 2. Tenant 2 lists keys -> should return 0 keys
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header("X-Session-ID", tenant_2.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_to_string(res.into_body()).await;
    let list_resp: ListApiKeysResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(list_resp.total, 0);

    // 3. Tenant 2 attempts to delete Tenant 1's key -> rejected with 404 Not Found
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{}", tenant_1_key.id))
        .header("X-Session-ID", tenant_2.to_string())
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_api_key_can_create_and_manage_keys() {
    let (app, state) = setup_test_app();
    let tenant_id = Uuid::new_v4();

    // Directly seed an Admin API key for tenant
    let mut scopes = std::collections::HashSet::new();
    scopes.insert(ApiKeyScope::Admin);
    let admin_gen = GeneratedApiKey::generate(tenant_id, "Master Admin Key", scopes, None).unwrap();
    state
        .api_key_storage
        .save_key(&admin_gen.record)
        .await
        .unwrap();

    // Use Admin API key Bearer token to generate a worker key
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", admin_gen.plaintext),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Worker Key Created by Admin Key",
                "scopes": ["read", "write"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_to_string(res.into_body()).await;
    let worker_key: CreateApiKeyResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(worker_key.tenant_id, tenant_id);
    assert_eq!(worker_key.name, "Worker Key Created by Admin Key");

    // Use Admin API key to list keys
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", admin_gen.plaintext),
        )
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_to_string(res.into_body()).await;
    let list_resp: ListApiKeysResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(list_resp.total, 2);
}
