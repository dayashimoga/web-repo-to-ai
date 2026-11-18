//! # Integration Tests for Golden Comparison Across Context Profiles (Subphase 7.4.4)
//!
//! Validates deterministic multi-profile benchmarking against golden repository fixtures:
//! - Multi-language golden repository CDM fixture structure and token accounting
//! - Cross-profile comparative evaluation across General AI, Code Review, Security Review, Debugging, and Raw Knapsack
//! - Profile-specific invariant verification (must-include pinning, domain suppression, budget bounds)
//! - Tight micro-budget knapsack allocation and boundary truncation
//! - Bit-for-bit determinism and BLAKE3 hash stability across runs
//! - Comprehensive markdown comparison tables and JSON serialization roundtrip
//! - Fail-closed rejection of empty or invalid documents

use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_context_optimizer::{
    create_golden_repository_fixture, run_golden_profile_comparison, ContextBudget,
    ContextOptimizerError, GoldenComparisonReport, GoldenProfileConfig, ProfileKind,
};

#[test]
fn test_golden_repository_fixture_baseline() {
    let doc = create_golden_repository_fixture();

    assert_eq!(
        doc.sections.len(),
        20,
        "Golden repository must contain exactly 20 diverse sections"
    );
    assert!(!doc.canonical_url.is_empty());
    assert_eq!(
        doc.source_identity.canonical_url,
        "https://github.com/dayashimoga/golden-repo-engine"
    );

    // Verify presence of all required architectural components
    let paths: Vec<&str> = doc
        .sections
        .iter()
        .filter_map(|s| s.original_path.as_deref())
        .collect();
    assert!(paths.contains(&"Cargo.toml"));
    assert!(paths.contains(&"package.json"));
    assert!(paths.contains(&"README.md"));
    assert!(paths.contains(&"docs/ARCHITECTURE.md"));
    assert!(paths.contains(&"docs/TROUBLESHOOTING.md"));
    assert!(paths.contains(&"SECURITY.md"));
    assert!(paths.contains(&"CHANGELOG_OLD.md"));
    assert!(paths.contains(&"src/core/engine.rs"));
    assert!(paths.contains(&"src/api/routes.rs"));
    assert!(paths.contains(&"src/auth/jwt.rs"));
    assert!(paths.contains(&"src/crypto/vault.rs"));
    assert!(paths.contains(&"src/errors.rs"));
    assert!(paths.contains(&"src/telemetry/tracer.rs"));
    assert!(paths.contains(&"src/features/payment.rs"));
    assert!(paths.contains(&"tests/payment_test.rs"));
    assert!(paths.contains(&"tests/security_auth_test.rs"));
    assert!(paths.contains(&"tests/negative_error_test.rs"));
    assert!(paths.contains(&"tests/benchmark_test.rs"));
    assert!(paths.contains(&"ui/styles/theme.css"));
    assert!(paths.contains(&"ui/icons/logo.svg"));

    // Verify hash integrity and token metrics across all sections
    for sec in &doc.sections {
        assert!(
            sec.verify_hash(),
            "Section content hash mismatch on {}",
            sec.section_id
        );
        assert!(
            sec.token_metrics.is_some(),
            "Token metrics must be present on {}",
            sec.section_id
        );
        let tm = sec.token_metrics.as_ref().unwrap();
        assert!(tm.estimated_tokens > 0 || sec.body.is_empty());
        assert!(tm.char_count > 0 || sec.body.is_empty());
    }
}

