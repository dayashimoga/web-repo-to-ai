//! Integration tests for Plain-Text exporter (`crates/exporters/tests/plain_text_export.rs`).

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_plain_text, export_plain_text_to_writer, HierarchyStyle, PlainTextExporter,
    PlainTextOptions,
};

fn create_sample_cdm_document() -> web_repo_to_ai_cdm::model::CdmDocument {
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
fn test_plain_text_export_default() {
    let doc = create_sample_cdm_document();
    let exporter = PlainTextExporter::new();
    let txt = exporter.export_to_string(&doc).unwrap();

    // 1. Header Banner
    assert!(txt.contains("DOCUMENT: Sample Repository"));
    assert!(txt.contains("SOURCE:   https://github.com/example/sample-repo"));
    assert!(txt.contains("TYPE:     github_public"));
    assert!(txt.contains("LICENSE:  MIT"));
    assert!(txt.contains("LANGUAGE: Rust"));
    assert!(txt.contains("AUTHORS:  Test Author"));
    assert!(txt.contains("COMMIT:   abc1234567890abcdef1234567890abcdef1234"));
    assert!(txt.contains("METRICS:  2 sections"));
    assert!(txt.contains("COMPLETENESS: 66.7% (2 extracted / 3 discovered items)"));
    assert!(txt.contains("A sample repository for exporter testing."));

    // 2. Hierarchy tree (ASCII default)
    assert!(txt.contains("DOCUMENT & DIRECTORY STRUCTURE"));
    assert!(txt.contains("sample-repo"));
    assert!(txt.contains("|-- src/"));
    assert!(txt.contains("|   \\-- lib.rs (42 B, 3 lines)"));
    assert!(txt.contains("\\-- README.md (64 B, 2 lines)"));

    // 3. Table of Contents
    assert!(txt.contains("TABLE OF CONTENTS"));
    assert!(txt.contains(" 1. FILE: src/lib.rs (rust, 3 lines, 42 B"));
    assert!(txt.contains(" 2. README.md (markdown, 2 lines, ~16 tokens)"));

    // 4. Sections
    assert!(txt.contains("SECTION 1: FILE: src/lib.rs"));
    assert!(txt.contains("Path: src/lib.rs | Language: rust | Lines: 1-3 | Bytes: 42 B"));
    assert!(txt.contains("pub fn compute(x: u32) -> u32 {\n    x * 2\n}"));
    // Ensure NO markdown backticks in plain text code body
    assert!(!txt.contains("```"));

    assert!(txt.contains("SECTION 2: PROSE: README.md"));
    assert!(txt.contains("Sample Repo is an exemplary project demonstrating AI-readiness."));

    // 5. Sources & Citations
    assert!(txt.contains("SOURCES & CITATIONS"));
    assert!(txt.contains("1. Official Repository: https://github.com/example/sample-repo"));

    // 6. Excluded Files & Content
    assert!(txt.contains("EXCLUDED FILES & CONTENT"));
    assert!(txt.contains("1. Path: target/debug/build.log"));
    assert!(txt.contains("Reason: rule_pattern | Pattern: target/** | Size: 1.0 KB"));
    assert!(txt.contains("Details: Build artifact excluded"));

    // 7. Completeness Accounting
    assert!(txt.contains("COMPLETENESS ACCOUNTING"));
    assert!(txt.contains("Status:       completed"));
    assert!(txt.contains("Discovered:   3"));
    assert!(txt.contains("Extracted:    2"));
    assert!(txt.contains("Excluded:     1"));
    assert!(txt.contains("Completeness: 66.7%"));
}

#[test]
fn test_plain_text_export_minimal() {
    let doc = create_sample_cdm_document();
    let options = PlainTextOptions::minimal();
    let txt = export_plain_text(&doc, &options).unwrap();

    // Raw sections only
    assert!(txt.contains("pub fn compute(x: u32) -> u32 {"));
    assert!(txt.contains("Sample Repo is an exemplary project demonstrating AI-readiness."));

    // Should NOT contain header banner, TOC, tree, section banners, citations, or exclusions
    assert!(!txt.contains("DOCUMENT: Sample Repository"));
    assert!(!txt.contains("DOCUMENT & DIRECTORY STRUCTURE"));
    assert!(!txt.contains("TABLE OF CONTENTS"));
    assert!(!txt.contains("SECTION 1: FILE: src/lib.rs"));
    assert!(!txt.contains("SOURCES & CITATIONS"));
    assert!(!txt.contains("EXCLUDED FILES & CONTENT"));
    assert!(!txt.contains("COMPLETENESS ACCOUNTING"));
}

#[test]
fn test_plain_text_export_compact() {
    let doc = create_sample_cdm_document();
    let options = PlainTextOptions::compact();
    let txt = export_plain_text(&doc, &options).unwrap();

    // Has header banner
    assert!(txt.contains("DOCUMENT: Sample Repository"));
    // Has section headers
    assert!(txt.contains("SECTION 1: FILE: src/lib.rs"));
    // Has exclusions and citations
    assert!(txt.contains("SOURCES & CITATIONS"));
    assert!(txt.contains("EXCLUDED FILES & CONTENT"));
    // Does NOT have TOC
    assert!(!txt.contains("TABLE OF CONTENTS"));
    // Does NOT have completeness block
    assert!(!txt.contains("COMPLETENESS ACCOUNTING"));
}

#[test]
fn test_plain_text_line_numbering() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");
    let code = "fn alpha() {}\nfn beta() {}\nfn gamma() {}";

    let section = CdmSectionBuilder::new(
        "sec-code",
        "https://example.com/math.rs",
        SectionKind::CodeFile,
        code,
    )
    .language("rust")
    .line_range(50, 52)
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(section)
        .build();

    let options = PlainTextOptions::default().with_line_numbers(true);
    let txt = export_plain_text(&doc, &options).unwrap();

    assert!(txt.contains("50 | fn alpha() {}"));
    assert!(txt.contains("51 | fn beta() {}"));
    assert!(txt.contains("52 | fn gamma() {}"));
}

