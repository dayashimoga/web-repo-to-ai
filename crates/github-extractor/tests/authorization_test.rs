//! Integration test suite for GitHub App installation authorization engine (Subphase 6.3.1).
//!
//! Validates tenant session scoping, account resolution, repository access grants
//! (`All` vs `Selected` with live verification), Rule 05 least-privilege enforcement,
//! proactive token renewal, revoked/suspended detection, and secret masking.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::authorization::{
    AuthorizationOutcome, AuthorizedInstallationContext, InstallationAuthorizer,
};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::installation::RepositorySelection;
use web_repo_to_ai_github_extractor::permissions::PermissionPolicy;
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
        .build()
        .expect("valid test app config")
}

fn valid_permissions() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("contents".to_string(), "read".to_string());
    map.insert("metadata".to_string(), "read".to_string());
    map
}

#[tokio::test]
async fn test_authorize_all_repos_success() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record = StoredTokenRecord::new(
        "ghs_ValidAllToken123",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9901, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "any-repo")
        .await
        .unwrap();

    assert!(outcome.is_authorized());
    let ctx = outcome.unwrap_authorized();
    assert_eq!(ctx.installation_id, 9901);
    assert_eq!(ctx.account_login, "acme-corp");
    assert_eq!(ctx.token, "ghs_ValidAllToken123");
    assert_eq!(ctx.owner, "acme-corp");
    assert_eq!(ctx.repo, "any-repo");
    assert_eq!(ctx.repository_selection, RepositorySelection::All);
}

#[tokio::test]
async fn test_authorize_selected_repo_in_cached_list_success() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record = StoredTokenRecord::new(
        "ghs_SelectedToken456",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::Selected)
    .with_repositories(vec![
        "acme-corp/repo1".to_string(),
        "acme-corp/repo2".to_string(),
    ])
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9902, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo2")
        .await
        .unwrap();

    assert!(outcome.is_authorized());
    let ctx = outcome.unwrap_authorized();
    assert_eq!(ctx.installation_id, 9902);
    assert_eq!(ctx.repo, "repo2");
    assert_eq!(ctx.repository_selection, RepositorySelection::Selected);
}

