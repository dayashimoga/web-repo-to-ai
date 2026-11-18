//! # Exact Content Deduplicator Engine
//!
//! Performs cryptographic hash grouping, canonical instance election, and duplicate
//! cluster formulation across CDM sections.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{CdmSection, SectionKind};

use crate::cluster::DuplicateCluster;
use crate::config::{CanonicalSelectionPolicy, ExactDeduplicationConfig, HashScope};

/// Computes the cryptographic comparison hash according to the selected [`HashScope`].
#[must_use]
pub fn compute_dedup_hash(body: &str, scope: HashScope) -> String {
    match scope {
        HashScope::ExactBytes => blake3::hash(body.as_bytes()).to_hex().to_string(),
        HashScope::Normalized => {
            let normalized = normalize_for_comparison(body);
            blake3::hash(normalized.as_bytes()).to_hex().to_string()
        }
    }
}

/// Normalizes text for comparison by canonicalizing line endings and whitespace.
fn normalize_for_comparison(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    // 1. CRLF/CR -> LF
    let mut lf_canon = String::with_capacity(text.len());
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            lf_canon.push('\n');
        } else {
            lf_canon.push(c);
        }
    }

    // 2. Trim trailing whitespace per line and trim outer blank lines
    let lines: Vec<&str> = lf_canon
        .lines()
        .map(|l| l.trim_end_matches([' ', '\t']))
        .collect();

    let mut start = 0;
    while start < lines.len() && lines[start].trim().is_empty() {
        start += 1;
    }
    let mut end = lines.len();
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }

    if start < end {
        for (i, line) in lines[start..end].iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(line);
        }
    }

    out
}

/// Internal indexed reference to a section.
#[derive(Clone, Copy)]
struct IndexedSection<'a> {
    index: usize,
    section: &'a CdmSection,
}

/// Core engine for finding duplicate clusters across [`CdmSection`] collections.
pub struct ExactDeduplicator;

impl ExactDeduplicator {
    /// Discovers all duplicate clusters across a slice of [`CdmSection`] items.
    #[must_use]
    pub fn find_duplicate_clusters(
        sections: &[CdmSection],
        config: &ExactDeduplicationConfig,
    ) -> Vec<DuplicateCluster> {
        // 1. Group candidate sections by hash
        let mut hash_groups: BTreeMap<String, Vec<IndexedSection<'_>>> = BTreeMap::new();

        for (index, section) in sections.iter().enumerate() {
            if section.body.len() < config.min_byte_threshold {
                continue;
            }

            let hash = compute_dedup_hash(&section.body, config.hash_scope);
            hash_groups
                .entry(hash)
                .or_default()
                .push(IndexedSection { index, section });
        }

        // 2. Filter groups with duplicate occurrences (> 1 instance)
        let mut clusters = Vec::new();

        for (hash, group) in hash_groups {
            if group.len() <= 1 {
                continue;
            }

            // 3. Elect canonical section using policy
            let canonical_idx_in_group = elect_canonical_index(&group, &config.canonical_policy);
            let canonical = group[canonical_idx_in_group].section;

            let mut duplicate_section_ids = Vec::with_capacity(group.len() - 1);
            let mut total_duplicate_bytes = 0;
            let mut estimated_tokens_saved = 0;

            for (i, item) in group.iter().enumerate() {
                if i != canonical_idx_in_group {
                    duplicate_section_ids.push(item.section.section_id.clone());
                    total_duplicate_bytes += item.section.body.len();
                    let tokens = item
                        .section
                        .token_metrics
                        .as_ref()
                        .map(|m| m.estimated_tokens)
                        .unwrap_or_else(|| item.section.body.chars().count().div_ceil(4));
                    estimated_tokens_saved += tokens;
                }
            }

            clusters.push(DuplicateCluster {
                content_hash: hash,
                canonical_section_id: canonical.section_id.clone(),
                canonical_title: canonical.title.clone(),
                canonical_path: canonical.original_path.clone(),
                canonical_kind: canonical.kind,
                duplicate_count: duplicate_section_ids.len(),
                duplicate_section_ids,
                bytes_per_instance: canonical.body.len(),
                total_duplicate_bytes,
                estimated_tokens_saved,
            });
        }

        // Sort clusters by net bytes saved descending for deterministic reporting
        clusters.sort_by_key(|b| std::cmp::Reverse(b.total_duplicate_bytes));

        clusters
    }
}

/// Elects the index within the group that is designated canonical.
fn elect_canonical_index(group: &[IndexedSection<'_>], policy: &CanonicalSelectionPolicy) -> usize {
    if group.is_empty() {
        return 0;
    }

    match policy {
        CanonicalSelectionPolicy::FirstSeen => group
            .iter()
            .enumerate()
            .min_by_key(|(_, item)| item.index)
            .map(|(idx, _)| idx)
            .unwrap_or(0),
        CanonicalSelectionPolicy::ShortestPath => group
            .iter()
            .enumerate()
            .min_by_key(|(_, item)| {
                let path_len = item
                    .section
                    .original_path
                    .as_ref()
                    .map(|p| p.len())
                    .unwrap_or(item.section.section_id.len());
                (path_len, item.index)
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0),
        CanonicalSelectionPolicy::StructuralHierarchy => {
            group
                .iter()
                .enumerate()
                .max_by_key(|(_, item)| {
                    let rank = kind_structural_rank(item.section.kind);
                    let path_len = item
                        .section
                        .original_path
                        .as_ref()
                        .map(|p| p.len())
                        .unwrap_or(item.section.section_id.len());
                    // Higher rank is better; shorter path is better; lower index is better
                    (
                        rank,
                        std::cmp::Reverse(path_len),
                        std::cmp::Reverse(item.index),
                    )
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        }
        CanonicalSelectionPolicy::PreferredPrefix(prefix) => group
            .iter()
            .enumerate()
            .max_by_key(|(_, item)| {
                let matches_prefix = item
                    .section
                    .original_path
                    .as_ref()
                    .map(|p| p.starts_with(prefix))
                    .unwrap_or(false);
                let rank = kind_structural_rank(item.section.kind);
                (matches_prefix, rank, std::cmp::Reverse(item.index))
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0),
    }
}

/// Assigns numerical priority to section kinds: higher number = higher canonical priority.
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
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::SectionTokenMetrics;

    fn make_test_section(id: &str, path: &str, kind: SectionKind, body: &str) -> CdmSection {
        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(id.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!("https://example.com/{path}"),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(body)),
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn test_duplicate_cluster_discovery_and_canonical_election() {
        let code = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let sec1 = make_test_section(
            "sec-snippet",
            "docs/example.md",
            SectionKind::CodeSnippet,
            code,
        );
        let sec2 = make_test_section("sec-file", "src/math.rs", SectionKind::CodeFile, code);

        let config = ExactDeduplicationConfig::conservative();
        let clusters = ExactDeduplicator::find_duplicate_clusters(&[sec1, sec2], &config);

        assert_eq!(clusters.len(), 1);
        let cluster = &clusters[0];
        // By structural hierarchy, CodeFile (sec2) must be elected canonical over CodeSnippet (sec1)
        assert_eq!(cluster.canonical_section_id, "sec-file");
        assert_eq!(cluster.duplicate_section_ids, vec!["sec-snippet"]);
        assert_eq!(cluster.duplicate_count, 1);
        assert!(cluster.total_duplicate_bytes > 0);
    }
}
