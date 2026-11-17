//! Integration tests for HTML document and fragment exporter (`crates/exporters/tests/html_export.rs`).

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_html, export_html_to_writer, HierarchyStyle, HtmlExporter, HtmlOptions,
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
        "pub fn hello() -> &'static str {\n    \"Hello, world!\"\n}",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 56)
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/example/sample-repo/blob/main/README.md",
        SectionKind::Prose,
        "# Sample Repo\n\nThis is a sample project with <tags> & symbols.",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 3)
    .byte_range(0, 60)
    .build();

    let mut root = HierarchyNode::root("sample-repo");
    let mut src_dir = HierarchyNode::directory("src-dir", "src", "src");
    let lib_file = HierarchyNode::file(
        "lib-file",
        "lib.rs",
        "src/lib.rs",
        Some("sec-1".to_string()),
        Some(56),
        Some(3),
    );
    src_dir.add_child(lib_file);
    root.add_child(src_dir);

    let citation = Citation::new(
        "cite-1",
        "https://example.com/docs",
        Some("Example Documentation".to_string()),
        None,
    );

    let exclusion = ExclusionRecord::new(
        "target/debug/build.log",
        ExclusionReason::BinaryFile,
        Some("target/**".to_string()),
        Some("Build artifact".to_string()),
        Some(1024),
    );

    let completeness = CompletenessReport::new(3, 2, 2, 1, 0, 0);

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
        title: Some("Sample Repository <Production>".to_string()),
        description: Some("Test repository for HTML exporter validation".to_string()),
        primary_language: Some("Rust".to_string()),
        license: Some("MIT".to_string()),
        authors: vec!["Alice Dev".to_string(), "Bob Architect".to_string()],
        language_breakdown: BTreeMap::from([("Rust".to_string(), 56)]),
        ..Default::default()
    })
    .hierarchy(root)
    .add_section(s1)
    .add_section(s2)
    .add_citation(citation)
    .add_exclusion(exclusion)
    .completeness(completeness)
    .build()
}

#[test]
fn test_html_export_standalone_default() {
    let doc = create_sample_document();
    let options = HtmlOptions::default();
    let (html, metrics) = export_html(&doc, &options).unwrap();

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html lang=\"en\">"));
    assert!(html.contains("<title>Sample Repository &lt;Production&gt;</title>"));
    assert!(html.contains("<style>"));
    assert!(html.contains("Sample Repository &lt;Production&gt;</h1>"));
    assert!(html.contains("Alice Dev, Bob Architect"));
    assert!(html.contains("abcdef1234567890abcdef1234567890abcdef12"));
    assert!(html.contains("src/lib.rs"));
    assert!(html.contains("README.md"));
    assert!(html.contains("Directory &amp; Structure Tree"));
    assert!(html.contains("Table of Contents"));
    assert!(html.contains("<details class=\"section-details\" open>"));
    assert!(html.contains("Example Documentation"));
    assert!(html.contains("target/debug/build.log"));
    assert!(html.contains("Completeness Yield"));
    assert!(html.contains("</main>"));
    assert!(html.ends_with("</html>\n") || html.ends_with("</html>"));

    assert_eq!(metrics.sections_exported, 2);
    assert_eq!(metrics.citations_exported, 1);
    assert_eq!(metrics.exclusions_exported, 1);
    assert!(!metrics.truncated);
    assert!(metrics.bytes_written > 0);
    assert!(metrics.lines_written > 0);
}

#[test]
fn test_html_export_fragment() {
    let doc = create_sample_document();
    let options = HtmlOptions::fragment();
    let (html, metrics) = export_html(&doc, &options).unwrap();

    assert!(!html.contains("<!DOCTYPE html>"));
    assert!(!html.contains("<html"));
    assert!(!html.contains("<head>"));
    assert!(!html.contains("<style>"));
    assert!(html.starts_with("<div class=\"cdm-container\">"));
    assert!(html.ends_with("</div>\n"));
    assert_eq!(metrics.sections_exported, 2);
}

