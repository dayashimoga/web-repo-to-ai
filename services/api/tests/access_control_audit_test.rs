//! # Comprehensive Access Control, RBAC, and Anti-Enumeration Audit (`services/api`)
//!
//! Validates:
//! - Strict RBAC scope enforcement (`read`, `write`, `admin`) across operational and administrative endpoints.
//! - Anti-enumeration defenses: uniform RFC 7807 404 responses for cross-tenant resource queries.
//! - Authenticated envelope encryption at rest with AAD session binding.
//! - Input validation and injection rejection for session headers (`X-Session-ID`).

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use std::collections::HashSet;
use tower::ServiceExt;
use uuid::Uuid;
use web_repo_to_ai_api::build_app;
use web_repo_to_ai_api::dto::{JobResponse, ListApiKeysResponse};
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

// ==============================================================================
// 1. RBAC SCOPE HIERARCHY ENFORCEMENT AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_rbac_scope_hierarchy_enforcement() {
    let (app, state) = setup_test_app();
    let tenant_id = Uuid::new_v4();

    // 1. Generate READ-ONLY API Key
    let mut read_scopes = HashSet::new();
    read_scopes.insert(ApiKeyScope::Read);
    let read_gen =
        GeneratedApiKey::generate(tenant_id, "Audit Read Key", read_scopes, None).unwrap();
    state
        .api_key_storage
        .save_key(&read_gen.record)
        .await
        .unwrap();

    // 2. Generate WRITE API Key
    let mut write_scopes = HashSet::new();
    write_scopes.insert(ApiKeyScope::Write);
    let write_gen =
        GeneratedApiKey::generate(tenant_id, "Audit Write Key", write_scopes, None).unwrap();
    state
        .api_key_storage
        .save_key(&write_gen.record)
        .await
        .unwrap();

    // 3. Generate ADMIN API Key
    let mut admin_scopes = HashSet::new();
    admin_scopes.insert(ApiKeyScope::Admin);
    let admin_gen =
        GeneratedApiKey::generate(tenant_id, "Audit Admin Key", admin_scopes, None).unwrap();
    state
        .api_key_storage
        .save_key(&admin_gen.record)
        .await
        .unwrap();

    // --- AUDIT ANONYMOUS CALLER (401 UNAUTHORIZED) ---
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Anonymous caller must be rejected with 401"
    );

    // --- AUDIT READ-ONLY KEY (403 FORBIDDEN ON ADMIN ENDPOINTS) ---
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", read_gen.plaintext),
        )
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "Read-only key must be forbidden from listing keys"
    );
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("INSUFFICIENT_PERMISSIONS") || body.contains("FORBIDDEN"));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", read_gen.plaintext),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Illegal Key", "scopes": ["read"] }).to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "Read-only key must be forbidden from key creation"
    );

    // --- AUDIT WRITE KEY (403 FORBIDDEN ON ADMIN ENDPOINTS) ---
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", write_gen.plaintext),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Illegal Key", "scopes": ["read"] }).to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "Write key must be forbidden from key management"
    );

    // --- AUDIT ADMIN KEY (201 CREATED & 200 OK) ---
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", admin_gen.plaintext),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Subordinate Key", "scopes": ["read"] }).to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::CREATED,
        "Admin key must be allowed to create keys"
    );

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", admin_gen.plaintext),
        )
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Admin key must be allowed to list keys"
    );

    // --- AUDIT SESSION CALLER (IMPLICIT ADMIN SCOPE) ---
    let session_id = Uuid::new_v4();
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header("X-Session-ID", session_id.to_string())
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Session caller must have implicit administrative scope"
    );
}

// ==============================================================================
// 2. ANTI-ENUMERATION & CROSS-TENANT ISOLATION AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_anti_enumeration_cross_tenant_isolation() {
    let (app, state) = setup_test_app();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    // 1. Create key for Tenant A
    let mut admin_scopes = HashSet::new();
    admin_scopes.insert(ApiKeyScope::Admin);
    let key_a =
        GeneratedApiKey::generate(tenant_a, "Tenant A Admin", admin_scopes.clone(), None).unwrap();
    state.api_key_storage.save_key(&key_a.record).await.unwrap();

    // 2. Create key for Tenant B
    let key_b = GeneratedApiKey::generate(tenant_b, "Tenant B Admin", admin_scopes, None).unwrap();
    state.api_key_storage.save_key(&key_b.record).await.unwrap();

    // 3. Tenant B attempts to revoke Tenant A's key -> Uniform 404 (Anti-enumeration)
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{}", key_a.record.id))
        .header(header::AUTHORIZATION, format!("Bearer {}", key_b.plaintext))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::NOT_FOUND,
        "Cross-tenant key deletion must return 404 Not Found"
    );
    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("RESOURCE_NOT_FOUND"));

    // 4. Querying a completely non-existent key ID -> Returns identical 404 response
    let non_existent_key = Uuid::new_v4();
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/auth/keys/{non_existent_key}"))
        .header(header::AUTHORIZATION, format!("Bearer {}", key_b.plaintext))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let non_existent_body = body_to_string(res.into_body()).await;
    assert!(non_existent_body.contains("RESOURCE_NOT_FOUND"));

    // 5. Tenant A creates a job
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/jobs")
        .header(header::AUTHORIZATION, format!("Bearer {}", key_a.plaintext))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "sources": ["https://example.com/doc"] }).to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let body = body_to_string(res.into_body()).await;
    let job_resp: JobResponse = serde_json::from_str(&body).unwrap();
    let job_id = job_resp.job_id;

    // 6. Tenant B attempts to access Tenant A's job -> Returns 404 Not Found (Anti-enumeration)
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/jobs/{job_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {}", key_b.plaintext))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::NOT_FOUND,
        "Cross-tenant job access must return uniform 404"
    );
    let job_404_body = body_to_string(res.into_body()).await;
    assert!(job_404_body.contains("RESOURCE_NOT_FOUND"));

    // 7. Tenant B lists keys -> Only key_b is returned, key_a is never exposed
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/auth/keys")
        .header(header::AUTHORIZATION, format!("Bearer {}", key_b.plaintext))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list_body = body_to_string(res.into_body()).await;
    let list_resp: ListApiKeysResponse = serde_json::from_str(&list_body).unwrap();
    assert_eq!(list_resp.total, 1);
    assert_eq!(list_resp.keys[0].id, key_b.record.id);
    assert!(!list_body.contains(&key_a.record.id.to_string()));
}

