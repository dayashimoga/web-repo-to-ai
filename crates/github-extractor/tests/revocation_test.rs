//! Integration test suite for GitHub App installation token revocation and lifecycle event synchronization.

use axum::http::StatusCode;
use axum::routing::delete;
use axum::Router;
use chrono::{Duration, Utc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::callback::{GitHubWebhookEvent, InstallationAction};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::installation::{
    GitHubAccountSummary, GitHubInstallation, GitHubInstallationToken, RepositorySelection,
};
use web_repo_to_ai_github_extractor::revocation::{
    RevocationOutcome, RevocationSummary, TokenRevocationCoordinator, WebhookLifecycleAction,
};
use web_repo_to_ai_github_extractor::token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenStore,
};

#[tokio::test]
async fn test_upstream_token_revocation_success_204() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = call_count.clone();

    // Mock GitHub DELETE /installation/token endpoint
    let app = Router::new().route(
        "/installation/token",
        delete(move || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                StatusCode::NO_CONTENT
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base_url = format!("http://{addr}");
    let client = GitHubApiClient::new().with_base_url_override(mock_base_url);

    let res = client
        .revoke_installation_access_token("ghs_ValidToken123")
        .await;

    assert!(res.is_ok());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_upstream_token_revocation_idempotent_on_401_or_404() {
    // Mock GitHub returning 401 Unauthorized (already expired/revoked)
    let app_401 = Router::new().route(
        "/installation/token",
        delete(|| async { StatusCode::UNAUTHORIZED }),
    );

    let listener_401 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_401 = listener_401.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener_401, app_401).await.unwrap();
    });

    let client_401 = GitHubApiClient::new().with_base_url_override(format!("http://{addr_401}"));
    assert!(client_401
        .revoke_installation_access_token("ghs_ExpiredToken")
        .await
        .is_ok());

    // Mock GitHub returning 404 Not Found
    let app_404 = Router::new().route(
        "/installation/token",
        delete(|| async { StatusCode::NOT_FOUND }),
    );

    let listener_404 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_404 = listener_404.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener_404, app_404).await.unwrap();
    });

    let client_404 = GitHubApiClient::new().with_base_url_override(format!("http://{addr_404}"));
    assert!(client_404
        .revoke_installation_access_token("ghs_MissingToken")
        .await
        .is_ok());
}

#[tokio::test]
async fn test_upstream_token_revocation_server_error() {
    let app_500 = Router::new().route(
        "/installation/token",
        delete(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "GitHub Error") }),
    );

    let listener_500 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_500 = listener_500.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener_500, app_500).await.unwrap();
    });

    let client_500 = GitHubApiClient::new().with_base_url_override(format!("http://{addr_500}"));
    let res = client_500
        .revoke_installation_access_token("ghs_TokenServerError")
        .await;

    assert!(res.is_err());
}

