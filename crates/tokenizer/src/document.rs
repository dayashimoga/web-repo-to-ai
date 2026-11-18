//! # Document & Section Token Enrichment
//!
//! Evaluates and updates [`SectionTokenMetrics`] across CDM documents using
//! chosen tokenizer adapters, computing document-level token budgets and summaries.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionTokenMetrics};
use web_repo_to_ai_core::DocumentId;

use crate::adapter::TokenizerAdapter;
use crate::precision::TokenPrecision;

/// Whole-document token utilization and telemetry summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTokenSummary {
    /// Document identifier.
    pub document_id: DocumentId,
    /// Canonical tokenizer adapter name.
    pub tokenizer_name: String,
    /// Precision tier of the token counts.
    pub precision: TokenPrecision,
    /// Total number of sections evaluated.
    pub total_sections: usize,
    /// Total token count across all sections.
    pub total_tokens: usize,
    /// Total character count across all sections.
    pub total_characters: usize,
    /// Total word count across all sections.
    pub total_words: usize,
    /// Empirical characters-per-token ratio.
    pub average_chars_per_token: f64,
    /// Maximum context window limit of the target model.
    pub context_window_limit: usize,
    /// Budget utilization percentage of the context window.
    pub budget_utilization_pct: f64,
    /// True if total tokens exceed the target model's maximum context window.
    pub exceeds_context_window: bool,
}

impl DocumentTokenSummary {
    /// Renders a human-readable GitHub Flavored Markdown summary table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("### Tokenization & Context Budget Summary\n\n");
        md.push_str(&format!("- **Document ID:** `{}`\n", self.document_id));
        md.push_str(&format!(
            "- **Tokenizer:** `{}` (Precision: `{:?}`)\n",
            self.tokenizer_name, self.precision
        ));
        md.push_str(&format!(
            "- **Total Tokens:** {} (Characters: {}, Words: {})\n",
            self.total_tokens, self.total_characters, self.total_words
        ));
        md.push_str(&format!(
            "- **Average Chars / Token:** `{:.2}`\n",
            self.average_chars_per_token
        ));
        md.push_str(&format!(
            "- **Target Context Window:** {} tokens (Budget Utilization: `{:.2}%`)\n",
            self.context_window_limit, self.budget_utilization_pct
        ));
        if self.exceeds_context_window {
            md.push_str(
                "> [!WARNING]\n> **Document exceeds target model context window limit!** Context budget truncation or profile optimization required.\n\n",
            );
        } else {
            md.push_str(
                "> [!NOTE]\n> Document fits safely within target model context window.\n\n",
            );
        }

        md
    }
}

/// Evaluates token metrics for an individual [`CdmSection`] using the given adapter.
#[must_use]
pub fn count_section_tokens(
    section: &CdmSection,
    adapter: &dyn TokenizerAdapter,
) -> SectionTokenMetrics {
    let result = adapter.count_tokens(&section.body);

    if result.precision.is_exact() {
        SectionTokenMetrics {
            word_count: result.word_count,
            char_count: result.character_count,
            line_count: result.line_count,
            estimated_tokens: result.count,
            exact_tokens: Some(result.count),
            tokenizer_name: Some(result.tokenizer_name.to_string()),
        }
    } else {
        SectionTokenMetrics {
            word_count: result.word_count,
            char_count: result.character_count,
            line_count: result.line_count,
            estimated_tokens: result.count,
            exact_tokens: None,
            tokenizer_name: Some(result.tokenizer_name.to_string()),
        }
    }
}

/// Enriches all section token metrics across a [`CdmDocument`] and computes a [`DocumentTokenSummary`].
pub fn enrich_document_token_metrics(
    doc: &mut CdmDocument,
    adapter: &dyn TokenizerAdapter,
) -> DocumentTokenSummary {
    let total_sections = doc.sections.len();
    let mut total_tokens = 0;
    let mut total_characters = 0;
    let mut total_words = 0;

    for section in &mut doc.sections {
        let metrics = count_section_tokens(section, adapter);
        total_tokens += metrics.exact_tokens.unwrap_or(metrics.estimated_tokens);
        total_characters += metrics.char_count;
        total_words += metrics.word_count;
        section.token_metrics = Some(metrics);
    }

    let average_chars_per_token = if total_tokens > 0 {
        ((total_characters as f64 / total_tokens as f64) * 100.0).round() / 100.0
    } else {
        0.0
    };

    let context_window_limit = adapter.context_window_limit();
    let budget_utilization_pct = if context_window_limit > 0 {
        ((total_tokens as f64 / context_window_limit as f64 * 100.0) * 100.0).round() / 100.0
    } else {
        0.0
    };

    let exceeds_context_window = total_tokens > context_window_limit;

    let precision = if adapter.is_exact() {
        TokenPrecision::Exact
    } else {
        TokenPrecision::CalibratedEstimate {
            uncertainty_pct: 7.5,
        }
    };

    DocumentTokenSummary {
        document_id: doc.document_id,
        tokenizer_name: adapter.name().to_string(),
        precision,
        total_sections,
        total_tokens,
        total_characters,
        total_words,
        average_chars_per_token,
        context_window_limit,
        budget_utilization_pct,
        exceeds_context_window,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::OpenAiAdapter;
    use crate::model::TargetModel;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, LineRange,
        RevisionInfo, SectionKind, SourceIdentity, SourceType,
    };

    fn make_test_doc() -> CdmDocument {
        let s1 = CdmSection {
            section_id: "sec-1".to_string(),
            parent_id: None,
            title: Some("Section 1".to_string()),
            kind: SectionKind::Prose,
            original_path: Some("doc.md".to_string()),
            source_url: "https://example.com/doc.md".to_string(),
            line_range: Some(LineRange::new(1, 10)),
            byte_range: None,
            language: Some("markdown".to_string()),
            content_hash: CdmSection::compute_hash("Hello world tokenization test."),
            body: "Hello world tokenization test.".to_string(),
            token_metrics: None,
            attributes: BTreeMap::new(),
        };

        CdmDocument {
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
            sections: vec![s1],
            relationships: Vec::new(),
            citations: Vec::new(),
            extraction_status: ExtractionStatus::Completed,
            warnings: Vec::new(),
            exclusions: Vec::new(),
            completeness: CompletenessReport::default(),
        }
    }

    #[test]
    fn test_enrich_document_token_metrics() {
        let mut doc = make_test_doc();
        let adapter = OpenAiAdapter::for_model(TargetModel::Gpt4o);

        let summary = enrich_document_token_metrics(&mut doc, &adapter);
        assert_eq!(summary.total_sections, 1);
        assert!(summary.total_tokens > 0);
        assert!(summary.precision.is_exact());

        let sec_metrics = doc.sections[0].token_metrics.as_ref().unwrap();
        assert_eq!(sec_metrics.tokenizer_name.as_deref(), Some("o200k_base"));
        assert!(sec_metrics.exact_tokens.is_some());
    }
}
