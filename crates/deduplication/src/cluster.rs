//! # Duplicate Cluster Models
//!
//! Grouping structures capturing identical content clusters, elected canonical sections,
//! and redundancy metrics.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::SectionKind;

/// A cluster of two or more sections containing byte-identical or normalized-identical content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateCluster {
    /// Cryptographic BLAKE3 hash identifying the shared content payload.
    pub content_hash: String,
    /// Identifier of the elected canonical section.
    pub canonical_section_id: String,
    /// Optional title of the canonical section.
    pub canonical_title: Option<String>,
    /// Optional original file path of the canonical section.
    pub canonical_path: Option<String>,
    /// Structural kind of the canonical section.
    pub canonical_kind: SectionKind,
    /// Identifiers of all duplicate sections identified within this cluster.
    pub duplicate_section_ids: Vec<String>,
    /// Total count of redundant duplicate instances (excluding canonical).
    pub duplicate_count: usize,
    /// Byte length of an individual section payload in this cluster.
    pub bytes_per_instance: usize,
    /// Net redundant byte volume eliminated across all duplicates in this cluster.
    pub total_duplicate_bytes: usize,
    /// Estimated tokens saved across all eliminated duplicate instances.
    pub estimated_tokens_saved: usize,
}

impl DuplicateCluster {
    /// Total number of sections in this cluster (canonical + duplicates).
    #[must_use]
    pub fn total_instances(&self) -> usize {
        self.duplicate_count + 1
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
