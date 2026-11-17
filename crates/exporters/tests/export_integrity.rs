//! # Export Integrity & Security Integration Tests (`crates/exporters/tests/export_integrity.rs`)
//!
//! Exhaustive test suite verifying cryptographic checksums, manifest integrity, zip-slip
//! path traversal defenses, format fidelity, token budget truncation, and edge cases.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::completeness::CompletenessReport;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_exporters::{
    export_html, export_json, export_jsonl, export_markdown, export_plain_text, export_zip_package,
    verify_html_syntax, verify_json_syntax, verify_jsonl_syntax, verify_markdown_syntax,
    verify_zip_package_bytes, HtmlOptions, JsonExportOptions, JsonlFlavor, JsonlOptions,
    MarkdownOptions, PlainTextOptions, ZipPackageOptions,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Helper to create a realistic test CDM document.
fn create_test_cdm_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/org/integrity-test-repo".to_string(),
        canonical_url: "https://github.com/org/integrity-test-repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("org".to_string()),
        repo_name: Some("integrity-test-repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let s1 = CdmSectionBuilder::new(
        "sec-1",
        "https://github.com/org/integrity-test-repo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "/// Calculates the cryptographic checksum.\npub fn compute() -> &'static str {\n    \"ok\"\n}\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 4)
    .byte_range(0, 85)
    .build();

    let s2 = CdmSectionBuilder::new(
        "sec-2",
        "https://github.com/org/integrity-test-repo/blob/main/docs/guide.md",
        SectionKind::Prose,
        "# User Guide\n\nWelcome to the integrity test repository documentation.\n",
    )
    .title("docs/guide.md")
    .original_path("docs/guide.md")
    .language("markdown")
    .line_range(1, 3)
    .byte_range(0, 68)
    .build();

    let mut root = HierarchyNode::root("integrity-test-repo");
    let mut src_dir = HierarchyNode::directory("src-dir", "src", "src");
    src_dir.add_child(HierarchyNode::file(
        "lib-node",
        "lib.rs",
        "src/lib.rs",
        Some("sec-1".to_string()),
        Some(85),
        Some(4),
    ));
    root.add_child(src_dir);

    let citation = Citation::new(
        "cite-1",
        "https://github.com/org/integrity-test-repo",
        Some("Upstream GitHub Repository".to_string()),
        None,
    );

    let exclusion = ExclusionRecord::new(
        "target/debug/build.log",
        ExclusionReason::BinaryFile,
        Some("target/**".to_string()),
        Some("Build artifact".to_string()),
        Some(45_000),
    );

    let completeness = CompletenessReport::new(3, 2, 2, 1, 0, 0);

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/org/integrity-test-repo",
    )
    .document_id(DocumentId::new())
    .revision(RevisionInfo {
        git_commit: Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
        git_branch: Some("main".to_string()),
        ..Default::default()
    })
    .metadata(DocumentMetadata {
        title: Some("integrity-test-repo".to_string()),
        description: Some(
            "Integration test repository for export integrity verification".to_string(),
        ),
        primary_language: Some("Rust".to_string()),
        license: Some("MIT".to_string()),
        authors: vec!["Integrity Team".to_string()],
        language_breakdown: BTreeMap::from([
            ("Rust".to_string(), 85),
            ("Markdown".to_string(), 68),
        ]),
        ..Default::default()
    })
    .hierarchy(root)
    .completeness(completeness)
    .add_section(s1)
    .add_section(s2)
    .add_citation(citation)
    .add_exclusion(exclusion)
    .build()
}

/// Helper to read all files in a zip into an in-memory map.
fn unpack_zip(zip_bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).expect("open zip");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("entry by index");
        let name = file.name().expect("entry name").to_string();
        if file.is_dir() || name.ends_with('/') {
            continue;
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("read entry");
        map.insert(name, buf);
    }
    map
}

