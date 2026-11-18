//! Integration test suite for GitHub App installation token refresh, expiration, and concurrent deduplication under Rule 05.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::installation::GitHubInstallationToken;
use web_repo_to_ai_github_extractor::token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenRefresher, InstallationTokenStore,
    StoredTokenRecord, TokenFreshness, TokenLifecyclePolicy,
};

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

fn test_app_config() -> web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig {
    GitHubAppConfig::builder()
        .app_id(123456)
        .client_id("Iv1.test_client_id")
        .client_secret("test_client_secret_value")
        .private_key_pem(TEST_PKCS8_PEM)
        .webhook_secret("test_webhook_secret_value")
        .build()
        .expect("valid test app config")
}

#[test]
fn test_token_freshness_evaluation() {
    let now = Utc::now();
    let margin = 300u64; // 5 minutes

    // 1. Fresh: expires in 45 minutes (2700s > 300s)
    let exp_fresh = now + Duration::minutes(45);
    let freshness = TokenFreshness::evaluate(exp_fresh, now, margin);
    assert!(freshness.is_fresh());
    assert!(!freshness.is_expired());
    assert!(!freshness.needs_refresh());
    match freshness {
        TokenFreshness::Fresh { seconds_remaining } => {
            assert!((2690..=2700).contains(&seconds_remaining));
        }
        _ => panic!("Expected Fresh"),
    }

    // 2. Expiring soon: expires in 3 minutes (180s <= 300s)
    let exp_soon = now + Duration::minutes(3);
    let freshness_soon = TokenFreshness::evaluate(exp_soon, now, margin);
    assert!(!freshness_soon.is_fresh());
    assert!(!freshness_soon.is_expired());
    assert!(freshness_soon.needs_refresh());
    match freshness_soon {
        TokenFreshness::ExpiringSoon { seconds_remaining } => {
            assert!((170..=180).contains(&seconds_remaining));
        }
        _ => panic!("Expected ExpiringSoon"),
    }

    // 3. Expired: expired 10 minutes ago
    let exp_past = now - Duration::minutes(10);
    let freshness_past = TokenFreshness::evaluate(exp_past, now, margin);
    assert!(!freshness_past.is_fresh());
    assert!(freshness_past.is_expired());
    assert!(freshness_past.needs_refresh());
    match freshness_past {
        TokenFreshness::Expired { expired_ago_secs } => {
            assert!((590..=610).contains(&expired_ago_secs));
        }
        _ => panic!("Expected Expired"),
    }
}

#[test]
fn test_stored_token_record_redaction() {
    let record = StoredTokenRecord::new(
        "ghs_SuperSecretSensitiveTokenValue1234567890",
        Utc::now() + Duration::hours(1),
        "octocat",
    );

    // Verify Debug masking
    let debug_str = format!("{record:?}");
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));
    assert!(!debug_str.contains("SuperSecretSensitiveTokenValue"));

    // Verify Display masking
    let display_str = format!("{record}");
    assert!(!display_str.contains("SuperSecretSensitiveTokenValue"));
    assert!(display_str.contains("account_login=octocat"));

    // Plaintext accessor
    assert_eq!(
        record.token(),
        "ghs_SuperSecretSensitiveTokenValue1234567890"
    );
}

#[tokio::test]
async fn test_in_memory_installation_token_store_crud() {
    let store = InMemoryInstallationTokenStore::new();
    let session = Uuid::new_v4();
    let install_id = 123456i64;

    // Initially none
    assert!(store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .is_none());

    // Save token
    let token = GitHubInstallationToken {
        token: "ghs_StoredToken1".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };
    store
        .save_token(&session, install_id, "user-one", &token)
        .await
        .unwrap();

    let fetched = store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .expect("present");
    assert_eq!(fetched.token(), "ghs_StoredToken1");
    assert_eq!(fetched.account_login, "user-one");

    // Revoke
    assert!(store.revoke_token(&session, install_id).await.unwrap());
    assert!(store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .is_none());
    assert!(!store.revoke_token(&session, install_id).await.unwrap());
}

