//! # Semantic Near-Duplicate Deduplication Engine
//!
//! Orchestrates MinHash and SimHash similarity analysis, connected component clustering,
//! canonical instance election, and near-duplicate consolidation across [`CdmDocument`] structures.

use std::collections::{BTreeMap, BTreeSet};
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, ExclusionReason, ExclusionRecord, Relationship, RelationshipKind,
    SectionKind, SectionTokenMetrics,
};

use crate::minhash::MinHashSignature;
use crate::semantic_cluster::{NearDuplicateCluster, NearDuplicateMember};
use crate::semantic_config::{
    SemanticAlgorithm, SemanticCanonicalPolicy, SemanticDeduplicationConfig,
    SemanticDeduplicationStrategy,
};
use crate::semantic_report::SemanticDeduplicationReport;
use crate::shingle::Shingler;
use crate::simhash::SimHash64;

/// Internal signature representation for similarity calculation.
enum SectionSignature {
    MinHash(MinHashSignature),
    SimHash(SimHash64),
}

/// Internal item representation for deduplication candidate matching.
struct CandidateSection<'a> {
    index: usize,
    section: &'a CdmSection,
    signature: SectionSignature,
}

/// Core engine for finding and consolidating near-duplicate sections.
#[derive(Debug, Clone)]
pub struct SemanticDeduplicator {
    config: SemanticDeduplicationConfig,
}

impl SemanticDeduplicator {
    /// Creates a new semantic deduplicator with the given configuration.
    #[must_use]
    pub fn new(config: SemanticDeduplicationConfig) -> Self {
        Self { config }
    }

    /// Creates a semantic deduplicator with default balanced settings.
    #[must_use]
    pub fn default_deduplicator() -> Self {
        Self::new(SemanticDeduplicationConfig::default())
    }

    /// Accesses the underlying configuration.
    #[must_use]
    pub fn config(&self) -> &SemanticDeduplicationConfig {
        &self.config
    }

    /// Identifies all near-duplicate clusters across a slice of [`CdmSection`] items.
    #[must_use]
    pub fn find_near_duplicate_clusters(
        &self,
        sections: &[CdmSection],
    ) -> Vec<NearDuplicateCluster> {
        // 1. Filter eligible candidates and extract signatures
        let mut candidates = Vec::with_capacity(sections.len());

        for (index, section) in sections.iter().enumerate() {
            let word_count = section.body.split_whitespace().count();
            if word_count < self.config.min_words_threshold {
                continue;
            }

            let sig = match self.config.algorithm {
                SemanticAlgorithm::MinHash {
                    shingle_size,
                    num_hashes,
                } => {
                    let shingles = Shingler::word_shingles(&section.body, shingle_size);
                    if shingles.is_empty() {
                        continue;
                    }
                    SectionSignature::MinHash(MinHashSignature::compute(&shingles, num_hashes))
                }
                SemanticAlgorithm::SimHash { .. } => {
                    SectionSignature::SimHash(SimHash64::compute(&section.body))
                }
            };

            candidates.push(CandidateSection {
                index,
                section,
                signature: sig,
            });
        }

        if candidates.len() < 2 {
            return Vec::new();
        }

        // 2. Pairwise similarity graph construction
        let mut adjacency: BTreeMap<usize, Vec<(usize, f64)>> = BTreeMap::new();

        for i in 0..candidates.len() {
            for j in (i + 1)..candidates.len() {
                let similarity = compute_similarity(
                    &candidates[i].signature,
                    &candidates[j].signature,
                    self.config.algorithm,
                );

                if similarity >= self.config.similarity_threshold {
                    adjacency.entry(i).or_default().push((j, similarity));
                    adjacency.entry(j).or_default().push((i, similarity));
                }
            }
        }

        if adjacency.is_empty() {
            return Vec::new();
        }

        // 3. Find connected components (clusters)
        let mut visited = BTreeSet::new();
        let mut raw_clusters = Vec::new();

        for &node in adjacency.keys() {
            if visited.contains(&node) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = vec![node];
            visited.insert(node);

            while let Some(curr) = queue.pop() {
                component.push(curr);
                if let Some(neighbors) = adjacency.get(&curr) {
                    for &(next, _) in neighbors {
                        if visited.insert(next) {
                            queue.push(next);
                        }
                    }
                }
            }

            if component.len() > 1 {
                raw_clusters.push(component);
            }
        }

        // 4. Elect canonical section and construct NearDuplicateCluster records
        let mut clusters = Vec::with_capacity(raw_clusters.len());

        for (cluster_idx, component) in raw_clusters.iter().enumerate() {
            let component_candidates: Vec<&CandidateSection<'_>> =
                component.iter().map(|&idx| &candidates[idx]).collect();

            let canonical_idx_in_comp =
                elect_canonical_candidate(&component_candidates, self.config.canonical_policy);
            let canonical = component_candidates[canonical_idx_in_comp];

            let mut near_duplicates = Vec::with_capacity(component_candidates.len() - 1);
            let mut total_duplicate_bytes = 0;
            let mut estimated_tokens_saved = 0;
            let mut similarity_sum = 0.0;

            for (i, &cand) in component_candidates.iter().enumerate() {
                if i != canonical_idx_in_comp {
                    let similarity = compute_similarity(
                        &canonical.signature,
                        &cand.signature,
                        self.config.algorithm,
                    );
                    similarity_sum += similarity;

                    let bytes = cand.section.body.len();
                    let tokens = cand
                        .section
                        .token_metrics
                        .as_ref()
                        .map(|m| m.estimated_tokens)
                        .unwrap_or_else(|| cand.section.body.chars().count().div_ceil(4));

                    total_duplicate_bytes += bytes;
                    estimated_tokens_saved += tokens;

                    near_duplicates.push(NearDuplicateMember {
                        section_id: cand.section.section_id.clone(),
                        original_path: cand.section.original_path.clone(),
                        kind: cand.section.kind,
                        similarity_to_canonical: similarity,
                        byte_count: bytes,
                        estimated_tokens: tokens,
                    });
                }
            }

            let avg_similarity = if !near_duplicates.is_empty() {
                similarity_sum / near_duplicates.len() as f64
            } else {
                1.0
            };

            clusters.push(NearDuplicateCluster {
                cluster_id: format!("sem-cluster-{}", cluster_idx + 1),
                canonical_section_id: canonical.section.section_id.clone(),
                canonical_title: canonical.section.title.clone(),
                canonical_path: canonical.section.original_path.clone(),
                canonical_kind: canonical.section.kind,
                near_duplicates,
                average_similarity: avg_similarity,
                total_duplicate_bytes,
                estimated_tokens_saved,
            });
        }

        // Sort by net duplicate bytes descending
        clusters.sort_by_key(|b| std::cmp::Reverse(b.total_duplicate_bytes));

        clusters
    }

