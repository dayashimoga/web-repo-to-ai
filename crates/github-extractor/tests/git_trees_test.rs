//! Integration tests for GitHub Git Trees API integration, file modes, and tree queries (`crates/github-extractor`).

use std::time::Duration;

use axum::extract::{Path, Query};
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    CommitAuthor, GitFileMode, GitHubApiClient, GitHubCommitMetadata, GitHubExtractorError,
    GitHubUrlParser, GitTreeEntry, GitTreeEntryType, GitTreeResponse, PinnedRevision,
};

#[test]
fn test_git_file_mode_classification() {
    let regular = GitFileMode::from_mode_str("100644");
    assert_eq!(regular, GitFileMode::RegularFile);
    assert_eq!(regular.as_str(), "100644");
    assert!(regular.is_file());
    assert!(!regular.is_directory());
    assert!(!regular.is_executable());
    assert!(!regular.is_symlink());
    assert!(!regular.is_submodule());
    assert_eq!(format!("{regular}"), "100644");

    let executable = GitFileMode::from_mode_str("100755");
    assert_eq!(executable, GitFileMode::ExecutableFile);
    assert!(executable.is_file());
    assert!(executable.is_executable());

    let dir = GitFileMode::from_mode_str("040000");
    assert_eq!(dir, GitFileMode::Subdirectory);
    assert!(dir.is_directory());
    assert!(!dir.is_file());

    let symlink = GitFileMode::from_mode_str("120000");
    assert_eq!(symlink, GitFileMode::Symlink);
    assert!(symlink.is_symlink());
    assert!(symlink.is_file());

    let submodule = GitFileMode::from_mode_str("160000");
    assert_eq!(submodule, GitFileMode::Submodule);
    assert!(submodule.is_submodule());
    assert!(!submodule.is_file());

    let other = GitFileMode::from_mode_str("unknown_mode");
    assert_eq!(other, GitFileMode::Other("unknown_mode".to_string()));
    assert_eq!(other.as_str(), "unknown_mode");

    // Serialization & Deserialization
    let json_str = serde_json::to_string(&regular).expect("must serialize mode");
    assert_eq!(json_str, "\"100644\"");
    let deserialized: GitFileMode = serde_json::from_str(&json_str).expect("must deserialize mode");
    assert_eq!(regular, deserialized);
}

#[test]
fn test_git_tree_entry_helpers_and_path_normalization() {
    // Normalization strips leading slashes and fixes backslashes
    let entry = GitTreeEntry::new(
        "\\crates\\core\\src\\lib.rs",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
        Some(2048),
    );

    assert_eq!(entry.path, "crates/core/src/lib.rs");
    assert_eq!(entry.file_name(), "lib.rs");
    assert_eq!(entry.extension(), Some("rs"));
    assert_eq!(entry.parent_path(), Some("crates/core/src"));
    assert!(entry.is_blob());
    assert!(!entry.is_tree());
    assert!(!entry.is_submodule());
    assert_eq!(entry.depth(), 3);
    assert_eq!(entry.size, Some(2048));

    // Root file with no extension
    let root_file = GitTreeEntry::new(
        "/LICENSE",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "1111222233334444555566667777888899990000",
        Some(1065),
    );
    assert_eq!(root_file.path, "LICENSE");
    assert_eq!(root_file.file_name(), "LICENSE");
    assert_eq!(root_file.extension(), None);
    assert_eq!(root_file.parent_path(), None);
    assert_eq!(root_file.depth(), 0);

    // Subtree directory entry
    let dir_entry = GitTreeEntry::new(
        "crates/core",
        GitFileMode::Subdirectory,
        GitTreeEntryType::Tree,
        "2222333344445555666677778888999900001111",
        None,
    );
    assert!(dir_entry.is_tree());
    assert_eq!(dir_entry.file_name(), "core");
    assert_eq!(dir_entry.parent_path(), Some("crates"));
    assert_eq!(dir_entry.depth(), 1);

    // Submodule commit entry
    let submodule_entry = GitTreeEntry::new(
        "vendor/subproject",
        GitFileMode::Submodule,
        GitTreeEntryType::Commit,
        "3333444455556666777788889999000011112222",
        None,
    );
    assert!(submodule_entry.is_submodule());
    assert_eq!(submodule_entry.file_name(), "subproject");
}

