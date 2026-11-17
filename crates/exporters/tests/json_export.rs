//! Integration tests for JSON exporter (`crates/exporters/tests/json_export.rs`).

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_cdm::validator::CdmValidator;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_json, export_json_to_writer, JsonExportOptions, JsonExporter, JsonFormatFlavor,
    StructuredContextEnvelope,
};

fn create_sample_cdm_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/example/sample-repo".to_string(),
        canonical_url: "https://github.com/example/sample-repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("sample-repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let section_code = CdmSectionBuilder::new(
        "sec-001",
        "https://github.com/example/sample-repo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn compute(x: u32) -> u32 {\n    x * 2\n}\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 42)
    .build();

    let section_readme = CdmSectionBuilder::new(
        "sec-002",
        "https://github.com/example/sample-repo/blob/main/README.md",
        SectionKind::Prose,
        "Sample Repo is an exemplary project demonstrating AI-readiness.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 2)
    .build();

    let mut root_node = HierarchyNode::root("sample-repo");
    let mut src_dir = HierarchyNode::directory("dir-src", "src", "src");
    src_dir.add_child(HierarchyNode::file(
        "file-lib",
        "lib.rs",
        "src/lib.rs",
        Some("sec-001".to_string()),
        Some(42),
        Some(3),
    ));
    root_node.add_child(src_dir);
    root_node.add_child(HierarchyNode::file(
        "file-readme",
        "README.md",
        "README.md",
        Some("sec-002".to_string()),
        Some(64),
        Some(2),
    ));

    let citation = Citation::new(
        "cit-001",
        "https://github.com/example/sample-repo",
        Some("Official Repository".to_string()),
        None,
    );

    let exclusion = ExclusionRecord::new(
        "target/debug/build.log",
        ExclusionReason::RulePattern,
        Some("target/**".to_string()),
        Some("Build artifact excluded".to_string()),
        Some(1024),
    );

    let completeness = CompletenessReport::new(3, 2, 2, 1, 0, 0);

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/sample-repo",
    )
    .document_id(DocumentId::new())
    .retrieved_at(Utc::now())
    .revision(RevisionInfo {
        git_commit: Some("abc1234567890abcdef1234567890abcdef1234".to_string()),
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .metadata(DocumentMetadata {
        title: Some("Sample Repository".to_string()),
        description: Some("A sample repository for exporter testing.".to_string()),
        license: Some("MIT".to_string()),
        authors: vec!["Test Author".to_string()],
        primary_language: Some("Rust".to_string()),
        language_breakdown: BTreeMap::from([("Rust".to_string(), 42)]),
        tags: vec!["ai".to_string(), "converter".to_string()],
        ..Default::default()
    })
    .hierarchy(root_node)
    .add_section(section_code)
    .add_section(section_readme)
    .add_citation(citation)
    .add_exclusion(exclusion)
    .completeness(completeness)
    .build()
}

#[test]
fn test_json_export_canonical_cdm_default() {
    let doc = create_sample_cdm_document();
    let exporter = JsonExporter::new();
    let json_str = exporter.export_to_string(&doc).unwrap();

    // Verify deserialization back to CdmDocument
    let deserialized: CdmDocument = web_repo_to_ai_cdm::from_json(&json_str).unwrap();
    assert_eq!(deserialized.schema_version, "1.0.0");
    assert_eq!(deserialized.canonical_url, doc.canonical_url);
    assert_eq!(deserialized.sections.len(), 2);
    assert_eq!(deserialized.citations.len(), 1);
    assert_eq!(deserialized.exclusions.len(), 1);

    // Validate invariant conformance
    let report = CdmValidator::default().validate(&deserialized);
    assert!(report.is_valid());
}

#[test]
fn test_json_export_structured_context_default() {
    let doc = create_sample_cdm_document();
    let options = JsonExportOptions::structured();
    let json_str = export_json(&doc, &options).unwrap();

    let envelope: StructuredContextEnvelope = serde_json::from_str(&json_str).unwrap();

    assert_eq!(envelope.title.as_deref(), Some("Sample Repository"));
    assert_eq!(
        envelope.canonical_url,
        "https://github.com/example/sample-repo"
    );
    assert_eq!(envelope.source_type, "github_public");
    assert_eq!(envelope.license.as_deref(), Some("MIT"));
    assert_eq!(envelope.primary_language.as_deref(), Some("Rust"));
    assert_eq!(envelope.authors, vec!["Test Author".to_string()]);
    assert_eq!(
        envelope.description.as_deref(),
        Some("A sample repository for exporter testing.")
    );

    assert_eq!(envelope.metrics.total_sections, 2);
    assert!(envelope.metrics.total_words > 0);
    assert!(envelope.metrics.estimated_tokens > 0);

    let completeness = envelope.completeness.unwrap();
    assert_eq!(completeness.discovered_count, 3);
    assert_eq!(completeness.extracted_count, 2);
    assert_eq!(completeness.excluded_count, 1);
    assert!((completeness.percentage - 66.666).abs() < 1.0);

    let hierarchy = envelope.hierarchy.unwrap();
    assert_eq!(hierarchy.name, "sample-repo");
    assert_eq!(hierarchy.children.len(), 2);

    assert_eq!(envelope.sections.len(), 2);
    assert_eq!(envelope.sections[0].id, "sec-001");
    assert_eq!(envelope.sections[0].kind, "code_file");
    assert_eq!(envelope.sections[0].language.as_deref(), Some("rust"));
    assert_eq!(envelope.sections[0].lines, Some([1, 3]));
    assert_eq!(envelope.sections[0].bytes, Some(42));
    assert!(envelope.sections[0].content.contains("pub fn compute"));

    assert_eq!(envelope.citations.len(), 1);
    assert_eq!(
        envelope.citations[0].url,
        "https://github.com/example/sample-repo"
    );

    assert_eq!(envelope.exclusions.len(), 1);
    assert_eq!(envelope.exclusions[0].path, "target/debug/build.log");
    assert_eq!(envelope.exclusions[0].reason, "rule_pattern");

    assert!(!envelope.truncated);
}

#[test]
fn test_json_export_minified() {
    let doc = create_sample_cdm_document();
    let options = JsonExportOptions::compact();
    let json_str = export_json(&doc, &options).unwrap();

    // Minified should not contain multi-line pretty whitespace
    assert!(!json_str.contains("\n  "));

    // Deserialization must succeed
    let deserialized: CdmDocument = web_repo_to_ai_cdm::from_json(&json_str).unwrap();
    assert_eq!(deserialized.document_id, doc.document_id);
    assert_eq!(deserialized.sections.len(), 2);
}

#[test]
fn test_json_export_token_budget_truncation() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");

    let sec1 = CdmSectionBuilder::new(
        "sec-1",
        "https://example.com/1",
        SectionKind::Prose,
        "First section with a modest amount of prose content.",
    )
    .title("Section 1")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-2",
        "https://example.com/2",
        SectionKind::Prose,
        "Second section that pushes us far beyond the tiny budget limit.",
    )
    .title("Section 2")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(sec1)
        .add_section(sec2)
        .build();

    // Structured context truncation
    let options_structured = JsonExportOptions::structured().with_max_token_budget(Some(15));
    let mut buffer = Vec::new();
    let metrics = export_json_to_writer(&doc, &options_structured, &mut buffer).unwrap();
    let json_str = String::from_utf8(buffer).unwrap();

    assert!(metrics.truncated);
    assert_eq!(metrics.sections_exported, 1);

    let envelope: StructuredContextEnvelope = serde_json::from_str(&json_str).unwrap();
    assert!(envelope.truncated);
    assert_eq!(envelope.sections.len(), 1);
    assert_eq!(envelope.sections[0].id, "sec-1");

    // Canonical CDM truncation
    let options_canonical = JsonExportOptions::canonical().with_max_token_budget(Some(15));
    let cdm_json = export_json(&doc, &options_canonical).unwrap();
    let pruned_cdm: CdmDocument = web_repo_to_ai_cdm::from_json(&cdm_json).unwrap();
    assert_eq!(pruned_cdm.sections.len(), 1);
    assert_eq!(pruned_cdm.sections[0].section_id, "sec-1");
}