    /// Consolidates near-duplicates across a full [`CdmDocument`].
    #[must_use]
    pub fn deduplicate_document(
        &self,
        doc: &CdmDocument,
    ) -> (CdmDocument, SemanticDeduplicationReport) {
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

        let clusters = self.find_near_duplicate_clusters(&doc.sections);

        if clusters.is_empty() {
            let report = SemanticDeduplicationReport::new(
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

        // Build mapping of duplicate section ID -> (cluster, member)
        let mut dup_map: BTreeMap<&str, (&NearDuplicateCluster, &NearDuplicateMember)> =
            BTreeMap::new();
        for cluster in &clusters {
            for member in &cluster.near_duplicates {
                dup_map.insert(member.section_id.as_str(), (cluster, member));
            }
        }

        let mut new_sections = Vec::with_capacity(doc.sections.len());
        let mut new_exclusions = Vec::new();
        let mut new_relationships = Vec::new();

        for section in &doc.sections {
            if let Some(&(cluster, member)) = dup_map.get(section.section_id.as_str()) {
                let original_path = section
                    .original_path
                    .clone()
                    .unwrap_or_else(|| section.section_id.clone());

                let rel = Relationship::new(
                    section.section_id.clone(),
                    cluster.canonical_section_id.clone(),
                    RelationshipKind::DuplicateOf,
                )
                .with_description(format!(
                    "Near-duplicate ({:.1}% similarity) of canonical section '{}'",
                    member.similarity_to_canonical * 100.0,
                    cluster.canonical_section_id
                ));
                new_relationships.push(rel);

                match self.config.strategy {
                    SemanticDeduplicationStrategy::Remove => {
                        if self.config.log_exclusions {
                            let exclusion = ExclusionRecord::new(
                                original_path,
                                ExclusionReason::Duplicate,
                                None,
                                Some(format!(
                                    "Near-duplicate ({:.1}% similarity) of canonical section '{}' ({})",
                                    member.similarity_to_canonical * 100.0,
                                    cluster.canonical_section_id,
                                    cluster.display_label()
                                )),
                                Some(section.body.len() as u64),
                            );
                            new_exclusions.push(exclusion);
                        }
                    }
                    SemanticDeduplicationStrategy::ReplaceWithAnnotatedPointer => {
                        let pointer_body = format!(
                            "<!-- [Near-duplicate omitted: {:.1}% similar to `{}` ({})] -->",
                            member.similarity_to_canonical * 100.0,
                            cluster.canonical_section_id,
                            cluster.display_label()
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
                                    "Near-duplicate body replaced with pointer to canonical section '{}'",
                                    cluster.canonical_section_id
                                )),
                                Some(section.body.len() as u64),
                            );
                            new_exclusions.push(exclusion);
                        }
                    }
                    SemanticDeduplicationStrategy::MarkOnly => {
                        let mut marked_sec = section.clone();
                        marked_sec.attributes.insert(
                            "near_duplicate_of".to_string(),
                            cluster.canonical_section_id.clone(),
                        );
                        marked_sec.attributes.insert(
                            "near_duplicate_similarity".to_string(),
                            format!("{:.3}", member.similarity_to_canonical),
                        );
                        new_sections.push(marked_sec);
                    }
                }
            } else {
                new_sections.push(section.clone());
            }
        }

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

