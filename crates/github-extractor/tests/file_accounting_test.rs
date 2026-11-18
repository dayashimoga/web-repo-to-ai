//! Integration tests for expected-vs-actual repository file accounting, discrepancy taxonomy,
//! set reconciliation, and balance sheet validation.

use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use std::time::Duration;
use web_repo_to_ai_cdm::{CdmSectionBuilder, SectionKind, SectionTokenMetrics};
use web_repo_to_ai_github_extractor::{
    compute_blake3_hash, AccountingDiscrepancyKind, ConfigTestExtractionReport,
    DependencyExtractionReport, DiscrepancySeverity, DocExtractionReport, ExtractedDocFile,
    ExtractedSourceFile, FileAccountingAuditor, FileCategory, FileExtractionStatus, GitFileMode,
    GitHubApiClient, GitTreeEntry, GitTreeEntryType, GitTreeResponse, ManifestFileEntry,
    RepoFileManifest, SourceExtractionReport,
};

fn sample_tree() -> GitTreeResponse {
    let entries = vec![
        GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(120),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(250),
        ),
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000003",
            Some(80),
        ),
        GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000004",
            Some(5000),
        ),
        GitTreeEntry::new(
            "assets/logo.png",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000005",
            Some(4096),
        ),
    ];
    GitTreeResponse::new("root_sha", entries, false)
}

