//! Integration tests for GitHub branch, tag, and reference discovery (`crates/github-extractor`).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Query;
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use chrono::{TimeZone, Utc};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    redact_secrets, GitHubApiClient, GitHubBranch, GitHubExtractorError, GitHubRateLimit,
    GitHubTag, GitHubUrlParser, RefDiscoveryReport,
};

#[test]
fn test_rate_limit_parsing_from_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
    headers.insert("x-ratelimit-remaining", "4980".parse().unwrap());
    headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());
    headers.insert("x-ratelimit-used", "20".parse().unwrap());

    let rl = GitHubRateLimit::from_headers(&headers).expect("must parse valid headers");
    assert_eq!(rl.limit, 5000);
    assert_eq!(rl.remaining, 4980);
    assert_eq!(rl.used, Some(20));
    assert_eq!(rl.reset_at, Utc.timestamp_opt(1700000000, 0).unwrap());
    assert!(!rl.is_exhausted());

    // Exhausted rate limit
    let mut exhausted_headers = HeaderMap::new();
    exhausted_headers.insert("x-ratelimit-limit", "60".parse().unwrap());
    exhausted_headers.insert("x-ratelimit-remaining", "0".parse().unwrap());
    exhausted_headers.insert("x-ratelimit-reset", "1700001000".parse().unwrap());

    let rl_exhausted =
        GitHubRateLimit::from_headers(&exhausted_headers).expect("must parse exhausted headers");
    assert_eq!(rl_exhausted.limit, 60);
    assert_eq!(rl_exhausted.remaining, 0);
    assert!(rl_exhausted.is_exhausted());
}

#[test]
fn test_token_redaction_utility() {
    let plain = "Safe error message with no secrets";
    assert_eq!(redact_secrets(plain), plain);

    let pat = "Authentication failed with token ghp_1234567890abcdef1234567890abcdef123456";
    assert_eq!(
        redact_secrets(pat),
        "Authentication failed with token [REDACTED_GITHUB_TOKEN]"
    );

    let app_token = "Error contacting API with ghs_9876543210zyxwvutsrqponmlkjihgfedcba";
    assert_eq!(
        redact_secrets(app_token),
        "Error contacting API with [REDACTED_GITHUB_TOKEN]"
    );

    let bearer = "Header: Bearer ghp_ABC123DEF456 failed";
    assert_eq!(
        redact_secrets(bearer),
        "Header: Bearer [REDACTED_GITHUB_TOKEN] failed"
    );

    let fine_grained = "Failed github_pat_11AAAAAAA00000000000000000_1234567890";
    assert_eq!(
        redact_secrets(fine_grained),
        "Failed [REDACTED_GITHUB_TOKEN]"
    );
}

#[test]
fn test_ref_discovery_report_lookups() {
    let branches = vec![
        GitHubBranch::new("main", "1111111111111111111111111111111111111111", true),
        GitHubBranch::new(
            "feature/pipeline-v2",
            "2222222222222222222222222222222222222222",
            false,
        ),
    ];

    let tags = vec![GitHubTag::new(
        "v1.0.0",
        "3333333333333333333333333333333333333333",
        Some("https://api.github.com/zipball".to_string()),
        Some("https://api.github.com/tarball".to_string()),
    )];

    let report = RefDiscoveryReport {
        owner: "dayashimoga".to_string(),
        repo: "web-repo-to-ai".to_string(),
        default_branch: Some("main".to_string()),
        branches,
        tags,
        rate_limit: None,
        discovered_at: Utc::now(),
    };

    assert!(report.find_branch("main").is_some());
    assert!(report.find_branch("nonexistent").is_none());

    assert!(report.find_tag("v1.0.0").is_some());
    assert!(report.find_tag("v2.0.0").is_none());

    // Resolve commit SHA
    assert_eq!(
        report.find_commit_sha_for_ref("main"),
        Some("1111111111111111111111111111111111111111")
    );
    assert_eq!(
        report.find_commit_sha_for_ref("feature/pipeline-v2"),
        Some("2222222222222222222222222222222222222222")
    );
    assert_eq!(
        report.find_commit_sha_for_ref("v1.0.0"),
        Some("3333333333333333333333333333333333333333")
    );

    // Direct 40-character hex commit SHA
    let direct_sha = "4444444444444444444444444444444444444444";
    assert_eq!(report.find_commit_sha_for_ref(direct_sha), Some(direct_sha));

    assert_eq!(report.find_commit_sha_for_ref("unknown"), None);
}

