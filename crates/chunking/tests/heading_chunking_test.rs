//! # Heading-Aware Markdown Chunking Integration Tests
//!
//! Validates hierarchical Markdown splitting, breadcrumbs, and paragraph boundaries.

use web_repo_to_ai_cdm::{CdmSection, LineRange, SectionKind};
use web_repo_to_ai_chunking::{
    chunk_markdown_section, ChunkKind, ChunkingConfig, ChunkingReport, TokenCounter,
};
use web_repo_to_ai_core::DocumentId;

#[test]
fn test_markdown_hierarchical_heading_breadcrumbs() {
    let markdown = r#"# Antigravity Architecture

Welcome to the architectural specification for the system.

## Storage Subsystem

The storage subsystem manages persistent state across database and file backends.

### Content-Addressed Storage

Content-addressed storage (CAS) shards data using cryptographic digests (BLAKE3 and SHA-256).
Every blob is indexed by its 64-character hexadecimal digest.

### Cache Eviction Policy

Cache eviction uses a least-recently-used (LRU) algorithm with bounded byte quotas.

## Networking Subsystem

The networking subsystem orchestrates HTTP connections and rate limiting.
"#;

    let section = CdmSection {
        section_id: "sec_md_1".to_string(),
        parent_id: None,
        title: Some("docs/ARCHITECTURE.md".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("docs/ARCHITECTURE.md".to_string()),
        source_url: "https://example.com/docs/architecture".to_string(),
        line_range: Some(LineRange::new(1, 20)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(markdown),
        body: markdown.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(60)
        .with_max_tokens(100);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_markdown_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(!chunks.is_empty());
    assert_eq!(report.heading_chunks_count, chunks.len());

    for chunk in &chunks {
        assert_eq!(chunk.kind, ChunkKind::MarkdownSection);
        assert!(chunk.metadata.contains_key("breadcrumbs"));
        assert!(chunk.metadata.contains_key("heading"));
        assert!(chunk.metadata.contains_key("heading_level"));
        assert!(chunk.content.contains("# Context:"));
    }

    // Verify breadcrumb contents
    let breadcrumb_strings: Vec<_> = chunks
        .iter()
        .map(|c| c.metadata.get("breadcrumbs").unwrap().as_str())
        .collect();

    let found_cas = breadcrumb_strings.iter().any(|b| {
        b.contains("ARCHITECTURE.md")
            && b.contains("Antigravity Architecture")
            && b.contains("Storage Subsystem")
    });
    assert!(
        found_cas,
        "breadcrumbs must capture full parent hierarchy stack"
    );
}

#[test]
fn test_oversized_prose_paragraph_splitting() {
    // Generate a long section with multiple paragraphs under a single heading
    let mut doc = String::from("# Detailed Performance Specification\n\n");
    for i in 0..15 {
        doc.push_str(&format!(
            "Paragraph {i}: The system performance is continuously benchmarked across diverse workloads. \
            Throughput must remain strictly under 50 milliseconds per operation, with bounded memory ceilings \
            and zero allocations on critical hot paths. Database transactions are isolated and verified.\n\n"
        ));
    }

    let section = CdmSection {
        section_id: "sec_perf_1".to_string(),
        parent_id: None,
        title: Some("PERFORMANCE.md".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("PERFORMANCE.md".to_string()),
        source_url: "https://example.com/perf".to_string(),
        line_range: None,
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(&doc),
        body: doc,
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(150);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_markdown_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("chunking succeeds");

    assert!(
        chunks.len() > 1,
        "oversized section must split across chunks"
    );
    assert!(report.oversized_splits_count >= 1);

    for (idx, chunk) in chunks.iter().enumerate() {
        assert!(chunk.content.contains("Detailed Performance Specification"));
        assert!(chunk.token_count <= 250);
        assert!(!chunk.raw_content.is_empty());
        let part_expected = format!("Part {}", idx + 1);
        assert!(chunk.title.as_ref().unwrap().contains(&part_expected));
    }
}

#[test]
fn test_markdown_deep_nested_and_skipped_headings() {
    let markdown = r#"# Root Title

Intro text.

#### Deep H4 Section (Skipped H2 and H3)

This section skips directly to level 4.

###### Deepest H6 Section

Final subsection under root.
"#;

    let section = CdmSection {
        section_id: "sec_h_skip".to_string(),
        parent_id: None,
        title: Some("guide.md".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("guide.md".to_string()),
        source_url: "https://example.com/guide.md".to_string(),
        line_range: None,
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(markdown),
        body: markdown.to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default().with_target_tokens(10);
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_markdown_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("markdown chunking succeeds");

    assert!(!chunks.is_empty());
    let h6_chunk = chunks
        .iter()
        .find(|c| c.content.contains("Deepest H6 Section"))
        .expect("must find chunk containing H6 heading");
    let breadcrumbs = h6_chunk.metadata.get("breadcrumbs").unwrap();
    assert!(breadcrumbs.contains("Root Title"));
    assert!(breadcrumbs.contains("Deep H4 Section"));
    assert!(breadcrumbs.contains("Deepest H6 Section"));
}

#[test]
fn test_empty_section_returns_empty_chunks() {
    let section = CdmSection {
        section_id: "sec_empty".to_string(),
        parent_id: None,
        title: Some("empty.md".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("empty.md".to_string()),
        source_url: "https://example.com/empty.md".to_string(),
        line_range: None,
        byte_range: None,
        language: None,
        content_hash: CdmSection::compute_hash(""),
        body: "   \n\n  \t  \n".to_string(),
        token_metrics: None,
        attributes: std::collections::BTreeMap::new(),
    };

    let doc_id = DocumentId::new();
    let config = ChunkingConfig::default();
    let token_counter = TokenCounter::default();
    let mut report = ChunkingReport::default();

    let chunks = chunk_markdown_section(&doc_id, &section, 0, &config, &token_counter, &mut report)
        .expect("empty markdown chunking succeeds");

    assert!(chunks.is_empty());
}
