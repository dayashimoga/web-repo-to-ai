//! Integration test suite for unauthorized repository access scenarios (Subphase 6.4.2).
//!
//! Validates that unauthorized repository requests across missing installations,
//! account mismatches, ungranted selected repositories, suspended/revoked installations,
//! permission policy violations, upstream HTTP 403/404 responses, and token failures
//! are strictly blocked, produce zero content leakage, and map to actionable
//! remediation plans under Rule 05.

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::authorization::{
    AuthorizationOutcome, AuthorizedInstallationContext, InstallationAuthorizer,
};
use web_repo_to_ai_github_extractor::authorization_recovery::{
    AuthorizationFailureKind, AuthorizationRecoveryCoordinator, RecoveryAction,
};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::error::GitHubExtractorError;
use web_repo_to_ai_github_extractor::installation::RepositorySelection;
use web_repo_to_ai_github_extractor::private_extractor::PrivateRepositoryExtractor;
use web_repo_to_ai_github_extractor::token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenStore, StoredTokenRecord,
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
        .app_id(998877)
        .app_slug("security-verify-app")
        .client_id("Iv1.mock_security_client_id")
        .client_secret("ghs_mock_security_client_secret_998877")
        .private_key_pem(TEST_PKCS8_PEM)
        .webhook_secret("security_webhook_secret_998877")
        .html_url("https://github.com")
        .base_api_url("https://api.github.com")
        .build()
        .expect("test app config must build")
}

/// Test Case 1: Unauthorized access when calling session has no registered installation.
///
/// Verifies that accessing a private repository without an installation fails with
/// NoInstallationForSession and produces a 1-click installation recovery plan.
#[tokio::test]
async fn test_unauthorized_no_installation_for_session() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store, client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();

    let outcome = authorizer
        .authorize(&session_id, "private-corp", "secret-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::NoInstallationForSession {
            session_id: returned_session,
        } => {
            assert_eq!(*returned_session, session_id);
        }
        other => panic!("Expected NoInstallationForSession, got: {other:?}"),
    }

    // Diagnose outcome with recovery coordinator
    let plan = coordinator.diagnose_outcome(&outcome, "private-corp", "secret-repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::NoInstallationForSession
    );
    assert!(!plan.is_auto_recoverable);
    match plan.prescribed_action {
        RecoveryAction::PromptAppInstallation {
            installation_url,
            target_account,
        } => {
            assert!(installation_url.contains("security-verify-app"));
            assert_eq!(target_account, Some("private-corp".to_string()));
        }
        other => panic!("Expected PromptAppInstallation, got: {other:?}"),
    }
}

/// Test Case 2: Unauthorized access when target repository owner differs from installed account.
///
/// Verifies that accessing an uninstalled account's repository yields AccountMismatch
/// and does not expose credentials belonging to the user's other installations.
#[tokio::test]
async fn test_unauthorized_account_mismatch() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store.clone(), client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();
    let install_id = 1111i64;

    // Session has an installation for 'alpha-org'
    let token_record = StoredTokenRecord::new(
        "ghs_AlphaToken1234567890",
        Utc::now() + Duration::hours(1),
        "alpha-org",
    )
    .with_repository_selection(RepositorySelection::All);

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    // User attempts to access repository owned by 'beta-corp'
    let outcome = authorizer
        .authorize(&session_id, "beta-corp", "proprietary-code")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::AccountMismatch {
            session_id: sid,
            requested_owner,
            available_accounts,
        } => {
            assert_eq!(*sid, session_id);
            assert_eq!(requested_owner, "beta-corp");
            assert_eq!(available_accounts, &vec!["alpha-org".to_string()]);
        }
        other => panic!("Expected AccountMismatch, got: {other:?}"),
    }

    let plan = coordinator.diagnose_outcome(&outcome, "beta-corp", "proprietary-code");
    assert_eq!(plan.failure_kind, AuthorizationFailureKind::AccountMismatch);
    match plan.prescribed_action {
        RecoveryAction::PromptAppInstallation { target_account, .. } => {
            assert_eq!(target_account, Some("beta-corp".to_string()));
        }
        other => panic!("Expected PromptAppInstallation, got: {other:?}"),
    }
}