#[test]
fn test_json_export_field_filtering() {
    let doc = create_sample_cdm_document();
    let options = JsonExportOptions::structured()
        .with_metadata(false)
        .with_hierarchy(false)
        .with_sections(false)
        .with_citations(false)
        .with_exclusions(false)
        .with_completeness(false);

    let json_str = export_json(&doc, &options).unwrap();
    let envelope: StructuredContextEnvelope = serde_json::from_str(&json_str).unwrap();

    assert!(envelope.title.is_none());
    assert!(envelope.license.is_none());
    assert!(envelope.primary_language.is_none());
    assert!(envelope.authors.is_empty());
    assert!(envelope.description.is_none());
    assert!(envelope.hierarchy.is_none());
    assert!(envelope.sections.is_empty());
    assert!(envelope.citations.is_empty());
    assert!(envelope.exclusions.is_empty());
    assert!(envelope.completeness.is_none());
}

#[test]
fn test_json_streaming_writer_equivalence() {
    let doc = create_sample_cdm_document();

    for flavor in [
        JsonFormatFlavor::CanonicalCdm,
        JsonFormatFlavor::StructuredContext,
        JsonFormatFlavor::Minified,
    ] {
        let options = JsonExportOptions::default().with_flavor(flavor);

        let string_output = export_json(&doc, &options).unwrap();

        let mut writer_bytes = Vec::new();
        let metrics = export_json_to_writer(&doc, &options, &mut writer_bytes).unwrap();
        let writer_output = String::from_utf8(writer_bytes).unwrap();

        assert_eq!(string_output, writer_output);
        assert_eq!(metrics.bytes_written, string_output.len());
        assert_eq!(metrics.lines_written, string_output.matches('\n').count());
        assert_eq!(metrics.sections_exported, 2);
        assert!(!metrics.truncated);
    }
}

#[test]
fn test_json_empty_document_handling() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let json_canonical = export_json(&doc, &JsonExportOptions::canonical()).unwrap();
    let parsed_cdm: CdmDocument = web_repo_to_ai_cdm::from_json(&json_canonical).unwrap();
    assert_eq!(parsed_cdm.sections.len(), 0);

    let json_structured = export_json(&doc, &JsonExportOptions::structured()).unwrap();
    let parsed_env: StructuredContextEnvelope = serde_json::from_str(&json_structured).unwrap();
    assert_eq!(parsed_env.sections.len(), 0);
    assert_eq!(parsed_env.metrics.total_sections, 0);
}

#[test]
fn test_json_options_serde_roundtrip() {
    let options = JsonExportOptions::default()
        .with_flavor(JsonFormatFlavor::StructuredContext)
        .with_pretty(false)
        .with_metadata(false)
        .with_max_token_budget(Some(8192));

    let json_str = serde_json::to_string(&options).unwrap();
    let deserialized: JsonExportOptions = serde_json::from_str(&json_str).unwrap();

    assert_eq!(options, deserialized);
}
