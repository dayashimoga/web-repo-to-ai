//! # Exact Content Deduplication Integration Tests (`crates/deduplication`)
//!
//! Validates exact cryptographic hash comparison, normalized deduplication, canonical
//! election policies, removal and pointer substitution strategies, exclusion logging,
//! relationship linking, idempotency, and telemetry reporting.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExclusionReason,
    ExtractionStatus, HierarchyNode, LineRange, RelationshipKind, RevisionInfo, SectionKind,
    SectionTokenMetrics, SourceIdentity, SourceType,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_deduplication::{
    compute_dedup_hash, deduplicate_document, CanonicalSelectionPolicy, ExactDeduplicationConfig,
    ExactDeduplicationReport, HashScope,
};

fn make_section(id: &str, path: &str, kind: SectionKind, language: &str, body: &str) -> CdmSection {
    CdmSection {
        section_id: id.to_string(),
        parent_id: None,
        title: Some(id.to_string()),
        kind,
        original_path: Some(path.to_string()),
        source_url: format!("https://example.com/{path}"),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some(language.to_string()),
        content_hash: CdmSection::compute_hash(body),
        body: body.to_string(),
        token_metrics: Some(SectionTokenMetrics::compute_baseline(body)),
        attributes: BTreeMap::new(),
    }
}

fn build_multi_cluster_document() -> CdmDocument {
    let synthetic_shared_notice = "\
Synthetic Disclaimer Notice for Module Distribution.
All rights reserved under the modular project distribution terms.
This document is distributed across multiple subpackages to provide
consistent legal and attribution documentation for all downstream consumers.
Please refer to the root documentation for comprehensive usage terms.";

    let synthetic_shared_code = "\
pub fn execute_retry_policy<F, R>(mut operation: F, max_attempts: usize) -> Result<R, &'static str>
where
    F: FnMut() -> Result<R, &'static str>,
{
    let mut attempts = 0;
    while attempts < max_attempts {
        match operation() {
            Ok(val) => return Ok(val),
            Err(_) => attempts += 1,
        }
    }
    Err(\"Max retry limit exceeded\")
}
";

    let synthetic_shared_nav = "\
Site Navigation Links:
- Documentation: /docs/overview
- API Reference: /docs/api/v1
- Source Code Repository: /repo/tree/main
- Issue Tracker: /repo/issues
- Project Roadmap: /docs/roadmap
";

    let unique_prose = "\
# Project Architecture Overview
This section outlines the standalone core engine processing flow,
verifying that unique sections are completely untouched by deduplication.";

    let sec1 = make_section(
        "sec-notice-root",
        "NOTICE.txt",
        SectionKind::Prose,
        "text",
        synthetic_shared_notice,
    );
    let sec2 = make_section(
        "sec-notice-sub",
        "crates/sub/NOTICE.txt",
        SectionKind::Prose,
        "text",
        synthetic_shared_notice,
    );
    let sec3 = make_section(
        "sec-notice-vendor",
        "vendor/pkg/NOTICE.txt",
        SectionKind::Prose,
        "text",
        synthetic_shared_notice,
    );

    let sec4 = make_section(
        "sec-code-src",
        "src/retry.rs",
        SectionKind::CodeFile,
        "rust",
        synthetic_shared_code,
    );
    let sec5 = make_section(
        "sec-code-fixture",
        "tests/fixtures/retry.rs",
        SectionKind::CodeSnippet,
        "rust",
        synthetic_shared_code,
    );

    let sec6 = make_section(
        "sec-nav-header",
        "templates/header.html",
        SectionKind::Navigation,
        "html",
        synthetic_shared_nav,
    );
    let sec7 = make_section(
        "sec-nav-footer",
        "templates/footer.html",
        SectionKind::Navigation,
        "html",
        synthetic_shared_nav,
    );

    let sec8 = make_section(
        "sec-unique",
        "docs/architecture.md",
        SectionKind::Prose,
        "markdown",
        unique_prose,
    );

    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
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
        sections: vec![sec1, sec2, sec3, sec4, sec5, sec6, sec7, sec8],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    }
}