#[test]
fn test_golden_multi_profile_comparison_standard_budget() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig::default();

    let report = run_golden_profile_comparison(&doc, &config)
        .expect("comparison should execute successfully");

    assert_eq!(report.benchmark_id, "golden-repo-v1");
    assert_eq!(report.original_sections_count, 20);
    assert_eq!(report.available_budget_tokens, 1300);
    assert_eq!(
        report.profiles.len(),
        5,
        "Report must benchmark all 5 profiles"
    );

    // All profiles must satisfy all domain-specific invariants
    assert!(
        report.all_invariants_passed(),
        "All invariants must pass on golden benchmark:\n{}",
        report.to_markdown_summary()
    );

    for (name, eval) in &report.profiles {
        assert!(
            eval.fitted_tokens <= report.available_budget_tokens,
            "Profile {} fitted {} exceeds available budget {}",
            name,
            eval.fitted_tokens,
            report.available_budget_tokens
        );
        assert!(
            eval.invariants_passed,
            "Profile {} invariants failed: {:?}",
            name, eval.invariant_diagnostics
        );
        assert!(
            eval.retained_sections_count > 0,
            "Profile {} retained zero sections",
            name
        );
        assert!(
            eval.excluded_sections_count > 0,
            "Profile {} excluded zero sections under budget",
            name
        );
        assert!(
            eval.preservation.fidelity_score > 0.0,
            "Profile {} zero fidelity score",
            name
        );
        assert!(
            eval.compression.overall.total_token_savings > 0,
            "Profile {} zero token savings",
            name
        );
    }
}

#[test]
fn test_golden_general_ai_invariants() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig::default();
    let report = run_golden_profile_comparison(&doc, &config).expect("comparison should succeed");

    let gen_ai = report
        .profiles
        .get("1_General_AI")
        .expect("General AI profile must be present");
    assert_eq!(gen_ai.profile_kind, Some(ProfileKind::GeneralAi));

    // Must preserve architectural and core implementation files
    assert!(
        gen_ai.has_path("README.md"),
        "General AI must preserve README.md"
    );
    assert!(
        gen_ai.has_path("Cargo.toml"),
        "General AI must preserve Cargo.toml"
    );
    assert!(
        gen_ai.has_path("src/core/engine.rs"),
        "General AI must preserve src/core/engine.rs"
    );

    // Must suppress low-signal benchmarks and cosmetic assets
    assert!(
        gen_ai.excludes_path("tests/benchmark_test.rs"),
        "General AI must suppress benchmark test"
    );
    assert!(
        gen_ai.excludes_path("ui/icons/logo.svg"),
        "General AI must suppress cosmetic SVG"
    );
    assert!(
        gen_ai.excludes_path("ui/styles/theme.css"),
        "General AI must suppress cosmetic CSS"
    );

    // Verify category distribution metrics
    let metrics = gen_ai
        .category_metrics
        .as_ref()
        .expect("Category metrics must be computed");
    assert_eq!(
        metrics.test_tokens, 0,
        "General AI with default config must include 0 test tokens"
    );
    assert!(
        metrics.core_code_tokens > 0,
        "General AI must include core code tokens"
    );
    assert!(
        metrics.documentation_tokens > 0,
        "General AI must include documentation tokens"
    );
    assert!(
        metrics.manifest_tokens > 0,
        "General AI must include manifest tokens"
    );
}

#[test]
fn test_golden_code_review_invariants() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig {
        changed_files: vec!["src/features/payment.rs".to_string()],
        ..Default::default()
    };

    let report = run_golden_profile_comparison(&doc, &config).expect("comparison should succeed");
    let cr = report
        .profiles
        .get("2_Code_Review")
        .expect("Code Review profile must be present");
    assert_eq!(cr.profile_kind, Some(ProfileKind::CodeReview));

    // Must unconditionally preserve modified file
    assert!(
        cr.has_path("src/features/payment.rs"),
        "Code Review must unconditionally preserve changed file"
    );

    // Must preserve associated unit tests for changed file
    assert!(
        cr.has_path("tests/payment_test.rs"),
        "Code Review must preserve associated unit test"
    );

    // Must suppress unmodified documentation to maximize diff review budget
    assert!(
        cr.excludes_path("docs/ARCHITECTURE.md"),
        "Code Review must suppress unmodified architecture doc"
    );
    assert!(
        cr.excludes_path("CHANGELOG_OLD.md"),
        "Code Review must suppress historical changelog"
    );

    // Must suppress cosmetic assets
    assert!(
        cr.excludes_path("ui/styles/theme.css"),
        "Code Review must suppress cosmetic stylesheet"
    );
    assert!(
        cr.excludes_path("ui/icons/logo.svg"),
        "Code Review must suppress cosmetic SVG"
    );
}

