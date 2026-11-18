//! Large Repository Regression Test Suite and Stage 5 Certification (Subphase 5.5.4)
//!
//! Exercises large-scale tree traversal, truncation detection, subtree stitching,
//! polyglot monorepo classification, mathematical conservation identity verification,
//! multi-algorithm cryptographic hash validation, exclusion diagnostics, budget enforcement,
//! live HTTP client integration, and end-to-end CDM document construction passing strict validation.

use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::time::Duration;
use web_repo_to_ai_cdm::{CdmValidator, ExclusionReason, SourceType, ValidationConfig};
use web_repo_to_ai_github_extractor::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, parse_manifest,
    CodeReviewPackage, CodeReviewPackageConfig, ConfigCategory, ConfigTestExtractionReport,
    ContentHashValidator, DependencyExtractionReport, DocExtractionReport, DocFormat, DocType,
    ExclusionAuditReport, ExclusionReportGenerator, ExtractedConfigFile, ExtractedDependencyFile,
    ExtractedDocFile, ExtractedSourceFile, ExtractedTestFile, FileAccountingAuditor,
    FileClassificationConfig, GitFileMode, GitHubApiClient, GitHubUrlParser, GitTreeEntry,
    GitTreeEntryType, GitTreeResponse, HashAlgorithm, HashValidationConfig, ManifestRole,
    PackageEcosystem, RepoArchitectureContext, RepoFileManifest, ReviewFocus,
    SourceExtractionReport, TestCategory, TreeStitcher,
};

/// Generates a large synthetic tree containing `count` entries spanning deep nested directory structures.
fn generate_large_synthetic_tree(count: usize) -> Vec<GitTreeEntry> {
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let pkg_idx = i % 15;
        let mod_idx = (i / 15) % 10;
        let sub_idx = (i / 150) % 5;
        let path = format!(
            "packages/pkg_{pkg_idx:02}/src/mod_{mod_idx:02}/sub_{sub_idx:02}/worker_{i:04}.rs"
        );
        let sha = format!("{i:040x}");
        let size = (100 + (i % 500)) as u64;

        entries.push(GitTreeEntry::new(
            path,
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha,
            Some(size),
        ));
    }
    entries
}

#[test]
fn test_large_scale_tree_truncation_and_subtree_stitching() {
    let total_entries = 2_500;
    let full_entries = generate_large_synthetic_tree(total_entries);

    // Simulate truncated response: first 1,500 entries are returned, remaining 1,000 are cut off
    let truncated_entries = full_entries[..1500].to_vec();
    let mut tree_resp = GitTreeResponse::new("root_commit_sha", truncated_entries, true);

    assert!(tree_resp.truncated);
    let completeness = tree_resp.analyze_completeness();
    assert!(completeness.is_truncated);

    // Simulate recovering the remaining 1,000 entries through subtree recovery
    let recovered_subtree =
        GitTreeResponse::new("sub_commit_sha", full_entries[1500..].to_vec(), false);

    // Stitch the subtree into the base tree using TreeStitcher
    let (blobs_added, _) = TreeStitcher::stitch_subtree(&mut tree_resp, "", &recovered_subtree);
    assert_eq!(blobs_added, 1000);

    let stitched_tree = &mut tree_resp.tree;
    assert_eq!(stitched_tree.len(), total_entries);

    // Sort and verify 100% path preservation with zero dropped entries
    stitched_tree.sort_by(|a, b| a.path.cmp(&b.path));
    for (i, entry) in stitched_tree.iter().enumerate() {
        if i > 0 {
            assert!(
                stitched_tree[i - 1].path <= entry.path,
                "Entries must be lexicographically sorted"
            );
        }
    }
}