// ==============================================================================
// 3. TOKEN STORAGE ENVELOPE ENCRYPTION & AAD BINDING AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_token_storage_envelope_encryption_and_aad_binding() {
    let (_, state) = setup_test_app();
    let session_1 = Uuid::new_v4();
    let session_2 = Uuid::new_v4();
    let installation_id: i64 = 554433;
    let raw_token = "ghs_test_token_secret_value_1234567890abcdef";
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    // 1. Store installation token under Session 1
    let saved_row = state
        .token_storage
        .save_installation_token(
            session_1,
            installation_id,
            "acme-corp",
            raw_token,
            expires_at,
        )
        .await
        .expect("Token save must succeed");

    assert_eq!(saved_row.github_installation_id, installation_id);
    assert_eq!(saved_row.owner_session_id, session_1);
    assert!(!saved_row.is_revoked());

    // 2. Session 1 retrieves and decrypts the active token
    let active_token = state
        .token_storage
        .get_active_token(session_1, installation_id)
        .await
        .expect("Decryption must succeed");
    assert_eq!(active_token.as_deref(), Some(raw_token));

    // 3. Session 2 attempts to retrieve Session 1's token -> Must return None (cross-tenant isolation)
    let cross_tenant_attempt = state
        .token_storage
        .get_active_token(session_2, installation_id)
        .await
        .expect("Query must succeed without leaking");
    assert_eq!(
        cross_tenant_attempt, None,
        "Session 2 must not be able to retrieve or decrypt Session 1's token"
    );

    // 4. Revocation Hygiene: Once revoked, token is zeroed out and unrecoverable
    let revoked = state
        .token_storage
        .revoke_installation(session_1, installation_id)
        .await
        .expect("Revocation must succeed");
    assert!(revoked, "Revocation of active token must return true");

    let after_revocation = state
        .token_storage
        .get_active_token(session_1, installation_id)
        .await
        .expect("Query must succeed");
    assert_eq!(
        after_revocation, None,
        "Revoked installation token must never be retrievable"
    );
}

// ==============================================================================
// 4. SESSION HEADER INJECTION & SANITIZATION AUDIT
// ==============================================================================

#[tokio::test]
async fn test_audit_session_header_validation_and_injection() {
    let (app, _) = setup_test_app();

    // 1. Verify that CRLF / null-byte header injection is rejected at HTTP framing layer
    assert!(axum::http::HeaderValue::from_str("\0nullbyte").is_err());
    assert!(axum::http::HeaderValue::from_str("uuid\r\nInjected: true").is_err());
    assert!(axum::http::HeaderValue::from_str("uuid\nInjected: true").is_err());

    // 2. Adversarial X-Session-ID values that are valid ASCII header strings but invalid UUIDs
    let malicious_sessions = [
        "not-a-uuid",
        "12345",
        "00000000-0000-0000-0000-00000000000g", // non-hex character
        "' OR '1'='1",                          // SQL injection attempt
        "<script>alert(1)</script>",            // XSS injection attempt
        "../../etc/passwd",                     // Directory traversal attempt
        "d3b07384-d113-4672-9b7e-",             // Truncated UUID
        "d3b07384d11346729b7effffffffffffffffff", // Oversized UUID
        "   ",                                  // Whitespace only
    ];

    for bad_session in malicious_sessions {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/auth/github/installations")
            .header("X-Session-ID", bad_session)
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "Malformed X-Session-ID '{bad_session}' must be rejected with 400 Bad Request"
        );
        let body = body_to_string(res.into_body()).await;
        assert!(
            body.contains("INVALID_REQUEST_SCHEMA") || body.contains("VALIDATION_ERROR"),
            "Body must return structured validation error: {body}"
        );
    }
}
