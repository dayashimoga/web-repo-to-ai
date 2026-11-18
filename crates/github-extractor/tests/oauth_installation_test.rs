//! # GitHub App OAuth & Installation Flow Tests
//!
//! Validates:
//! - Cryptographic OAuth state nonce generation, TTL expiration, and constant-time equality.
//! - Single-use replay protection and session binding in [`InMemoryOAuthStateStore`].
//! - Authorization and installation URL generation.
//! - Sensitive token secret masking in Debug/Display for OAuth and installation tokens.
//! - RS256 JWT generation from PKCS#1 and PKCS#8 RSA private keys.
//! - Full mock HTTP integration of OAuth code exchange and installation APIs.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    build_app_installation_url, build_oauth_authorize_url, decode_base64, generate_app_jwt_at_time,
    GitHubApiClient, GitHubAppPermissions, GitHubInstallationToken, GitHubOAuthTokenResponse,
    InMemoryOAuthStateStore, OAuthState, OAuthStateStore, RepositorySelection,
};

const TEST_PKCS1_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
MIIEpAIBAAKCAQEAmLRSiTwPUZeG6OvDJwFxxXQ7fGSnhu2hYNQSq0Mm1jBbbdle\n\
BQDC+m292l2Za5AJ9ldZPbCV3yMKpPgUeeAProSnXPTro2LpdJX/lnnTBJh9bxOC\n\
zmGJMDdLsjyVEX2CDMeyTor7JcKYtQ6u+VilVeVcBs9LpvDkbKqEIPbN+d3sEwLB\n\
TfqryHgbTRIJWguK7t8602RJTWfsf7QkhU+OSEdeojsZkl6N9/+kcC9cMhSue1Yc\n\
1p2v65Y9UX6NqTyT3ObqEAV2tI7JaKTNLcyHSmBol7fSbDEOxJAtCB4lKtRzo1R0\n\
Vfv6VbH9LB9R07g5wdZFw/6jUnIXwC3K0d6jJwIDAQABAoIBAGVF8hchxecO32JK\n\
Obr8sj+JvonZF2Hc51IYU+lRN8iFeZ14s2oN/0Gm1mqj+MPlWzYM6BqJqlu6Ga0e\n\
+jV56+yJ/ZUgucY6Kyou8LsTQGej2kx4XCATf7CCe6lRlGttGXrwVcjRU47PaAds\n\
HY8wwzYcTtQVVs74pAWNcOOLyz8ScOL4pXFI+VX4X5MMsdn2fejDJ15E30NA3Jao\n\
I57fLsZvWj5vooO+K4aWD81lwwEcJ0Qo/+JjE0Ty+s0Q2i3/c21vtvLd23LqgzAs\n\
Li5cZHByUM5CrAjtEm6g5eMHn3fOaXvUV5TfqHr/W0CbfpKpGQqsraH/6Tmof2w4\n\
GlrE/8ECgYEAx0J2gEG3nMzVGbM0IfQujffkmk1wAXL41r3IWVvrPbVoxYzW58lL\n\
LCtl/dF2nHeUY9zqBVthMwX/v3NEFC3rUhhYi2VDnl7E6PytKeF/KxugwaQnMUtb\nsW1UZkPUIb491cNgyHwZJdPP0djzNL3Vy9HCoEDPfBc/b/XJJkq0ZzMCgYEAxDAY\n\
2yoM/0BTq+rvKvmguR3810SHBnNiCKk/738h/AIhLHx6SbPGEDUaD5TvZ7dNTlbk\n\
sZbUspDUjKnPYF+yIEjBO6ReJnRhkLoYWTYYAR6MAHmZbV6x3SdvTGvoXf26j628\n\
PRz9F1quWSAOGnUpdroXXf4uMR35QfALXBalxD0CgYEAr7Sdxg8tF+lPaHArYl1t\n\
x+Z2ANKUF9MINmP3jUSOtUWIc9rzjkRkMzi5oQ3Zp7OSeFTPT5X6qgF6OZosyBaN\n\
0UCCo7jR3Y731/Mnxyx+9MZSBS75GHC5FEWyDNN29aJJ6Ht2lpNHoaGbOhIys6lK\n\
I4Z+Ns2vqF7fo5x6mXd6iR0CgYAjPjSo4lP1YpvlTU3rnW5W8uEh1qie6wlYXvci\n\
UedL+yH0eOQK62WQMcndtLaRHiegBBIPKPZKoaPdJQAg4u//xrIXwb48EDoaoqOP\n\
Gx2Szeb+itmaASHYhaYDx3q1r8fc2avwGo4IrPyW/CAsdpGdR269LtUjc6v8QV/5\n\
CgR7TQKBgQC5FDoq/cD+1LIVbunErGmeguY+6CDQ5XLeDOw6zwe08lavBpXL/UWI\n\
+mKGwoUo9TCXLe9J1y0E0zlwUY23+jZ37UPJ7PHcF8hBsEjNdMsOdqYE2P/C475B\n\
RcqVsKc2U0SH0vgS8z0YQP8Hehn7/7mq0SzYV68sCNmKMfR483HBRg==\n\
-----END RSA PRIVATE KEY-----";

