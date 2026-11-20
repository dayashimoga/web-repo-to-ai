//! # Integration Tests: Structured Document Golden Benchmark Suite
//!
//! Evaluates the 8 canonical structured document archetypes against quantitative
//! ground-truth expectations: OpenAPI JSON, OpenAPI YAML, JSON Schema, Kubernetes YAML,
//! RFC 4180 CSV, Metrics TSV, Jupyter Notebook, and Adversarial Limits.

use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_structured_extractor::golden::{
    all_golden_fixtures, build_adversarial_limits_fixture, build_jsonschema_fixture,
    build_jupyter_notebook_fixture, build_openapi_json_fixture, build_openapi_yaml_fixture,
    build_tabular_csv_fixture, build_tabular_tsv_fixture, build_yaml_stream_fixture,
    evaluate_golden_fixture, run_all_golden_benchmarks,
};
use web_repo_to_ai_structured_extractor::{StructuredLimits, StructuredOptions};

#[test]
fn test_all_golden_benchmarks_pass_with_high_recall() {
    let options = StructuredOptions::default();
    let report = run_all_golden_benchmarks(&options).expect("execute all golden benchmarks");

    assert_eq!(
        report.total_fixtures, 8,
        "Suite must contain exactly 8 golden archetypes"
    );
    assert_eq!(
        report.passed_fixtures,
        8,
        "All 8 golden archetypes must pass all acceptance criteria. Summary:\n{}",
        report.to_markdown_summary()
    );
    assert!(
        report.mean_fragment_recall >= 0.95,
        "Mean fragment recall must be >= 95%, got {:.2}%",
        report.mean_fragment_recall * 100.0
    );
    assert!(
        report.total_duration_ms < 50,
        "Entire 8-archetype golden benchmark suite should complete in < 50ms, took {} ms",
        report.total_duration_ms
    );

    let summary = report.to_markdown_summary();
    assert!(summary.contains("# Structured Document Golden Benchmark Report"));
    assert!(summary.contains("OpenAPI 3.0 JSON Specification"));
    assert!(summary.contains("OpenAPI 3.1 YAML Specification"));
    assert!(summary.contains("JSON Schema Draft-07 Data Model"));
    assert!(summary.contains("Kubernetes Multi-Document YAML"));
    assert!(summary.contains("RFC 4180 E-Commerce Orders CSV"));
    assert!(summary.contains("Server Performance Metrics TSV"));
    assert!(summary.contains("Machine Learning Pipeline Notebook"));
    assert!(summary.contains("Adversarial Resource Limits"));
}

#[test]
fn test_golden_openapi_json_evaluation() {
    let fixture = build_openapi_json_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate OpenAPI JSON");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res.metrics.section_kinds_matched);
    assert!(res.metrics.section_count >= 3);
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::Prose));
}

#[test]
fn test_golden_openapi_yaml_evaluation() {
    let fixture = build_openapi_yaml_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate OpenAPI YAML");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res
        .cdm_doc
        .metadata
        .title
        .as_deref()
        .unwrap()
        .contains("Cloud Fleet Manager API"));
}

#[test]
fn test_golden_jsonschema_evaluation() {
    let fixture = build_jsonschema_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate JSON Schema");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res.metrics.section_count >= 2);
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::Prose));
}

#[test]
fn test_golden_yaml_stream_evaluation() {
    let fixture = build_yaml_stream_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate YAML Stream");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res.metrics.section_count >= 3); // 3 docs
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::CodeSnippet));
}

#[test]
fn test_golden_tabular_csv_evaluation() {
    let fixture = build_tabular_csv_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate Tabular CSV");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert_eq!(res.metrics.section_count, 2); // 1 summary + 1 data table
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::Table));
}

#[test]
fn test_golden_tabular_tsv_evaluation() {
    let fixture = build_tabular_tsv_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate Tabular TSV");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::Table));
}

#[test]
fn test_golden_jupyter_notebook_evaluation() {
    let fixture = build_jupyter_notebook_fixture();
    let options = StructuredOptions::default();
    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate Jupyter Notebook");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert_eq!(res.metrics.fragment_recall, 1.0);
    assert!(res.metrics.section_count >= 6);
    assert!(res
        .cdm_doc
        .sections
        .iter()
        .any(|s| s.kind == SectionKind::CodeSnippet));
}

#[test]
fn test_golden_adversarial_limits_evaluation() {
    let fixture = build_adversarial_limits_fixture();
    let limits = StructuredLimits::default()
        .with_max_tabular_rows(10)
        .with_truncate_on_row_limit(true);
    let options = StructuredOptions::default().with_limits(limits);

    let res = evaluate_golden_fixture(&fixture, &options).expect("evaluate Adversarial Limits");

    assert!(res.metrics.is_passing);
    assert!(res.metrics.schema_valid);
    assert!(res.metrics.fragment_recall >= 0.95);
    // Completeness invariant holds under truncation
    assert_eq!(
        res.cdm_doc.completeness.discovered_count,
        res.cdm_doc.completeness.extracted_count + res.cdm_doc.completeness.excluded_count
    );
}

#[test]
fn test_all_fixtures_have_valid_filenames_and_content() {
    let fixtures = all_golden_fixtures();
    for f in fixtures {
        assert!(!f.name.is_empty());
        assert!(!f.filename.is_empty());
        assert!(!f.raw_content.is_empty());
        assert!(!f.expected_fragments.is_empty());
        assert!(f.expected_min_sections >= 1);
    }
}