#[tokio::test]
async fn test_token_refresher_reuses_fresh_token() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 998877i64;

    // Pre-populate store with a fresh token expiring in 40 minutes
    let fresh_token = GitHubInstallationToken {
        token: "ghs_CachedFreshToken".to_string(),
        expires_at: Utc::now() + Duration::minutes(40),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };
    store
        .save_token(&session, install_id, "cached-user", &fresh_token)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let app_config = Arc::new(test_app_config());
    let policy = TokenLifecyclePolicy::default(); // 5 min margin

    let refresher = InstallationTokenRefresher::new(store.clone(), client, app_config, policy);

    // Call get_or_refresh_token: should return cached token without attempting network request
    let token = refresher
        .get_or_refresh_token(&session, install_id, "cached-user")
        .await
        .expect("retrieval succeeds");

    assert_eq!(token, "ghs_CachedFreshToken");
}

#[tokio::test]
async fn test_token_refresher_renews_expiring_soon_token_with_mock_server() {
    let request_count = Arc::new(AtomicUsize::new(0));
    let count_clone = request_count.clone();

    // Mock GitHub token renewal endpoint
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(move |Path(id): Path<u64>| {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                let expires_at = Utc::now() + Duration::hours(1);
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "token": format!("ghs_RenewedTokenFor_{id}"),
                        "expires_at": expires_at.to_rfc3339(),
                        "permissions": {
                            "contents": "read",
                            "metadata": "read"
                        },
                        "repository_selection": "all"
                    })),
                )
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 771122i64;

    // Token expiring in 2 minutes (policy margin is 5 minutes, so it is ExpiringSoon)
    let expiring_token = GitHubInstallationToken {
        token: "ghs_OldExpiringToken".to_string(),
        expires_at: Utc::now() + Duration::minutes(2),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };
    store
        .save_token(&session, install_id, "renew-user", &expiring_token)
        .await
        .unwrap();

    let refresher = InstallationTokenRefresher::new(
        store.clone(),
        client,
        app_config,
        TokenLifecyclePolicy::default(),
    );

    // Should detect ExpiringSoon, call mock server, renew, and update store
    let renewed = refresher
        .get_or_refresh_token(&session, install_id, "renew-user")
        .await
        .unwrap();

    assert_eq!(renewed, format!("ghs_RenewedTokenFor_{install_id}"));
    assert_eq!(request_count.load(Ordering::SeqCst), 1);

    // Verify store updated with new token
    let stored = store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.token(), format!("ghs_RenewedTokenFor_{install_id}"));
    assert!(stored.freshness(300).is_fresh());
}

#[tokio::test]
async fn test_token_refresher_concurrent_single_flight_deduplication() {
    let request_count = Arc::new(AtomicUsize::new(0));
    let count_clone = request_count.clone();

    // Mock endpoint introduces a small delay to test race conditions
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(move |Path(id): Path<u64>| {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                let expires_at = Utc::now() + Duration::hours(1);
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "token": format!("ghs_DeduplicatedToken_{id}"),
                        "expires_at": expires_at.to_rfc3339(),
                        "permissions": {
                            "contents": "read",
                            "metadata": "read"
                        },
                        "repository_selection": "all"
                    })),
                )
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 554433i64;

    let refresher = Arc::new(InstallationTokenRefresher::new(
        store,
        client,
        app_config,
        TokenLifecyclePolicy::default(),
    ));

    // Spawn 20 concurrent tasks requesting token for the SAME installation simultaneously
    let mut handles = Vec::new();
    for _ in 0..20 {
        let r = refresher.clone();
        let s = session;
        handles.push(tokio::spawn(async move {
            r.get_or_refresh_token(&s, install_id, "concurrent-org")
                .await
        }));
    }

    // Await all 20 tasks
    for handle in handles {
        let res = handle.await.unwrap();
        assert_eq!(res.unwrap(), format!("ghs_DeduplicatedToken_{install_id}"));
    }

    // Exactly 1 request must have been made to the GitHub token renewal endpoint
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        1,
        "Single-flight deduplication failed: multiple API calls issued"
    );
}

