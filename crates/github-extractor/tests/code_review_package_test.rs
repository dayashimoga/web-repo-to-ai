//! Integration tests for automated AI code-review package assembly, review prioritization tiering,
//! review checklist synthesis, and end-to-end CDM document construction.

use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use std::time::Duration;
use web_repo_to_ai_cdm::{CdmValidator, SectionKind, SourceType, ValidationConfig};
use web_repo_to_ai_github_extractor::{
    CodeReviewPackage, CodeReviewPackageConfig, ConfigCategory, ConfigTestExtractionReport,
    ExtractedConfigFile, ExtractedSourceFile, GitFileMode, GitHubApiClient, GitTreeEntry,
    GitTreeEntryType, GitTreeResponse, RepoArchitectureContext, ReviewFocus, ReviewPriorityTier,
    SourceExtractionReport,
};

fn sample_classified_repo() -> (
    GitTreeResponse,
    web_repo_to_ai_github_extractor::TreeClassificationReport,
    SourceExtractionReport,
    ConfigTestExtractionReport,
) {
    let entries = vec![
        GitTreeEntry::new(
            "crates/url-security/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(3000),
        ),
        GitTreeEntry::new(
            "services/api/src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(2500),
        ),
        GitTreeEntry::new(
            "crates/core/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000003",
            Some(4000),
        ),
        GitTreeEntry::new(
            "apps/web/src/routes/+page.svelte",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000004",
            Some(1800),
        ),
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000005",
            Some(500),
        ),
        GitTreeEntry::new(
            ".github/workflows/ci.yml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000006",
            Some(1200),
        ),
        GitTreeEntry::new(
            "tests/integration_test.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000007",
            Some(1500),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000008",
            Some(2000),
        ),
        GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000009",
            Some(50000),
        ),
    ];

    let tree_response = GitTreeResponse::new("root_sha", entries, false);
    let classification = tree_response.classify(&Default::default());

    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "crates/url-security/src/lib.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000001".to_string(),
        content: "pub fn validate_url() -> bool { true }".to_string(),
        byte_size: 38,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "hash_sec".to_string(),
        sha256_hash: "sha_sec".to_string(),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "services/api/src/main.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000002".to_string(),
        content: "fn main() { println!(\"api\"); }".to_string(),
        byte_size: 30,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "hash_api".to_string(),
        sha256_hash: "sha_api".to_string(),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "crates/core/src/lib.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000003".to_string(),
        content: "pub struct Engine;".to_string(),
        byte_size: 18,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "hash_core".to_string(),
        sha256_hash: "sha_core".to_string(),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "apps/web/src/routes/+page.svelte".to_string(),
        git_sha: "1000000000000000000000000000000000000004".to_string(),
        content: "<h1>Converter</h1>".to_string(),
        byte_size: 18,
        line_count: 1,
        language: Some("svelte".to_string()),
        blake3_hash: "hash_ui".to_string(),
        sha256_hash: "sha_ui".to_string(),
        is_truncated: false,
    });

    let mut configs = ConfigTestExtractionReport::new();
    configs.add_config(ExtractedConfigFile {
        path: ".github/workflows/ci.yml".to_string(),
        git_sha: "1000000000000000000000000000000000000006".to_string(),
        config_category: ConfigCategory::CiCd,
        format: "yaml".to_string(),
        content: "name: CI".to_string(),
        byte_size: 8,
        line_count: 1,
        blake3_hash: "hash_ci".to_string(),
        sha256_hash: "sha_ci".to_string(),
    });

    (tree_response, classification, sources, configs)
}

#[test]
fn test_review_focus_and_config_defaults() {
    let focus = ReviewFocus::default();
    assert_eq!(focus, ReviewFocus::FullRepository);
    assert_eq!(focus.as_str(), "full_repository");
    assert_eq!(focus.display_title(), "Full Repository Code Review");

    let cfg = CodeReviewPackageConfig::new();
    assert_eq!(cfg.focus, ReviewFocus::FullRepository);
    assert_eq!(cfg.max_files, 50);
    assert_eq!(cfg.max_total_lines, 10_000);
    assert!(cfg.include_tree);
    assert!(cfg.include_manifest);
    assert!(cfg.include_architecture);
    assert!(cfg.include_checklist);

    let sec_focus = ReviewFocus::SecurityAudit;
    assert_eq!(sec_focus.as_str(), "security_audit");
    assert_eq!(
        sec_focus.display_title(),
        "Application Security & Defense Audit"
    );
}

