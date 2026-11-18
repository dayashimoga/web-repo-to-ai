//! Integration test suite for selected repository access, traversal, search,
//! access verification, and dynamic grant synchronization (Subphase 6.3.2).

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::installation::{
    InstalledRepositorySummary, RepositorySelection,
};
use web_repo_to_ai_github_extractor::selected_repositories::{
    RepositoryAccessStatus, SelectedRepositoryListing, SelectedRepositoryManager,
};
use web_repo_to_ai_github_extractor::token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenStore, StoredTokenRecord,
};

#[derive(Deserialize)]
#[allow(dead_code)]
struct PaginationQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

fn sample_summary(
    id: u64,
    name: &str,
    full_name: &str,
    private: bool,
) -> InstalledRepositorySummary {
    InstalledRepositorySummary {
        id,
        node_id: format!("node_{id}"),
        name: name.to_string(),
        full_name: full_name.to_string(),
        private,
        default_branch: "main".to_string(),
        description: Some(format!("Description for {name}")),
        html_url: format!("https://github.com/{full_name}"),
    }
}

#[tokio::test]
async fn test_fetch_all_single_page() {
    let app = Router::new().route(
        "/installation/repositories",
        get(|| async {
            (
                StatusCode::OK,
                Json(json!({
                    "total_count": 2,
                    "repository_selection": "selected",
                    "repositories": [
                        {
                            "id": 101,
                            "name": "core-engine",
                            "full_name": "acme-corp/core-engine",
                            "private": true,
                            "default_branch": "main",
                            "description": "Core processing engine",
                            "html_url": "https://github.com/acme-corp/core-engine"
                        },
                        {
                            "id": 102,
                            "name": "docs-site",
                            "full_name": "acme-corp/docs-site",
                            "private": false,
                            "default_branch": "main",
                            "description": "Public documentation site",
                            "html_url": "https://github.com/acme-corp/docs-site"
                        }
                    ]
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
    let manager = SelectedRepositoryManager::new(client);

    let listing = manager.fetch_all("ghs_TestToken").await.unwrap();

    assert_eq!(listing.total_count, 2);
    assert_eq!(listing.repository_selection, RepositorySelection::Selected);
    assert_eq!(listing.len(), 2);
    assert_eq!(listing.pages_fetched, 1);
    assert!(listing.is_complete);
    assert!(listing.has_repository("acme-corp", "core-engine"));
    assert!(listing.has_repository("acme-corp", "docs-site"));
    assert!(!listing.has_repository("acme-corp", "non-existent"));
}

#[tokio::test]
async fn test_fetch_all_multi_page_pagination() {
    let page_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = page_counter.clone();

    // 5 repositories across 3 pages (per_page=2)
    let app = Router::new().route(
        "/installation/repositories",
        get(move |Query(params): Query<PaginationQuery>| {
            let page = params.page.unwrap_or(1);
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                let repos = match page {
                    1 => vec![
                        json!({"id": 1, "name": "repo1", "full_name": "org/repo1", "private": true}),
                        json!({"id": 2, "name": "repo2", "full_name": "org/repo2", "private": false}),
                    ],
                    2 => vec![
                        json!({"id": 3, "name": "repo3", "full_name": "org/repo3", "private": true}),
                        json!({"id": 4, "name": "repo4", "full_name": "org/repo4", "private": false}),
                    ],
                    3 => vec![
                        json!({"id": 5, "name": "repo5", "full_name": "org/repo5", "private": true}),
                    ],
                    _ => vec![],
                };

                (
                    StatusCode::OK,
                    Json(json!({
                        "total_count": 5,
                        "repository_selection": "selected",
                        "repositories": repos
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

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let manager = SelectedRepositoryManager::new(client).with_per_page(2);

    let listing = manager.fetch_all("ghs_PaginatedToken").await.unwrap();

    assert_eq!(listing.total_count, 5);
    assert_eq!(listing.len(), 5);
    assert_eq!(listing.pages_fetched, 3);
    assert!(listing.is_complete);
    assert_eq!(page_counter.load(Ordering::SeqCst), 3);
    assert_eq!(
        listing.full_names(),
        vec![
            "org/repo1",
            "org/repo2",
            "org/repo3",
            "org/repo4",
            "org/repo5"
        ]
    );
}

#[tokio::test]
async fn test_fetch_all_reaches_max_pages_cap() {
    let app = Router::new().route(
        "/installation/repositories",
        get(|Query(params): Query<PaginationQuery>| async move {
            let page = params.page.unwrap_or(1);
            let repos = vec![
                json!({"id": page * 10 + 1, "name": format!("repo{}", page * 10 + 1), "full_name": format!("org/repo{}", page * 10 + 1), "private": true}),
                json!({"id": page * 10 + 2, "name": format!("repo{}", page * 10 + 2), "full_name": format!("org/repo{}", page * 10 + 2), "private": false}),
            ];
            (
                StatusCode::OK,
                Json(json!({
                    "total_count": 20, // 20 total, but cap at 2 pages = 4 items
                    "repository_selection": "selected",
                    "repositories": repos
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
    let manager = SelectedRepositoryManager::new(client)
        .with_per_page(2)
        .with_max_pages(2);

    let listing = manager.fetch_all("ghs_CappedToken").await.unwrap();

    assert_eq!(listing.total_count, 20);
    assert_eq!(listing.len(), 4);
    assert_eq!(listing.pages_fetched, 2);
    assert!(!listing.is_complete); // Hit max pages cap
}

#[tokio::test]
async fn test_fetch_all_for_user() {
    let app = Router::new().route(
        "/user/installations/:id/repositories",
        get(|Path(inst_id): Path<u64>| async move {
            if inst_id == 778899 {
                (
                    StatusCode::OK,
                    Json(json!({
                        "total_count": 1,
                        "repository_selection": "selected",
                        "repositories": [
                            {
                                "id": 555,
                                "name": "user-app",
                                "full_name": "developer/user-app",
                                "private": true,
                                "default_branch": "main"
                            }
                        ]
                    })),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"message": "Installation not found"})),
                )
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let manager = SelectedRepositoryManager::new(client);

    let listing = manager
        .fetch_all_for_user("ghu_UserOAuthToken", 778899)
        .await
        .unwrap();

    assert_eq!(listing.total_count, 1);
    assert_eq!(listing.len(), 1);
    assert_eq!(listing.repositories[0].full_name, "developer/user-app");
}

#[test]
fn test_selected_repository_search_and_filter() {
    let listing = SelectedRepositoryListing {
        total_count: 4,
        repository_selection: RepositorySelection::Selected,
        repositories: vec![
            sample_summary(1, "api-gateway", "acme/api-gateway", true),
            sample_summary(2, "web-frontend", "acme/web-frontend", false),
            sample_summary(3, "auth-service", "acme/auth-service", true),
            sample_summary(4, "documentation", "acme/documentation", false),
        ],
        pages_fetched: 1,
        is_complete: true,
    };

    // Keyword search
    let api_results = listing.search("api");
    assert_eq!(api_results.len(), 1);
    assert_eq!(api_results[0].name, "api-gateway");

    let service_results = listing.search("service");
    assert_eq!(service_results.len(), 1);
    assert_eq!(service_results[0].name, "auth-service");

    let empty_search = listing.search("");
    assert_eq!(empty_search.len(), 4);

    // Visibility filter
    let private_repos = listing.filter_by_visibility(true);
    assert_eq!(private_repos.len(), 2);
    assert_eq!(private_repos[0].name, "api-gateway");
    assert_eq!(private_repos[1].name, "auth-service");

    let public_repos = listing.filter_by_visibility(false);
    assert_eq!(public_repos.len(), 2);
    assert_eq!(public_repos[0].name, "web-frontend");
    assert_eq!(public_repos[1].name, "documentation");

    // Slug lookup
    let found = listing.find_by_slug("ACME", "API-GATEWAY");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 1);

    assert!(listing.find_by_slug("acme", "unknown").is_none());
}

#[tokio::test]
async fn test_verify_repository_access_granted() {
    let app = Router::new().route(
        "/repos/:owner/:repo",
        get(|Path((owner, repo)): Path<(String, String)>| async move {
            if owner == "acme" && repo == "verified-repo" {
                (
                    StatusCode::OK,
                    Json(json!({
                        "id": 999,
                        "name": "verified-repo",
                        "full_name": "acme/verified-repo",
                        "private": true,
                        "default_branch": "main",
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
    let manager = SelectedRepositoryManager::new(client);

    let status = manager
        .verify_repository_access("ghs_CheckToken", "acme", "verified-repo")
        .await
        .unwrap();

    assert!(status.is_granted());
    let repo_summary = status.unwrap_granted();
    assert_eq!(repo_summary.id, 999);
    assert_eq!(repo_summary.full_name, "acme/verified-repo");
}

#[tokio::test]
async fn test_verify_repository_access_not_granted() {
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
    let manager = SelectedRepositoryManager::new(client);

    let status = manager
        .verify_repository_access("ghs_CheckToken", "acme", "forbidden-repo")
        .await
        .unwrap();

    match status {
        RepositoryAccessStatus::NotGranted { owner, repo, .. } => {
            assert_eq!(owner, "acme");
            assert_eq!(repo, "forbidden-repo");
        }
        other => panic!("Expected NotGranted, got {other:?}"),
    }
}

#[tokio::test]
async fn test_verify_repository_access_token_invalid_401() {
    let app = Router::new().route(
        "/repos/:owner/:repo",
        get(|| async {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "Bad credentials"})),
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let manager = SelectedRepositoryManager::new(client);

    let status = manager
        .verify_repository_access("ghs_BadToken", "acme", "any-repo")
        .await
        .unwrap();

    match status {
        RepositoryAccessStatus::TokenInvalid { reason } => {
            assert!(reason.contains("Bad credentials") || reason.contains("401"));
        }
        other => panic!("Expected TokenInvalid, got {other:?}"),
    }
}

#[tokio::test]
async fn test_sync_grant_changes_added_and_removed() {
    let session_id = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record =
        StoredTokenRecord::new("ghs_SyncToken", Utc::now() + Duration::hours(1), "acme-org")
            .with_repository_selection(RepositorySelection::Selected)
            .with_repositories(vec![
                "acme-org/repo1".to_string(),
                "acme-org/repo2".to_string(),
            ]);

    store
        .save_token_record(&session_id, 4455, record)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let manager = SelectedRepositoryManager::new(client);

    let added = vec![sample_summary(3, "repo3", "acme-org/repo3", false)];
    let removed = vec![sample_summary(1, "repo1", "acme-org/repo1", true)];

    let sync_result = manager
        .sync_grant_change(&*store, 4455, &added, &removed)
        .await
        .unwrap();

    assert_eq!(sync_result.installation_id, 4455);
    assert_eq!(sync_result.added_count, 1);
    assert_eq!(sync_result.removed_count, 1);
    assert_eq!(sync_result.updated_sessions_count, 1);

    // Verify updated store state
    let updated = store.get_token(&session_id, 4455).await.unwrap().unwrap();
    assert!(!updated.has_repository("acme-org", "repo1"));
    assert!(updated.has_repository("acme-org", "repo2"));
    assert!(updated.has_repository("acme-org", "repo3"));
}

#[tokio::test]
async fn test_sync_grant_changes_multi_session() {
    let session1 = Uuid::new_v4();
    let session2 = Uuid::new_v4();
    let store = Arc::new(InMemoryInstallationTokenStore::new());

    let record1 = StoredTokenRecord::new("ghs_Tok1", Utc::now() + Duration::hours(1), "org")
        .with_repository_selection(RepositorySelection::Selected)
        .with_repositories(vec!["org/alpha".to_string()]);

    let record2 = StoredTokenRecord::new("ghs_Tok2", Utc::now() + Duration::hours(1), "org")
        .with_repository_selection(RepositorySelection::Selected)
        .with_repositories(vec!["org/alpha".to_string()]);

    store
        .save_token_record(&session1, 888, record1)
        .await
        .unwrap();
    store
        .save_token_record(&session2, 888, record2)
        .await
        .unwrap();

    let client = Arc::new(GitHubApiClient::new());
    let manager = SelectedRepositoryManager::new(client);

    let added = vec![sample_summary(2, "beta", "org/beta", true)];
    let removed = vec![];

    let summary = manager
        .sync_grant_change(&*store, 888, &added, &removed)
        .await
        .unwrap();

    assert_eq!(summary.updated_sessions_count, 2);

    let rec1 = store.get_token(&session1, 888).await.unwrap().unwrap();
    let rec2 = store.get_token(&session2, 888).await.unwrap().unwrap();
    assert!(rec1.has_repository("org", "beta"));
    assert!(rec2.has_repository("org", "beta"));
}

#[tokio::test]
async fn test_empty_repository_listing() {
    let app = Router::new().route(
        "/installation/repositories",
        get(|| async {
            (
                StatusCode::OK,
                Json(json!({
                    "total_count": 0,
                    "repository_selection": "selected",
                    "repositories": []
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
    let manager = SelectedRepositoryManager::new(client);

    let listing = manager.fetch_all("ghs_EmptyToken").await.unwrap();

    assert_eq!(listing.total_count, 0);
    assert!(listing.is_empty());
    assert_eq!(listing.len(), 0);
    assert_eq!(listing.pages_fetched, 1);
    assert!(listing.is_complete);
}