#[tokio::test]
async fn test_token_refresher_force_refresh() {
    let request_count = Arc::new(AtomicUsize::new(0));
    let count_clone = request_count.clone();

    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(move |Path(id): Path<u64>| {
            let count = count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                let expires_at = Utc::now() + Duration::hours(1);
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "token": format!("ghs_ForcedToken_{id}_v{current}"),
                        "expires_at": expires_at.to_rfc3339(),
                        "permissions": {
                            "contents": "read",
                            "metadata": "read"
                        },
                        "repository_selection": "all"
                    })),
                )
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 889900i64;

    let refresher = InstallationTokenRefresher::new(
        store.clone(),
        client,
        app_config,
        TokenLifecyclePolicy::default(),
    );

    // Initial get_or_refresh -> v1
    let t1 = refresher
        .get_or_refresh_token(&session, install_id, "force-org")
        .await
        .unwrap();
    assert_eq!(t1, format!("ghs_ForcedToken_{install_id}_v1"));

    // force_refresh -> v2
    let t2 = refresher
        .force_refresh(&session, install_id, "force-org")
        .await
        .unwrap();
    assert_eq!(t2, format!("ghs_ForcedToken_{install_id}_v2"));
    assert_eq!(request_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_token_refresher_rule05_permission_validation() {
    // Mock server returning token with forbidden administrative scope
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(|Path(_id): Path<u64>| async move {
            (
                StatusCode::CREATED,
                Json(json!({
                    "token": "ghs_DangerousWriteToken",
                    "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339(),
                    "permissions": {
                        "contents": "write",
                        "administration": "write"
                    },
                    "repository_selection": "all"
                })),
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 110022i64;

    let refresher = InstallationTokenRefresher::new(
        store.clone(),
        client,
        app_config,
        TokenLifecyclePolicy::default(),
    );

    // Must fail closed due to Rule 05 permission violation
    let res = refresher
        .get_or_refresh_token(&session, install_id, "bad-perm-org")
        .await;

    assert!(res.is_err());
    let err_str = res.err().unwrap().to_string();
    assert!(
        err_str.contains("Permission")
            || err_str.contains("forbidden")
            || err_str.contains("write")
    );

    // Nothing stored
    assert!(store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_token_refresher_retry_on_transient_failure() {
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let count_clone = attempt_count.clone();

    // Mock server fails first 2 attempts with HTTP 500, then succeeds
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(move |Path(id): Path<u64>| {
            let count = count_clone.clone();
            async move {
                let attempt = count.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt < 3 {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Server Error".into_response(),
                    )
                } else {
                    let expires_at = Utc::now() + Duration::hours(1);
                    (
                        StatusCode::CREATED,
                        Json(json!({
                            "token": format!("ghs_SucceededOnAttempt3_{id}"),
                            "expires_at": expires_at.to_rfc3339(),
                            "permissions": {
                                "contents": "read",
                                "metadata": "read"
                            },
                            "repository_selection": "all"
                        }))
                        .into_response(),
                    )
                }
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 992233i64;

    let policy = TokenLifecyclePolicy::default()
        .with_max_retries(3)
        .with_retry_backoff_ms(10); // Fast retry for tests

    let refresher = InstallationTokenRefresher::new(store, client, app_config, policy);

    let token = refresher
        .get_or_refresh_token(&session, install_id, "retry-org")
        .await
        .unwrap();

    assert_eq!(token, format!("ghs_SucceededOnAttempt3_{install_id}"));
    assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_token_refresher_exhausted_retries_fails() {
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(|Path(_id): Path<u64>| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Persistent GitHub Outage",
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(mock_base_url));
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session = Uuid::new_v4();
    let install_id = 444444i64;

    let policy = TokenLifecyclePolicy::default()
        .with_max_retries(2)
        .with_retry_backoff_ms(10);

    let refresher = InstallationTokenRefresher::new(store, client, app_config, policy);

    let res = refresher
        .get_or_refresh_token(&session, install_id, "outage-org")
        .await;

    assert!(res.is_err());
}
