//! Integration test suite for authorization failure recovery and diagnostic classification (Subphase 6.3.4).
//!
//! Verifies:
//! - Diagnostic root-cause classification across all `AuthorizationOutcome` variants.
//! - Runtime `GitHubExtractorError` diagnosis (HTTP 401, 403, 404, 429).
//! - Automatic cache invalidation and transparent retry on HTTP 401 Bad credentials.
//! - Self-healing private repository extraction on token expiration.
//! - Strict Rule 05 zero secret leakage across diagnostics and recovery guidance.

use axum::extract::Path;
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::authorization::{
    AuthorizationOutcome, AuthorizedInstallationContext,
};
use web_repo_to_ai_github_extractor::authorization_recovery::{
    AuthorizationFailureKind, AuthorizationRecoveryCoordinator, RecoveryAction,
};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::error::GitHubExtractorError;
use web_repo_to_ai_github_extractor::installation::RepositorySelection;
use web_repo_to_ai_github_extractor::private_extractor::{
    PrivateExtractionConfig, PrivateRepositoryExtractor,
};
use web_repo_to_ai_github_extractor::source::compute_git_blob_sha;
use web_repo_to_ai_github_extractor::token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenRefresher, InstallationTokenStore,
    StoredTokenRecord, TokenLifecyclePolicy,
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

fn test_app_config() -> GitHubAppConfig {
    GitHubAppConfig::builder()
        .app_id(123456)
        .client_id("Iv1.test_client_id")
        .client_secret("test_client_secret_value")
        .private_key_pem(TEST_PKCS8_PEM)
        .webhook_secret("test_webhook_secret_value")
        .app_name("acme-converter")
        .app_slug("acme-converter-app")
        .build()
        .expect("valid test app config")
}

#[test]
fn test_diagnose_outcome_no_installation_for_session() {
    let app_cfg = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_cfg);

    let session_id = Uuid::new_v4();
    let outcome = AuthorizationOutcome::NoInstallationForSession { session_id };

    let plan = coordinator.diagnose_outcome(&outcome, "acme-corp", "secret-repo");

    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::NoInstallationForSession
    );
    assert!(!plan.is_auto_recoverable());
    assert_eq!(plan.target_slug.as_deref(), Some("acme-corp/secret-repo"));
    assert_eq!(plan.action_code(), "prompt_app_installation");

    match &plan.prescribed_action {
        RecoveryAction::PromptAppInstallation {
            installation_url,
            target_account,
        } => {
            assert!(installation_url.contains("apps/acme-converter-app/installations/new"));
            assert_eq!(target_account.as_deref(), Some("acme-corp"));
        }
        other => panic!("Unexpected action: {other:?}"),
    }

    let summary = plan.summary_card();
    assert!(summary.contains("GitHub App Authorization Diagnostic"));
    assert!(summary.contains("no_installation_for_session"));
    assert!(summary.contains("Install the GitHub App on your personal account"));
}

#[test]
fn test_diagnose_outcome_account_mismatch() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    let session_id = Uuid::new_v4();
    let outcome = AuthorizationOutcome::AccountMismatch {
        session_id,
        requested_owner: "target-org".to_string(),
        available_accounts: vec!["personal-user".to_string()],
    };

    let plan = coordinator.diagnose_outcome(&outcome, "target-org", "core-lib");

    assert_eq!(plan.failure_kind, AuthorizationFailureKind::AccountMismatch);
    assert!(!plan.is_auto_recoverable());
    assert!(plan.diagnosis.contains("target-org"));
    assert!(plan.diagnosis.contains("personal-user"));
    assert_eq!(plan.action_code(), "prompt_app_installation");
}

