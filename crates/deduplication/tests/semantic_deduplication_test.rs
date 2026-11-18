//! # Semantic Near-Duplicate Deduplication Integration Tests (`crates/deduplication`)
//!
//! Validates MinHash and SimHash near-duplicate detection, connected cluster grouping,
//! canonical election policies, pointer and removal strategies, exclusion logging,
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
    deduplicate_semantic, SemanticCanonicalPolicy, SemanticDeduplicationConfig,
    SemanticDeduplicationReport, SemanticDeduplicationStrategy, SemanticDeduplicator,
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

fn build_semantic_test_document() -> CdmDocument {
    // Section A1: Original comprehensive documentation
    let body_a1 = "\
# Distributed Queue Processor Overview
The distributed queue engine coordinates asynchronous worker processes across multiple
isolated containers. It provides reliable at-least-once message delivery, exponential
backoff retry policies, persistent dead-letter queue storage, and automated health checks.
All internal state is synchronized through durable transactional database leases with automated failover.";

    // Section A2: Near-duplicate documentation (minor wording change)
    let body_a2 = "\
# Distributed Queue Processor Guide
The distributed queue engine coordinates asynchronous worker processes across multiple
isolated containers. It provides reliable at-least-once message delivery, exponential
backoff retry policies, persistent dead-letter queue storage, and automated health checks.
All internal state is synchronized through durable transactional database leases.";

    // Section B1: Comprehensive code module
    let body_b1 = "\
pub fn process_event_batch(events: &[EventPayload], max_concurrency: usize) -> Result<BatchReport, ProcessingError> {
    // Process all events sequentially with comprehensive input validation and metrics tracking
    let mut successes = 0;
    let mut failures = 0;
    for event in events {
        if validate_event(event) {
            successes += 1;
        } else {
            failures += 1;
        }
    }
    Ok(BatchReport { successes, failures })
}
";

    // Section B2: Near-duplicate code snippet (minor comment variation)
    let body_b2 = "\
pub fn process_event_batch(events: &[EventPayload], max_concurrency: usize) -> Result<BatchReport, ProcessingError> {
    // Process all events sequentially with input validation
    let mut successes = 0;
    let mut failures = 0;
    for event in events {
        if validate_event(event) {
            successes += 1;
        } else {
            failures += 1;
        }
    }
    Ok(BatchReport { successes, failures })
}
";

    // Section C: Standalone unique section (completely different topic)
    let body_c = "\
# Authentication Security Specification
This section details OAuth 2.0 PKCE authentication flow and JSON Web Token verification
standards implemented across all public API ingress gateways.";

    let sec_a1 = make_section(
        "sec-queue-overview",
        "docs/queue/overview.md",
        SectionKind::Prose,
        "markdown",
        body_a1,
    );
    let sec_a2 = make_section(
        "sec-queue-guide",
        "docs/queue/guide.md",
        SectionKind::Prose,
        "markdown",
        body_a2,
    );

    let sec_b1 = make_section(
        "sec-code-full",
        "src/engine/batch.rs",
        SectionKind::CodeFile,
        "rust",
        body_b1,
    );
    let sec_b2 = make_section(
        "sec-code-snippet",
        "docs/examples/batch_snippet.rs",
        SectionKind::CodeSnippet,
        "rust",
        body_b2,
    );

    let sec_c = make_section(
        "sec-auth",
        "docs/security/auth.md",
        SectionKind::Prose,
        "markdown",
        body_c,
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
        sections: vec![sec_a1, sec_a2, sec_b1, sec_b2, sec_c],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    }
}

#[test]
fn test_minhash_near_duplicate_cluster_discovery() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::balanced();
    let deduplicator = SemanticDeduplicator::new(config);

    let clusters = deduplicator.find_near_duplicate_clusters(&doc.sections);

    // 2 near-duplicate clusters discovered (Queue doc cluster, Code batch cluster)
    assert_eq!(clusters.len(), 2);

    // Cluster 1: Queue docs
    let doc_cluster = clusters
        .iter()
        .find(|c| c.canonical_section_id == "sec-queue-overview")
        .unwrap();
    assert_eq!(doc_cluster.near_duplicates.len(), 1);
    assert_eq!(doc_cluster.near_duplicates[0].section_id, "sec-queue-guide");
    assert!(doc_cluster.average_similarity >= 0.75);

    // Cluster 2: Code batch
    let code_cluster = clusters
        .iter()
        .find(|c| c.canonical_section_id == "sec-code-full")
        .unwrap();
    assert_eq!(code_cluster.near_duplicates.len(), 1);
    assert_eq!(
        code_cluster.near_duplicates[0].section_id,
        "sec-code-snippet"
    );
}

#[test]
fn test_semantic_pointer_replacement_strategy() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::balanced();
    let (dedup_doc, report) = deduplicate_semantic(&doc, &config);

    assert_eq!(dedup_doc.sections.len(), 5);
    assert_eq!(report.near_duplicate_sections_found, 2);
    assert_eq!(report.near_duplicate_clusters_found, 2);
    assert!(report.byte_reduction > 0);
    assert!(report.token_reduction > 0);

    // Canonical sections preserve original content
    let canonical_doc = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-queue-overview")
        .unwrap();
    assert!(canonical_doc.body.contains("The distributed queue engine"));

    // Near-duplicate sections replaced with annotated pointer comments
    let dup_doc = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-queue-guide")
        .unwrap();
    assert!(dup_doc.body.starts_with("<!-- [Near-duplicate omitted:"));
    assert!(dup_doc.body.contains("sec-queue-overview"));

    // All section hashes remain cryptographically verified
    assert!(dedup_doc.verify_section_hashes());

    // Provenance relationships recorded
    assert_eq!(dedup_doc.relationships.len(), 2);
    for rel in &dedup_doc.relationships {
        assert_eq!(rel.kind, RelationshipKind::DuplicateOf);
        assert!(rel.description.as_ref().unwrap().contains("Near-duplicate"));
    }
}

