//! # CLI Batch Conversion Command Integration Tests
//!
//! Validates `web-repo-to-ai batch`:
//! - Newline-delimited and JSON array input files
//! - Comment and whitespace stripping
//! - Mixed local directory and web URL batch processing
//! - Bounded concurrency via Semaphore
//! - Continue-on-error tolerance and granular result accounting
//! - Structured JSON summary envelopes
//! - Output file generation in destination directory

use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use web_repo_to_ai_cli::commands::batch::{
    classify_target, execute_batch_with_options, load_targets, sanitize_filename, BatchTargetKind,
};
use web_repo_to_ai_cli::{run_cli_with_args, BatchArgs, ExitCode, OutputFormat, OutputWriter};
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::http::MockHttpTransport;

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(200);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "web_repo_to_ai_test_batch_{prefix}_{}_{seq}",
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

fn setup_mock_url_environment() -> (Arc<MockHttpTransport>, SecureDnsResolver) {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = Arc::new(MockDnsResolver::new());
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            443,
        )],
    );
    let resolver = SecureDnsResolver::with_resolver(policy, mock_dns);

    let mock_transport = Arc::new(MockHttpTransport::new());
    mock_transport.set_html(
        "https://example.com/page1",
        "<html><head><title>Page 1</title></head><body><article><h1>Doc One</h1><p>Content of page 1</p></article></body></html>",
    );
    mock_transport.set_html(
        "https://example.com/page2",
        "<html><head><title>Page 2</title></head><body><article><h1>Doc Two</h1><p>Content of page 2</p></article></body></html>",
    );

    (mock_transport, resolver)
}

#[tokio::test]
async fn test_batch_cli_help() {
    let res = run_cli_with_args(["web-repo-to-ai", "batch", "--help"]).await;
    assert_eq!(res.unwrap(), 0);
}

#[tokio::test]
async fn test_batch_empty_targets_validation_error() {
    let res = run_cli_with_args(["web-repo-to-ai", "batch"]).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.exit_code(), ExitCode::Validation.code());
}

#[test]
fn test_batch_classification_and_filename_sanitization() {
    assert_eq!(
        classify_target("https://example.com/doc"),
        BatchTargetKind::Url
    );
    assert_eq!(
        classify_target("https://github.com/owner/repo"),
        BatchTargetKind::GitHub
    );
    assert_eq!(classify_target("rust-lang/rust"), BatchTargetKind::GitHub);
    assert_eq!(
        classify_target("rust-lang/rust@v1.80.0"),
        BatchTargetKind::GitHub
    );

    // Filename sanitization
    let fn1 = sanitize_filename(0, "https://example.com/docs/intro", OutputFormat::Markdown);
    assert_eq!(fn1, "001_example_com_docs_intro.md");

    let fn2 = sanitize_filename(1, "octocat/Hello-World", OutputFormat::Json);
    assert_eq!(fn2, "002_octocat_Hello_World.json");

    let fn3 = sanitize_filename(2, "./crates/cli", OutputFormat::Html);
    assert_eq!(fn3, "003_crates_cli.html");
}

#[test]
fn test_batch_load_targets_parsing() {
    let temp = TestDir::new("parse");
    let input_txt = temp.path().join("targets.txt");
    fs::write(
        &input_txt,
        r#"
# Main API documentation
https://example.com/docs/api

// Secondary guide
https://example.com/docs/guide

# Repositories
owner/repo1
owner/repo2

"#,
    )
    .unwrap();

    let args = BatchArgs {
        input_file: Some(input_txt),
        targets: vec!["https://example.com/extra".to_string()],
        output_dir: PathBuf::from("./out"),
        format: OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        concurrency: 4,
        continue_on_error: true,
        timeout: 60,
        github_token: None,
        api_url: None,
        api_key: None,
    };

    let targets = load_targets(&args).expect("load targets");
    assert_eq!(
        targets,
        vec![
            "https://example.com/docs/api",
            "https://example.com/docs/guide",
            "owner/repo1",
            "owner/repo2",
            "https://example.com/extra"
        ]
    );

    // JSON array input
    let input_json = temp.path().join("targets.json");
    fs::write(
        &input_json,
        r#"["https://example.com/one", "https://example.com/two"]"#,
    )
    .unwrap();

    let json_args = BatchArgs {
        input_file: Some(input_json),
        targets: Vec::new(),
        output_dir: PathBuf::from("./out"),
        format: OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        concurrency: 4,
        continue_on_error: true,
        timeout: 60,
        github_token: None,
        api_url: None,
        api_key: None,
    };

    let json_targets = load_targets(&json_args).expect("load json targets");
    assert_eq!(
        json_targets,
        vec!["https://example.com/one", "https://example.com/two"]
    );
}

