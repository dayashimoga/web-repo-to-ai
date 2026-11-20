//! # PDF Golden Accuracy & Fidelity Validation Test Suite (Subphase 10.1.4)
//!
//! Validates extraction accuracy, layout reconstruction, reading order, table precision,
//! scanned/encryption heuristics, and resource bounds across the golden PDF corpus:
//!
//! 1. Academic Multi-Column Paper (two-column layout, abstract, reading order, references)
//! 2. Financial Quarterly Report (multi-column/row table matrix, captions, numerical data)
//! 3. Technical API Specification (hierarchical outline bookmarks, code blocks, parameter tables)
//! 4. Hybrid Digital/Scanned Document (raster image XObjects, text density heuristics, OCR recommendations)
//! 5. Encrypted Permission-Restricted PDF (ISO 32000-1 bitmask decoding, permission enforcement)
//! 6. Adversarial Resource Limits (DoS protection, graceful page truncation, completeness invariant)
//! 7. Spatial Noise Sorting (out-of-order operator stream 2D coordinate sorting)

use web_repo_to_ai_cdm::{ExtractionStatus, SectionKind};
use web_repo_to_ai_pdf_extractor::error::PdfError;
use web_repo_to_ai_pdf_extractor::extractor::{PdfExtractionOptions, PdfExtractor};
use web_repo_to_ai_pdf_extractor::golden::{
    all_golden_fixtures, build_academic_multicolumn_fixture, build_adversarial_limits_fixture,
    build_api_specification_fixture, build_encrypted_restricted_fixture,
    build_financial_report_fixture, build_hybrid_scanned_fixture, build_spatial_noise_fixture,
    evaluate_golden_fixture, run_all_golden_benchmarks, PdfArchetype,
};
use web_repo_to_ai_pdf_extractor::limits::PdfLimits;

#[test]
fn test_golden_corpus_benchmark_all_passed() {
    let report =
        run_all_golden_benchmarks(None).expect("all golden benchmarks must run without crash");

    println!("{}", report.to_markdown_summary());

    assert_eq!(report.benchmark_id, "golden-pdf-corpus-v1");
    assert_eq!(report.results.len(), 7, "Must benchmark all 7 archetypes");
    assert!(
        report.overall_text_recall >= 0.98,
        "Overall text recall must be >= 98%, got {:.2}%",
        report.overall_text_recall * 100.0
    );
    assert!(
        report.overall_reading_order_fidelity >= 0.98,
        "Overall reading order fidelity must be >= 98%, got {:.2}%",
        report.overall_reading_order_fidelity * 100.0
    );
    assert!(
        report.overall_table_accuracy >= 0.98,
        "Overall table accuracy must be >= 98%, got {:.2}%",
        report.overall_table_accuracy * 100.0
    );
    assert!(
        report.all_passed,
        "All golden archetype benchmarks must pass their individual fidelity thresholds:\n{}",
        report.to_markdown_summary()
    );
}

#[test]
fn test_golden_archetype_1_academic_multicolumn_reading_order() {
    let fixture = build_academic_multicolumn_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::AcademicMultiColumn);
    assert_eq!(fixture.expected_pages, 2);

    let result = evaluate_golden_fixture(&fixture, None).expect("evaluation succeeds");
    assert!(
        result.metrics.passed,
        "Fidelity notes: {:?}",
        result.metrics.notes
    );
    assert_eq!(result.extracted_pages, 2);
    assert_eq!(result.metrics.text_recall, 1.0);
    assert_eq!(result.metrics.reading_order_fidelity, 1.0);

    // Extract low-level CDM and verify section kinds and reading order
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    assert_eq!(cdm.sections.len(), 2, "2 pages -> 2 prose sections");
    let p1_body = &cdm.sections[0].body;

    // Verify left column appears before right column in page 1 body
    let intro_pos = p1_body
        .find("1. Introduction")
        .expect("Introduction present");
    let task_pos = p1_body
        .find("2. Task Scheduling")
        .expect("Task Scheduling present");
    let exec_pos = p1_body
        .find("3. Execution Semantics")
        .expect("Execution Semantics present");
    let exp_pos = p1_body
        .find("4. Experimental Results")
        .expect("Experimental Results present");

    assert!(intro_pos < task_pos, "Intro must precede Task Scheduling");
    assert!(
        task_pos < exec_pos,
        "Left column must precede Right column (Task Scheduling < Execution Semantics)"
    );
    assert!(
        exec_pos < exp_pos,
        "Execution Semantics must precede Experimental Results"
    );

    // Verify running header and footer are separated from content body
    assert!(
        !p1_body.contains("ACM Transactions on Computer Systems"),
        "Running page header should be filtered from body text"
    );
    assert!(
        !p1_body.contains("ACM Copyright 2026"),
        "Running page footer should be filtered from body text"
    );
}