#[test]
fn test_review_priority_tier_assignment() {
    let (_tree, classification, sources, configs) = sample_classified_repo();
    let config = CodeReviewPackageConfig::default();

    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        None,
        None,
        None,
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        Some("main".to_string()),
        Some("1234567890abcdef".to_string()),
    );

    // Verify ordering by priority tier
    assert!(!package.files.is_empty());
    let url_sec = package
        .files
        .iter()
        .find(|f| f.path == "crates/url-security/src/lib.rs")
        .expect("url-security file");
    assert_eq!(url_sec.priority_tier, ReviewPriorityTier::Critical);

    let api_main = package
        .files
        .iter()
        .find(|f| f.path == "services/api/src/main.rs")
        .expect("api main file");
    assert_eq!(api_main.priority_tier, ReviewPriorityTier::Critical);

    let core_lib = package
        .files
        .iter()
        .find(|f| f.path == "crates/core/src/lib.rs")
        .expect("core lib file");
    assert_eq!(core_lib.priority_tier, ReviewPriorityTier::High);

    let ui_route = package
        .files
        .iter()
        .find(|f| f.path == "apps/web/src/routes/+page.svelte")
        .expect("ui file");
    assert_eq!(ui_route.priority_tier, ReviewPriorityTier::High);

    // Excluded files (Cargo.lock) must not be in reviewed files list
    assert!(package.files.iter().all(|f| f.path != "Cargo.lock"));
}

#[test]
fn test_security_audit_focus_prioritization() {
    let (_tree, classification, sources, configs) = sample_classified_repo();
    let config = CodeReviewPackageConfig::default().with_focus(ReviewFocus::SecurityAudit);

    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        None,
        None,
        None,
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        None,
        None,
    );

    assert_eq!(package.focus, ReviewFocus::SecurityAudit);

    // Checklist contains SSRF, Secret Redaction, Path Traversal
    let checklist_cats: Vec<&str> = package
        .checklist
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    assert!(checklist_cats.contains(&"SSRF & Network Defense"));
    assert!(checklist_cats.contains(&"Secret & Credential Redaction"));
    assert!(checklist_cats.contains(&"Path Traversal & Injection Defense"));
    assert!(checklist_cats.contains(&"Resource Bounds & DoS Prevention"));
}

#[test]
fn test_code_review_package_assembly_with_budgets() {
    let (_tree, classification, sources, configs) = sample_classified_repo();
    let config = CodeReviewPackageConfig::default().with_max_files(2);

    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        None,
        None,
        None,
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        None,
        None,
    );

    assert_eq!(package.reviewed_file_count, 2);
    assert_eq!(package.files.len(), 2);
    assert_eq!(package.total_repo_files, 9);
    // Should select top 2 critical files
    assert_eq!(package.files[0].priority_tier, ReviewPriorityTier::Critical);
    assert_eq!(package.files[1].priority_tier, ReviewPriorityTier::Critical);
}

#[test]
fn test_code_review_package_markdown_formatting() {
    let (tree_resp, classification, sources, configs) = sample_classified_repo();
    let tree = tree_resp.to_repo_tree();
    let manifest = classification.to_file_manifest();
    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        Some(&manifest),
        None,
        None,
        Some(&configs),
        None,
    );

    let config = CodeReviewPackageConfig::default();
    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        Some(&tree),
        Some(&manifest),
        Some(&arch),
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        Some("main".to_string()),
        Some("1234567890abcdef1234567890abcdef12345678".to_string()),
    );

    let md = package.to_markdown();

    assert!(md.contains("# Code Review Package: Full Repository Code Review"));
    assert!(md.contains("## Review Package Metadata"));
    assert!(md.contains("| Review Focus | `full_repository` |"));
    assert!(md.contains("## Reviewer Instructions & Verification Checklist"));
    assert!(md.contains("## Prioritized Review Files Index"));
    assert!(md.contains("| `critical` | `crates/url-security/src/lib.rs` |"));
    assert!(md.contains("# Prioritized Code Review Files"));
    assert!(md.contains("## File: `crates/url-security/src/lib.rs` [CRITICAL]"));
    assert!(md.contains("pub fn validate_url() -> bool { true }"));
}