#[test]
fn test_exact_bytes_hash_vs_normalized_hash() {
    let text1 = "fn calculate() {\r\n    let x = 42;   \r\n}\r\n";
    let text2 = "fn calculate() {\n    let x = 42;\n}\n";

    let hash_exact1 = compute_dedup_hash(text1, HashScope::ExactBytes);
    let hash_exact2 = compute_dedup_hash(text2, HashScope::ExactBytes);
    assert_ne!(hash_exact1, hash_exact2);

    let hash_norm1 = compute_dedup_hash(text1, HashScope::Normalized);
    let hash_norm2 = compute_dedup_hash(text2, HashScope::Normalized);
    assert_eq!(hash_norm1, hash_norm2);
}

#[test]
fn test_aggressive_removal_strategy_with_exclusion_logging() {
    let doc = build_multi_cluster_document();
    assert_eq!(doc.sections.len(), 8);

    let config = ExactDeduplicationConfig::aggressive();
    let (dedup_doc, report) = deduplicate_document(&doc, &config);

    // 8 sections total:
    // - notice: 3 copies -> 1 canonical, 2 removed
    // - code: 2 copies -> 1 canonical, 1 removed
    // - nav: 2 copies -> 1 canonical, 1 removed
    // - unique: 1 copy -> 1 canonical
    // Total remaining sections: 4
    assert_eq!(dedup_doc.sections.len(), 4);
    assert_eq!(report.duplicate_sections_found, 4);
    assert_eq!(report.duplicate_clusters_found, 3);
    assert_eq!(report.unique_sections, 4);
    assert!(report.byte_reduction > 0);
    assert!(report.token_reduction > 0);

    // Verify exclusions
    assert_eq!(dedup_doc.exclusions.len(), 4);
    for exc in &dedup_doc.exclusions {
        assert_eq!(exc.reason, ExclusionReason::Duplicate);
        assert!(exc.details.as_ref().unwrap().contains("Exact duplicate"));
        assert!(exc.byte_size.unwrap() > 0);
    }

    // Verify relationships
    assert_eq!(dedup_doc.relationships.len(), 4);
    for rel in &dedup_doc.relationships {
        assert_eq!(rel.kind, RelationshipKind::DuplicateOf);
    }

    assert!(dedup_doc.verify_section_hashes());
}

#[test]
fn test_conservative_pointer_replacement_strategy() {
    let doc = build_multi_cluster_document();
    let config = ExactDeduplicationConfig::conservative();
    let (dedup_doc, report) = deduplicate_document(&doc, &config);

    // In pointer replacement, all 8 sections remain in the document outline
    assert_eq!(dedup_doc.sections.len(), 8);
    assert_eq!(report.duplicate_sections_found, 4);

    // Canonical sections have their original bodies intact
    let canonical_code = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-code-src")
        .unwrap();
    assert!(canonical_code.body.contains("pub fn execute_retry_policy"));

    // Duplicate sections have pointer bodies
    let dup_code = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-code-fixture")
        .unwrap();
    assert!(dup_code
        .body
        .starts_with("<!-- [Duplicate omitted: identical to `sec-code-src`] -->"));

    // Hashes must match the new pointer bodies
    assert!(dedup_doc.verify_section_hashes());
    assert!(report.byte_reduction > 0);
}

#[test]
fn test_lossless_mark_only_strategy() {
    let doc = build_multi_cluster_document();
    let config = ExactDeduplicationConfig::lossless();
    let (dedup_doc, report) = deduplicate_document(&doc, &config);

    assert_eq!(dedup_doc.sections.len(), 8);
    assert_eq!(report.duplicate_sections_found, 4);
    assert_eq!(report.byte_reduction, 0);
    assert_eq!(dedup_doc.exclusions.len(), 0);
    assert_eq!(dedup_doc.relationships.len(), 4);

    // Bodies are 100% byte-for-byte untouched
    for (orig, after) in doc.sections.iter().zip(dedup_doc.sections.iter()) {
        assert_eq!(orig.body, after.body);
    }
}

#[test]
fn test_canonical_election_structural_hierarchy() {
    let code = "pub fn compute_hash_sum() {\n    println!(\"calculating\");\n}\n";
    let sec_snippet = make_section(
        "snippet",
        "docs/run.md",
        SectionKind::CodeSnippet,
        "rust",
        code,
    );
    let sec_file = make_section("file", "src/run.rs", SectionKind::CodeFile, "rust", code);

    let doc = CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
        source_type: SourceType::WebpageStatic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![sec_snippet, sec_file],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    };

    let config = ExactDeduplicationConfig::aggressive();
    let (dedup_doc, _) = deduplicate_document(&doc, &config);

    // CodeFile must win over CodeSnippet regardless of index order
    assert_eq!(dedup_doc.sections.len(), 1);
    assert_eq!(dedup_doc.sections[0].section_id, "file");
}

