//! Comprehensive Integration Tests for Missing and Excluded File Diagnostics and Reports (Subphase 5.5.3)

use web_repo_to_ai_cdm::ExclusionReason;
use web_repo_to_ai_github_extractor::{
    AccountingDiscrepancy, AccountingDiscrepancyKind, DiscrepancySeverity, ExcludedFileRecord,
    ExclusionAuditReport, ExclusionCategory, ExclusionReportGenerator, ExtractionFailureCategory,
    FailedFileRecord, FileAccountingBalanceSheet, FileCategory, FileExtractionStatus,
    GitHubApiClient, ManifestFileEntry, MissingFileRecord, RepoFileManifest,
};

#[test]
fn test_exclusion_category_inference() {
    // 1. Sensitive security patterns
    assert_eq!(
        ExclusionCategory::infer(".env", None),
        ExclusionCategory::SensitiveSecurityPattern
    );
    assert_eq!(
        ExclusionCategory::infer("config/.env.production", None),
        ExclusionCategory::SensitiveSecurityPattern
    );
    assert_eq!(
        ExclusionCategory::infer("certs/server.pem", None),
        ExclusionCategory::SensitiveSecurityPattern
    );
    assert_eq!(
        ExclusionCategory::infer("keys/id_rsa", None),
        ExclusionCategory::SensitiveSecurityPattern
    );
    assert_eq!(
        ExclusionCategory::infer("config/secrets.json", None),
        ExclusionCategory::SensitiveSecurityPattern
    );

    // 2. Vendor / Third-party directories
    assert_eq!(
        ExclusionCategory::infer("node_modules/lodash/index.js", None),
        ExclusionCategory::VendorOrThirdParty
    );
    assert_eq!(
        ExclusionCategory::infer("vendor/bundle/ruby/setup.rb", None),
        ExclusionCategory::VendorOrThirdParty
    );
    assert_eq!(
        ExclusionCategory::infer("target/debug/build/main.rs", None),
        ExclusionCategory::VendorOrThirdParty
    );
    assert_eq!(
        ExclusionCategory::infer("third_party/zlib/deflate.c", None),
        ExclusionCategory::VendorOrThirdParty
    );

    // 3. Category hints
    assert_eq!(
        ExclusionCategory::infer("data/giant.csv", Some(FileCategory::Oversized)),
        ExclusionCategory::OversizedFile
    );
    assert_eq!(
        ExclusionCategory::infer("build/bundle.js", Some(FileCategory::GeneratedOrLockfile)),
        ExclusionCategory::GeneratedOrLockfile
    );
    assert_eq!(
        ExclusionCategory::infer("assets/font.ttf", Some(FileCategory::Binary)),
        ExclusionCategory::BinaryOrAsset
    );

    // 4. Filename / Extension heuristics
    assert_eq!(
        ExclusionCategory::infer("Cargo.lock", None),
        ExclusionCategory::GeneratedOrLockfile
    );
    assert_eq!(
        ExclusionCategory::infer("package-lock.json", None),
        ExclusionCategory::GeneratedOrLockfile
    );
    assert_eq!(
        ExclusionCategory::infer("dist/app.min.js", None),
        ExclusionCategory::GeneratedOrLockfile
    );
    assert_eq!(
        ExclusionCategory::infer("dist/app.map", None),
        ExclusionCategory::GeneratedOrLockfile
    );
    assert_eq!(
        ExclusionCategory::infer("static/logo.png", None),
        ExclusionCategory::BinaryOrAsset
    );
    assert_eq!(
        ExclusionCategory::infer("bin/engine.wasm", None),
        ExclusionCategory::BinaryOrAsset
    );
    assert_eq!(
        ExclusionCategory::infer("archive.tar.gz", None),
        ExclusionCategory::BinaryOrAsset
    );

    // 5. Fallback
    assert_eq!(
        ExclusionCategory::infer("random_unknown_file.xyz", None),
        ExclusionCategory::Other
    );
}

#[test]
fn test_exclusion_category_cdm_mapping_and_remediation() {
    let categories = [
        (ExclusionCategory::OversizedFile, ExclusionReason::SizeLimit),
        (
            ExclusionCategory::GeneratedOrLockfile,
            ExclusionReason::ExcludedExtension,
        ),
        (
            ExclusionCategory::VendorOrThirdParty,
            ExclusionReason::RulePattern,
        ),
        (
            ExclusionCategory::BinaryOrAsset,
            ExclusionReason::BinaryFile,
        ),
        (
            ExclusionCategory::SubmoduleOrSymlink,
            ExclusionReason::UnsupportedType,
        ),
        (
            ExclusionCategory::PolicyPattern,
            ExclusionReason::RulePattern,
        ),
        (
            ExclusionCategory::SensitiveSecurityPattern,
            ExclusionReason::SecurityRestricted,
        ),
        (ExclusionCategory::Other, ExclusionReason::UnsupportedType),
    ];

    for (cat, expected_cdm) in categories {
        assert_eq!(cat.to_cdm_reason(), expected_cdm);
        assert!(!cat.as_str().is_empty());
        assert!(!cat.default_remediation().is_empty());
    }
}

