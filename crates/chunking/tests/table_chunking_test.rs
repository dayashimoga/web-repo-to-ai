//! # Tabular Document Chunking Integration Tests
//!
//! Validates tabular data splitting with header row retention across all chunks.

use web_repo_to_ai_cdm::{CdmSection, LineRange, SectionKind};
use web_repo_to_ai_chunking::{
    chunk_structured_section, ChunkKind, ChunkingConfig, ChunkingReport, TokenCounter,
};
use web_repo_to_ai_core::DocumentId;

#[test]
fn test_tabular_data_header_retention_across_chunks() {
    let mut table_text =
        String::from("| Order ID | Customer Name | Amount | Status | Timestamp |\n");
    table_text.push_str("| --- | --- | --- | --- | --- |\n");

    for i in 1..=50 {
        table_text.push_str(&format!(
            "| ORD-{i:04} | Customer_{i:03} | ${i}.99 | Completed | 2026-11-20T12:{i:02}:00Z |\n"
        ));
    }

    let section = CdmSection {
        section_id: "sec_table_1".to_string(),
        parent_id: None,
        title: Some("orders.csv".to_string()),
        kind: SectionKind::Table,
        original_path: Some("data/orders.csv".to_string()),
        source_url: "https://example.com/data/orders.csv".to_string(),
        line_range: Some(LineRange::new(1, 52)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(&table_text),
        body: table_text,
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(180);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks =
        chunk_structured_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
            .expect("table chunking succeeds");

    assert!(chunks.len() > 1, "table must be split into multiple chunks");
    assert_eq!(report.table_chunks_count, chunks.len());

    // CRITICAL: Every chunk must contain the column header row and separator!
    for chunk in &chunks {
        assert_eq!(chunk.kind, ChunkKind::Table);
        assert!(
            chunk
                .raw_content
                .contains("| Order ID | Customer Name | Amount | Status | Timestamp |"),
            "table chunk MUST retain the header row"
        );
        assert!(
            chunk
                .raw_content
                .contains("| --- | --- | --- | --- | --- |"),
            "table chunk MUST retain the table separator row"
        );
        assert_eq!(
            chunk.metadata.get("header_retained").map(String::as_str),
            Some("true")
        );
        assert!(chunk.metadata.contains_key("rows_in_chunk"));
    }
}

#[test]
fn test_small_table_single_chunk() {
    let table_text = "| Key | Value |\n| --- | --- |\n| Port | 8080 |\n| Host | localhost |\n";

    let section = CdmSection {
        section_id: "sec_tbl_sm".to_string(),
        parent_id: None,
        title: Some("config.csv".to_string()),
        kind: SectionKind::Table,
        original_path: Some("config.csv".to_string()),
        source_url: "https://example.com/config.csv".to_string(),
        line_range: Some(LineRange::new(1, 4)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(table_text),
        body: table_text.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default().with_target_tokens(100);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks =
        chunk_structured_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
            .expect("table chunking succeeds");

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].kind, ChunkKind::Table);
    assert!(chunks[0].content.contains("| Key | Value |"));
    assert!(chunks[0].content.contains("| Port | 8080 |"));
}
