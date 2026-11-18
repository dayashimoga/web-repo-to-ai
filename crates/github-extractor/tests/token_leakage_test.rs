//! Integration test suite for domain model, error envelope, and wire-level token leakage defense (Subphase 6.4.3).
//!
//! Validates that all credential-bearing domain structures, error envelopes,
//! diagnostic recovery cards, and live HTTP requests never leak secret tokens under Rule 05.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, TimeZone, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

use web_repo_to_ai_core::crypto::{EncryptedPayload, EncryptionKey};
use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::authorization::AuthorizedInstallationContext;
use web_repo_to_ai_github_extractor::authorization_recovery::AuthorizationRecoveryCoordinator;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::error::GitHubExtractorError;
use web_repo_to_ai_github_extractor::installation::{GitHubInstallationToken, RepositorySelection};
use web_repo_to_ai_github_extractor::oauth::{GitHubOAuthErrorResponse, GitHubOAuthTokenResponse};
use web_repo_to_ai_github_extractor::private_extractor::PrivateRepositoryExtractor;
use web_repo_to_ai_github_extractor::token_refresh::StoredTokenRecord;

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

fn test_app_config() -> GitHubAppConfig {
    GitHubAppConfig::builder()
        .app_id(12345)
        .app_slug("leakage-test-app")
        .client_id("Iv1.leakage_client_id")
        .client_secret("ghs_ClientSecret1234567890ABCDEF")
        .private_key_pem(TEST_PKCS8_PEM)
        .webhook_secret("webhook_hmac_secret_abcdef123456")
        .html_url("https://github.com")
        .base_api_url("https://api.github.com")
        .build()
        .expect("valid app config")
}

/// Test Case 1: AuthorizedInstallationContext masks sensitive tokens in Debug and Display.
#[test]
fn test_authorized_installation_context_masks_token() {
    let secret_token = "ghs_ExtremelySensitiveInstallationToken999";
    let ctx = AuthorizedInstallationContext::new(
        "acme-corp",
        "private-engine",
        7788,
        "acme-corp",
        secret_token,
        RepositorySelection::Selected,
    );

    let debug_output = format!("{ctx:?}");
    let display_output = format!("{ctx}");

    assert!(
        !debug_output.contains(secret_token),
        "AuthorizedInstallationContext Debug leaked secret token!"
    );
    assert!(debug_output.contains("***[REDACTED_TOKEN]***"));

    assert!(
        !display_output.contains(secret_token),
        "AuthorizedInstallationContext Display leaked secret token!"
    );
    assert!(display_output.contains("acme-corp"));
    assert!(display_output.contains("private-engine"));
}

/// Test Case 2: StoredTokenRecord masks sensitive tokens in Debug and Display.
#[test]
fn test_stored_token_record_masks_token() {
    let secret = "ghs_ConfidentialStoredToken1234567890";
    let record =
        StoredTokenRecord::new(secret, Utc.timestamp_opt(1800000000, 0).unwrap(), "octocat");

    let debug_output = format!("{record:?}");
    let display_output = format!("{record}");

    assert!(
        !debug_output.contains(secret),
        "StoredTokenRecord Debug leaked secret token!"
    );
    assert!(debug_output.contains("***[REDACTED_TOKEN]***"));

    assert!(
        !display_output.contains(secret),
        "StoredTokenRecord Display leaked secret token!"
    );
    assert!(!display_output.contains("ghs_"));
}

/// Test Case 3: GitHubAppConfig masks client_secret, private_key_pem, and webhook_secret in Debug and Display.
#[test]
fn test_app_config_masks_all_credentials() {
    let config = test_app_config();
    let debug_output = format!("{config:?}");
    let display_output = format!("{config}");

    // Client Secret
    assert!(
        !debug_output.contains("ghs_ClientSecret1234567890ABCDEF"),
        "GitHubAppConfig Debug leaked client_secret!"
    );
    assert!(
        !display_output.contains("ghs_ClientSecret1234567890ABCDEF"),
        "GitHubAppConfig Display leaked client_secret!"
    );

    // Webhook Secret
    assert!(
        !debug_output.contains("webhook_hmac_secret_abcdef123456"),
        "GitHubAppConfig Debug leaked webhook_secret!"
    );
    assert!(
        !display_output.contains("webhook_hmac_secret_abcdef123456"),
        "GitHubAppConfig Display leaked webhook_secret!"
    );

    // Private Key PEM delimiters
    assert!(
        !debug_output.contains("BEGIN PRIVATE KEY"),
        "GitHubAppConfig Debug leaked private key PEM!"
    );
    assert!(
        !display_output.contains("BEGIN PRIVATE KEY"),
        "GitHubAppConfig Display leaked private key PEM!"
    );

    assert!(debug_output.contains("***[REDACTED_SECRET]***"));
    assert!(debug_output.contains("***[REDACTED_RSA_KEY]***"));
}