#[test]
fn test_balanced_and_consistent_accounting() {
    let tree = sample_tree();
    let classification = tree.classify(&Default::default());

    let entries = vec![
        ManifestFileEntry {
            path: "src/main.rs".to_string(),
            git_sha: "1000000000000000000000000000000000000001".to_string(),
            category: FileCategory::SourceCode,
            size_bytes: 120,
            line_count: Some(10),
            language: Some("rust".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-code-001".to_string()),
        },
        ManifestFileEntry {
            path: "README.md".to_string(),
            git_sha: "1000000000000000000000000000000000000002".to_string(),
            category: FileCategory::Documentation,
            size_bytes: 250,
            line_count: Some(15),
            language: Some("markdown".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-doc-001".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.toml".to_string(),
            git_sha: "1000000000000000000000000000000000000003".to_string(),
            category: FileCategory::DependencyManifest,
            size_bytes: 80,
            line_count: Some(8),
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-dep-001".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.lock".to_string(),
            git_sha: "1000000000000000000000000000000000000004".to_string(),
            category: FileCategory::GeneratedOrLockfile,
            size_bytes: 5000,
            line_count: None,
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Excluded {
                reason: "Lockfile excluded".to_string(),
            },
            cdm_section_id: None,
        },
        ManifestFileEntry {
            path: "assets/logo.png".to_string(),
            git_sha: "1000000000000000000000000000000000000005".to_string(),
            category: FileCategory::Binary,
            size_bytes: 4096,
            line_count: None,
            language: None,
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Unsupported {
                reason: "Non-text binary asset".to_string(),
            },
            cdm_section_id: None,
        },
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let sheet = FileAccountingAuditor::audit_manifest(
        &tree,
        Some(&classification),
        &manifest,
        "owner/repo",
        Some("main".to_string()),
        Some("commit_sha_123".to_string()),
    );

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.actual_extracted, 3);
    assert_eq!(sheet.actual_excluded, 1);
    assert_eq!(sheet.actual_unsupported, 1);
    assert_eq!(sheet.actual_failed, 0);
    assert_eq!(sheet.missing_unaccounted, 0);
    assert_eq!(sheet.unexpected_extra, 0);
    assert_eq!(sheet.byte_size_mismatches, 0);
    assert_eq!(sheet.git_sha_mismatches, 0);
    assert!(sheet.discrepancies.is_empty());

    assert!(sheet.is_balanced());
    assert!(sheet.is_fully_accounted());
    assert!(sheet.is_perfectly_consistent());

    let completeness = sheet.to_completeness_report();
    assert_eq!(completeness.discovered_count, 5);
    assert_eq!(completeness.extracted_count, 3);
    assert_eq!(completeness.excluded_count, 1);
    assert_eq!(completeness.unsupported_count, 1);
    assert!(completeness.formula_valid());
}

#[test]
fn test_detect_missing_unaccounted_files() {
    let tree = sample_tree(); // 5 blobs

    // Manifest missing Cargo.toml completely!
    let entries = vec![
        ManifestFileEntry {
            path: "src/main.rs".to_string(),
            git_sha: "1000000000000000000000000000000000000001".to_string(),
            category: FileCategory::SourceCode,
            size_bytes: 120,
            line_count: Some(10),
            language: Some("rust".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-code-001".to_string()),
        },
        ManifestFileEntry {
            path: "README.md".to_string(),
            git_sha: "1000000000000000000000000000000000000002".to_string(),
            category: FileCategory::Documentation,
            size_bytes: 250,
            line_count: Some(15),
            language: Some("markdown".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-doc-001".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.lock".to_string(),
            git_sha: "1000000000000000000000000000000000000004".to_string(),
            category: FileCategory::GeneratedOrLockfile,
            size_bytes: 5000,
            line_count: None,
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Excluded {
                reason: "Lockfile excluded".to_string(),
            },
            cdm_section_id: None,
        },
        ManifestFileEntry {
            path: "assets/logo.png".to_string(),
            git_sha: "1000000000000000000000000000000000000005".to_string(),
            category: FileCategory::Binary,
            size_bytes: 4096,
            line_count: None,
            language: None,
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Unsupported {
                reason: "Non-text binary asset".to_string(),
            },
            cdm_section_id: None,
        },
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let sheet =
        FileAccountingAuditor::audit_manifest(&tree, None, &manifest, "owner/repo", None, None);

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.actual_extracted, 2);
    assert_eq!(sheet.missing_unaccounted, 1);
    assert!(sheet.is_balanced()); // 2 + 1 + 1 + 1 = 5
    assert!(!sheet.is_fully_accounted());
    assert!(!sheet.is_perfectly_consistent());

    let criticals = sheet.critical_discrepancies();
    assert_eq!(criticals.len(), 1);
    assert_eq!(criticals[0].path, "Cargo.toml");
    assert_eq!(
        criticals[0].kind,
        AccountingDiscrepancyKind::MissingUnaccounted
    );
    assert_eq!(criticals[0].severity, DiscrepancySeverity::Critical);
}

#[test]
fn test_detect_unexpected_extra_files() {
    let tree = sample_tree(); // 5 blobs

    // Manifest includes a phantom extra file!
    let mut entries = Vec::new();
    for entry in &tree.tree {
        entries.push(ManifestFileEntry {
            path: entry.path.clone(),
            git_sha: entry.sha.clone(),
            category: FileCategory::SourceCode,
            size_bytes: entry.size.unwrap_or(100),
            line_count: Some(5),
            language: Some("rust".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec".to_string()),
        });
    }

    // Extra phantom file
    entries.push(ManifestFileEntry {
        path: "phantom/synthetic.rs".to_string(),
        git_sha: "9999999999999999999999999999999999999999".to_string(),
        category: FileCategory::SourceCode,
        size_bytes: 42,
        line_count: Some(2),
        language: Some("rust".to_string()),
        blake3_hash: None,
        sha256_hash: None,
        status: FileExtractionStatus::Extracted,
        cdm_section_id: Some("sec-phantom".to_string()),
    });

    let manifest = RepoFileManifest::from_entries(entries);
    let sheet =
        FileAccountingAuditor::audit_manifest(&tree, None, &manifest, "owner/repo", None, None);

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.unexpected_extra, 1);
    assert!(!sheet.is_fully_accounted());
    assert!(!sheet.is_perfectly_consistent());

    let extras: Vec<_> = sheet
        .discrepancies
        .iter()
        .filter(|d| d.kind == AccountingDiscrepancyKind::UnexpectedExtra)
        .collect();
    assert_eq!(extras.len(), 1);
    assert_eq!(extras[0].path, "phantom/synthetic.rs");
}

#[test]
fn test_detect_byte_size_mismatch() {
    let tree = sample_tree();

    // In manifest, src/main.rs has size 60 instead of expected 120
    let entries = vec![
        ManifestFileEntry {
            path: "src/main.rs".to_string(),
            git_sha: "1000000000000000000000000000000000000001".to_string(),
            category: FileCategory::SourceCode,
            size_bytes: 60, // Mismatched! Expected 120
            line_count: Some(5),
            language: Some("rust".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-01".to_string()),
        },
        ManifestFileEntry {
            path: "README.md".to_string(),
            git_sha: "1000000000000000000000000000000000000002".to_string(),
            category: FileCategory::Documentation,
            size_bytes: 250,
            line_count: Some(15),
            language: Some("markdown".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-02".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.toml".to_string(),
            git_sha: "1000000000000000000000000000000000000003".to_string(),
            category: FileCategory::DependencyManifest,
            size_bytes: 80,
            line_count: Some(8),
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-03".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.lock".to_string(),
            git_sha: "1000000000000000000000000000000000000004".to_string(),
            category: FileCategory::GeneratedOrLockfile,
            size_bytes: 5000,
            line_count: None,
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Excluded {
                reason: "Lockfile excluded".to_string(),
            },
            cdm_section_id: None,
        },
        ManifestFileEntry {
            path: "assets/logo.png".to_string(),
            git_sha: "1000000000000000000000000000000000000005".to_string(),
            category: FileCategory::Binary,
            size_bytes: 4096,
            line_count: None,
            language: None,
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Unsupported {
                reason: "Non-text binary asset".to_string(),
            },
            cdm_section_id: None,
        },
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let sheet =
        FileAccountingAuditor::audit_manifest(&tree, None, &manifest, "owner/repo", None, None);

    assert_eq!(sheet.byte_size_mismatches, 1);
    assert!(!sheet.is_perfectly_consistent());

    let mismatch = sheet
        .discrepancies
        .iter()
        .find(|d| d.kind == AccountingDiscrepancyKind::ByteSizeMismatch)
        .expect("byte size mismatch discrepancy not found");
    assert_eq!(mismatch.path, "src/main.rs");
    assert_eq!(mismatch.expected_detail, "120 bytes");
    assert_eq!(mismatch.actual_detail, "60 bytes");
}

#[test]
fn test_detect_git_sha_mismatch() {
    let tree = sample_tree();

    let entries = vec![
        ManifestFileEntry {
            path: "src/main.rs".to_string(),
            git_sha: "9999000000000000000000000000000000000001".to_string(), // Mismatched SHA!
            category: FileCategory::SourceCode,
            size_bytes: 120,
            line_count: Some(10),
            language: Some("rust".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-01".to_string()),
        },
        ManifestFileEntry {
            path: "README.md".to_string(),
            git_sha: "1000000000000000000000000000000000000002".to_string(),
            category: FileCategory::Documentation,
            size_bytes: 250,
            line_count: Some(15),
            language: Some("markdown".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-02".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.toml".to_string(),
            git_sha: "1000000000000000000000000000000000000003".to_string(),
            category: FileCategory::DependencyManifest,
            size_bytes: 80,
            line_count: Some(8),
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Extracted,
            cdm_section_id: Some("sec-03".to_string()),
        },
        ManifestFileEntry {
            path: "Cargo.lock".to_string(),
            git_sha: "1000000000000000000000000000000000000004".to_string(),
            category: FileCategory::GeneratedOrLockfile,
            size_bytes: 5000,
            line_count: None,
            language: Some("toml".to_string()),
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Excluded {
                reason: "Lockfile excluded".to_string(),
            },
            cdm_section_id: None,
        },
        ManifestFileEntry {
            path: "assets/logo.png".to_string(),
            git_sha: "1000000000000000000000000000000000000005".to_string(),
            category: FileCategory::Binary,
            size_bytes: 4096,
            line_count: None,
            language: None,
            blake3_hash: None,
            sha256_hash: None,
            status: FileExtractionStatus::Unsupported {
                reason: "Non-text binary asset".to_string(),
            },
            cdm_section_id: None,
        },
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let sheet =
        FileAccountingAuditor::audit_manifest(&tree, None, &manifest, "owner/repo", None, None);

    assert_eq!(sheet.git_sha_mismatches, 1);
    assert!(!sheet.is_perfectly_consistent());

    let sha_mismatch = sheet
        .discrepancies
        .iter()
        .find(|d| d.kind == AccountingDiscrepancyKind::GitShaMismatch)
        .expect("git sha mismatch discrepancy not found");
    assert_eq!(sha_mismatch.path, "src/main.rs");
}

#[test]
fn test_audit_reports_integration() {
    let tree = sample_tree();
    let classification = tree.classify(&Default::default());

    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/main.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000001".to_string(),
        content: "fn main() {}".to_string(),
        byte_size: 120,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "blake_main".to_string(),
        sha256_hash: "sha_main".to_string(),
        is_truncated: false,
    });

    let mut docs = DocExtractionReport::new();
    docs.add_extracted(ExtractedDocFile {
        path: "README.md".to_string(),
        git_sha: "1000000000000000000000000000000000000002".to_string(),
        content: "# Test Repo".to_string(),
        byte_size: 250,
        line_count: 1,
        doc_type: web_repo_to_ai_github_extractor::DocType::Readme,
        format: web_repo_to_ai_github_extractor::DocFormat::Markdown,
        title: "Test Repo".to_string(),
        has_frontmatter: false,
        blake3_hash: "hash_readme".to_string(),
        sha256_hash: "sha_readme".to_string(),
    });

    let mut deps = DependencyExtractionReport::new();
    deps.add_extracted(web_repo_to_ai_github_extractor::ExtractedDependencyFile {
        path: "Cargo.toml".to_string(),
        git_sha: "1000000000000000000000000000000000000003".to_string(),
        content: "[package]\nname = \"test\"".to_string(),
        byte_size: 80,
        line_count: 2,
        ecosystem: web_repo_to_ai_github_extractor::PackageEcosystem::Cargo,
        role: web_repo_to_ai_github_extractor::ManifestRole::Manifest,
        parsed: None,
        blake3_hash: "hash_cargo".to_string(),
        sha256_hash: "sha_cargo".to_string(),
    });

    let configs = ConfigTestExtractionReport::new();

    let sheet = FileAccountingAuditor::audit_reports(
        &tree,
        Some(&classification),
        Some(&sources),
        Some(&docs),
        Some(&deps),
        Some(&configs),
        "owner/repo",
        Some("main".to_string()),
        Some("sha_xyz".to_string()),
    );

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.actual_extracted, 3);
    assert_eq!(sheet.actual_excluded, 1); // Cargo.lock excluded by classification
    assert_eq!(sheet.actual_unsupported, 1); // logo.png binary by classification
    assert_eq!(sheet.missing_unaccounted, 0);
    assert!(sheet.is_balanced());
    assert!(sheet.is_fully_accounted());
}

#[test]
fn test_cdm_section_and_markdown_formatting() {
    let tree = sample_tree();
    let classification = tree.classify(&Default::default());
    let manifest = RepoFileManifest::from_entries(vec![]);

    let sheet = FileAccountingAuditor::audit_manifest(
        &tree,
        Some(&classification),
        &manifest,
        "owner/test-repo",
        Some("main".to_string()),
        Some("sha_123".to_string()),
    );

    let markdown = sheet.to_markdown();
    assert!(markdown.contains("# File Accounting & Completeness Audit: owner/test-repo"));
    assert!(markdown.contains("Accounting Balance Sheet"));
    assert!(markdown.contains("Discrepancy Log"));

    let json_pretty = sheet.to_json().expect("to_json failed");
    assert!(json_pretty.contains("\"discovered_blobs\": 5"));

    let json_compact = sheet.to_json_compact().expect("to_json_compact failed");
    assert!(json_compact.contains("\"discovered_blobs\":5"));

    let section = sheet.to_cdm_section("https://github.com/owner/test-repo");
    assert_eq!(section.section_id, "sec-file-accounting-audit");
    assert_eq!(section.kind, SectionKind::Manifest);
    assert_eq!(
        section.attributes.get("manifest_role").map(String::as_str),
        Some("file_accounting_audit")
    );
    assert_eq!(
        section
            .attributes
            .get("discovered_blobs")
            .map(String::as_str),
        Some("5")
    );
    assert!(!section.content_hash.is_empty());
}

#[test]
fn test_audit_sections_reconciliation() {
    let tree = sample_tree();

    let sec1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/owner/repo",
        SectionKind::CodeFile,
        "fn main() {}",
    )
    .original_path("src/main.rs")
    .attribute("git_blob_sha", "1000000000000000000000000000000000000001")
    .content_hash(compute_blake3_hash(b"fn main() {}"))
    .token_metrics(SectionTokenMetrics::compute_baseline("fn main() {}"))
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/owner/repo",
        SectionKind::Prose,
        "# Test",
    )
    .original_path("README.md")
    .attribute("git_blob_sha", "1000000000000000000000000000000000000002")
    .content_hash(compute_blake3_hash(b"# Test"))
    .token_metrics(SectionTokenMetrics::compute_baseline("# Test"))
    .build();

    let sections = vec![sec1, sec2];
    let sheet =
        FileAccountingAuditor::audit_sections(&tree, None, &sections, "owner/repo", None, None);

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.actual_extracted, 2);
    assert_eq!(sheet.missing_unaccounted, 3);
}

#[tokio::test]
async fn test_client_audit_file_accounting() {
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

    let tree = sample_tree();
    let classification = tree.classify(&Default::default());
    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/main.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000001".to_string(),
        content: "code".to_string(),
        byte_size: 120,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "hash".to_string(),
        sha256_hash: "sha".to_string(),
        is_truncated: false,
    });

    let sheet = client.audit_file_accounting(
        &tree,
        Some(&classification),
        None,
        Some(&sources),
        None,
        None,
        None,
        "owner/repo",
        Some("main".to_string()),
        Some("sha123".to_string()),
    );

    assert_eq!(sheet.discovered_blobs, 5);
    assert_eq!(sheet.actual_extracted, 1);
    assert!(sheet.is_balanced());
}
