//! Integration tests for Git Trees truncation detection and completeness auditing (`crates/github-extractor`).

use std::time::Duration;

use axum::extract::{Path, Query};
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    GitFileMode, GitHubApiClient, GitHubExtractorError, GitHubUrlParser, GitTreeEntry,
    GitTreeEntryType, GitTreeResponse, RecoveryStrategy, TreeCompleteness, TruncationReason,
    GITHUB_TREE_ENTRY_LIMIT,
};

#[test]
fn test_complete_tree_analysis() {
    let entries = vec![
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha1",
            Some(1000),
        ),
        GitTreeEntry::new(
            "crates",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha2",
            None,
        ),
        GitTreeEntry::new(
            "crates/core",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha3",
            None,
        ),
        GitTreeEntry::new(
            "crates/core/src",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha4",
            None,
        ),
        GitTreeEntry::new(
            "crates/core/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha5",
            Some(2500),
        ),
    ];

    let tree = GitTreeResponse::new("tree_sha_root", entries, false);

    assert!(!tree.is_truncated());
    assert!(tree.find_unexpanded_subtrees().is_empty());

    let report = tree.analyze_completeness();
    assert!(!report.is_truncated);
    assert!(report.reasons.is_empty());
    assert_eq!(report.total_entries, 5);
    assert_eq!(report.total_blobs, 2);
    assert_eq!(report.total_trees, 3);
    assert!(report.unexpanded_subtrees.is_empty());
    assert_eq!(
        report.deepest_path.as_deref(),
        Some("crates/core/src/lib.rs")
    );
    assert_eq!(report.max_depth, 3);
    assert_eq!(report.recommended_recovery_strategy, RecoveryStrategy::None);

    assert_eq!(tree.completeness(), TreeCompleteness::Complete);
    assert!(tree.completeness().is_complete());
    assert!(!tree.completeness().is_truncated());
}

#[test]
fn test_explicit_github_truncation_flag() {
    let entries = vec![
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha1",
            Some(1000),
        ),
        GitTreeEntry::new(
            "crates",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha2",
            None,
        ),
    ];

    // Tree with explicit truncated: true
    let tree = GitTreeResponse::new("tree_sha_root", entries, true);

    assert!(tree.is_truncated());

    let report = tree.analyze_completeness();
    assert!(report.is_truncated);
    assert!(report.reasons.contains(&TruncationReason::GitHubFlag));
    assert_eq!(
        report.recommended_recovery_strategy,
        RecoveryStrategy::SubtreeTraversal
    );

    match tree.completeness() {
        TreeCompleteness::Truncated(rep) => {
            assert!(rep.is_truncated);
            assert_eq!(rep.total_entries, 2);
        }
        TreeCompleteness::Complete => panic!("expected TreeCompleteness::Truncated"),
    }
}

#[test]
fn test_unexpanded_subtree_detection() {
    let entries = vec![
        GitTreeEntry::new(
            "crates",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha1",
            None,
        ),
        GitTreeEntry::new(
            "crates/core",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha2",
            None,
        ),
        GitTreeEntry::new(
            "crates/core/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha3",
            Some(2000),
        ),
        // Truncated/unexpanded directories (no children present in response)
        GitTreeEntry::new(
            "crates/massive_module",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha_massive",
            None,
        ),
        GitTreeEntry::new(
            "docs/unexpanded_guide",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha_docs",
            None,
        ),
    ];

    let tree = GitTreeResponse::new("tree_sha_root", entries, true);

    let unexpanded = tree.find_unexpanded_subtrees();
    let unexpanded_paths: Vec<&str> = unexpanded.iter().map(|e| e.path.as_str()).collect();

    assert_eq!(unexpanded_paths.len(), 2);
    assert!(unexpanded_paths.contains(&"crates/massive_module"));
    assert!(unexpanded_paths.contains(&"docs/unexpanded_guide"));

    let report = tree.analyze_completeness();
    assert!(report.is_truncated);
    assert!(report
        .reasons
        .contains(&TruncationReason::IncompleteSubtrees));
    assert!(report.reasons.contains(&TruncationReason::GitHubFlag));
    assert_eq!(report.unexpanded_subtrees.len(), 2);
    assert_eq!(
        report.recommended_recovery_strategy,
        RecoveryStrategy::SubtreeTraversal
    );
}

#[test]
fn test_entry_count_ceiling_recommends_shallow_archive() {
    // Generate simulated tree reaching GITHUB_TREE_ENTRY_LIMIT
    let mut entries = Vec::with_capacity(15);
    for i in 0..15 {
        entries.push(GitTreeEntry::new(
            format!("file_{i}.txt"),
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            format!("sha_{i}"),
            Some(100),
        ));
    }

    // Add unexpanded subtrees exceeding SUBTREE_TRAVERSAL_LIMIT (501 unexpanded trees)
    for i in 0..505 {
        entries.push(GitTreeEntry::new(
            format!("dir_{i}"),
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            format!("tree_sha_{i}"),
            None,
        ));
    }

    let tree = GitTreeResponse::new("tree_root", entries, true);
    let report = tree.analyze_completeness();

    assert!(report.is_truncated);
    assert_eq!(report.unexpanded_subtrees.len(), 505);
    // Because unexpanded subtrees > 500, recommends ShallowArchiveFallback
    assert_eq!(
        report.recommended_recovery_strategy,
        RecoveryStrategy::ShallowArchiveFallback
    );
    assert_eq!(
        report.recommended_recovery_strategy.as_str(),
        "shallow_archive_fallback"
    );

    // Verify TruncationReason descriptions
    assert_eq!(
        TruncationReason::GitHubFlag.description(),
        "GitHub Git Trees API returned truncated: true"
    );
    assert_eq!(
        TruncationReason::EntryCountCap.description(),
        "Discovered entries reached or exceeded the 100,000 limit"
    );
    assert!(TruncationReason::IncompleteSubtrees
        .description()
        .contains("directory subtrees"));
    assert_eq!(GITHUB_TREE_ENTRY_LIMIT, 100_000);
}