#[test]
fn test_golden_security_review_invariants() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig::default();

    let report = run_golden_profile_comparison(&doc, &config).expect("comparison should succeed");
    let sec = report
        .profiles
        .get("3_Security_Review")
        .expect("Security Review profile must be present");
    assert_eq!(sec.profile_kind, Some(ProfileKind::SecurityReview));

    // Must unconditionally preserve auth, crypto, and security policy
    assert!(
        sec.has_path("src/auth/jwt.rs"),
        "Security Review must preserve src/auth/jwt.rs"
    );
    assert!(
        sec.has_path("src/crypto/vault.rs"),
        "Security Review must preserve src/crypto/vault.rs"
    );
    assert!(
        sec.has_path("SECURITY.md"),
        "Security Review must preserve SECURITY.md"
    );

    // Must preserve security regression tests
    assert!(
        sec.has_path("tests/security_auth_test.rs"),
        "Security Review must preserve security test suite"
    );

    // Must suppress benign presentation components
    assert!(
        sec.excludes_path("ui/styles/theme.css"),
        "Security Review must suppress cosmetic styling"
    );
    assert!(
        sec.excludes_path("ui/icons/logo.svg"),
        "Security Review must suppress cosmetic SVG"
    );
}

#[test]
fn test_golden_debugging_invariants() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig {
        target_error_patterns: vec![
            "ERR_PAYMENT_FAILED".to_string(),
            "ERR_AUTH_FAILED".to_string(),
        ],
        ..Default::default()
    };

    let report = run_golden_profile_comparison(&doc, &config).expect("comparison should succeed");
    let deb = report
        .profiles
        .get("4_Debugging")
        .expect("Debugging profile must be present");
    assert_eq!(
        deb.profile_kind,
        Some(ProfileKind::DebuggingAndDocumentation)
    );

    // Must unconditionally preserve error taxonomy, telemetry, and troubleshooting runbook
    assert!(
        deb.has_path("src/errors.rs"),
        "Debugging must preserve error taxonomy src/errors.rs"
    );
    assert!(
        deb.has_path("src/telemetry/tracer.rs"),
        "Debugging must preserve telemetry tracer"
    );
    assert!(
        deb.has_path("docs/TROUBLESHOOTING.md"),
        "Debugging must preserve troubleshooting runbook"
    );

    // Must preserve negative failure-propagation tests
    assert!(
        deb.has_path("tests/negative_error_test.rs"),
        "Debugging must preserve negative tests"
    );

    // Must suppress cosmetic styling
    assert!(
        deb.excludes_path("ui/styles/theme.css"),
        "Debugging must suppress cosmetic stylesheet"
    );
}

#[test]
fn test_golden_tight_micro_budget_knapsack() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig {
        budget: ContextBudget::new(500, 100, 50, 50),
        ..Default::default()
    };

    let report = run_golden_profile_comparison(&doc, &config)
        .expect("comparison should succeed on tight budget");
    assert_eq!(report.available_budget_tokens, 300);

    for (name, eval) in &report.profiles {
        assert!(
            eval.fitted_tokens <= 300,
            "Profile {} fitted {} exceeded tight budget limit of 300 tokens",
            name,
            eval.fitted_tokens
        );
        assert!(eval.budget_utilization_pct > 0.0);
        assert!(eval.retained_sections_count < doc.sections.len());
        assert!(eval.excluded_sections_count > 0);

        // Verify that exclusions report has diagnostic remediations for every dropped section
        assert!(!eval.exclusions.explanations.is_empty());
        for exp in &eval.exclusions.explanations {
            assert!(!exp.rationale.is_empty());
            assert!(!exp.remediation.action.is_empty());
        }
    }
}

#[test]
fn test_golden_determinism_and_reproducibility() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig::default();

    let report1 = run_golden_profile_comparison(&doc, &config).expect("first run should succeed");
    let report2 = run_golden_profile_comparison(&doc, &config).expect("second run should succeed");

    assert_eq!(report1.original_tokens, report2.original_tokens);
    assert_eq!(
        report1.available_budget_tokens,
        report2.available_budget_tokens
    );
    assert_eq!(report1.profiles.len(), report2.profiles.len());

    for (name, eval1) in &report1.profiles {
        let eval2 = report2
            .profiles
            .get(name)
            .expect("profile must exist in both runs");
        assert_eq!(
            eval1.fitted_tokens, eval2.fitted_tokens,
            "Fitted tokens must be deterministic for {}",
            name
        );
        assert_eq!(
            eval1.retained_paths, eval2.retained_paths,
            "Retained paths sequence must be deterministic for {}",
            name
        );
        assert_eq!(
            eval1.excluded_paths, eval2.excluded_paths,
            "Excluded paths sequence must be deterministic for {}",
            name
        );
        assert_eq!(
            eval1.invariants_passed, eval2.invariants_passed,
            "Invariant status must be deterministic for {}",
            name
        );
    }
}