#[test]
fn test_code_review_package_json_serialization() {
    let (tree_resp, classification, sources, configs) = sample_classified_repo();
    let tree = tree_resp.to_repo_tree();
    let config = CodeReviewPackageConfig::default();

    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        Some(&tree),
        None,
        None,
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        None,
        None,
    );

    let pretty_json = package.to_json().expect("pretty json serialized");
    assert!(pretty_json.contains("\"focus\": \"full_repository\""));

    let compact = package.to_json_compact().expect("compact json serialized");
    assert!(compact.contains("\"focus\":\"full_repository\""));

    let deserialized: CodeReviewPackage =
        serde_json::from_str(&pretty_json).expect("deserialized successfully");
    assert_eq!(deserialized.focus, package.focus);
    assert_eq!(
        deserialized.reviewed_file_count,
        package.reviewed_file_count
    );
    assert_eq!(deserialized.files.len(), package.files.len());
}

#[test]
fn test_code_review_package_to_cdm_document() {
    let (tree_resp, classification, sources, configs) = sample_classified_repo();
    let tree = tree_resp.to_repo_tree();
    let manifest = classification.to_file_manifest();
    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        Some(&manifest),
        None,
        None,
        Some(&configs),
        None,
    );

    let config = CodeReviewPackageConfig::default();
    let package = CodeReviewPackage::assemble(
        &config,
        &classification,
        Some(&tree),
        Some(&manifest),
        Some(&arch),
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        Some("main".to_string()),
        Some("commit_sha_123".to_string()),
    );

    let doc = package.to_cdm_document("https://github.com/test-owner/test-repo");

    assert_eq!(doc.source_type, SourceType::GitHubPublic);
    assert_eq!(doc.source_identity.raw_input, "repository");
    assert_eq!(
        doc.source_identity.canonical_url,
        "https://github.com/test-owner/test-repo"
    );
    assert_eq!(doc.source_identity.git_ref.as_deref(), Some("main"));

    // Check sections
    assert!(!doc.sections.is_empty());
    let kinds: Vec<SectionKind> = doc.sections.iter().map(|s| s.kind).collect();
    assert!(kinds.contains(&SectionKind::Frontmatter));
    assert!(kinds.contains(&SectionKind::Manifest)); // tree & manifest
    assert!(kinds.contains(&SectionKind::Prose)); // architecture
    assert!(kinds.contains(&SectionKind::CodeFile)); // code files
    assert!(kinds.contains(&SectionKind::Admonition)); // review checklist

    // Run official CDM schema validator
    let validator = CdmValidator::with_config(ValidationConfig::strict());
    let report = validator.validate(&doc);
    assert!(
        report.is_valid(),
        "CDM Document validation failed: {:?}",
        report.errors().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_client_generate_code_review_package() {
    let app = Router::new().route(
        "/rate_limit",
        get(|| async {
            Json(json!({
                "resources": {
                    "core": { "limit": 5000, "remaining": 4999, "reset": 1700000000, "used": 1 }
                }
            }))
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bound to ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server serves");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(format!("http://{addr}"))
        .with_timeout(Duration::from_millis(500));

    let (tree_resp, classification, sources, configs) = sample_classified_repo();
    let config = CodeReviewPackageConfig::default();

    let package = client.generate_code_review_package(
        &config,
        &classification,
        Some(&tree_resp.to_repo_tree()),
        None,
        None,
        Some(&sources),
        None,
        None,
        Some(&configs),
        None,
        Some("main".to_string()),
        Some("abc1234".to_string()),
    );

    assert_eq!(package.focus, ReviewFocus::FullRepository);
    assert!(!package.files.is_empty());
    assert_eq!(package.pinned_sha.as_deref(), Some("abc1234"));
}