#[test]
fn test_git_tree_response_queries_and_filters() {
    let entries = vec![
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha1",
            Some(1200),
        ),
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha2",
            Some(800),
        ),
        GitTreeEntry::new(
            "crates",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha3",
            None,
        ),
        GitTreeEntry::new(
            "crates/core",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "sha4",
            None,
        ),
        GitTreeEntry::new(
            "crates/core/Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha5",
            Some(500),
        ),
        GitTreeEntry::new(
            "crates/core/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha6",
            Some(3000),
        ),
        GitTreeEntry::new(
            "crates/core/src/main.rs",
            GitFileMode::ExecutableFile,
            GitTreeEntryType::Blob,
            "sha7",
            Some(1500),
        ),
        GitTreeEntry::new(
            "vendor/libgit2",
            GitFileMode::Submodule,
            GitTreeEntryType::Commit,
            "sha8",
            None,
        ),
    ];

    let response = GitTreeResponse::new("tree_sha_root", entries, false);

    // Metrics
    assert_eq!(response.blob_count(), 5);
    assert_eq!(response.tree_count(), 2);
    assert_eq!(response.submodule_count(), 1);
    assert_eq!(response.total_blob_size(), 1200 + 800 + 500 + 3000 + 1500);
    assert!(!response.truncated);

    // Blobs iterator
    let blob_paths: Vec<&str> = response.blobs().map(|b| b.path.as_str()).collect();
    assert_eq!(
        blob_paths,
        vec![
            "README.md",
            "Cargo.toml",
            "crates/core/Cargo.toml",
            "crates/core/src/lib.rs",
            "crates/core/src/main.rs"
        ]
    );

    // Find by path
    let found = response
        .find_by_path("crates/core/src/lib.rs")
        .expect("must find");
    assert_eq!(found.sha, "sha6");
    assert_eq!(found.size, Some(3000));
    assert!(response.find_by_path("nonexistent").is_none());

    // Filter by prefix
    let core_entries = response.filter_by_prefix("crates/core");
    assert_eq!(core_entries.len(), 4); // crates/core, crates/core/Cargo.toml, crates/core/src/lib.rs, crates/core/src/main.rs

    // Filter by extension
    let rs_files = response.filter_by_extension("rs");
    assert_eq!(rs_files.len(), 2);
    assert_eq!(rs_files[0].path, "crates/core/src/lib.rs");
    assert_eq!(rs_files[1].path, "crates/core/src/main.rs");

    let toml_files = response.filter_by_extension(".toml");
    assert_eq!(toml_files.len(), 2);

    // Path set
    let path_set = response.to_path_set();
    assert_eq!(path_set.len(), 8);
    assert!(path_set.contains("README.md"));
    assert!(path_set.contains("crates/core/src/lib.rs"));
}

// -----------------------------------------------------------------------------
// Live In-Process HTTP Server Integration Tests
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TreeQueryParams {
    recursive: Option<u8>,
}

