//! Integration tests for cryptographic content hash validation across extracted repository files,
//! reports, manifests, and CDM sections.

use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use web_repo_to_ai_cdm::{CdmSectionBuilder, SectionKind, SectionTokenMetrics};
use web_repo_to_ai_github_extractor::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, ConfigCategory,
    ConfigTestExtractionReport, ContentHashValidator, DependencyExtractionReport,
    DocExtractionReport, DocFormat, DocType, ExtractedConfigFile, ExtractedDependencyFile,
    ExtractedDocFile, ExtractedSourceFile, FileExtractionStatus, GitHubApiClient, HashAlgorithm,
    HashValidationConfig, HashValidationReport, ManifestFileEntry, ManifestRole, PackageEcosystem,
    RepoFileManifest, SourceExtractionReport,
};

#[test]
fn test_valid_content_hash_verification() {
    let content = b"pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let git_sha = compute_git_blob_sha(content);
    let blake3 = compute_blake3_hash(content);
    let sha256 = compute_sha256_hash(content);

    let config = HashValidationConfig::strict();
    let mut report = HashValidationReport::new("owner/repo");

    ContentHashValidator::validate_file_bytes(
        "src/lib.rs",
        content,
        Some(&git_sha),
        Some(&blake3),
        Some(&sha256),
        &config,
        &mut report,
    );

    assert_eq!(report.total_files_checked, 1);
    assert_eq!(report.total_failures, 0);
    assert!(report.is_valid());
    assert_eq!(report.git_sha_metrics.passed, 1);
    assert_eq!(report.git_sha_metrics.failed, 0);
    assert_eq!(report.blake3_metrics.passed, 1);
    assert_eq!(report.blake3_metrics.failed, 0);
    assert_eq!(report.sha256_metrics.passed, 1);
    assert_eq!(report.sha256_metrics.failed, 0);
}

#[test]
fn test_detect_git_blob_sha_mismatch() {
    let content = b"fn main() {}";
    let wrong_git_sha = "1000000000000000000000000000000000000000";
    let blake3 = compute_blake3_hash(content);
    let sha256 = compute_sha256_hash(content);

    let config = HashValidationConfig::strict();
    let mut report = HashValidationReport::new("owner/repo");

    ContentHashValidator::validate_file_bytes(
        "src/main.rs",
        content,
        Some(wrong_git_sha),
        Some(&blake3),
        Some(&sha256),
        &config,
        &mut report,
    );

    assert_eq!(report.total_failures, 1);
    assert!(!report.is_valid());
    assert_eq!(report.git_sha_metrics.failed, 1);
    assert_eq!(report.blake3_metrics.passed, 1);
    assert_eq!(report.sha256_metrics.passed, 1);

    let failure = &report.failures[0];
    assert_eq!(failure.path, "src/main.rs");
    assert_eq!(failure.algorithm, HashAlgorithm::GitBlobSha1);
    assert_eq!(failure.expected_hash, wrong_git_sha);
    assert_eq!(failure.calculated_hash, compute_git_blob_sha(content));
}

#[test]
fn test_detect_blake3_mismatch() {
    let content = b"# Documentation Heading";
    let git_sha = compute_git_blob_sha(content);
    let wrong_blake3 = "0000000000000000000000000000000000000000000000000000000000000000";
    let sha256 = compute_sha256_hash(content);

    let config = HashValidationConfig::strict();
    let mut report = HashValidationReport::new("owner/repo");

    ContentHashValidator::validate_file_bytes(
        "README.md",
        content,
        Some(&git_sha),
        Some(wrong_blake3),
        Some(&sha256),
        &config,
        &mut report,
    );

    assert_eq!(report.total_failures, 1);
    assert!(!report.is_valid());
    assert_eq!(report.blake3_metrics.failed, 1);
    assert_eq!(report.failures[0].algorithm, HashAlgorithm::Blake3);
}

