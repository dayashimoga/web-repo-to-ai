//! # Content Classification Telemetry & Audit Reporting
//!
//! Provides structured summary reporting and Markdown/JSON serialization of
//! document classification metrics, density distributions, and low-signal candidate sections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_repo_to_ai_core::DocumentId;

use crate::classifier::{ContentCategory, InformationDensity};

/// Comprehensive report quantifying document content composition and signal quality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentClassificationReport {
    /// Document identifier.
    pub document_id: DocumentId,
    /// Timestamp when classification was executed.
    pub generated_at: DateTime<Utc>,
    /// Total number of sections evaluated.
    pub total_sections: usize,
    /// Total estimated tokens across all sections.
    pub total_tokens: usize,
    /// Total bytes across all sections.
    pub total_bytes: usize,
    /// Number of sections per [`ContentCategory`].
    pub category_counts: BTreeMap<ContentCategory, usize>,
    /// Token distribution per [`ContentCategory`].
    pub category_tokens: BTreeMap<ContentCategory, usize>,
    /// Section counts per [`InformationDensity`].
    pub density_counts: BTreeMap<InformationDensity, usize>,
    /// Overall document average signal score in $[0.0, 1.0]$.
    pub average_signal_score: f64,
    /// Number of automated test and fixture sections.
    pub test_section_count: usize,
    /// Number of machine-generated, boilerplate, or lockfile sections.
    pub generated_section_count: usize,
    /// Identifiers of sections falling below the low-signal threshold.
    pub low_signal_section_ids: Vec<String>,
}

impl ContentClassificationReport {
    /// Constructs a new [`ContentClassificationReport`].
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_id: DocumentId,
        total_sections: usize,
        total_tokens: usize,
        total_bytes: usize,
        category_counts: BTreeMap<ContentCategory, usize>,
        category_tokens: BTreeMap<ContentCategory, usize>,
        density_counts: BTreeMap<InformationDensity, usize>,
        average_signal_score: f64,
        test_section_count: usize,
        generated_section_count: usize,
        low_signal_section_ids: Vec<String>,
    ) -> Self {
        Self {
            document_id,
            generated_at: Utc::now(),
            total_sections,
            total_tokens,
            total_bytes,
            category_counts,
            category_tokens,
            density_counts,
            average_signal_score,
            test_section_count,
            generated_section_count,
            low_signal_section_ids,
        }
    }

    /// Renders a human-readable GitHub Flavored Markdown summary table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("### Content Classification Summary\n\n");
        md.push_str(&format!("- **Document ID:** `{}`\n", self.document_id));
        md.push_str(&format!(
            "- **Total Sections:** {} (Tokens: {}, Bytes: {})\n",
            self.total_sections, self.total_tokens, self.total_bytes
        ));
        md.push_str(&format!(
            "- **Average Signal Score:** `{:.4}`\n",
            self.average_signal_score
        ));
        md.push_str(&format!(
            "- **Test Sections:** {} | **Generated / Boilerplate Sections:** {}\n",
            self.test_section_count, self.generated_section_count
        ));
        md.push_str(&format!(
            "- **Low-Signal Sections Identified:** {}\n\n",
            self.low_signal_section_ids.len()
        ));

        md.push_str("#### Category Breakdown\n\n");
        md.push_str("| Category | Sections | Estimated Tokens | Token Share |\n");
        md.push_str("| :--- | :---: | :---: | :---: |\n");

        for (cat, count) in &self.category_counts {
            let tokens = self.category_tokens.get(cat).copied().unwrap_or(0);
            let share = if self.total_tokens > 0 {
                (tokens as f64 / self.total_tokens as f64) * 100.0
            } else {
                0.0
            };
            md.push_str(&format!(
                "| `{}` | {} | {} | {:.1}% |\n",
                cat.as_str(),
                count,
                tokens,
                share
            ));
        }

        md.push_str("\n#### Information Density Distribution\n\n");
        md.push_str("| Density Tier | Section Count | Section Share |\n");
        md.push_str("| :--- | :---: | :---: |\n");

        for (density, count) in &self.density_counts {
            let share = if self.total_sections > 0 {
                (*count as f64 / self.total_sections as f64) * 100.0
            } else {
                0.0
            };
            md.push_str(&format!(
                "| `{}` | {} | {:.1}% |\n",
                density.as_str(),
                count,
                share
            ));
        }

        if !self.low_signal_section_ids.is_empty() {
            md.push_str("\n#### Low-Signal Candidates for Optimization / Budget Exclusion\n\n");
            for id in &self.low_signal_section_ids {
                md.push_str(&format!("- `{id}`\n"));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_report_markdown_and_serialization() {
        let mut category_counts = BTreeMap::new();
        category_counts.insert(ContentCategory::SourceCode, 5);
        category_counts.insert(ContentCategory::Documentation, 3);

        let mut category_tokens = BTreeMap::new();
        category_tokens.insert(ContentCategory::SourceCode, 1200);
        category_tokens.insert(ContentCategory::Documentation, 600);

        let mut density_counts = BTreeMap::new();
        density_counts.insert(InformationDensity::High, 4);
        density_counts.insert(InformationDensity::Medium, 4);

        let report = ContentClassificationReport::new(
            DocumentId::new(),
            8,
            1800,
            7200,
            category_counts,
            category_tokens,
            density_counts,
            0.8250,
            0,
            0,
            vec!["low-sec-1".to_string()],
        );

        let md = report.to_markdown_summary();
        assert!(md.contains("### Content Classification Summary"));
        assert!(md.contains("| `source_code` | 5 | 1200 | 66.7% |"));
        assert!(md.contains("- `low-sec-1`"));

        let json = serde_json::to_string_pretty(&report).expect("valid json serialization");
        let deserialized: ContentClassificationReport =
            serde_json::from_str(&json).expect("valid json deserialization");
        assert_eq!(report, deserialized);
    }
}
