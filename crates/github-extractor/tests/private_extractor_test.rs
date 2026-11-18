//! Integration test suite for private repository tree and content extraction (Subphase 6.3.3).
//!
//! Verifies:
//! - End-to-end private repository extraction with ephemeral token authentication.
//! - Mathematical conservation identity (Discovered = Extracted + Excluded + Failed) with zero discrepancies.
//! - Bounded concurrent extraction across sources, docs, dependencies, configs, and tests.
//! - Multi-algorithm cryptographic hash validation (Git SHA-1, SHA-256, BLAKE3).
//! - Subtree recovery integration on truncated git trees.
//! - Proactive token renewal with `InstallationTokenRefresher`.
//! - Security invariants: zero bearer token leakage in errors, results, or debug traces.

use axum::extract::Path;
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

use web_repo_to_ai_github_extractor::app_manifest::GitHubAppConfig;
use web_repo_to_ai_github_extractor::authorization::AuthorizedInstallationContext;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
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
        .build()
        .expect("valid test app config")
}

#[tokio::test]
async fn test_private_extraction_success_full_flow() {
    let rust_code = "fn main() {\n    println!(\"Hello, private world!\");\n}\n";
    let doc_content = "# Acme Core\nProprietary internal system documentation.\n";
    let cargo_toml = "[package]\nname = \"secret-core\"\nversion = \"1.0.0\"\nedition = \"2021\"\n";
    let config_toml = "max_width = 100\nedition = \"2021\"\n";
    let test_code = "#[test]\nfn test_private() {\n    assert!(true);\n}\n";

    let rust_sha = compute_git_blob_sha(rust_code.as_bytes());
    let doc_sha = compute_git_blob_sha(doc_content.as_bytes());
    let cargo_sha = compute_git_blob_sha(cargo_toml.as_bytes());
    let config_sha = compute_git_blob_sha(config_toml.as_bytes());
    let test_sha = compute_git_blob_sha(test_code.as_bytes());

    let mut blobs: HashMap<String, Vec<u8>> = HashMap::new();
    blobs.insert(rust_sha.clone(), rust_code.as_bytes().to_vec());
    blobs.insert(doc_sha.clone(), doc_content.as_bytes().to_vec());
    blobs.insert(cargo_sha.clone(), cargo_toml.as_bytes().to_vec());
    blobs.insert(config_sha.clone(), config_toml.as_bytes().to_vec());
    blobs.insert(test_sha.clone(), test_code.as_bytes().to_vec());

    let blobs = Arc::new(blobs);
    let blobs_clone = Arc::clone(&blobs);

    let commit_sha = "1111222233334444555566667777888899990000";
    let tree_sha = "aaaabbbbccccddddeeeeffff0000111122223333";

    let app = Router::new()
        .route(
            "/repos/acme-corp/secret-core/commits/HEAD",
            get(move || async move {
                let body = json!({
                    "sha": commit_sha,
                    "node_id": "C_node_id_123",
                    "commit": {
                        "message": "feat(core): initial private release",
                        "author": {
                            "name": "Acme Engineer",
                            "email": "eng@acme.com",
                            "date": "2026-03-01T10:00:00Z"
                        },
                        "committer": {
                            "name": "Acme Engineer",
                            "email": "eng@acme.com",
                            "date": "2026-03-01T10:00:00Z"
                        },
                        "tree": {
                            "sha": tree_sha,
                            "url": format!("https://api.github.com/repos/acme-corp/secret-core/git/trees/{tree_sha}")
                        },
                        "comment_count": 0
                    },
                    "parents": []
                });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            "/repos/acme-corp/secret-core",
            get(|| async {
                let body = json!({
                    "id": 888999,
                    "node_id": "R_node_id_888",
                    "name": "secret-core",
                    "full_name": "acme-corp/secret-core",
                    "owner": {
                        "login": "acme-corp"
                    },
                    "description": "Proprietary high-throughput AI service",
                    "fork": false,
                    "archived": false,
                    "disabled": false,
                    "visibility": "private",
                    "default_branch": "main",
                    "homepage": null,
                    "language": "Rust",
                    "topics": ["ai", "private", "enterprise"],
                    "license": null,
                    "stargazers_count": 5,
                    "watchers_count": 5,
                    "forks_count": 0,
                    "open_issues_count": 0,
                    "size": 1024,
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-03-01T10:00:00Z",
                    "pushed_at": "2026-03-01T10:00:00Z"
                });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            &format!("/repos/acme-corp/secret-core/git/trees/{tree_sha}"),
            get({
                let r_sha = rust_sha.clone();
                let d_sha = doc_sha.clone();
                let c_sha = cargo_sha.clone();
                let cfg_sha = config_sha.clone();
                let t_sha = test_sha.clone();
                move || {
                    let r = r_sha.clone();
                    let d = d_sha.clone();
                    let c = c_sha.clone();
                    let cfg = cfg_sha.clone();
                    let t = t_sha.clone();
                    async move {
                        let body = json!({
                            "sha": tree_sha,
                            "url": "https://api.github.com/repos/acme-corp/secret-core/git/trees/root",
                            "truncated": false,
                            "tree": [
                                {
                                    "path": "src/main.rs",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": r,
                                    "size": rust_code.len()
                                },
                                {
                                    "path": "README.md",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": d,
                                    "size": doc_content.len()
                                },
                                {
                                    "path": "Cargo.toml",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": c,
                                    "size": cargo_toml.len()
                                },
                                {
                                    "path": "rustfmt.toml",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": cfg,
                                    "size": config_toml.len()
                                },
                                {
                                    "path": "tests/test_basic.rs",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": t,
                                    "size": test_code.len()
                                }
                            ]
                        });
                        (StatusCode::OK, Json(body))
                    }
                }
            }),
        )
        .route(
            "/repos/acme-corp/secret-core/git/blobs/:sha",
            get(move |Path(sha): Path<String>| {
                let blobs_map = Arc::clone(&blobs_clone);
                async move {
                    if let Some(content) = blobs_map.get(&sha) {
                        let mut headers = AxumHeaderMap::new();
                        headers.insert(
                            axum::http::header::CONTENT_TYPE,
                            "application/vnd.github.raw".parse().unwrap(),
                        );
                        (StatusCode::OK, headers, content.clone()).into_response()
                    } else {
                        (StatusCode::NOT_FOUND, "Blob not found").into_response()
                    }
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
        "acme-corp",
        "secret-core",
        456,
        "acme-corp",
        "ghs_ephemeral_token_xyz",
        RepositorySelection::Selected,
    );

    let result = extractor
        .extract(&ctx, None)
        .await
        .expect("private extraction must succeed");

    assert_eq!(result.target.owner, "acme-corp");
    assert_eq!(result.target.repo, "secret-core");
    assert_eq!(result.installation_id, 456);
    assert_eq!(result.account_login, "acme-corp");
    assert_eq!(result.commit_sha(), commit_sha);
    assert_eq!(result.root_tree_sha(), tree_sha);

    // Verify mathematical conservation identity
    assert!(result.is_balanced());
    assert!(result.accounting.is_balanced());
    assert_eq!(result.accounting.discrepancies.len(), 0);

    // Verify file counts across categories
    assert_eq!(result.total_discovered_entries(), 5);
    assert_eq!(result.total_extracted_files(), 5);
    assert_eq!(result.source_report.extracted.len(), 1);
    assert_eq!(result.doc_report.extracted.len(), 1);
    assert_eq!(result.dependency_report.extracted.len(), 1);
    assert_eq!(result.config_test_report.extracted_configs.len(), 1);
    assert_eq!(result.config_test_report.extracted_tests.len(), 1);

    // Verify byte accounting
    let expected_bytes = (rust_code.len()
        + doc_content.len()
        + cargo_toml.len()
        + config_toml.len()
        + test_code.len()) as u64;
    assert_eq!(result.total_extracted_bytes(), expected_bytes);

    // Verify multi-algorithm cryptographic hash validation
    assert!(result.hash_validation.is_valid());
    assert_eq!(result.hash_validation.failures.len(), 0);

    // Verify manifest and architecture context
    assert_eq!(result.manifest.files.len(), 5);
    assert!(result.metadata.is_some());
    assert_eq!(
        result.metadata.as_ref().unwrap().description.as_deref(),
        Some("Proprietary high-throughput AI service")
    );

    // Verify Display formatting
    let display_str = format!("{result}");
    assert!(display_str.contains("acme-corp/secret-core"));
    assert!(display_str.contains("extracted_files=5"));
    assert!(display_str.contains("balanced=true"));
}

#[tokio::test]
async fn test_private_extraction_custom_ref_and_no_metadata() {
    let code = "pub fn secret_algorithm() -> i32 { 999 }\n";
    let code_sha = compute_git_blob_sha(code.as_bytes());

    let commit_sha = "abcdef112233445566778899aabbccddeeff0011";
    let tree_sha = "tree99887766554433221100ffeeddccbbaa9988";

    let c_sha = code_sha.clone();
    let app = Router::new()
        .route(
            "/repos/corp/lib/commits/v2.5.0",
            get(move || async move {
                let body = json!({
                    "sha": commit_sha,
                    "node_id": "C_custom_ref_123",
                    "commit": {
                        "message": "release: v2.5.0",
                        "author": { "name": "Dev", "email": "dev@corp.com", "date": "2026-03-01T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@corp.com", "date": "2026-03-01T00:00:00Z" },
                        "tree": { "sha": tree_sha, "url": "https://api.github.com/tree" },
                        "comment_count": 0
                    },
                    "parents": []
                });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            &format!("/repos/corp/lib/git/trees/{tree_sha}"),
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
            "/repos/corp/lib/git/blobs/:sha",
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
    let config = PrivateExtractionConfig::default()
        .with_target_ref("v2.5.0")
        .with_fetch_metadata(false);

    let extractor = PrivateRepositoryExtractor::new(client).with_config(config);

    let ctx = AuthorizedInstallationContext::new(
        "corp",
        "lib",
        1001,
        "corp",
        "ghs_token_v2",
        RepositorySelection::Selected,
    );

    let result = extractor
        .extract(&ctx, None)
        .await
        .expect("must succeed with custom ref");

    assert_eq!(result.commit_sha(), commit_sha);
    assert_eq!(result.total_extracted_files(), 1);
    assert!(result.metadata.is_none());
    assert!(result.is_balanced());
}

#[tokio::test]
async fn test_private_extraction_with_proactive_token_renewal() {
    let app_config = Arc::new(test_app_config());
    let store = Arc::new(InMemoryInstallationTokenStore::new());
    let session_id = Uuid::new_v4();
    let installation_id = 999;

    let refresh_count = Arc::new(AtomicUsize::new(0));
    let refresh_count_clone = Arc::clone(&refresh_count);

    let code = "pub fn add_one(x: i32) -> i32 { x + 1 }\n";
    let code_sha = compute_git_blob_sha(code.as_bytes());
    let commit_sha = "ccccdddd44445555666677778888999900001111";
    let tree_sha = "eeeeffff00001111222233334444555566667777";

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
                        "token": "ghs_freshly_renewed_token_xyz",
                        "expires_at": expires_at.to_rfc3339(),
                        "permissions": { "contents": "read", "metadata": "read" },
                        "repository_selection": "selected"
                    });
                    (StatusCode::CREATED, Json(body))
                }
            }),
        )
        .route(
            "/repos/org/renewed/commits/HEAD",
            get(move || async move {
                let body = json!({
                    "sha": commit_sha,
                    "node_id": "C_renewed_1",
                    "commit": {
                        "message": "renewed test",
                        "author": { "name": "Dev", "email": "dev@org.com", "date": "2026-03-01T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@org.com", "date": "2026-03-01T00:00:00Z" },
                        "tree": { "sha": tree_sha, "url": "https://api.github.com/tree" },
                        "comment_count": 0
                    },
                    "parents": []
                });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            &format!("/repos/org/renewed/git/trees/{tree_sha}"),
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
            "/repos/org/renewed/git/blobs/:sha",
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

    // Store a token record that will expire in 60 seconds (well within the default 300s proactive threshold)
    let stale_token = StoredTokenRecord::new(
        "ghs_stale_token_expiring_soon",
        Utc::now() + Duration::seconds(60),
        "org",
    )
    .with_repository_selection(RepositorySelection::Selected);

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

    let config = PrivateExtractionConfig::default()
        .with_fetch_metadata(false)
        .with_proactive_token_renewal_margin_secs(300);

    let extractor = PrivateRepositoryExtractor::new(Arc::clone(&client))
        .with_token_refresher(refresher, Arc::clone(&store_trait), session_id)
        .with_config(config);

    let ctx = AuthorizedInstallationContext::new(
        "org",
        "renewed",
        installation_id,
        "org",
        "ghs_stale_token_expiring_soon",
        RepositorySelection::Selected,
    );

    let result = extractor
        .extract(&ctx, None)
        .await
        .expect("must succeed with renewed token");

    assert_eq!(refresh_count.load(Ordering::SeqCst), 1);
    assert!(result.is_balanced());
    assert_eq!(result.total_extracted_files(), 1);

    // Verify token store now has the renewed token
    let updated_record = store
        .get_token(&session_id, installation_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_record.token, "ghs_freshly_renewed_token_xyz");
}

#[tokio::test]
async fn test_private_extraction_subtree_recovery_on_truncation() {
    let main_code = "fn main() {}\n";
    let utils_code = "pub fn helper() {}\n";
    let main_sha = compute_git_blob_sha(main_code.as_bytes());
    let utils_sha = compute_git_blob_sha(utils_code.as_bytes());

    let commit_sha = "trunc_commit_1111111111111111111111111111";
    let root_tree_sha = "root_tree_sha_22222222222222222222222222";
    let subtree_sha = "src_subtree_sha_333333333333333333333333";

    let m_sha = main_sha.clone();
    let u_sha = utils_sha.clone();

    let app = Router::new()
        .route(
            "/repos/org/large-repo/commits/HEAD",
            get(move || async move {
                let body = json!({
                    "sha": commit_sha,
                    "node_id": "C_large_1",
                    "commit": {
                        "message": "large tree test",
                        "author": { "name": "Dev", "email": "dev@org.com", "date": "2026-03-01T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@org.com", "date": "2026-03-01T00:00:00Z" },
                        "tree": { "sha": root_tree_sha, "url": "https://api.github.com/tree" },
                        "comment_count": 0
                    },
                    "parents": []
                });
                (StatusCode::OK, Json(body))
            }),
        )
        // Root tree is marked truncated: true and contains a subtree entry for 'src'
        .route(
            &format!("/repos/org/large-repo/git/trees/{root_tree_sha}"),
            get(move || async move {
                let body = json!({
                    "sha": root_tree_sha,
                    "url": "https://api.github.com/tree",
                    "truncated": true,
                    "tree": [
                        {
                            "path": "src",
                            "mode": "040000",
                            "type": "tree",
                            "sha": subtree_sha,
                            "size": null
                        }
                    ]
                });
                (StatusCode::OK, Json(body))
            }),
        )
        // Subtree recovery fetches the 'src' subtree
        .route(
            &format!("/repos/org/large-repo/git/trees/{subtree_sha}"),
            get(move || {
                let ms = m_sha.clone();
                let us = u_sha.clone();
                async move {
                    let body = json!({
                        "sha": subtree_sha,
                        "url": "https://api.github.com/subtree",
                        "truncated": false,
                        "tree": [
                            {
                                "path": "main.rs",
                                "mode": "100644",
                                "type": "blob",
                                "sha": ms,
                                "size": main_code.len()
                            },
                            {
                                "path": "utils.rs",
                                "mode": "100644",
                                "type": "blob",
                                "sha": us,
                                "size": utils_code.len()
                            }
                        ]
                    });
                    (StatusCode::OK, Json(body))
                }
            }),
        )
        .route(
            "/repos/org/large-repo/git/blobs/:sha",
            get(move |Path(sha): Path<String>| {
                let ms = main_sha.clone();
                let us = utils_sha.clone();
                async move {
                    let mut headers = AxumHeaderMap::new();
                    headers.insert(
                        axum::http::header::CONTENT_TYPE,
                        "application/vnd.github.raw".parse().unwrap(),
                    );
                    if sha == ms {
                        (StatusCode::OK, headers, main_code.as_bytes().to_vec()).into_response()
                    } else if sha == us {
                        (StatusCode::OK, headers, utils_code.as_bytes().to_vec()).into_response()
                    } else {
                        (StatusCode::NOT_FOUND, "Not found").into_response()
                    }
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let config = PrivateExtractionConfig::default().with_fetch_metadata(false);
    let extractor = PrivateRepositoryExtractor::new(client).with_config(config);

    let ctx = AuthorizedInstallationContext::new(
        "org",
        "large-repo",
        888,
        "org",
        "ghs_token_large",
        RepositorySelection::Selected,
    );

    let result = extractor
        .extract(&ctx, None)
        .await
        .expect("must succeed with subtree recovery");

    assert!(result.is_balanced());
    assert_eq!(result.total_extracted_files(), 2);
    assert_eq!(result.source_report.extracted.len(), 2);
}

#[tokio::test]
async fn test_private_extraction_unauthorized_token_401_sanitized() {
    let app = Router::new().route(
        "/repos/acme/private-service/commits/HEAD",
        get(|| async {
            let body = json!({
                "message": "Bad credentials",
                "documentation_url": "https://docs.github.com/rest"
            });
            (StatusCode::UNAUTHORIZED, Json(body))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = Arc::new(GitHubApiClient::new().with_base_url_override(format!("http://{addr}")));
    let extractor = PrivateRepositoryExtractor::new(client);

    let secret_token = "ghs_SUPER_CONFIDENTIAL_BEARER_SECRET_123456789";
    let ctx = AuthorizedInstallationContext::new(
        "acme",
        "private-service",
        999,
        "acme",
        secret_token,
        RepositorySelection::Selected,
    );

    let err = extractor
        .extract(&ctx, None)
        .await
        .expect_err("must fail with unauthorized error");

    let err_str = format!("{err:?}");
    // Crucial Rule 05 security guarantee: the secret token MUST NOT leak in the error message or debug output
    assert!(
        !err_str.contains(secret_token),
        "Secret token must never be leaked in error traces"
    );
}

#[test]
fn test_private_extraction_secret_masking() {
    let ctx = AuthorizedInstallationContext::new(
        "acme",
        "private-repo",
        123,
        "acme",
        "ghs_secret_should_never_leak_in_debug",
        RepositorySelection::Selected,
    );
    let debug_str = format!("{ctx:?}");
    assert!(!debug_str.contains("ghs_secret_should_never_leak_in_debug"));
    assert!(debug_str.contains("[REDACTED_TOKEN]"));

    let display_str = format!("{ctx}");
    assert!(!display_str.contains("ghs_secret_should_never_leak_in_debug"));
}
