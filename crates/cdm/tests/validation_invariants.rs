//! Integration tests for CDM schema validation and invariant checks.
//!
//! Validates:
//! 1. Full valid document passes all invariant checks.
//! 2. Incompatible major schema version is rejected.
//! 3. Nil document ID is rejected.
//! 4. Malformed canonical URL and invalid schemes are rejected.
//! 5. Empty section IDs and empty source URLs are rejected.
//! 6. Line range zero and inverted ranges are rejected.
//! 7. Byte range inverted is rejected.
//! 8. Tampered BLAKE3 content hashes are rejected.
//! 9. Duplicate hierarchy node IDs and tree depth bounds are enforced.
//! 10. Hierarchy tree referential integrity (node section_id must exist in sections).
//! 11. Relationship referential integrity (source_id and internal target_id must exist).
//! 12. Completeness arithmetic invariant formula is strictly enforced.
//! 13. Completed document status with failed items is rejected.
//! 14. Empty exclusion paths and empty extraction warning fields are rejected.
//! 15. Validation report filtering, diagnostics formatting, and strict vs permissive modes.

use web_repo_to_ai_cdm::{
    ByteRange, CdmDocumentBuilder, CdmSectionBuilder, CompletenessReport, ExtractionStatus,
    ExtractionWarning, HierarchyNode, LineRange, Relationship, RelationshipKind, SectionKind,
    SourceIdentity, SourceType, ValidationConfig, ValidationSeverity, WarningSeverity,
};
use web_repo_to_ai_core::DocumentId;

fn create_base_document() -> web_repo_to_ai_cdm::CdmDocument {
    let identity = SourceIdentity {
        raw_input: "https://github.com/rust-lang/cargo".to_string(),
        canonical_url: "https://github.com/rust-lang/cargo".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("rust-lang".to_string()),
        repo_name: Some("cargo".to_string()),
        git_ref: Some("master".to_string()),
        subpath: None,
    };

    let sec1 = CdmSectionBuilder::new(
        "sec-lib",
        "https://github.com/rust-lang/cargo/blob/master/src/cargo/lib.rs",
        SectionKind::CodeFile,
        "//! Cargo crate root\npub mod core;\npub mod util;\n",
    )
    .title("src/cargo/lib.rs")
    .original_path("src/cargo/lib.rs")
    .language("rust")
    .line_range(1, 3)
    .byte_range(0, 48)
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-readme",
        "https://github.com/rust-lang/cargo/blob/master/README.md",
        SectionKind::Prose,
        "# Cargo\nCargo is the Rust package manager.\n",
    )
    .title("README.md")
    .original_path("README.md")
    .language("markdown")
    .line_range(1, 2)
    .byte_range(0, 42)
    .build();

    let mut root = HierarchyNode::root("cargo");
    let mut src_dir = HierarchyNode::directory("dir-src", "src", "src");
    let mut cargo_dir = HierarchyNode::directory("dir-cargo", "cargo", "src/cargo");
    cargo_dir.add_child(HierarchyNode::file(
        "file-lib",
        "lib.rs",
        "src/cargo/lib.rs",
        Some("sec-lib".to_string()),
        Some(48),
        Some(3),
    ));
    src_dir.add_child(cargo_dir);
    root.add_child(src_dir);
    root.add_child(HierarchyNode::file(
        "file-readme",
        "README.md",
        "README.md",
        Some("sec-readme".to_string()),
        Some(42),
        Some(2),
    ));

    CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/rust-lang/cargo",
    )
    .title("rust-lang/cargo")
    .hierarchy(root)
    .add_section(sec1)
    .add_section(sec2)
    .add_relationship(Relationship {
        source_id: "sec-readme".to_string(),
        target_id: "sec-lib".to_string(),
        kind: RelationshipKind::References,
        description: Some("README references crate root".to_string()),
    })
    .completeness(CompletenessReport::new(2, 2, 2, 0, 0, 0))
    .build()
}

#[test]
fn test_full_valid_cdm_document_passes_all_checks() {
    let doc = create_base_document();
    let report = doc.validation_report();

    assert!(report.is_valid(), "Validation failed: {}", report.summary());
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.checked_sections_count, 2);
    assert_eq!(report.checked_nodes_count, 5); // root, src, cargo, lib, readme
    assert_eq!(report.checked_relationships_count, 1);
    assert!(doc.validate().is_ok());
    assert!(doc.validate_strict().is_ok());
}