#[test]
fn test_semantic_remove_strategy_with_exclusions() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::aggressive();
    let (dedup_doc, report) = deduplicate_semantic(&doc, &config);

    // 5 original sections -> 2 near-duplicates removed -> 3 sections remaining
    assert_eq!(dedup_doc.sections.len(), 3);
    assert_eq!(report.near_duplicate_sections_found, 2);

    // Verify exclusions
    assert_eq!(dedup_doc.exclusions.len(), 2);
    for exc in &dedup_doc.exclusions {
        assert_eq!(exc.reason, ExclusionReason::Duplicate);
        assert!(exc.details.as_ref().unwrap().contains("Near-duplicate"));
        assert!(exc.byte_size.unwrap() > 0);
    }

    assert!(dedup_doc.verify_section_hashes());
}

#[test]
fn test_semantic_mark_only_strategy() {
    let doc = build_semantic_test_document();
    let mut config = SemanticDeduplicationConfig::balanced();
    config.strategy = SemanticDeduplicationStrategy::MarkOnly;
    let (dedup_doc, report) = deduplicate_semantic(&doc, &config);

    assert_eq!(dedup_doc.sections.len(), 5);
    assert_eq!(report.near_duplicate_sections_found, 2);
    assert_eq!(report.byte_reduction, 0);

    // Verify metadata attributes added to near-duplicates
    let dup_sec = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-queue-guide")
        .unwrap();
    assert_eq!(
        dup_sec.attributes.get("near_duplicate_of").unwrap(),
        "sec-queue-overview"
    );
    assert!(dup_sec.attributes.contains_key("near_duplicate_similarity"));
}

#[test]
fn test_simhash_algorithm_near_duplicate_detection() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::simhash_based(10);
    let (dedup_doc, report) = deduplicate_semantic(&doc, &config);

    assert!(report.near_duplicate_sections_found >= 1);
    assert!(dedup_doc.verify_section_hashes());
}

#[test]
fn test_canonical_election_structural_hierarchy() {
    let text1 = "Function implementing cryptographic signature generation and verification across token envelopes.";
    let text2 = "Function implementing cryptographic signature generation and verification across token envelopes with timing checks.";

    let sec_snippet = make_section(
        "sec-snippet",
        "docs/crypto.md",
        SectionKind::CodeSnippet,
        "rust",
        text1,
    );
    let sec_file = make_section(
        "sec-file",
        "src/crypto.rs",
        SectionKind::CodeFile,
        "rust",
        text2,
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
        sections: vec![sec_snippet, sec_file],
        relationships: Vec::new(),
        citations: Vec::new(),
        extraction_status: ExtractionStatus::Completed,
        warnings: Vec::new(),
        exclusions: Vec::new(),
        completeness: CompletenessReport::default(),
    };

    let mut config = SemanticDeduplicationConfig::balanced();
    config.canonical_policy = SemanticCanonicalPolicy::StructuralHierarchy;
    let (dedup_doc, _) = deduplicate_semantic(&doc, &config);

    // CodeFile must win over CodeSnippet
    let canonical = dedup_doc
        .sections
        .iter()
        .find(|s| s.section_id == "sec-file")
        .unwrap();
    assert!(!canonical.body.starts_with("<!-- [Near-duplicate"));
}

#[test]
fn test_min_words_threshold_ignores_short_phrases() {
    let short1 = "Status check ok";
    let short2 = "Status check ready";

    let sec1 = make_section("s1", "a.md", SectionKind::Prose, "text", short1);
    let sec2 = make_section("s2", "b.md", SectionKind::Prose, "text", short2);

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

    let config = SemanticDeduplicationConfig::balanced(); // min_words_threshold = 8
    let (dedup_doc, report) = deduplicate_semantic(&doc, &config);

    assert_eq!(report.near_duplicate_sections_found, 0);
    assert_eq!(dedup_doc.sections.len(), 2);
}

#[test]
fn test_semantic_deduplication_idempotency() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::balanced();

    let (pass1_doc, pass1_report) = deduplicate_semantic(&doc, &config);
    assert_eq!(pass1_report.near_duplicate_sections_found, 2);

    let (pass2_doc, pass2_report) = deduplicate_semantic(&pass1_doc, &config);
    assert_eq!(pass2_report.near_duplicate_sections_found, 0);
    assert_eq!(pass2_report.byte_reduction, 0);
    assert_eq!(pass1_doc.sections, pass2_doc.sections);
}

#[test]
fn test_semantic_deduplication_report_serialization() {
    let doc = build_semantic_test_document();
    let config = SemanticDeduplicationConfig::balanced();
    let (_, report) = deduplicate_semantic(&doc, &config);

    let md_summary = report.to_markdown_summary();
    assert!(md_summary.contains("### Semantic Near-Duplicate Deduplication Summary"));
    assert!(md_summary.contains(&doc.document_id.to_string()));
    assert!(md_summary.contains("| Canonical Section | Kind | Near-Duplicates |"));

    let json_str = serde_json::to_string_pretty(&report).expect("valid json serialization");
    let deserialized: SemanticDeduplicationReport =
        serde_json::from_str(&json_str).expect("valid json deserialization");
    assert_eq!(report, deserialized);
}
