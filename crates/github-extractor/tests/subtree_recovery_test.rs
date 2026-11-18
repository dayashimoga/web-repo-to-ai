//! # Integration Tests for Git Subtree Recovery and Archive Fallback (`5.2.3`)
//!
//! Validates:
//! 1. Relative path stitching, normalization, and deduplication (`TreeStitcher`)
//! 2. In-memory zipball parsing with root directory prefix stripping (`scan_zip_archive`)
//! 3. Configuration thresholds, recursion depth, and fallback options (`SubtreeRecoveryConfig`)
//! 4. Live in-process Axum mock server recursive subtree recovery traversal
//! 5. Automatic shallow archive fallback when subtree thresholds are exceeded
//! 6. Archive fallback on upstream API failure

use std::io::{Cursor, Write};
use std::net::SocketAddr;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use web_repo_to_ai_github_extractor::{
    scan_zip_archive, stitch_child_path, ArchiveFormat, GitFileMode, GitHubApiClient,
    GitHubUrlParser, GitTreeEntry, GitTreeEntryType, GitTreeResponse, SubtreeRecoveryConfig,
    TreeStitcher,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Helper to generate a dummy in-memory zip archive with given files and directory permissions.
fn create_test_zip_archive(files: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(&mut buf);

    for (path, content, mode) in files {
        let options = SimpleFileOptions::default()
            .system(zip::System::Unix)
            .unix_permissions(*mode);
        if path.ends_with('/') {
            writer
                .add_directory(*path, options)
                .expect("Failed to add directory to zip");
        } else {
            writer
                .start_file(*path, options)
                .expect("Failed to start file in zip");
            writer
                .write_all(content)
                .expect("Failed to write content to zip");
        }
    }

    writer.finish().expect("Failed to finalize zip archive");
    buf.into_inner()
}

#[test]
fn test_path_stitching_and_normalization() {
    assert_eq!(
        stitch_child_path("crates/core", "src/lib.rs"),
        "crates/core/src/lib.rs"
    );
    assert_eq!(
        stitch_child_path("crates/core/", "/src/lib.rs"),
        "crates/core/src/lib.rs"
    );
    assert_eq!(
        stitch_child_path("crates\\core\\", "\\src\\lib.rs"),
        "crates/core/src/lib.rs"
    );
    assert_eq!(stitch_child_path("", "README.md"), "README.md");
    assert_eq!(stitch_child_path("crates/core", ""), "crates/core");
    assert_eq!(
        stitch_child_path("services/api/src", "handlers/auth.rs"),
        "services/api/src/handlers/auth.rs"
    );
}

#[test]
fn test_tree_stitcher_in_place_merging_and_deduplication() {
    let mut base_tree = GitTreeResponse::new(
        "root-sha",
        vec![
            GitTreeEntry::new(
                "Cargo.toml",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "sha-cargo",
                Some(500),
            ),
            GitTreeEntry::new(
                "src",
                GitFileMode::Subdirectory,
                GitTreeEntryType::Tree,
                "sha-src",
                None,
            ),
        ],
        true,
    );

    let child_subtree = GitTreeResponse::new(
        "sha-src",
        vec![
            GitTreeEntry::new(
                "lib.rs",
                GitFileMode::RegularFile,
                GitTreeEntryType::Blob,
                "sha-lib",
                Some(1200),
            ),
            GitTreeEntry::new(
                "util",
                GitFileMode::Subdirectory,
                GitTreeEntryType::Tree,
                "sha-util",
                None,
            ),
        ],
        false,
    );

    let (blobs_added, trees_added) =
        TreeStitcher::stitch_subtree(&mut base_tree, "src", &child_subtree);

    assert_eq!(blobs_added, 1);
    assert_eq!(trees_added, 1);
    assert_eq!(base_tree.tree.len(), 4);

    let lib_entry = base_tree.find_by_path("src/lib.rs");
    assert!(lib_entry.is_some());
    assert_eq!(lib_entry.unwrap().size, Some(1200));

    let util_entry = base_tree.find_by_path("src/util");
    assert!(util_entry.is_some());
    assert!(util_entry.unwrap().is_tree());

    // Re-stitching identical entries must be a no-op (zero duplicates)
    let (blobs_again, trees_again) =
        TreeStitcher::stitch_subtree(&mut base_tree, "src", &child_subtree);
    assert_eq!(blobs_again, 0);
    assert_eq!(trees_again, 0);
    assert_eq!(base_tree.tree.len(), 4);
}

#[test]
fn test_recovery_config_defaults_and_builders() {
    let default_config = SubtreeRecoveryConfig::default();
    assert_eq!(default_config.max_subtrees, 500);
    assert_eq!(default_config.max_depth, 10);
    assert!(default_config.auto_fallback);
    assert_eq!(default_config.concurrency, 1);

    let custom = SubtreeRecoveryConfig::new()
        .with_max_subtrees(50)
        .with_max_depth(5)
        .with_auto_fallback(false)
        .with_concurrency(4);

    assert_eq!(custom.max_subtrees, 50);
    assert_eq!(custom.max_depth, 5);
    assert!(!custom.auto_fallback);
    assert_eq!(custom.concurrency, 4);
}

#[test]
fn test_in_memory_zip_archive_parsing() {
    let zip_bytes = create_test_zip_archive(&[
        (
            "acme-repo-1a2b3c4/Cargo.toml",
            b"[package]\nname = \"acme\"",
            0o644,
        ),
        ("acme-repo-1a2b3c4/src/", b"", 0o755),
        ("acme-repo-1a2b3c4/src/main.rs", b"fn main() {}", 0o644),
        ("acme-repo-1a2b3c4/scripts/deploy.sh", b"#!/bin/bash", 0o755),
    ]);

    let tree_response = scan_zip_archive(&zip_bytes, true).expect("Zip parsing failed");

    assert_eq!(tree_response.blob_count(), 3);
    assert_eq!(tree_response.tree_count(), 1);

    let cargo = tree_response
        .find_by_path("Cargo.toml")
        .expect("Cargo.toml missing");
    assert_eq!(cargo.mode, GitFileMode::RegularFile);

    let deploy_script = tree_response
        .find_by_path("scripts/deploy.sh")
        .expect("deploy.sh missing");
    assert_eq!(deploy_script.mode, GitFileMode::ExecutableFile);

    let src_dir = tree_response.find_by_path("src").expect("src dir missing");
    assert_eq!(src_dir.mode, GitFileMode::Subdirectory);

    let completeness = tree_response.completeness();
    assert!(completeness.is_complete());
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TreeQuery {
    recursive: Option<u8>,
}

#[tokio::test]
async fn test_live_http_recursive_subtree_recovery_axum() {
    let app = Router::new()
        .route(
            "/repos/test-org/test-repo/git/trees/root-sha",
            get(|Query(_q): Query<TreeQuery>| async {
                Json(json!({
                    "sha": "root-sha",
                    "truncated": true,
                    "tree": [
                        {
                            "path": "README.md",
                            "mode": "100644",
                            "type": "blob",
                            "sha": "sha-readme",
                            "size": 350
                        },
                        {
                            "path": "src",
                            "mode": "040000",
                            "type": "tree",
                            "sha": "sha-src-subtree"
                        },
                        {
                            "path": "docs",
                            "mode": "040000",
                            "type": "tree",
                            "sha": "sha-docs-subtree"
                        }
                    ]
                }))
            }),
        )
        .route(
            "/repos/test-org/test-repo/git/trees/sha-src-subtree",
            get(|Query(_q): Query<TreeQuery>| async {
                Json(json!({
                    "sha": "sha-src-subtree",
                    "truncated": false,
                    "tree": [
                        {
                            "path": "lib.rs",
                            "mode": "100644",
                            "type": "blob",
                            "sha": "sha-lib",
                            "size": 1200
                        },
                        {
                            "path": "main.rs",
                            "mode": "100644",
                            "type": "blob",
                            "sha": "sha-main",
                            "size": 800
                        }
                    ]
                }))
            }),
        )
        .route(
            "/repos/test-org/test-repo/git/trees/sha-docs-subtree",
            get(|Query(_q): Query<TreeQuery>| async {
                Json(json!({
                    "sha": "sha-docs-subtree",
                    "truncated": false,
                    "tree": [
                        {
                            "path": "ARCHITECTURE.md",
                            "mode": "100644",
                            "type": "blob",
                            "sha": "sha-arch",
                            "size": 4500
                        }
                    ]
                }))
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral TCP port");
    let addr: SocketAddr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));
    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();

    let (initial_tree, _rl) = client.get_tree(&target, "root-sha", true).await.unwrap();
    assert!(initial_tree.is_truncated());

    let config = SubtreeRecoveryConfig::new().with_max_subtrees(10);
    let (recovered_tree, report) = client
        .recover_tree(&target, initial_tree, &config, None)
        .await
        .expect("Subtree recovery failed");

    assert_eq!(report.subtrees_attempted, 2);
    assert_eq!(report.subtrees_succeeded, 2);
    assert_eq!(report.subtrees_failed, 0);
    assert_eq!(report.additional_blobs_recovered, 3);
    assert_eq!(report.final_total_entries, 6);
    assert!(report.is_fully_recovered());
    assert!(!report.fallback_triggered);

    assert!(recovered_tree.find_by_path("README.md").is_some());
    assert!(recovered_tree.find_by_path("src/lib.rs").is_some());
    assert!(recovered_tree.find_by_path("src/main.rs").is_some());
    assert!(recovered_tree
        .find_by_path("docs/ARCHITECTURE.md")
        .is_some());
    assert!(!recovered_tree.is_truncated());
}

#[tokio::test]
async fn test_live_http_archive_fallback_on_exceeded_subtrees() {
    let zip_bytes = create_test_zip_archive(&[
        (
            "test-repo-pinned123/Cargo.toml",
            b"[package]\nname = \"large-repo\"",
            0o644,
        ),
        ("test-repo-pinned123/src/lib.rs", b"pub fn run() {}", 0o644),
        ("test-repo-pinned123/pkg1/mod.rs", b"// pkg1", 0o644),
        ("test-repo-pinned123/pkg2/mod.rs", b"// pkg2", 0o644),
        ("test-repo-pinned123/pkg3/mod.rs", b"// pkg3", 0o644),
    ]);

    let app = Router::new()
        .route(
            "/repos/test-org/test-repo/git/trees/root-truncated",
            get(|| async {
                Json(json!({
                    "sha": "root-truncated",
                    "truncated": true,
                    "tree": [
                        { "path": "pkg1", "mode": "040000", "type": "tree", "sha": "sha-1" },
                        { "path": "pkg2", "mode": "040000", "type": "tree", "sha": "sha-2" },
                        { "path": "pkg3", "mode": "040000", "type": "tree", "sha": "sha-3" }
                    ]
                }))
            }),
        )
        .route(
            "/repos/test-org/test-repo/zipball/pinned123",
            get(move || {
                let bytes = zip_bytes.clone();
                async move {
                    (
                        StatusCode::OK,
                        [("content-type", ArchiveFormat::Zipball.media_type())],
                        bytes,
                    )
                        .into_response()
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral TCP port");
    let addr: SocketAddr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));
    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();

    let (initial_tree, _rl) = client
        .get_tree(&target, "root-truncated", true)
        .await
        .unwrap();

    // Set max_subtrees = 2 (smaller than the 3 unexpanded subtrees present)
    let config = SubtreeRecoveryConfig::new()
        .with_max_subtrees(2)
        .with_auto_fallback(true);

    let (recovered_tree, report) = client
        .recover_tree(&target, initial_tree, &config, Some("pinned123"))
        .await
        .expect("Recovery with archive fallback failed");

    assert!(report.fallback_triggered);
    assert!(report.is_fully_recovered());
    assert_eq!(recovered_tree.blob_count(), 5);
    assert!(recovered_tree.find_by_path("Cargo.toml").is_some());
    assert!(recovered_tree.find_by_path("src/lib.rs").is_some());
    assert!(recovered_tree.find_by_path("pkg1/mod.rs").is_some());
}

#[tokio::test]
async fn test_live_http_archive_fallback_on_traversal_failure() {
    let zip_bytes = create_test_zip_archive(&[
        (
            "test-repo-fallback456/README.md",
            b"# Fallback Success",
            0o644,
        ),
        ("test-repo-fallback456/src/lib.rs", b"pub fn ok() {}", 0o644),
    ]);

    let app = Router::new()
        .route(
            "/repos/test-org/test-repo/git/trees/root-failing",
            get(|| async {
                Json(json!({
                    "sha": "root-failing",
                    "truncated": true,
                    "tree": [
                        { "path": "broken-subtree", "mode": "040000", "type": "tree", "sha": "sha-broken" }
                    ]
                }))
            }),
        )
        .route(
            "/repos/test-org/test-repo/git/trees/sha-broken",
            get(|| async {
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        )
        .route(
            "/repos/test-org/test-repo/zipball/fallback456",
            get(move || {
                let bytes = zip_bytes.clone();
                async move {
                    (
                        StatusCode::OK,
                        [("content-type", ArchiveFormat::Zipball.media_type())],
                        bytes,
                    )
                        .into_response()
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral TCP port");
    let addr: SocketAddr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    let client = GitHubApiClient::new().with_base_url_override(format!("http://{addr}"));
    let target = GitHubUrlParser::parse_url("https://github.com/test-org/test-repo").unwrap();

    let (initial_tree, _rl) = client
        .get_tree(&target, "root-failing", true)
        .await
        .unwrap();

    let config = SubtreeRecoveryConfig::new()
        .with_max_subtrees(5)
        .with_auto_fallback(true);

    let (recovered_tree, report) = client
        .recover_tree(&target, initial_tree, &config, Some("fallback456"))
        .await
        .expect("Recovery with fallback failed");

    assert_eq!(report.subtrees_attempted, 1);
    assert_eq!(report.subtrees_failed, 1);
    assert!(report.fallback_triggered);
    assert!(report.is_fully_recovered());
    assert!(recovered_tree.find_by_path("README.md").is_some());
    assert!(recovered_tree.find_by_path("src/lib.rs").is_some());
}