/// Helper to repack a map of files into a zip.
fn repack_zip(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for (name, content) in files {
        zip.start_file(name, opts).expect("start file");
        zip.write_all(content).expect("write file");
    }
    zip.finish().expect("finish zip");
    buffer.into_inner()
}

#[test]
fn test_integrity_valid_package_roundtrip() {
    let doc = create_test_cdm_document();
    let (zip_bytes, metrics) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    assert!(zip_bytes.len() > 1000);
    assert_eq!(metrics.files_archived, 12);

    let report = verify_zip_package_bytes(&zip_bytes).expect("verify zip");
    assert!(report.is_valid, "Expected package to be completely valid");
    assert!(!report.has_errors());
    assert!(report.mismatched_checksums.is_empty());
    assert!(report.missing_files.is_empty());
    assert!(report.unlisted_files.is_empty());
    assert!(report.path_traversal_violations.is_empty());
    assert!(report.total_verified() >= 8);

    // Verify metadata structures parsed properly
    let manifest = report.manifest.expect("manifest parsed");
    assert_eq!(
        manifest.source_url,
        "https://github.com/org/integrity-test-repo"
    );
    let prov = report.provenance.expect("provenance parsed");
    assert_eq!(prov.source_type, "github_public");
    let qr = report.quality_report.expect("quality report parsed");
    assert_eq!(qr.discovered_count, 3);
    assert_eq!(qr.extracted_count, 2);
}

#[test]
fn test_integrity_tampered_entry_detection() {
    let doc = create_test_cdm_document();
    let (zip_bytes, _) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    let mut files = unpack_zip(&zip_bytes);
    // Tamper with context.md by changing a single byte
    let md = files.get_mut("context.md").expect("context.md in zip");
    md[0] = if md[0] == b'#' { b'X' } else { b'#' };

    let tampered_zip = repack_zip(&files);
    let report = verify_zip_package_bytes(&tampered_zip).expect("verify tampered zip");

    assert!(!report.is_valid, "Tampered zip must not be valid");
    assert!(report.has_errors());
    assert!(
        report
            .mismatched_checksums
            .iter()
            .any(|m| m.path == "context.md"),
        "context.md mismatch must be captured in report"
    );
}

#[test]
fn test_integrity_truncated_entry_detection() {
    let doc = create_test_cdm_document();
    let (zip_bytes, _) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    let mut files = unpack_zip(&zip_bytes);
    // Truncate context.txt to 10 bytes
    let txt = files.get_mut("context.txt").expect("context.txt in zip");
    txt.truncate(10);

    let truncated_zip = repack_zip(&files);
    let report = verify_zip_package_bytes(&truncated_zip).expect("verify truncated zip");

    assert!(!report.is_valid);
    assert!(report.has_errors());
    assert!(
        report
            .mismatched_checksums
            .iter()
            .any(|m| m.path == "context.txt"),
        "Truncated file must be flagged for size and checksum mismatch"
    );
}

#[test]
fn test_integrity_missing_file_detection() {
    let doc = create_test_cdm_document();
    let (zip_bytes, _) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    let mut files = unpack_zip(&zip_bytes);
    // Delete context.html, leaving it declared in manifest.json and checksums.sha256
    files.remove("context.html").expect("context.html removed");

    let incomplete_zip = repack_zip(&files);
    let report = verify_zip_package_bytes(&incomplete_zip).expect("verify incomplete zip");

    assert!(!report.is_valid);
    assert!(report.has_errors());
    assert!(
        report.missing_files.contains(&"context.html".to_string()),
        "Missing context.html must be recorded in missing_files: {:?}",
        report.missing_files
    );
}

