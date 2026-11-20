//! # Integration Tests: JSONL Streaming Chunk Export (`tests/export_jsonl_test.rs`)

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, LineRange};
use web_repo_to_ai_chunking::{
    export_chunks_to_jsonl, Chunk, ChunkCitation, ChunkExportMetrics, ChunkExportOptions,
    ChunkExporter, ChunkId, ChunkKind, GenericRagRecord, VectorTarget,
};
use web_repo_to_ai_core::DocumentId;

fn create_sample_chunks() -> Vec<Chunk> {
    let doc_id = DocumentId::new();

    let chunk1 = Chunk {
        chunk_id: ChunkId::new("chk_export_s0_c0_a1b2c3d4"),
        document_id: doc_id,
        section_id: "sec_0".to_string(),
        chunk_index: 0,
        total_chunks: 2,
        title: Some("Authentication Service Overview".to_string()),
        original_path: Some("docs/auth.md".to_string()),
        source_url: "https://example.com/docs/auth".to_string(),
        line_range: LineRange::new(1, 25),
        byte_range: ByteRange::new(0, 500),
        content: "### Authentication Service Overview\n\nHandles JWT issuance and validation."
            .to_string(),
        raw_content: "### Authentication Service Overview\n\nHandles JWT issuance and validation."
            .to_string(),
        context_header: None,
        content_hash: "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0"
            .to_string(),
        token_count: 35,
        token_estimator: "exact:o200k_base".to_string(),
        language: Some("markdown".to_string()),
        kind: ChunkKind::MarkdownSection,
        ast_symbols: Vec::new(),
        metadata: BTreeMap::new(),
        citation: Some(ChunkCitation {
            citation_id: "cit_0".to_string(),
            source_url: "https://example.com/docs/auth".to_string(),
            permalink: "https://example.com/docs/auth#overview".to_string(),
            display_citation: "docs/auth.md > Overview".to_string(),
            markdown_citation: "[docs/auth.md > Overview](https://example.com/docs/auth#overview)"
                .to_string(),
            file_path: Some("docs/auth.md".to_string()),
            line_span: Some((1, 25)),
            commit_sha: None,
            source_title: Some("Documentation".to_string()),
            anchor: Some("overview".to_string()),
        }),
        cross_references: Vec::new(),
        prev_chunk_id: None,
        next_chunk_id: Some(ChunkId::new("chk_export_s0_c1_e5f6a7b8")),
        parent_chunk_id: None,
    };

    let chunk2 = Chunk {
        chunk_id: ChunkId::new("chk_export_s0_c1_e5f6a7b8"),
        document_id: doc_id,
        section_id: "sec_0".to_string(),
        chunk_index: 1,
        total_chunks: 2,
        title: Some("Token Refresh Flow".to_string()),
        original_path: Some("docs/auth.md".to_string()),
        source_url: "https://example.com/docs/auth".to_string(),
        line_range: LineRange::new(26, 60),
        byte_range: ByteRange::new(501, 1200),
        content: "### Token Refresh Flow\n\nClients exchange refresh tokens for short-lived access tokens.".to_string(),
        raw_content: "### Token Refresh Flow\n\nClients exchange refresh tokens for short-lived access tokens.".to_string(),
        context_header: None,
        content_hash: "e5f6a7b8c9d0123456789abcdef0123456789abcdef0123456789abcdef01234".to_string(),
        token_count: 42,
        token_estimator: "exact:o200k_base".to_string(),
        language: Some("markdown".to_string()),
        kind: ChunkKind::MarkdownSection,
        ast_symbols: Vec::new(),
        metadata: BTreeMap::new(),
        citation: Some(ChunkCitation {
            citation_id: "cit_1".to_string(),
            source_url: "https://example.com/docs/auth".to_string(),
            permalink: "https://example.com/docs/auth#refresh".to_string(),
            display_citation: "docs/auth.md > Refresh".to_string(),
            markdown_citation: "[docs/auth.md > Refresh](https://example.com/docs/auth#refresh)".to_string(),
            file_path: Some("docs/auth.md".to_string()),
            line_span: Some((26, 60)),
            commit_sha: None,
            source_title: Some("Documentation".to_string()),
            anchor: Some("refresh".to_string()),
        }),
        cross_references: Vec::new(),
        prev_chunk_id: Some(ChunkId::new("chk_export_s0_c0_a1b2c3d4")),
        next_chunk_id: None,
        parent_chunk_id: None,
    };

    vec![chunk1, chunk2]
}

