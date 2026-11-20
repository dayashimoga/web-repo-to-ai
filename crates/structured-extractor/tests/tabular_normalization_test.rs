//! # Tabular Normalization & CDM Synthesis Integration Tests
//!
//! Tests ragged row alignment, null normalization, whitespace trimming,
//! and Canonical Context Document Model (CDM) synthesis.

use web_repo_to_ai_cdm::{ExtractionStatus, SectionKind, SourceType};
use web_repo_to_ai_structured_extractor::{
    parse_tabular, StructuredExtractor, StructuredInput, StructuredOptions,
};

#[test]
fn test_ragged_rows_padding_and_alignment() {
    let ragged_csv = "\
col1,col2,col3\n\
val1,val2\n\
val3,val4,val5,extra_val\n\
val6\n";

    let options = StructuredOptions::default();
    let table = parse_tabular(ragged_csv, None, None, &options).expect("Failed to parse");

    // Highest column count is 4 (due to extra_val)
    assert_eq!(table.headers.len(), 4);
    assert_eq!(table.headers[3], "column_4");

    // All rows are padded to 4 columns
    for row in &table.rows {
        assert_eq!(row.len(), 4);
    }

    assert_eq!(table.rows[0], vec!["val1", "val2", "", ""]);
    assert_eq!(table.rows[1], vec!["val3", "val4", "val5", "extra_val"]);
    assert_eq!(table.rows[2], vec!["val6", "", "", ""]);
}

#[test]
fn test_null_value_normalization() {
    let null_variations_csv = "\
id,val_a,val_b,val_c,val_d\n\
1,NA,N/A,null,NULL\n\
2,NaN,-,--,#N/A\n\
3,None,none,<null>,valid_data\n";

    let options = StructuredOptions::default();
    let table = parse_tabular(null_variations_csv, None, None, &options)
        .expect("Failed to parse null variations");

    // Rows 1 and 2: all values (except id) normalized to empty string
    assert_eq!(table.rows[0], vec!["1", "", "", "", ""]);
    assert_eq!(table.rows[1], vec!["2", "", "", "", ""]);

    // Row 3: first 3 normalized to empty, last remains valid
    assert_eq!(table.rows[2], vec!["3", "", "", "", "valid_data"]);
}

#[test]
fn test_whitespace_trimming_toggle() {
    let spaced_csv = "  col_a  ,  col_b  \n  hello  ,  world  \n";

    // 1. With trimming enabled (default)
    let options_trimmed = StructuredOptions::default();
    let table_trimmed = parse_tabular(spaced_csv, None, None, &options_trimmed).unwrap();
    assert_eq!(table_trimmed.headers, vec!["col_a", "col_b"]);
    assert_eq!(table_trimmed.rows[0], vec!["hello", "world"]);

    // 2. With trimming disabled
    let options_raw = StructuredOptions::default().with_tabular_trim_whitespace(false);
    let table_raw = parse_tabular(spaced_csv, None, None, &options_raw).unwrap();
    assert_eq!(table_raw.headers, vec!["col_a", "col_b"]);
    assert_eq!(table_raw.rows[0], vec!["  hello  ", "  world  "]);
}

#[test]
fn test_tabular_cdm_synthesis_and_sections() {
    let csv = "\
transaction_id,customer_name,amount,status,created_at\n\
tx_001,Alice Smith,150.00,completed,2026-11-20\n\
tx_002,Bob Jones,75.50,pending,2026-11-20\n\
tx_003,Charlie Brown,200.00,completed,2026-11-20\n";

    let input = StructuredInput::from_str(csv).with_filename("transactions.csv");
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction to CDM failed");

    assert_eq!(doc.source_type, SourceType::StructuredDocument);
    assert_eq!(doc.extraction_status, ExtractionStatus::Completed);
    assert_eq!(doc.metadata.title, Some("transactions.csv".to_string()));

    // Check extra metadata attributes
    let extra = &doc.metadata.extra;
    assert_eq!(extra.get("tabular_total_rows"), Some(&"3".to_string()));
    assert_eq!(extra.get("tabular_extracted_rows"), Some(&"3".to_string()));
    assert_eq!(extra.get("tabular_columns"), Some(&"5".to_string()));
    assert_eq!(extra.get("tabular_has_header"), Some(&"true".to_string()));
    assert_eq!(extra.get("tabular_truncated"), Some(&"false".to_string()));

    // Check sections: should have Summary & Schema (Prose) and Data Table (Table)
    assert_eq!(doc.sections.len(), 2);

    let summary_sec = &doc.sections[0];
    assert_eq!(summary_sec.kind, SectionKind::Prose);
    assert!(summary_sec.body.contains("# transactions.csv Overview"));
    assert!(summary_sec.body.contains("transaction_id"));
    assert!(summary_sec.body.contains("`datetime`"));

    let data_sec = &doc.sections[1];
    assert_eq!(data_sec.kind, SectionKind::Table);
    assert!(data_sec
        .body
        .contains("| transaction_id | customer_name | amount | status | created_at |"));
    assert!(data_sec
        .body
        .contains("| tx_001 | Alice Smith | 150.00 | completed | 2026-11-20 |"));

    // Check CompletenessReport
    let completeness = &doc.completeness;
    assert_eq!(completeness.discovered_count, 3);
    assert_eq!(completeness.extracted_count, 3);
    assert_eq!(completeness.excluded_count, 0);
    assert_eq!(
        completeness.discovered_count,
        completeness.extracted_count + completeness.excluded_count
    );
}