#[test]
fn test_plain_text_token_budget_truncation() {
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

    let options = PlainTextOptions::default().with_max_token_budget(Some(15));
    let mut buffer = Vec::new();
    let metrics = export_plain_text_to_writer(&doc, &options, &mut buffer).unwrap();
    let txt = String::from_utf8(buffer).unwrap();

    assert!(metrics.truncated);
    assert_eq!(metrics.sections_exported, 1);
    assert!(txt.contains("Section 1"));
    assert!(!txt.contains("SECTION 2: PROSE: Section 2"));
    assert!(txt.contains(
        "[WARNING: Token budget limit of 15 tokens reached. Remaining sections omitted.]"
    ));
}

#[test]
fn test_plain_text_streaming_writer_equivalence() {
    let doc = create_sample_cdm_document();
    let options = PlainTextOptions::default();

    let string_output = export_plain_text(&doc, &options).unwrap();

    let mut writer_bytes = Vec::new();
    let metrics = export_plain_text_to_writer(&doc, &options, &mut writer_bytes).unwrap();
    let writer_output = String::from_utf8(writer_bytes).unwrap();

    assert_eq!(string_output, writer_output);
    assert_eq!(metrics.bytes_written, string_output.len());
    assert_eq!(metrics.lines_written, string_output.matches('\n').count());
    assert_eq!(metrics.sections_exported, 2);
    assert_eq!(metrics.citations_exported, 1);
    assert_eq!(metrics.exclusions_exported, 1);
    assert!(!metrics.truncated);
}

#[test]
fn test_plain_text_hierarchy_styles() {
    let doc = create_sample_cdm_document();

    // Unicode style
    let unicode_txt = export_plain_text(
        &doc,
        &PlainTextOptions::default().with_hierarchy_style(HierarchyStyle::UnicodeTree),
    )
    .unwrap();
    assert!(unicode_txt.contains("├── src/"));
    assert!(unicode_txt.contains("└── README.md"));

    // Bulleted list style
    let bulleted_txt = export_plain_text(
        &doc,
        &PlainTextOptions::default().with_hierarchy_style(HierarchyStyle::BulletedList),
    )
    .unwrap();
    assert!(bulleted_txt.contains("- sample-repo/"));
    assert!(bulleted_txt.contains("  - src/"));
    assert!(bulleted_txt.contains("  - lib.rs"));

    // None style
    let none_txt = export_plain_text(
        &doc,
        &PlainTextOptions::default().with_hierarchy_style(HierarchyStyle::None),
    )
    .unwrap();
    assert!(!none_txt.contains("DOCUMENT & DIRECTORY STRUCTURE"));
}

#[test]
fn test_plain_text_custom_divider() {
    let doc = create_sample_cdm_document();
    let options = PlainTextOptions::default().with_divider('*', 50);
    let txt = export_plain_text(&doc, &options).unwrap();

    let expected_divider = "*".repeat(50);
    assert!(txt.contains(&expected_divider));
}

#[test]
fn test_plain_text_empty_document() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let txt = export_plain_text(&doc, &PlainTextOptions::default()).unwrap();
    assert!(txt.contains("DOCUMENT: Context Document"));
    assert!(txt.contains("METRICS:  0 sections"));
    // Since sections are empty, Table of Contents should not be rendered
    assert!(!txt.contains("TABLE OF CONTENTS"));
}

#[test]
fn test_plain_text_options_serde_roundtrip() {
    let options = PlainTextOptions::default()
        .with_header_banner(false)
        .with_hierarchy_style(HierarchyStyle::UnicodeTree)
        .with_divider('~', 64)
        .with_line_numbers(true)
        .with_max_token_budget(Some(2048));

    let json_str = serde_json::to_string(&options).unwrap();
    let deserialized: PlainTextOptions = serde_json::from_str(&json_str).unwrap();

    assert_eq!(options, deserialized);
}
