//! # Tabular Data Extraction Integration Tests
//!
//! Tests RFC 4180 CSV parsing, TSV, semicolon-separated, pipe-separated,
//! delimiter sniffing, header detection, and column type inference.

use web_repo_to_ai_structured_extractor::{
    infer_column_type, parse_tabular, sniff_delimiter, sniff_has_headers, ColumnDataType,
    StructuredOptions, TabularDelimiter,
};

#[test]
fn test_rfc4180_csv_parsing_and_escapes() {
    let csv_content = "\
id,name,description,amount\r\n\
1,\"Simple Item\",\"Basic description\",10.50\r\n\
2,\"Complex, Item with Comma\",\"Description with \"\"escaped quotes\"\" inside\",25.00\r\n\
3,\"Multiline\r\nItem\",\"Line 1\nLine 2\",99.99\r\n";

    let options = StructuredOptions::default();
    let table = parse_tabular(
        csv_content,
        None,
        Some("products.csv".to_string()),
        &options,
    )
    .expect("Failed to parse RFC 4180 CSV");

    assert_eq!(table.delimiter, TabularDelimiter::Comma);
    assert!(table.has_headers);
    assert_eq!(table.headers, vec!["id", "name", "description", "amount"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.total_rows, 3);
    assert!(!table.truncated);

    // Row 1
    assert_eq!(table.rows[0][0], "1");
    assert_eq!(table.rows[0][1], "Simple Item");
    assert_eq!(table.rows[0][3], "10.50");

    // Row 2: comma and escaped quotes inside quoted strings
    assert_eq!(table.rows[1][1], "Complex, Item with Comma");
    assert_eq!(
        table.rows[1][2],
        "Description with \"escaped quotes\" inside"
    );

    // Row 3: multiline cell
    assert_eq!(table.rows[2][1], "Multiline\r\nItem");
    assert_eq!(table.rows[2][2], "Line 1\nLine 2");
}

#[test]
fn test_tsv_sniffing_and_parsing() {
    let tsv_content = "user_id\tusername\trole\tactive\n\
101\talice\tadmin\ttrue\n\
102\tbob\teditor\tfalse\n\
103\tcharlie\tviewer\ttrue\n";

    let detected_delim = sniff_delimiter(tsv_content);
    assert_eq!(detected_delim, TabularDelimiter::Tab);

    let options = StructuredOptions::default();
    let table = parse_tabular(tsv_content, None, Some("users.tsv".to_string()), &options)
        .expect("Failed to parse TSV");

    assert_eq!(table.delimiter, TabularDelimiter::Tab);
    assert!(table.has_headers);
    assert_eq!(table.headers, vec!["user_id", "username", "role", "active"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec!["101", "alice", "admin", "true"]);
    assert_eq!(table.rows[1], vec!["102", "bob", "editor", "false"]);
}

#[test]
fn test_semicolon_separated_european_format() {
    let semi_content = "Produkt;Menge;Preis;Gültig\n\
Kaffee;500;8.99;ja\n\
Tee;250;4.50;ja\n\
Zucker;1000;1.20;nein\n";

    let detected_delim = sniff_delimiter(semi_content);
    assert_eq!(detected_delim, TabularDelimiter::Semicolon);

    let options = StructuredOptions::default();
    let table = parse_tabular(
        semi_content,
        None,
        Some("produkte.csv".to_string()),
        &options,
    )
    .expect("Failed to parse semicolon CSV");

    assert_eq!(table.delimiter, TabularDelimiter::Semicolon);
    assert!(table.has_headers);
    assert_eq!(table.headers, vec!["Produkt", "Menge", "Preis", "Gültig"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec!["Kaffee", "500", "8.99", "ja"]);
}

#[test]
fn test_pipe_separated_values() {
    let pipe_content = "code|city|country|population\n\
SFO|San Francisco|USA|873965\n\
LHR|London|UK|8982000\n\
HND|Tokyo|Japan|13960000\n";

    let detected_delim = sniff_delimiter(pipe_content);
    assert_eq!(detected_delim, TabularDelimiter::Pipe);

    let options = StructuredOptions::default();
    let table = parse_tabular(pipe_content, None, Some("cities.psv".to_string()), &options)
        .expect("Failed to parse pipe delimited");

    assert_eq!(table.delimiter, TabularDelimiter::Pipe);
    assert!(table.has_headers);
    assert_eq!(table.headers, vec!["code", "city", "country", "population"]);
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec!["SFO", "San Francisco", "USA", "873965"]);
}

#[test]
fn test_header_detection_heuristics() {
    // 1. Definite headers: string names vs numeric data
    let with_headers = vec![
        vec!["id".to_string(), "count".to_string(), "score".to_string()],
        vec!["1".to_string(), "10".to_string(), "95.5".to_string()],
        vec!["2".to_string(), "20".to_string(), "88.0".to_string()],
    ];
    assert!(sniff_has_headers(&with_headers));

    // 2. Pure numerical data without headers
    let without_headers = vec![
        vec!["100".to_string(), "200".to_string(), "300".to_string()],
        vec!["400".to_string(), "500".to_string(), "600".to_string()],
        vec!["700".to_string(), "800".to_string(), "900".to_string()],
    ];
    assert!(!sniff_has_headers(&without_headers));
}

#[test]
fn test_column_type_inference() {
    assert_eq!(
        infer_column_type(&["1", "42", "-100", "0"]),
        ColumnDataType::Integer
    );
    assert_eq!(
        infer_column_type(&["3.14", "-0.05", "100.0", "1e-4"]),
        ColumnDataType::Float
    );
    assert_eq!(
        infer_column_type(&["true", "false", "yes", "no"]),
        ColumnDataType::Boolean
    );
    assert_eq!(
        infer_column_type(&["2026-11-20", "2025-01-01", "2024-12-31"]),
        ColumnDataType::DateTime
    );
    assert_eq!(
        infer_column_type(&["hello", "123", "world"]),
        ColumnDataType::String
    );
    assert_eq!(infer_column_type(&[]), ColumnDataType::String);
}

#[test]
fn test_column_statistics_profiling() {
    let csv = "\
id,name,score,registered\n\
1,Alice,98.5,2026-01-10\n\
2,Bob,85.0,2026-02-15\n\
3,Charlie,,\n\
4,,92.0,2026-03-20\n";

    let options = StructuredOptions::default();
    let table = parse_tabular(csv, None, None, &options).expect("Failed to parse");

    assert_eq!(table.columns.len(), 4);

    // Column 0: id (Integer)
    assert_eq!(table.columns[0].name, "id");
    assert_eq!(table.columns[0].data_type, ColumnDataType::Integer);
    assert_eq!(table.columns[0].non_empty_count, 4);
    assert_eq!(table.columns[0].null_count, 0);

    // Column 1: name (String, 1 null)
    assert_eq!(table.columns[1].name, "name");
    assert_eq!(table.columns[1].data_type, ColumnDataType::String);
    assert_eq!(table.columns[1].non_empty_count, 3);
    assert_eq!(table.columns[1].null_count, 1);

    // Column 2: score (Float, 1 null)
    assert_eq!(table.columns[2].name, "score");
    assert_eq!(table.columns[2].data_type, ColumnDataType::Float);
    assert_eq!(table.columns[2].non_empty_count, 3);
    assert_eq!(table.columns[2].null_count, 1);

    // Column 3: registered (DateTime, 1 null)
    assert_eq!(table.columns[3].name, "registered");
    assert_eq!(table.columns[3].data_type, ColumnDataType::DateTime);
    assert_eq!(table.columns[3].non_empty_count, 3);
    assert_eq!(table.columns[3].null_count, 1);

    // Overall data density: 13 populated cells out of 16 = 0.8125
    assert!((table.data_density() - 0.8125).abs() < 1e-4);
}