#[test]
fn test_golden_markdown_tables_and_json_serialization() {
    let doc = create_golden_repository_fixture();
    let config = GoldenProfileConfig::default();
    let report = run_golden_profile_comparison(&doc, &config).expect("comparison should succeed");

    // Markdown comparison table
    let comp_table = report.to_markdown_comparison_table();
    assert!(comp_table.contains("| Profile | Fitted / Budget | Util (%) | Retained | Excluded | Fidelity (%) | Cost Savings / 1k (GPT-4o) | Invariants |"));
    assert!(comp_table.contains("General AI"));
    assert!(comp_table.contains("Code Review"));
    assert!(comp_table.contains("Security Review"));
    assert!(comp_table.contains("Debugging"));
    assert!(comp_table.contains("Raw Knapsack"));

    // Category distribution table
    let cat_table = report.to_markdown_category_table();
    assert!(cat_table.contains(
        "| Profile | Documentation | Core Code | Manifests | Tests | Other | Total Fitted |"
    ));
    assert!(cat_table.contains("Core Code"));

    // Full diagnostic summary
    let summary = report.to_markdown_summary();
    assert!(summary.contains("## Golden Context Profile Benchmark: `golden-repo-v1`"));
    assert!(summary.contains("### 1. Cross-Profile Performance Matrix"));
    assert!(summary.contains("### 2. Content Category Token Allocation"));
    assert!(summary.contains("### 3. Profile Invariant Diagnostic Results"));

    // JSON roundtrip serialization
    let json = report.to_json().expect("JSON serialization must succeed");
    let roundtrip =
        GoldenComparisonReport::from_json(&json).expect("JSON deserialization must succeed");

    assert_eq!(roundtrip.benchmark_id, report.benchmark_id);
    assert_eq!(roundtrip.original_tokens, report.original_tokens);
    assert_eq!(
        roundtrip.available_budget_tokens,
        report.available_budget_tokens
    );
    assert_eq!(roundtrip.profiles.len(), report.profiles.len());

    for (k, v) in &report.profiles {
        let rv = roundtrip
            .profiles
            .get(k)
            .expect("profile must exist in roundtrip");
        assert_eq!(v.fitted_tokens, rv.fitted_tokens);
        assert_eq!(v.retained_sections_count, rv.retained_sections_count);
        assert_eq!(v.retained_paths, rv.retained_paths);
        assert_eq!(v.invariants_passed, rv.invariants_passed);
    }
}

#[test]
fn test_golden_fail_closed_empty_doc() {
    let empty_doc = CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: web_repo_to_ai_core::DocumentId::new(),
        source_identity: web_repo_to_ai_cdm::SourceIdentity::new(
            "https://example.com",
            "https://example.com",
        ),
        source_type: web_repo_to_ai_cdm::SourceType::GitHubPublic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: chrono::Utc::now(),
        revision: web_repo_to_ai_cdm::RevisionInfo::default(),
        content_type: "text/plain".to_string(),
        metadata: web_repo_to_ai_cdm::DocumentMetadata::default(),
        hierarchy: web_repo_to_ai_cdm::HierarchyNode::root("root"),
        sections: Vec::new(),
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: web_repo_to_ai_cdm::ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: web_repo_to_ai_cdm::CompletenessReport::default(),
    };

    let config = GoldenProfileConfig::default();
    let result = run_golden_profile_comparison(&empty_doc, &config);

    assert!(result.is_err(), "Empty document must fail closed");
    match result.unwrap_err() {
        ContextOptimizerError::DocumentEmpty => {}
        other => panic!("Expected DocumentEmpty error, got: {:?}", other),
    }
}