#[tokio::test]
async fn test_authorize_selected_repo_via_live_github_check_success() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    // Record has selected mode, but repo3 is not in local cache
    let record = StoredTokenRecord::new(
        "ghs_SelectedLiveToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::Selected)
    .with_repositories(vec!["acme-corp/repo1".to_string()])
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9903, record)
        .await
        .unwrap();

    // Mock GitHub GET /repos/acme-corp/repo3
    let app = Router::new().route(
        "/repos/:owner/:repo",
        get(|Path((owner, repo)): Path<(String, String)>| async move {
            if owner == "acme-corp" && repo == "repo3" {
                (
                    StatusCode::OK,
                    Json(json!({
                        "id": 1234567,
                        "name": "repo3",
                        "full_name": "acme-corp/repo3",
                        "private": true,
                        "permissions": {
                            "admin": false,
                            "push": false,
                            "pull": true
                        }
                    })),
                )
                    .into_response()
            } else {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo3")
        .await
        .unwrap();

    assert!(outcome.is_authorized());
    let ctx = outcome.unwrap_authorized();
    assert_eq!(ctx.repo, "repo3");
}

#[tokio::test]
async fn test_authorize_selected_repo_not_granted_returns_404_on_github() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record = StoredTokenRecord::new(
        "ghs_SelectedMissingToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::Selected)
    .with_repositories(vec!["acme-corp/repo1".to_string()])
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9904, record)
        .await
        .unwrap();

    // Mock GitHub GET /repos/:owner/:repo returning 404
    let app = Router::new().route(
        "/repos/:owner/:repo",
        get(|| async { (StatusCode::NOT_FOUND, "Not Found") }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "unauthorized-repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::RepositoryNotGranted {
            installation_id,
            account_login,
            requested_repo,
            repository_selection,
        } => {
            assert_eq!(installation_id, 9904);
            assert_eq!(account_login, "acme-corp");
            assert_eq!(requested_repo, "acme-corp/unauthorized-repo");
            assert_eq!(repository_selection, RepositorySelection::Selected);
        }
        other => panic!("Expected RepositoryNotGranted, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_no_installation_for_session() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "my-repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::NoInstallationForSession {
            session_id: actual_session,
        } => {
            assert_eq!(actual_session, session_id);
        }
        other => panic!("Expected NoInstallationForSession, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_account_mismatch() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record = StoredTokenRecord::new(
        "ghs_AcmeToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9905, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "other-corp", "other-repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::AccountMismatch {
            session_id: actual_session,
            requested_owner,
            available_accounts,
        } => {
            assert_eq!(actual_session, session_id);
            assert_eq!(requested_owner, "other-corp");
            assert_eq!(available_accounts, vec!["acme-corp"]);
        }
        other => panic!("Expected AccountMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_installation_revoked() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let mut record = StoredTokenRecord::new(
        "ghs_RevokedToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());
    record.is_revoked = true;

    store
        .save_token_record(&session_id, 9906, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::InstallationRevoked {
            installation_id,
            account_login,
        } => {
            assert_eq!(installation_id, 9906);
            assert_eq!(account_login, "acme-corp");
        }
        other => panic!("Expected InstallationRevoked, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_installation_suspended() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let mut record = StoredTokenRecord::new(
        "ghs_SuspendedToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());
    record.is_suspended = true;

    store
        .save_token_record(&session_id, 9907, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::InstallationSuspended {
            installation_id,
            account_login,
        } => {
            assert_eq!(installation_id, 9907);
            assert_eq!(account_login, "acme-corp");
        }
        other => panic!("Expected InstallationSuspended, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_insufficient_permissions_rejected_under_rule_05() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let mut dangerous_perms = HashMap::new();
    dangerous_perms.insert("contents".to_string(), "write".to_string());
    dangerous_perms.insert("administration".to_string(), "write".to_string());

    let record = StoredTokenRecord::new(
        "ghs_DangerousPermsToken",
        Utc::now() + Duration::hours(1),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(dangerous_perms);

    store
        .save_token_record(&session_id, 9908, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client)
        .with_permission_policy(PermissionPolicy::strict_minimal());

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::InsufficientPermissions {
            installation_id,
            account_login,
            violations,
        } => {
            assert_eq!(installation_id, 9908);
            assert_eq!(account_login, "acme-corp");
            assert!(!violations.is_empty());
        }
        other => panic!("Expected InsufficientPermissions, got {other:?}"),
    }
}

#[tokio::test]
async fn test_authorize_proactive_token_renewal_success() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    // Token expiring in 60 seconds (safety margin 300s -> triggers proactive renewal)
    let record = StoredTokenRecord::new(
        "ghs_OldExpiringToken",
        Utc::now() + Duration::seconds(60),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9909, record)
        .await
        .unwrap();

    // Mock GitHub POST /app/installations/:id/access_tokens
    let app = Router::new().route(
        "/app/installations/:id/access_tokens",
        post(|Path(_id): Path<u64>| async move {
            (
                StatusCode::CREATED,
                Json(json!({
                    "token": "ghs_RenewedFreshToken999",
                    "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339(),
                    "permissions": {
                        "contents": "read",
                        "metadata": "read"
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

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let refresher = Arc::new(InstallationTokenRefresher::new(
        store.clone(),
        client.clone(),
        Arc::new(test_app_config()),
        TokenLifecyclePolicy::default(),
    ));

    let authorizer = InstallationAuthorizer::new(store, client)
        .with_refresher(refresher)
        .with_safety_margin_secs(300);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "target-repo")
        .await
        .unwrap();

    assert!(outcome.is_authorized());
    let ctx = outcome.unwrap_authorized();
    assert_eq!(ctx.token, "ghs_RenewedFreshToken999");
    assert_eq!(ctx.owner, "acme-corp");
    assert_eq!(ctx.repo, "target-repo");
}

#[tokio::test]
async fn test_authorize_expired_token_without_refresher_fails() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    // Expired 10 minutes ago
    let record = StoredTokenRecord::new(
        "ghs_DeadToken",
        Utc::now() - Duration::minutes(10),
        "acme-corp",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(valid_permissions());

    store
        .save_token_record(&session_id, 9910, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);

    let outcome = authorizer
        .authorize(&session_id, "acme-corp", "repo")
        .await
        .unwrap();

    match outcome {
        AuthorizationOutcome::TokenUnavailable {
            installation_id,
            account_login,
            reason,
        } => {
            assert_eq!(installation_id, 9910);
            assert_eq!(account_login, "acme-corp");
            assert!(reason.contains("expired"));
        }
        other => panic!("Expected TokenUnavailable, got {other:?}"),
    }
}

#[test]
fn test_authorized_context_secret_masking() {
    let ctx = AuthorizedInstallationContext {
        installation_id: 12345,
        account_login: "secure-org".to_string(),
        token: "ghs_SecretPlaintextTokenToNeverLeak".to_string(),
        repository_selection: RepositorySelection::Selected,
        owner: "secure-org".to_string(),
        repo: "private-core".to_string(),
        permissions: valid_permissions(),
    };

    let debug_repr = format!("{ctx:?}");
    assert!(
        !debug_repr.contains("ghs_SecretPlaintextTokenToNeverLeak"),
        "Plaintext token must NEVER appear in Debug format"
    );
    assert!(
        debug_repr.contains("***[REDACTED_TOKEN]***"),
        "Debug must contain redacted mask"
    );

    let display_repr = format!("{ctx}");
    assert!(
        !display_repr.contains("ghs_SecretPlaintextTokenToNeverLeak"),
        "Plaintext token must NEVER appear in Display format"
    );
    assert!(display_repr.contains("secure-org"));
    assert!(display_repr.contains("private-core"));
}