/// Test Case 3: Unauthorized access when repository is not granted under Selected mode (local check).
///
/// Verifies that when an installation is configured with Selected repository access and the
/// target repository is neither in the granted cache nor accessible live, access is denied with
/// RepositoryNotGranted and produces a settings update plan.
#[tokio::test]
async fn test_unauthorized_selected_repository_not_granted_local_check() {
    // Spin up mock server returning 404 for check_repository_access
    let app = Router::new().route(
        "/repos/acme-org/restricted-repo",
        get(|| async {
            let body = json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest"
            });
            (StatusCode::NOT_FOUND, Json(body))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let authorizer = InstallationAuthorizer::new(store.clone(), client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();
    let install_id = 2222i64;

    // Granted only 'allowed-service'
    let token_record = StoredTokenRecord::new(
        "ghs_SelectedToken1234567890",
        Utc::now() + Duration::hours(1),
        "acme-org",
    )
    .with_repository_selection(RepositorySelection::Selected)
    .with_repositories(Some(vec!["acme-org/allowed-service".to_string()]));

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    // Query 'restricted-repo'
    let outcome = authorizer
        .authorize(&session_id, "acme-org", "restricted-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::RepositoryNotGranted {
            installation_id,
            account_login,
            requested_repo,
            repository_selection,
        } => {
            assert_eq!(*installation_id, install_id);
            assert_eq!(account_login, "acme-org");
            assert_eq!(requested_repo, "acme-org/restricted-repo");
            assert_eq!(*repository_selection, RepositorySelection::Selected);
        }
        other => panic!("Expected RepositoryNotGranted, got: {other:?}"),
    }

    let plan = coordinator.diagnose_outcome(&outcome, "acme-org", "restricted-repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::RepositoryNotGranted
    );
    match plan.prescribed_action {
        RecoveryAction::PromptRepositorySelectionUpdate {
            settings_url,
            installation_id,
            repository,
        } => {
            assert_eq!(installation_id, install_id);
            assert_eq!(repository, "acme-org/restricted-repo");
            assert!(settings_url.contains(&install_id.to_string()));
        }
        other => panic!("Expected PromptRepositorySelectionUpdate, got: {other:?}"),
    }
}

/// Test Case 4: Unauthorized access when live repository check returns HTTP 403 Forbidden.
///
/// Verifies that if live GitHub check returns 403 Forbidden, access is treated as not granted.
#[tokio::test]
async fn test_unauthorized_selected_repository_forbidden_upstream() {
    let app = Router::new().route(
        "/repos/acme-org/forbidden-service",
        get(|| async {
            let body = json!({
                "message": "Resource not accessible by integration",
                "documentation_url": "https://docs.github.com/rest"
            });
            (StatusCode::FORBIDDEN, Json(body))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let authorizer = InstallationAuthorizer::new(store.clone(), client);

    let session_id = Uuid::new_v4();
    let install_id = 3333i64;

    let token_record = StoredTokenRecord::new(
        "ghs_TokenForbidden123456789",
        Utc::now() + Duration::hours(1),
        "acme-org",
    )
    .with_repository_selection(RepositorySelection::Selected)
    .with_repositories(Some(vec![]));

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    let outcome = authorizer
        .authorize(&session_id, "acme-org", "forbidden-service")
        .await
        .expect("authorization evaluation must complete");

    assert!(matches!(
        outcome,
        AuthorizationOutcome::RepositoryNotGranted { .. }
    ));
}

/// Test Case 5: Unauthorized access when installation is suspended.
///
/// Verifies that suspended installations are immediately blocked with InstallationSuspended
/// and produce an unsuspend guidance plan.
#[tokio::test]
async fn test_unauthorized_suspended_installation() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store.clone(), client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();
    let install_id = 4444i64;

    let token_record = StoredTokenRecord::new(
        "ghs_SuspendedToken1234567890",
        Utc::now() + Duration::hours(1),
        "suspended-corp",
    )
    .with_suspended(true);

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    let outcome = authorizer
        .authorize(&session_id, "suspended-corp", "any-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::InstallationSuspended {
            installation_id,
            account_login,
        } => {
            assert_eq!(*installation_id, install_id);
            assert_eq!(account_login, "suspended-corp");
        }
        other => panic!("Expected InstallationSuspended, got: {other:?}"),
    }

    let plan = coordinator.diagnose_outcome(&outcome, "suspended-corp", "any-repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::InstallationSuspended
    );
    match plan.prescribed_action {
        RecoveryAction::PromptUnsuspendInstallation {
            settings_url,
            installation_id,
            account_login,
        } => {
            assert_eq!(installation_id, install_id);
            assert_eq!(account_login, "suspended-corp");
            assert!(settings_url.contains(&install_id.to_string()));
        }
        other => panic!("Expected PromptUnsuspendInstallation, got: {other:?}"),
    }
}

/// Test Case 6: Unauthorized access when installation is revoked.
///
/// Verifies that revoked installations are blocked with InstallationRevoked and instruct
/// the user to reinstall the app.
#[tokio::test]
async fn test_unauthorized_revoked_installation() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store.clone(), client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();
    let install_id = 5555i64;

    let token_record = StoredTokenRecord::new(
        "ghs_RevokedToken1234567890",
        Utc::now() + Duration::hours(1),
        "revoked-org",
    )
    .with_revoked(true);

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    let outcome = authorizer
        .authorize(&session_id, "revoked-org", "any-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::InstallationRevoked {
            installation_id,
            account_login,
        } => {
            assert_eq!(*installation_id, install_id);
            assert_eq!(account_login, "revoked-org");
        }
        other => panic!("Expected InstallationRevoked, got: {other:?}"),
    }

    let plan = coordinator.diagnose_outcome(&outcome, "revoked-org", "any-repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::InstallationRevoked
    );
    assert!(matches!(
        plan.prescribed_action,
        RecoveryAction::PromptAppInstallation { .. }
    ));
}

