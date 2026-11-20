//! # Cross-Reference & Relationship Injection Integration Tests
//!
//! Validates sequential chunk chaining, multi-part split hierarchy, intra-document symbol
//! call/import edges, CDM relationship propagation, and context header injection.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocumentBuilder, CdmSection, LineRange, Relationship, RelationshipKind, SectionKind,
    SourceIdentity, SourceType,
};
use web_repo_to_ai_chunking::{chunk_document, Chunk, ChunkingConfig, CrossReferenceKind};

#[test]
fn test_sequential_chunk_chaining() {
    let identity = SourceIdentity::new("https://example.com/doc", "https://example.com/doc");

    let sec1 = CdmSection {
        section_id: "sec_1".to_string(),
        parent_id: None,
        title: Some("Section 1".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("doc.md".to_string()),
        source_url: "https://example.com/doc#sec1".to_string(),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash1".to_string(),
        body: "# Introduction\n\nFirst part of the manual.\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec2 = CdmSection {
        section_id: "sec_2".to_string(),
        parent_id: None,
        title: Some("Section 2".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("doc.md".to_string()),
        source_url: "https://example.com/doc#sec2".to_string(),
        line_range: Some(LineRange::new(11, 20)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash2".to_string(),
        body: "# Setup\n\nSecond part of the manual.\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec3 = CdmSection {
        section_id: "sec_3".to_string(),
        parent_id: None,
        title: Some("Section 3".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("doc.md".to_string()),
        source_url: "https://example.com/doc#sec3".to_string(),
        line_range: Some(LineRange::new(21, 30)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash3".to_string(),
        body: "# Advanced Usage\n\nThird part of the manual.\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/doc",
    )
    .title("Complete User Manual")
    .add_section(sec1)
    .add_section(sec2)
    .add_section(sec3)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    assert_eq!(result.chunks.len(), 3);

    let c0 = &result.chunks[0];
    let c1 = &result.chunks[1];
    let c2 = &result.chunks[2];

    // Assert Chunk 0 links
    assert_eq!(c0.prev_chunk_id, None);
    assert_eq!(c0.next_chunk_id.as_ref(), Some(&c1.chunk_id));

    // Assert Chunk 1 links
    assert_eq!(c1.prev_chunk_id.as_ref(), Some(&c0.chunk_id));
    assert_eq!(c1.next_chunk_id.as_ref(), Some(&c2.chunk_id));

    // Assert Chunk 2 links
    assert_eq!(c2.prev_chunk_id.as_ref(), Some(&c1.chunk_id));
    assert_eq!(c2.next_chunk_id, None);

    // Assert cross-references contain previous and next kinds
    assert!(c1
        .cross_references
        .iter()
        .any(|r| r.kind == CrossReferenceKind::PreviousChunk
            && r.target_chunk_id.as_ref() == Some(&c0.chunk_id)));
    assert!(c1
        .cross_references
        .iter()
        .any(|r| r.kind == CrossReferenceKind::NextChunk
            && r.target_chunk_id.as_ref() == Some(&c2.chunk_id)));
}

#[test]
fn test_split_part_parent_child_chaining() {
    let identity = SourceIdentity::new("https://example.com/repo", "https://example.com/repo");

    // Construct an oversized function with many lines that forces sub-splitting
    let mut large_body = String::from("pub fn process_huge_pipeline() {\n");
    for i in 0..120 {
        large_body.push_str(&format!(
            "    let intermediate_step_{i} = perform_heavy_operation({i});\n"
        ));
    }
    large_body.push_str("}\n");

    let sec = CdmSection {
        section_id: "sec_oversized".to_string(),
        parent_id: None,
        title: Some("crates/pipeline/src/huge.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/pipeline/src/huge.rs".to_string()),
        source_url: "https://example.com/repo/blob/main/crates/pipeline/src/huge.rs".to_string(),
        line_range: Some(LineRange::new(1, 125)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_huge".to_string(),
        body: large_body,
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://example.com/repo",
    )
    .add_section(sec)
    .build();

    let config = ChunkingConfig::default()
        .with_target_tokens(100)
        .with_max_tokens(150);

    let result = chunk_document(&doc, config).expect("chunking succeeds");
    assert!(result.chunks.len() >= 2);

    let part1 = &result.chunks[0];
    let part2 = &result.chunks[1];

    // Part 2 parent should point to Part 1
    assert_eq!(part2.parent_chunk_id.as_ref(), Some(&part1.chunk_id));

    // Part 2 should have SplitPart reference pointing to Part 1
    assert!(part2
        .cross_references
        .iter()
        .any(|r| r.kind == CrossReferenceKind::SplitPart
            && r.target_chunk_id.as_ref() == Some(&part1.chunk_id)));

    // Part 1 should have ChildChunk reference pointing to Part 2
    assert!(part1
        .cross_references
        .iter()
        .any(|r| r.kind == CrossReferenceKind::ChildChunk
            && r.target_chunk_id.as_ref() == Some(&part2.chunk_id)));
}

#[test]
fn test_intra_document_symbol_call_and_import_resolution() {
    let identity = SourceIdentity::new("https://example.com/repo", "https://example.com/repo");

    let sec_math = CdmSection {
        section_id: "sec_math".to_string(),
        parent_id: None,
        title: Some("crates/core/src/math.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/core/src/math.rs".to_string()),
        source_url: "https://example.com/repo/blob/main/crates/core/src/math.rs".to_string(),
        line_range: Some(LineRange::new(1, 20)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_math".to_string(),
        body: "pub fn compute_entropy(bytes: &[u8]) -> f64 {\n    42.0\n}\n\npub struct EntropyEngine;\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec_service = CdmSection {
        section_id: "sec_service".to_string(),
        parent_id: None,
        title: Some("crates/core/src/service.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/core/src/service.rs".to_string()),
        source_url: "https://example.com/repo/blob/main/crates/core/src/service.rs".to_string(),
        line_range: Some(LineRange::new(1, 25)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_service".to_string(),
        body: "use crate::math::compute_entropy;\n\npub fn evaluate_security(buffer: &[u8]) -> bool {\n    let score = compute_entropy(buffer);\n    score > 0.0\n}\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://example.com/repo",
    )
    .add_section(sec_math)
    .add_section(sec_service)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");

    let math_chunk = result
        .chunks
        .iter()
        .find(|c| {
            c.section_id == "sec_math" && c.ast_symbols.iter().any(|s| s.name == "compute_entropy")
        })
        .expect("math chunk found");

    let service_chunk = result
        .chunks
        .iter()
        .find(|c| c.section_id == "sec_service")
        .expect("service chunk found");

    // The service chunk should reference the compute_entropy symbol in math chunk
    let found_ref = service_chunk.cross_references.iter().find(|r| {
        r.target_symbol.as_deref() == Some("compute_entropy")
            && r.target_chunk_id.as_ref() == Some(&math_chunk.chunk_id)
    });

    assert!(
        found_ref.is_some(),
        "service chunk must cross-reference compute_entropy in math chunk"
    );
    let r = found_ref.unwrap();
    assert!(matches!(
        r.kind,
        CrossReferenceKind::Imports | CrossReferenceKind::Calls | CrossReferenceKind::References
    ));
}

#[test]
fn test_cdm_relationship_propagation_to_chunks() {
    let identity = SourceIdentity::new("https://example.com/repo", "https://example.com/repo");

    let sec1 = CdmSection {
        section_id: "sec_controller".to_string(),
        parent_id: None,
        title: Some("src/controller.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/controller.rs".to_string()),
        source_url: "https://example.com/repo/blob/main/src/controller.rs".to_string(),
        line_range: Some(LineRange::new(1, 20)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_c".to_string(),
        body: "pub fn handle_request() -> bool { true }\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let sec2 = CdmSection {
        section_id: "sec_database".to_string(),
        parent_id: None,
        title: Some("src/database.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/database.rs".to_string()),
        source_url: "https://example.com/repo/blob/main/src/database.rs".to_string(),
        line_range: Some(LineRange::new(1, 20)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_d".to_string(),
        body: "pub fn fetch_user_record() -> Option<String> { None }\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let rel = Relationship::new("sec_controller", "sec_database", RelationshipKind::Calls)
        .with_description("Controller queries persistence layer");

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://example.com/repo",
    )
    .add_section(sec1)
    .add_section(sec2)
    .add_relationship(rel)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");

    let c_chunk = result
        .chunks
        .iter()
        .find(|c| c.section_id == "sec_controller")
        .expect("controller chunk found");

    let db_chunk = result
        .chunks
        .iter()
        .find(|c| c.section_id == "sec_database")
        .expect("database chunk found");

    assert!(c_chunk.cross_references.iter().any(|r| {
        r.target_chunk_id.as_ref() == Some(&db_chunk.chunk_id)
            && r.kind == CrossReferenceKind::Calls
            && r.description.as_deref() == Some("Controller queries persistence layer")
    }));
}

#[test]
fn test_injected_context_header_formatting() {
    let identity = SourceIdentity::new(
        "https://github.com/example/monorepo",
        "https://github.com/example/monorepo",
    );

    let sec = CdmSection {
        section_id: "sec_code".to_string(),
        parent_id: None,
        title: Some("src/main.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/main.rs".to_string()),
        source_url: "https://github.com/example/monorepo/blob/main/src/main.rs".to_string(),
        line_range: Some(LineRange::new(1, 15)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_m".to_string(),
        body: "pub fn main() {\n    println!(\"Hello World\");\n}\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/monorepo",
    )
    .title("Monorepo Example")
    .add_section(sec)
    .build();

    let config = ChunkingConfig::default()
        .with_citations(true)
        .with_sequence_links(true)
        .with_preserve_headers(true);

    let result = chunk_document(&doc, config).expect("chunking succeeds");
    assert!(!result.chunks.is_empty());

    let chunk = &result.chunks[0];
    let header = chunk
        .context_header
        .as_ref()
        .expect("header must be present");

    assert!(header.contains("// File: src/main.rs"));
    assert!(header.contains("// Source: [src/main.rs#L1-L"));
    assert!(header.contains("// Sequence: Chunk 1 of 1 (Prev: None, Next: None)"));
    assert!(chunk.content.starts_with(header));
}

#[test]
fn test_chunk_serde_json_roundtrip_with_citations_and_cross_refs() {
    let identity = SourceIdentity::new(
        "https://github.com/example/roundtrip",
        "https://github.com/example/roundtrip",
    );

    let sec = CdmSection {
        section_id: "sec_code".to_string(),
        parent_id: None,
        title: Some("src/lib.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/lib.rs".to_string()),
        source_url: "https://github.com/example/roundtrip/blob/main/src/lib.rs".to_string(),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_rt".to_string(),
        body: "pub fn test_rt() -> i32 { 100 }\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/roundtrip",
    )
    .add_section(sec)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    let original_chunk = &result.chunks[0];

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(original_chunk).expect("serialization succeeds");
    assert!(json_str.contains("\"citation\""));
    assert!(json_str.contains("\"permalink\""));

    // Deserialize from JSON
    let deserialized_chunk: Chunk =
        serde_json::from_str(&json_str).expect("deserialization succeeds");

    assert_eq!(original_chunk.chunk_id, deserialized_chunk.chunk_id);
    assert_eq!(original_chunk.content_hash, deserialized_chunk.content_hash);
    assert_eq!(original_chunk.token_count, deserialized_chunk.token_count);
    assert_eq!(original_chunk.citation, deserialized_chunk.citation);
    assert_eq!(
        original_chunk.cross_references,
        deserialized_chunk.cross_references
    );
    assert_eq!(
        original_chunk.prev_chunk_id,
        deserialized_chunk.prev_chunk_id
    );
    assert_eq!(
        original_chunk.next_chunk_id,
        deserialized_chunk.next_chunk_id
    );
}