#[test]
fn test_incompatible_major_schema_version_rejected() {
    let mut doc = create_base_document();
    doc.schema_version = "2.0.0".to_string();

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("SCHEMA_VERSION_INCOMPATIBLE");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, ValidationSeverity::Error);
    assert_eq!(issues[0].path, "schema_version");

    let err = doc.validate().unwrap_err();
    assert!(err.to_string().contains("SCHEMA_VERSION_INCOMPATIBLE"));
}

#[test]
fn test_nil_document_id_rejected() {
    let mut doc = create_base_document();
    doc.document_id = DocumentId::from_uuid(uuid::Uuid::nil());

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("DOCUMENT_ID_NIL");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].path, "document_id");
}

#[test]
fn test_malformed_canonical_url_and_schemes_rejected() {
    let mut doc = create_base_document();
    doc.canonical_url = "not a valid url".to_string();

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("CANONICAL_URL_INVALID").len(), 1);

    // Invalid scheme (e.g. ftp or file for web source)
    doc.canonical_url = "ftp://ftp.example.com/repo".to_string();
    let report2 = doc.validation_report();
    assert!(!report2.is_valid());
    assert_eq!(
        report2.filter_by_code("CANONICAL_URL_SCHEME_INVALID").len(),
        1
    );
}

#[test]
fn test_empty_section_id_and_empty_source_url_rejected() {
    let mut doc = create_base_document();
    doc.sections[0].section_id = "   ".to_string();
    doc.sections[1].source_url = String::new();

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("SECTION_ID_EMPTY").len(), 1);
    assert_eq!(report.filter_by_code("SECTION_URL_EMPTY").len(), 1);
}

#[test]
fn test_line_range_zero_and_inverted_rejected() {
    let mut doc = create_base_document();
    // Line number 0
    doc.sections[0].line_range = Some(LineRange::new(0, 10));
    // Inverted range
    doc.sections[1].line_range = Some(LineRange::new(50, 20));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("LINE_RANGE_ZERO").len(), 1);
    assert_eq!(report.filter_by_code("LINE_RANGE_INVERTED").len(), 1);
}

#[test]
fn test_byte_range_inverted_rejected() {
    let mut doc = create_base_document();
    doc.sections[0].byte_range = Some(ByteRange::new(500, 100));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("BYTE_RANGE_INVERTED").len(), 1);
}

#[test]
fn test_tampered_blake3_content_hash_rejected() {
    let mut doc = create_base_document();
    // Tamper with body without updating hash
    doc.sections[0].body.push_str("// Injected backdoor\n");

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("SECTION_HASH_MISMATCH");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].path, "sections[0].content_hash");

    // When verify_hashes is disabled, hash mismatch is ignored
    let config = ValidationConfig::default().with_verify_hashes(false);
    let report_no_hash = doc.validate_with_config(&config);
    assert!(report_no_hash.is_valid());
}

#[test]
fn test_acyclicity_and_duplicate_nodes_rejected() {
    let mut doc = create_base_document();
    // Insert child node with duplicate ID "file-lib"
    doc.hierarchy.add_child(HierarchyNode::file(
        "file-lib",
        "duplicate_lib.rs",
        "src/duplicate_lib.rs",
        Some("sec-lib".to_string()),
        None,
        None,
    ));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("TREE_NODE_ID_DUPLICATE").len(), 1);
}

#[test]
fn test_tree_referential_integrity_section_existence() {
    let mut doc = create_base_document();
    doc.hierarchy.add_child(HierarchyNode::file(
        "file-missing",
        "missing.rs",
        "src/missing.rs",
        Some("non-existent-sec-id".to_string()),
        None,
        None,
    ));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("REFERENTIAL_INTEGRITY_MISSING_SECTION");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("non-existent-sec-id"));
}

#[test]
fn test_relationships_referential_integrity_source_and_target() {
    let mut doc = create_base_document();
    doc.relationships.push(Relationship {
        source_id: "sec-missing-source".to_string(),
        target_id: "sec-lib".to_string(),
        kind: RelationshipKind::Calls,
        description: None,
    });
    doc.relationships.push(Relationship {
        source_id: "sec-lib".to_string(),
        target_id: "sec-missing-target".to_string(),
        kind: RelationshipKind::Imports,
        description: None,
    });

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(
        report.filter_by_code("RELATIONSHIP_SOURCE_NOT_FOUND").len(),
        1
    );
    assert_eq!(
        report.filter_by_code("RELATIONSHIP_TARGET_NOT_FOUND").len(),
        1
    );
}

