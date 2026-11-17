//! Integration tests for CDM schema versioning, diffing, and migration framework.
//!
//! Validates:
//! 1. Multi-step legacy migration from v0.8.0 to v1.0.0.
//! 2. Direct preview migration from v0.9.0 to v1.0.0.
//! 3. Auto-migration via `JsonOptions::with_auto_migrate(true)`.
//! 4. Convenience deserialization via `CdmDocument::from_json_migrated`.
//! 5. Structural and semantic document diffing via `SchemaDiff`.
//! 6. Migrated documents pass all invariant and validation checks.
//! 7. Unregistered migration paths and downgrade attempts return typed errors.
//! 8. SemVer parsing, ordering, and compatibility checks.

use std::io::Cursor;
use web_repo_to_ai_cdm::{
    migrate_document_json, migrate_document_value, CdmDocument, CdmSection, JsonOptions,
    MigrationRegistry, SchemaVersion,
};

#[test]
fn test_semver_parsing_and_ordering_matrix() {
    let v0_8_0 = SchemaVersion::parse("0.8.0").unwrap();
    let v0_9_0 = SchemaVersion::parse("0.9.0").unwrap();
    let v0_9_1 = SchemaVersion::parse("0.9.1").unwrap();
    let v1_0_0 = SchemaVersion::parse("1.0.0").unwrap();
    let v1_0_1 = SchemaVersion::parse("1.0.1").unwrap();
    let v1_1_0 = SchemaVersion::parse("1.1.0").unwrap();
    let v2_0_0 = SchemaVersion::parse("2.0.0").unwrap();

    assert!(v0_8_0 < v0_9_0);
    assert!(v0_9_0 < v0_9_1);
    assert!(v0_9_1 < v1_0_0);
    assert!(v1_0_0 < v1_0_1);
    assert!(v1_0_1 < v1_1_0);
    assert!(v1_1_0 < v2_0_0);

    // Compatibility
    assert!(v1_0_0.is_compatible_with(&v1_1_0));
    assert!(v1_1_0.is_compatible_with(&v1_0_1));
    assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    assert!(!v0_8_0.is_compatible_with(&v0_9_0));
}

#[test]
fn test_multistep_legacy_migration_v0_8_to_v1_0() {
    let legacy_payload = serde_json::json!({
        "schema_version": "0.8.0",
        "url": "https://github.com/tokio-rs/tokio",
        "canonical_url": "https://github.com/tokio-rs/tokio",
        "content_blocks": [
            {
                "block_id": "blk-readme",
                "text": "# Tokio\nA runtime for writing reliable asynchronous applications.\n",
                "url": "https://github.com/tokio-rs/tokio/blob/master/README.md"
            },
            {
                "block_id": "blk-lib",
                "text": "pub mod runtime;\npub mod task;\n",
                "url": "https://github.com/tokio-rs/tokio/blob/master/tokio/src/lib.rs"
            }
        ]
    });

    let doc = migrate_document_value(legacy_payload).expect("Migration must succeed");

    assert_eq!(doc.schema_version, "1.0.0");
    assert_eq!(doc.canonical_url, "https://github.com/tokio-rs/tokio");
    assert_eq!(doc.sections.len(), 2);

    // Verify field remapping
    assert_eq!(doc.sections[0].section_id, "blk-readme");
    assert!(doc.sections[0].body.contains("A runtime for writing"));
    assert_eq!(
        doc.sections[0].content_hash,
        CdmSection::compute_hash(&doc.sections[0].body)
    );
    assert!(doc.sections[0].token_metrics.is_some());

    assert_eq!(doc.sections[1].section_id, "blk-lib");
    assert_eq!(
        doc.sections[1].content_hash,
        CdmSection::compute_hash(&doc.sections[1].body)
    );

    // Verify completeness report was derived
    assert!(doc.completeness.formula_valid());
    assert!(doc.completeness.is_complete());
    assert_eq!(doc.completeness.discovered_count, 2);
    assert_eq!(doc.completeness.extracted_count, 2);

    // Verify hierarchy tree was created
    assert_eq!(doc.hierarchy.id, "root");
    assert_eq!(doc.hierarchy.children.len(), 2);
    assert_eq!(
        doc.hierarchy.children[0].section_id,
        Some("blk-readme".to_string())
    );
    assert_eq!(
        doc.hierarchy.children[1].section_id,
        Some("blk-lib".to_string())
    );

    // Verify that the migrated document satisfies all CDM validation invariants!
    let report = doc.validation_report();
    assert!(report.is_valid(), "Validation issues: {:?}", report.issues);
    assert!(doc.validate().is_ok());
}