const TEST_PKCS8_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDuXcICii5M19pB\n\
cWL0U0iaaGDba2VTktwEb3OZ/ckRoOmCTI5XFxs8igg8TCoor0tQAo7Q30f3AW2K\n\
CH62+6vpKdS2TTWSXGA5r/3QaNLxHqRomQ2cmmx1ikUs2x3ACITkJfVniJ1ijoG4\n\
/TSC63pOYWU6+yWlZkyJll+scRkyRyBZj9AfRwuzeDathZGKapM0GW6UQYpl+qWt\n\
J8ILTm4ij44JiTqcbMPnbFCyc6+DpPnNeFdSjkPRnqIieB1Mi0q3J9ODdCFc5ayn\n\
GQLazca48sJYUyLJ2nJhGwNDzpfw4kQjdTiv4zApuVtMZynaJjMTIlAyOp4Zm9DM\n\
IeB86ZO9AgMBAAECggEACFnQijY9GQQW6WQHi3e/e2lcu1Mv4ZWrj5w3+JUjjjKM\n\
+7ui4Po2QhCz6Sjh8Try8IKmbjrkUosCQ2u4xMBcbXbBtQFPdyk1VdAMnFiPy1K/\n\
f8qc53ehhRyq/79Nc4TY4wVRa6clnGZRtSFpzYxiZIyU91QFJZ5H4Cocf/YWY+R4\n\
toTq5Os+EYqHZh47uJ/s9DDvVxSDLcgQcaFCkZ4yrG+9CHEHj5l2314IG+9ibW3c\n\
kOSw8VzLgpncQZJKTYzDwjeeR/KSpMfIKnqwVVu+afJC0V1NZoQT0INRfPbvH/gF\n\
Qu+z484MvfbibAIddaeQ7o5GtQDuViruxaedbPyUAQKBgQD43C9DnEjc6t+BOXKv\n\
LAtW0ZtKD654PIqJGeXhRXZm5w5o8ryBp6x47/yUwoHBdI45SBrKI8xhAkym2bPJ\n\
Y/E7g5CN0/0eA9Z+oIr1p4qd47fMMgA0iiOQrGl/yGHwS3zGgZWNoCrPeSyI3dYk\n\
xvqkIoyz3kjPgsRFgabdOHcUnQKBgQD1NH+cXNhi4J0QXSVpL5UGaBR+m1Yp/V54\n\
QsniPGR7yfQMF+hfr/QyykpUP2NlDGF010m5G2LKyydLLitdvP+ZkqgqFQ5cnl0f\n\
gHBNawC7YE4N1D9x1kuVYA0qQIGbR9VMPk9MHqrv2hGvUmi3f26da8RMk1Dgh4lj\n\
+mGxlWUBoQKBgQDtnNboGF4dyYkUirSs+0HsaIB8a5KrrSqdk4PB3yAZSr6giDxj\n\
E6UDaClkJH/eJhryKJ2jqiCn85kaqr0WKn4X1Bv+v41rzkBMcZ+cxJb/AIHqz0cQ\n\
42Dn7ydgYgTZekouA2a3blnR9aIHIMEqJm8ErxRnXXNp5zLeJtgbW6GfpQKBgFbs\n\
+oQzAd7jo6qGatnELX9s7D8J0FzoXlIh0zHpmPGtYV/cJduuI9LRldIR+ql63x2t\n\
qIv/omEj7dUgE+2SL8aAkC8C8elxlq+xN56qXUMP2/vZD3jKsFbPCzu8THYA3CPA\n\
PxpMEOsgkn5lp4vNmCgkaR2AZ/6A7frMa0+qF3mhAoGAN6PpsKudPiaiaFKC0d3y\n\
03caFBwlVuwQJHeMkTsAOTxYEsg7/Wrm493DVqLNiRX/jukfisTSRODT7yi1X3xN\n\
k5QL4v0Ipj2JdivK9Oq5pE88ToQb53qddSmUIq6uwCUxDO9CCIXRELxO7EvVmqOh\n\
7VMLOGnvxgv2Jm33r8tIRSk=\n\
-----END PRIVATE KEY-----";

