//! Integration tests for Markdown exporter (`crates/exporters/tests/markdown_export.rs`).

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
    export_markdown, export_markdown_to_writer, CodeFenceStyle, HierarchyStyle, MarkdownExporter,
    MarkdownOptions,
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
fn test_markdown_export_default() {
    let doc = create_sample_cdm_document();
    let exporter = MarkdownExporter::new();
    let md = exporter.export_to_string(&doc).unwrap();

    // 1. Frontmatter
    assert!(md.starts_with("---\n"));
    assert!(md.contains("title: Sample Repository\n"));
    assert!(md.contains("source_type: github_public\n"));
    assert!(md.contains("license: MIT\n"));
    assert!(md.contains("primary_language: Rust\n"));
    assert!(md.contains("git_commit: \"abc1234567890abcdef1234567890abcdef1234\"\n"));
    assert!(md.contains("total_sections: 2\n"));

    // 2. Metadata block
    assert!(md.contains("# Sample Repository\n"));
    assert!(md.contains("> **Source:** <https://github.com/example/sample-repo>"));
    assert!(md.contains("> **Type:** `github_public`"));
    assert!(md.contains("> **Authors:** Test Author"));
    assert!(md.contains("A sample repository for exporter testing."));

    // 3. Hierarchy tree
    assert!(md.contains("## Document & Directory Structure"));
    assert!(md.contains("sample-repo"));
    assert!(md.contains("├── src/"));
    assert!(md.contains("│   └── lib.rs (42 B, 3 lines)"));
    assert!(md.contains("└── README.md (64 B, 2 lines)"));

    // 4. Table of contents
    assert!(md.contains("## Table of Contents"));
    assert!(md.contains("- [src/lib.rs](#file-srclibrs)"));
    assert!(md.contains("- [README.md](#readmemd)"));

    // 5. Sections
    assert!(md.contains("## File: src/lib.rs"));
    assert!(md.contains(
        "> **Path:** `src/lib.rs` | **Language:** `rust` | **Lines:** 1-3 | **Bytes:** 42 B"
    ));
    assert!(md.contains("```rust\npub fn compute(x: u32) -> u32 {\n    x * 2\n}\n```"));
    assert!(md.contains("## README.md"));
    assert!(md.contains("Sample Repo is an exemplary project demonstrating AI-readiness."));

    // 6. Citations
    assert!(md.contains("## Sources & Citations"));
    assert!(md.contains("1. [Official Repository](https://github.com/example/sample-repo)"));

    // 7. Exclusions
    assert!(md.contains("## Excluded Files & Content"));
    assert!(md.contains("| `target/debug/build.log` | rule_pattern | `target/**` | 1.0 KB | Build artifact excluded |"));

    // 8. Completeness
    assert!(md.contains("## Completeness Accounting"));
    assert!(md.contains("- **Discovered Items:** 3"));
    assert!(md.contains("- **Extracted Items:** 2"));
    assert!(md.contains("- **Excluded Items:** 1"));
    assert!(md.contains("- **Completeness Percentage:** 66.7%"));
}

#[test]
fn test_markdown_export_minimal() {
    let doc = create_sample_cdm_document();
    let options = MarkdownOptions::minimal();
    let md = export_markdown(&doc, &options).unwrap();

    assert!(md.starts_with("---\n"));
    assert!(md.contains("## File: src/lib.rs"));
    assert!(md.contains("```rust\npub fn compute(x: u32) -> u32 {"));

    // Should NOT contain TOC, tree, metadata block, exclusions, or citations
    assert!(!md.contains("## Document & Directory Structure"));
    assert!(!md.contains("## Table of Contents"));
    assert!(!md.contains("## Sources & Citations"));
    assert!(!md.contains("## Excluded Files & Content"));
    assert!(!md.contains("## Completeness Accounting"));
}

#[test]
fn test_markdown_export_compact() {
    let doc = create_sample_cdm_document();
    let options = MarkdownOptions::compact();
    let md = export_markdown(&doc, &options).unwrap();

    // Compact uses bulleted list hierarchy
    assert!(md.contains("- **sample-repo/**"));
    assert!(md.contains("- **src/**"));
    assert!(md.contains("- `lib.rs` (42 B, 3 lines)"));
    // Has exclusions and citations
    assert!(md.contains("## Excluded Files & Content"));
    assert!(md.contains("## Sources & Citations"));
    // Does NOT have verbose TOC or per-section metadata
    assert!(!md.contains("## Table of Contents"));
    assert!(!md.contains("> **Path:** `src/lib.rs`"));
}

