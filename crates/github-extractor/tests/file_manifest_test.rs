//! Integration tests for repository file manifest indexing, completeness accounting,
//! GFM Markdown formatting, JSON serialization, and CDM manifest section construction.

use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::json;
use std::time::Duration;
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_github_extractor::{
    ConfigCategory, ConfigTestExtractionReport, DependencyExtractionReport, DocExtractionReport,
    DocFormat, DocType, ExtractedConfigFile, ExtractedDependencyFile, ExtractedDocFile,
    ExtractedSourceFile, ExtractedTestFile, FileCategory, FileExtractionStatus, GitFileMode,
    GitHubApiClient, GitTreeEntry, GitTreeEntryType, GitTreeResponse, ManifestFileEntry,
    ManifestRole, PackageEcosystem, RepoFileManifest, SourceExtractionReport, TestCategory,
};

#[test]
fn test_manifest_from_explicit_entries() {
    let entries = vec![
        ManifestFileEntry::new(
            "src/main.rs",
            "1000000000000000000000000000000000000001",
            FileCategory::SourceCode,
            1200,
            FileExtractionStatus::Extracted,
        )
        .with_line_count(45)
        .with_language("rust")
        .with_blake3_hash("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .with_sha256_hash("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        ManifestFileEntry::new(
            "src/lib.rs",
            "1000000000000000000000000000000000000002",
            FileCategory::SourceCode,
            800,
            FileExtractionStatus::Extracted,
        )
        .with_line_count(30)
        .with_language("rust")
        .with_blake3_hash("1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ManifestFileEntry::new(
            "Cargo.lock",
            "1000000000000000000000000000000000000003",
            FileCategory::GeneratedOrLockfile,
            45000,
            FileExtractionStatus::Excluded {
                reason: "Lockfiles excluded by default policy".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "corrupted.rs",
            "1000000000000000000000000000000000000004",
            FileCategory::SourceCode,
            500,
            FileExtractionStatus::Failed {
                error: "Git blob SHA-1 mismatch".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "assets/logo.png",
            "1000000000000000000000000000000000000005",
            FileCategory::Binary,
            15000,
            FileExtractionStatus::Unsupported {
                reason: "Binary PNG image format".to_string(),
            },
        ),
    ];

    let manifest = RepoFileManifest::from_entries(entries);

    assert_eq!(manifest.total_files, 5);
    assert_eq!(manifest.extracted_files, 2);
    assert_eq!(manifest.excluded_files, 1);
    assert_eq!(manifest.failed_files, 1);
    assert_eq!(manifest.unsupported_files, 1);

    assert_eq!(manifest.total_bytes, 1200 + 800 + 45000 + 500 + 15000);
    assert_eq!(manifest.extracted_bytes, 1200 + 800);
    assert_eq!(manifest.total_lines, 45 + 30);

    // Fundamental conservation equation
    assert!(manifest.verify_completeness());
    assert!(!manifest.is_complete()); // has failed and unsupported files

    // Query iterators
    assert_eq!(manifest.extracted_entries().count(), 2);
    assert_eq!(manifest.excluded_entries().count(), 1);
    assert_eq!(manifest.failed_entries().count(), 1);
    assert_eq!(manifest.unsupported_entries().count(), 1);

    let found = manifest.find_by_path("src/main.rs").expect("file found");
    assert!(found.is_extracted());
    assert_eq!(found.line_count, Some(45));
    assert_eq!(found.language.as_deref(), Some("rust"));

    let excluded = manifest.find_by_path("Cargo.lock").expect("file found");
    assert!(excluded.status.is_excluded());
    assert_eq!(
        excluded.status.reason_or_error(),
        Some("Lockfiles excluded by default policy")
    );
}

#[test]
fn test_manifest_from_classification_and_multi_reports() {
    let tree_entries = vec![
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(350),
        ),
        GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(45000),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000003",
            Some(2100),
        ),
        GitTreeEntry::new(
            ".github/workflows/ci.yml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000004",
            Some(1200),
        ),
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000005",
            Some(5600),
        ),
        GitTreeEntry::new(
            "src/bad.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000006",
            Some(500),
        ),
        GitTreeEntry::new(
            "tests/lib_test.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000007",
            Some(3400),
        ),
        GitTreeEntry::new(
            "assets/logo.png",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000008",
            Some(8900),
        ),
    ];

    let tree_response = GitTreeResponse::new("sha-root", tree_entries, false);
    let classification = tree_response.classify(&Default::default());

    // 1. Source report: src/lib.rs extracted, src/bad.rs failed
    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/lib.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000005".to_string(),
        content: "pub fn hello() {}".to_string(),
        byte_size: 17,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "b3hash1".to_string(),
        sha256_hash: "sha256hash1".to_string(),
        is_truncated: false,
    });
    sources.add_failed(
        "src/bad.rs",
        "1000000000000000000000000000000000000006",
        "Corrupted blob content",
    );

    // 2. Doc report: README.md extracted
    let mut docs = DocExtractionReport::new();
    docs.add_extracted(ExtractedDocFile {
        path: "README.md".to_string(),
        git_sha: "1000000000000000000000000000000000000003".to_string(),
        doc_type: DocType::Readme,
        format: DocFormat::Markdown,
        title: "Test Repo".to_string(),
        has_frontmatter: false,
        content: "# Test Repo".to_string(),
        byte_size: 11,
        line_count: 1,
        blake3_hash: "b3hash2".to_string(),
        sha256_hash: "sha256hash2".to_string(),
    });

    // 3. Dependency report: Cargo.toml extracted
    let mut dependencies = DependencyExtractionReport::default();
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "Cargo.toml".to_string(),
        git_sha: "1000000000000000000000000000000000000001".to_string(),
        ecosystem: PackageEcosystem::Cargo,
        role: ManifestRole::Manifest,
        content: "[package]\nname = \"test\"".to_string(),
        parsed: None,
        byte_size: 24,
        line_count: 2,
        blake3_hash: "b3hash3".to_string(),
        sha256_hash: "sha256hash3".to_string(),
    });

    // 4. Config & Test report: ci.yml and lib_test.rs extracted
    let mut configs_tests = ConfigTestExtractionReport::new();
    configs_tests.add_config(ExtractedConfigFile {
        path: ".github/workflows/ci.yml".to_string(),
        git_sha: "1000000000000000000000000000000000000004".to_string(),
        config_category: ConfigCategory::CiCd,
        format: "yaml".to_string(),
        content: "name: CI".to_string(),
        byte_size: 8,
        line_count: 1,
        blake3_hash: "b3hash4".to_string(),
        sha256_hash: "sha256hash4".to_string(),
    });
    configs_tests.add_test(ExtractedTestFile {
        path: "tests/lib_test.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000007".to_string(),
        test_category: TestCategory::Integration,
        language: Some("rust".to_string()),
        content: "#[test]\nfn test_it() {}".to_string(),
        byte_size: 24,
        line_count: 2,
        blake3_hash: "b3hash5".to_string(),
        sha256_hash: "sha256hash5".to_string(),
    });

    // Assemble unified manifest
    let manifest = RepoFileManifest::from_classification_and_reports(
        &classification,
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
    );

    assert_eq!(manifest.total_files, 8);
    // Extracted: src/lib.rs, README.md, Cargo.toml, .github/workflows/ci.yml, tests/lib_test.rs = 5
    assert_eq!(manifest.extracted_files, 5);
    // Excluded: Cargo.lock = 1
    assert_eq!(manifest.excluded_files, 1);
    // Failed: src/bad.rs = 1
    assert_eq!(manifest.failed_files, 1);
    // Unsupported: assets/logo.png = 1
    assert_eq!(manifest.unsupported_files, 1);

    // Strict completeness formula validation
    assert!(manifest.verify_completeness());

    let lock = manifest
        .find_by_path("Cargo.lock")
        .expect("Cargo.lock found");
    assert!(lock.status.is_excluded());

    let logo = manifest
        .find_by_path("assets/logo.png")
        .expect("logo found");
    assert!(logo.status.is_unsupported());

    let bad = manifest.find_by_path("src/bad.rs").expect("bad.rs found");
    assert!(bad.status.is_failed());
    assert_eq!(bad.status.reason_or_error(), Some("Corrupted blob content"));

    let lib = manifest.find_by_path("src/lib.rs").expect("lib.rs found");
    assert!(lib.status.is_extracted());
    assert_eq!(lib.blake3_hash.as_deref(), Some("b3hash1"));
}