#[test]
fn test_extraction_failure_category_inference_and_retryability() {
    let cases = [
        (
            "HTTP connect timeout after 30s",
            ExtractionFailureCategory::NetworkTimeout,
            true,
        ),
        (
            "Connection reset by peer",
            ExtractionFailureCategory::NetworkTimeout,
            true,
        ),
        (
            "Secondary rate limit exceeded (429)",
            ExtractionFailureCategory::RateLimitExceeded,
            true,
        ),
        (
            "Blob too large: exceeds 100 MB ceiling",
            ExtractionFailureCategory::BlobTooLarge,
            false,
        ),
        (
            "Base64 decoding failed or corrupted SHA mismatch",
            ExtractionFailureCategory::CorruptedContent,
            false,
        ),
        (
            "Subtree traversal recursion depth limit reached",
            ExtractionFailureCategory::SubtreeTraversalFailed,
            false,
        ),
        (
            "Disk I/O error writing blob to storage",
            ExtractionFailureCategory::IoOrStorage,
            false,
        ),
        (
            "Unknown internal error encountered",
            ExtractionFailureCategory::Unknown,
            false,
        ),
    ];

    for (msg, expected_cat, expected_retryable) in cases {
        let inferred = ExtractionFailureCategory::infer(msg);
        assert_eq!(inferred, expected_cat);
        assert_eq!(inferred.is_retryable(), expected_retryable);
        assert!(!inferred.as_str().is_empty());
    }
}

#[test]
fn test_excluded_file_record_and_cdm_conversion() {
    let record = ExcludedFileRecord::new(
        "Cargo.lock",
        "abc123def456abc123def456abc123def456abc1",
        ExclusionCategory::GeneratedOrLockfile,
        15_420,
        Some("TOML".to_string()),
        "Dependency lockfile excluded to conserve token budget",
        Some("*.lock".to_string()),
    );

    assert_eq!(record.path, "Cargo.lock");
    assert_eq!(record.category, ExclusionCategory::GeneratedOrLockfile);
    assert_eq!(record.size_bytes, 15_420);
    assert_eq!(record.language.as_deref(), Some("TOML"));
    assert!(record.remediation.contains("include_lockfiles"));

    let cdm_rec = record.to_cdm_record();
    assert_eq!(cdm_rec.original_path, "Cargo.lock");
    assert_eq!(cdm_rec.reason, ExclusionReason::ExcludedExtension);
    assert_eq!(cdm_rec.pattern.as_deref(), Some("*.lock"));
    assert_eq!(cdm_rec.byte_size, Some(15_420));
    assert!(cdm_rec.details.unwrap().contains("Dependency lockfile"));
}

#[test]
fn test_failed_file_record_creation() {
    let failure = FailedFileRecord::new(
        "src/heavy_parser.rs",
        "1111222233334444555566667777888899990000",
        "GitHub API timed out while fetching blob",
        Some(504),
    );

    assert_eq!(failure.path, "src/heavy_parser.rs");
    assert_eq!(
        failure.error_category,
        ExtractionFailureCategory::NetworkTimeout
    );
    assert!(failure.retry_eligible);
    assert_eq!(failure.http_status, Some(504));
}

#[test]
fn test_missing_file_record_creation() {
    let missing = MissingFileRecord::new(
        "docs/missing_guide.md",
        "aabbccddeeff00112233445566778899aabbccdd",
        Some(4096),
        "Discovered in tree root but completely absent in all extraction manifests",
    );

    assert_eq!(missing.path, "docs/missing_guide.md");
    assert_eq!(missing.expected_size, Some(4096));
    assert!(missing
        .diagnostics
        .contains("absent in all extraction manifests"));
}

#[test]
fn test_exclusion_audit_report_clean_vs_critical() {
    let mut report = ExclusionAuditReport::new("test-owner/clean-repo");
    report.total_discovered_files = 10;
    report.total_excluded_files = 3;
    assert!(report.is_clean());
    assert!(!report.has_critical_issues());

    // Introduce a failure
    report.total_failed_files = 1;
    assert!(!report.is_clean());
    assert!(report.has_critical_issues());

    // Reset and introduce a missing file
    report.total_failed_files = 0;
    report.total_missing_files = 1;
    assert!(!report.is_clean());
    assert!(report.has_critical_issues());
}

