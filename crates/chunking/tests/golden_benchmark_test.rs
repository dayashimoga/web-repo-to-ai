//! # Integration Tests: Golden Benchmark Suite & Retrieval Validation (`tests/golden_benchmark_test.rs`)

use web_repo_to_ai_chunking::{
    evaluate_golden_fixture, get_all_golden_fixtures, run_all_golden_benchmarks, ChunkingArchetype,
};

#[test]
fn test_all_8_golden_benchmarks_pass() {
    let report = run_all_golden_benchmarks().expect("Golden benchmarks should run cleanly");

    assert_eq!(
        report.results.len(),
        8,
        "Must evaluate all 8 golden archetypes"
    );
    assert!(
        report.all_passed,
        "All golden benchmarks must pass. Summary:\n{}",
        report.to_markdown_summary()
    );

    // Verify aggregate execution time is well within budget (< 500ms total for all 8)
    assert!(
        report.total_elapsed_micros < 500_000,
        "Total elapsed time {} us exceeded 500,000 us budget",
        report.total_elapsed_micros
    );

    // Verify markdown summary format contains key columns
    let md = report.to_markdown_summary();
    assert!(md.contains("RAG Chunking Golden Benchmark Evaluation Report"));
    assert!(md.contains("ALL BENCHMARKS PASSED"));
    assert!(md.contains("Multi-Language AST Code Repository"));
    assert!(md.contains("Hierarchical Markdown Documentation"));
    assert!(md.contains("Tabular Data Matrix (Header Retention)"));
    assert!(md.contains("OpenAPI 3.1 REST API Specification"));
    assert!(md.contains("Jupyter Notebook Interactive Pipeline"));
    assert!(md.contains("Oversized Declarations (Statement Splitting)"));
    assert!(md.contains("Cross-Reference Call & Import Graph"));
    assert!(md.contains("Adversarial Token & Boundary Limits"));
}

#[test]
fn test_individual_multilang_code_fixture() {
    let fixtures = get_all_golden_fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.archetype == ChunkingArchetype::MultiLanguageAstCode)
        .expect("MultiLanguageAstCode fixture must exist");

    let result = evaluate_golden_fixture(fixture).expect("Evaluation should succeed");
    assert!(
        result.passed,
        "MultiLanguageAstCode must pass: {:?}",
        result.error_message
    );
    assert!(result.metrics.bit_for_bit_repeatable);
    assert_eq!(result.metrics.retrieval_top1_accuracy, 1.0);
    assert!(result.metrics.chunk_count >= 3);
}

#[test]
fn test_individual_hierarchical_markdown_fixture() {
    let fixtures = get_all_golden_fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.archetype == ChunkingArchetype::HierarchicalMarkdownDocumentation)
        .expect("HierarchicalMarkdownDocumentation fixture must exist");

    let result = evaluate_golden_fixture(fixture).expect("Evaluation should succeed");
    assert!(
        result.passed,
        "HierarchicalMarkdownDocumentation must pass: {:?}",
        result.error_message
    );
    assert!(result.metrics.bit_for_bit_repeatable);
    assert_eq!(result.metrics.retrieval_top1_accuracy, 1.0);
}

#[test]
fn test_individual_tabular_data_header_retention() {
    let fixtures = get_all_golden_fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.archetype == ChunkingArchetype::TabularDataMatrix)
        .expect("TabularDataMatrix fixture must exist");

    let result = evaluate_golden_fixture(fixture).expect("Evaluation should succeed");
    assert!(
        result.passed,
        "TabularDataMatrix must pass: {:?}",
        result.error_message
    );
    assert!(result.metrics.chunk_count >= 2);
}

#[test]
fn test_individual_oversized_declaration_splitting() {
    let fixtures = get_all_golden_fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.archetype == ChunkingArchetype::OversizedDeclarationRoutines)
        .expect("OversizedDeclarationRoutines fixture must exist");

    let result = evaluate_golden_fixture(fixture).expect("Evaluation should succeed");
    assert!(
        result.passed,
        "OversizedDeclarationRoutines must pass: {:?}",
        result.error_message
    );
    assert!(result.metrics.chunk_count >= 2);
}

#[test]
fn test_individual_cross_reference_graph_fixture() {
    let fixtures = get_all_golden_fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.archetype == ChunkingArchetype::CrossReferenceGraphCorpus)
        .expect("CrossReferenceGraphCorpus fixture must exist");

    let result = evaluate_golden_fixture(fixture).expect("Evaluation should succeed");
    assert!(
        result.passed,
        "CrossReferenceGraphCorpus must pass: {:?}",
        result.error_message
    );
    assert!(result.metrics.cross_references_count > 0);
}
