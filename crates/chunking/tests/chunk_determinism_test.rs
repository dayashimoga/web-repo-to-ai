//! # Integration Tests: Chunk Determinism & Graph Invariants (`tests/chunk_determinism_test.rs`)

use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

use web_repo_to_ai_cdm::{
    ByteRange, CdmDocument, CdmDocumentBuilder, CdmSection, LineRange, SectionKind, SourceIdentity,
    SourceType,
};
use web_repo_to_ai_chunking::{chunk_document, Chunker, ChunkingConfig};

fn build_determinism_test_document() -> CdmDocument {
    let url = "https://github.com/example/determinism-repo";
    let source_identity = SourceIdentity::new(url, url);

    let rust_code = r#"// File: src/engine.rs
use std::sync::Arc;

pub struct ExecutionEngine {
    concurrency_limit: usize,
}

impl ExecutionEngine {
    pub fn new(concurrency_limit: usize) -> Self {
        Self { concurrency_limit }
    }

    pub async fn execute_task(&self, task_id: &str) -> bool {
        println!("Executing task {}", task_id);
        true
    }
}
"#;

    let markdown_guide = r#"# Architecture Specification

## Concurrency Model

The execution runtime distributes work across cooperative async tasks using bounded channels.

## Memory Governance

Buffer pools maintain hard upper bounds preventing unbounded memory growth under load.
"#;

    let sec1 = CdmSection {
        section_id: "sec_code".to_string(),
        parent_id: None,
        title: Some("engine.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/engine.rs".to_string()),
        source_url: format!("{url}/blob/main/src/engine.rs"),
        line_range: Some(LineRange::new(1, 18)),
        byte_range: Some(ByteRange::new(0, rust_code.len() as u64)),
        language: Some("rust".to_string()),
        content_hash: blake3::hash(rust_code.as_bytes()).to_hex().to_string(),
        body: rust_code.to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec2 = CdmSection {
        section_id: "sec_doc".to_string(),
        parent_id: None,
        title: Some("Architecture Specification".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("docs/spec.md".to_string()),
        source_url: format!("{url}/blob/main/docs/spec.md"),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: Some(ByteRange::new(0, markdown_guide.len() as u64)),
        language: Some("markdown".to_string()),
        content_hash: blake3::hash(markdown_guide.as_bytes()).to_hex().to_string(),
        body: markdown_guide.to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    CdmDocumentBuilder::new(source_identity, SourceType::GitHubPublic, url)
        .title("Determinism Repo")
        .add_section(sec1)
        .add_section(sec2)
        .build()
}

#[test]
fn test_ten_iteration_sequential_determinism() {
    let doc = build_determinism_test_document();
    let config = ChunkingConfig::default();
    let chunker = Chunker::new(config);

    let baseline = chunker.chunk_document(&doc).expect("Baseline chunking");
    assert!(!baseline.chunks.is_empty());

    for i in 1..=10 {
        let current = chunker.chunk_document(&doc).expect("Subsequent chunking");
        assert_eq!(
            baseline.chunks.len(),
            current.chunks.len(),
            "Iteration {i}: Chunk count mismatch"
        );

        for (idx, (b, c)) in baseline
            .chunks
            .iter()
            .zip(current.chunks.iter())
            .enumerate()
        {
            assert_eq!(
                b.chunk_id, c.chunk_id,
                "Iteration {i}, Chunk {idx}: ID mismatch"
            );
            assert_eq!(
                b.content_hash, c.content_hash,
                "Iteration {i}, Chunk {idx}: Hash mismatch"
            );
            assert_eq!(
                b.token_count, c.token_count,
                "Iteration {i}, Chunk {idx}: Token count mismatch"
            );
            assert_eq!(
                b.content, c.content,
                "Iteration {i}, Chunk {idx}: Content mismatch"
            );
            assert_eq!(
                b.prev_chunk_id, c.prev_chunk_id,
                "Iteration {i}, Chunk {idx}: prev_chunk_id mismatch"
            );
            assert_eq!(
                b.next_chunk_id, c.next_chunk_id,
                "Iteration {i}, Chunk {idx}: next_chunk_id mismatch"
            );
        }
    }
}

#[test]
fn test_concurrent_multithreaded_determinism() {
    let doc = Arc::new(build_determinism_test_document());
    let config = Arc::new(ChunkingConfig::default());

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let doc_clone = Arc::clone(&doc);
            let config_clone = Arc::clone(&config);
            thread::spawn(move || {
                let chunker = Chunker::new((*config_clone).clone());
                chunker.chunk_document(&doc_clone).expect("Thread chunking")
            })
        })
        .collect();

    let mut results = Vec::new();
    for h in handles {
        results.push(h.join().expect("Thread should join cleanly"));
    }

    let first = &results[0];
    for (thread_idx, other) in results.iter().enumerate().skip(1) {
        assert_eq!(
            first.chunks.len(),
            other.chunks.len(),
            "Thread {thread_idx} produced different chunk count"
        );
        for (idx, (a, b)) in first.chunks.iter().zip(other.chunks.iter()).enumerate() {
            assert_eq!(
                a.chunk_id, b.chunk_id,
                "Thread {thread_idx}, Chunk {idx} ID mismatch"
            );
            assert_eq!(
                a.content_hash, b.content_hash,
                "Thread {thread_idx}, Chunk {idx} Content hash mismatch"
            );
        }
    }
}

#[test]
fn test_sequential_chaining_bidirectional_symmetry() {
    let doc = build_determinism_test_document();
    let res = chunk_document(&doc, ChunkingConfig::default()).expect("Chunking");
    let chunks = &res.chunks;

    for i in 0..chunks.len() {
        if i > 0 {
            assert_eq!(
                chunks[i].prev_chunk_id,
                Some(chunks[i - 1].chunk_id.clone()),
                "Chunk {i} prev_chunk_id must point to Chunk {}",
                i - 1
            );
        } else {
            assert_eq!(
                chunks[i].prev_chunk_id, None,
                "First chunk must have None prev_chunk_id"
            );
        }

        if i + 1 < chunks.len() {
            assert_eq!(
                chunks[i].next_chunk_id,
                Some(chunks[i + 1].chunk_id.clone()),
                "Chunk {i} next_chunk_id must point to Chunk {}",
                i + 1
            );
        } else {
            assert_eq!(
                chunks[i].next_chunk_id, None,
                "Last chunk must have None next_chunk_id"
            );
        }
    }
}

#[test]
fn test_coordinate_invariants_and_non_empty_content() {
    let doc = build_determinism_test_document();
    let res = chunk_document(&doc, ChunkingConfig::default()).expect("Chunking");

    for (idx, chunk) in res.chunks.iter().enumerate() {
        assert!(
            !chunk.content.trim().is_empty(),
            "Chunk {idx} content cannot be empty"
        );
        assert!(
            chunk.line_range.start_line <= chunk.line_range.end_line,
            "Chunk {idx} start_line {} must be <= end_line {}",
            chunk.line_range.start_line,
            chunk.line_range.end_line
        );
        assert!(
            chunk.byte_range.start_byte <= chunk.byte_range.end_byte,
            "Chunk {idx} start_byte {} must be <= end_byte {}",
            chunk.byte_range.start_byte,
            chunk.byte_range.end_byte
        );
        assert!(chunk.token_count > 0, "Chunk {idx} token count must be > 0");
    }
}