#[test]
fn test_detect_sha256_mismatch() {
    let content = b"[package]\nname = \"test\"";
    let git_sha = compute_git_blob_sha(content);
    let blake3 = compute_blake3_hash(content);
    let wrong_sha256 = "1111111111111111111111111111111111111111111111111111111111111111";

    let config = HashValidationConfig::strict();
    let mut report = HashValidationReport::new("owner/repo");

    ContentHashValidator::validate_file_bytes(
        "Cargo.toml",
        content,
        Some(&git_sha),
        Some(&blake3),
        Some(wrong_sha256),
        &config,
        &mut report,
    );

    assert_eq!(report.total_failures, 1);
    assert!(!report.is_valid());
    assert_eq!(report.sha256_metrics.failed, 1);
    assert_eq!(report.failures[0].algorithm, HashAlgorithm::Sha256);
}

#[test]
fn test_detect_malformed_hex_hash() {
    let content = b"content";
    let malformed_short = "tooshort";

    let config = HashValidationConfig::strict();
    let mut report = HashValidationReport::new("owner/repo");

    ContentHashValidator::validate_file_bytes(
        "file.txt",
        content,
        Some(malformed_short),
        None,
        None,
        &config,
        &mut report,
    );

    assert_eq!(report.total_failures, 1);
    assert!(report.failures[0]
        .context
        .contains("not a valid 40-character lowercase hex digest"));
}

#[test]
fn test_multi_report_validation() {
    let src_bytes = b"fn run() {}";
    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/run.rs".to_string(),
        git_sha: compute_git_blob_sha(src_bytes),
        content: String::from_utf8_lossy(src_bytes).to_string(),
        byte_size: src_bytes.len(),
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: compute_blake3_hash(src_bytes),
        sha256_hash: compute_sha256_hash(src_bytes),
        is_truncated: false,
    });

    let doc_bytes = b"# Guide";
    let mut docs = DocExtractionReport::new();
    docs.add_extracted(ExtractedDocFile {
        path: "docs/GUIDE.md".to_string(),
        git_sha: compute_git_blob_sha(doc_bytes),
        doc_type: DocType::Architecture,
        format: DocFormat::Markdown,
        title: "Guide".to_string(),
        has_frontmatter: false,
        content: String::from_utf8_lossy(doc_bytes).to_string(),
        byte_size: doc_bytes.len(),
        line_count: 1,
        blake3_hash: compute_blake3_hash(doc_bytes),
        sha256_hash: compute_sha256_hash(doc_bytes),
    });

    let dep_bytes = b"[dependencies]";
    let mut deps = DependencyExtractionReport::new();
    deps.add_extracted(ExtractedDependencyFile {
        path: "Cargo.toml".to_string(),
        git_sha: compute_git_blob_sha(dep_bytes),
        ecosystem: PackageEcosystem::Cargo,
        role: ManifestRole::Manifest,
        content: String::from_utf8_lossy(dep_bytes).to_string(),
        parsed: None,
        byte_size: dep_bytes.len(),
        line_count: 1,
        blake3_hash: compute_blake3_hash(dep_bytes),
        sha256_hash: compute_sha256_hash(dep_bytes),
    });

    let cfg_bytes = b"name: CI";
    let mut cfgs = ConfigTestExtractionReport::new();
    cfgs.add_config(ExtractedConfigFile {
        path: ".github/workflows/ci.yml".to_string(),
        git_sha: compute_git_blob_sha(cfg_bytes),
        config_category: ConfigCategory::CiCd,
        format: "yaml".to_string(),
        content: String::from_utf8_lossy(cfg_bytes).to_string(),
        byte_size: cfg_bytes.len(),
        line_count: 1,
        blake3_hash: compute_blake3_hash(cfg_bytes),
        sha256_hash: compute_sha256_hash(cfg_bytes),
    });

    let report = ContentHashValidator::validate_reports(
        Some(&sources),
        Some(&docs),
        Some(&deps),
        Some(&cfgs),
        &HashValidationConfig::strict(),
        "owner/repo",
    );

    assert_eq!(report.total_files_checked, 4);
    assert_eq!(report.total_failures, 0);
    assert!(report.is_valid());
    assert_eq!(report.git_sha_metrics.passed, 4);
    assert_eq!(report.blake3_metrics.passed, 4);
    assert_eq!(report.sha256_metrics.passed, 4);
}