#[test]
fn test_manifest_markdown_table_formatting() {
    let entries = vec![
        ManifestFileEntry::new(
            "Cargo.toml",
            "1111111111111111111111111111111111111111",
            FileCategory::DependencyManifest,
            350,
            FileExtractionStatus::Extracted,
        )
        .with_line_count(15)
        .with_language("toml")
        .with_blake3_hash("0123456789abcdef"),
        ManifestFileEntry::new(
            "src/lib.rs",
            "2222222222222222222222222222222222222222",
            FileCategory::SourceCode,
            1200,
            FileExtractionStatus::Extracted,
        )
        .with_line_count(40)
        .with_language("rust")
        .with_blake3_hash("abcdef0123456789"),
        ManifestFileEntry::new(
            "Cargo.lock",
            "3333333333333333333333333333333333333333",
            FileCategory::GeneratedOrLockfile,
            45000,
            FileExtractionStatus::Excluded {
                reason: "Lockfiles excluded by default".to_string(),
            },
        ),
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let md = manifest.to_markdown_table();

    assert!(md.contains("# Repository File Manifest & Completeness Index"));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("| Total Discovered Files | 3 |"));
    assert!(md.contains("| Successfully Extracted | 2 |"));
    assert!(md.contains("| Intentionally Excluded | 1 |"));
    assert!(md.contains("| Completeness Invariant | Valid (`total == extracted + excluded + failed + unsupported`) |"));
    assert!(md.contains("## Category Breakdown"));
    assert!(md.contains("| `source_code` | 1 |"));
    assert!(md.contains("## File Index"));
    assert!(md.contains("| `src/lib.rs` | source_code | rust |"));
    assert!(md.contains("| `Cargo.lock` | generated_or_lockfile | - |"));
}