#[test]
fn test_canonical_election_shortest_path() {
    let text = "Synthetic repeated configuration block across multiple test suites.";
    let sec_deep = make_section(
        "deep",
        "crates/service/src/vendor/config.txt",
        SectionKind::Prose,
        "text",
        text,
    );
    let sec_root = make_section("root", "config.txt", SectionKind::Prose, "text", text);

    let doc = CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
        source_type: SourceType::WebpageStatic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![sec_deep, sec_root],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    };

    let mut config = ExactDeduplicationConfig::aggressive();
    config.canonical_policy = CanonicalSelectionPolicy::ShortestPath;
    let (dedup_doc, _) = deduplicate_document(&doc, &config);

    assert_eq!(dedup_doc.sections.len(), 1);
    assert_eq!(dedup_doc.sections[0].section_id, "root");
}

#[test]
fn test_canonical_election_preferred_prefix() {
    let text = "Shared common config template across repository modules.";
    let sec_vendor = make_section(
        "vendor",
        "vendor/template.conf",
        SectionKind::Prose,
        "text",
        text,
    );
    let sec_src = make_section(
        "src",
        "src/config/template.conf",
        SectionKind::Prose,
        "text",
        text,
    );

    let doc = CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
        source_type: SourceType::WebpageStatic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![sec_vendor, sec_src],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    };

    let mut config = ExactDeduplicationConfig::aggressive();
    config.canonical_policy = CanonicalSelectionPolicy::PreferredPrefix("src/".to_string());
    let (dedup_doc, _) = deduplicate_document(&doc, &config);

    assert_eq!(dedup_doc.sections.len(), 1);
    assert_eq!(dedup_doc.sections[0].section_id, "src");
}

#[test]
fn test_min_byte_threshold_ignores_tiny_stubs() {
    let stub = "{}\n"; // 3 bytes
    let sec1 = make_section("s1", "a.rs", SectionKind::CodeFile, "rust", stub);
    let sec2 = make_section("s2", "b.rs", SectionKind::CodeFile, "rust", stub);

    let doc = CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
        source_type: SourceType::WebpageStatic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
        metadata: DocumentMetadata::default(),
        hierarchy: HierarchyNode::root("root"),
        sections: vec![sec1, sec2],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    };

    let config = ExactDeduplicationConfig::aggressive(); // default threshold 16
    let (dedup_doc, report) = deduplicate_document(&doc, &config);

    assert_eq!(report.duplicate_sections_found, 0);
    assert_eq!(dedup_doc.sections.len(), 2);
}

#[test]
fn test_deduplication_idempotency() {
    let doc = build_multi_cluster_document();
    let config = ExactDeduplicationConfig::aggressive();

    let (pass1_doc, pass1_report) = deduplicate_document(&doc, &config);
    assert_eq!(pass1_report.duplicate_sections_found, 4);

    let (pass2_doc, pass2_report) = deduplicate_document(&pass1_doc, &config);
    assert_eq!(pass2_report.duplicate_sections_found, 0);
    assert_eq!(pass2_report.byte_reduction, 0);
    assert_eq!(pass1_doc.sections, pass2_doc.sections);
}

#[test]
fn test_deduplication_report_markdown_and_json_serialization() {
    let doc = build_multi_cluster_document();
    let config = ExactDeduplicationConfig::aggressive();
    let (_, report) = deduplicate_document(&doc, &config);

    let md_summary = report.to_markdown_summary();
    assert!(md_summary.contains("### Exact Content Deduplication Summary"));
    assert!(md_summary.contains(&doc.document_id.to_string()));
    assert!(md_summary.contains("| Canonical Section | Kind | Duplicates |"));

    let json_str = serde_json::to_string_pretty(&report).expect("valid json serialization");
    let deserialized: ExactDeduplicationReport =
        serde_json::from_str(&json_str).expect("valid json deserialization");
    assert_eq!(report, deserialized);
}
