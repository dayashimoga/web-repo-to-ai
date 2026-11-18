//! Integration tests for file classification and completeness mapping in `crates/github-extractor`.

use std::collections::HashMap;

use web_repo_to_ai_cdm::CompletenessReport;
use web_repo_to_ai_github_extractor::{
    detect_language, FileCategory, FileClassificationConfig, FileClassifier, GitFileMode,
    GitTreeEntry, GitTreeEntryType, GitTreeResponse,
};

#[test]
fn test_canonical_file_categories_classification() {
    let config = FileClassificationConfig::default();

    let cases = vec![
        ("src/lib.rs", FileCategory::SourceCode, true),
        ("frontend/app.tsx", FileCategory::SourceCode, true),
        ("scripts/deploy.py", FileCategory::SourceCode, true),
        ("README.md", FileCategory::Documentation, true),
        ("docs/ARCHITECTURE.md", FileCategory::Documentation, true),
        ("LICENSE-MIT", FileCategory::Documentation, true),
        (
            ".github/workflows/ci.yml",
            FileCategory::Configuration,
            true,
        ),
        ("tsconfig.json", FileCategory::Configuration, true),
        ("Cargo.toml", FileCategory::DependencyManifest, true),
        ("package.json", FileCategory::DependencyManifest, true),
        ("requirements.txt", FileCategory::DependencyManifest, true),
        ("tests/integration_test.rs", FileCategory::TestFixture, true),
        ("src/component.test.ts", FileCategory::TestFixture, true),
        ("Cargo.lock", FileCategory::GeneratedOrLockfile, false),
        (
            "package-lock.json",
            FileCategory::GeneratedOrLockfile,
            false,
        ),
        ("bundle.min.js", FileCategory::GeneratedOrLockfile, false),
        ("assets/icon.png", FileCategory::Binary, false),
        ("fonts/inter.woff2", FileCategory::Binary, false),
        ("models/weights.bin", FileCategory::Binary, false),
    ];

    for (path, expected_cat, expected_eligible) in cases {
        let entry = GitTreeEntry::new(
            path,
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "abc1234",
            Some(1024),
        );
        let classified = FileClassifier::classify_entry(&entry, &config);

        assert_eq!(
            classified.category, expected_cat,
            "Path '{}' expected category {:?}, got {:?}",
            path, expected_cat, classified.category
        );
        assert_eq!(
            classified.is_eligible, expected_eligible,
            "Path '{}' expected eligible {}, got {}",
            path, expected_eligible, classified.is_eligible
        );
    }
}

#[test]
fn test_programming_language_detection() {
    let test_matrix = vec![
        ("main.rs", Some("rs"), Some("rust")),
        ("server.ts", Some("ts"), Some("typescript")),
        ("app.tsx", Some("tsx"), Some("typescript")),
        ("index.js", Some("js"), Some("javascript")),
        ("script.py", Some("py"), Some("python")),
        ("service.go", Some("go"), Some("go")),
        ("Application.java", Some("java"), Some("java")),
        ("kernel.c", Some("c"), Some("c")),
        ("engine.cpp", Some("cpp"), Some("cpp")),
        ("Program.cs", Some("cs"), Some("csharp")),
        ("gem.rb", Some("rb"), Some("ruby")),
        ("index.php", Some("php"), Some("php")),
        ("App.swift", Some("swift"), Some("swift")),
        ("Activity.kt", Some("kt"), Some("kotlin")),
        ("Main.scala", Some("scala"), Some("scala")),
        ("deploy.sh", Some("sh"), Some("shell")),
        ("schema.sql", Some("sql"), Some("sql")),
        ("index.html", Some("html"), Some("html")),
        ("style.css", Some("css"), Some("css")),
        ("data.json", Some("json"), Some("json")),
        ("config.yaml", Some("yaml"), Some("yaml")),
        ("Settings.toml", Some("toml"), Some("toml")),
        ("page.svelte", Some("svelte"), Some("svelte")),
        ("Component.vue", Some("vue"), Some("vue")),
        ("main.zig", Some("zig"), Some("zig")),
        ("plugin.lua", Some("lua"), Some("lua")),
        ("service.proto", Some("proto"), Some("protobuf")),
        ("Dockerfile", None, Some("dockerfile")),
        ("Dockerfile.production", None, Some("dockerfile")),
        ("Makefile", None, Some("makefile")),
        ("Makefile.rules", None, Some("makefile")),
        ("archive.unknown", Some("unknown"), None),
    ];

    for (name, ext, expected_lang) in test_matrix {
        let detected = detect_language(name, ext);
        assert_eq!(
            detected.as_deref(),
            expected_lang,
            "File '{}' with ext {:?} expected lang {:?}, got {:?}",
            name,
            ext,
            expected_lang,
            detected
        );
    }
}

#[test]
fn test_size_threshold_and_oversized_handling() {
    let config = FileClassificationConfig::default().with_max_file_size(500 * 1024); // 500 KiB

    let small_file = GitTreeEntry::new(
        "src/normal.rs",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "sha1",
        Some(400 * 1024),
    );
    let classified_small = FileClassifier::classify_entry(&small_file, &config);
    assert_eq!(classified_small.category, FileCategory::SourceCode);
    assert!(classified_small.is_eligible);

    let large_file = GitTreeEntry::new(
        "src/generated_dataset.rs",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "sha2",
        Some(600 * 1024),
    );
    let classified_large = FileClassifier::classify_entry(&large_file, &config);
    assert_eq!(classified_large.category, FileCategory::Oversized);
    assert!(!classified_large.is_eligible);
    assert!(classified_large
        .exclusion_reason
        .as_ref()
        .unwrap()
        .contains("exceeds limit"));
}