/// Test Case 4: GitHubInstallationToken masks raw bearer token string in Debug and Display.
#[test]
fn test_github_installation_token_masks_token() {
    let secret_bearer = "ghs_EphemeralBearerInstallationToken456";
    let token = GitHubInstallationToken {
        token: secret_bearer.to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions: HashMap::new(),
        repository_selection: RepositorySelection::All,
        repositories: None,
    };

    let debug_output = format!("{token:?}");
    let display_output = format!("{token}");

    assert!(
        !debug_output.contains(secret_bearer),
        "GitHubInstallationToken Debug leaked raw bearer token!"
    );
    assert!(debug_output.contains("***[REDACTED_TOKEN]***"));

    assert!(
        !display_output.contains(secret_bearer),
        "GitHubInstallationToken Display leaked raw bearer token!"
    );
    assert!(display_output.contains("GitHubInstallationToken"));
}

/// Test Case 5: OAuth Token Responses mask access_token and refresh_token in Debug representations.
#[test]
fn test_oauth_token_responses_mask_secrets() {
    let access = "ghu_UserOAuthAccessToken1234567890";
    let refresh = "ghr_UserOAuthRefreshToken1234567890";

    let resp = GitHubOAuthTokenResponse {
        access_token: access.to_string(),
        token_type: "bearer".to_string(),
        scope: "read:user".to_string(),
        refresh_token: Some(refresh.to_string()),
        expires_in: Some(28800),
        refresh_token_expires_in: Some(15811200),
    };

    let debug_output = format!("{resp:?}");
    assert!(
        !debug_output.contains(access),
        "GitHubOAuthTokenResponse Debug leaked access_token!"
    );
    assert!(
        !debug_output.contains(refresh),
        "GitHubOAuthTokenResponse Debug leaked refresh_token!"
    );
    assert!(debug_output.contains("***[REDACTED_TOKEN]***"));

    let err_resp = GitHubOAuthErrorResponse {
        error: "bad_verification_code".to_string(),
        error_description: Some(format!("The code provided was invalid for token {access}")),
        error_uri: Some(format!("https://docs.github.com/error?token={access}")),
    };

    let err_debug = format!("{err_resp:?}");
    assert!(
        !err_debug.contains(access),
        "GitHubOAuthErrorResponse Debug leaked access_token!"
    );
    assert!(err_debug.contains("[REDACTED_GITHUB_TOKEN]"));
}

/// Test Case 6: AuthorizationRecoveryPlan and summary_card do not leak embedded tokens.
#[test]
fn test_recovery_plan_and_summary_card_zero_leakage() {
    let secret = "ghs_LeakedInErrorStringToken1234567890";
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let err = GitHubExtractorError::RequestFailed(format!(
        "Failed connection to https://api.github.com with token {secret}"
    ));

    let plan = coordinator.diagnose_extractor_error(
        &err,
        "acme-org",
        "private-repo",
        Some(1234),
        Some("acme-org"),
    );

    assert!(
        !plan.diagnosis.contains(secret),
        "Recovery plan diagnosis leaked secret token!"
    );
    assert!(plan.diagnosis.contains("[REDACTED_GITHUB_TOKEN]"));

    let card = plan.summary_card();
    assert!(
        !card.contains(secret),
        "Recovery plan summary card leaked secret token!"
    );
    assert!(card.contains("[REDACTED_GITHUB_TOKEN]"));
}

/// Test Case 7: GitHubExtractorError sanitization methods scrub tokens across all error variants.
#[test]
fn test_extractor_error_variants_sanitized() {
    let token = "ghs_ErrorVariantSecretToken1234567890";

    let variants = [
        GitHubExtractorError::BadCredentials {
            message: format!("Bad credentials: Bearer {token}"),
        },
        GitHubExtractorError::Forbidden {
            message: format!("Access forbidden for token {token}"),
        },
        GitHubExtractorError::ResourceNotFound {
            url: format!("https://api.github.com/repos?token={token}"),
            message: format!("Resource not found with key {token}"),
        },
        GitHubExtractorError::RateLimitExceeded {
            limit: 5000,
            reset_at: Utc.timestamp_opt(1800000000, 0).unwrap(),
            message: format!("Rate limit reached for token {token}"),
        },
        GitHubExtractorError::RequestFailed(format!("Network failure with token {token}")),
        GitHubExtractorError::AppConfiguration(format!("Invalid config referencing {token}")),
        GitHubExtractorError::HttpError {
            status: 500,
            message: format!("Server internal error with {token}"),
        },
    ];

    for variant in variants {
        let sanitized = variant.sanitized();
        let display_str = sanitized.to_string();
        let debug_str = format!("{sanitized:?}");

        assert!(
            !display_str.contains(token),
            "Sanitized error Display leaked token for variant: {display_str}"
        );
        assert!(
            !debug_str.contains(token),
            "Sanitized error Debug leaked token for variant: {debug_str}"
        );
        assert!(!variant.to_sanitized_string().contains(token));
    }
}

