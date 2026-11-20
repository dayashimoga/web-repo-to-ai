//! # Tabular Limits & DoS Protection Integration Tests
//!
//! Verifies row ceilings, cell size bounds, column limits, and graceful truncation
//! with mathematical completeness invariant accounting.

use web_repo_to_ai_cdm::{ExtractionStatus, SourceType};
use web_repo_to_ai_structured_extractor::{
    parse_tabular, StructuredError, StructuredExtractor, StructuredInput, StructuredLimits,
    StructuredOptions,
};

#[test]
fn test_tabular_graceful_row_truncation() {
    // Generate 50 rows of data
    let mut csv = String::from("id,item_name,value\n");
    for i in 1..=50 {
        csv.push_str(&format!("{i},Item_{i},{}.99\n", i * 10));
    }

    // Limit to 10 rows with graceful truncation enabled
    let limits = StructuredLimits::default()
        .with_max_tabular_rows(10)
        .with_truncate_on_row_limit(true);
    let options = StructuredOptions::default().with_limits(limits);

    let input = StructuredInput::from_str(&csv).with_filename("large_dataset.csv");
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction should succeed with partial truncation");

    assert_eq!(doc.source_type, SourceType::StructuredDocument);
    assert_eq!(doc.extraction_status, ExtractionStatus::Partial);

    // Completeness Report invariant: discovered == extracted + excluded
    let completeness = &doc.completeness;
    assert_eq!(completeness.discovered_count, 50);
    assert_eq!(completeness.extracted_count, 10);
    assert_eq!(completeness.excluded_count, 40);
    assert_eq!(
        completeness.discovered_count,
        completeness.extracted_count + completeness.excluded_count
    );

    // Data section contains the truncation note
    let data_sec = &doc.sections[1];
    assert!(data_sec
        .body
        .contains("*... and 40 more rows omitted (total: 50)*"));
}

#[test]
fn test_tabular_strict_row_limit_rejection() {
    let mut csv = String::from("id,val\n");
    for i in 1..=20 {
        csv.push_str(&format!("{i},test\n"));
    }

    // Limit to 5 rows with strict rejection (truncate_on_row_limit = false)
    let limits = StructuredLimits::default()
        .with_max_tabular_rows(5)
        .with_truncate_on_row_limit(false);
    let options = StructuredOptions::default().with_limits(limits);

    let err = parse_tabular(&csv, None, None, &options).expect_err("Should reject on row limit");
    match err {
        StructuredError::LimitExceeded(msg) => {
            assert!(msg.contains("Row count exceeds maximum tabular limit"));
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_tabular_max_cell_bytes_limit() {
    // Large 200 KB cell
    let huge_cell = "A".repeat(200 * 1024);
    let csv = format!("id,data\n1,{huge_cell}\n");

    let limits = StructuredLimits::default().with_max_cell_bytes(64 * 1024); // 64 KiB
    let options = StructuredOptions::default().with_limits(limits);

    let err = parse_tabular(&csv, None, None, &options).expect_err("Should reject oversized cell");
    match err {
        StructuredError::LimitExceeded(msg) => {
            assert!(msg.contains("Cell size exceeds maximum allowed limit"));
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_tabular_max_columns_limit() {
    // 50 columns
    let headers: Vec<String> = (1..=50).map(|i| format!("col_{i}")).collect();
    let row: Vec<String> = (1..=50).map(|i| format!("val_{i}")).collect();
    let csv = format!("{}\n{}\n", headers.join(","), row.join(","));

    // Limit to 10 columns
    let limits = StructuredLimits::default().with_max_tabular_columns(10);
    let options = StructuredOptions::default().with_limits(limits);

    let err =
        parse_tabular(&csv, None, None, &options).expect_err("Should reject oversized columns");
    match err {
        StructuredError::LimitExceeded(msg) => {
            assert!(msg.contains("Column count 50 exceeds maximum tabular limit of 10"));
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}
