//! # Semantic Near-Duplicate Cluster Models
//!
//! Grouping structures capturing clusters of semantically equivalent sections,
//! pairwise similarity metrics, and redundancy savings.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::SectionKind;

/// An individual near-duplicate member within a semantic cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearDuplicateMember {
    /// Identifier of the duplicate section.
    pub section_id: String,
    /// Original file path or selector, if available.
    pub original_path: Option<String>,
    /// Structural kind of the section.
    pub kind: SectionKind,
    /// Estimated similarity coefficient $\in [0.0, 1.0]$ to the elected canonical section.
    pub similarity_to_canonical: f64,
    /// Byte length of this duplicate section.
    pub byte_count: usize,
    /// Estimated token volume of this duplicate section.
    pub estimated_tokens: usize,
}

/// A cluster of two or more semantically near-duplicate sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearDuplicateCluster {
    /// Unique cluster identifier (e.g. `"sem-cluster-1"`).
    pub cluster_id: String,
    /// Identifier of the elected canonical section.
    pub canonical_section_id: String,
    /// Optional title of the canonical section.
    pub canonical_title: Option<String>,
    /// Optional original file path of the canonical section.
    pub canonical_path: Option<String>,
    /// Structural kind of the canonical section.
    pub canonical_kind: SectionKind,
    /// All redundant near-duplicate member sections in this cluster.
    pub near_duplicates: Vec<NearDuplicateMember>,
    /// Mean similarity coefficient across all duplicates relative to canonical.
    pub average_similarity: f64,
    /// Total redundant byte volume contained in the near-duplicate members.
    pub total_duplicate_bytes: usize,
    /// Estimated tokens saved if these near-duplicates are consolidated.
    pub estimated_tokens_saved: usize,
}

impl NearDuplicateCluster {
    /// Total number of sections in this cluster (canonical + duplicates).
    #[must_use]
    pub fn total_instances(&self) -> usize {
        self.near_duplicates.len() + 1
    }

    /// Descriptive label for display in reports and logs.
    #[must_use]
    pub fn display_label(&self) -> String {
        if let Some(ref path) = self.canonical_path {
            format!("{} (`{}`)", self.canonical_section_id, path)
        } else if let Some(ref title) = self.canonical_title {
            format!("{} (\"{}\")", self.canonical_section_id, title)
        } else {
            self.canonical_section_id.clone()
        }
    }
}