#[test]
fn test_disambiguate_ref_and_subpath_longest_prefix() {
    let branches = vec![
        GitHubBranch::new("main", "1111", true),
        GitHubBranch::new("feature/pipeline-v2", "2222", false),
        GitHubBranch::new("feature/pipeline-v2/extended", "3333", false),
        GitHubBranch::new("release/2026/q4", "4444", true),
    ];

    let tags = vec![GitHubTag::new("v1.0.0", "5555", None, None)];

    let report = RefDiscoveryReport {
        owner: "dayashimoga".to_string(),
        repo: "web-repo-to-ai".to_string(),
        default_branch: Some("main".to_string()),
        branches,
        tags,
        rate_limit: None,
        discovered_at: Utc::now(),
    };

    // Standard single-segment branch
    assert_eq!(
        report.disambiguate_ref_and_subpath("main/crates/core"),
        Some(("main".to_string(), Some("crates/core".to_string())))
    );

    // Slash branch with subpath
    assert_eq!(
        report.disambiguate_ref_and_subpath("feature/pipeline-v2/crates/github-extractor/src"),
        Some((
            "feature/pipeline-v2".to_string(),
            Some("crates/github-extractor/src".to_string())
        ))
    );

    // Longer slash branch preferred via longest-prefix matching
    assert_eq!(
        report.disambiguate_ref_and_subpath("feature/pipeline-v2/extended/docs/guide.md"),
        Some((
            "feature/pipeline-v2/extended".to_string(),
            Some("docs/guide.md".to_string())
        ))
    );

    // Deep release branch
    assert_eq!(
        report.disambiguate_ref_and_subpath("release/2026/q4/CHANGELOG.md"),
        Some((
            "release/2026/q4".to_string(),
            Some("CHANGELOG.md".to_string())
        ))
    );

    // Tag match
    assert_eq!(
        report.disambiguate_ref_and_subpath("v1.0.0/README.md"),
        Some(("v1.0.0".to_string(), Some("README.md".to_string())))
    );

    // Branch match without subpath
    assert_eq!(
        report.disambiguate_ref_and_subpath("feature/pipeline-v2"),
        Some(("feature/pipeline-v2".to_string(), None))
    );

    // Unknown branch fallback to first segment
    assert_eq!(
        report.disambiguate_ref_and_subpath("unknown-branch/sub/path"),
        Some(("unknown-branch".to_string(), Some("sub/path".to_string())))
    );
}

#[test]
fn test_disambiguate_target_integration() {
    let target = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/feature/new-api/crates/core",
    )
    .expect("must parse");

    // Initially parsed with single-segment ref "feature" and subpath "new-api/crates/core"
    assert_eq!(target.ref_spec(), Some("feature"));
    assert_eq!(target.subpath(), Some("new-api/crates/core"));

    let report = RefDiscoveryReport {
        owner: "dayashimoga".to_string(),
        repo: "web-repo-to-ai".to_string(),
        default_branch: Some("main".to_string()),
        branches: vec![GitHubBranch::new("feature/new-api", "abcdef", false)],
        tags: Vec::new(),
        rate_limit: None,
        discovered_at: Utc::now(),
    };

    let client = GitHubApiClient::new();
    let refined = client.disambiguate_target(&target, &report);

    assert_eq!(refined.ref_spec(), Some("feature/new-api"));
    assert_eq!(refined.subpath(), Some("crates/core"));
    assert_eq!(
        refined.canonical_target_url(),
        "https://github.com/dayashimoga/web-repo-to-ai/tree/feature/new-api/crates/core"
    );
}