#[test]
fn test_generate_from_manifest_with_mixed_entries() {
    let entries = vec![
        ManifestFileEntry::new(
            "src/main.rs",
            "1111111111111111111111111111111111111111",
            FileCategory::SourceCode,
            2048,
            FileExtractionStatus::Extracted,
        )
        .with_language("Rust"),
        ManifestFileEntry::new(
            "package-lock.json",
            "2222222222222222222222222222222222222222",
            FileCategory::GeneratedOrLockfile,
            35_000,
            FileExtractionStatus::Excluded {
                reason: "Generated lockfile skipped".to_string(),
            },
        )
        .with_language("JSON"),
        ManifestFileEntry::new(
            "static/hero.png",
            "3333333333333333333333333333333333333333",
            FileCategory::Binary,
            150_000,
            FileExtractionStatus::Excluded {
                reason: "Binary asset not ingested".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "giant_dataset.csv",
            "4444444444444444444444444444444444444444",
            FileCategory::Oversized,
            15_000_000,
            FileExtractionStatus::Excluded {
                reason: "Exceeds max file size ceiling".to_string(),
            },
        ),
        ManifestFileEntry::new(
            ".env.production",
            "5555555555555555555555555555555555555555",
            FileCategory::Configuration,
            512,
            FileExtractionStatus::Excluded {
                reason: "Sensitive credentials pattern detected".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "src/broken.rs",
            "6666666666666666666666666666666666666666",
            FileCategory::SourceCode,
            1024,
            FileExtractionStatus::Failed {
                error: "Connection timed out connecting to GitHub".to_string(),
            },
        )
        .with_language("Rust"),
    ];
    let manifest = RepoFileManifest::from_entries(entries);

    // Mock an accounting balance sheet containing a missing file
    let mut balance_sheet = FileAccountingBalanceSheet::new("acme/mega-repo");
    balance_sheet.discrepancies.push(AccountingDiscrepancy {
        path: "docs/architecture.md".to_string(),
        kind: AccountingDiscrepancyKind::MissingUnaccounted,
        severity: DiscrepancySeverity::Critical,
        expected_detail: "7777777777777777777777777777777777777777".to_string(),
        actual_detail: "Discovered in tree but omitted from extraction".to_string(),
    });

    let report = ExclusionReportGenerator::generate_from_manifest(
        &manifest,
        Some(&balance_sheet),
        "acme/mega-repo",
        Some("main".to_string()),
        Some("fedcba9876543210fedcba9876543210fedcba98".to_string()),
    );

    assert_eq!(report.target_repo, "acme/mega-repo");
    assert_eq!(report.target_ref.as_deref(), Some("main"));
    assert_eq!(
        report.pinned_sha.as_deref(),
        Some("fedcba9876543210fedcba9876543210fedcba98")
    );
    assert_eq!(report.total_discovered_files, 6);
    assert_eq!(report.total_excluded_files, 4);
    assert_eq!(report.total_failed_files, 1);
    assert_eq!(report.total_missing_files, 1);
    assert!(!report.is_clean());
    assert!(report.has_critical_issues());

    assert_eq!(report.retryable_failures().len(), 1);
    assert_eq!(report.retryable_failures()[0].path, "src/broken.rs");

    // Breakdown validations
    assert_eq!(
        report.exclusion_breakdown.get("generated_or_lockfile"),
        Some(&1)
    );
    assert_eq!(report.exclusion_breakdown.get("binary_or_asset"), Some(&1));
    assert_eq!(report.exclusion_breakdown.get("oversized_file"), Some(&1));
    assert_eq!(
        report.exclusion_breakdown.get("sensitive_security_pattern"),
        Some(&1)
    );
    assert_eq!(report.failure_breakdown.get("network_timeout"), Some(&1));

    // CDM Exclusion records conversion
    let cdm_records = report.to_cdm_exclusion_records();
    assert_eq!(cdm_records.len(), 4);
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == ".env.production"
            && r.reason == ExclusionReason::SecurityRestricted));
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == "package-lock.json"
            && r.reason == ExclusionReason::ExcludedExtension));
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == "static/hero.png" && r.reason == ExclusionReason::BinaryFile));
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == "giant_dataset.csv" && r.reason == ExclusionReason::SizeLimit));
}