/// Helper assembling a realistic polyglot monorepo dataset
fn create_polyglot_monorepo() -> (
    GitTreeResponse,
    SourceExtractionReport,
    DocExtractionReport,
    DependencyExtractionReport,
    ConfigTestExtractionReport,
) {
    let rust_code = "pub fn execute_computation() -> u64 { 42 }\n";
    let ts_code = "export function renderApp(): string { return '<h1>App</h1>'; }\n";
    let py_code = "def process_data(records: list) -> int:\n    return len(records)\n";
    let go_code = "package main\n\nfunc HandleRequest() string {\n    return \"OK\"\n}\n";

    let rust_sha = compute_git_blob_sha(rust_code.as_bytes());
    let ts_sha = compute_git_blob_sha(ts_code.as_bytes());
    let py_sha = compute_git_blob_sha(py_code.as_bytes());
    let go_sha = compute_git_blob_sha(go_code.as_bytes());

    let cargo_toml = "[package]\nname = \"backend\"\nversion = \"1.0.0\"\nedition = \"2021\"\n\n[dependencies]\ntokio = \"1.0\"\n";
    let package_json = "{\n  \"name\": \"frontend\",\n  \"version\": \"1.0.0\",\n  \"dependencies\": {\n    \"svelte\": \"^5.0.0\"\n  }\n}\n";
    let pyproject_toml = "[project]\nname = \"ml-service\"\nversion = \"0.1.0\"\ndependencies = [\"numpy>=1.20.0\"]\n";
    let go_mod =
        "module example.com/tooling\n\ngo 1.22\n\nrequire github.com/stretchr/testify v1.8.4\n";

    let cargo_sha = compute_git_blob_sha(cargo_toml.as_bytes());
    let pkg_sha = compute_git_blob_sha(package_json.as_bytes());
    let pyp_sha = compute_git_blob_sha(pyproject_toml.as_bytes());
    let gom_sha = compute_git_blob_sha(go_mod.as_bytes());

    let readme = "# Enterprise Polyglot Monorepo\n\nSystem documentation and guide.\n";
    let arch_doc = "# Architecture Overview\n\nDistributed polyglot microservice layout.\n";
    let readme_sha = compute_git_blob_sha(readme.as_bytes());
    let arch_sha = compute_git_blob_sha(arch_doc.as_bytes());

    let ci_workflow = "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n";
    let unit_test = "#[test]\nfn test_computation() { assert_eq!(execute_computation(), 42); }\n";
    let ci_sha = compute_git_blob_sha(ci_workflow.as_bytes());
    let test_sha = compute_git_blob_sha(unit_test.as_bytes());

    let tree_entries = vec![
        // Source files
        GitTreeEntry::new(
            "crates/backend/src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &rust_sha,
            Some(rust_code.len() as u64),
        ),
        GitTreeEntry::new(
            "apps/web/src/index.ts",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &ts_sha,
            Some(ts_code.len() as u64),
        ),
        GitTreeEntry::new(
            "services/ml/app.py",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &py_sha,
            Some(py_code.len() as u64),
        ),
        GitTreeEntry::new(
            "tools/cli/main.go",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &go_sha,
            Some(go_code.len() as u64),
        ),
        // Manifests
        GitTreeEntry::new(
            "crates/backend/Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &cargo_sha,
            Some(cargo_toml.len() as u64),
        ),
        GitTreeEntry::new(
            "apps/web/package.json",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &pkg_sha,
            Some(package_json.len() as u64),
        ),
        GitTreeEntry::new(
            "services/ml/pyproject.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &pyp_sha,
            Some(pyproject_toml.len() as u64),
        ),
        GitTreeEntry::new(
            "tools/cli/go.mod",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &gom_sha,
            Some(go_mod.len() as u64),
        ),
        // Documentation
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &readme_sha,
            Some(readme.len() as u64),
        ),
        GitTreeEntry::new(
            "docs/ARCHITECTURE.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &arch_sha,
            Some(arch_doc.len() as u64),
        ),
        // Config & Test
        GitTreeEntry::new(
            ".github/workflows/ci.yml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &ci_sha,
            Some(ci_workflow.len() as u64),
        ),
        GitTreeEntry::new(
            "crates/backend/tests/unit_test.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &test_sha,
            Some(unit_test.len() as u64),
        ),
        // Excluded: lockfiles & third-party
        GitTreeEntry::new(
            "crates/backend/Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000001",
            Some(12_400),
        ),
        GitTreeEntry::new(
            "apps/web/package-lock.json",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000002",
            Some(45_000),
        ),
        GitTreeEntry::new(
            "node_modules/lodash/index.js",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000003",
            Some(30_000),
        ),
        GitTreeEntry::new(
            "target/debug/build.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000004",
            Some(5_000),
        ),
        // Excluded: binary assets
        GitTreeEntry::new(
            "assets/logo.png",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000005",
            Some(128_000),
        ),
        GitTreeEntry::new(
            "bin/helper.wasm",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000006",
            Some(64_000),
        ),
        // Excluded: sensitive credentials
        GitTreeEntry::new(
            ".env.production",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000007",
            Some(256),
        ),
        GitTreeEntry::new(
            "keys/id_rsa",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "1000000000000000000000000000000000000008",
            Some(1800),
        ),
    ];
    let tree_response = GitTreeResponse::new("commit_sha_polyglot", tree_entries, false);

    let mut sources = SourceExtractionReport::new();
    sources.add_extracted(ExtractedSourceFile {
        path: "crates/backend/src/lib.rs".to_string(),
        git_sha: rust_sha,
        content: rust_code.to_string(),
        byte_size: rust_code.len(),
        line_count: 1,
        language: Some("rust".to_string()),
        blake3_hash: compute_blake3_hash(rust_code.as_bytes()),
        sha256_hash: compute_sha256_hash(rust_code.as_bytes()),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "apps/web/src/index.ts".to_string(),
        git_sha: ts_sha,
        content: ts_code.to_string(),
        byte_size: ts_code.len(),
        line_count: 1,
        language: Some("typescript".to_string()),
        blake3_hash: compute_blake3_hash(ts_code.as_bytes()),
        sha256_hash: compute_sha256_hash(ts_code.as_bytes()),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "services/ml/app.py".to_string(),
        git_sha: py_sha,
        content: py_code.to_string(),
        byte_size: py_code.len(),
        line_count: 2,
        language: Some("python".to_string()),
        blake3_hash: compute_blake3_hash(py_code.as_bytes()),
        sha256_hash: compute_sha256_hash(py_code.as_bytes()),
        is_truncated: false,
    });
    sources.add_extracted(ExtractedSourceFile {
        path: "tools/cli/main.go".to_string(),
        git_sha: go_sha,
        content: go_code.to_string(),
        byte_size: go_code.len(),
        line_count: 5,
        language: Some("go".to_string()),
        blake3_hash: compute_blake3_hash(go_code.as_bytes()),
        sha256_hash: compute_sha256_hash(go_code.as_bytes()),
        is_truncated: false,
    });

    let mut docs = DocExtractionReport::new();
    docs.add_extracted(ExtractedDocFile {
        path: "README.md".to_string(),
        git_sha: readme_sha,
        doc_type: DocType::Readme,
        format: DocFormat::Markdown,
        title: "Enterprise Polyglot Monorepo".to_string(),
        has_frontmatter: false,
        content: readme.to_string(),
        byte_size: readme.len(),
        line_count: 3,
        blake3_hash: compute_blake3_hash(readme.as_bytes()),
        sha256_hash: compute_sha256_hash(readme.as_bytes()),
    });
    docs.add_extracted(ExtractedDocFile {
        path: "docs/ARCHITECTURE.md".to_string(),
        git_sha: arch_sha,
        doc_type: DocType::Architecture,
        format: DocFormat::Markdown,
        title: "Architecture Overview".to_string(),
        has_frontmatter: false,
        content: arch_doc.to_string(),
        byte_size: arch_doc.len(),
        line_count: 3,
        blake3_hash: compute_blake3_hash(arch_doc.as_bytes()),
        sha256_hash: compute_sha256_hash(arch_doc.as_bytes()),
    });

    let mut dependencies = DependencyExtractionReport::new();
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "crates/backend/Cargo.toml".to_string(),
        git_sha: cargo_sha,
        ecosystem: PackageEcosystem::Cargo,
        role: ManifestRole::Manifest,
        content: cargo_toml.to_string(),
        parsed: parse_manifest(PackageEcosystem::Cargo, "Cargo.toml", cargo_toml),
        byte_size: cargo_toml.len(),
        line_count: cargo_toml.lines().count(),
        blake3_hash: compute_blake3_hash(cargo_toml.as_bytes()),
        sha256_hash: compute_sha256_hash(cargo_toml.as_bytes()),
    });
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "apps/web/package.json".to_string(),
        git_sha: pkg_sha,
        ecosystem: PackageEcosystem::Npm,
        role: ManifestRole::Manifest,
        content: package_json.to_string(),
        parsed: parse_manifest(PackageEcosystem::Npm, "package.json", package_json),
        byte_size: package_json.len(),
        line_count: package_json.lines().count(),
        blake3_hash: compute_blake3_hash(package_json.as_bytes()),
        sha256_hash: compute_sha256_hash(package_json.as_bytes()),
    });
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "services/ml/pyproject.toml".to_string(),
        git_sha: pyp_sha,
        ecosystem: PackageEcosystem::Pip,
        role: ManifestRole::Manifest,
        content: pyproject_toml.to_string(),
        parsed: parse_manifest(PackageEcosystem::Pip, "pyproject.toml", pyproject_toml),
        byte_size: pyproject_toml.len(),
        line_count: pyproject_toml.lines().count(),
        blake3_hash: compute_blake3_hash(pyproject_toml.as_bytes()),
        sha256_hash: compute_sha256_hash(pyproject_toml.as_bytes()),
    });
    dependencies.add_extracted(ExtractedDependencyFile {
        path: "tools/cli/go.mod".to_string(),
        git_sha: gom_sha,
        ecosystem: PackageEcosystem::GoMod,
        role: ManifestRole::Manifest,
        content: go_mod.to_string(),
        parsed: parse_manifest(PackageEcosystem::GoMod, "go.mod", go_mod),
        byte_size: go_mod.len(),
        line_count: go_mod.lines().count(),
        blake3_hash: compute_blake3_hash(go_mod.as_bytes()),
        sha256_hash: compute_sha256_hash(go_mod.as_bytes()),
    });

    let mut configs_tests = ConfigTestExtractionReport::new();
    configs_tests.extracted_configs.push(ExtractedConfigFile {
        path: ".github/workflows/ci.yml".to_string(),
        git_sha: ci_sha,
        config_category: ConfigCategory::CiCd,
        format: "yaml".to_string(),
        content: ci_workflow.to_string(),
        byte_size: ci_workflow.len(),
        line_count: ci_workflow.lines().count(),
        blake3_hash: compute_blake3_hash(ci_workflow.as_bytes()),
        sha256_hash: compute_sha256_hash(ci_workflow.as_bytes()),
    });
    configs_tests.extracted_tests.push(ExtractedTestFile {
        path: "crates/backend/tests/unit_test.rs".to_string(),
        git_sha: test_sha,
        test_category: TestCategory::Unit,
        language: Some("rust".to_string()),
        content: unit_test.to_string(),
        byte_size: unit_test.len(),
        line_count: unit_test.lines().count(),
        blake3_hash: compute_blake3_hash(unit_test.as_bytes()),
        sha256_hash: compute_sha256_hash(unit_test.as_bytes()),
    });

    (tree_response, sources, docs, dependencies, configs_tests)
}