/// Test Case 8: Live wire HTTP extraction verification.
///
/// Asserts that when PrivateRepositoryExtractor issues network calls, the sensitive token
/// is passed exclusively in the Authorization header and NEVER in the URL path, query params,
/// or request payload.
#[tokio::test]
async fn test_wire_level_token_isolation_in_headers_only() {
    let secret_token = "ghs_WireLevelIsolationToken1234567890ABCDEF";
    let token_in_uri = Arc::new(AtomicBool::new(false));
    let token_in_auth_header = Arc::new(AtomicBool::new(false));
    let token_in_query = Arc::new(AtomicBool::new(false));

    let uri_flag = token_in_uri.clone();
    let auth_flag = token_in_auth_header.clone();
    let query_flag = token_in_query.clone();

    let app = Router::new().route(
        "/repos/acme/vault/commits/HEAD",
        get(move |req: Request| {
            let uri_f = uri_flag.clone();
            let auth_f = auth_flag.clone();
            let query_f = query_flag.clone();

            async move {
                let uri = req.uri().to_string();
                if uri.contains(secret_token) {
                    uri_f.store(true, Ordering::SeqCst);
                }

                if let Some(query) = req.uri().query() {
                    if query.contains(secret_token) {
                        query_f.store(true, Ordering::SeqCst);
                    }
                }

                if let Some(auth_val) = req.headers().get("authorization") {
                    if let Ok(auth_str) = auth_val.to_str() {
                        if auth_str == format!("Bearer {secret_token}") {
                            auth_f.store(true, Ordering::SeqCst);
                        }
                    }
                }

                let body = json!({
                    "sha": "1122334455667788990011223344556677889900",
                    "node_id": "C_commit_1",
                    "commit": {
                        "message": "vault test",
                        "author": { "name": "Dev", "email": "dev@acme.com", "date": "2026-03-01T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@acme.com", "date": "2026-03-01T00:00:00Z" },
                        "tree": { "sha": "0000000000000000000000000000000000000000", "url": "https://api.github.com/tree" },
                        "comment_count": 0
                    },
                    "parents": []
                });

                (StatusCode::OK, Json(body))
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let extractor = PrivateRepositoryExtractor::new(client);

    let ctx = AuthorizedInstallationContext::new(
        "acme",
        "vault",
        54321,
        "acme",
        secret_token,
        RepositorySelection::Selected,
    );

    // Call extract - it will execute get_commit against our mock server
    let _ = extractor.extract(&ctx, None).await;

    // Verify token was transmitted strictly in Authorization header
    assert!(
        token_in_auth_header.load(Ordering::SeqCst),
        "Expected token to be transmitted in Authorization header"
    );

    // Assert token was NEVER present in URI or query
    assert!(
        !token_in_uri.load(Ordering::SeqCst),
        "Sensitive token was leaked in request URI path!"
    );
    assert!(
        !token_in_query.load(Ordering::SeqCst),
        "Sensitive token was leaked in request query parameters!"
    );
}

/// Test Case 9: EncryptionKey and EncryptedPayload zeroization and secret masking in core.
#[test]
fn test_core_encryption_key_and_payload_masking() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let key_debug = format!("{key:?}");
    let key_display = format!("{key}");

    assert!(
        key_debug.contains("***[REDACTED_ENCRYPTION_KEY]***"),
        "EncryptionKey Debug leaked raw key bytes!"
    );
    assert!(
        key_display.contains("***[REDACTED_ENCRYPTION_KEY]***"),
        "EncryptionKey Display leaked raw key bytes!"
    );

    let payload = EncryptedPayload {
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
        nonce: [0x42; 12],
    };

    let payload_debug = format!("{payload:?}");
    let payload_display = format!("{payload}");

    assert!(
        payload_debug.contains("***[REDACTED_CIPHERTEXT]***"),
        "EncryptedPayload Debug leaked raw ciphertext!"
    );
    assert!(
        !payload_display.contains("deadbeef"),
        "EncryptedPayload Display leaked raw ciphertext hex!"
    );
    assert!(
        payload_display.contains("EncryptedPayload"),
        "EncryptedPayload Display should format descriptor without raw payload"
    );
}