#[test]
fn test_oauth_state_generation_and_properties() {
    let state = OAuthState::new()
        .with_session_id("sess_user_alpha_42")
        .with_tenant_id("tenant_enterprise_99")
        .with_redirect_uri("https://app.example.com/callback");

    assert!(!state.nonce.is_empty());
    assert_eq!(state.nonce.len(), 64); // BLAKE3 hex representation is 64 chars (256 bits)
    assert_eq!(state.session_id.as_deref(), Some("sess_user_alpha_42"));
    assert_eq!(state.tenant_id.as_deref(), Some("tenant_enterprise_99"));
    assert_eq!(
        state.redirect_uri.as_deref(),
        Some("https://app.example.com/callback")
    );
    assert!(!state.is_expired());
    assert!(state.expires_at > state.created_at);
}

#[test]
fn test_oauth_state_constant_time_comparison() {
    let state = OAuthState::new();

    // Positive match
    assert!(state.constant_time_matches(&state.nonce));

    // Tampered nonce
    let mut tampered = state.nonce.clone();
    tampered.replace_range(0..1, if tampered.starts_with('a') { "b" } else { "a" });
    assert!(!state.constant_time_matches(&tampered));

    // Length mismatch
    assert!(!state.constant_time_matches("short"));
}

#[test]
fn test_in_memory_oauth_state_store_save_and_single_use() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("sess_1");
    let nonce = state.nonce.clone();

    store.save_state(state).expect("Save failed");
    assert_eq!(store.active_count(), 1);

    // First consumption succeeds
    let consumed = store
        .validate_and_consume(&nonce, Some("sess_1"))
        .expect("First consume should succeed");
    assert_eq!(consumed.nonce, nonce);
    assert_eq!(store.active_count(), 0);

    // Replay attempt must fail
    let replay_err = store
        .validate_and_consume(&nonce, Some("sess_1"))
        .expect_err("Replay must fail");
    assert!(replay_err.to_string().contains("already consumed"));
}

#[test]
fn test_in_memory_oauth_state_store_session_binding() {
    let mut store = InMemoryOAuthStateStore::new();
    let state = OAuthState::new().with_session_id("valid_session_owner");
    let nonce = state.nonce.clone();

    store.save_state(state).expect("Save failed");

    // Attempting to consume with attacker session ID must fail
    let mismatch_err = store
        .validate_and_consume(&nonce, Some("attacker_impersonator_session"))
        .expect_err("Session mismatch must fail");
    assert!(mismatch_err.to_string().contains("session mismatch"));
}

#[test]
fn test_in_memory_oauth_state_store_prunes_expired() {
    let mut store = InMemoryOAuthStateStore::new();
    // Create state that expired 1 second ago
    let expired_state = OAuthState::with_ttl(Duration::seconds(-1));
    let nonce = expired_state.nonce.clone();

    store.save_state(expired_state).expect("Save failed");

    // Prune should purge it
    store.prune_expired();
    assert_eq!(store.active_count(), 0);

    // Validation should report not found
    assert!(store.validate_and_consume(&nonce, None).is_err());
}