// -----------------------------------------------------------------------------
// Live In-Process HTTP Server Integration Tests
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<usize>,
}

#[tokio::test]
async fn test_live_http_branch_and_tag_discovery_with_pagination() {
    let branch_request_count = Arc::new(AtomicUsize::new(0));
    let tag_request_count = Arc::new(AtomicUsize::new(0));

    let b_count = Arc::clone(&branch_request_count);
    let t_count = Arc::clone(&tag_request_count);

    let app = Router::new()
        .route(
            "/repos/test-owner/test-repo/branches",
            get(move |Query(query): Query<PaginationQuery>| {
                let count = Arc::clone(&b_count);
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    let page = query.page.unwrap_or(1);
                    let mut headers = AxumHeaderMap::new();
                    headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                    headers.insert("x-ratelimit-remaining", "4950".parse().unwrap());
                    headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                    if page == 1 {
                        let data = json!([
                            {
                                "name": "main",
                                "commit": { "sha": "1111111111111111111111111111111111111111" },
                                "protected": true
                            },
                            {
                                "name": "feature/api-v2",
                                "commit": { "sha": "2222222222222222222222222222222222222222" },
                                "protected": false
                            }
                        ]);
                        (headers, Json(data)).into_response()
                    } else {
                        let data = json!([
                            {
                                "name": "legacy",
                                "commit": { "sha": "3333333333333333333333333333333333333333" },
                                "protected": false
                            }
                        ]);
                        (headers, Json(data)).into_response()
                    }
                }
            }),
        )
        .route(
            "/repos/test-owner/test-repo/branches/feature/api-v2",
            get(|| async {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4949".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                let data = json!({
                    "name": "feature/api-v2",
                    "commit": { "sha": "2222222222222222222222222222222222222222" },
                    "protected": false
                });
                (headers, Json(data))
            }),
        )
        .route(
            "/repos/test-owner/test-repo/tags",
            get(move || {
                let count = Arc::clone(&t_count);
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    let mut headers = AxumHeaderMap::new();
                    headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                    headers.insert("x-ratelimit-remaining", "4948".parse().unwrap());
                    headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                    let data = json!([
                        {
                            "name": "v1.0.0",
                            "commit": { "sha": "4444444444444444444444444444444444444444" },
                            "zipball_url": "https://api.github.com/zipball/v1.0.0",
                            "tarball_url": "https://api.github.com/tarball/v1.0.0"
                        }
                    ]);
                    (headers, Json(data))
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("must bind ephemeral test port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base = format!("http://{addr}");
    let client = GitHubApiClient::new()
        .with_base_url_override(&mock_base)
        .with_per_page(2) // trigger pagination with page size 2
        .with_auth_token("ghp_test_token_1234567890abcdef")
        .with_timeout(Duration::from_secs(5));

    let target = GitHubUrlParser::parse_url("test-owner/test-repo").expect("must parse target");

    // 1. Test branch listing with pagination
    let (branches, rate_limit) = client
        .list_branches(&target)
        .await
        .expect("branches must succeed");
    assert_eq!(branches.len(), 3);
    assert_eq!(branches[0].name, "main");
    assert_eq!(branches[1].name, "feature/api-v2");
    assert_eq!(branches[2].name, "legacy");
    assert!(rate_limit.is_some());
    assert_eq!(rate_limit.unwrap().remaining, 4950);
    assert_eq!(branch_request_count.load(Ordering::SeqCst), 2); // 2 pages fetched

    // 2. Test single branch retrieval
    let (single_branch, _) = client
        .get_branch(&target, "feature/api-v2")
        .await
        .expect("get_branch must succeed");
    assert_eq!(single_branch.name, "feature/api-v2");
    assert_eq!(
        single_branch.commit_sha,
        "2222222222222222222222222222222222222222"
    );

    // 3. Test tag listing
    let (tags, _) = client.list_tags(&target).await.expect("tags must succeed");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "v1.0.0");
    assert_eq!(
        tags[0].commit_sha,
        "4444444444444444444444444444444444444444"
    );

    // 4. Test consolidated discover_refs report
    let report = client
        .discover_refs(&target)
        .await
        .expect("discover_refs must succeed");
    assert_eq!(report.owner, "test-owner");
    assert_eq!(report.repo, "test-repo");
    assert_eq!(report.default_branch, Some("main".to_string()));
    assert_eq!(report.branches.len(), 3);
    assert_eq!(report.tags.len(), 1);
}

#[tokio::test]
async fn test_live_http_error_handling_and_token_redaction() {
    let auth_header_received = Arc::new(std::sync::Mutex::new(None));
    let auth_clone = Arc::clone(&auth_header_received);

    let app = Router::new()
        .route(
            "/repos/test-owner/rate-limited/branches",
            get(|| async {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "60".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "0".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                (
                    StatusCode::FORBIDDEN,
                    headers,
                    Json(json!({ "message": "API rate limit exceeded for IP" })),
                )
            }),
        )
        .route(
            "/repos/test-owner/unauthorized/branches",
            get(move |headers: AxumHeaderMap| {
                let auth = Arc::clone(&auth_clone);
                async move {
                    if let Some(auth_val) = headers.get("authorization") {
                        *auth.lock().unwrap() = Some(auth_val.to_str().unwrap().to_string());
                    }
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "message": "Bad credentials token ghp_secret12345" })),
                    )
                }
            }),
        )
        .route(
            "/repos/test-owner/not-found/branches",
            get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "message": "Not Found" })),
                )
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("must bind ephemeral test port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mock_base = format!("http://{addr}");
    let client = GitHubApiClient::new()
        .with_base_url_override(&mock_base)
        .with_auth_token("ghp_my_secret_token_value_9999");

    // 1. Test Rate Limit Exceeded
    let target_rl = GitHubUrlParser::parse_url("test-owner/rate-limited").unwrap();
    let err_rl = client.list_branches(&target_rl).await.unwrap_err();
    match err_rl {
        GitHubExtractorError::RateLimitExceeded {
            limit, reset_at, ..
        } => {
            assert_eq!(limit, 60);
            assert_eq!(reset_at, Utc.timestamp_opt(1700000000, 0).unwrap());
        }
        other => panic!("Expected RateLimitExceeded, got {other:?}"),
    }

    // 2. Test Bad Credentials with Token Redaction
    let target_unauth = GitHubUrlParser::parse_url("test-owner/unauthorized").unwrap();
    let err_unauth = client.list_branches(&target_unauth).await.unwrap_err();
    match err_unauth {
        GitHubExtractorError::BadCredentials { message } => {
            assert!(!message.contains("ghp_secret12345"));
            assert!(message.contains("[REDACTED_GITHUB_TOKEN]"));
        }
        other => panic!("Expected BadCredentials, got {other:?}"),
    }
    // Verify server did receive the Authorization header
    assert_eq!(
        *auth_header_received.lock().unwrap(),
        Some("Bearer ghp_my_secret_token_value_9999".to_string())
    );

    // 3. Test Resource Not Found (404)
    let target_nf = GitHubUrlParser::parse_url("test-owner/not-found").unwrap();
    let err_nf = client.list_branches(&target_nf).await.unwrap_err();
    match err_nf {
        GitHubExtractorError::ResourceNotFound { url, message } => {
            assert!(url.contains("repos/test-owner/not-found/branches"));
            assert_eq!(message, "Not Found");
        }
        other => panic!("Expected ResourceNotFound, got {other:?}"),
    }
}