#[test]
fn test_diagnose_outcome_repository_not_granted_selected() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    let outcome = AuthorizationOutcome::RepositoryNotGranted {
        installation_id: 12345,
        account_login: "acme".to_string(),
        requested_repo: "acme/private-lib".to_string(),
        repository_selection: RepositorySelection::Selected,
    };

    let plan = coordinator.diagnose_outcome(&outcome, "acme", "private-lib");

    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::RepositoryNotGranted
    );
    assert!(!plan.is_auto_recoverable());
    assert_eq!(plan.action_code(), "prompt_repository_selection_update");

    match &plan.prescribed_action {
        RecoveryAction::PromptRepositorySelectionUpdate {
            settings_url,
            installation_id,
            repository,
        } => {
            assert_eq!(*installation_id, 12345);
            assert_eq!(repository, "acme/private-lib");
            assert!(settings_url.contains("settings/installations/12345"));
        }
        other => panic!("Unexpected action: {other:?}"),
    }

    assert!(plan
        .remediation_guidance
        .iter()
        .any(|g| g.contains("Only select repositories")));
}

#[test]
fn test_diagnose_outcome_installation_suspended_and_revoked() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    // Suspended
    let suspended = AuthorizationOutcome::InstallationSuspended {
        installation_id: 9999,
        account_login: "corp".to_string(),
    };
    let p_susp = coordinator.diagnose_outcome(&suspended, "corp", "lib");
    assert_eq!(
        p_susp.failure_kind,
        AuthorizationFailureKind::InstallationSuspended
    );
    assert_eq!(p_susp.action_code(), "prompt_unsuspend_installation");
    assert!(p_susp.summary_card().contains("Unsuspend"));

    // Revoked
    let revoked = AuthorizationOutcome::InstallationRevoked {
        installation_id: 8888,
        account_login: "corp".to_string(),
    };
    let p_rev = coordinator.diagnose_outcome(&revoked, "corp", "lib");
    assert_eq!(
        p_rev.failure_kind,
        AuthorizationFailureKind::InstallationRevoked
    );
    assert_eq!(p_rev.action_code(), "prompt_app_installation");
}

#[test]
fn test_diagnose_outcome_insufficient_permissions() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    let outcome = AuthorizationOutcome::InsufficientPermissions {
        installation_id: 555,
        account_login: "org".to_string(),
        violations: vec!["Forbidden write access".to_string()],
    };

    let plan = coordinator.diagnose_outcome(&outcome, "org", "repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::InsufficientPermissions
    );
    assert_eq!(plan.action_code(), "contact_administrator");
}

#[test]
fn test_diagnose_extractor_errors() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    // 401 Bad Credentials without refresher -> not auto-recoverable
    let err_401 = GitHubExtractorError::BadCredentials {
        message: "Bad credentials".to_string(),
    };
    let plan_401 =
        coordinator.diagnose_extractor_error(&err_401, "acme", "repo", Some(123), Some("acme"));
    assert_eq!(
        plan_401.failure_kind,
        AuthorizationFailureKind::TokenInvalid
    );
    assert!(!plan_401.is_auto_recoverable());

    // 404 Resource Not Found
    let err_404 = GitHubExtractorError::ResourceNotFound {
        url: "https://api.github.com/repos/acme/repo".to_string(),
        message: "Not Found".to_string(),
    };
    let plan_404 =
        coordinator.diagnose_extractor_error(&err_404, "acme", "repo", Some(123), Some("acme"));
    assert_eq!(
        plan_404.failure_kind,
        AuthorizationFailureKind::RepositoryNotGranted
    );
    assert_eq!(plan_404.action_code(), "prompt_repository_selection_update");

    // Rate limit
    let reset_time = Utc::now() + Duration::seconds(120);
    let err_rate = GitHubExtractorError::RateLimitExceeded {
        limit: 5000,
        reset_at: reset_time,
        message: "API rate limit exceeded".to_string(),
    };
    let plan_rate =
        coordinator.diagnose_extractor_error(&err_rate, "acme", "repo", Some(123), Some("acme"));
    assert_eq!(
        plan_rate.failure_kind,
        AuthorizationFailureKind::RateLimited
    );
    assert_eq!(plan_rate.action_code(), "wait_rate_limit_reset");
}

