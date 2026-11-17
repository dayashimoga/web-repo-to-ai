//! # CDM JSON Serialization and Deserialization Integration Tests
//!
//! Validates lossless roundtrips, deterministic encoding, streaming adapters,
//! memory boundary defenses (`PayloadTooLarge`), and version validation.

use chrono::Utc;
use std::collections::BTreeMap;
use std::io::Cursor;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmError, CdmSectionBuilder, Citation, CompletenessReport,
    DocumentMetadata, ExclusionReason, ExclusionRecord, ExtractionStatus, ExtractionWarning,
    HierarchyNode, JsonOptions, Relationship, RelationshipKind, RevisionInfo, SectionKind,
    SourceIdentity, SourceType, WarningSeverity,
};
use web_repo_to_ai_core::DocumentId;

fn create_rich_test_document() -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/org/repo".to_string(),
        canonical_url: "https://github.com/org/repo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("org".to_string()),
        repo_name: Some("repo".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let sec1 = CdmSectionBuilder::new(
        "sec-001",
        "https://github.com/org/repo/blob/main/README.md",
        SectionKind::Prose,
        "# Project Title\n\nComprehensive repository documentation.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 3)
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-002",
        "https://github.com/org/repo/blob/main/src/lib.rs",
        SectionKind::CodeFile,
        "pub fn core_engine() -> bool {\n    true\n}\n",
    )
    .title("src/lib.rs")
    .original_path("src/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .attribute("git_blob_sha", "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391")
    .build();

    let sec3 = CdmSectionBuilder::new(
        "sec-003",
        "https://github.com/org/repo/blob/main/spec.md#table",
        SectionKind::Table,
        "| ID | Status |\n|---|---|\n| 1 | Active |\n",
    )
    .title("Status Specification")
    .original_path("spec.md")
    .build();

    let mut root_node = HierarchyNode::root("org/repo");
    let mut src_dir = HierarchyNode::directory("dir-src", "src", "src");
    src_dir.add_child(HierarchyNode::file(
        "node-lib",
        "lib.rs",
        "src/lib.rs",
        Some("sec-002".to_string()),
        Some(42),
        Some(3),
    ));
    root_node.add_child(src_dir);
    root_node.add_child(HierarchyNode::file(
        "node-readme",
        "README.md",
        "README.md",
        Some("sec-001".to_string()),
        Some(55),
        Some(3),
    ));

    let rel = Relationship::new("sec-001", "sec-002", RelationshipKind::References)
        .with_description("README references core library module");

    let citation = Citation::new(
        "cit-001",
        "https://github.com/org/repo",
        Some("org/repo on GitHub".to_string()),
        None,
    );

    let exclusion = ExclusionRecord::new(
        "Cargo.lock",
        ExclusionReason::RulePattern,
        Some("*.lock".to_string()),
        Some("Lockfile excluded by policy".to_string()),
        Some(2048),
    );

    let warning = ExtractionWarning::new(
        "TEST_WARNING",
        "diagnostics",
        "Sample non-fatal warning",
        WarningSeverity::Info,
    );

    let completeness = CompletenessReport::new(4, 3, 3, 1, 0, 0);

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/org/repo",
    )
    .document_id(DocumentId::new())
    .retrieved_at(Utc::now())
    .revision(RevisionInfo {
        git_commit: Some("4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string()),
        git_branch: Some("main".to_string()),
        content_hash_blake3: Some(
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f".to_string(),
        ),
        ..Default::default()
    })
    .content_type("application/vnd.github.v3+json")
    .metadata(DocumentMetadata {
        title: Some("org/repo".to_string()),
        description: Some("Comprehensive test repository".to_string()),
        license: Some("MIT".to_string()),
        authors: vec!["Engineer A".to_string(), "Engineer B".to_string()],
        primary_language: Some("Rust".to_string()),
        language_breakdown: BTreeMap::from([
            ("Rust".to_string(), 42),
            ("Markdown".to_string(), 55),
        ]),
        tags: vec!["ai".to_string(), "converter".to_string()],
        ..Default::default()
    })
    .hierarchy(root_node)
    .add_section(sec1)
    .add_section(sec2)
    .add_section(sec3)
    .add_relationship(rel)
    .add_citation(citation)
    .add_exclusion(exclusion)
    .add_warning(warning)
    .extraction_status(ExtractionStatus::Completed)
    .completeness(completeness)
    .build()
}

#[test]
fn test_json_lossless_roundtrip_complex_document() {
    let doc = create_rich_test_document();

    // 1. Pretty roundtrip
    let pretty_json = doc.to_json_pretty().expect("pretty serialization succeeds");
    assert!(pretty_json.contains("\"schema_version\": \"1.0.0\""));
    assert!(pretty_json.contains("\"title\": \"org/repo\""));

    let from_pretty =
        CdmDocument::from_json(&pretty_json).expect("deserialization from pretty succeeds");
    assert_eq!(doc, from_pretty);

    // 2. Compact roundtrip
    let compact_json = doc
        .to_json_compact()
        .expect("compact serialization succeeds");
    assert!(!compact_json.contains('\n'));

    let from_compact =
        CdmDocument::from_json(&compact_json).expect("deserialization from compact succeeds");
    assert_eq!(doc, from_compact);

    // 3. From slice roundtrip
    let from_slice =
        CdmDocument::from_slice(compact_json.as_bytes()).expect("slice deserialization succeeds");
    assert_eq!(doc, from_slice);
}

#[test]
fn test_json_deterministic_serialization() {
    let doc = create_rich_test_document();

    let json1 = doc.to_json_compact().unwrap();
    let json2 = doc.to_json_compact().unwrap();

    assert_eq!(
        json1, json2,
        "Repeated JSON serializations must produce identical byte sequences"
    );
}

#[test]
fn test_json_streaming_writer_and_reader() {
    let doc = create_rich_test_document();
    let mut buffer = Vec::new();

    web_repo_to_ai_cdm::to_writer_pretty(&doc, &mut buffer).expect("streaming write succeeds");
    assert!(!buffer.is_empty());

    let cursor = Cursor::new(buffer);
    let recovered = CdmDocument::from_reader(cursor).expect("streaming read succeeds");

    assert_eq!(doc, recovered);
}

#[test]
fn test_payload_size_ceiling_defense() {
    let doc = create_rich_test_document();
    let json_bytes = doc.to_json_compact().unwrap();

    // Set limit far smaller than actual JSON payload
    let strict_options = JsonOptions::default().with_max_bytes(100);

    let err_str = web_repo_to_ai_cdm::from_json_with_options(&json_bytes, &strict_options);
    assert!(matches!(
        err_str.unwrap_err(),
        CdmError::PayloadTooLarge { .. }
    ));

    let err_slice =
        web_repo_to_ai_cdm::from_slice_with_options(json_bytes.as_bytes(), &strict_options);
    assert!(matches!(
        err_slice.unwrap_err(),
        CdmError::PayloadTooLarge { .. }
    ));

    let cursor = Cursor::new(json_bytes.as_bytes());
    let err_reader = web_repo_to_ai_cdm::from_reader_with_options(cursor, &strict_options);
    assert!(matches!(
        err_reader.unwrap_err(),
        CdmError::PayloadTooLarge { .. }
    ));
}

#[test]
fn test_corrupted_json_syntax_rejected() {
    let invalid_json = "{ \"schema_version\": \"1.0.0\", \"document_id\": ";
    let err = CdmDocument::from_json(invalid_json);
    assert!(matches!(err.unwrap_err(), CdmError::Deserialization(_)));
}

#[test]
fn test_schema_version_validation_logic() {
    let doc = create_rich_test_document();
    let mut val: serde_json::Value = serde_json::to_value(&doc).unwrap();

    // 1. Incompatible major version
    val["schema_version"] = serde_json::json!("2.0.0");
    let err = CdmDocument::from_json(&serde_json::to_string(&val).unwrap());
    assert!(matches!(
        err.unwrap_err(),
        CdmError::UnsupportedSchemaVersion { .. }
    ));

    // 2. Compatible minor version (same major '1')
    val["schema_version"] = serde_json::json!("1.1.0");
    let ok = CdmDocument::from_json(&serde_json::to_string(&val).unwrap());
    assert!(ok.is_ok(), "Same major version should pass validation");

    // 3. Disabled version validation passes even major changes
    val["schema_version"] = serde_json::json!("99.0.0");
    let relaxed = JsonOptions::default().with_validate_version(false);
    let ok_relaxed =
        web_repo_to_ai_cdm::from_json_with_options(&serde_json::to_string(&val).unwrap(), &relaxed);
    assert!(ok_relaxed.is_ok());
}

#[test]
fn test_hash_integrity_verification_on_deserialization() {
    let doc = create_rich_test_document();
    let mut val: serde_json::Value = serde_json::to_value(&doc).unwrap();

    // Tamper with section body
    val["sections"][0]["body"] =
        serde_json::json!("# Tampered Header\n\nAttacker modified content.\n");
    let tampered_json = serde_json::to_string(&val).unwrap();

    // Default options do not check hash
    let unverified = CdmDocument::from_json(&tampered_json);
    assert!(unverified.is_ok());

    // Strict hash checking catches the tamper
    let strict = JsonOptions::default().with_verify_hashes(true);
    let err = web_repo_to_ai_cdm::from_json_with_options(&tampered_json, &strict);
    assert!(matches!(err.unwrap_err(), CdmError::HashMismatch { .. }));
}