#[test]
fn test_manifest_validation() {
    let bytes = b"console.log('hello');";
    let path = "index.js";
    let git_sha = compute_git_blob_sha(bytes);
    let blake3 = compute_blake3_hash(bytes);
    let sha256 = compute_sha256_hash(bytes);

    let manifest = RepoFileManifest::from_entries(vec![ManifestFileEntry {
        path: path.to_string(),
        git_sha,
        category: web_repo_to_ai_github_extractor::FileCategory::SourceCode,
        size_bytes: bytes.len() as u64,
        line_count: Some(1),
        language: Some("javascript".to_string()),
        blake3_hash: Some(blake3),
        sha256_hash: Some(sha256),
        status: FileExtractionStatus::Extracted,
        cdm_section_id: Some("sec-1".to_string()),
    }]);

    let mut content_map = HashMap::new();
    content_map.insert(path.to_string(), bytes.as_slice());

    let report = ContentHashValidator::validate_manifest(
        &manifest,
        &content_map,
        &HashValidationConfig::strict(),
        "owner/repo",
    );

    assert_eq!(report.total_files_checked, 1);
    assert!(report.is_valid());
}

#[test]
fn test_cdm_sections_hash_validation() {
    let code = "fn answer() -> i32 { 42 }";
    let git_sha = compute_git_blob_sha(code.as_bytes());
    let blake3 = compute_blake3_hash(code.as_bytes());

    let sec = CdmSectionBuilder::new(
        "sec-answer",
        "https://github.com/owner/repo",
        SectionKind::CodeFile,
        code,
    )
    .original_path("src/answer.rs")
    .attribute("git_blob_sha", &git_sha)
    .content_hash(&blake3)
    .token_metrics(SectionTokenMetrics::compute_baseline(code))
    .build();

    let report = ContentHashValidator::validate_cdm_sections(
        &[sec],
        &HashValidationConfig::strict(),
        "owner/repo",
    );

    assert_eq!(report.total_files_checked, 1);
    assert!(report.is_valid());
}

#[test]
fn test_fail_fast_behavior() {
    let cfg = HashValidationConfig::strict().with_fail_fast(true);
    let mut report = HashValidationReport::new("owner/repo");

    // First file: corrupted
    ContentHashValidator::validate_file_bytes(
        "bad.rs",
        b"corrupt",
        Some("wrong_sha_123456789012345678901234567890"),
        None,
        None,
        &cfg,
        &mut report,
    );

    assert_eq!(report.total_failures, 1);
    assert!(!report.is_valid());
}

#[test]
fn test_markdown_json_and_cdm_section_export() {
    let report = HashValidationReport::new("owner/repo");

    let markdown = report.to_markdown();
    assert!(markdown.contains("# Cryptographic Content Hash Validation: owner/repo"));
    assert!(markdown.contains("Checksum Algorithm Breakdown"));

    let json_pretty = report.to_json().expect("to_json failed");
    assert!(json_pretty.contains("\"total_files_checked\": 0"));

    let json_compact = report.to_json_compact().expect("to_json_compact failed");
    assert!(json_compact.contains("\"total_files_checked\":0"));

    let section = report.to_cdm_section("https://github.com/owner/repo");
    assert_eq!(section.section_id, "sec-content-hash-validation");
    assert_eq!(section.kind, SectionKind::Manifest);
    assert_eq!(
        section.attributes.get("manifest_role").map(String::as_str),
        Some("content_hash_validation")
    );
    assert_eq!(
        section.attributes.get("is_valid").map(String::as_str),
        Some("true")
    );
}

#[tokio::test]
async fn test_client_validate_content_hashes() {
    let app = Router::new().route(
        "/rate_limit",
        get(|| async {
            Json(json!({
                "resources": {
                    "core": { "limit": 5000, "remaining": 4999, "reset": 1893456000, "used": 1 }
                }
            }))
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve axum");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(format!("http://{addr}"))
        .with_timeout(Duration::from_millis(500));

    let content = b"fn hello() {}";
    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/hello.rs".to_string(),
        git_sha: compute_git_blob_sha(content),
        content: String::from_utf8_lossy(content).to_string(),
        byte_size: content.len(),
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: compute_blake3_hash(content),
        sha256_hash: compute_sha256_hash(content),
        is_truncated: false,
    });

    let report = client.validate_content_hashes(
        Some(&sources),
        None,
        None,
        None,
        &HashValidationConfig::strict(),
        "owner/repo",
    );

    assert_eq!(report.total_files_checked, 1);
    assert!(report.is_valid());
}
