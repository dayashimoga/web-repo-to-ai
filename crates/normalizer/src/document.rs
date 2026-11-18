//! # Document-Level Canonical Normalizer
//!
//! Orchestrates whole-document normalization across all sections of a [`CdmDocument`].
//! Normalizes sections according to their [`SectionKind`], aggregates granular transformation
//! telemetry into a [`NormalizationReport`], updates aggregate token metrics, and validates
//! document consistency.

use web_repo_to_ai_cdm::CdmDocument;

use crate::config::NormalizationConfig;
use crate::report::NormalizationReport;
use crate::section::SectionNormalizer;

/// High-level normalizer coordinating transformations across entire [`CdmDocument`] structures.
#[derive(Debug, Clone)]
pub struct DocumentNormalizer {
    config: NormalizationConfig,
}

impl DocumentNormalizer {
    /// Creates a new document normalizer with the specified configuration.
    #[must_use]
    pub fn new(config: NormalizationConfig) -> Self {
        Self { config }
    }

    /// Creates a document normalizer with default balanced settings.
    #[must_use]
    pub fn default_normalizer() -> Self {
        Self::new(NormalizationConfig::default())
    }

    /// Accesses the underlying normalization configuration.
    #[must_use]
    pub fn config(&self) -> &NormalizationConfig {
        &self.config
    }

    /// Normalizes all sections within a [`CdmDocument`], returning the updated document
    /// and a comprehensive [`NormalizationReport`].
    #[must_use]
    pub fn normalize_document(&self, doc: &CdmDocument) -> (CdmDocument, NormalizationReport) {
        let mut normalized_sections = Vec::with_capacity(doc.sections.len());
        let mut section_audits = Vec::with_capacity(doc.sections.len());

        for section in &doc.sections {
            let (norm_sec, audit) = SectionNormalizer::normalize_section(section, &self.config);
            normalized_sections.push(norm_sec);
            section_audits.push(audit);
        }

        let report = NormalizationReport::build(doc.document_id, section_audits);

        let mut normalized_doc = doc.clone();
        normalized_doc.sections = normalized_sections;

        // If any section was modified, update content hash in revision
        if report.sections_modified > 0 {
            let mut hasher = blake3::Hasher::new();
            for sec in &normalized_doc.sections {
                hasher.update(sec.content_hash.as_bytes());
            }
            let aggregate_hash = hasher.finalize().to_hex().to_string();
            normalized_doc.revision.content_hash_blake3 = Some(aggregate_hash);
        }

        (normalized_doc, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
        RevisionInfo, SectionKind, SectionTokenMetrics, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn build_test_cdm_document() -> CdmDocument {
        let doc_id = DocumentId::new();
        let body1 = "  # Main Heading  \r\n\r\nSome prose text with   trailing whitespace.  \r\n\r\n```RUST\r\nfn snippet() {}\r\n```\r\n";
        let body2 = "fn hello() {\r\n    println!(\"world\");   \r\n}\r\n";

        let sec1 = CdmSection {
            section_id: "sec-1".to_string(),
            parent_id: None,
            title: Some("Introduction".to_string()),
            kind: SectionKind::Prose,
            original_path: Some("docs/intro.md".to_string()),
            source_url: "https://example.com/docs/intro.md".to_string(),
            line_range: None,
            byte_range: None,
            language: Some("markdown".to_string()),
            content_hash: CdmSection::compute_hash(body1),
            body: body1.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(body1)),
            attributes: BTreeMap::new(),
        };

        let sec2 = CdmSection {
            section_id: "sec-2".to_string(),
            parent_id: None,
            title: Some("Snippet".to_string()),
            kind: SectionKind::CodeSnippet,
            original_path: Some("docs/intro.md".to_string()),
            source_url: "https://example.com/docs/intro.md#snippet".to_string(),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(body2),
            body: body2.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(body2)),
            attributes: BTreeMap::new(),
        };

        CdmDocument {
            schema_version: "1.0.0".to_string(),
            document_id: doc_id,
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
        }
    }

    #[test]
    fn test_normalize_document_end_to_end() {
        let doc = build_test_cdm_document();
        let normalizer = DocumentNormalizer::default_normalizer();
        let (norm_doc, report) = normalizer.normalize_document(&doc);

        assert_eq!(norm_doc.sections.len(), 2);
        assert!(report.sections_modified > 0);
        assert!(norm_doc.verify_section_hashes());
        assert!(norm_doc.sections[0]
            .body
            .starts_with("# Main Heading\n\nSome prose text"));
        assert!(norm_doc.sections[0]
            .body
            .contains("```rust\nfn snippet() {}\n```"));
        assert_eq!(
            norm_doc.sections[1].body,
            "fn hello() {\n    println!(\"world\");\n}"
        );
    }
}