/// Test Case 7: Unauthorized access when granted permissions lack contents: read.
///
/// Verifies that a token lacking mandatory contents: read scope is rejected with
/// InsufficientPermissions under Rule 05.
#[tokio::test]
async fn test_unauthorized_policy_violating_permissions_missing_contents_read() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store.clone(), client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let session_id = Uuid::new_v4();
    let install_id = 6666i64;

    // Permissions contain ONLY metadata: read (missing contents: read)
    let mut permissions = HashMap::new();
    permissions.insert("metadata".to_string(), "read".to_string());

    let token_record = StoredTokenRecord::new(
        "ghs_NoContentsReadToken123456",
        Utc::now() + Duration::hours(1),
        "perm-org",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(permissions);

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    let outcome = authorizer
        .authorize(&session_id, "perm-org", "target-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::InsufficientPermissions {
            installation_id,
            account_login,
            violations,
        } => {
            assert_eq!(*installation_id, install_id);
            assert_eq!(account_login, "perm-org");
            assert!(violations
                .iter()
                .any(|v| v.contains("Missing required permission scope 'contents'")));
        }
        other => panic!("Expected InsufficientPermissions, got: {other:?}"),
    }

    let plan = coordinator.diagnose_outcome(&outcome, "perm-org", "target-repo");
    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::InsufficientPermissions
    );
    match plan.prescribed_action {
        RecoveryAction::ContactAdministrator { account_login, .. } => {
            assert_eq!(account_login, "perm-org");
        }
        other => panic!("Expected ContactAdministrator, got: {other:?}"),
    }
}

/// Test Case 8: Unauthorized access when token has forbidden write scopes.
///
/// Verifies that any token granted write/push permissions (e.g. contents: write)
/// is strictly rejected with InsufficientPermissions under Rule 05 read-only guarantee.
#[tokio::test]
async fn test_unauthorized_policy_violating_permissions_forbidden_write_scope() {
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let client = Arc::new(GitHubApiClient::new());
    let authorizer = InstallationAuthorizer::new(store.clone(), client);

    let session_id = Uuid::new_v4();
    let install_id = 7777i64;

    // Dangerous write permission
    let mut permissions = HashMap::new();
    permissions.insert("contents".to_string(), "write".to_string());
    permissions.insert("metadata".to_string(), "read".to_string());

    let token_record = StoredTokenRecord::new(
        "ghs_WriteScopeToken1234567890",
        Utc::now() + Duration::hours(1),
        "write-danger-org",
    )
    .with_repository_selection(RepositorySelection::All)
    .with_permissions(permissions);

    store
        .save_token_record(&session_id, install_id, token_record)
        .await
        .unwrap();

    let outcome = authorizer
        .authorize(&session_id, "write-danger-org", "target-repo")
        .await
        .expect("authorization evaluation must complete");

    match &outcome {
        AuthorizationOutcome::InsufficientPermissions { violations, .. } => {
            assert!(violations.iter().any(|v| v
                .contains("Excessive permission level for scope 'contents'")
                && v.contains("granted 'write'")));
        }
        other => panic!("Expected InsufficientPermissions, got: {other:?}"),
    }
}