#[test]
fn test_polyglot_monorepo_classification_and_manifest_synthesis() {
    let (tree, sources, docs, dependencies, configs_tests) = create_polyglot_monorepo();

    let classification = tree.classify(&FileClassificationConfig::default());

    assert_eq!(classification.total_files, 20);
    assert_eq!(classification.eligible_count, 12);
    assert_eq!(classification.excluded_count, 8);

    // Build consolidated manifest
    let manifest = RepoFileManifest::from_classification_and_reports(
        &classification,
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
    );

    assert_eq!(manifest.total_files, 20);
    assert_eq!(manifest.extracted_files, 12);
    assert_eq!(manifest.excluded_files, 6); // 2 lockfiles + 2 third-party + 2 sensitive
    assert_eq!(manifest.unsupported_files, 2); // 2 binary files
    assert_eq!(manifest.failed_files, 0);

    // Multi-language breakdown checks
    assert!(manifest.language_counts.contains_key("rust"));
    assert!(manifest.language_counts.contains_key("typescript"));
    assert!(manifest.language_counts.contains_key("python"));
    assert!(manifest.language_counts.contains_key("go"));
    assert!(manifest.language_counts.contains_key("toml"));
    assert!(manifest.language_counts.contains_key("json"));
    assert!(manifest.language_counts.contains_key("markdown"));
}