// -----------------------------------------------------------------------------
// Live In-Process HTTP Server Integration Tests
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TreeQueryParams {
    #[serde(default)]
    _recursive: Option<u8>,
}

#[tokio::test]
async fn test_live_http_truncated_git_tree_detection() {
    let app = Router::new().route(
        "/repos/:owner/:repo/git/trees/:tree_sha",
        get(
            |Path((_owner, _repo, tree_sha)): Path<(String, String, String)>,
             Query(_query): Query<TreeQueryParams>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4970".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                match tree_sha.as_str() {
                    "truncated_tree_sha" => {
                        let body = json!({
                            "sha": "truncated_tree_sha",
                            "tree": [
                                {
                                    "path": "README.md",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": "blob111111111111111111111111111111111111",
                                    "size": 1200
                                },
                                {
                                    "path": "crates",
                                    "mode": "040000",
                                    "type": "tree",
                                    "sha": "tree222222222222222222222222222222222222"
                                },
                                {
                                    "path": "crates/core/src/lib.rs",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": "blob333333333333333333333333333333333333",
                                    "size": 3400
                                },
                                {
                                    "path": "vendor/massive_crate",
                                    "mode": "040000",
                                    "type": "tree",
                                    "sha": "tree444444444444444444444444444444444444"
                                }
                            ],
                            "truncated": true
                        });
                        (StatusCode::OK, headers, Json(body)).into_response()
                    }
                    "complete_tree_sha" => {
                        let body = json!({
                            "sha": "complete_tree_sha",
                            "tree": [
                                {
                                    "path": "README.md",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": "blob111111111111111111111111111111111111",
                                    "size": 1200
                                },
                                {
                                    "path": "crates",
                                    "mode": "040000",
                                    "type": "tree",
                                    "sha": "tree222222222222222222222222222222222222"
                                },
                                {
                                    "path": "crates/core/src/lib.rs",
                                    "mode": "100644",
                                    "type": "blob",
                                    "sha": "blob333333333333333333333333333333333333",
                                    "size": 3400
                                }
                            ],
                            "truncated": false
                        });
                        (StatusCode::OK, headers, Json(body)).into_response()
                    }
                    "nonexistent" => {
                        let body = json!({
                            "message": "Tree Not Found",
                            "documentation_url": "https://docs.github.com/rest/git/trees"
                        });
                        (StatusCode::NOT_FOUND, headers, Json(body)).into_response()
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, headers, "Internal Error")
                        .into_response(),
                }
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
        .with_auth_token("ghp_TruncationSecretToken1234567890abcdef")
        .with_timeout(Duration::from_secs(5));

    let target = GitHubUrlParser::parse_url("https://github.com/test-owner/test-repo")
        .expect("must parse target");

    // 1. Fetch truncated tree and audit
    let (truncated_tree, rate_limit) = client
        .get_tree(&target, "truncated_tree_sha", true)
        .await
        .expect("must fetch tree");

    assert!(truncated_tree.is_truncated());
    let report = truncated_tree.analyze_completeness();
    assert!(report.is_truncated);
    assert!(report.reasons.contains(&TruncationReason::GitHubFlag));
    assert!(report
        .reasons
        .contains(&TruncationReason::IncompleteSubtrees));
    assert_eq!(
        report.unexpanded_subtrees,
        vec!["vendor/massive_crate".to_string()]
    );
    assert_eq!(
        report.recommended_recovery_strategy,
        RecoveryStrategy::SubtreeTraversal
    );

    let rl = rate_limit.expect("rate limit must be parsed");
    assert_eq!(rl.limit, 5000);
    assert_eq!(rl.remaining, 4970);

    // 2. Fetch complete tree and audit
    let (complete_tree, _) = client
        .get_tree(&target, "complete_tree_sha", true)
        .await
        .expect("must fetch complete tree");

    assert!(!complete_tree.is_truncated());
    let complete_report = complete_tree.analyze_completeness();
    assert!(!complete_report.is_truncated);
    assert!(complete_report.reasons.is_empty());
    assert_eq!(complete_tree.completeness(), TreeCompleteness::Complete);
    assert_eq!(
        complete_report.recommended_recovery_strategy,
        RecoveryStrategy::None
    );

    // 3. Error handling: 404 Tree Not Found
    let not_found_err = client
        .get_tree(&target, "nonexistent", true)
        .await
        .expect_err("nonexistent tree must fail");

    match not_found_err {
        GitHubExtractorError::ResourceNotFound { message, .. } => {
            assert!(message.contains("Tree Not Found"));
            assert!(!message.contains("ghp_TruncationSecretToken"));
        }
        other => panic!("expected ResourceNotFound, got: {other:?}"),
    }

    // 4. Token redaction verification on server error
    let server_err = client
        .get_tree(&target, "server-error-trigger", true)
        .await
        .expect_err("500 error must fail");

    let err_str = server_err.to_string();
    assert!(!err_str.contains("ghp_TruncationSecretToken"));
}
