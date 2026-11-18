//! # Document Normalization Report
//!
//! Quantitative metrics, audit ledgers, and telemetry reports summarizing the impact
//! of structural content normalization across a [`web_repo_to_ai_cdm::CdmDocument`].

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::DocumentId;

use crate::section::SectionNormalizationAudit;

/// Comprehensive report detailing volume reduction, section modifications,
/// and cryptographic hash transformations across an entire normalized document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizationReport {
    /// Document identifier of the processed document.
    pub document_id: DocumentId,
    /// Total number of sections evaluated.
    pub sections_total: usize,
    /// Number of sections that were modified by normalization.
    pub sections_modified: usize,
    /// Number of sections that were unchanged (already canonical).
    pub sections_unchanged: usize,
    /// Total bytes before normalization.
    pub bytes_before: usize,
    /// Total bytes after normalization.
    pub bytes_after: usize,
    /// Total characters before normalization.
    pub chars_before: usize,
    /// Total characters after normalization.
    pub chars_after: usize,
    /// Total lines before normalization.
    pub lines_before: usize,
    /// Total lines after normalization.
    pub lines_after: usize,
    /// Estimated tokens before normalization.
    pub estimated_tokens_before: usize,
    /// Estimated tokens after normalization.
    pub estimated_tokens_after: usize,
    /// Net byte reduction achieved.
    pub byte_reduction: usize,
    /// Net token reduction achieved.
    pub token_reduction: usize,
    /// Compression ratio achieved exclusively via structural cleanup (e.g. 0.08 for 8% reduction).
    pub reduction_percentage: f64,
    /// Audit records for each section processed.
    pub section_audits: Vec<SectionNormalizationAudit>,
}

impl NormalizationReport {
    /// Builds a new [`NormalizationReport`] by aggregating section audits.
    #[must_use]
    pub fn build(document_id: DocumentId, section_audits: Vec<SectionNormalizationAudit>) -> Self {
        let sections_total = section_audits.len();
        let sections_modified = section_audits.iter().filter(|s| s.modified).count();
        let sections_unchanged = sections_total.saturating_sub(sections_modified);

        let bytes_before: usize = section_audits.iter().map(|s| s.bytes_before).sum();
        let bytes_after: usize = section_audits.iter().map(|s| s.bytes_after).sum();
        let lines_before: usize = section_audits.iter().map(|s| s.lines_before).sum();
        let lines_after: usize = section_audits.iter().map(|s| s.lines_after).sum();
        let estimated_tokens_before: usize = section_audits.iter().map(|s| s.tokens_before).sum();
        let estimated_tokens_after: usize = section_audits.iter().map(|s| s.tokens_after).sum();

        let byte_reduction = bytes_before.saturating_sub(bytes_after);
        let token_reduction = estimated_tokens_before.saturating_sub(estimated_tokens_after);

        let reduction_percentage = if bytes_before > 0 {
            (byte_reduction as f64 / bytes_before as f64) * 100.0
        } else {
            0.0
        };

        Self {
            document_id,
            sections_total,
            sections_modified,
            sections_unchanged,
            bytes_before,
            bytes_after,
            chars_before: bytes_before, // approximation when chars not individually logged
            chars_after: bytes_after,
            lines_before,
            lines_after,
            estimated_tokens_before,
            estimated_tokens_after,
            byte_reduction,
            token_reduction,
            reduction_percentage,
            section_audits,
        }
    }

    /// Renders a concise GitHub-Flavored Markdown summary report.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("### Structural Normalization Summary\n\n");
        md.push_str(&format!("- **Document ID:** `{}`\n", self.document_id));
        md.push_str(&format!(
            "- **Sections Processed:** {} (Modified: {}, Unchanged: {})\n",
            self.sections_total, self.sections_modified, self.sections_unchanged
        ));
        md.push_str(&format!(
            "- **Bytes:** {} → {} (Reduced by {} bytes, {:.2}%)\n",
            self.bytes_before, self.bytes_after, self.byte_reduction, self.reduction_percentage
        ));
        md.push_str(&format!(
            "- **Estimated Tokens:** {} → {} (Saved {} tokens)\n",
            self.estimated_tokens_before, self.estimated_tokens_after, self.token_reduction
        ));
        md.push_str(&format!(
            "- **Lines:** {} → {}\n\n",
            self.lines_before, self.lines_after
        ));

        if !self.section_audits.is_empty() {
            md.push_str("| Section ID | Kind | Bytes (Before → After) | Tokens (Before → After) | Status |\n");
            md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

            for audit in &self.section_audits {
                let status_badge = if audit.modified {
                    "`NORMALIZED`"
                } else {
                    "`CANONICAL`"
                };
                md.push_str(&format!(
                    "| `{}` | `{}` | {} → {} | {} → {} | {} |\n",
                    audit.section_id,
                    audit.kind.as_str(),
                    audit.bytes_before,
                    audit.bytes_after,
                    audit.tokens_before,
                    audit.tokens_after,
                    status_badge
                ));
            }
        }

        md
    }

    /// Serializes the report to pretty-printed JSON.
    ///
    /// # Errors
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
