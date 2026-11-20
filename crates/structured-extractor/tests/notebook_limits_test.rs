//! # Integration Tests: Jupyter Notebook Limits and Resource Bounds
//!
//! Validates cell limits, graceful truncation vs strict error rejection, output byte
//! bounds, empty cell resiliency, and malformed notebook schema error handling.

use web_repo_to_ai_cdm::model::ExtractionStatus;
use web_repo_to_ai_cdm::validator::CdmValidator;
use web_repo_to_ai_structured_extractor::{
    parse_notebook, StructuredError, StructuredExtractor, StructuredInput, StructuredLimits,
    StructuredOptions,
};

fn generate_synthetic_notebook_json(cell_count: usize) -> String {
    let mut cells = Vec::new();
    for i in 0..cell_count {
        cells.push(format!(
            r#"{{
              "cell_type": "code",
              "execution_count": {},
              "metadata": {{}},
              "source": "x_{} = {} * 2\nprint(x_{})",
              "outputs": [
                {{
                  "output_type": "stream",
                  "name": "stdout",
                  "text": ["{}\n"]
                }}
              ]
            }}"#,
            i + 1,
            i,
            i,
            i,
            i * 2
        ));
    }

    format!(
        r#"{{
          "cells": [{}],
          "metadata": {{
            "language_info": {{ "name": "python" }},
            "kernelspec": {{ "name": "python3" }}
          }},
          "nbformat": 4,
          "nbformat_minor": 5
        }}"#,
        cells.join(",")
    )
}

#[test]
fn test_notebook_graceful_cell_truncation() {
    let json = generate_synthetic_notebook_json(80);
    let limits = StructuredLimits::default()
        .with_max_notebook_cells(20)
        .with_truncate_on_cell_limit(true);
    let options = StructuredOptions::default().with_limits(limits);

    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(&json).with_filename("large.ipynb");
    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract truncated notebook");

    assert_eq!(doc.extraction_status, ExtractionStatus::Partial);
    assert_eq!(doc.completeness.discovered_count, 80);
    assert_eq!(doc.completeness.extracted_count, 20);
    assert_eq!(doc.completeness.excluded_count, 60);
    assert_eq!(
        doc.completeness.discovered_count,
        doc.completeness.extracted_count + doc.completeness.excluded_count
    );

    let val_report = CdmValidator::new().validate(&doc);
    assert!(
        val_report.is_valid(),
        "Validation errors: {:?}",
        val_report.issues
    );
}

#[test]
fn test_notebook_strict_cell_limit_rejection() {
    let json = generate_synthetic_notebook_json(50);
    let limits = StructuredLimits::default()
        .with_max_notebook_cells(20)
        .with_truncate_on_cell_limit(false);
    let options = StructuredOptions::default().with_limits(limits.clone());

    let res = parse_notebook(&json, &options, &limits);
    match res {
        Err(StructuredError::LimitExceeded(msg)) => {
            assert!(msg.contains("cell count 50 exceeds maximum allowed limit of 20"));
        }
        other => panic!("Expected LimitExceeded error, got: {other:?}"),
    }
}

#[test]
fn test_notebook_output_byte_limit() {
    // Large output exceeding max_output_bytes
    let large_payload = "A".repeat(50_000);
    let json = format!(
        r#"{{
          "cells": [
            {{
              "cell_type": "code",
              "execution_count": 1,
              "metadata": {{}},
              "source": "print('large')",
              "outputs": [
                {{
                  "output_type": "stream",
                  "name": "stdout",
                  "text": "{large_payload}"
                }}
              ]
            }}
          ],
          "metadata": {{}},
          "nbformat": 4,
          "nbformat_minor": 0
        }}"#
    );

    let limits = StructuredLimits::default().with_max_output_bytes(10_000);
    let options = StructuredOptions::default().with_limits(limits.clone());
    let nb = parse_notebook(&json, &options, &limits).expect("parse notebook");

    let out_text = nb.cells[0].outputs[0].text.as_ref().unwrap();
    assert!(out_text.len() <= 10_000 + 100);
    assert!(out_text.contains("[Output truncated by byte limit]"));
}

#[test]
fn test_empty_cells_resilience() {
    let json = r#"{
      "cells": [
        {
          "cell_type": "markdown",
          "metadata": {},
          "source": ""
        },
        {
          "cell_type": "code",
          "metadata": {},
          "source": []
        },
        {
          "cell_type": "raw",
          "metadata": {},
          "source": ""
        }
      ],
      "metadata": {},
      "nbformat": 4,
      "nbformat_minor": 0
    }"#;

    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(json).with_filename("empty_cells.ipynb");
    let options = StructuredOptions::default();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract empty cells");

    assert_eq!(doc.sections.len(), 4); // 1 summary + 3 cells
    let val_report = CdmValidator::new().validate(&doc);
    assert!(
        val_report.is_valid(),
        "Validation failed: {:?}",
        val_report.issues
    );
}

#[test]
fn test_malformed_notebook_rejections() {
    let options = StructuredOptions::default();
    let limits = StructuredLimits::default();

    // 1. Root is array
    let err1 = parse_notebook("[]", &options, &limits).unwrap_err();
    assert!(matches!(err1, StructuredError::InvalidNotebook(_)));

    // 2. Missing cells array
    let err2 = parse_notebook(r#"{"metadata": {}, "nbformat": 4}"#, &options, &limits).unwrap_err();
    assert!(matches!(err2, StructuredError::InvalidNotebook(_)));

    // 3. Invalid JSON syntax
    let err3 = parse_notebook(r#"{"cells": [ unquoted ]}"#, &options, &limits).unwrap_err();
    assert!(matches!(err3, StructuredError::InvalidNotebook(_)));
}
