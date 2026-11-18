//! # Document-Level Deduplication Engine
//!
//! Applies exact deduplication strategies across entire [`CdmDocument`] structures,
//! substituting reference pointers or removing redundant sections, logging auditable
//! [`ExclusionRecord`] entries, and registering `RelationshipKind::DuplicateOf` edges.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, ExclusionReason, ExclusionRecord, Relationship, RelationshipKind,
    SectionTokenMetrics,
};

use crate::cluster::DuplicateCluster;
use crate::config::{DeduplicationStrategy, ExactDeduplicationConfig};
use crate::exact::ExactDeduplicator;
use crate::report::ExactDeduplicationReport;

/// High-level deduplicator orchestrating exact duplicate detection and consolidation across [`CdmDocument`] instances.
#[derive(Debug, Clone)]
pub struct DocumentDeduplicator {
    config: ExactDeduplicationConfig,
}

impl DocumentDeduplicator {
    /// Creates a new document deduplicator with the given configuration.
    #[must_use]
    pub fn new(config: ExactDeduplicationConfig) -> Self {
        Self { config }
    }

    /// Creates a document deduplicator with default balanced settings.
    #[must_use]
    pub fn default_deduplicator() -> Self {
        Self::new(ExactDeduplicationConfig::default())
    }

    /// Accesses the underlying deduplication configuration.
    #[must_use]
    pub fn config(&self) -> &ExactDeduplicationConfig {
        &self.config
    }