#[test]
fn test_direct_preview_migration_v0_9_to_v1_0() {
    let preview_json = serde_json::json!({
        "schema_version": "0.9.0",
        "canonical_url": "https://docs.rs/serde/latest/serde/",
        "source_type": "webpage_static",
        "retrieved_at": "2026-11-17T10:00:00Z",
        "content_type": "text/html",
        "source_identity": {
            "raw_input": "https://docs.rs/serde",
            "canonical_url": "https://docs.rs/serde/latest/serde/",
            "domain": "docs.rs"
        },
        "sections": [
            {
                "section_id": "sec-overview",
                "source_url": "https://docs.rs/serde/latest/serde/#overview",
                "kind": "prose",
                "body": "Serde is a framework for serializing and deserializing Rust data structures.\n"
            }
        ]
    });

    let doc = migrate_document_value(preview_json).expect("Migration must succeed");

    assert_eq!(doc.schema_version, "1.0.0");
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].section_id, "sec-overview");
    assert!(doc.sections[0].token_metrics.is_some());
    assert_eq!(doc.completeness.discovered_count, 1);
    assert!(doc.completeness.is_complete());

    let report = doc.validation_report();
    assert!(report.is_valid(), "Validation issues: {:?}", report.issues);
}

#[test]
fn test_auto_migration_via_json_options() {
    let legacy_str = serde_json::to_string(&serde_json::json!({
        "schema_version": "0.8.0",
        "canonical_url": "https://example.com/api",
        "content_blocks": [
            {
                "block_id": "b-1",
                "text": "API Reference documentation\n",
                "url": "https://example.com/api/v1"
            }
        ]
    }))
    .unwrap();

    // 1. Deserialization with default options (auto_migrate = false) fails or enforces strict version
    let strict_options = JsonOptions::default();
    let res_strict = web_repo_to_ai_cdm::from_json_with_options(&legacy_str, &strict_options);
    assert!(res_strict.is_err());

    // 2. Deserialization with auto_migrate enabled succeeds
    let auto_options = JsonOptions::default().with_auto_migrate(true);
    let doc = web_repo_to_ai_cdm::from_json_with_options(&legacy_str, &auto_options)
        .expect("Auto migration should succeed");

    assert_eq!(doc.schema_version, "1.0.0");
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].section_id, "b-1");

    // 3. from_slice_with_options
    let doc_slice =
        web_repo_to_ai_cdm::from_slice_with_options(legacy_str.as_bytes(), &auto_options)
            .expect("Auto migration from slice should succeed");
    assert_eq!(doc_slice.schema_version, "1.0.0");

    // 4. from_reader_with_options
    let cursor = Cursor::new(legacy_str.as_bytes());
    let doc_reader = web_repo_to_ai_cdm::from_reader_with_options(cursor, &auto_options)
        .expect("Auto migration from reader should succeed");
    assert_eq!(doc_reader.schema_version, "1.0.0");
}

#[test]
fn test_cdm_document_from_json_migrated_convenience() {
    let legacy_json = r#"{
        "schema_version": "0.9.0",
        "canonical_url": "https://example.com/convenience",
        "sections": [
            {
                "section_id": "sec-conv",
                "source_url": "https://example.com/convenience#1",
                "kind": "prose",
                "body": "Convenience method test.\n"
            }
        ]
    }"#;

    let doc =
        CdmDocument::from_json_migrated(legacy_json).expect("Convenience migration must work");
    assert_eq!(doc.schema_version, "1.0.0");
    assert_eq!(doc.sections[0].section_id, "sec-conv");
    assert!(doc.validate().is_ok());

    let val: serde_json::Value = serde_json::from_str(legacy_json).unwrap();
    let doc2 = CdmDocument::from_value_migrated(val).expect("From value migrated must work");
    assert_eq!(doc2.schema_version, "1.0.0");
}

#[test]
fn test_schema_diff_between_versions() {
    let doc_v1 = migrate_document_json(
        r#"{
        "schema_version": "0.9.0",
        "canonical_url": "https://example.com/diff",
        "sections": [
            {
                "section_id": "sec-1",
                "source_url": "https://example.com/diff#1",
                "kind": "prose",
                "body": "Original text\n"
            }
        ]
    }"#,
    )
    .unwrap();

    let mut doc_v2 = doc_v1.clone();
    doc_v2.sections[0].body = "Updated text with edits\n".to_string();
    doc_v2.sections[0].content_hash = CdmSection::compute_hash(&doc_v2.sections[0].body);

    let diff = doc_v1.diff(&doc_v2);
    assert!(!diff.is_identical());
    assert!(diff.has_content_changes());
    assert_eq!(diff.sections_modified.len(), 1);
    assert_eq!(diff.sections_modified[0].section_id, "sec-1");
    assert!(diff.sections_modified[0].body_changed);
    assert!(diff.sections_modified[0].hash_changed);

    let summary = diff.summary();
    assert!(summary.contains("Sections modified (1)"));
}

#[test]
fn test_unregistered_migration_and_downgrade_errors() {
    let registry = MigrationRegistry::default();

    // Jumps from unsupported version
    let bad_version = SchemaVersion::new(0, 1, 0);
    let target = SchemaVersion::v1_0_0();
    let err = registry.find_path(&bad_version, &target);
    assert!(err.is_err());

    // Downgrade attempt
    let modern_doc = serde_json::json!({
        "schema_version": "1.0.0",
        "canonical_url": "https://example.com/modern"
    });
    let downgrade_err = registry
        .migrate_value(modern_doc, Some(&SchemaVersion::v0_9_0()))
        .unwrap_err();
    assert!(downgrade_err.to_string().contains("Downgrading"));
}
