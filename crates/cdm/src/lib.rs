//! # Canonical Context Document Model (`web-repo-to-ai-cdm`)
//!
//! Defines the versioned intermediate representation (CDM v1.0)
//! shared across extractors, normalizers, optimizers, tokenizers, chunkers, and exporters.
//!
//! Serving as the architectural nexus of the platform, every content acquisition source
//! (static HTML, dynamic Playwright browser rendering, public and private GitHub repositories,
//! PDF/OCR documents) maps cleanly into [`CdmDocument`]. All downstream AI format generators
//! (Markdown, Plain Text, JSON, HTML, JSONL, and ZIP packages) read strictly from [`CdmDocument`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod builder;
pub mod completeness;
pub mod diff;
pub mod error;
pub mod hierarchy;
pub mod json;
pub mod metadata;
pub mod migration;
pub mod model;
pub mod relationships;
pub mod section;
pub mod validator;
pub mod version;

// Primary re-exports
pub use builder::{CdmDocumentBuilder, CdmSectionBuilder};
pub use completeness::CompletenessReport;
pub use diff::{SchemaDiff, SectionDiff};
pub use error::{CdmError, Result};
pub use hierarchy::{HierarchyKind, HierarchyNode};
pub use json::{
    from_json, from_json_with_options, from_reader, from_reader_with_options, from_slice,
    from_slice_with_options, to_json_compact, to_json_pretty, to_writer, to_writer_pretty,
    JsonOptions, DEFAULT_MAX_JSON_BYTES,
};
pub use metadata::{Citation, DocumentMetadata, ExclusionReason, ExclusionRecord};
pub use migration::{
    migrate_document_json, migrate_document_value, Migration0_8To0_9, Migration0_9To1_0,
    MigrationRegistry, MigrationStep,
};
pub use model::{
    CdmDocument, ExtractionStatus, ExtractionWarning, RevisionInfo, SourceIdentity, SourceType,
    WarningSeverity,
};
pub use relationships::{Relationship, RelationshipKind};
pub use section::{ByteRange, CdmSection, LineRange, SectionKind, SectionTokenMetrics};
pub use validator::{
    CdmValidator, ValidationConfig, ValidationIssue, ValidationReport, ValidationSeverity,
};
pub use version::SchemaVersion;

/// Canonical Context Document Model (CDM) schema version constant.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Returns the supported CDM schema version string (`1.0.0`).
#[must_use]
pub const fn schema_version() -> &'static str {
    SCHEMA_VERSION
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use web_repo_to_ai_core::DocumentId;

    #[test]
    fn test_cdm_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-cdm");
        assert_eq!(schema_version(), "1.0.0");
        assert_eq!(SCHEMA_VERSION, "1.0.0");
    }

    #[test]
    fn test_end_to_end_cdm_document_construction() {
        let identity = SourceIdentity {
            raw_input: "https://github.com/owner/repo".to_string(),
            canonical_url: "https://github.com/owner/repo".to_string(),
            domain: Some("github.com".to_string()),
            repo_owner: Some("owner".to_string()),
            repo_name: Some("repo".to_string()),
            git_ref: Some("main".to_string()),
            subpath: None,
        };

        let section_code = CdmSectionBuilder::new(
            "sec-001",
            "https://github.com/owner/repo/blob/main/src/lib.rs",
            SectionKind::CodeFile,
            "pub fn answer() -> u32 { 42 }\n",
        )
        .title("src/lib.rs")
        .original_path("src/lib.rs")
        .language("rust")
        .line_range(1, 2)
        .byte_range(0, 31)
        .attribute("git_blob_sha", "abc1234567")
        .build();

        let section_readme = CdmSectionBuilder::new(
            "sec-002",
            "https://github.com/owner/repo/blob/main/README.md",
            SectionKind::Prose,
            "# My Cool Project\n\nHigh-performance AI converter.\n",
        )
        .title("README.md")
        .original_path("README.md")
        .language("markdown")
        .line_range(1, 3)
        .build();

        let mut root_node = HierarchyNode::root("owner/repo");
        let mut src_dir = HierarchyNode::directory("node-src", "src", "src");
        src_dir.add_child(HierarchyNode::file(
            "node-lib",
            "lib.rs",
            "src/lib.rs",
            Some("sec-001".to_string()),
            Some(31),
            Some(2),
        ));
        root_node.add_child(src_dir);
        root_node.add_child(HierarchyNode::file(
            "node-readme",
            "README.md",
            "README.md",
            Some("sec-002".to_string()),
            Some(48),
            Some(3),
        ));

        let rel = Relationship::new("sec-002", "sec-001", RelationshipKind::References)
            .with_description("README documents src/lib.rs API");

        let citation = Citation::new(
            "cit-001",
            "https://github.com/owner/repo",
            Some("owner/repo repository".to_string()),
            None,
        );

        let exclusion = ExclusionRecord::new(
            "target/debug/test.exe",
            ExclusionReason::BinaryFile,
            Some("target/**".to_string()),
            Some("Binary executable ignored".to_string()),
            Some(1024 * 512),
        );

        let completeness = CompletenessReport::new(3, 2, 2, 1, 0, 0);

        let doc = CdmDocumentBuilder::new(
            identity,
            SourceType::GitHubPublic,
            "https://github.com/owner/repo",
        )
        .document_id(DocumentId::new())
        .retrieved_at(Utc::now())
        .revision(RevisionInfo {
            git_commit: Some("9f8e7d6c5b4a321".to_string()),
            git_branch: Some("main".to_string()),
            ..Default::default()
        })
        .content_type("application/vnd.github.v3+json")
        .metadata(DocumentMetadata {
            title: Some("owner/repo".to_string()),
            description: Some("AI converter".to_string()),
            license: Some("MIT".to_string()),
            authors: vec!["Alice".to_string()],
            primary_language: Some("Rust".to_string()),
            language_breakdown: BTreeMap::from([
                ("Rust".to_string(), 31),
                ("Markdown".to_string(), 48),
            ]),
            ..Default::default()
        })
        .hierarchy(root_node)
        .add_section(section_code)
        .add_section(section_readme)
        .add_relationship(rel)
        .add_citation(citation)
        .add_exclusion(exclusion)
        .completeness(completeness)
        .build();

        assert_eq!(doc.schema_version, "1.0.0");
        assert_eq!(doc.source_type, SourceType::GitHubPublic);
        assert_eq!(doc.source_type.as_str(), "github_public");
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.relationships.len(), 1);
        assert_eq!(doc.citations.len(), 1);
        assert_eq!(doc.exclusions.len(), 1);
        assert!(doc.completeness.is_complete());
        assert!(doc.completeness.formula_valid());
        assert_eq!(doc.hierarchy.total_files(), 2);
        assert_eq!(doc.hierarchy.total_nodes(), 4);
        assert!(doc.verify_section_hashes());
        assert!(doc.total_char_count() > 0);
        assert!(doc.total_word_count() > 0);
        assert!(doc.total_estimated_tokens() > 0);
        assert!(doc.find_section("sec-001").is_some());
        assert!(doc.find_section("nonexistent").is_none());
    }
}