#[test]
fn test_golden_archetype_2_financial_report_table_extraction() {
    let fixture = build_financial_report_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::FinancialReport);
    assert_eq!(fixture.expected_tables, 1);

    let result = evaluate_golden_fixture(&fixture, None).expect("evaluation succeeds");
    assert!(
        result.metrics.passed,
        "Fidelity notes: {:?}",
        result.metrics.notes
    );
    assert_eq!(result.extracted_tables, 1);
    assert_eq!(result.metrics.table_accuracy, 1.0);

    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    // Must have at least 1 prose section and 1 table section
    let table_sec = cdm
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Table)
        .expect("Table section must be present");

    assert!(table_sec
        .title
        .as_deref()
        .unwrap_or("")
        .contains("Table 1: Operational Segment Performance"));
    assert_eq!(
        table_sec.attributes.get("table_columns"),
        Some(&"4".to_string())
    );
    assert_eq!(
        table_sec.attributes.get("table_rows"),
        Some(&"5".to_string())
    ); // 1 header + 4 data rows

    // Check GFM table pipe formatting and cell values
    let table_body = &table_sec.body;
    assert!(table_body.contains("| Segment | Q1 ($M) | Q2 ($M) | Growth |"));
    assert!(table_body.contains("|---|---|---|---|"));
    assert!(table_body.contains("| Cloud Infrastructure | 145.2 | 182.6 | +25.8% |"));
    assert!(table_body.contains("| Developer Tools | 84.0 | 96.5 | +14.9% |"));
    assert!(table_body.contains("| Enterprise Support | 52.3 | 55.1 | +5.4% |"));
    assert!(table_body.contains("| Hardware Solutions | 31.0 | 28.4 | -8.4% |"));
}

#[test]
fn test_golden_archetype_3_api_specification_bookmarks_and_code() {
    let fixture = build_api_specification_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::ApiSpecification);

    let result = evaluate_golden_fixture(&fixture, None).expect("evaluation succeeds");
    assert!(
        result.metrics.passed,
        "Fidelity notes: {:?}",
        result.metrics.notes
    );

    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    // Verify hierarchy tree generated from outline bookmarks
    let root_node = &cdm.hierarchy;
    assert_eq!(root_node.name, "WebRepoToAI REST API Reference");
    assert_eq!(root_node.children.len(), 2);
    assert_eq!(root_node.children[0].name, "API Overview");
    assert_eq!(root_node.children[1].name, "POST /api/v1/jobs");

    // Verify code block preservation
    let full_text = cdm
        .sections
        .iter()
        .map(|s| s.body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    assert!(full_text.contains("curl -X POST https://api.example.com/api/v1/jobs"));

    // Verify parameters table detected
    let table_sec = cdm
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Table)
        .expect("Parameters table section must exist");
    assert!(table_sec.body.contains("| url | string | true |"));
    assert!(table_sec.body.contains("| format | string | false |"));
}

#[test]
fn test_golden_archetype_4_hybrid_scanned_ocr_recommendation() {
    let fixture = build_hybrid_scanned_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::HybridScanned);

    let result = evaluate_golden_fixture(&fixture, None).expect("evaluation succeeds");
    assert!(
        result.metrics.passed,
        "Fidelity notes: {:?}",
        result.metrics.notes
    );
    assert_eq!(result.extracted_pages, 3);
    assert_eq!(result.metrics.scanned_classification_accuracy, 1.0);

    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    // Document-level scanned metadata
    assert_eq!(
        cdm.metadata.extra.get("is_scanned"),
        Some(&"true".to_string())
    );
    assert_eq!(
        cdm.metadata.extra.get("ocr_recommended"),
        Some(&"true".to_string())
    );
    assert_eq!(
        cdm.metadata.extra.get("scanned_page_count"),
        Some(&"2".to_string())
    );

    // Page-level section verification
    // Page 1 is digital
    let p1_sec = cdm
        .sections
        .iter()
        .find(|s| s.section_id == "page_1")
        .unwrap();
    assert_eq!(
        p1_sec.attributes.get("is_scanned"),
        Some(&"false".to_string())
    );

    // Page 2 is scanned bitmap
    let p2_sec = cdm
        .sections
        .iter()
        .find(|s| s.section_id == "page_2")
        .unwrap();
    assert_eq!(
        p2_sec.attributes.get("is_scanned"),
        Some(&"true".to_string())
    );
    assert_eq!(
        p2_sec.attributes.get("scanned_confidence"),
        Some(&"1.0000".to_string())
    );

    // Page 3 is sparse annotated scanned
    let p3_sec = cdm
        .sections
        .iter()
        .find(|s| s.section_id == "page_3")
        .unwrap();
    assert_eq!(
        p3_sec.attributes.get("is_scanned"),
        Some(&"true".to_string())
    );
}

