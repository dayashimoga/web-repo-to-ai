//! # Citation & Provenance Anchoring Integration Tests
//!
//! Validates permalink synthesis, line span tracking, and formatted citations across
//! GitHub repositories, web documentation, PDFs, and structured specifications.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocumentBuilder, CdmSection, LineRange, RevisionInfo, SectionKind, SourceIdentity,
    SourceType,
};
use web_repo_to_ai_chunking::{chunk_document, chunk_text, ChunkingConfig};

#[test]
fn test_github_permalink_and_citation_generation() {
    let mut identity = SourceIdentity::new(
        "https://github.com/dayashimoga/web-repo-to-ai",
        "https://github.com/dayashimoga/web-repo-to-ai",
    );
    identity.domain = Some("github.com".to_string());
    identity.repo_owner = Some("dayashimoga".to_string());
    identity.repo_name = Some("web-repo-to-ai".to_string());
    identity.git_ref = Some("main".to_string());

    let revision = RevisionInfo {
        git_commit: Some("e329e95a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e".to_string()),
        ..Default::default()
    };

    let sec = CdmSection {
        section_id: "sec_code".to_string(),
        parent_id: None,
        title: Some("crates/chunking/src/lib.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("crates/chunking/src/lib.rs".to_string()),
        source_url:
            "https://github.com/dayashimoga/web-repo-to-ai/blob/main/crates/chunking/src/lib.rs"
                .to_string(),
        line_range: Some(LineRange::new(1, 25)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: "hash_code".to_string(),
        body: "pub fn execute_chunking() -> bool {\n    true\n}\n".to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/dayashimoga/web-repo-to-ai",
    )
    .revision(revision)
    .title("web-repo-to-ai Monorepo")
    .add_section(sec)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    assert!(!result.chunks.is_empty());

    let chunk = &result.chunks[0];
    let citation = chunk.citation.as_ref().expect("citation must be present");

    assert_eq!(
        citation.source_url,
        "https://github.com/dayashimoga/web-repo-to-ai/blob/main/crates/chunking/src/lib.rs"
    );
    assert!(citation.permalink.starts_with(
        "https://github.com/dayashimoga/web-repo-to-ai/blob/e329e95a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e/crates/chunking/src/lib.rs#L"
    ));
    assert!(citation.permalink.contains("#L1-L"));
    assert!(citation
        .display_citation
        .contains("crates/chunking/src/lib.rs:"));
    assert!(citation.display_citation.contains("(e329e95)"));
    assert_eq!(
        citation.commit_sha.as_deref(),
        Some("e329e95a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e")
    );
    assert_eq!(
        citation.file_path.as_deref(),
        Some("crates/chunking/src/lib.rs")
    );
    assert!(citation.line_span.is_some());
}

#[test]
fn test_web_article_and_doc_citation_with_anchors() {
    let identity = SourceIdentity::new(
        "https://docs.rs/tokio/1.38.0/tokio/index.html",
        "https://docs.rs/tokio/1.38.0/tokio/index.html",
    );

    let mut attributes = BTreeMap::new();
    attributes.insert(
        "heading_slug".to_string(),
        "asynchronous-runtime".to_string(),
    );
    attributes.insert("title".to_string(), "Tokio Documentation".to_string());

    let sec = CdmSection {
        section_id: "sec_runtime".to_string(),
        parent_id: None,
        title: Some("Asynchronous Runtime Architecture".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("tokio/index.html".to_string()),
        source_url: "https://docs.rs/tokio/1.38.0/tokio/index.html".to_string(),
        line_range: Some(LineRange::new(10, 40)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: "hash_runtime".to_string(),
        body: "# Asynchronous Runtime\n\nTokio is an event-driven, non-blocking I/O platform.\n"
            .to_string(),
        token_metrics: None,
        attributes,
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://docs.rs/tokio/1.38.0/tokio/index.html",
    )
    .title("Tokio Documentation")
    .add_section(sec)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    assert!(!result.chunks.is_empty());

    let chunk = &result.chunks[0];
    let citation = chunk.citation.as_ref().expect("citation must be present");

    assert_eq!(
        citation.permalink,
        "https://docs.rs/tokio/1.38.0/tokio/index.html#asynchronous-runtime"
    );
    assert!(citation
        .markdown_citation
        .contains("https://docs.rs/tokio/1.38.0/tokio/index.html#asynchronous-runtime"));
    assert_eq!(citation.anchor.as_deref(), Some("asynchronous-runtime"));
}

#[test]
fn test_pdf_page_citation_generation() {
    let identity = SourceIdentity::new(
        "https://arxiv.org/pdf/1706.03762.pdf",
        "https://arxiv.org/pdf/1706.03762.pdf",
    );

    let mut attributes = BTreeMap::new();
    attributes.insert("page_number".to_string(), "4".to_string());

    let sec = CdmSection {
        section_id: "sec_page_4".to_string(),
        parent_id: None,
        title: Some("Attention Is All You Need".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("transformer_paper.pdf".to_string()),
        source_url: "https://arxiv.org/pdf/1706.03762.pdf".to_string(),
        line_range: Some(LineRange::new(1, 50)),
        byte_range: None,
        language: None,
        content_hash: "hash_pdf".to_string(),
        body: "Multi-head attention allows the model to jointly attend to information...\n"
            .to_string(),
        token_metrics: None,
        attributes,
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::DocumentPdf,
        "https://arxiv.org/pdf/1706.03762.pdf",
    )
    .title("Attention Is All You Need")
    .add_section(sec)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    assert!(!result.chunks.is_empty());

    let chunk = &result.chunks[0];
    let citation = chunk.citation.as_ref().expect("citation must be present");

    assert_eq!(
        citation.permalink,
        "https://arxiv.org/pdf/1706.03762.pdf#page=4"
    );
    assert_eq!(
        citation.display_citation,
        "Attention Is All You Need (p. 4)"
    );
    assert_eq!(citation.anchor.as_deref(), Some("page=4"));
}

#[test]
fn test_structured_spec_operation_citation() {
    let identity = SourceIdentity::new(
        "https://api.example.com/openapi.yaml",
        "https://api.example.com/openapi.yaml",
    );

    let mut attributes = BTreeMap::new();
    attributes.insert("operation_path".to_string(), "GET /api/v1/jobs".to_string());

    let sec = CdmSection {
        section_id: "sec_openapi_op".to_string(),
        parent_id: None,
        title: Some("List Jobs Operation".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("docs/openapi.yaml".to_string()),
        source_url: "https://api.example.com/openapi.yaml".to_string(),
        line_range: Some(LineRange::new(100, 150)),
        byte_range: None,
        language: Some("yaml".to_string()),
        content_hash: "hash_spec".to_string(),
        body: "/api/v1/jobs:\n  get:\n    summary: List all extraction jobs\n".to_string(),
        token_metrics: None,
        attributes,
    };

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::StructuredDocument,
        "https://api.example.com/openapi.yaml",
    )
    .title("Converter API Specification")
    .add_section(sec)
    .build();

    let result = chunk_document(&doc, ChunkingConfig::default()).expect("chunking succeeds");
    assert!(!result.chunks.is_empty());

    let chunk = &result.chunks[0];
    let citation = chunk.citation.as_ref().expect("citation must be present");

    assert_eq!(
        citation.display_citation,
        "docs/openapi.yaml > GET /api/v1/jobs"
    );
    assert_eq!(
        citation.permalink,
        "https://api.example.com/openapi.yaml#GET--api-v1-jobs"
    );
}

#[test]
fn test_local_file_fallback_citation() {
    let rust_text = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    let res = chunk_text(
        rust_text,
        Some("src/math.rs"),
        Some("rust"),
        ChunkingConfig::default(),
    )
    .expect("chunk_text succeeds");

    assert!(!res.chunks.is_empty());
    let citation = res.chunks[0]
        .citation
        .as_ref()
        .expect("citation must be present");

    assert!(citation.permalink.contains("src/math.rs"));
    assert!(citation.display_citation.contains("src/math.rs:"));
    assert!(citation.line_span.is_some());
}
