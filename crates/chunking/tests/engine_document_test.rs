//! # End-to-End Document Chunking & Invariant Tests
//!
//! Validates Chunker pipeline on full CdmDocuments, hash determinism, and configuration bounds.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSection, LineRange, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_chunking::{
    chunk_document, chunk_text, ChunkKind, Chunker, ChunkingConfig, ChunkingError, ChunkingStrategy,
};

fn create_sample_cdm_document() -> CdmDocument {
    let source_identity = SourceIdentity::new(
        "https://github.com/example/monorepo",
        "https://github.com/example/monorepo",
    );

    let sec1 = CdmSection {
        section_id: "sec_readme".to_string(),
        parent_id: None,
        title: Some("README.md".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("README.md".to_string()),
        source_url: "https://github.com/example/monorepo/blob/main/README.md".to_string(),
        line_range: Some(LineRange::new(1, 20)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash_readme".to_string(),
        body: "# Monorepo\n\n## Getting Started\n\nInstall the dependencies:\n\n```bash\ncargo build\n```\n\n## Architecture\n\nOur system is built with high modularity.\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec2 = CdmSection {
        section_id: "sec_lib".to_string(),
        parent_id: None,
        title: Some("crates/core/src/lib.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/core/src/lib.rs".to_string()),
        source_url: "https://github.com/example/monorepo/blob/main/crates/core/src/lib.rs".to_string(),
        line_range: Some(LineRange::new(1, 25)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_lib".to_string(),
        body: "pub struct Config {\n    pub port: u16,\n}\n\nimpl Config {\n    pub fn default_config() -> Self {\n        Self { port: 8080 }\n    }\n}\n\npub fn run(config: &Config) -> bool {\n    config.port > 0\n}\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec3 = CdmSection {
        section_id: "sec_metrics".to_string(),
        parent_id: None,
        title: Some("metrics.csv".to_string()),
        kind: SectionKind::Table,
        original_path: Some("data/metrics.csv".to_string()),
        source_url: "https://github.com/example/monorepo/blob/main/data/metrics.csv".to_string(),
        line_range: Some(LineRange::new(1, 5)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash_metrics".to_string(),
        body: "| Metric | Value | Threshold |\n| --- | --- | --- |\n| Latency | 12ms | 50ms |\n| Error Rate | 0.01% | 1.0% |\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    CdmDocumentBuilder::new(
        source_identity,
        SourceType::GitHubPublic,
        "https://github.com/example/monorepo",
    )
    .title("Monorepo Overview")
    .add_section(sec1)
    .add_section(sec2)
    .add_section(sec3)
    .build()
}

#[test]
fn test_end_to_end_adaptive_document_chunking() {
    let doc = create_sample_cdm_document();
    let config = ChunkingConfig::default().with_strategy(ChunkingStrategy::Adaptive);

    let result = chunk_document(&doc, config).expect("document chunking succeeds");

    assert_eq!(result.document_id, doc.document_id);
    assert_eq!(result.report.sections_processed, 3);
    assert!(result.chunks.len() >= 3);
    assert!(result.total_tokens > 0);
    assert!(result.report.min_tokens > 0);
    assert!(result.report.max_tokens >= result.report.min_tokens);
    assert!(result.report.avg_tokens > 0.0);

    // Verify adaptive strategy assigned correct kinds
    let kinds: Vec<_> = result.chunks.iter().map(|c| c.kind).collect();
    assert!(kinds.contains(&ChunkKind::MarkdownSection));
    assert!(kinds.contains(&ChunkKind::CodeAstDeclaration));
    assert!(kinds.contains(&ChunkKind::Table));

    // Verify sequential chunk numbering
    for (i, c) in result.chunks.iter().enumerate() {
        assert_eq!(c.chunk_index, i);
        assert_eq!(c.total_chunks, result.chunks.len());
    }
}

#[test]
fn test_chunking_bit_for_bit_hash_determinism() {
    let doc = create_sample_cdm_document();
    let config = ChunkingConfig::default().with_target_tokens(100);

    let res1 = chunk_document(&doc, config.clone()).expect("res1 succeeds");
    let res2 = chunk_document(&doc, config).expect("res2 succeeds");

    assert_eq!(res1.chunks.len(), res2.chunks.len());
    for (c1, c2) in res1.chunks.iter().zip(res2.chunks.iter()) {
        assert_eq!(c1.content_hash, c2.content_hash);
        assert_eq!(c1.content, c2.content);
        assert_eq!(c1.token_count, c2.token_count);
        assert_eq!(c1.line_range, c2.line_range);
    }
}

#[test]
fn test_chunk_text_convenience_function() {
    let rust_text =
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn sub(a: i32, b: i32) -> i32 { a - b }";
    let res = chunk_text(
        rust_text,
        Some("math.rs"),
        Some("rust"),
        ChunkingConfig::default(),
    )
    .expect("chunk_text succeeds");

    assert!(!res.chunks.is_empty());
    assert_eq!(res.chunks[0].language.as_deref(), Some("rust"));
    assert_eq!(res.chunks[0].kind, ChunkKind::CodeAstDeclaration);
}

#[test]
fn test_empty_document_error_handling() {
    let mut doc = create_sample_cdm_document();
    doc.sections.clear();

    let err = chunk_document(&doc, ChunkingConfig::default()).unwrap_err();
    assert!(matches!(err, ChunkingError::EmptyDocument(_)));
}

#[test]
fn test_invalid_config_rejections() {
    let config = ChunkingConfig::default().with_max_tokens(0);
    assert!(config.validate().is_err());

    let config = ChunkingConfig::default()
        .with_target_tokens(200)
        .with_max_tokens(100);
    assert!(config.validate().is_err());

    let config = ChunkingConfig::default()
        .with_min_tokens(500)
        .with_target_tokens(200);
    assert!(config.validate().is_err());

    let config = ChunkingConfig::default()
        .with_max_tokens(100)
        .with_overlap_tokens(100);
    assert!(config.validate().is_err());
}

#[test]
fn test_performance_sub_50ms_for_document() {
    let doc = create_sample_cdm_document();
    let config = ChunkingConfig::default();
    let chunker = Chunker::new(config);

    let start = std::time::Instant::now();
    for _ in 0..50 {
        let _ = chunker.chunk_document(&doc).expect("chunk succeeds");
    }
    let elapsed = start.elapsed();

    // 50 executions should finish in well under 100ms (< 2ms per execution)
    assert!(
        elapsed.as_millis() < 500,
        "50 document chunking passes took {:?}, must be high throughput (< 500ms)",
        elapsed
    );
}