#[test]
fn test_oauth_authorize_and_installation_url_builders() {
    let auth_url = build_oauth_authorize_url(
        "Iv1.test_client_id_123",
        Some("https://app.com/api/v1/github/callback"),
        "state_nonce_abc",
        false,
        None,
    )
    .expect("Auth URL build failed");

    assert!(auth_url.starts_with("https://github.com/login/oauth/authorize?"));
    assert!(auth_url.contains("client_id=Iv1.test_client_id_123"));
    assert!(auth_url.contains("state=state_nonce_abc"));
    assert!(auth_url.contains("allow_signup=false"));

    let install_url = build_app_installation_url("web-repo-ai", Some("state_install_xyz"), None)
        .expect("Install URL build failed");
    assert_eq!(
        install_url,
        "https://github.com/apps/web-repo-ai/installations/new?state=state_install_xyz"
    );
}

#[test]
fn test_oauth_token_response_secret_masking() {
    let secret_access = "ghu_1234567890abcdefghijklmnopqrstuvwxyz";
    let secret_refresh = "ghr_0987654321zyxwvutsrqponmlkjihgfedcba";

    let token_resp = GitHubOAuthTokenResponse {
        access_token: secret_access.to_string(),
        token_type: "bearer".to_string(),
        scope: "read:user".to_string(),
        expires_in: Some(28800),
        refresh_token: Some(secret_refresh.to_string()),
        refresh_token_expires_in: Some(15552000),
    };

    let debug_str = format!("{token_resp:?}");
    let display_str = format!("{token_resp}");

    assert!(!debug_str.contains(secret_access));
    assert!(!debug_str.contains(secret_refresh));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));

    assert!(!display_str.contains(secret_access));
    assert!(!display_str.contains(secret_refresh));
}

#[test]
fn test_installation_token_secret_masking_and_expiration() {
    let secret_token = "ghs_installation_access_token_super_secret_99";
    let expires = Utc::now() + Duration::minutes(60);

    let inst_token = GitHubInstallationToken {
        token: secret_token.to_string(),
        expires_at: expires,
        permissions: std::collections::HashMap::new(),
        repository_selection: RepositorySelection::Selected,
        repositories: None,
    };

    let debug_str = format!("{inst_token:?}");
    let display_str = format!("{inst_token}");

    assert!(!debug_str.contains(secret_token));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));
    assert!(!display_str.contains(secret_token));

    assert!(!inst_token.is_expired());
    assert!(inst_token.is_expiring_soon(3700));
    assert!(!inst_token.is_expiring_soon(60));
}

#[test]
fn test_rs256_jwt_generation_from_pkcs1_and_pkcs8() {
    let now = Utc::now();

    // 1. Generate JWT using PKCS#1 RSA private key
    let jwt_pkcs1 = generate_app_jwt_at_time(102030, TEST_PKCS1_PEM, now)
        .expect("PKCS#1 JWT generation failed");
    let parts_pkcs1: Vec<&str> = jwt_pkcs1.split('.').collect();
    assert_eq!(
        parts_pkcs1.len(),
        3,
        "JWT must contain 3 dot-separated parts"
    );

    // Verify header decodes to RS256 JWT
    let header_bytes = decode_base64(&parts_pkcs1[0].replace('-', "+").replace('_', "/"))
        .or_else(|_| {
            decode_base64(&format!(
                "{}=",
                parts_pkcs1[0].replace('-', "+").replace('_', "/")
            ))
        })
        .or_else(|_| {
            decode_base64(&format!(
                "{}==",
                parts_pkcs1[0].replace('-', "+").replace('_', "/")
            ))
        })
        .expect("Decode header base64 failed");
    let header_json: serde_json::Value =
        serde_json::from_slice(&header_bytes).expect("Header JSON parse failed");
    assert_eq!(header_json["alg"], "RS256");
    assert_eq!(header_json["typ"], "JWT");

    // 2. Generate JWT using PKCS#8 RSA private key
    let jwt_pkcs8 = generate_app_jwt_at_time(405060, TEST_PKCS8_PEM, now)
        .expect("PKCS#8 JWT generation failed");
    let parts_pkcs8: Vec<&str> = jwt_pkcs8.split('.').collect();
    assert_eq!(
        parts_pkcs8.len(),
        3,
        "JWT must contain 3 dot-separated parts"
    );
}