#[test]
fn test_integrity_unlisted_rogue_file_detection() {
    let doc = create_test_cdm_document();
    let (zip_bytes, _) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    let mut files = unpack_zip(&zip_bytes);
    // Inject rogue unlisted file
    files.insert(
        "rogue_backdoor.sh".to_string(),
        b"#!/bin/bash\necho compromised\n".to_vec(),
    );

    let rogue_zip = repack_zip(&files);
    let report = verify_zip_package_bytes(&rogue_zip).expect("verify rogue zip");

    assert!(!report.is_valid);
    assert!(report.has_errors());
    assert!(
        report
            .unlisted_files
            .contains(&"rogue_backdoor.sh".to_string()),
        "Rogue file must be detected in unlisted_files: {:?}",
        report.unlisted_files
    );
}

#[test]
fn test_integrity_zip_slip_and_path_traversal_detection() {
    let doc = create_test_cdm_document();
    let (zip_bytes, _) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");

    let mut files = unpack_zip(&zip_bytes);
    // Inject adversarial path traversal entries
    files.insert("../../etc/shadow".to_string(), b"root:*:0:0:::".to_vec());
    files.insert(
        "C:\\Windows\\System32\\calc.exe".to_string(),
        b"MZ".to_vec(),
    );
    files.insert("/var/run/secret".to_string(), b"token".to_vec());

    let traversal_zip = repack_zip(&files);
    let report = verify_zip_package_bytes(&traversal_zip).expect("verify traversal zip");

    assert!(!report.is_valid);
    assert!(report.has_errors());
    assert!(report.path_traversal_violations.len() >= 3);
    assert!(report
        .path_traversal_violations
        .iter()
        .any(|p| p.contains("..")));
    assert!(report
        .path_traversal_violations
        .iter()
        .any(|p| p.contains('\\')));
    assert!(report
        .path_traversal_violations
        .iter()
        .any(|p| p.starts_with('/')));
}

