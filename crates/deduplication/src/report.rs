//! # Exact Deduplication Telemetry & Reporting
//!
//! Structured reports, compression statistics, and Markdown summaries detailing
//! identical content cluster elimination across a CDM document.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::DocumentId;

use crate::cluster::DuplicateCluster;

/// Comprehensive telemetry report detailing exact duplicate elimination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExactDeduplicationReport {
    /// Document identifier of the processed document.
    pub document_id: DocumentId,
    /// Total number of sections evaluated for duplication.
    pub total_sections_evaluated: usize,
    /// Number of unique content sections remaining.
    pub unique_sections: usize,
    /// Number of duplicate section instances identified and deduplicated.
    pub duplicate_sections_found: usize,
    /// Number of distinct duplicate clusters discovered.
    pub duplicate_clusters_found: usize,
    /// Total byte volume across evaluated sections prior to deduplication.
    pub bytes_before: usize,
    /// Total byte volume across evaluated sections after deduplication.
    pub bytes_after: usize,
    /// Net byte volume saved.
    pub byte_reduction: usize,
    /// Estimated token volume prior to deduplication.
    pub tokens_before: usize,
    /// Estimated token volume after deduplication.
    pub tokens_after: usize,
    /// Net tokens saved.
    pub token_reduction: usize,
    /// Percentage compression achieved via duplicate elimination.
    pub reduction_percentage: f64,
    /// Detailed cluster breakdowns.
    pub clusters: Vec<DuplicateCluster>,
}

impl ExactDeduplicationReport {
    /// Constructs a report from aggregate metrics and cluster records.
    #[must_use]
    pub fn new(
        document_id: DocumentId,
        total_sections: usize,
        bytes_before: usize,
        bytes_after: usize,
        tokens_before: usize,
        tokens_after: usize,
        clusters: Vec<DuplicateCluster>,
    ) -> Self {
        let duplicate_sections_found: usize = clusters.iter().map(|c| c.duplicate_count).sum();
        let duplicate_clusters_found = clusters.len();
        let unique_sections = total_sections.saturating_sub(duplicate_sections_found);

        let byte_reduction = bytes_before.saturating_sub(bytes_after);
        let token_reduction = tokens_before.saturating_sub(tokens_after);

        let reduction_percentage = if bytes_before > 0 {
            (byte_reduction as f64 / bytes_before as f64) * 100.0
        } else {
            0.0
        };

        Self {
            document_id,
            total_sections_evaluated: total_sections,
            unique_sections,
            duplicate_sections_found,
            duplicate_clusters_found,
            bytes_before,
            bytes_after,
            byte_reduction,
            tokens_before,
            tokens_after,
            token_reduction,
            reduction_percentage,
            clusters,
        }
    }

    /// Renders a concise GitHub-Flavored Markdown summary report.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("### Exact Content Deduplication Summary\n\n");
        md.push_str(&format!("- **Document ID:** `{}`\n", self.document_id));
        md.push_str(&format!(
            "- **Sections Evaluated:** {} (Unique: {}, Duplicates: {})\n",
            self.total_sections_evaluated, self.unique_sections, self.duplicate_sections_found
        ));
        md.push_str(&format!(
            "- **Duplicate Clusters:** {}\n",
            self.duplicate_clusters_found
        ));
        md.push_str(&format!(
            "- **Byte Reduction:** {} bytes -> {} bytes ({:.2}% reduction, {} bytes saved)\n",
            self.bytes_before, self.bytes_after, self.reduction_percentage, self.byte_reduction
        ));
        md.push_str(&format!(
            "- **Estimated Token Reduction:** {} tokens -> {} tokens ({} tokens saved)\n\n",
            self.tokens_before, self.tokens_after, self.token_reduction
        ));

        if !self.clusters.is_empty() {
            md.push_str(
                "| Canonical Section | Kind | Duplicates | Saved Bytes | Saved Tokens | Hash |\n",
            );
            md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
            for c in &self.clusters {
                let short_hash = if c.content_hash.len() > 12 {
                    &c.content_hash[..12]
                } else {
                    &c.content_hash
                };
                md.push_str(&format!(
                    "| {} | `{}` | {} | {} | {} | `{}` |\n",
                    c.display_label(),
                    c.canonical_kind.as_str(),
                    c.duplicate_count,
                    c.total_duplicate_bytes,
                    c.estimated_tokens_saved,
                    short_hash
                ));
            }
        } else {
            md.push_str(
                "*Zero exact duplicate sections found. All content sections are unique.*\n",
            );
        }

        md
    }
}