#[tokio::test]
async fn test_live_http_git_trees_integration() {
    let app = Router::new().route(
        "/repos/:owner/:repo/git/trees/:tree_sha",
        get(
            |Path((owner, repo, tree_sha)): Path<(String, String, String)>,
             Query(query): Query<TreeQueryParams>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4975".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                if owner != "test-owner" || repo != "test-repo" {
                    return (StatusCode::NOT_FOUND, headers, Json(json!({"message": "Not Found"}))).into_response();
                }

                if tree_sha == "nonexistent" {
                    return (StatusCode::NOT_FOUND, headers, Json(json!({"message": "Tree Not Found"}))).into_response();
                }

                if tree_sha == "server-error" {
                    return (StatusCode::INTERNAL_SERVER_ERROR, headers, "Internal Server Error").into_response();
                }

                let is_recursive = query.recursive.unwrap_or(0) == 1;

                if is_recursive {
                    let body = json!({
                        "sha": tree_sha,
                        "url": format!("https://api.github.com/repos/test-owner/test-repo/git/trees/{tree_sha}"),
                        "tree": [
                            {
                                "path": "README.md",
                                "mode": "100644",
                                "type": "blob",
                                "sha": "blob111111111111111111111111111111111111",
                                "size": 1500,
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/blobs/blob1111"
                            },
                            {
                                "path": "crates",
                                "mode": "040000",
                                "type": "tree",
                                "sha": "tree222222222222222222222222222222222222",
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/trees/tree2222"
                            },
                            {
                                "path": "crates/core",
                                "mode": "040000",
                                "type": "tree",
                                "sha": "tree333333333333333333333333333333333333",
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/trees/tree3333"
                            },
                            {
                                "path": "crates/core/Cargo.toml",
                                "mode": "100644",
                                "type": "blob",
                                "sha": "blob444444444444444444444444444444444444",
                                "size": 650,
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/blobs/blob4444"
                            },
                            {
                                "path": "crates/core/src/lib.rs",
                                "mode": "100644",
                                "type": "blob",
                                "sha": "blob555555555555555555555555555555555555",
                                "size": 4200,
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/blobs/blob5555"
                            },
                            {
                                "path": "scripts/build.sh",
                                "mode": "100755",
                                "type": "blob",
                                "sha": "blob666666666666666666666666666666666666",
                                "size": 800,
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/blobs/blob6666"
                            },
                            {
                                "path": "vendor/subproject",
                                "mode": "160000",
                                "type": "commit",
                                "sha": "commit77777777777777777777777777777777777",
                                "url": "https://api.github.com/repos/test-owner/test-repo/git/commits/commit7777"
                            }
                        ],
                        "truncated": false
                    });
                    (StatusCode::OK, headers, Json(body)).into_response()
                } else {
                    let body = json!({
                        "sha": tree_sha,
                        "url": format!("https://api.github.com/repos/test-owner/test-repo/git/trees/{tree_sha}"),
                        "tree": [
                            {
                                "path": "README.md",
                                "mode": "100644",
                                "type": "blob",
                                "sha": "blob111111111111111111111111111111111111",
                                "size": 1500
                            },
                            {
                                "path": "crates",
                                "mode": "040000",
                                "type": "tree",
                                "sha": "tree222222222222222222222222222222222222"
                            }
                        ],
                        "truncated": false
                    });
                    (StatusCode::OK, headers, Json(body)).into_response()
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
        .with_auth_token("ghp_TreeSecretToken1234567890abcdef123456")
        .with_timeout(Duration::from_secs(5));

    let target = GitHubUrlParser::parse_url("https://github.com/test-owner/test-repo")
        .expect("must parse target");

    // 1. Recursive Git Tree fetch
    let (rec_tree, rate_limit) = client
        .get_tree(&target, "root_tree_sha_123", true)
        .await
        .expect("must fetch recursive tree");

    assert_eq!(rec_tree.sha, "root_tree_sha_123");
    assert_eq!(rec_tree.tree.len(), 7);
    assert_eq!(rec_tree.blob_count(), 4);
    assert_eq!(rec_tree.tree_count(), 2);
    assert_eq!(rec_tree.submodule_count(), 1);
    assert_eq!(rec_tree.total_blob_size(), 1500 + 650 + 4200 + 800);
    assert!(!rec_tree.truncated);

    let rl = rate_limit.expect("rate limit must be present");
    assert_eq!(rl.limit, 5000);
    assert_eq!(rl.remaining, 4975);

    // Verify executable file mode
    let build_sh = rec_tree
        .find_by_path("scripts/build.sh")
        .expect("build.sh must exist");
    assert!(build_sh.mode.is_executable());
    assert_eq!(build_sh.mode.as_str(), "100755");

    // Verify submodule
    let subproject = rec_tree
        .find_by_path("vendor/subproject")
        .expect("submodule must exist");
    assert!(subproject.is_submodule());
    assert_eq!(subproject.mode.as_str(), "160000");

    // 2. Non-recursive Git Tree fetch
    let (flat_tree, _) = client
        .get_tree(&target, "root_tree_sha_123", false)
        .await
        .expect("must fetch non-recursive tree");

    assert_eq!(flat_tree.tree.len(), 2);
    assert_eq!(flat_tree.blob_count(), 1);
    assert_eq!(flat_tree.tree_count(), 1);

    // 3. fetch_pinned_tree directly from PinnedRevision
    let commit_meta = GitHubCommitMetadata {
        sha: "commit_sha_abc123".to_string(),
        node_id: None,
        message: "test: pinned commit".to_string(),
        author: Some(CommitAuthor::new("Dev", "dev@example.com", Utc::now())),
        committer: None,
        tree_sha: "root_tree_sha_123".to_string(),
        parents: Vec::new(),
        comment_count: 0,
        verification: None,
    };

    let pinned = PinnedRevision::new(
        target.clone(),
        "commit_sha_abc123",
        "root_tree_sha_123",
        commit_meta,
        Some("main".to_string()),
    );

    let pinned_tree = client
        .fetch_pinned_tree(&pinned, true)
        .await
        .expect("must fetch pinned tree");

    assert_eq!(pinned_tree.sha, "root_tree_sha_123");
    assert_eq!(pinned_tree.tree.len(), 7);

    // 4. Error handling: 404 Tree Not Found
    let not_found_err = client
        .get_tree(&target, "nonexistent", true)
        .await
        .expect_err("nonexistent tree must fail");

    match not_found_err {
        GitHubExtractorError::ResourceNotFound { message, .. } => {
            assert!(message.contains("Tree Not Found"));
            assert!(!message.contains("ghp_TreeSecretToken"));
        }
        other => panic!("expected ResourceNotFound, got: {other:?}"),
    }

    // 5. Secret redaction verification on server error
    let server_err = client
        .get_tree(&target, "server-error", true)
        .await
        .expect_err("500 error must fail");

    let err_str = server_err.to_string();
    assert!(!err_str.contains("ghp_TreeSecretToken"));
}