#[tokio::test]
async fn test_batch_local_directories_execution() {
    let temp = TestDir::new("local_batch");

    // Project 1
    let proj1 = temp.path().join("project1");
    fs::create_dir_all(proj1.join("src")).unwrap();
    fs::write(proj1.join("README.md"), "# Project 1\nAlpha version.").unwrap();
    fs::write(proj1.join("src/lib.rs"), "pub fn alpha() -> u32 { 1 }").unwrap();

    // Project 2
    let proj2 = temp.path().join("project2");
    fs::create_dir_all(proj2.join("src")).unwrap();
    fs::write(proj2.join("README.md"), "# Project 2\nBeta version.").unwrap();
    fs::write(
        proj2.join("src/main.rs"),
        "fn main() { println!(\"beta\"); }",
    )
    .unwrap();

    let out_dir = temp.path().join("batch_out");

    let res = run_cli_with_args([
        "web-repo-to-ai",
        "batch",
        proj1.to_str().unwrap(),
        proj2.to_str().unwrap(),
        "--output-dir",
        out_dir.to_str().unwrap(),
        "--format",
        "markdown",
        "--concurrency",
        "2",
    ])
    .await;

    assert_eq!(res.unwrap(), ExitCode::Success as i32);

    // Verify files generated in out_dir
    let files: Vec<_> = fs::read_dir(&out_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(files.len(), 2);
    assert!(files
        .iter()
        .any(|f| f.ends_with(".md") && f.contains("project1")));
    assert!(files
        .iter()
        .any(|f| f.ends_with(".md") && f.contains("project2")));

    // Verify content of one generated file
    let p1_file = files.iter().find(|f| f.contains("project1")).unwrap();
    let content = fs::read_to_string(out_dir.join(p1_file)).unwrap();
    assert!(content.contains("# Project 1"));
    assert!(content.contains("pub fn alpha()"));
}

#[tokio::test]
async fn test_batch_with_mock_urls_and_concurrency() {
    let temp = TestDir::new("url_batch");
    let out_dir = temp.path().join("batch_url_out");
    let (transport, resolver) = setup_mock_url_environment();

    let writer = OutputWriter::new(false, true, false);

    let args = BatchArgs {
        input_file: None,
        targets: vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
        ],
        output_dir: out_dir.clone(),
        format: OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        concurrency: 2,
        continue_on_error: true,
        timeout: 30,
        github_token: None,
        api_url: None,
        api_key: None,
    };

    let res =
        execute_batch_with_options(&args, &writer, Some(transport), Some(resolver), None).await;

    assert_eq!(res.unwrap(), ExitCode::Success as i32);

    let entries: Vec<_> = fs::read_dir(&out_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|f| f.contains("example_com_page1")));
    assert!(entries.iter().any(|f| f.contains("example_com_page2")));
}

#[tokio::test]
async fn test_batch_continue_on_error() {
    let temp = TestDir::new("error_batch");
    let valid_proj = temp.path().join("valid_project");
    fs::create_dir_all(&valid_proj).unwrap();
    fs::write(valid_proj.join("README.md"), "# Valid Project").unwrap();

    let invalid_target = temp.path().join("non_existent_folder_xyz");
    let out_dir = temp.path().join("out");

    let writer = OutputWriter::new(false, true, false);

    // 1. With continue_on_error = true (returns 1 for partial failure, but completes valid items)
    let args_continue = BatchArgs {
        input_file: None,
        targets: vec![
            valid_proj.to_str().unwrap().to_string(),
            invalid_target.to_str().unwrap().to_string(),
        ],
        output_dir: out_dir.clone(),
        format: OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        concurrency: 2,
        continue_on_error: true,
        timeout: 10,
        github_token: None,
        api_url: None,
        api_key: None,
    };

    let res = execute_batch_with_options(&args_continue, &writer, None, None, None).await;
    // Returns ExitCode::Validation (1) due to 1 failed target
    assert_eq!(res.unwrap(), ExitCode::Validation as i32);

    // The valid item still wrote its artifact
    let entries: Vec<_> = fs::read_dir(&out_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(entries.len(), 1);

    // 2. With continue_on_error = false (returns Err)
    let out_dir_abort = temp.path().join("out_abort");
    let args_abort = BatchArgs {
        input_file: None,
        targets: vec![
            invalid_target.to_str().unwrap().to_string(),
            valid_proj.to_str().unwrap().to_string(),
        ],
        output_dir: out_dir_abort,
        format: OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        concurrency: 1,
        continue_on_error: false,
        timeout: 10,
        github_token: None,
        api_url: None,
        api_key: None,
    };

    let res_abort = execute_batch_with_options(&args_abort, &writer, None, None, None).await;
    assert!(res_abort.is_err());
}