#[test]
fn test_export_chunks_to_jsonl_basic() {
    let chunks = create_sample_chunks();
    let jsonl = export_chunks_to_jsonl(&chunks).expect("JSONL export should succeed");

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Should emit exactly 2 newline-delimited lines"
    );

    // Verify line 1 deserializes into GenericRagRecord
    let record1: GenericRagRecord =
        serde_json::from_str(lines[0]).expect("Line 1 should deserialize into GenericRagRecord");
    assert_eq!(record1.id, "chk_export_s0_c0_a1b2c3d4");
    assert_eq!(record1.token_count, 35);
    assert_eq!(record1.chunk_index, 0);
    assert_eq!(record1.total_chunks, 2);
    assert_eq!(record1.line_span, [1, 25]);
    assert_eq!(
        record1.next_chunk_id.as_deref(),
        Some("chk_export_s0_c1_e5f6a7b8")
    );

    // Verify citation presence
    let cit1 = record1.citation.expect("Citation should be present");
    assert_eq!(cit1.permalink, "https://example.com/docs/auth#overview");
    assert_eq!(cit1.display_citation, "docs/auth.md > Overview");

    // Verify line 2 deserializes
    let record2: GenericRagRecord =
        serde_json::from_str(lines[1]).expect("Line 2 should deserialize into GenericRagRecord");
    assert_eq!(record2.id, "chk_export_s0_c1_e5f6a7b8");
    assert_eq!(record2.token_count, 42);
    assert_eq!(
        record2.prev_chunk_id.as_deref(),
        Some("chk_export_s0_c0_a1b2c3d4")
    );
}

#[test]
fn test_export_to_writer_streaming_metrics() {
    let chunks = create_sample_chunks();
    let exporter = ChunkExporter::new();

    let mut buffer = Vec::new();
    let metrics: ChunkExportMetrics = exporter
        .export_to_writer(&chunks, &mut buffer)
        .expect("Streaming export to writer should succeed");

    assert_eq!(metrics.chunks_exported, 2);
    assert_eq!(metrics.lines_written, 2);
    assert_eq!(metrics.tokens_exported, 77); // 35 + 42
    assert!(!metrics.truncated);
    assert_eq!(metrics.bytes_written, buffer.len());
    assert!(metrics.bytes_written > 0);
}

#[test]
fn test_export_token_budget_truncation() {
    let chunks = create_sample_chunks();
    // Total tokens = 77. Set budget to 40 so only chunk 1 (35 tokens) fits.
    let options = ChunkExportOptions::default()
        .with_target(VectorTarget::GenericJsonl)
        .with_max_token_budget(Some(40));

    let exporter = ChunkExporter::with_options(options);
    let (output, metrics) = exporter
        .export_to_string(&chunks)
        .expect("Export should succeed");

    assert_eq!(metrics.chunks_exported, 1);
    assert_eq!(metrics.tokens_exported, 35);
    assert!(metrics.truncated, "Export should be marked as truncated");

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 1, "Only first chunk should be exported");
}

#[test]
fn test_export_empty_chunks_list() {
    let empty_chunks: Vec<Chunk> = Vec::new();
    let exporter = ChunkExporter::new();

    let (output, metrics) = exporter
        .export_to_string(&empty_chunks)
        .expect("Exporting empty list should succeed");

    assert!(output.is_empty());
    assert_eq!(metrics.chunks_exported, 0);
    assert_eq!(metrics.tokens_exported, 0);
    assert_eq!(metrics.lines_written, 0);
    assert_eq!(metrics.bytes_written, 0);
}