#[test]
fn test_excluded_path_prefixes_filtering() {
    let config = FileClassificationConfig::default()
        .with_excluded_prefix("vendor/")
        .with_excluded_prefix("third_party/");

    let excluded_cases = vec![
        "node_modules/express/index.js",
        "target/debug/build/main.rs",
        ".git/HEAD",
        ".venv/lib/python3.11/site.py",
        "vendor/bundle/gems/rails.rb",
        "third_party/zlib/deflate.c",
        "nested/sub/node_modules/pkg/index.js",
    ];

    for path in excluded_cases {
        let entry = GitTreeEntry::new(
            path,
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "sha123",
            Some(1024),
        );
        let classified = FileClassifier::classify_entry(&entry, &config);
        assert!(
            !classified.is_eligible,
            "Path '{}' should be excluded from extraction",
            path
        );
        assert!(classified.exclusion_reason.is_some());
    }
}

#[test]
fn test_custom_overrides_precedence() {
    let mut overrides = HashMap::new();
    overrides.insert("scripts/vendor_shim.js".to_string(), FileCategory::Other);
    overrides.insert("docs/special.bin".to_string(), FileCategory::Documentation);

    let config = FileClassificationConfig {
        custom_overrides: overrides,
        ..Default::default()
    };

    let shim = GitTreeEntry::new(
        "scripts/vendor_shim.js",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "s1",
        Some(100),
    );
    let classified_shim = FileClassifier::classify_entry(&shim, &config);
    assert_eq!(classified_shim.category, FileCategory::Other);
    assert!(!classified_shim.is_eligible);

    let special = GitTreeEntry::new(
        "docs/special.bin",
        GitFileMode::RegularFile,
        GitTreeEntryType::Blob,
        "s2",
        Some(100),
    );
    let classified_special = FileClassifier::classify_entry(&special, &config);
    assert_eq!(classified_special.category, FileCategory::Documentation);
    assert!(classified_special.is_eligible);
}

#[test]
fn test_cdm_completeness_report_formula_invariants() {
    let entries = vec![
        GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s1",
            Some(1000),
        ),
        GitTreeEntry::new(
            "src/utils.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s2",
            Some(2000),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s3",
            Some(3000),
        ),
        GitTreeEntry::new(
            "Cargo.toml",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s4",
            Some(500),
        ),
        GitTreeEntry::new(
            "Cargo.lock",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s5",
            Some(4000),
        ),
        GitTreeEntry::new(
            "logo.png",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s6",
            Some(50000),
        ),
        GitTreeEntry::new(
            "huge.dat",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s7",
            Some(2 * 1024 * 1024),
        ),
        GitTreeEntry::new(
            "node_modules/dep/index.js",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s8",
            Some(100),
        ),
    ];

    let tree = GitTreeResponse::new("tree_sha_123", entries, false);
    let config = FileClassificationConfig::default();
    let report = tree.classify(&config);

    assert_eq!(report.total_files, 8);
    // Eligible: src/main.rs, src/utils.rs, README.md, Cargo.toml (4)
    assert_eq!(report.eligible_count, 4);
    // Excluded: Cargo.lock, logo.png (binary), huge.dat (oversized), node_modules (excluded prefix) (4)
    assert_eq!(report.excluded_count, 4);
    assert_eq!(report.binary_count, 1);
    assert_eq!(report.oversized_count, 1);

    let cdm_report: CompletenessReport = report.to_expected_completeness_report();
    assert!(
        cdm_report.formula_valid(),
        "Completeness report formula must be strictly valid"
    );
    assert_eq!(cdm_report.discovered_count, 8);
    assert_eq!(cdm_report.eligible_count, 4);
    assert_eq!(cdm_report.extracted_count, 4);
    assert_eq!(cdm_report.unsupported_count, 1); // binary
    assert_eq!(cdm_report.excluded_count, 3); // 4 excluded minus 1 unsupported
    assert_eq!(cdm_report.failed_count, 0);

    // Sum invariant: discovered == extracted + excluded + failed + unsupported
    assert_eq!(
        cdm_report.discovered_count,
        cdm_report.extracted_count
            + cdm_report.excluded_count
            + cdm_report.failed_count
            + cdm_report.unsupported_count
    );
}

#[test]
fn test_tree_classify_filter_and_aggregation_helpers() {
    let entries = vec![
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s1",
            Some(100),
        ),
        GitTreeEntry::new(
            "src/app.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s2",
            Some(200),
        ),
        GitTreeEntry::new(
            "tests/lib_test.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s3",
            Some(300),
        ),
        GitTreeEntry::new(
            "docs/manual.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s4",
            Some(400),
        ),
        GitTreeEntry::new(
            "banner.jpg",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            "s5",
            Some(500),
        ),
    ];

    let tree = GitTreeResponse::new("sha999", entries, false);
    let config = FileClassificationConfig::default();
    let report = tree.classify(&config);

    let source_files = report.filter_by_category(FileCategory::SourceCode);
    assert_eq!(source_files.len(), 2);

    let test_files = report.filter_by_category(FileCategory::TestFixture);
    assert_eq!(test_files.len(), 1);

    let eligible = report.eligible_files();
    assert_eq!(eligible.len(), 4); // 2 source + 1 test + 1 doc

    let excluded = report.excluded_files();
    assert_eq!(excluded.len(), 1); // banner.jpg (binary)

    assert_eq!(report.category_counts.get("source_code"), Some(&2));
    assert_eq!(report.category_sizes.get("source_code"), Some(&300));
}