#[test]
fn test_exclusion_audit_report_markdown_and_json_formatting() {
    let entries = vec![
        ManifestFileEntry::new(
            ".env",
            "1111111111111111111111111111111111111111",
            FileCategory::Configuration,
            128,
            FileExtractionStatus::Excluded {
                reason: "Secret credentials file".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "src/broken.rs",
            "2222222222222222222222222222222222222222",
            FileCategory::SourceCode,
            1024,
            FileExtractionStatus::Failed {
                error: "HTTP 429 rate limit exceeded".to_string(),
            },
        )
        .with_language("Rust"),
    ];
    let manifest = RepoFileManifest::from_entries(entries);

    let report = ExclusionReportGenerator::generate_from_manifest(
        &manifest,
        None,
        "owner/repo",
        Some("v1.0.0".to_string()),
        Some("abcdef0123456789abcdef0123456789abcdef01".to_string()),
    );

    // Markdown verification
    let md = report.to_markdown();
    assert!(md.contains("# Missing & Excluded Files Audit: owner/repo"));
    assert!(md.contains("CRITICAL: Missing / Failed Files Detected"));
    assert!(md.contains("- **Ref**: `v1.0.0`"));
    assert!(md.contains("- **Commit**: `abcdef0123456789abcdef0123456789abcdef01`"));
    assert!(md.contains("### Exclusion Category Breakdown"));
    assert!(md.contains("sensitive_security_pattern"));
    assert!(md.contains("### Extraction Failure Log (1 files)"));
    assert!(md.contains("src/broken.rs"));
    assert!(md.contains("rate_limit_exceeded"));
    assert!(md.contains("### Excluded Files Index (1 files)"));
    assert!(md.contains(".env"));

    // JSON serialization / deserialization roundtrip
    let json_pretty = report.to_json().expect("Pretty JSON failed");
    let json_compact = report.to_json_compact().expect("Compact JSON failed");

    let deserialized: ExclusionAuditReport =
        serde_json::from_str(&json_pretty).expect("Deserialization failed");
    assert_eq!(deserialized, report);

    let deserialized_compact: ExclusionAuditReport =
        serde_json::from_str(&json_compact).expect("Deserialization compact failed");
    assert_eq!(deserialized_compact, report);
}

#[test]
fn test_exclusion_audit_report_cdm_section_generation() {
    let entries = vec![ManifestFileEntry::new(
        "package-lock.json",
        "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
        FileCategory::GeneratedOrLockfile,
        8192,
        FileExtractionStatus::Excluded {
            reason: "Lockfile excluded".to_string(),
        },
    )
    .with_language("JSON")];
    let manifest = RepoFileManifest::from_entries(entries);

    let report = ExclusionReportGenerator::generate_from_manifest(
        &manifest,
        None,
        "org/project",
        None,
        None,
    );

    let cdm_section = report.to_cdm_section("https://github.com/org/project");
    assert_eq!(cdm_section.section_id, "sec-exclusion-and-missing-report");
    assert_eq!(
        cdm_section.title.as_deref(),
        Some("Missing and Excluded Repository Files Audit")
    );
    assert_eq!(
        cdm_section
            .attributes
            .get("manifest_role")
            .map(String::as_str),
        Some("exclusion_and_missing_report")
    );
    assert_eq!(
        cdm_section.attributes.get("is_clean").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        cdm_section
            .attributes
            .get("total_discovered_files")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        cdm_section
            .attributes
            .get("total_excluded_files")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        cdm_section
            .attributes
            .get("total_failed_files")
            .map(String::as_str),
        Some("0")
    );
    assert_eq!(
        cdm_section
            .attributes
            .get("total_missing_files")
            .map(String::as_str),
        Some("0")
    );
    assert_eq!(cdm_section.content_hash.len(), 64);
    assert!(cdm_section.token_metrics.as_ref().unwrap().estimated_tokens > 0);
}

#[test]
fn test_client_generate_exclusion_report_integration() {
    let client = GitHubApiClient::new();

    let entries = vec![
        ManifestFileEntry::new(
            "bin/tool.wasm",
            "9999888877776666555544443333222211110000",
            FileCategory::Binary,
            65_536,
            FileExtractionStatus::Excluded {
                reason: "Binary asset".to_string(),
            },
        ),
        ManifestFileEntry::new(
            "src/tool.rs",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            FileCategory::SourceCode,
            1024,
            FileExtractionStatus::Extracted,
        )
        .with_language("Rust"),
    ];
    let manifest = RepoFileManifest::from_entries(entries);

    let report = client.generate_exclusion_report(
        Some(&manifest),
        None,
        None,
        None,
        None,
        None,
        None,
        "client-test/repo",
        Some("master".to_string()),
        None,
    );

    assert_eq!(report.target_repo, "client-test/repo");
    assert_eq!(report.total_discovered_files, 2);
    assert_eq!(report.total_excluded_files, 1);
    assert_eq!(report.excluded_files[0].path, "bin/tool.wasm");
    assert!(report.is_clean());
}