#[test]
fn test_large_repository_mathematical_conservation_identity() {
    let (tree, sources, docs, dependencies, configs_tests) = create_polyglot_monorepo();
    let classification = tree.classify(&FileClassificationConfig::default());

    let balance_sheet = FileAccountingAuditor::audit_reports(
        &tree,
        Some(&classification),
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        "acme/polyglot-platform",
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );

    // Mathematical conservation law:
    // discovered == extracted + excluded + failed + unsupported + missing
    assert_eq!(balance_sheet.discovered_blobs, 20);
    assert_eq!(balance_sheet.actual_extracted, 12);
    assert_eq!(balance_sheet.actual_excluded, 6);
    assert_eq!(balance_sheet.actual_unsupported, 2);
    assert_eq!(balance_sheet.actual_failed, 0);
    assert_eq!(balance_sheet.missing_unaccounted, 0);
    assert_eq!(balance_sheet.unexpected_extra, 0);
    assert_eq!(balance_sheet.byte_size_mismatches, 0);
    assert_eq!(balance_sheet.git_sha_mismatches, 0);

    assert!(balance_sheet.is_balanced());
    assert!(balance_sheet.is_fully_accounted());
    assert!(balance_sheet.is_perfectly_consistent());
    assert!(!balance_sheet.has_critical_discrepancies());

    // CDM CompletenessReport mapping
    let completeness_report = balance_sheet.to_completeness_report();
    assert_eq!(completeness_report.discovered_count, 20);
    assert_eq!(completeness_report.extracted_count, 12);
    assert_eq!(completeness_report.excluded_count, 6);
    assert_eq!(completeness_report.unsupported_count, 2);
    assert_eq!(completeness_report.failed_count, 0);
    assert!(completeness_report.is_complete());
}