#[test]
fn test_html_export_minimal() {
    let doc = create_sample_document();
    let options = HtmlOptions::minimal();
    let (html, metrics) = export_html(&doc, &options).unwrap();

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(!html.contains("Directory &amp; Structure Tree"));
    assert!(!html.contains("Table of Contents"));
    assert!(!html.contains("<details class=\"section-details\""));
    assert!(!html.contains("Sources &amp; Citations"));
    assert!(!html.contains("Excluded Content"));
    assert!(!html.contains("Completeness Yield"));
    assert!(html.contains("pub fn hello()"));
    assert_eq!(metrics.sections_exported, 2);
}

#[test]
fn test_html_export_line_numbers() {
    let doc = create_sample_document();
    let options = HtmlOptions::default().with_line_numbers(true);
    let (html, _) = export_html(&doc, &options).unwrap();

    assert!(html.contains("<span class=\"line-num\">   1</span>"));
    assert!(html.contains("<span class=\"line-num\">   2</span>"));
    assert!(html.contains("<span class=\"line-num\">   3</span>"));
}

#[test]
fn test_html_export_token_budget_truncation() {
    let doc = create_sample_document();
    let options = HtmlOptions::default().with_max_token_budget(Some(1));
    let (html, metrics) = export_html(&doc, &options).unwrap();

    assert!(metrics.truncated);
    assert!(metrics.sections_exported < doc.sections.len());
    assert!(html.contains("budget-alert"));
    assert!(html.contains("reached maximum token budget of 1 tokens"));
}

#[test]
fn test_html_streaming_writer_equivalence() {
    let doc = create_sample_document();
    let options = HtmlOptions::default();

    let (in_memory_str, in_memory_metrics) = export_html(&doc, &options).unwrap();

    let mut stream_buf = Vec::new();
    let stream_metrics = export_html_to_writer(&doc, &options, &mut stream_buf).unwrap();
    let stream_str = String::from_utf8(stream_buf).unwrap();

    assert_eq!(in_memory_str, stream_str);
    assert_eq!(in_memory_metrics, stream_metrics);
}

#[test]
fn test_html_empty_document_handling() {
    let identity = SourceIdentity::new("https://example.com/empty", "https://example.com/empty");
    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/empty",
    )
    .build();

    let options = HtmlOptions::default();
    let (html, metrics) = export_html(&doc, &options).unwrap();

    assert!(html.contains("<!DOCTYPE html>"));
    assert_eq!(metrics.sections_exported, 0);
    assert_eq!(metrics.estimated_tokens, 0);
}

#[test]
fn test_html_options_serde_roundtrip() {
    let options = HtmlOptions::default()
        .with_standalone(false)
        .with_line_numbers(true)
        .with_max_token_budget(Some(5000))
        .with_hierarchy_style(HierarchyStyle::AsciiTree);

    let json = serde_json::to_string(&options).unwrap();
    let deserialized: HtmlOptions = serde_json::from_str(&json).unwrap();

    assert_eq!(options, deserialized);
}

#[test]
fn test_html_escaping_and_xss_defense() {
    let identity = SourceIdentity::new("https://example.com/test", "https://example.com/test");
    let sec = CdmSectionBuilder::new(
        "xss-sec",
        "https://example.com/test#sec",
        SectionKind::CodeSnippet,
        "if (x < 10 && y > 20) { alert(\"hello\"); }",
    )
    .title("<img src=x onerror=alert(1)>")
    .build();

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/test",
    )
    .metadata(DocumentMetadata {
        title: Some("<script>alert('xss')</script>".to_string()),
        ..Default::default()
    })
    .add_section(sec)
    .build();
    let (html, _) = export_html(&doc, &HtmlOptions::default()).unwrap();

    // Verify raw unescaped script tag is not present
    assert!(!html.contains("<script>alert('xss')</script>"));
    assert!(html.contains("&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"));
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(html.contains("if (x &lt; 10 &amp;&amp; y &gt; 20) { alert(&quot;hello&quot;); }"));
}

#[test]
fn test_html_exporter_builder_api() {
    let doc = create_sample_document();
    let exporter = HtmlExporter::new();
    let (html1, m1) = exporter.export_to_string(&doc).unwrap();

    let mut buf = Vec::new();
    let exporter_writer = HtmlExporter::with_options(HtmlOptions::default());
    let m2 = exporter_writer.export_to_writer(&doc, &mut buf).unwrap();

    assert_eq!(html1, String::from_utf8(buf).unwrap());
    assert_eq!(m1, m2);
}
