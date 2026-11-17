//! Integration tests for JSON Lines (`.jsonl`) exporter (`crates/exporters/tests/jsonl_export.rs`).

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::metadata::DocumentMetadata;
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::{CdmSection, SectionKind};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_jsonl, export_jsonl_to_writer, FineTuningChatRecord, FineTuningPromptCompletionRecord,
    JsonlExporter, JsonlFlavor, JsonlOptions, RagChunkRecord,
};

fn create_sample_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/example/sample-repo".to_string(),
        canonical_url: "https://github.com/example/sample-repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("sample-repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/example/sample-repo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 48)
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/sample-repo/blob/main/README.md",
        SectionKind::Prose,
        "# Sample Repo\n\nA simple calculator library in Rust.",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 3)
    .byte_range(0, 50)
    .build();

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/sample-repo",
    )
    .document_id(DocumentId::new())
    .revision(RevisionInfo {
        git_commit: Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .metadata(DocumentMetadata {
        title: Some("Sample Repository".to_string()),
        description: Some("Test repository for JSONL exporter validation".to_string()),
        primary_language: Some("Rust".to_string()),
        license: Some("MIT".to_string()),
        ..Default::default()
    })
    .add_section(s1)
    .add_section(s2)
    .build()
}

#[test]
fn test_jsonl_export_rag_chunks_default() {
    let doc = create_sample_document();
    let options = JsonlOptions::default();
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(metrics.sections_exported, 2);
    assert_eq!(metrics.lines_written, 2);
    assert!(!metrics.truncated);

    // Verify line 1 is valid RagChunkRecord
    let chunk1: RagChunkRecord = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(chunk1.id, "sec-1");
    assert_eq!(chunk1.document_title.as_deref(), Some("Sample Repository"));
    assert_eq!(chunk1.source_url, "https://github.com/example/sample-repo");
    assert_eq!(chunk1.path.as_deref(), Some("src/lib.rs"));
    assert_eq!(chunk1.kind, "code_file");
    assert_eq!(chunk1.language.as_deref(), Some("rust"));
    assert_eq!(chunk1.line_range, Some([1, 3]));
    assert_eq!(chunk1.byte_range, Some([0, 48]));
    assert!(chunk1.content.contains("pub fn add"));
    assert!(!chunk1.content_hash.is_empty());

    // Verify line 2 is valid RagChunkRecord
    let chunk2: RagChunkRecord = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(chunk2.id, "sec-2");
    assert_eq!(chunk2.kind, "prose");
    assert_eq!(chunk2.language.as_deref(), Some("markdown"));
    assert!(chunk2.content.contains("# Sample Repo"));
}

#[test]
fn test_jsonl_export_fine_tuning_chat() {
    let doc = create_sample_document();
    let options = JsonlOptions::chat_fine_tuning(Some("Custom instruction assistant."));
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(metrics.sections_exported, 2);

    let record1: FineTuningChatRecord = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(record1.messages.len(), 3);
    assert_eq!(record1.messages[0].role, "system");
    assert_eq!(record1.messages[0].content, "Custom instruction assistant.");
    assert_eq!(record1.messages[1].role, "user");
    assert!(record1.messages[1].content.contains("src/lib.rs"));
    assert_eq!(record1.messages[2].role, "assistant");
    assert!(record1.messages[2].content.contains("pub fn add"));

    let record2: FineTuningChatRecord = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(record2.messages.len(), 3);
    assert_eq!(record2.messages[0].role, "system");
    assert!(record2.messages[1].content.contains("README.md"));
    assert!(record2.messages[2].content.contains("# Sample Repo"));
}

#[test]
fn test_jsonl_export_fine_tuning_prompt_completion() {
    let doc = create_sample_document();
    let options = JsonlOptions::prompt_completion_fine_tuning();
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(metrics.sections_exported, 2);

    let record1: FineTuningPromptCompletionRecord = serde_json::from_str(lines[0]).unwrap();
    assert!(record1.prompt.contains("Document: Sample Repository"));
    assert!(record1.prompt.contains("Path: src/lib.rs"));
    assert!(record1.completion.contains("pub fn add"));

    let record2: FineTuningPromptCompletionRecord = serde_json::from_str(lines[1]).unwrap();
    assert!(record2.prompt.contains("Path: README.md"));
    assert!(record2.completion.contains("# Sample Repo"));
}

#[test]
fn test_jsonl_export_canonical_sections() {
    let doc = create_sample_document();
    let options = JsonlOptions::canonical();
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(metrics.sections_exported, 2);

    // Each line must deserialize into a lossless CdmSection
    let sec1: CdmSection = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(sec1.section_id, "sec-1");
    assert_eq!(sec1.title.as_deref(), Some("src/lib.rs"));
    assert_eq!(sec1.original_path.as_deref(), Some("src/lib.rs"));
    assert!(sec1.body.contains("pub fn add"));

    let sec2: CdmSection = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(sec2.section_id, "sec-2");
    assert_eq!(sec2.title.as_deref(), Some("README.md"));
    assert!(sec2.body.contains("# Sample Repo"));
}

#[test]
fn test_jsonl_export_token_budget_truncation() {
    let doc = create_sample_document();
    // Set tight budget allowing only 1 section
    let options = JsonlOptions::default().with_max_token_budget(Some(1));
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    assert!(metrics.truncated);
    assert_eq!(metrics.sections_exported, 0);
    assert!(jsonl.is_empty());
}

#[test]
fn test_jsonl_streaming_writer_equivalence() {
    let doc = create_sample_document();
    let options = JsonlOptions::default();

    let (in_memory_str, in_memory_metrics) = export_jsonl(&doc, &options).unwrap();

    let mut stream_buf = Vec::new();
    let stream_metrics = export_jsonl_to_writer(&doc, &options, &mut stream_buf).unwrap();
    let stream_str = String::from_utf8(stream_buf).unwrap();

    assert_eq!(in_memory_str, stream_str);
    assert_eq!(in_memory_metrics, stream_metrics);
}

#[test]
fn test_jsonl_empty_document_handling() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let options = JsonlOptions::default();
    let (jsonl, metrics) = export_jsonl(&doc, &options).unwrap();

    assert!(jsonl.is_empty());
    assert_eq!(metrics.sections_exported, 0);
    assert_eq!(metrics.lines_written, 0);
    assert_eq!(metrics.bytes_written, 0);
}

#[test]
fn test_jsonl_options_serde_roundtrip() {
    let options = JsonlOptions::default()
        .with_flavor(JsonlFlavor::FineTuningChat)
        .with_system_prompt("Test system prompt")
        .with_max_token_budget(Some(1000));

    let json = serde_json::to_string(&options).unwrap();
    let deserialized: JsonlOptions = serde_json::from_str(&json).unwrap();

    assert_eq!(options, deserialized);
}

#[test]
fn test_jsonl_exporter_builder_api() {
    let doc = create_sample_document();
    let exporter = JsonlExporter::new();
    let (jsonl1, m1) = exporter.export_to_string(&doc).unwrap();

    let mut buf = Vec::new();
    let exporter_writer = JsonlExporter::with_options(JsonlOptions::default());
    let m2 = exporter_writer.export_to_writer(&doc, &mut buf).unwrap();

    assert_eq!(jsonl1, String::from_utf8(buf).unwrap());
    assert_eq!(m1, m2);
}