#[test]
fn test_integrity_format_syntax_verifiers() {
    // 1. Markdown syntax
    assert!(verify_markdown_syntax("# Heading\n\n```rust\nlet x = 1;\n```\n").is_ok());
    assert!(verify_markdown_syntax("   ").is_err()); // empty
    assert!(verify_markdown_syntax("```rust\nlet x = 1;\n").is_err()); // unclosed fence

    // 2. JSON syntax
    let valid_json = r#"{"name": "test", "active": true, "count": 42}"#;
    assert!(verify_json_syntax(valid_json).is_ok());
    assert!(verify_json_syntax(r#"{"name": "incomplete""#).is_err());

    // 3. HTML syntax
    let valid_html =
        "<!DOCTYPE html>\n<html><head><title>Test</title></head><body><h1>Hi</h1></body></html>";
    assert!(verify_html_syntax(valid_html, true).is_ok());
    assert!(verify_html_syntax("<h1>Hi</h1>", true).is_err()); // missing doctype and closing html
    assert!(verify_html_syntax("<h1>Hi</h1>", false).is_ok()); // fragment allowed when standalone=false
    assert!(verify_html_syntax("   ", false).is_err()); // empty

    // 4. JSONL syntax
    let valid_jsonl = "{\"chunk\": 1}\n{\"chunk\": 2}\n{\"chunk\": 3}\n";
    assert_eq!(verify_jsonl_syntax(valid_jsonl).unwrap(), 3);
    let invalid_jsonl = "{\"chunk\": 1}\nBROKEN_JSON\n{\"chunk\": 3}\n";
    assert!(verify_jsonl_syntax(invalid_jsonl).is_err());
}

#[test]
fn test_integrity_token_budget_truncation_fidelity() {
    let doc = create_test_cdm_document();

    // 1. Markdown truncation with tiny budget
    let md_opts = MarkdownOptions::default().with_max_token_budget(Some(20));
    let md = export_markdown(&doc, &md_opts).expect("export markdown with budget");
    assert!(
        verify_markdown_syntax(&md).is_ok(),
        "Truncated markdown must have valid syntax"
    );

    // 2. HTML truncation with tiny budget
    let html_opts = HtmlOptions::default().with_max_token_budget(Some(20));
    let (html, _) = export_html(&doc, &html_opts).expect("export html with budget");
    assert!(
        verify_html_syntax(&html, true).is_ok(),
        "Truncated HTML must have valid DOCTYPE and closing tags"
    );

    // 3. Structured JSON truncation with tiny budget
    let json_opts = JsonExportOptions::structured().with_max_token_budget(Some(20));
    let json = export_json(&doc, &json_opts).expect("export json with budget");
    let parsed_val = verify_json_syntax(&json).expect("Truncated JSON must parse cleanly");
    assert_eq!(
        parsed_val.get("truncated"),
        Some(&serde_json::Value::Bool(true))
    );
    assert!(parsed_val.get("canonical_url").is_some());

    // 4. JSONL truncation with tiny budget
    let jsonl_opts = JsonlOptions::default()
        .with_flavor(JsonlFlavor::RagChunks)
        .with_max_token_budget(Some(20));
    let (jsonl, _) = export_jsonl(&doc, &jsonl_opts).expect("export jsonl with budget");
    let count = verify_jsonl_syntax(&jsonl).expect("Truncated JSONL must have valid JSON per line");
    assert!(count <= 2);
}

#[test]
fn test_integrity_edge_cases_and_multi_byte_unicode() {
    let identity = SourceIdentity {
        raw_input: "https://github.com/org/unicode-test".to_string(),
        canonical_url: "https://github.com/org/unicode-test".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("org".to_string()),
        repo_name: Some("unicode-test".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    // Multi-byte Unicode: Arabic RTL, CJK Kanji/Kana, mathematical symbols, Emojis
    let unicode_content = "/* \u{1F980} Rust crab */\n\
        // \u{4E2D}\u{6587}\u{6D4B}\u{8BD5} (Chinese test)\n\
        // \u{0645}\u{0631}\u{062D}\u{0628}\u{0627} (Arabic test)\n\
        // \u{1F680}\u{2728}\u{1F525} (Emojis)\n\
        pub const FORMULA: &str = \"\u{2200}x \u{2208} X, \u{2203}y \u{2208} Y\";\n";

    let sec = CdmSectionBuilder::new(
        "sec-unicode",
        "https://github.com/org/unicode-test/blob/main/src/nested/deeply/unicode.rs",
        SectionKind::CodeFile,
        unicode_content,
    )
    .title("src/nested/deeply/unicode.rs")
    .original_path("src/nested/deeply/unicode.rs")
    .language("rust")
    .build();

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/org/unicode-test",
    )
    .document_id(DocumentId::new())
    .add_section(sec)
    .build();

    // Verify all single formats preserve unicode lossless
    let md = export_markdown(&doc, &MarkdownOptions::default()).expect("markdown");
    assert!(md.contains('\u{1F980}'));
    assert!(md.contains("\u{4E2D}\u{6587}"));
    assert!(verify_markdown_syntax(&md).is_ok());

    let txt = export_plain_text(&doc, &PlainTextOptions::default()).expect("txt");
    assert!(txt.contains('\u{1F980}'));

    let json = export_json(&doc, &JsonExportOptions::canonical()).expect("json");
    assert!(json.contains('\u{1F980}'));
    assert!(verify_json_syntax(&json).is_ok());

    let (html, _) = export_html(&doc, &HtmlOptions::default()).expect("html");
    assert!(verify_html_syntax(&html, true).is_ok());

    // Verify ZIP package with deep nesting and unicode
    let (zip_bytes, metrics) =
        export_zip_package(&doc, &ZipPackageOptions::default()).expect("export zip package");
    assert!(metrics.files_archived >= 8);

    let report = verify_zip_package_bytes(&zip_bytes).expect("verify zip");
    assert!(
        report.is_valid,
        "Unicode package must verify with 100% cryptographic integrity"
    );
    assert!(!report.has_errors());
    assert!(report
        .verified_files
        .contains(&"sources/src/nested/deeply/unicode.rs".to_string()));
}
