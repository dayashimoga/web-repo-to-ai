//! # Integration Tests for Debugging and Documentation Context Profile (Subphase 7.3.4)
//!
//! Validates error definitions, observability/tracing, API contracts, troubleshooting runbooks,
//! targeted error pattern matching, boundary truncation, and telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    apply_debugging_profile, apply_debugging_profile_for_model,
    apply_debugging_profile_with_target, apply_debugging_profile_with_target_for_model,
    ContextBudget, ContextProfile, DebuggingProfile, ProfileKind, ProfileOptimizationResult,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/diagnostic-engine",
            "https://github.com/org/diagnostic-engine",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/diagnostic-engine".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "application/vnd.github.v3+json".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections,
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    }
}

/// Helper to create a section with specific attributes.
fn make_section(
    id: &str,
    path: &str,
    kind: SectionKind,
    body: &str,
    attrs: Vec<(&str, &str)>,
) -> CdmSection {
    let mut attributes = BTreeMap::new();
    for (k, v) in attrs {
        attributes.insert(k.to_string(), v.to_string());
    }

    CdmSection {
        section_id: id.to_string(),
        parent_id: None,
        title: Some(path.to_string()),
        kind,
        original_path: Some(path.to_string()),
        source_url: format!("https://github.com/org/diagnostic-engine/blob/main/{path}"),
        line_range: None,
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(body),
        body: body.to_string(),
        token_metrics: None,
        attributes,
    }
}

#[test]
fn test_debugging_profile_pins_errors_observability_and_api_contracts() {
    let s_err = make_section(
        "s_err",
        "crates/core/src/error.rs",
        SectionKind::CodeFile,
        "#[derive(thiserror::Error, Debug)]\npub enum CoreError {\n    #[error(\"network failure\")]\n    Network,\n}",
        vec![("code_nature", "core_implementation")],
    );

    let s_obs = make_section(
        "s_obs",
        "crates/observability/src/telemetry.rs",
        SectionKind::CodeFile,
        "pub fn record_latency_span(route: &str, ms: u64) {\n    tracing::info!(route, ms, \"latency measured\");\n}",
        vec![("code_nature", "core_implementation")],
    );

    let s_api = make_section(
        "s_api",
        "services/api/src/routes/export_api.rs",
        SectionKind::CodeFile,
        "pub async fn export_handler() -> Result<Json<ExportDto>, StatusCode> {\n    Ok(Json(ExportDto::new()))\n}",
        vec![("code_nature", "interface_or_type_definition")],
    );

    let s_runbook = make_section(
        "s_runbook",
        "docs/TROUBLESHOOTING.md",
        SectionKind::Prose,
        "# Operational Troubleshooting\nWhen encountering 500 status codes, check database pool latency.",
        vec![],
    );

    let s_manifest = make_section(
        "s_manifest",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"diagnostic-engine\"\nversion = \"0.1.0\"",
        vec![],
    );

    let s_ui = make_section(
        "s_ui",
        "apps/web/src/lib/components/Modal.svelte",
        SectionKind::CodeFile,
        "<div class=\"modal-backdrop fixed inset-0 bg-black/50\">Dialog</div>",
        vec![("code_nature", "ui_component")],
    );

    let s_changelog = make_section(
        "s_changelog",
        "CHANGELOG.md",
        SectionKind::Prose,
        "# Changelog\nHistorical notes.\n## [0.1.0] - initial release",
        vec![],
    );

    let doc = make_test_doc(vec![
        s_err,
        s_obs,
        s_api,
        s_runbook,
        s_manifest,
        s_ui,
        s_changelog,
    ]);

    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let profile = DebuggingProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    assert_eq!(result.profile_kind, ProfileKind::DebuggingAndDocumentation);

    let included_paths: Vec<&str> = result
        .document
        .sections
        .iter()
        .filter_map(|s| s.original_path.as_deref())
        .collect();

    // Critical debugging context MUST be included
    assert!(included_paths.contains(&"crates/core/src/error.rs"));
    assert!(included_paths.contains(&"crates/observability/src/telemetry.rs"));
    assert!(included_paths.contains(&"services/api/src/routes/export_api.rs"));
    assert!(included_paths.contains(&"docs/TROUBLESHOOTING.md"));
    assert!(included_paths.contains(&"Cargo.toml"));

    // Cosmetic UI and changelog MUST be excluded
    assert!(!included_paths.contains(&"apps/web/src/lib/components/Modal.svelte"));
    assert!(!included_paths.contains(&"CHANGELOG.md"));

    // Audit trail must track excluded UI components
    assert!(result
        .document
        .exclusions
        .iter()
        .any(|ex| ex.original_path == "apps/web/src/lib/components/Modal.svelte"));
}

#[test]
fn test_debugging_profile_targeted_error_pattern_matching() {
    let s_general = make_section(
        "s_gen",
        "crates/core/src/worker.rs",
        SectionKind::CodeFile,
        "pub fn run_worker_loop() { loop {} }",
        vec![],
    );

    let s_specific_err = make_section(
        "s_ratelimit",
        "crates/core/src/rate_limiter.rs",
        SectionKind::CodeFile,
        "pub struct RateLimitExceeded;\nimpl RateLimitExceeded {\n    pub fn retry_after(&self) -> u64 { 60 }\n}",
        vec![],
    );

    let doc = make_test_doc(vec![s_general, s_specific_err]);

    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let result = apply_debugging_profile_with_target(&doc, "RateLimitExceeded", &budget)
        .expect("Targeted optimization succeeds");

    assert_eq!(result.document.sections.len(), 2);

    let rate_limit_sec = result
        .document
        .sections
        .iter()
        .find(|s| s.original_path.as_deref() == Some("crates/core/src/rate_limiter.rs"))
        .expect("Rate limiter section included");

    assert_eq!(
        rate_limit_sec
            .attributes
            .get("debugging_category")
            .map(|s| s.as_str()),
        Some("error_definitions_and_propagation")
    );
    assert_eq!(
        rate_limit_sec.attributes.get("pin").map(|s| s.as_str()),
        Some("must_include")
    );
}