#[test]
fn test_fence_escalation_backticks() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");
    let code_with_backticks =
        "Here is an example markdown file:\n```rust\nfn inner() {}\n```\nDone.";

    let section = CdmSectionBuilder::new(
        "sec-001",
        "https://example.com/test.md",
        SectionKind::CodeFile,
        code_with_backticks,
    )
    .language("markdown")
    .title("test.md")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(section)
        .build();

    let md = export_markdown(&doc, &MarkdownOptions::default()).unwrap();

    // Inner triple backticks must cause outer fence to escalate to 4 backticks
    assert!(md.contains(
        "````markdown\nHere is an example markdown file:\n```rust\nfn inner() {}\n```\nDone.\n````"
    ));
}

#[test]
fn test_fence_escalation_tildes() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");
    let code_with_tildes = "Code with tildes:\n~~~sh\necho hello\n~~~\n";

    let section = CdmSectionBuilder::new(
        "sec-001",
        "https://example.com/test.sh",
        SectionKind::CodeFile,
        code_with_tildes,
    )
    .language("bash")
    .title("test.sh")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(section)
        .build();

    let options = MarkdownOptions::default().with_code_fence_style(CodeFenceStyle::Tildes);
    let md = export_markdown(&doc, &options).unwrap();

    // Inner triple tildes must cause outer fence to escalate to 4 tildes
    assert!(md.contains("~~~~bash\nCode with tildes:\n~~~sh\necho hello\n~~~\n~~~~"));
}

#[test]
fn test_admonition_rendering() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");

    let mut section = CdmSectionBuilder::new(
        "sec-adm",
        "https://example.com/doc",
        SectionKind::Admonition,
        "Make sure to set the environment variable.\nOtherwise startup fails.",
    )
    .title("Important Warning")
    .attribute("admonition_type", "warning")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(section.clone())
        .build();

    let md = export_markdown(&doc, &MarkdownOptions::default()).unwrap();
    assert!(md.contains("> [!WARNING]"));
    assert!(md.contains("> Make sure to set the environment variable."));
    assert!(md.contains("> Otherwise startup fails."));

    // Default note admonition
    section.attributes.clear();
    let doc_note = CdmDocumentBuilder::new(
        SourceIdentity::new("https://example.com", "https://example.com"),
        SourceType::WebpageStatic,
        "https://example.com",
    )
    .add_section(section)
    .build();

    let md_note = export_markdown(&doc_note, &MarkdownOptions::default()).unwrap();
    assert!(md_note.contains("> [!NOTE]"));
}

#[test]
fn test_line_numbering() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");
    let code = "fn one() {}\nfn two() {}\nfn three() {}";

    let section = CdmSectionBuilder::new(
        "sec-code",
        "https://example.com/file.rs",
        SectionKind::CodeFile,
        code,
    )
    .language("rust")
    .line_range(10, 12)
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(section)
        .build();

    let options = MarkdownOptions::default().with_line_numbers(true);
    let md = export_markdown(&doc, &options).unwrap();

    assert!(md.contains("10 | fn one() {}"));
    assert!(md.contains("11 | fn two() {}"));
    assert!(md.contains("12 | fn three() {}"));
}

#[test]
fn test_token_budget_truncation() {
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

    // Budget of 15 tokens will fit sec1 (~14 tokens) but reject sec2
    let options = MarkdownOptions::default().with_max_token_budget(Some(15));
    let mut buffer = Vec::new();
    let metrics = export_markdown_to_writer(&doc, &options, &mut buffer).unwrap();

    let md = String::from_utf8(buffer).unwrap();

    assert!(metrics.truncated);
    assert_eq!(metrics.sections_exported, 1);
    assert!(md.contains("Section 1"));
    assert!(!md.contains("Section 2"));
    assert!(md.contains("Token Budget Limit Exceeded"));
}