#[test]
fn test_completeness_invariant_broken_formula_rejected() {
    let mut doc = create_base_document();
    // Formula: discovered != extracted + excluded + failed + unsupported
    doc.completeness.discovered_count = 100; // actual items = 2

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("COMPLETENESS_INVARIANT_VIOLATED");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].path, "completeness");
}

#[test]
fn test_completed_document_with_failed_items_rejected() {
    let mut doc = create_base_document();
    // 3 discovered, 3 eligible, 2 extracted, 0 excluded, 1 failed, 0 unsupported
    doc.completeness = CompletenessReport::new(3, 3, 2, 0, 1, 0);
    doc.extraction_status = ExtractionStatus::Completed;

    let report = doc.validation_report();
    assert!(!report.is_valid());
    let issues = report.filter_by_code("COMPLETENESS_STATUS_INCONSISTENT");
    assert_eq!(issues.len(), 1);
}

#[test]
fn test_empty_exclusion_paths_and_empty_warning_fields_rejected() {
    let mut doc = create_base_document();
    doc.exclusions.push(web_repo_to_ai_cdm::ExclusionRecord {
        original_path: "  ".to_string(),
        reason: web_repo_to_ai_cdm::ExclusionReason::BinaryFile,
        pattern: None,
        details: None,
        byte_size: None,
    });
    doc.warnings.push(ExtractionWarning::new(
        "",
        "structure",
        "Some message",
        WarningSeverity::Warning,
    ));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.filter_by_code("EXCLUSION_PATH_EMPTY").len(), 1);
    assert_eq!(report.filter_by_code("WARNING_CODE_EMPTY").len(), 1);
}

#[test]
fn test_validation_report_filtering_and_formatting() {
    let mut doc = create_base_document();
    doc.sections[0].line_range = Some(LineRange::new(0, 5));
    doc.sections[1].byte_range = Some(ByteRange::new(100, 10));

    let report = doc.validation_report();
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 2);
    assert_eq!(report.warning_count(), 0);

    let sec0_issues = report.filter_by_path_prefix("sections[0]");
    assert_eq!(sec0_issues.len(), 1);
    assert_eq!(sec0_issues[0].code, "LINE_RANGE_ZERO");

    let sec1_issues = report.filter_by_path_prefix("sections[1]");
    assert_eq!(sec1_issues.len(), 1);
    assert_eq!(sec1_issues[0].code, "BYTE_RANGE_INVERTED");

    let summary = report.summary();
    assert!(summary.contains("CDM validation failed"));
    assert!(summary.contains("LINE_RANGE_ZERO"));
    assert!(summary.contains("BYTE_RANGE_INVERTED"));
}

#[test]
fn test_permissive_vs_strict_validation_modes() {
    let mut doc = create_base_document();
    // Tamper with section body
    doc.sections[0].body.push_str("// Tampered\n");
    // Empty title triggers warning
    doc.metadata.title = None;

    // Permissive config ignores hash verification and completeness
    let permissive = ValidationConfig::permissive();
    let report_permissive = doc.validate_with_config(&permissive);
    assert!(report_permissive.is_valid());

    // Default config catches hash mismatch
    let report_default = doc.validation_report();
    assert!(!report_default.is_valid());

    // Recompute hash to satisfy default config
    doc.sections[0].content_hash =
        web_repo_to_ai_cdm::CdmSection::compute_hash(&doc.sections[0].body);
    let report_fixed = doc.validation_report();
    assert!(report_fixed.is_valid());
    assert!(report_fixed.has_warnings()); // TITLE_EMPTY warning

    // Strict config fails on the warnings (TITLE_EMPTY and TOKEN_METRICS_CHAR_MISMATCH)
    let strict = ValidationConfig::strict();
    let report_strict = doc.validate_with_config(&strict);
    assert!(!report_strict.is_valid());
    assert_eq!(report_strict.warning_count(), 2);
    assert_eq!(report_strict.filter_by_code("TITLE_EMPTY").len(), 1);
    assert_eq!(
        report_strict
            .filter_by_code("TOKEN_METRICS_CHAR_MISMATCH")
            .len(),
        1
    );
}
