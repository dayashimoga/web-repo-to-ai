//! # Repository Conversion Command Integration Tests
//!
//! Validates the full CLI repository ingestion, validation, security controls,
//! local filesystem walking, GitHub REST API client integration, multi-format
//! artifact export, token budget optimization, and JSON envelope output.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_cli::{run_cli_with_args, run_cli_with_args_and_clients, ExitCode};
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_github_extractor::source::compute_git_blob_sha;

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "web_repo_to_ai_test_{prefix}_{}_{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn test_repo_cli_help_and_version() {
    let code_repo = run_cli_with_args(["web-repo-to-ai", "repo", "--help"]).await;
    assert_eq!(code_repo.unwrap(), 0);

    let code_alias = run_cli_with_args(["web-repo-to-ai", "repository", "--help"]).await;
    assert_eq!(code_alias.unwrap(), 0);
}

#[tokio::test]
async fn test_repo_invalid_targets_and_validation_errors() {
    // Missing repository parameter
    let res = run_cli_with_args(["web-repo-to-ai", "repo"]).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.exit_code(), ExitCode::Validation.code());

    // Invalid format
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        "owner/repo",
        "--format",
        "unsupported_fmt",
    ])
    .await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().exit_code(), ExitCode::Validation.code());

    // Malformed target syntax (not a directory, and not a valid GitHub identifier)
    let res = run_cli_with_args(["web-repo-to-ai", "repo", "not_a_valid_repo_target"]).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().exit_code(), ExitCode::Validation.code());
}

#[tokio::test]
async fn test_repo_security_directory_traversal() {
    // Directory traversal in subpath parameter
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        "owner/repo",
        "--subpath",
        "../etc/passwd",
    ])
    .await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );

    // Directory traversal in repository target
    let res = run_cli_with_args(["web-repo-to-ai", "repo", "owner/../repo"]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );
}

#[tokio::test]
async fn test_repo_local_directory_to_stdout() {
    let tmp = TestDir::new("repo_stdout");
    let readme = tmp.path().join("README.md");
    fs::write(&readme, "# Local Project\n\nDeterministic compilation.").unwrap();

    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    let main_rs = src.join("main.rs");
    fs::write(&main_rs, "fn main() { println!(\"Hello World!\"); }\n").unwrap();

    // Binary file that must be skipped by binary detection
    let binary = tmp.path().join("asset.bin");
    fs::write(&binary, [0u8, 159, 255, 0, 0, 1]).unwrap();

    let code = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        tmp.path().to_str().unwrap(),
        "-o",
        "-",
    ])
    .await;

    assert_eq!(code.unwrap(), 0);
}

#[tokio::test]
async fn test_repo_local_directory_multi_format_outputs() {
    let tmp = TestDir::new("repo_formats_in");
    let out_dir = TestDir::new("repo_formats_out");

    let readme = tmp.path().join("README.md");
    fs::write(&readme, "# Multi-Format Test\n\nVerifying all 8 formats.").unwrap();

    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    let lib_rs = src.join("lib.rs");
    fs::write(&lib_rs, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

    let formats = [
        ("markdown", "doc.md"),
        ("txt", "doc.txt"),
        ("json", "doc.json"),
        ("html", "doc.html"),
        ("jsonl", "doc.jsonl"),
        ("zip", "bundle.zip"),
        ("manifest", "manifest.json"),
        ("checksums", "checksums.txt"),
    ];

    for (fmt_name, file_name) in formats {
        let out_path = out_dir.path().join(file_name);
        let out_str = out_path.to_str().unwrap();

        let code = run_cli_with_args([
            "web-repo-to-ai",
            "repo",
            tmp.path().to_str().unwrap(),
            "--format",
            fmt_name,
            "-o",
            out_str,
        ])
        .await;

        assert_eq!(
            code.unwrap(),
            0,
            "Format {fmt_name} conversion should succeed"
        );
        assert!(out_path.exists(), "Output {file_name} must exist");

        let content = fs::read(&out_path).unwrap();
        assert!(!content.is_empty(), "{file_name} must not be empty");

        match fmt_name {
            "json" => {
                let parsed: Result<CdmDocument, _> = serde_json::from_slice(&content);
                assert!(parsed.is_ok(), "JSON format must deserialize into CDM");
                let doc = parsed.unwrap();
                assert_eq!(doc.sections.len(), 2);
            }
            "zip" => {
                // ZIP magic bytes PK\x03\x04
                assert_eq!(&content[0..4], b"PK\x03\x04");
            }
            "checksums" => {
                let text = String::from_utf8(content).unwrap();
                assert!(
                    text.contains("document.md"),
                    "Checksums manifest must record file entries"
                );
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_repo_local_directory_subpath_filter() {
    let tmp = TestDir::new("repo_subpath_in");
    let out_dir = TestDir::new("repo_subpath_out");

    let docs = tmp.path().join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("manual.md"), "# Manual\n\nDocs content.").unwrap();

    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("lib.rs"), "pub fn compute() -> i32 { 42 }\n").unwrap();

    let out_json = out_dir.path().join("filtered.json");

    let code = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        tmp.path().to_str().unwrap(),
        "--subpath",
        "src",
        "-f",
        "json",
        "-o",
        out_json.to_str().unwrap(),
    ])
    .await;

    assert_eq!(code.unwrap(), 0);

    let content = fs::read(&out_json).unwrap();
    let doc: CdmDocument = serde_json::from_slice(&content).unwrap();

    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].section_id, "src/lib.rs");
}