// ---------------------------------------------------------------------------
// Mock HTTP Server & End-to-End Client Integration
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MockOAuthBody {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: Option<String>,
}

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn mock_oauth_exchange(Json(body): Json<MockOAuthBody>) -> impl IntoResponse {
    if body.code == "invalid_code" {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "error": "bad_verification_code",
                "error_description": "The code passed is incorrect or expired.",
                "error_uri": "https://docs.github.com"
            })),
        );
    }

    assert_eq!(body.client_id, "mock_client_id_1");
    assert_eq!(body.client_secret, "mock_client_secret_1");
    assert_eq!(body.code, "valid_code_123");
    assert_eq!(
        body.redirect_uri.as_deref(),
        Some("https://app.com/callback")
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "access_token": "ghu_mock_user_access_token_999",
            "token_type": "bearer",
            "scope": "read:user",
            "expires_in": 28800,
            "refresh_token": "ghr_mock_refresh_token_888",
            "refresh_token_expires_in": 15552000
        })),
    )
}

async fn mock_list_user_installations(Query(query): Query<PaginationQuery>) -> impl IntoResponse {
    assert_eq!(query.page, Some(1));
    assert_eq!(query.per_page, Some(10));

    Json(serde_json::json!({
        "total_count": 1,
        "installations": [{
            "id": 881234,
            "account": {
                "id": 991,
                "login": "acme-corp",
                "type": "Organization",
                "avatar_url": "https://avatars.githubusercontent.com/u/991",
                "html_url": "https://github.com/acme-corp"
            },
            "repository_selection": "selected",
            "access_tokens_url": "https://api.github.com/app/installations/881234/access_tokens",
            "repositories_url": "https://api.github.com/installation/repositories",
            "html_url": "https://github.com/settings/installations/881234",
            "app_id": 5544,
            "app_slug": "web-repo-to-ai",
            "target_id": 991,
            "target_type": "Organization",
            "permissions": {
                "contents": "read",
                "metadata": "read"
            },
            "events": ["installation", "installation_repositories"],
            "created_at": "2026-11-18T10:00:00Z",
            "updated_at": "2026-11-18T10:00:00Z"
        }]
    }))
}

async fn mock_get_installation(Path(id): Path<u64>) -> impl IntoResponse {
    assert_eq!(id, 881234);

    Json(serde_json::json!({
        "id": 881234,
        "account": {
            "id": 991,
            "login": "acme-corp",
            "type": "Organization",
            "html_url": "https://github.com/acme-corp"
        },
        "repository_selection": "selected",
        "access_tokens_url": "https://api.github.com/app/installations/881234/access_tokens",
        "repositories_url": "https://api.github.com/installation/repositories",
        "html_url": "https://github.com/settings/installations/881234",
        "app_id": 5544,
        "app_slug": "web-repo-to-ai",
        "target_id": 991,
        "target_type": "Organization",
        "permissions": {
            "contents": "read",
            "metadata": "read"
        },
        "events": ["installation"],
        "created_at": "2026-11-18T10:00:00Z",
        "updated_at": "2026-11-18T10:00:00Z"
    }))
}

async fn mock_create_installation_access_token(Path(id): Path<u64>) -> impl IntoResponse {
    assert_eq!(id, 881234);

    Json(serde_json::json!({
        "token": "ghs_installation_access_token_mock_111",
        "expires_at": "2026-11-18T12:00:00Z",
        "permissions": {
            "contents": "read",
            "metadata": "read"
        },
        "repository_selection": "selected"
    }))
}