#[tokio::test]
async fn test_coordinator_revoke_token_and_cache() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let app = Router::new().route(
        "/installation/token",
        delete(move || {
            let count = count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                StatusCode::NO_CONTENT
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let session = Uuid::new_v4();
    let install_id = 991122i64;

    // Save token in store
    let token = GitHubInstallationToken {
        token: "ghs_ActiveCoordinatorToken".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };
    store
        .save_token(&session, install_id, "test-user", &token)
        .await
        .unwrap();

    let coordinator = TokenRevocationCoordinator::new(client, store.clone());

    // Execute coordinated revocation
    let outcome = coordinator
        .revoke_token_and_cache(&session, install_id)
        .await
        .unwrap();

    assert_eq!(outcome, RevocationOutcome::RevokedUpstreamAndLocally);
    assert!(outcome.is_revoked());
    assert!(outcome.is_upstream_revoked());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    // Verify token evicted from store
    assert!(store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .is_none());

    // Secondary revocation when already evicted returns NotFound
    let outcome_again = coordinator
        .revoke_token_and_cache(&session, install_id)
        .await
        .unwrap();
    assert_eq!(outcome_again, RevocationOutcome::NotFound);
}

#[tokio::test]
async fn test_coordinator_webhook_event_installation_deleted() {
    let client = Arc::new(GitHubApiClient::new());
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let session1 = Uuid::new_v4();
    let session2 = Uuid::new_v4();
    let install_id = 778899i64;

    let token = GitHubInstallationToken {
        token: "ghs_TokenForDeletedApp".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };

    store
        .save_token(&session1, install_id, "deleted-org", &token)
        .await
        .unwrap();
    store
        .save_token(&session2, install_id, "deleted-org", &token)
        .await
        .unwrap();

    let coordinator = TokenRevocationCoordinator::new(client, store.clone());

    let webhook_event = GitHubWebhookEvent::Installation {
        action: InstallationAction::Deleted,
        installation: GitHubInstallation {
            id: install_id as u64,
            account: GitHubAccountSummary {
                login: "deleted-org".to_string(),
                id: 112233,
                avatar_url: None,
                html_url: None,
                account_type: "Organization".to_string(),
            },
            repository_selection: RepositorySelection::All,
            access_tokens_url: "".to_string(),
            repositories_url: "".to_string(),
            html_url: "".to_string(),
            app_id: 12345,
            target_id: 112233,
            target_type: "Organization".to_string(),
            permissions: Default::default(),
            events: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            suspended_by: None,
            suspended_at: None,
            app_slug: Some("test-app".to_string()),
        },
    };

    let action = coordinator
        .handle_webhook_event(&webhook_event)
        .await
        .unwrap();

    match action {
        WebhookLifecycleAction::InstallationDeleted {
            installation_id,
            account_login,
            evicted_cached_tokens,
        } => {
            assert_eq!(installation_id, install_id);
            assert_eq!(account_login, "deleted-org");
            assert_eq!(evicted_cached_tokens, 2);
        }
        _ => panic!("Expected InstallationDeleted"),
    }

    // Both sessions should now have empty cache
    assert!(store
        .get_token(&session1, install_id)
        .await
        .unwrap()
        .is_none());
    assert!(store
        .get_token(&session2, install_id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_coordinator_webhook_event_installation_suspended() {
    let client = Arc::new(GitHubApiClient::new());
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let session = Uuid::new_v4();
    let install_id = 334455i64;

    let token = GitHubInstallationToken {
        token: "ghs_TokenForSuspendedApp".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        permissions: [("contents".to_string(), "read".to_string())].into(),
        repository_selection: Default::default(),
        repositories: None,
    };
    store
        .save_token(&session, install_id, "suspended-user", &token)
        .await
        .unwrap();

    let coordinator = TokenRevocationCoordinator::new(client, store.clone());

    let webhook_event = GitHubWebhookEvent::Installation {
        action: InstallationAction::Suspend,
        installation: GitHubInstallation {
            id: install_id as u64,
            account: GitHubAccountSummary {
                login: "suspended-user".to_string(),
                id: 445566,
                avatar_url: None,
                html_url: None,
                account_type: "User".to_string(),
            },
            repository_selection: RepositorySelection::All,
            access_tokens_url: "".to_string(),
            repositories_url: "".to_string(),
            html_url: "".to_string(),
            app_id: 12345,
            target_id: 445566,
            target_type: "User".to_string(),
            permissions: Default::default(),
            events: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            suspended_by: None,
            suspended_at: Some(Utc::now()),
            app_slug: Some("test-app".to_string()),
        },
    };

    let action = coordinator
        .handle_webhook_event(&webhook_event)
        .await
        .unwrap();

    match action {
        WebhookLifecycleAction::InstallationSuspended {
            installation_id,
            account_login,
            evicted_cached_tokens,
        } => {
            assert_eq!(installation_id, install_id);
            assert_eq!(account_login, "suspended-user");
            assert_eq!(evicted_cached_tokens, 1);
        }
        _ => panic!("Expected InstallationSuspended"),
    }

    assert!(store
        .get_token(&session, install_id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_revocation_summary_model() {
    let session = Uuid::new_v4();
    let install_id = 123456i64;

    let summary = RevocationSummary::new(
        session,
        install_id,
        RevocationOutcome::RevokedUpstreamAndLocally,
        Some("octocat".to_string()),
    );

    assert_eq!(summary.session_id, session);
    assert_eq!(summary.installation_id, install_id);
    assert_eq!(
        summary.outcome,
        RevocationOutcome::RevokedUpstreamAndLocally
    );
    assert!(summary.upstream_revoked);
    assert_eq!(summary.account_login.as_deref(), Some("octocat"));

    let json_str = serde_json::to_string(&summary).unwrap();
    assert!(json_str.contains("revoked_upstream_and_locally"));
    assert!(json_str.contains("octocat"));
}