#[test]
fn test_golden_archetype_5_encrypted_permission_enforcement() {
    let fixture = build_encrypted_restricted_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::EncryptedRestricted);

    // 1. When enforce_permissions = true, extraction must be rejected with PermissionDenied
    let strict_opts = PdfExtractionOptions {
        enforce_permissions: true,
        ..Default::default()
    };
    let strict_extractor = PdfExtractor::with_options(strict_opts);
    let err = strict_extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect_err("Strict extractor must reject document with can_copy=false");

    match err {
        PdfError::PermissionDenied { permissions } => {
            assert!(!permissions.can_copy, "Copy permission must be false");
        }
        other => panic!("Expected PermissionDenied, got {:?}", other),
    }

    // 2. When enforce_permissions = false and no password, returns EncryptedDocument / PasswordRequired
    let lenient_opts = PdfExtractionOptions {
        enforce_permissions: false,
        ..Default::default()
    };
    let lenient_extractor = PdfExtractor::with_options(lenient_opts);
    let err = lenient_extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect_err(
            "Lenient extraction without password must require password / return EncryptedDocument",
        );
    assert!(matches!(
        err,
        PdfError::EncryptedDocument | PdfError::PasswordRequired { .. }
    ));
}

#[test]
fn test_golden_archetype_6_adversarial_limits_graceful_truncation() {
    let fixture = build_adversarial_limits_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::AdversarialLimits);
    assert_eq!(fixture.expected_pages, 10);

    let opts = PdfExtractionOptions {
        limits: PdfLimits {
            max_page_count: 3,
            truncate_on_page_limit: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let extractor = PdfExtractor::with_options(opts);
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("Graceful truncation extraction succeeds");

    assert_eq!(
        cdm.sections.len(),
        3,
        "Extracted pages must be truncated at exactly 3"
    );
    assert_eq!(cdm.extraction_status, ExtractionStatus::Partial);
    assert_eq!(
        cdm.metadata.extra.get("truncated_by_limit"),
        Some(&"true".to_string())
    );

    // Verify mathematical completeness invariant: discovered = extracted + excluded
    let comp = &cdm.completeness;
    assert_eq!(comp.discovered_count, 10);
    assert_eq!(comp.extracted_count, 3);
    assert_eq!(comp.excluded_count, 7);
    assert_eq!(
        comp.discovered_count,
        comp.extracted_count + comp.excluded_count,
        "Mathematical invariant discovered == extracted + excluded must strictly hold"
    );
}

#[test]
fn test_golden_archetype_7_spatial_noise_sorting() {
    let fixture = build_spatial_noise_fixture();
    assert_eq!(fixture.archetype, PdfArchetype::SpatialNoise);

    let result = evaluate_golden_fixture(&fixture, None).expect("evaluation succeeds");
    assert!(
        result.metrics.passed,
        "Fidelity notes: {:?}",
        result.metrics.notes
    );
    assert_eq!(result.metrics.reading_order_fidelity, 1.0);

    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let text = &cdm.sections[0].body;

    // Despite stream emitting Step 4 first and Step 1 last, spatial sorting must place Step 1 before Step 3 before Step 4
    let step1_pos = text
        .find("Step 1: System Boot Sequence")
        .expect("Step 1 found");
    let step3_pos = text
        .find("Step 3: Mid-execution checkpoint verification")
        .expect("Step 3 found");
    let step4_pos = text
        .find("Step 4: Concluding observations and shutdown sequence")
        .expect("Step 4 found");

    assert!(step1_pos < step3_pos, "Step 1 must appear before Step 3");
    assert!(step3_pos < step4_pos, "Step 3 must appear before Step 4");
}

#[test]
fn test_all_fixtures_collection() {
    let fixtures = all_golden_fixtures();
    assert_eq!(fixtures.len(), 7);

    for fixture in &fixtures {
        assert!(!fixture.name.is_empty());
        assert!(!fixture.description.is_empty());
        assert!(!fixture.pdf_bytes.is_empty());
        assert!(fixture.expected_pages > 0);
    }
}
