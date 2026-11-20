//! # Sliding Window Chunking Integration Tests
//!
//! Validates fixed/rolling window chunking with configurable overlap.

use web_repo_to_ai_cdm::{CdmSection, SectionKind};
use web_repo_to_ai_chunking::{
    chunk_sliding_window, ChunkKind, ChunkingConfig, ChunkingReport, TokenCounter,
};
use web_repo_to_ai_core::DocumentId;

#[test]
fn test_sliding_window_with_overlap_continuity() {
    let mut body = String::new();
    for i in 1..=40 {
        body.push_str(&format!(
            "Line {i:02}: This is a sample text line explaining sentence {i} in the document.\n"
        ));
    }

    let section = CdmSection {
        section_id: "sec_win_1".to_string(),
        parent_id: None,
        title: Some("log.txt".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("log.txt".to_string()),
        source_url: "https://example.com/log.txt".to_string(),
        line_range: None,
        byte_range: None,
        language: None,
        content_hash: CdmSection::compute_hash(&body),
        body,
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(180)
        .with_overlap_tokens(30);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_sliding_window(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("sliding window succeeds");

    assert!(chunks.len() > 1, "must produce multiple window chunks");
    assert_eq!(report.window_chunks_count, chunks.len());

    for chunk in &chunks {
        assert_eq!(chunk.kind, ChunkKind::SlidingWindow);
        assert!(!chunk.content.is_empty());
    }

    // Verify overlap continuity: chunk N shares text lines with chunk N-1
    for i in 1..chunks.len() {
        let prev_chunk = &chunks[i - 1];
        let curr_chunk = &chunks[i];

        assert_eq!(
            curr_chunk
                .metadata
                .get("has_overlap_prefix")
                .map(String::as_str),
            Some("true")
        );

        // The starting line of curr_chunk must be <= ending line of prev_chunk
        assert!(
            curr_chunk.line_range.start_line <= prev_chunk.line_range.end_line,
            "current chunk start ({}) must overlap with previous chunk end ({})",
            curr_chunk.line_range.start_line,
            prev_chunk.line_range.end_line
        );
    }
}

#[test]
fn test_sliding_window_with_zero_overlap() {
    let mut body = String::new();
    for i in 1..=30 {
        body.push_str(&format!(
            "Row {i}: Discrete data item number {i} without shared context.\n"
        ));
    }

    let section = CdmSection {
        section_id: "sec_no_ov".to_string(),
        parent_id: None,
        title: Some("raw_data.txt".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("raw_data.txt".to_string()),
        source_url: "https://example.com/raw_data.txt".to_string(),
        line_range: None,
        byte_range: None,
        language: None,
        content_hash: CdmSection::compute_hash(&body),
        body,
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(60)
        .with_overlap_tokens(0);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_sliding_window(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert_eq!(chunk.metadata.get("has_overlap_prefix"), None);
    }

    // With zero overlap, chunks should strictly abut each other
    for i in 1..chunks.len() {
        assert_eq!(
            chunks[i].line_range.start_line,
            chunks[i - 1].line_range.end_line + 1
        );
    }
}
