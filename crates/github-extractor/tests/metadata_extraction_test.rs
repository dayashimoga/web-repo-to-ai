//! Integration tests for GitHub repository metadata extraction, languages breakdown,
//! and CDM metadata integration (`crates/github-extractor`).

use std::collections::BTreeMap;
use std::time::Duration;

use axum::extract::Path;
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use chrono::{TimeZone, Utc};
use serde_json::json;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    CommitAuthor, GitHubApiClient, GitHubCommitMetadata, GitHubExtractorError, GitHubLanguages,
    GitHubLicense, GitHubRepoContext, GitHubRepositoryMetadata, GitHubUrlParser, PinnedRevision,
};

#[test]
fn test_license_data_structures() {
    let license = GitHubLicense::new(
        "mit",
        "MIT License",
        Some("MIT".to_string()),
        Some("https://api.github.com/licenses/mit".to_string()),
    );
    assert_eq!(license.key, "mit");
    assert_eq!(license.name, "MIT License");
    assert_eq!(license.spdx_id.as_deref(), Some("MIT"));
    assert_eq!(
        license.url.as_deref(),
        Some("https://api.github.com/licenses/mit")
    );

    // Serialization & Deserialization
    let json_str = serde_json::to_string(&license).expect("must serialize");
    let deserialized: GitHubLicense = serde_json::from_str(&json_str).expect("must deserialize");
    assert_eq!(license, deserialized);
}

#[test]
fn test_languages_byte_breakdown_and_percentages() {
    // Empty language breakdown
    let empty_langs = GitHubLanguages::default();
    assert_eq!(empty_langs.total_bytes(), 0);
    assert_eq!(empty_langs.percentage("Rust"), None);
    assert!(empty_langs.percentages().is_empty());
    assert_eq!(empty_langs.primary_language(), None);

    // Populated language breakdown
    let mut map = BTreeMap::new();
    map.insert("Rust".to_string(), 75_000);
    map.insert("TypeScript".to_string(), 20_000);
    map.insert("HTML".to_string(), 5_000);

    let langs = GitHubLanguages::new(map);
    assert_eq!(langs.total_bytes(), 100_000);

    // Percentage calculation
    let rust_pct = langs.percentage("Rust").expect("Rust must exist");
    assert!((rust_pct - 75.0).abs() < f64::EPSILON);

    let ts_pct = langs
        .percentage("TypeScript")
        .expect("TypeScript must exist");
    assert!((ts_pct - 20.0).abs() < f64::EPSILON);

    let html_pct = langs.percentage("HTML").expect("HTML must exist");
    assert!((html_pct - 5.0).abs() < f64::EPSILON);

    assert_eq!(langs.percentage("Python"), None);

    // Primary language
    assert_eq!(langs.primary_language(), Some("Rust"));

    // Sorted percentages (descending by bytes)
    let sorted = langs.percentages();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].0, "Rust");
    assert_eq!(sorted[1].0, "TypeScript");
    assert_eq!(sorted[2].0, "HTML");
}