#[test]
fn test_streaming_writer_equivalence() {
    let doc = create_sample_cdm_document();
    let options = MarkdownOptions::default();

    let string_output = export_markdown(&doc, &options).unwrap();

    let mut writer_bytes = Vec::new();
    let metrics = export_markdown_to_writer(&doc, &options, &mut writer_bytes).unwrap();
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
fn test_hierarchy_tree_styles() {
    let doc = create_sample_cdm_document();

    // Ascii style
    let ascii_md = export_markdown(
        &doc,
        &MarkdownOptions::default().with_hierarchy_style(HierarchyStyle::AsciiTree),
    )
    .unwrap();
    assert!(ascii_md.contains("|-- src/"));
    assert!(ascii_md.contains("\\-- README.md"));

    // None style
    let none_md = export_markdown(
        &doc,
        &MarkdownOptions::default().with_hierarchy_style(HierarchyStyle::None),
    )
    .unwrap();
    assert!(!none_md.contains("## Document & Directory Structure"));
}

#[test]
fn test_yaml_frontmatter_escaping() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .metadata(DocumentMetadata {
            title: Some("Project: \"The Future\" #1 [AI]".to_string()),
            license: Some("Apache-2.0 WITH LLVM-exception".to_string()),
            tags: vec!["tag:one".to_string(), "tag#two".to_string()],
            ..Default::default()
        })
        .build();

    let md = export_markdown(&doc, &MarkdownOptions::default()).unwrap();
    assert!(md.contains("title: \"Project: \\\"The Future\\\" #1 [AI]\""));
    assert!(md.contains("license: Apache-2.0 WITH LLVM-exception"));
    assert!(md.contains("  - \"tag:one\""));
    assert!(md.contains("  - \"tag#two\""));
}

#[test]
fn test_empty_document_handling() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let md = export_markdown(&doc, &MarkdownOptions::default()).unwrap();
    assert!(md.starts_with("---\n"));
    assert!(md.contains("total_sections: 0\n"));
    assert!(md.contains("# Context Document\n"));
    // Since sections are empty, Table of Contents should not be rendered
    assert!(!md.contains("## Table of Contents"));
}

#[test]
fn test_table_and_snippet_export() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");

    let table_body = "| Column A | Column B |\n| :--- | :--- |\n| Val 1 | Val 2 |";
    let sec_table = CdmSectionBuilder::new(
        "sec-tbl",
        "https://example.com/table",
        SectionKind::Table,
        table_body,
    )
    .title("Feature Matrix")
    .build();

    let sec_snippet = CdmSectionBuilder::new(
        "sec-snip",
        "https://example.com/snip",
        SectionKind::CodeSnippet,
        "let x = 42;",
    )
    .title("Snippet One")
    .language("rust")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(sec_table)
        .add_section(sec_snippet)
        .build();

    let md = export_markdown(&doc, &MarkdownOptions::default()).unwrap();
    assert!(md.contains("## Table: Feature Matrix\n"));
    assert!(md.contains("| Column A | Column B |"));
    assert!(md.contains("## Snippet: Snippet One\n"));
    assert!(md.contains("```rust\nlet x = 42;\n```"));
}

#[test]
fn test_custom_heading_levels() {
    let identity = SourceIdentity::new("https://example.com", "https://example.com");
    let sec = CdmSectionBuilder::new(
        "sec-1",
        "https://example.com/1",
        SectionKind::Prose,
        "Body content here.",
    )
    .title("My Section")
    .build();

    let doc = CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
        .add_section(sec)
        .build();

    let options_h1 = MarkdownOptions::default().with_section_heading_level(1);
    let md_h1 = export_markdown(&doc, &options_h1).unwrap();
    assert!(md_h1.contains("# My Section\n"));

    let options_h3 = MarkdownOptions::default().with_section_heading_level(3);
    let md_h3 = export_markdown(&doc, &options_h3).unwrap();
    assert!(md_h3.contains("### My Section\n"));
}

#[test]
fn test_options_serde_roundtrip() {
    let options = MarkdownOptions::default()
        .with_frontmatter(false)
        .with_hierarchy_style(HierarchyStyle::AsciiTree)
        .with_code_fence_style(CodeFenceStyle::Tildes)
        .with_line_numbers(true)
        .with_max_token_budget(Some(4096));

    let json_str = serde_json::to_string(&options).unwrap();
    let deserialized: MarkdownOptions = serde_json::from_str(&json_str).unwrap();

    assert_eq!(options, deserialized);
}