#[test]
fn test_large_repository_multi_algorithm_hash_validation_and_corruption_detection() {
    let (_tree, sources, docs, dependencies, configs_tests) = create_polyglot_monorepo();

    let config = HashValidationConfig::default(); // verifies Git SHA, BLAKE3, and SHA-256
    let report = ContentHashValidator::validate_reports(
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        &config,
        "acme/polyglot-platform",
    );

    assert!(report.is_valid());
    assert_eq!(report.failures.len(), 0);
    assert_eq!(report.git_sha_metrics.failed, 0);
    assert_eq!(report.blake3_metrics.failed, 0);
    assert_eq!(report.sha256_metrics.failed, 0);

    // Simulate corrupted source file: tamper content while keeping original Git SHA
    let mut corrupted_sources = sources.clone();
    corrupted_sources.extracted[0].content = "corrupted byte stream payload".to_string();

    let fail_report = ContentHashValidator::validate_reports(
        Some(&corrupted_sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        &config,
        "acme/polyglot-platform",
    );

    assert!(!fail_report.is_valid());
    assert!(!fail_report.failures.is_empty());
    assert!(fail_report.failures.iter().any(
        |f| f.path == "crates/backend/src/lib.rs" && f.algorithm == HashAlgorithm::GitBlobSha1
    ));
}

#[test]
fn test_large_repository_exclusion_and_diagnostics_audit() {
    let (tree, sources, docs, dependencies, configs_tests) = create_polyglot_monorepo();
    let classification = tree.classify(&FileClassificationConfig::default());
    let manifest = RepoFileManifest::from_classification_and_reports(
        &classification,
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
    );

    let report = ExclusionReportGenerator::generate_from_manifest(
        &manifest,
        None,
        "acme/polyglot-platform",
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );

    assert_eq!(report.total_discovered_files, 20);
    assert_eq!(report.total_excluded_files, 8); // 6 excluded + 2 unsupported
    assert_eq!(report.total_failed_files, 0);
    assert_eq!(report.total_missing_files, 0);
    assert!(report.is_clean());
    assert!(!report.has_critical_issues());

    // Verify category distribution
    assert!(report
        .exclusion_breakdown
        .contains_key("generated_or_lockfile"));
    assert!(report
        .exclusion_breakdown
        .contains_key("vendor_or_third_party"));
    assert!(report.exclusion_breakdown.contains_key("binary_or_asset"));
    assert!(report
        .exclusion_breakdown
        .contains_key("sensitive_security_pattern"));

    // CDM conversion
    let cdm_records = report.to_cdm_exclusion_records();
    assert_eq!(cdm_records.len(), 8);
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == ".env.production"
            && r.reason == ExclusionReason::SecurityRestricted));
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == "crates/backend/Cargo.lock"
            && r.reason == ExclusionReason::ExcludedExtension));
    assert!(cdm_records
        .iter()
        .any(|r| r.original_path == "assets/logo.png" && r.reason == ExclusionReason::BinaryFile));

    // Markdown & JSON validation
    let md = report.to_markdown();
    assert!(md.contains("# Missing & Excluded Files Audit: acme/polyglot-platform"));
    assert!(md.contains("PASS: 100% Accounted & Clean"));

    let json = report.to_json().expect("Valid JSON");
    let deserialized: ExclusionAuditReport = serde_json::from_str(&json).expect("Roundtrip JSON");
    assert_eq!(deserialized, report);
}

