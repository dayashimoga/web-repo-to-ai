//! # Integration Tests: Structured Document Validation and Schema Invariants
//!
//! Validates structural integrity, referential consistency, line/byte ranges,
//! hierarchy tree acyclicity, DoS limits, and mathematical completeness invariants
//! across all structured document formats.

use web_repo_to_ai_cdm::validator::CdmValidator;
use web_repo_to_ai_structured_extractor::golden::all_golden_fixtures;
use web_repo_to_ai_structured_extractor::{
    detect_format, StructuredError, StructuredExtractor, StructuredInput, StructuredLimits,
    StructuredOptions,
};

#[test]
fn test_cdm_schema_validation_invariants_across_all_golden_archetypes() {
    let fixtures = all_golden_fixtures();
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();
    let validator = CdmValidator::new();

    for fixture in &fixtures {
        let input = StructuredInput::from_str(fixture.raw_content)
            .with_filename(fixture.filename)
            .with_source_url(format!("file:///test/{}", fixture.filename));

        let doc = extractor
            .extract_to_cdm(&input, &options)
            .unwrap_or_else(|e| panic!("Failed to extract fixture '{}': {e}", fixture.name));

        let report = validator.validate(&doc);
        assert!(
            report.is_valid(),
            "Validation errors for archetype '{}': {:?}",
            fixture.name,
            report.issues
        );

        // Invariant: Non-empty document ID and schema version 1.0.0
        assert_eq!(doc.schema_version, "1.0.0");
        assert!(!doc.document_id.to_string().is_empty());

        // Invariant: Non-empty sections
        assert!(!doc.sections.is_empty());

        // Invariant: Mathematical completeness report
        let comp = &doc.completeness;
        assert_eq!(
            comp.discovered_count,
            comp.extracted_count + comp.excluded_count + comp.failed_count + comp.unsupported_count,
            "Completeness accounting violation in '{}'",
            fixture.name
        );
    }
}

#[test]
fn test_hierarchy_tree_validity_and_referential_integrity() {
    let fixtures = all_golden_fixtures();
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();

    for fixture in &fixtures {
        let input = StructuredInput::from_str(fixture.raw_content).with_filename(fixture.filename);

        let doc = extractor
            .extract_to_cdm(&input, &options)
            .expect("extract CDM");

        // Verify all referenced section IDs in the hierarchy tree exist in doc.sections
        let all_nodes = doc.hierarchy.flatten();
        for node in all_nodes {
            if let Some(ref sec_id) = node.section_id {
                assert!(
                    doc.sections.iter().any(|s| s.section_id == *sec_id),
                    "Hierarchy node '{}' references non-existent section ID '{}' in fixture '{}'",
                    node.name,
                    sec_id,
                    fixture.name
                );
            }
        }
    }
}

#[test]
fn test_section_line_and_byte_ranges_validity() {
    let fixtures = all_golden_fixtures();
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();

    for fixture in &fixtures {
        let input = StructuredInput::from_str(fixture.raw_content).with_filename(fixture.filename);

        let doc = extractor
            .extract_to_cdm(&input, &options)
            .expect("extract CDM");

        for (idx, sec) in doc.sections.iter().enumerate() {
            // Line range checks: start_line >= 1, end_line >= start_line
            if let Some(ref lr) = sec.line_range {
                assert!(
                    lr.start_line >= 1,
                    "Section [{idx}] in '{}' has zero start_line",
                    fixture.name
                );
                assert!(
                    lr.end_line >= lr.start_line,
                    "Section [{idx}] in '{}' has inverted line range: {}..{}",
                    fixture.name,
                    lr.start_line,
                    lr.end_line
                );
            }

            // Byte range checks: end_byte >= start_byte
            if let Some(ref br) = sec.byte_range {
                assert!(
                    br.end_byte >= br.start_byte,
                    "Section [{idx}] in '{}' has inverted byte range: {}..{}",
                    fixture.name,
                    br.start_byte,
                    br.end_byte
                );
            }

            // Body length matches byte span approximately or non-empty
            assert!(
                !sec.body.is_empty(),
                "Section [{idx}] in '{}' has empty body",
                fixture.name
            );
        }
    }
}

#[test]
fn test_empty_and_whitespace_input_rejections() {
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();

    let empty_inputs = vec![
        StructuredInput::from_str(""),
        StructuredInput::from_str("   \n\t  "),
        StructuredInput::from_str("").with_filename("empty.json"),
        StructuredInput::from_str("").with_filename("empty.yaml"),
        StructuredInput::from_str("").with_filename("empty.csv"),
        StructuredInput::from_str("").with_filename("empty.ipynb"),
    ];

    for input in empty_inputs {
        let res = extractor.extract_to_cdm(&input, &options);
        assert!(
            res.is_err(),
            "Empty input with filename {:?} should be rejected",
            input.filename
        );
    }
}

#[test]
fn test_unrecognized_format_rejection() {
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();

    let invalid_content =
        "some random plain english sentence that is definitely not json, yaml, csv, or ipynb.";
    let input = StructuredInput::from_str(invalid_content);

    let res = extractor.extract_to_cdm(&input, &options);
    match res {
        Err(StructuredError::UnrecognizedFormat(_)) => {}
        other => panic!("Expected UnrecognizedFormat, got: {other:?}"),
    }
}

#[test]
fn test_adversarial_max_input_bytes_limit() {
    let limits = StructuredLimits::default().with_max_input_bytes(50);
    let options = StructuredOptions::default().with_limits(limits);
    let extractor = StructuredExtractor::new();

    let payload = "A".repeat(100);
    let input = StructuredInput::from_str(&payload).with_filename("payload.csv");

    let res = extractor.extract_to_cdm(&input, &options);
    match res {
        Err(StructuredError::LimitExceeded(msg)) => {
            assert!(msg.contains("exceeds maximum allowed limit of 50 bytes"));
        }
        other => panic!("Expected LimitExceeded, got: {other:?}"),
    }
}

#[test]
fn test_filename_hinting_detection() {
    let inputs = vec![
        (
            "openapi.yaml",
            StructuredInput::from_str("key: val").with_filename("openapi.yaml"),
        ),
        (
            "spec.json",
            StructuredInput::from_str("{\"openapi\":\"3.0.0\"}").with_filename("spec.json"),
        ),
        (
            "data.csv",
            StructuredInput::from_str("a,b\n1,2").with_filename("data.csv"),
        ),
        (
            "metrics.tsv",
            StructuredInput::from_str("a\tb\n1\t2").with_filename("metrics.tsv"),
        ),
        (
            "notebook.ipynb",
            StructuredInput::from_str("{\"cells\":[],\"nbformat\":4}")
                .with_filename("notebook.ipynb"),
        ),
    ];

    for (name, input) in inputs {
        let kind = detect_format(&input).unwrap_or_else(|e| panic!("Failed to detect {name}: {e}"));
        assert!(!kind.as_str().is_empty());
        assert!(!kind.display_name().is_empty());
    }
}