#[test]
fn test_manifest_json_serialization_roundtrip() {
    let entries = vec![
        ManifestFileEntry::new(
            "README.md",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            FileCategory::Documentation,
            1500,
            FileExtractionStatus::Extracted,
        )
        .with_line_count(50)
        .with_language("markdown")
        .with_blake3_hash("hash123")
        .with_sha256_hash("sha456"),
        ManifestFileEntry::new(
            "docs/spec.pdf",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            FileCategory::Binary,
            120000,
            FileExtractionStatus::Unsupported {
                reason: "Binary PDF document".to_string(),
            },
        ),
    ];

    let manifest = RepoFileManifest::from_entries(entries);

    let pretty_json = manifest.to_json().expect("pretty json serialized");
    assert!(pretty_json.contains("\"total_files\": 2"));

    let compact_json = manifest.to_json_compact().expect("compact json serialized");
    assert!(compact_json.contains("\"total_files\":2"));

    let deserialized: RepoFileManifest =
        serde_json::from_str(&pretty_json).expect("deserialized successfully");
    assert_eq!(deserialized.total_files, manifest.total_files);
    assert_eq!(deserialized.extracted_files, manifest.extracted_files);
    assert_eq!(deserialized.unsupported_files, manifest.unsupported_files);
    assert_eq!(deserialized.files.len(), 2);
    assert_eq!(deserialized.files[0].path, "docs/spec.pdf"); // alphabetical sort
    assert_eq!(deserialized.files[1].path, "README.md");
}