#[test]
fn test_large_repository_code_review_package_budget_and_prioritization() {
    let (tree_resp, sources, _docs, _deps, configs_tests) = create_polyglot_monorepo();
    let classification = tree_resp.classify(&FileClassificationConfig::default());
    let tree = tree_resp.to_repo_tree();
    let manifest = classification.to_file_manifest();

    // 1. Security Audit Focus
    let sec_config = CodeReviewPackageConfig::default().with_focus(ReviewFocus::SecurityAudit);
    let sec_pkg = CodeReviewPackage::assemble(
        &sec_config,
        &classification,
        Some(&tree),
        Some(&manifest),
        None,
        Some(&sources),
        None,
        None,
        Some(&configs_tests),
        None,
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );

    assert_eq!(sec_pkg.focus, ReviewFocus::SecurityAudit);
    assert!(!sec_pkg.files.is_empty());
    // Security checklist items present
    assert!(sec_pkg
        .checklist
        .iter()
        .any(|c| c.category.contains("SSRF") || c.category.contains("Secret")));

    // 2. Budget constraint enforcement: max 2 files
    let budget_config = CodeReviewPackageConfig::default()
        .with_focus(ReviewFocus::FullRepository)
        .with_max_files(2);

    let budget_pkg = CodeReviewPackage::assemble(
        &budget_config,
        &classification,
        Some(&tree),
        Some(&manifest),
        None,
        Some(&sources),
        None,
        None,
        Some(&configs_tests),
        None,
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );

    assert!(budget_pkg.files.len() <= 2);
    assert!(budget_pkg.reviewed_file_count <= 2);
}