#[tokio::test]
async fn test_execute_with_retry_on_401_transparent_recovery() {
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session_id = Uuid::new_v4();
    let installation_id = 777;

    let refresh_count = Arc::new(AtomicUsize::new(0));
    let refresh_count_clone = Arc::clone(&refresh_count);

    let app = Router::new().route(
        &format!("/app/installations/{installation_id}/access_tokens"),
        post(move || {
            let cnt = Arc::clone(&refresh_count_clone);
            async move {
                cnt.fetch_add(1, Ordering::SeqCst);
                let expires_at = Utc::now() + Duration::hours(1);
                let body = json!({
                    "token": "ghs_brand_new_token_12345",
                    "expires_at": expires_at.to_rfc3339(),
                    "permissions": { "contents": "read", "metadata": "read" },
                    "repository_selection": "selected"
                });
                (StatusCode::CREATED, Json(body))
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));

    // Store initially stale token
    let stale_token = StoredTokenRecord::new(
        "ghs_stale_token_bad",
        Utc::now() + Duration::hours(1),
        "acme",
    );
    store
        .save_token_record(&session_id, installation_id, stale_token)
        .await
        .unwrap();

    let store_trait: Arc<dyn InstallationTokenStore> = store.clone();
    let refresher = Arc::new(InstallationTokenRefresher::new(
        Arc::clone(&store_trait),
        Arc::clone(&client),
        app_config,
        TokenLifecyclePolicy::default(),
    ));

    let coordinator = AuthorizationRecoveryCoordinator::new()
        .with_token_store(Arc::clone(&store_trait))
        .with_token_refresher(refresher);

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let result = coordinator
        .execute_with_retry_on_401(&session_id, installation_id, "acme", |maybe_token| {
            let cnt = Arc::clone(&call_count_clone);
            let token_opt = maybe_token.map(ToString::to_string);
            async move {
                let call = cnt.fetch_add(1, Ordering::SeqCst);
                if call == 0 {
                    // First call fails with 401
                    Err(GitHubExtractorError::BadCredentials {
                        message: "Bad credentials".to_string(),
                    })
                } else {
                    // Second call receives the renewed token
                    assert_eq!(token_opt.as_deref(), Some("ghs_brand_new_token_12345"));
                    Ok("success_data")
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), "success_data");
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
    assert_eq!(refresh_count.load(Ordering::SeqCst), 1);

    // Verify token store updated with new token
    let stored = store
        .get_token(&session_id, installation_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.token, "ghs_brand_new_token_12345");
}

#[tokio::test]
async fn test_private_extractor_transparent_401_recovery() {
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session_id = Uuid::new_v4();
    let installation_id = 999;

    let refresh_count = Arc::new(AtomicUsize::new(0));
    let refresh_count_clone = Arc::clone(&refresh_count);

    let code = "pub fn add_numbers(a: i32, b: i32) -> i32 { a + b }\n";
    let code_sha = compute_git_blob_sha(code.as_bytes());
    let commit_sha = "0000111122223333444455556666777788889999";
    let tree_sha = "9999888877776666555544443333222211110000";

    let commit_attempts = Arc::new(AtomicUsize::new(0));
    let commit_attempts_clone = Arc::clone(&commit_attempts);

    let c_sha = code_sha.clone();
    let app = Router::new()
        .route(
            &format!("/app/installations/{installation_id}/access_tokens"),
            post(move || {
                let cnt = Arc::clone(&refresh_count_clone);
                async move {
                    cnt.fetch_add(1, Ordering::SeqCst);
                    let expires_at = Utc::now() + Duration::hours(1);
                    let body = json!({
                        "token": "ghs_renewed_token_after_401",
                        "expires_at": expires_at.to_rfc3339(),
                        "permissions": { "contents": "read", "metadata": "read" },
                        "repository_selection": "selected"
                    });
                    (StatusCode::CREATED, Json(body))
                }
            }),
        )
        // First commit query with stale token returns 401; second with renewed token returns 200
        .route(
            "/repos/acme/self-healing/commits/HEAD",
            get(move || {
                let attempts = Arc::clone(&commit_attempts_clone);
                async move {
                    let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt == 0 {
                        let body = json!({
                            "message": "Bad credentials",
                            "documentation_url": "https://docs.github.com/rest"
                        });
                        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
                    } else {
                        let body = json!({
                            "sha": commit_sha,
                            "node_id": "C_commit_id_123",
                            "commit": {
                                "message": "feat: self healing test",
                                "author": { "name": "Dev", "email": "dev@acme.com", "date": "2026-03-01T00:00:00Z" },
                                "committer": { "name": "Dev", "email": "dev@acme.com", "date": "2026-03-01T00:00:00Z" },
                                "tree": { "sha": tree_sha, "url": "https://api.github.com/tree" },
                                "comment_count": 0
                            },
                            "parents": []
                        });
                        (StatusCode::OK, Json(body)).into_response()
                    }
                }
            }),
        )
        .route(
            &format!("/repos/acme/self-healing/git/trees/{tree_sha}"),
            get(move || {
                let s = c_sha.clone();
                async move {
                    let body = json!({
                        "sha": tree_sha,
                        "url": "https://api.github.com/tree",
                        "truncated": false,
                        "tree": [
                            {
                                "path": "src/lib.rs",
                                "mode": "100644",
                                "type": "blob",
                                "sha": s,
                                "size": code.len()
                            }
                        ]
                    });
                    (StatusCode::OK, Json(body))
                }
            }),
        )
        .route(
            "/repos/acme/self-healing/git/blobs/:sha",
            get(move |Path(_sha): Path<String>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.github.raw".parse().unwrap(),
                );
                (StatusCode::OK, headers, code.as_bytes().to_vec()).into_response()
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));

    let stale_token = StoredTokenRecord::new(
        "ghs_stale_token_that_will_fail_401",
        Utc::now() + Duration::hours(1),
        "acme",
    );
    store
        .save_token_record(&session_id, installation_id, stale_token)
        .await
        .unwrap();

    let store_trait: Arc<dyn InstallationTokenStore> = store.clone();
    let refresher = Arc::new(InstallationTokenRefresher::new(
        Arc::clone(&store_trait),
        Arc::clone(&client),
        app_config,
        TokenLifecyclePolicy::default(),
    ));

    let config = PrivateExtractionConfig::default().with_fetch_metadata(false);
    let extractor = PrivateRepositoryExtractor::new(client)
        .with_token_refresher(refresher, Arc::clone(&store_trait), session_id)
        .with_config(config);

    let ctx = AuthorizedInstallationContext::new(
        "acme",
        "self-healing",
        installation_id,
        "acme",
        "ghs_stale_token_that_will_fail_401",
        RepositorySelection::Selected,
    );

    // Extraction should transparently self-heal by evicting stale token and retrying with renewed token
    let result = extractor
        .extract(&ctx, None)
        .await
        .expect("private extraction must succeed after transparent 401 retry");

    assert_eq!(result.commit_sha(), commit_sha);
    assert_eq!(result.total_extracted_files(), 1);
    assert!(result.is_balanced());
    assert_eq!(commit_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(refresh_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_authorization_recovery_zero_secret_leakage() {
    let coordinator = AuthorizationRecoveryCoordinator::new();

    let secret_token = "ghs_SecretInstallationToken1234567890ABCDEF";
    let err = GitHubExtractorError::BadCredentials {
        message: format!("Bad credentials for token: {secret_token}"),
    };

    let plan = coordinator.diagnose_extractor_error(
        &err,
        "secret-org",
        "secret-repo",
        Some(123),
        Some("secret-org"),
    );

    let diagnosis = &plan.diagnosis;
    assert!(!diagnosis.contains(secret_token));

    let card = plan.summary_card();
    assert!(!card.contains(secret_token));

    let json_str = serde_json::to_string(&plan).expect("must serialize");
    assert!(!json_str.contains(secret_token));
}