#[tokio::test]
async fn test_repo_local_directory_budget_optimization() {
    let tmp = TestDir::new("repo_budget");

    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();

    let mut big_body = String::new();
    for i in 0..50 {
        big_body.push_str(&format!(
            "// Line {i}: High throughput context optimization for token bounded ingestion.\n"
        ));
    }
    fs::write(src.join("big.rs"), big_body).unwrap();

    // 1. With tight budget ceiling
    let code_opt = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        tmp.path().to_str().unwrap(),
        "-b",
        "50",
        "-o",
        "-",
    ])
    .await;
    assert_eq!(code_opt.unwrap(), 0);

    // 2. With --no-optimize
    let code_no_opt = run_cli_with_args([
        "web-repo-to-ai",
        "repo",
        tmp.path().to_str().unwrap(),
        "-b",
        "50",
        "--no-optimize",
        "-o",
        "-",
    ])
    .await;
    assert_eq!(code_no_opt.unwrap(), 0);
}

#[tokio::test]
async fn test_repo_json_execution_envelope() {
    let tmp = TestDir::new("repo_envelope");
    fs::write(
        tmp.path().join("README.md"),
        "# JSON Envelope Test\n\nTesting envelope serialization.",
    )
    .unwrap();

    let code = run_cli_with_args([
        "web-repo-to-ai",
        "--json",
        "repo",
        tmp.path().to_str().unwrap(),
        "-o",
        "-",
    ])
    .await;

    assert_eq!(code.unwrap(), 0);
}

#[tokio::test]
async fn test_repo_github_api_with_mock_server() {
    let test_commit_sha = "1111111111111111111111111111111111111111";

    let b1_content = b"# Hello World\n";
    let b1_sha = compute_git_blob_sha(b1_content);
    let b2_content = b"fn main() {\n    println!(\"Hello\");\n}\n";
    let b2_sha = compute_git_blob_sha(b2_content);

    let b1_sha_clone = b1_sha.clone();
    let b2_sha_clone = b2_sha.clone();
    let b1_sha_for_tree = b1_sha.clone();
    let b2_sha_for_tree = b2_sha.clone();

    let app = Router::new()
        .route(
            "/repos/octocat/hello-world",
            get(|| async {
                Json(json!({
                    "id": 1296269,
                    "node_id": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
                    "name": "hello-world",
                    "full_name": "octocat/hello-world",
                    "owner": { "login": "octocat" },
                    "description": "My first repository",
                    "default_branch": "main",
                    "stargazers_count": 80,
                    "watchers_count": 80,
                    "forks_count": 9,
                    "open_issues_count": 0,
                    "size": 108
                }))
            }),
        )
        .route(
            "/repos/octocat/hello-world/commits/main",
            get(|| async {
                Json(json!({
                    "sha": "1111111111111111111111111111111111111111",
                    "commit": {
                        "author": { "name": "Octocat", "email": "octocat@github.com", "date": "2026-11-19T00:00:00Z" },
                        "committer": { "name": "Octocat", "email": "octocat@github.com", "date": "2026-11-19T00:00:00Z" },
                        "message": "Initial commit",
                        "tree": { "sha": "tree123" }
                    },
                    "parents": []
                }))
            }),
        )
        .route(
            &format!("/repos/octocat/hello-world/git/trees/{test_commit_sha}"),
            get(move || {
                let s1 = b1_sha_for_tree.clone();
                let s2 = b2_sha_for_tree.clone();
                async move {
                    Json(json!({
                        "sha": "tree123",
                        "tree": [
                            { "path": "README.md", "mode": "100644", "type": "blob", "sha": s1, "size": 14 },
                            { "path": "src/main.rs", "mode": "100644", "type": "blob", "sha": s2, "size": 33 }
                        ],
                        "truncated": false
                    }))
                }
            }),
        )
        .route(
            &format!("/repos/octocat/hello-world/git/blobs/{b1_sha}"),
            get(move || {
                let s1 = b1_sha_clone.clone();
                async move {
                    // Base64 encoding of "# Hello World\n" is "IyBIZWxsbyBXb3JsZAo="
                    Json(json!({
                        "sha": s1,
                        "size": 14,
                        "encoding": "base64",
                        "content": "IyBIZWxsbyBXb3JsZAo="
                    }))
                }
            }),
        )
        .route(
            &format!("/repos/octocat/hello-world/git/blobs/{b2_sha}"),
            get(move || {
                let s2 = b2_sha_clone.clone();
                async move {
                    // Base64 encoding of "fn main() {\n    println!(\"Hello\");\n}\n"
                    Json(json!({
                        "sha": s2,
                        "size": 33,
                        "encoding": "base64",
                        "content": "Zm4gbWFpbigpIHsKICAgIHByaW50bG4hKCJIZWxsbyIpOwp9Cg=="
                    }))
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("mock server serve");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(format!("http://{addr}"))
        .with_timeout(Duration::from_secs(5));

    let out_dir = TestDir::new("repo_mock_out");
    let out_file = out_dir.path().join("github_repo.md");

    let code = run_cli_with_args_and_clients(
        [
            "web-repo-to-ai",
            "repo",
            "octocat/hello-world",
            "-o",
            out_file.to_str().unwrap(),
        ],
        None,
        None,
        Some(client),
    )
    .await;

    assert_eq!(code.unwrap(), 0);
    assert!(out_file.exists());

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("hello-world"));
    assert!(content.contains("README.md"));
    assert!(content.contains("src/main.rs"));
}