#[test]
fn test_repository_metadata_helpers_and_cdm_mapping() {
    let t_created = Utc.timestamp_opt(1672531199, 0).unwrap();
    let t_pushed = Utc.timestamp_opt(1700000000, 0).unwrap();

    let license = GitHubLicense::new(
        "apache-2.0",
        "Apache License 2.0",
        Some("Apache-2.0".to_string()),
        None,
    );

    let metadata = GitHubRepositoryMetadata {
        id: 98765432,
        node_id: Some("R_kgDOABC123".to_string()),
        name: "web-repo-to-ai".to_string(),
        full_name: "dayashimoga/web-repo-to-ai".to_string(),
        owner: "dayashimoga".to_string(),
        description: Some(
            "Universal URL & GitHub Repository -> AI-Ready Content Converter".to_string(),
        ),
        fork: false,
        archived: false,
        disabled: false,
        visibility: "public".to_string(),
        default_branch: "main".to_string(),
        homepage: Some("https://web-repo-to-ai.example.com".to_string()),
        primary_language: Some("Rust".to_string()),
        topics: vec![
            "rust".to_string(),
            "ai".to_string(),
            "llm".to_string(),
            "converter".to_string(),
        ],
        license: Some(license),
        stargazers_count: 350,
        watchers_count: 350,
        forks_count: 42,
        open_issues_count: 3,
        size_kb: 12450,
        created_at: Some(t_created),
        updated_at: Some(t_pushed),
        pushed_at: Some(t_pushed),
    };

    // Summary formatting
    let summary = metadata.summary();
    assert!(summary.contains("dayashimoga/web-repo-to-ai"));
    assert!(summary.contains("Rust"));
    assert!(summary.contains("Apache-2.0"));
    assert!(summary.contains("350 stars"));

    // Language breakdown
    let mut lang_map = BTreeMap::new();
    lang_map.insert("Rust".to_string(), 85_000);
    lang_map.insert("TypeScript".to_string(), 15_000);
    let langs = GitHubLanguages::new(lang_map);

    // CDM DocumentMetadata mapping
    let cdm_meta = metadata.to_cdm_metadata(&langs);
    assert_eq!(
        cdm_meta.title.as_deref(),
        Some("dayashimoga/web-repo-to-ai")
    );
    assert_eq!(
        cdm_meta.description.as_deref(),
        Some("Universal URL & GitHub Repository -> AI-Ready Content Converter")
    );
    assert_eq!(cdm_meta.authors, vec!["dayashimoga".to_string()]);
    assert_eq!(cdm_meta.license.as_deref(), Some("Apache-2.0"));
    assert_eq!(cdm_meta.primary_language.as_deref(), Some("Rust"));
    assert_eq!(cdm_meta.published_at, Some(t_created));
    assert_eq!(cdm_meta.modified_at, Some(t_pushed));
    assert_eq!(
        cdm_meta.tags,
        vec![
            "rust".to_string(),
            "ai".to_string(),
            "llm".to_string(),
            "converter".to_string()
        ]
    );
    assert_eq!(
        cdm_meta.language_breakdown.get("Rust").copied(),
        Some(85_000)
    );
    assert_eq!(
        cdm_meta.language_breakdown.get("TypeScript").copied(),
        Some(15_000)
    );

    // Extra attributes
    assert_eq!(
        cdm_meta.extra.get("github.repo_id").map(String::as_str),
        Some("98765432")
    );
    assert_eq!(
        cdm_meta
            .extra
            .get("github.default_branch")
            .map(String::as_str),
        Some("main")
    );
    assert_eq!(
        cdm_meta.extra.get("github.visibility").map(String::as_str),
        Some("public")
    );
    assert_eq!(
        cdm_meta.extra.get("github.is_fork").map(String::as_str),
        Some("false")
    );
    assert_eq!(
        cdm_meta.extra.get("github.is_archived").map(String::as_str),
        Some("false")
    );
    assert_eq!(
        cdm_meta
            .extra
            .get("github.stargazers_count")
            .map(String::as_str),
        Some("350")
    );
    assert_eq!(
        cdm_meta.extra.get("github.homepage").map(String::as_str),
        Some("https://web-repo-to-ai.example.com")
    );

    // Consolidated GitHubRepoContext
    let target = GitHubUrlParser::parse_url("https://github.com/dayashimoga/web-repo-to-ai")
        .expect("must parse");

    let commit_meta = GitHubCommitMetadata {
        sha: "11223344556677889900aabbccddeeff00112233".to_string(),
        node_id: None,
        message: "feat: initial commit".to_string(),
        author: Some(CommitAuthor::new("Dev", "dev@example.com", t_created)),
        committer: Some(CommitAuthor::new("Dev", "dev@example.com", t_created)),
        tree_sha: "tree11223344556677889900aabbccddeeff0011".to_string(),
        parents: Vec::new(),
        comment_count: 0,
        verification: None,
    };

    let pinned = PinnedRevision::new(
        target.clone(),
        "11223344556677889900aabbccddeeff00112233",
        "tree11223344556677889900aabbccddeeff0011",
        commit_meta,
        Some("main".to_string()),
    );

    let repo_context = GitHubRepoContext::new(target, metadata, langs, Some(pinned));

    let context_cdm_meta = repo_context.to_cdm_metadata();
    assert_eq!(
        context_cdm_meta
            .extra
            .get("git.commit_sha")
            .map(String::as_str),
        Some("11223344556677889900aabbccddeeff00112233")
    );
    assert_eq!(
        context_cdm_meta
            .extra
            .get("git.root_tree_sha")
            .map(String::as_str),
        Some("tree11223344556677889900aabbccddeeff0011")
    );

    let card = repo_context.summary_card();
    assert!(card.contains("# dayashimoga/web-repo-to-ai"));
    assert!(card.contains("350 stars"));
    assert!(card.contains("**Pinned Revision:** `1122334`"));
}