#[test]
fn test_manifest_completeness_report_mapping() {
    let entries = vec![
        ManifestFileEntry::new(
            "file1.rs",
            "1111111111111111111111111111111111111111",
            FileCategory::SourceCode,
            100,
            FileExtractionStatus::Extracted,
        ),
        ManifestFileEntry::new(
            "file2.rs",
            "2222222222222222222222222222222222222222",
            FileCategory::SourceCode,
            200,
            FileExtractionStatus::Extracted,
        ),
        ManifestFileEntry::new(
            "file3.lock",
            "3333333333333333333333333333333333333333",
            FileCategory::GeneratedOrLockfile,
            300,
            FileExtractionStatus::Excluded {
                reason: "policy".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "file4.rs",
            "4444444444444444444444444444444444444444",
            FileCategory::SourceCode,
            400,
            FileExtractionStatus::Failed {
                error: "timeout".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "file5.bin",
            "5555555555555555555555555555555555555555",
            FileCategory::Binary,
            500,
            FileExtractionStatus::Unsupported {
                reason: "binary".to_string(),
            },
        ),
    ];

    let manifest = RepoFileManifest::from_entries(entries);
    let report = manifest.to_completeness_report();

    assert_eq!(report.discovered_count, 5);
    assert_eq!(report.eligible_count, 4); // extracted + failed + unsupported
    assert_eq!(report.extracted_count, 2);
    assert_eq!(report.excluded_count, 1);
    assert_eq!(report.failed_count, 1);
    assert_eq!(report.unsupported_count, 1);
    assert!(report.formula_valid());
    assert!(!report.is_complete());

    // When everything eligible succeeded and 0 failures:
    let perfect_entries = vec![
        ManifestFileEntry::new(
            "a.rs",
            "1111111111111111111111111111111111111111",
            FileCategory::SourceCode,
            100,
            FileExtractionStatus::Extracted,
        ),
        ManifestFileEntry::new(
            "b.lock",
            "2222222222222222222222222222222222222222",
            FileCategory::GeneratedOrLockfile,
            200,
            FileExtractionStatus::Excluded {
                reason: "lockfile".to_string(),
            },
        ),
    ];
    let perfect_manifest = RepoFileManifest::from_entries(perfect_entries);
    assert!(perfect_manifest.is_complete());
    assert!(perfect_manifest.to_completeness_report().is_complete());
}

#[test]
fn test_manifest_to_cdm_section() {
    let entries = vec![ManifestFileEntry::new(
        "src/main.rs",
        "1111111111111111111111111111111111111111",
        FileCategory::SourceCode,
        1000,
        FileExtractionStatus::Extracted,
    )
    .with_line_count(50)
    .with_language("rust")];

    let manifest = RepoFileManifest::from_entries(entries);
    let section = manifest.to_cdm_section(
        "sec-manifest-001",
        "https://github.com/test-owner/test-repo",
    );

    assert_eq!(section.section_id, "sec-manifest-001");
    assert_eq!(
        section.source_url,
        "https://github.com/test-owner/test-repo"
    );
    assert_eq!(section.kind, SectionKind::Manifest);
    assert_eq!(
        section.title.as_deref(),
        Some("Repository File Manifest & Completeness Index")
    );
    assert_eq!(section.original_path.as_deref(), Some("MANIFEST.md"));
    assert_eq!(section.language.as_deref(), Some("markdown"));
    assert_eq!(
        section.attributes.get("manifest_role").map(String::as_str),
        Some("file_manifest")
    );
    assert_eq!(
        section.attributes.get("total_files").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        section
            .attributes
            .get("extracted_files")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        section
            .attributes
            .get("completeness_valid")
            .map(String::as_str),
        Some("true")
    );
    assert!(!section.body.is_empty());
}

#[tokio::test]
async fn test_client_generate_file_manifest() {
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

    let entries = vec![
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(300),
        ),
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(1200),
        ),
    ];
    let tree_response = GitTreeResponse::new("sha-root", entries, false);
    let classification = tree_response.classify(&Default::default());

    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "src/lib.rs".to_string(),
        git_sha: "1000000000000000000000000000000000000002".to_string(),
        content: "pub fn test() {}".to_string(),
        byte_size: 16,
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: "hash_lib".to_string(),
        sha256_hash: "sha_lib".to_string(),
        is_truncated: false,
    });

    let manifest = client.generate_file_manifest(&classification, Some(&sources), None, None, None);

    assert_eq!(manifest.total_files, 2);
    assert_eq!(manifest.extracted_files, 1);
    assert!(manifest.verify_completeness());

    let lib = manifest.find_by_path("src/lib.rs").expect("lib found");
    assert!(lib.is_extracted());
}