async fn mock_list_installation_repositories() -> impl IntoResponse {
    Json(serde_json::json!({
        "total_count": 2,
        "repository_selection": "selected",
        "repositories": [
            {
                "id": 1001,
                "node_id": "R_kgDO1001",
                "name": "backend-core",
                "full_name": "acme-corp/backend-core",
                "private": true,
                "default_branch": "main",
                "description": "Core backend service",
                "html_url": "https://github.com/acme-corp/backend-core"
            },
            {
                "id": 1002,
                "node_id": "R_kgDO1002",
                "name": "frontend-web",
                "full_name": "acme-corp/frontend-web",
                "private": true,
                "default_branch": "main",
                "description": "Web client",
                "html_url": "https://github.com/acme-corp/frontend-web"
            }
        ]
    }))
}

#[tokio::test]
async fn test_live_mock_http_oauth_and_installation_flow() {
    let app = Router::new()
        .route("/login/oauth/access_token", post(mock_oauth_exchange))
        .route("/user/installations", get(mock_list_user_installations))
        .route("/app/installations/:id", get(mock_get_installation))
        .route(
            "/app/installations/:id/access_tokens",
            post(mock_create_installation_access_token),
        )
        .route(
            "/installation/repositories",
            get(mock_list_installation_repositories),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral mock port");
    let addr: SocketAddr = listener.local_addr().expect("Local addr failed");
    let server_url = format!("http://{addr}");

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = GitHubApiClient::new().with_base_url_override(server_url);

    // 1. Exchange OAuth code
    let token_resp = client
        .exchange_oauth_code(
            "mock_client_id_1",
            "mock_client_secret_1",
            "valid_code_123",
            Some("https://app.com/callback"),
        )
        .await
        .expect("OAuth code exchange failed");
    assert_eq!(token_resp.access_token(), "ghu_mock_user_access_token_999");
    assert_eq!(token_resp.token_type, "bearer");
    assert_eq!(token_resp.expires_in, Some(28800));

    // 1b. Error exchange
    let err_resp = client
        .exchange_oauth_code(
            "mock_client_id_1",
            "mock_client_secret_1",
            "invalid_code",
            None,
        )
        .await
        .expect_err("Expected error code exchange to fail");
    assert!(err_resp.to_string().contains("bad_verification_code"));

    // 2. List user installations
    let installations = client
        .list_user_installations(token_resp.access_token(), Some(1), Some(10))
        .await
        .expect("List user installations failed");
    assert_eq!(installations.total_count, 1);
    let inst = &installations.installations[0];
    assert_eq!(inst.id, 881234);
    assert_eq!(inst.account.login, "acme-corp");
    assert!(inst.is_organization());
    assert!(!inst.is_suspended());

    // 3. Get installation details via App JWT
    let app_jwt = "mock_app_jwt_for_testing";
    let single_inst = client
        .get_installation(881234, app_jwt)
        .await
        .expect("Get installation failed");
    assert_eq!(single_inst.id, 881234);
    assert_eq!(single_inst.account.login, "acme-corp");

    // 4. Request installation access token
    let permissions = GitHubAppPermissions::minimal_read_only();
    let inst_token = client
        .create_installation_access_token(
            881234,
            app_jwt,
            Some(&["backend-core".to_string()]),
            Some(&permissions),
        )
        .await
        .expect("Create installation token failed");
    assert_eq!(inst_token.token(), "ghs_installation_access_token_mock_111");
    assert_eq!(
        inst_token.repository_selection,
        RepositorySelection::Selected
    );

    // 5. List installation repositories
    let repos_resp = client
        .list_installation_repositories(inst_token.token(), Some(1), Some(10))
        .await
        .expect("List repositories failed");
    assert_eq!(repos_resp.total_count, 2);
    assert_eq!(repos_resp.repositories.len(), 2);
    assert_eq!(repos_resp.repositories[0].name, "backend-core");
    assert_eq!(
        repos_resp.repositories[0].full_name,
        "acme-corp/backend-core"
    );
    assert!(repos_resp.repositories[0].private);
    assert_eq!(repos_resp.repositories[1].name, "frontend-web");

    server_handle.abort();
}