// -----------------------------------------------------------------------------
// Live In-Process HTTP Server Integration Tests
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_live_http_repository_metadata_and_languages() {
    let app = Router::new()
        .route(
            "/repos/:owner/:repo",
            get(|Path((owner, repo)): Path<(String, String)>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4988".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                match (owner.as_str(), repo.as_str()) {
                    ("test-owner", "test-repo") => {
                        let body = json!({
                            "id": 12345678,
                            "node_id": "R_kgDOHello",
                            "name": "test-repo",
                            "full_name": "test-owner/test-repo",
                            "owner": {
                                "login": "test-owner"
                            },
                            "description": "Production test repository for metadata validation",
                            "fork": false,
                            "archived": false,
                            "disabled": false,
                            "visibility": "public",
                            "default_branch": "main",
                            "homepage": "https://test.example.com",
                            "language": "Rust",
                            "topics": ["rust", "microservice", "testing"],
                            "license": {
                                "key": "mit",
                                "name": "MIT License",
                                "spdx_id": "MIT",
                                "url": "https://api.github.com/licenses/mit"
                            },
                            "stargazers_count": 120,
                            "watchers_count": 120,
                            "forks_count": 15,
                            "open_issues_count": 2,
                            "size": 4096,
                            "created_at": "2025-01-01T00:00:00Z",
                            "updated_at": "2026-03-01T10:00:00Z",
                            "pushed_at": "2026-03-15T12:00:00Z"
                        });
                        (StatusCode::OK, headers, Json(body)).into_response()
                    }
                    ("test-owner", "nonexistent-repo") => {
                        let body = json!({
                            "message": "Not Found",
                            "documentation_url": "https://docs.github.com/rest/repos"
                        });
                        (StatusCode::NOT_FOUND, headers, Json(body)).into_response()
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, headers, "Internal Error")
                        .into_response(),
                }
            }),
        )
        .route(
            "/repos/:owner/:repo/languages",
            get(|Path((owner, repo)): Path<(String, String)>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4987".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                match (owner.as_str(), repo.as_str()) {
                    ("test-owner", "test-repo") => {
                        let body = json!({
                            "Rust": 80000,
                            "TypeScript": 18000,
                            "Shell": 2000
                        });
                        (StatusCode::OK, headers, Json(body)).into_response()
                    }
                    ("test-owner", "nonexistent-repo") => {
                        let body = json!({
                            "message": "Not Found",
                            "documentation_url": "https://docs.github.com/rest/repos"
                        });
                        (StatusCode::NOT_FOUND, headers, Json(body)).into_response()
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, headers, "Internal Error")
                        .into_response(),
                }
            }),
        )
        .route(
            "/repos/:owner/:repo/commits/:commit_ref",
            get(
                |Path((_owner, _repo, _commit_ref)): Path<(String, String, String)>| async move {
                    let mut headers = AxumHeaderMap::new();
                    headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                    headers.insert("x-ratelimit-remaining", "4986".parse().unwrap());

                    let body = json!({
                        "sha": "abcdef1234567890abcdef1234567890abcdef12",
                        "commit": {
                            "message": "chore: test commit",
                            "tree": {
                                "sha": "tree999999999999999999999999999999999999"
                            },
                            "comment_count": 0
                        },
                        "parents": []
                    });
                    (StatusCode::OK, headers, Json(body)).into_response()
                },
            ),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("must bind test server");
    let local_addr = listener.local_addr().expect("must get local address");
    let server_base_url = format!("http://{}", local_addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("must serve");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(&server_base_url)
        .with_auth_token("ghp_SecretToken1234567890abcdef12345678")
        .with_timeout(Duration::from_secs(5));

    let target = GitHubUrlParser::parse_url("https://github.com/test-owner/test-repo")
        .expect("must parse target");

    // 1. Get repository metadata
    let (metadata, rate_limit) = client
        .get_repo_metadata(&target)
        .await
        .expect("must get repo metadata");

    assert_eq!(metadata.id, 12345678);
    assert_eq!(metadata.name, "test-repo");
    assert_eq!(metadata.full_name, "test-owner/test-repo");
    assert_eq!(metadata.owner, "test-owner");
    assert_eq!(
        metadata.description.as_deref(),
        Some("Production test repository for metadata validation")
    );
    assert!(!metadata.fork);
    assert!(!metadata.archived);
    assert_eq!(metadata.default_branch, "main");
    assert_eq!(metadata.primary_language.as_deref(), Some("Rust"));
    assert_eq!(metadata.topics, vec!["rust", "microservice", "testing"]);

    let license = metadata.license.expect("license must be present");
    assert_eq!(license.key, "mit");
    assert_eq!(license.spdx_id.as_deref(), Some("MIT"));

    assert_eq!(metadata.stargazers_count, 120);
    assert_eq!(metadata.forks_count, 15);
    assert_eq!(metadata.open_issues_count, 2);
    assert_eq!(metadata.size_kb, 4096);

    let rl = rate_limit.expect("rate limit must be parsed");
    assert_eq!(rl.limit, 5000);
    assert_eq!(rl.remaining, 4988);

    // 2. Get repository languages breakdown
    let (langs, lang_rl) = client
        .get_repo_languages(&target)
        .await
        .expect("must get languages");

    assert_eq!(langs.total_bytes(), 100000);
    assert_eq!(langs.primary_language(), Some("Rust"));
    assert!((langs.percentage("Rust").unwrap() - 80.0).abs() < f64::EPSILON);
    assert!((langs.percentage("TypeScript").unwrap() - 18.0).abs() < f64::EPSILON);
    assert!((langs.percentage("Shell").unwrap() - 2.0).abs() < f64::EPSILON);
    assert!(lang_rl.is_some());

    // 3. Consolidated fetch_repo_context with revision pinning
    let repo_context = client
        .fetch_repo_context(&target, true)
        .await
        .expect("must fetch unified repo context");

    assert_eq!(repo_context.metadata().full_name, "test-owner/test-repo");
    assert_eq!(repo_context.languages().total_bytes(), 100000);

    let pinned = repo_context
        .pinned_revision()
        .expect("pinned revision must exist");
    assert_eq!(
        pinned.commit_sha(),
        "abcdef1234567890abcdef1234567890abcdef12"
    );
    assert_eq!(
        pinned.root_tree_sha(),
        "tree999999999999999999999999999999999999"
    );

    // 4. CDM DocumentMetadata conversion from context
    let cdm_meta = repo_context.to_cdm_metadata();
    assert_eq!(cdm_meta.title.as_deref(), Some("test-owner/test-repo"));
    assert_eq!(cdm_meta.authors, vec!["test-owner".to_string()]);
    assert_eq!(cdm_meta.license.as_deref(), Some("MIT"));
    assert_eq!(
        cdm_meta.extra.get("git.commit_sha").map(String::as_str),
        Some("abcdef1234567890abcdef1234567890abcdef12")
    );

    // 5. Error handling: 404 Not Found on nonexistent repo
    let non_target = GitHubUrlParser::parse_url("https://github.com/test-owner/nonexistent-repo")
        .expect("must parse target");
    let err = client
        .get_repo_metadata(&non_target)
        .await
        .expect_err("nonexistent repo must 404");

    match err {
        GitHubExtractorError::ResourceNotFound { message, .. } => {
            assert!(message.contains("Not Found"));
            assert!(!message.contains("ghp_SecretToken"));
        }
        other => panic!("expected ResourceNotFound, got: {other:?}"),
    }

    // 6. Token redaction verification on server error
    let err_target = GitHubUrlParser::parse_url("https://github.com/test-owner/error-repo")
        .expect("must parse target");
    let server_err = client
        .get_repo_metadata(&err_target)
        .await
        .expect_err("500 error must fail");

    let err_str = server_err.to_string();
    assert!(!err_str.contains("ghp_SecretToken"));
}
