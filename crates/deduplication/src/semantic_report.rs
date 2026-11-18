//! # Semantic Deduplication Telemetry & Reporting
//!
//! Structured reports, compression statistics, and Markdown summaries detailing
//! near-duplicate content consolidation across a CDM document.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::DocumentId;

use crate::semantic_cluster::NearDuplicateCluster;

/// Comprehensive telemetry report detailing semantic near-duplicate consolidation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDeduplicationReport {
    /// Document identifier of the processed document.
    pub document_id: DocumentId,
    /// Total number of sections evaluated.
    pub total_sections_evaluated: usize,
    /// Number of unique content sections remaining.
    pub unique_sections: usize,
    /// Number of near-duplicate section instances identified.
    pub near_duplicate_sections_found: usize,
    /// Number of distinct near-duplicate clusters discovered.
    pub near_duplicate_clusters_found: usize,
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
    /// Percentage compression achieved.
    pub reduction_percentage: f64,
    /// Detailed cluster breakdowns.
    pub clusters: Vec<NearDuplicateCluster>,
}

impl SemanticDeduplicationReport {
    /// Constructs a new report from aggregate metrics and cluster records.
    #[must_use]
    pub fn new(
        document_id: DocumentId,
        total_sections: usize,
        bytes_before: usize,
        bytes_after: usize,
        tokens_before: usize,
        tokens_after: usize,
        clusters: Vec<NearDuplicateCluster>,
    ) -> Self {
        let near_duplicate_sections_found: usize =
            clusters.iter().map(|c| c.near_duplicates.len()).sum();
        let near_duplicate_clusters_found = clusters.len();
        let unique_sections = total_sections.saturating_sub(near_duplicate_sections_found);

        let byte_reduction = bytes_before.saturating_sub(bytes_after);
        let token_reduction = tokens_before.saturating_sub(tokens_after);

        let reduction_percentage = if bytes_before > 0 {
            let raw = (byte_reduction as f64 / bytes_before as f64) * 100.0;
            (raw * 10000.0).round() / 10000.0
        } else {
            0.0
        };

        Self {
            document_id,
            total_sections_evaluated: total_sections,
            unique_sections,
            near_duplicate_sections_found,
            near_duplicate_clusters_found,
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
        md.push_str("### Semantic Near-Duplicate Deduplication Summary\n\n");
        md.push_str(&format!("- **Document ID:** `{}`\n", self.document_id));
        md.push_str(&format!(
            "- **Sections Evaluated:** {} (Unique: {}, Near-Duplicates: {})\n",
            self.total_sections_evaluated, self.unique_sections, self.near_duplicate_sections_found
        ));
        md.push_str(&format!(
            "- **Near-Duplicate Clusters:** {}\n",
            self.near_duplicate_clusters_found
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
            md.push_str("| Canonical Section | Kind | Near-Duplicates | Avg Similarity | Saved Bytes | Saved Tokens |\n");
            md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
            for c in &self.clusters {
                md.push_str(&format!(
                    "| {} | `{}` | {} | {:.1}% | {} | {} |\n",
                    c.display_label(),
                    c.canonical_kind.as_str(),
                    c.near_duplicates.len(),
                    c.average_similarity * 100.0,
                    c.total_duplicate_bytes,
                    c.estimated_tokens_saved,
                ));
            }
        } else {
            md.push_str(
                "*Zero near-duplicate clusters found. All content sections are distinct.*\n",
            );
        }

        md
    }
}