    /// Deduplicates all identical sections across a [`CdmDocument`], returning the updated
    /// document and a comprehensive [`ExactDeduplicationReport`].
    #[must_use]
    pub fn deduplicate_document(
        &self,
        doc: &CdmDocument,
    ) -> (CdmDocument, ExactDeduplicationReport) {
        let total_sections = doc.sections.len();
        let bytes_before: usize = doc.sections.iter().map(|s| s.body.len()).sum();
        let tokens_before: usize = doc
            .sections
            .iter()
            .map(|s| {
                s.token_metrics
                    .as_ref()
                    .map(|m| m.estimated_tokens)
                    .unwrap_or_else(|| s.body.chars().count().div_ceil(4))
            })
            .sum();

        // 1. Discover duplicate clusters
        let clusters = ExactDeduplicator::find_duplicate_clusters(&doc.sections, &self.config);

        if clusters.is_empty() {
            let report = ExactDeduplicationReport::new(
                doc.document_id,
                total_sections,
                bytes_before,
                bytes_before,
                tokens_before,
                tokens_before,
                Vec::new(),
            );
            return (doc.clone(), report);
        }

        // 2. Build index of duplicate section ID -> cluster
        let mut dup_map: BTreeMap<&str, &DuplicateCluster> = BTreeMap::new();
        for cluster in &clusters {
            for dup_id in &cluster.duplicate_section_ids {
                dup_map.insert(dup_id.as_str(), cluster);
            }
        }

        // 3. Process sections according to strategy
        let mut new_sections = Vec::with_capacity(doc.sections.len());
        let mut new_exclusions = Vec::new();
        let mut new_relationships = Vec::new();

        for section in &doc.sections {
            if let Some(cluster) = dup_map.get(section.section_id.as_str()) {
                // Section is a duplicate
                let original_path = section
                    .original_path
                    .clone()
                    .unwrap_or_else(|| section.section_id.clone());

                // Build relationship linking duplicate -> canonical
                let rel = Relationship::new(
                    section.section_id.clone(),
                    cluster.canonical_section_id.clone(),
                    RelationshipKind::DuplicateOf,
                )
                .with_description(format!(
                    "Exact duplicate of canonical section '{}'",
                    cluster.canonical_section_id
                ));
                new_relationships.push(rel);

                match self.config.strategy {
                    DeduplicationStrategy::Remove => {
                        // Section is completely omitted from new_sections
                        if self.config.log_exclusions {
                            let exclusion = ExclusionRecord::new(
                                original_path,
                                ExclusionReason::Duplicate,
                                None,
                                Some(format!(
                                    "Exact duplicate of canonical section '{}' ({})",
                                    cluster.canonical_section_id,
                                    cluster.display_label()
                                )),
                                Some(section.body.len() as u64),
                            );
                            new_exclusions.push(exclusion);
                        }
                    }
                    DeduplicationStrategy::ReplaceWithPointer => {
                        let pointer_body = format!(
                            "<!-- [Duplicate omitted: identical to `{}`] -->",
                            cluster.canonical_section_id
                        );
                        let mut replaced_sec = section.clone();
                        replaced_sec.content_hash = CdmSection::compute_hash(&pointer_body);
                        replaced_sec.token_metrics =
                            Some(SectionTokenMetrics::compute_baseline(&pointer_body));
                        replaced_sec.body = pointer_body;
                        new_sections.push(replaced_sec);

                        if self.config.log_exclusions {
                            let exclusion = ExclusionRecord::new(
                                original_path,
                                ExclusionReason::Duplicate,
                                None,
                                Some(format!(
                                    "Duplicate content body replaced with pointer to canonical section '{}'",
                                    cluster.canonical_section_id
                                )),
                                Some(section.body.len() as u64),
                            );
                            new_exclusions.push(exclusion);
                        }
                    }
                    DeduplicationStrategy::MarkOnly => {
                        // Body preserved intact
                        new_sections.push(section.clone());
                    }
                }
            } else {
                // Canonical or unique section
                new_sections.push(section.clone());
            }
        }

        // 4. Calculate metrics after deduplication
        let bytes_after: usize = new_sections.iter().map(|s| s.body.len()).sum();
        let tokens_after: usize = new_sections
            .iter()
            .map(|s| {
                s.token_metrics
                    .as_ref()
                    .map(|m| m.estimated_tokens)
                    .unwrap_or_else(|| s.body.chars().count().div_ceil(4))
            })
            .sum();

        let report = ExactDeduplicationReport::new(
            doc.document_id,
            total_sections,
            bytes_before,
            bytes_after,
            tokens_before,
            tokens_after,
            clusters,
        );

        // 5. Build updated CdmDocument
        let mut new_doc = doc.clone();
        new_doc.sections = new_sections;
        new_doc.exclusions.extend(new_exclusions);
        new_doc.relationships.extend(new_relationships);

        // 6. Recalculate revision BLAKE3 hash if sections were modified or removed
        if report.duplicate_sections_found > 0
            && self.config.strategy != DeduplicationStrategy::MarkOnly
        {
            let mut hasher = blake3::Hasher::new();
            for sec in &new_doc.sections {
                hasher.update(sec.content_hash.as_bytes());
            }
            new_doc.revision.content_hash_blake3 = Some(hasher.finalize().to_hex().to_string());
        }

        (new_doc, report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, LineRange,
        RevisionInfo, SectionKind, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn build_document_with_duplicates() -> CdmDocument {
        let code = "pub fn calculate() -> u64 {\n    let val = 42;\n    val * 2\n}\n";

        let sec1 = CdmSection {
            section_id: "sec-src".to_string(),
            parent_id: None,
            title: Some("calculate.rs".to_string()),
            kind: SectionKind::CodeFile,
            original_path: Some("src/calculate.rs".to_string()),
            source_url: "https://example.com/src/calculate.rs".to_string(),
            line_range: Some(LineRange::new(1, 10)),
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(code),
            body: code.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(code)),
            attributes: BTreeMap::new(),
        };

        let sec2 = CdmSection {
            section_id: "sec-fixture".to_string(),
            parent_id: None,
            title: Some("fixture.rs".to_string()),
            kind: SectionKind::CodeSnippet,
            original_path: Some("tests/fixtures/calculate.rs".to_string()),
            source_url: "https://example.com/tests/fixtures/calculate.rs".to_string(),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(code),
            body: code.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(code)),
            attributes: BTreeMap::new(),
        };

        CdmDocument {
            schema_version: "1.0.0".to_string(),
            document_id: DocumentId::new(),
            source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
            source_type: SourceType::GitHubPublic,
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
    fn test_document_deduplication_remove_strategy() {
        let doc = build_document_with_duplicates();
        let config = ExactDeduplicationConfig::aggressive();
        let deduplicator = DocumentDeduplicator::new(config);
        let (dedup_doc, report) = deduplicator.deduplicate_document(&doc);

        assert_eq!(report.duplicate_sections_found, 1);
        assert_eq!(dedup_doc.sections.len(), 1);
        assert_eq!(dedup_doc.sections[0].section_id, "sec-src");
        assert_eq!(dedup_doc.exclusions.len(), 1);
        assert_eq!(dedup_doc.exclusions[0].reason, ExclusionReason::Duplicate);
        assert_eq!(dedup_doc.relationships.len(), 1);
        assert_eq!(
            dedup_doc.relationships[0].kind,
            RelationshipKind::DuplicateOf
        );
        assert!(dedup_doc.verify_section_hashes());
    }

    #[test]
    fn test_document_deduplication_pointer_strategy() {
        let doc = build_document_with_duplicates();
        let config = ExactDeduplicationConfig::conservative();
        let deduplicator = DocumentDeduplicator::new(config);
        let (dedup_doc, report) = deduplicator.deduplicate_document(&doc);

        assert_eq!(report.duplicate_sections_found, 1);
        assert_eq!(dedup_doc.sections.len(), 2);
        assert_eq!(dedup_doc.sections[0].section_id, "sec-src");
        assert_eq!(dedup_doc.sections[1].section_id, "sec-fixture");
        assert!(dedup_doc.sections[1]
            .body
            .starts_with("<!-- [Duplicate omitted:"));
        assert!(dedup_doc.verify_section_hashes());
    }
}
