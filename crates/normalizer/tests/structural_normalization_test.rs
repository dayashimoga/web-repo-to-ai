//! # Structural Normalization Integration Tests (`crates/normalizer`)
//!
//! Validates end-to-end normalization correctness, encoding repair, whitespace standardizing,
//! Markdown canonicalization, semantic code fidelity preservation, idempotency, and reporting.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    LineRange, RevisionInfo, SectionKind, SectionTokenMetrics, SourceIdentity, SourceType,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_normalizer::{normalize_document, normalize_text, NormalizationConfig};

fn sample_multi_section_document() -> CdmDocument {
    let doc_id = DocumentId::new();

    let sec1_body = "\u{FEFF}#  Document Overview  \r\n\r\nHere is an overview with “smart quotes” and trailing spaces.   \r\n\r\n\r\n\r\nNext paragraph.\r\n";
    let sec2_body = "    pub fn calculate(val: u64) -> u64 {\r\n        // Indentation must be preserved!   \r\n        let result = val * 2;\r\n\r\n\r\n\r\n\r\n        result\r\n    }\r\n";
    let sec3_body =
        "| Header A | Header B |\r\n|---|---|\r\n| Cell 1 | Cell 2 | Cell 3 |\r\n| Cell 4 |\r\n";
    let sec4_body = "* First item\r\n  * Nested bullet\r\n* Third item\r\n";

    let sec1 = CdmSection {
        section_id: "sec-prose-1".to_string(),
        parent_id: None,
        title: Some("Overview".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("README.md".to_string()),
        source_url: "https://example.com/README.md".to_string(),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(sec1_body),
        body: sec1_body.to_string(),
        token_metrics: Some(SectionTokenMetrics::compute_baseline(sec1_body)),
        attributes: BTreeMap::new(),
    };

    let sec2 = CdmSection {
        section_id: "sec-code-2".to_string(),
        parent_id: None,
        title: Some("lib.rs".to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some("src/lib.rs".to_string()),
        source_url: "https://example.com/src/lib.rs".to_string(),
        line_range: Some(LineRange::new(1, 15)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(sec2_body),
        body: sec2_body.to_string(),
        token_metrics: Some(SectionTokenMetrics::compute_baseline(sec2_body)),
        attributes: BTreeMap::new(),
    };

    let sec3 = CdmSection {
        section_id: "sec-table-3".to_string(),
        parent_id: None,
        title: Some("Metrics".to_string()),
        kind: SectionKind::Table,
        original_path: Some("docs/metrics.md".to_string()),
        source_url: "https://example.com/docs/metrics.md".to_string(),
        line_range: None,
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(sec3_body),
        body: sec3_body.to_string(),
        token_metrics: Some(SectionTokenMetrics::compute_baseline(sec3_body)),
        attributes: BTreeMap::new(),
    };

    let sec4 = CdmSection {
        section_id: "sec-prose-4".to_string(),
        parent_id: None,
        title: Some("List".to_string()),
        kind: SectionKind::Prose,
        original_path: Some("docs/list.md".to_string()),
        source_url: "https://example.com/docs/list.md".to_string(),
        line_range: None,
        byte_range: None,
        language: Some("markdown".to_string()),
        content_hash: CdmSection::compute_hash(sec4_body),
        body: sec4_body.to_string(),
        token_metrics: Some(SectionTokenMetrics::compute_baseline(sec4_body)),
        attributes: BTreeMap::new(),
    };

    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: doc_id,
        source_identity: SourceIdentity::new(
            "https://example.com/repo",
            "https://example.com/repo",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://example.com/repo".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![sec1, sec2, sec3, sec4],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    }
}

#[test]
fn test_line_ending_and_whitespace_canonicalization() {
    let raw = "line 1\r\nline 2\rline 3\nline 4   \r\n\r\n\r\n\r\nline 5\r\n";
    let config = NormalizationConfig::lossless();
    let normalized = normalize_text(raw, &config, false);

    assert_eq!(normalized, "line 1\nline 2\nline 3\nline 4\n\nline 5");
    assert!(!normalized.contains('\r'));
}

#[test]
fn test_encoding_sanitization_bom_null_control_zero_width() {
    let raw = "\u{FEFF}Clean\0Text\u{200B}With\x07Control\u{200D}Chars\tAnd\nLines";
    let config = NormalizationConfig::lossless();
    let normalized = normalize_text(raw, &config, false);

    assert_eq!(normalized, "CleanTextWithControlChars\tAnd\nLines");
    assert!(!normalized.starts_with('\u{FEFF}'));
    assert!(!normalized.contains('\0'));
    assert!(!normalized.contains('\u{200B}'));
    assert!(!normalized.contains('\u{200D}'));
    assert!(!normalized.contains('\x07'));
    assert!(normalized.contains('\t'));
}

#[test]
fn test_typographic_punctuation_compact_vs_lossless() {
    let prose = "“Hello world” — said ‘Alice’ …";

    let lossless_cfg = NormalizationConfig::lossless();
    let lossless_out = normalize_text(prose, &lossless_cfg, false);
    assert_eq!(lossless_out, prose);

    let compact_cfg = NormalizationConfig::compact();
    let compact_out = normalize_text(prose, &compact_cfg, false);
    assert_eq!(compact_out, "\"Hello world\" -- said 'Alice' ...");
}

#[test]
fn test_code_section_strictly_preserves_quotes_and_symbols() {
    let code = "    let s = \"“hello”\";\r\n    let val = 42;   \r\n\r\n\r\n\r\n    val\r\n";
    let code_cfg = NormalizationConfig::code_focused();
    let out = normalize_text(code, &code_cfg, true);

    // Quotes inside code must NEVER be altered even in compact/code mode
    assert!(out.contains("let s = \"“hello”\";"));
    // 4-space indentation must be preserved
    assert!(out.starts_with("    let s"));
    // Blank lines in code capped at max_consecutive_blank_lines_code (2 in code_focused)
    assert_eq!(
        out,
        "    let s = \"“hello”\";\n    let val = 42;\n\n\n    val"
    );

    // In compact mode, blank lines are collapsed to 1
    let compact_cfg = NormalizationConfig::compact();
    let compact_out = normalize_text(code, &compact_cfg, true);
    assert_eq!(
        compact_out,
        "    let s = \"“hello”\";\n    let val = 42;\n\n    val"
    );
}

#[test]
fn test_markdown_code_fence_normalization() {
    let md = "```PYTHON\ndef foo():\n    pass\n````\n\n~~~ Rust\nfn bar() {}\n~~~";
    let config = NormalizationConfig::default();
    let out = normalize_text(md, &config, false);

    assert_eq!(
        out,
        "```python\ndef foo():\n    pass\n```\n\n```rust\nfn bar() {}\n```"
    );
}

#[test]
fn test_markdown_heading_spacing_normalization() {
    let md = "#NoSpace\n##   TooManySpaces   \n### Exact Space\n######LastLevel";
    let config = NormalizationConfig::default();
    let out = normalize_text(md, &config, false);

    assert_eq!(
        out,
        "# NoSpace\n## TooManySpaces\n### Exact Space\n###### LastLevel"
    );
}

#[test]
fn test_markdown_list_marker_standardization() {
    let md = "* Item A\n  * Subitem B\n    * Subitem C\n- Already Dash\n* Item D";
    let config = NormalizationConfig::default();
    let out = normalize_text(md, &config, false);

    assert_eq!(
        out,
        "- Item A\n  - Subitem B\n    - Subitem C\n- Already Dash\n- Item D"
    );
}

#[test]
fn test_markdown_table_rectangularization() {
    let table =
        "| Col 1 | Col 2 | Col 3 |\n|---|---|---|\n| Val 1 |\n| Val 2 | Val 3 | Val 4 | Extra |\n";
    let config = NormalizationConfig::default();
    let out = normalize_text(table, &config, false);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "| Col 1 | Col 2 | Col 3 |");
    assert_eq!(lines[1], "| :--- | :--- | :--- |");
    assert_eq!(lines[2], "| Val 1 |  |  |");
    assert_eq!(lines[3], "| Val 2 | Val 3 | Val 4 |");
}

#[test]
fn test_full_document_normalization_end_to_end() {
    let doc = sample_multi_section_document();
    let config = NormalizationConfig::compact();
    let (norm_doc, report) = normalize_document(&doc, &config);

    assert_eq!(norm_doc.sections.len(), 4);
    assert_eq!(report.sections_total, 4);
    assert!(report.sections_modified > 0);
    assert!(report.byte_reduction > 0);
    assert!(norm_doc.verify_section_hashes());

    // Prose 1: BOM stripped, smart quotes normalized to ASCII in compact mode, heading cleaned
    let sec1 = &norm_doc.sections[0];
    assert!(sec1.body.starts_with("# Document Overview"));
    assert!(sec1.body.contains("\"smart quotes\""));
    assert!(!sec1.body.contains('\u{FEFF}'));

    // Code 2: indentation preserved, trailing spaces trimmed, blank lines capped at 2
    let sec2 = &norm_doc.sections[1];
    assert!(sec2.body.starts_with("    pub fn calculate"));
    assert!(sec2.body.contains("// Indentation must be preserved!"));

    // Table 3: rectangularized
    let sec3 = &norm_doc.sections[2];
    assert!(sec3.body.contains("| :--- | :--- |"));

    // List 4: standardized list markers
    let sec4 = &norm_doc.sections[3];
    assert!(sec4.body.starts_with("- First item"));
    assert!(sec4.body.contains("  - Nested bullet"));
}

#[test]
fn test_document_normalization_idempotency() {
    let doc = sample_multi_section_document();
    let config = NormalizationConfig::compact();

    let (pass1_doc, pass1_report) = normalize_document(&doc, &config);
    assert!(pass1_report.sections_modified > 0);

    let (pass2_doc, pass2_report) = normalize_document(&pass1_doc, &config);
    assert_eq!(pass2_report.sections_modified, 0);
    assert_eq!(pass2_report.byte_reduction, 0);
    assert_eq!(pass1_doc, pass2_doc);
}

#[test]
fn test_raw_text_normalization_idempotency() {
    let raw =
        "\u{FEFF}#  Heading  \r\n\r\n*  List item  \r\n\r\n\r\nParagraph with “quotes”.\t  \r\n";
    let config = NormalizationConfig::compact();

    let pass1 = normalize_text(raw, &config, false);
    let pass2 = normalize_text(&pass1, &config, false);

    assert_eq!(pass1, pass2);
}

#[test]
fn test_empty_and_whitespace_only_sections() {
    let empty_out = normalize_text("", &NormalizationConfig::default(), false);
    assert_eq!(empty_out, "");

    let ws_only = "   \t   \r\n\r\n   ";
    let ws_out = normalize_text(ws_only, &NormalizationConfig::default(), false);
    assert_eq!(ws_out, "");
}

#[test]
fn test_normalization_report_markdown_and_json_serialization() {
    let doc = sample_multi_section_document();
    let config = NormalizationConfig::compact();
    let (_, report) = normalize_document(&doc, &config);

    let md_summary = report.to_markdown_summary();
    assert!(md_summary.contains("### Structural Normalization Summary"));
    assert!(md_summary.contains(&doc.document_id.to_string()));
    assert!(md_summary.contains("| Section ID | Kind |"));

    let json_str = serde_json::to_string_pretty(&report).expect("valid json serialization");
    let deserialized: web_repo_to_ai_normalizer::NormalizationReport =
        serde_json::from_str(&json_str).expect("valid json deserialization");
    assert_eq!(report, deserialized);
}