/// Test Case 9: Extraction failure when commit resolution returns HTTP 403 Forbidden.
///
/// Verifies that if upstream GitHub responds with 403 during extraction, PrivateRepositoryExtractor
/// halts immediately with sanitized error and recovery coordinator diagnoses actionable remediation.
#[tokio::test]
async fn test_unauthorized_private_extractor_commit_403_resource_not_accessible() {
    let app = Router::new().route(
        "/repos/secure-org/private-repo/commits/HEAD",
        get(|| async {
            let body = json!({
                "message": "Resource not accessible by integration",
                "documentation_url": "https://docs.github.com/rest"
            });
            (StatusCode::FORBIDDEN, Json(body))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let extractor = PrivateRepositoryExtractor::new(client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let ctx = AuthorizedInstallationContext::new(
        "secure-org",
        "private-repo",
        8888,
        "secure-org",
        "ghs_ValidTokenBlockedUpstream123",
        RepositorySelection::Selected,
    );

    let result = extractor.extract(&ctx, None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();

    // Diagnose extractor error
    let plan = coordinator.diagnose_extractor_error(
        &err,
        "secure-org",
        "private-repo",
        Some(8888),
        Some("secure-org"),
    );

    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::InsufficientPermissions
    );
    match plan.prescribed_action {
        RecoveryAction::ContactAdministrator { account_login, .. } => {
            assert_eq!(account_login, "secure-org");
        }
        other => panic!("Expected ContactAdministrator, got: {other:?}"),
    }
}

/// Test Case 10: Extraction failure when private repository commit returns HTTP 404 Not Found.
///
/// Verifies that HTTP 404 on private repositories is diagnosed as RepositoryNotGranted with
/// a prompt to update repository selection.
#[tokio::test]
async fn test_unauthorized_private_extractor_commit_404_not_found() {
    let app = Router::new().route(
        "/repos/secure-org/unauthorized-repo/commits/HEAD",
        get(|| async {
            let body = json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest"
            });
            (StatusCode::NOT_FOUND, Json(body))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let extractor = PrivateRepositoryExtractor::new(client);
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let ctx = AuthorizedInstallationContext::new(
        "secure-org",
        "unauthorized-repo",
        9999,
        "secure-org",
        "ghs_ValidTokenNotFound12345",
        RepositorySelection::Selected,
    );

    let result = extractor.extract(&ctx, None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();

    let plan = coordinator.diagnose_extractor_error(
        &err,
        "secure-org",
        "unauthorized-repo",
        Some(9999),
        Some("secure-org"),
    );

    assert_eq!(
        plan.failure_kind,
        AuthorizationFailureKind::RepositoryNotGranted
    );
    match plan.prescribed_action {
        RecoveryAction::PromptRepositorySelectionUpdate {
            installation_id,
            repository,
            ..
        } => {
            assert_eq!(installation_id, 9999);
            assert_eq!(repository, "secure-org/unauthorized-repo");
        }
        other => panic!("Expected PromptRepositorySelectionUpdate, got: {other:?}"),
    }
}

/// Test Case 11: Zero secret leakage across unauthorized failure modes.
///
/// Verifies that sensitive bearer tokens and credentials are never leaked in error
/// strings, diagnostic plans, or formatted summary cards under any unauthorized failure.
#[test]
fn test_unauthorized_zero_secret_leakage_in_diagnoses_and_cards() {
    let app_config = Arc::new(test_app_config());
    let coordinator = AuthorizationRecoveryCoordinator::new().with_app_config(app_config);

    let secret_token = "ghs_SecretInstallationToken1234567890ABCDEF";

    // 1. BadCredentials error with secret token in raw response
    let raw_error = GitHubExtractorError::BadCredentials {
        message: format!("Invalid header authorization: Bearer {secret_token}"),
    };
    let plan = coordinator.diagnose_extractor_error(
        &raw_error,
        "victim-org",
        "confidential-repo",
        Some(10101),
        Some("victim-org"),
    );

    let summary = plan.summary_card();
    assert!(
        !summary.contains(secret_token),
        "Summary card must redact secret tokens"
    );
    assert!(
        !plan.diagnosis.contains(secret_token),
        "Diagnosis must redact secret tokens"
    );

    // 2. ResourceNotFound with token in query or body
    let not_found_err = GitHubExtractorError::ResourceNotFound {
        url: format!("https://api.github.com/repos/victim/repo?token={secret_token}"),
        message: format!("Token {secret_token} unauthorized"),
    };
    let plan2 = coordinator.diagnose_extractor_error(
        &not_found_err,
        "victim",
        "repo",
        Some(10101),
        Some("victim"),
    );
    assert!(!plan2.summary_card().contains(secret_token));
    assert!(!plan2.diagnosis.contains(secret_token));

    // 3. AuthorizedInstallationContext debug representation zero-leakage check
    let ctx = AuthorizedInstallationContext::new(
        "victim",
        "repo",
        10101,
        "victim",
        secret_token,
        RepositorySelection::Selected,
    );
    let debug_str = format!("{ctx:?}");
    assert!(!debug_str.contains(secret_token));
    assert!(debug_str.contains("***[REDACTED_TOKEN]***"));
}