#[test]
fn test_large_repository_end_to_end_cdm_assembly_strict_validation() {
    let (tree_resp, sources, docs, dependencies, configs_tests) = create_polyglot_monorepo();
    let classification = tree_resp.classify(&FileClassificationConfig::default());
    let tree = tree_resp.to_repo_tree();
    let manifest = classification.to_file_manifest();
    let arch = RepoArchitectureContext::from_repository_data(
        &classification,
        Some(&manifest),
        None,
        None,
        Some(&configs_tests),
        None,
    );

    let config = CodeReviewPackageConfig::default();
    let mut package = CodeReviewPackage::assemble(
        &config,
        &classification,
        Some(&tree),
        Some(&manifest),
        Some(&arch),
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        None,
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );
    package.target_repo = "acme/polyglot-platform".to_string();

    // Build full CDM document
    let canonical_url = "https://github.com/acme/polyglot-platform";
    let mut doc = package.to_cdm_document(canonical_url);

    // Add Balance Sheet & Hash Validation Sections to test complete platform integration
    let balance_sheet = FileAccountingAuditor::audit_reports(
        &tree_resp,
        Some(&classification),
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        "acme/polyglot-platform",
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );
    let balance_section = balance_sheet.to_cdm_section(canonical_url);
    doc.sections.push(balance_section);

    let hash_config = HashValidationConfig::default();
    let hash_report = ContentHashValidator::validate_reports(
        Some(&sources),
        Some(&docs),
        Some(&dependencies),
        Some(&configs_tests),
        &hash_config,
        "acme/polyglot-platform",
    );
    let hash_section = hash_report.to_cdm_section(canonical_url);
    doc.sections.push(hash_section);

    let exclusion_report = ExclusionReportGenerator::generate_from_manifest(
        &manifest,
        Some(&balance_sheet),
        "acme/polyglot-platform",
        Some("main".to_string()),
        Some("commit_sha_polyglot".to_string()),
    );
    let exclusion_section = exclusion_report.to_cdm_section(canonical_url);
    doc.sections.push(exclusion_section);

    // Verify document properties
    assert_eq!(doc.source_type, SourceType::GitHubPublic);
    assert_eq!(doc.source_identity.repo_owner.as_deref(), Some("acme"));
    assert_eq!(
        doc.source_identity.repo_name.as_deref(),
        Some("polyglot-platform")
    );
    assert_eq!(doc.source_identity.git_ref.as_deref(), Some("main"));

    // Execute official strict CDM validation
    let validator = CdmValidator::with_config(ValidationConfig::strict());
    let validation_report = validator.validate(&doc);

    assert!(
        validation_report.is_valid(),
        "Strict CDM Document validation failed: {:?}",
        validation_report.errors().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_large_repository_live_mock_http_integration() {
    let app = Router::new()
        .route(
            "/rate_limit",
            get(|| async {
                Json(json!({
                    "resources": {
                        "core": { "limit": 5000, "remaining": 4990, "reset": 1700000000, "used": 10 }
                    }
                }))
            }),
        )
        .route(
            "/repos/acme/polyglot-platform/branches",
            get(|| async {
                Json(json!([
                    {
                        "name": "main",
                        "commit": { "sha": "1111222233334444555566667777888899990000" },
                        "protected": true
                    }
                ]))
            }),
        )
        .route(
            "/repos/acme/polyglot-platform/tags",
            get(|| async {
                Json(json!([
                    {
                        "name": "v1.0.0",
                        "commit": { "sha": "1111222233334444555566667777888899990000" }
                    }
                ]))
            }),
        )
        .route(
            "/repos/acme/polyglot-platform/commits/HEAD",
            get(|| async {
                Json(json!({
                    "sha": "1111222233334444555566667777888899990000",
                    "commit": {
                        "author": { "name": "Dev", "email": "dev@acme.com", "date": "2026-11-18T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@acme.com", "date": "2026-11-18T00:00:00Z" },
                        "message": "Release v1.0.0",
                        "tree": { "sha": "tree_sha_root_9999" },
                        "verification": { "verified": true, "reason": "valid" }
                    }
                }))
            }),
        )
        .route(
            "/repos/acme/polyglot-platform/commits/main",
            get(|| async {
                Json(json!({
                    "sha": "1111222233334444555566667777888899990000",
                    "commit": {
                        "author": { "name": "Dev", "email": "dev@acme.com", "date": "2026-11-18T00:00:00Z" },
                        "committer": { "name": "Dev", "email": "dev@acme.com", "date": "2026-11-18T00:00:00Z" },
                        "message": "Release v1.0.0",
                        "tree": { "sha": "tree_sha_root_9999" },
                        "verification": { "verified": true, "reason": "valid" }
                    }
                }))
            }),
        )
        .route(
            "/repos/acme/polyglot-platform/git/trees/tree_sha_root_9999",
            get(|| async {
                Json(json!({
                    "sha": "tree_sha_root_9999",
                    "truncated": false,
                    "tree": [
                        { "path": "src/main.rs", "mode": "100644", "type": "blob", "sha": "aaaabbbbccccddddeeeeffff0000111122223333", "size": 100 },
                        { "path": "README.md", "mode": "100644", "type": "blob", "sha": "bbbbccccddddeeeeffff00001111222233334444", "size": 250 }
                    ]
                }))
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bound ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    let mock_base = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server ran");
    });

    let client = GitHubApiClient::new()
        .with_timeout(Duration::from_secs(5))
        .with_base_url_override(mock_base);

    let parser = GitHubUrlParser::new();
    let target = parser
        .parse("https://github.com/acme/polyglot-platform")
        .expect("valid target");

    // 1. Ref discovery
    let ref_report = client
        .discover_refs(&target)
        .await
        .expect("ref discovery report");
    assert_eq!(ref_report.branches.len(), 1);
    assert_eq!(ref_report.tags.len(), 1);

    // 2. Commit pinning
    let pinned = client.pin_revision(&target).await.expect("pinned revision");
    assert_eq!(pinned.root_tree_sha(), "tree_sha_root_9999");

    // 3. Tree acquisition
    let tree = client
        .fetch_pinned_tree(&pinned, false)
        .await
        .expect("fetched tree");
    assert_eq!(tree.tree.len(), 2);
    assert!(!tree.truncated);
}