        let report = SemanticDeduplicationReport::new(
            doc.document_id,
            total_sections,
            bytes_before,
            bytes_after,
            tokens_before,
            tokens_after,
            clusters,
        );

        let mut new_doc = doc.clone();
        new_doc.sections = new_sections;
        new_doc.exclusions.extend(new_exclusions);
        new_doc.relationships.extend(new_relationships);

        if report.near_duplicate_sections_found > 0
            && self.config.strategy != SemanticDeduplicationStrategy::MarkOnly
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

/// Evaluates similarity between two section signatures.
fn compute_similarity(
    sig_a: &SectionSignature,
    sig_b: &SectionSignature,
    algorithm: SemanticAlgorithm,
) -> f64 {
    match (sig_a, sig_b) {
        (SectionSignature::MinHash(a), SectionSignature::MinHash(b)) => a.jaccard_similarity(b),
        (SectionSignature::SimHash(a), SectionSignature::SimHash(b)) => {
            if let SemanticAlgorithm::SimHash {
                max_hamming_distance,
            } = algorithm
            {
                let dist = a.hamming_distance(*b);
                if dist <= max_hamming_distance {
                    a.similarity(*b)
                } else {
                    0.0
                }
            } else {
                a.similarity(*b)
            }
        }
        _ => 0.0,
    }
}

/// Elects the index of the canonical candidate in a component.
fn elect_canonical_candidate(
    candidates: &[&CandidateSection<'_>],
    policy: SemanticCanonicalPolicy,
) -> usize {
    if candidates.is_empty() {
        return 0;
    }

    match policy {
        SemanticCanonicalPolicy::InformationRichness => candidates
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| (c.section.body.len(), std::cmp::Reverse(c.index)))
            .map(|(idx, _)| idx)
            .unwrap_or(0),
        SemanticCanonicalPolicy::StructuralHierarchy => candidates
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| {
                (
                    kind_structural_rank(c.section.kind),
                    c.section.body.len(),
                    std::cmp::Reverse(c.index),
                )
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0),
        SemanticCanonicalPolicy::FirstSeen => candidates
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.index)
            .map(|(idx, _)| idx)
            .unwrap_or(0),
        SemanticCanonicalPolicy::ShortestPath => candidates
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| {
                let path_len = c
                    .section
                    .original_path
                    .as_ref()
                    .map(|p| p.len())
                    .unwrap_or(c.section.section_id.len());
                (path_len, c.index)
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0),
    }
}

fn kind_structural_rank(kind: SectionKind) -> u8 {
    match kind {
        SectionKind::CodeFile => 7,
        SectionKind::CodeSnippet => 6,
        SectionKind::Prose => 5,
        SectionKind::Table => 4,
        SectionKind::Admonition => 3,
        SectionKind::Frontmatter => 2,
        SectionKind::Manifest => 1,
        SectionKind::Navigation => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, LineRange,
        RevisionInfo, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn make_test_section(id: &str, body: &str) -> CdmSection {
        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(id.to_string()),
            kind: SectionKind::Prose,
            original_path: Some(format!("docs/{id}.md")),
            source_url: format!("https://example.com/docs/{id}.md"),
            line_range: Some(LineRange::new(1, 10)),
            byte_range: None,
            language: Some("markdown".to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(body)),
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn test_near_duplicate_cluster_detection() {
        let text1 = "Our system processes asynchronous message batches with bounded queues and adaptive concurrency scaling. It maintains stable latency under heavy load and prevents cascade failures.";
        let text2 = "Our system processes asynchronous message batches with bounded queues and adaptive concurrency scaling. It maintains stable latency under heavy load and prevents network failures.";

        let sec1 = make_test_section("sec-1", text1);
        let sec2 = make_test_section("sec-2", text2);

        let deduplicator = SemanticDeduplicator::default_deduplicator();
        let clusters = deduplicator.find_near_duplicate_clusters(&[sec1, sec2]);

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].near_duplicates.len(), 1);
        assert!(clusters[0].average_similarity >= 0.75);
    }

    #[test]
    fn test_semantic_deduplicate_document_end_to_end() {
        let text1 = "High-throughput asynchronous message broker designed for distributed data streams with fault tolerance. Features include partitioned queues, at-least-once delivery, and zero external runtime dependencies.";
        let text2 = "High-throughput asynchronous message broker designed for distributed data streams with fault tolerance. Features include partitioned queues, at-most-once delivery, and zero external runtime dependencies.";

        let sec1 = make_test_section("sec-1", text1);
        let sec2 = make_test_section("sec-2", text2);

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

        let deduplicator = SemanticDeduplicator::default_deduplicator();
        let (dedup_doc, report) = deduplicator.deduplicate_document(&doc);

        assert_eq!(report.near_duplicate_sections_found, 1);
        assert_eq!(dedup_doc.relationships.len(), 1);
        assert_eq!(
            dedup_doc.relationships[0].kind,
            RelationshipKind::DuplicateOf
        );
        assert!(dedup_doc.verify_section_hashes());
    }
}