#[test]
fn test_debugging_profile_elevates_negative_tests_while_suppressing_benign_tests() {
    let s_negative_test = make_section(
        "s_neg_test",
        "tests/error_recovery_test.rs",
        SectionKind::CodeFile,
        "#[test]\nfn test_handle_disconnect_expect_err() {\n    let res = client.connect();\n    assert!(res.is_err());\n}",
        vec![("is_test", "true")],
    );

    let s_gui_test = make_section(
        "s_gui_test",
        "apps/web/tests/typography_rendering_test.ts",
        SectionKind::CodeFile,
        "test('font weight classes', () => { expect(renderFont()).toBe('bold'); });",
        vec![("is_test", "true")],
    );

    let doc = make_test_doc(vec![s_negative_test, s_gui_test]);

    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let profile = DebuggingProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    let included_paths: Vec<&str> = result
        .document
        .sections
        .iter()
        .filter_map(|s| s.original_path.as_deref())
        .collect();

    // Negative test is included
    assert!(included_paths.contains(&"tests/error_recovery_test.rs"));

    // Cosmetic GUI test is excluded
    assert!(!included_paths.contains(&"apps/web/tests/typography_rendering_test.ts"));
}

#[test]
fn test_debugging_profile_model_targeting() {
    let s1 = make_section(
        "s1",
        "crates/core/src/error.rs",
        SectionKind::CodeFile,
        "#[derive(Debug)]\npub enum ServiceError { NotFound, Timeout }",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "docs/API.md",
        SectionKind::Prose,
        "# API Reference\nEndpoints and status codes.",
        vec![],
    );

    let doc = make_test_doc(vec![s1, s2]);

    for model in [
        TargetModel::Gpt4o,
        TargetModel::Claude35Sonnet,
        TargetModel::Gemini15Pro,
        TargetModel::Llama3,
    ] {
        let result =
            apply_debugging_profile_for_model(&doc, model).expect("Model optimization succeeds");

        assert_eq!(result.profile_kind, ProfileKind::DebuggingAndDocumentation);
        assert!(result.plan.is_within_budget());
        assert!(result.document.sections.len() >= 2);

        // Also test targeted convenience for model
        let targeted_result =
            apply_debugging_profile_with_target_for_model(&doc, "NotFound", model)
                .expect("Targeted model optimization succeeds");
        assert!(targeted_result.plan.is_within_budget());
    }
}

#[test]
fn test_debugging_profile_tight_budget_boundary_truncation() {
    let large_error_body = "pub fn format_diagnostic_stack_trace() -> String {\n".to_string()
        + &"    // Error frame inspection and diagnostic symbol resolution\n    let frame = \"trace\";\n".repeat(30)
        + "    frame.to_string()\n}\n";

    let s_large_err = make_section(
        "s_err",
        "crates/core/src/diagnostics/stack_trace.rs",
        SectionKind::CodeFile,
        &large_error_body,
        vec![],
    );

    let doc = make_test_doc(vec![s_large_err]);

    // Budget: 160 total tokens, with 50+30+10 headroom -> 70 usable tokens
    let budget = ContextBudget::new(160, 50, 30, 10);
    let profile = DebuggingProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    assert!(result.plan.is_within_budget());
    assert_eq!(result.document.sections.len(), 1);

    let section = &result.document.sections[0];
    assert!(section.body.contains("[Truncated:"));
}

#[test]
fn test_debugging_profile_markdown_summary_and_json_serialization() {
    let s_err = make_section(
        "s_err",
        "crates/core/src/error.rs",
        SectionKind::CodeFile,
        "pub enum AppError { Fatal }",
        vec![],
    );
    let s_doc = make_section(
        "s_doc",
        "docs/DEPLOYMENT.md",
        SectionKind::Prose,
        "# Deployment Runbook\nHealthcheck probes and alerts.",
        vec![],
    );

    let doc = make_test_doc(vec![s_err, s_doc]);
    let budget = ContextBudget::new(5000, 300, 500, 100);
    let result = apply_debugging_profile(&doc, &budget).expect("Profile succeeds");

    // Markdown summary report and table validation
    let summary = result.to_markdown_summary();
    assert!(summary.contains("debugging_and_documentation"));
    assert!(summary.contains("Documentation & Prose"));
    assert!(summary.contains("Core Implementation Code"));
    assert!(summary.contains("Section Budget Decisions"));

    let markdown_table = result.metrics.to_markdown_table();
    assert!(markdown_table.contains("Documentation & Prose"));
    assert!(markdown_table.contains("Core Implementation Code"));

    // JSON serialization roundtrip
    let json = serde_json::to_string_pretty(&result).expect("JSON serialization succeeds");
    assert!(json.contains("debugging_and_documentation"));
    assert!(json.contains("profile_kind"));

    let deserialized: ProfileOptimizationResult =
        serde_json::from_str(&json).expect("JSON deserialization succeeds");
    assert_eq!(
        deserialized.profile_kind,
        ProfileKind::DebuggingAndDocumentation
    );
    assert_eq!(deserialized.document.sections.len(), 2);
}
