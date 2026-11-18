//! # Cross-Model Token Counting & Efficiency Comparison
//!
//! Evaluates and contrasts token counts, context window utilization, and compression ratios
//! for the same text or CDM document across multiple target model architectures.

use crate::model::TargetModel;
use crate::tokenizers::{
    Cl100kTokenizer, ClaudeTokenizer, GeminiTokenizer, LlamaTokenizer, MistralTokenizer,
    O200kTokenizer,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::CdmDocument;

/// Comparative metrics for a single target model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelComparisonEntry {
    /// Target model evaluated.
    pub model: TargetModel,
    /// Canonical tokenizer identifier.
    pub tokenizer_name: String,
    /// Exact token count.
    pub token_count: usize,
    /// Total character count.
    pub char_count: usize,
    /// Empirical characters-per-token ratio.
    pub chars_per_token: f64,
    /// Maximum context window limit of the model.
    pub context_window_limit: usize,
    /// Context window utilization percentage.
    pub context_utilization_pct: f64,
    /// Efficiency ratio relative to the baseline model (e.g. 0.85 = 15% fewer tokens).
    pub relative_ratio_vs_baseline: f64,
}

/// Comprehensive multi-model token comparison report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelTokenComparison {
    /// Baseline target model (typically `TargetModel::Gpt4Turbo` / `cl100k_base`).
    pub baseline_model: TargetModel,
    /// Individual model evaluation entries.
    pub entries: Vec<ModelComparisonEntry>,
    /// Model that produced the smallest token count.
    pub most_compact_model: TargetModel,
    /// Model that produced the largest token count.
    pub least_compact_model: TargetModel,
    /// Percentage variance between the most and least compact models.
    pub max_token_variance_pct: f64,
}

impl ModelTokenComparison {
    /// Compares token counts for `text` across the specified target models (or standard defaults if empty).
    #[must_use]
    pub fn compare_text(text: &str, models: &[TargetModel]) -> Self {
        let models_to_eval = if models.is_empty() {
            TargetModel::supported_models()
        } else {
            models
        };

        let baseline_model = TargetModel::Gpt4Turbo;
        let char_count = text.chars().count();

        let mut token_counts: BTreeMap<TargetModel, usize> = BTreeMap::new();
        for &model in models_to_eval {
            let count = match model {
                TargetModel::Gpt4o | TargetModel::Gpt4oMini => O200kTokenizer::count(text),
                TargetModel::Gpt4Turbo => Cl100kTokenizer::count(text),
                TargetModel::Claude35Sonnet
                | TargetModel::Claude3Opus
                | TargetModel::Claude3Haiku => ClaudeTokenizer::count(text),
                TargetModel::Gemini15Pro | TargetModel::Gemini15Flash => {
                    GeminiTokenizer::count(text)
                }
                TargetModel::Llama3 | TargetModel::Llama370B => LlamaTokenizer::count(text),
                TargetModel::MistralLarge | TargetModel::Mixtral8x7B => {
                    MistralTokenizer::count(text)
                }
                TargetModel::GenericFallback => Cl100kTokenizer::count(text),
            };
            token_counts.insert(model, count);
        }

        let baseline_count = token_counts
            .get(&baseline_model)
            .copied()
            .unwrap_or(1)
            .max(1);

        let mut entries = Vec::new();
        let mut min_tokens = usize::MAX;
        let mut max_tokens = 0;
        let mut most_compact = TargetModel::Gpt4o;
        let mut least_compact = TargetModel::Gpt4Turbo;

        for &model in models_to_eval {
            let count = token_counts.get(&model).copied().unwrap_or(0);
            if count < min_tokens {
                min_tokens = count;
                most_compact = model;
            }
            if count > max_tokens {
                max_tokens = count;
                least_compact = model;
            }

            let chars_per_token = if count > 0 {
                ((char_count as f64 / count as f64) * 100.0).round() / 100.0
            } else {
                0.0
            };

            let limit = model.context_window_limit();
            let util_pct = if limit > 0 {
                ((count as f64 / limit as f64 * 100.0) * 1000.0).round() / 1000.0
            } else {
                0.0
            };

            let rel_ratio = ((count as f64 / baseline_count as f64) * 100.0).round() / 100.0;

            entries.push(ModelComparisonEntry {
                model,
                tokenizer_name: model.default_tokenizer_name().to_string(),
                token_count: count,
                char_count,
                chars_per_token,
                context_window_limit: limit,
                context_utilization_pct: util_pct,
                relative_ratio_vs_baseline: rel_ratio,
            });
        }

        let max_token_variance_pct = if min_tokens > 0 && max_tokens >= min_tokens {
            (((max_tokens - min_tokens) as f64 / min_tokens as f64 * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };

        Self {
            baseline_model,
            entries,
            most_compact_model: most_compact,
            least_compact_model: least_compact,
            max_token_variance_pct,
        }
    }

    /// Compares token counts for all sections across a [`CdmDocument`].
    #[must_use]
    pub fn compare_document(doc: &CdmDocument, models: &[TargetModel]) -> Self {
        let concatenated_body: String = doc
            .sections
            .iter()
            .map(|s| s.body.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        Self::compare_text(&concatenated_body, models)
    }

    /// Renders a human-readable GitHub Flavored Markdown comparison table.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let mut md = String::new();
        md.push_str("### Cross-Model Token Intelligence & Efficiency Comparison\n\n");
        md.push_str("| Target Model | Tokenizer | Tokens | Chars/Token | Context Window | Utilization (%) | vs Baseline (`cl100k`) |\n");
        md.push_str("| :--- | :--- | :---: | :---: | :---: | :---: | :---: |\n");

        for entry in &self.entries {
            let marker = if entry.model == self.most_compact_model {
                " **(Most Compact)**"
            } else {
                ""
            };
            let diff_str = if entry.relative_ratio_vs_baseline < 1.0 {
                format!(
                    "-{}%",
                    ((1.0 - entry.relative_ratio_vs_baseline) * 100.0).round()
                )
            } else if entry.relative_ratio_vs_baseline > 1.0 {
                format!(
                    "+{}%",
                    ((entry.relative_ratio_vs_baseline - 1.0) * 100.0).round()
                )
            } else {
                "0.0% (Baseline)".to_string()
            };

            md.push_str(&format!(
                "| `{}`{} | `{}` | `{}` | `{}` | `{}` | `{}%` | `{}` |\n",
                entry.model.as_str(),
                marker,
                entry.tokenizer_name,
                entry.token_count,
                entry.chars_per_token,
                entry.context_window_limit,
                entry.context_utilization_pct,
                diff_str
            ));
        }

        md.push_str(&format!(
            "\n- **Most Compact Model:** `{}`\n- **Token Variance Across Architectures:** `{}%`\n",
            self.most_compact_model.as_str(),
            self.max_token_variance_pct
        ));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_model_comparison_text() {
        let text = "export async function processData(items: Array<Item>): Promise<Result> {\n    // Multi-lingual comment: 人工智能\n    return items.map(i => i.transform());\n}";
        let comp = ModelTokenComparison::compare_text(text, &[]);

        assert!(!comp.entries.is_empty());
        assert!(comp.entries.len() >= 5);
        assert!(comp.max_token_variance_pct >= 0.0);

        let md = comp.to_markdown_table();
        assert!(md.contains("Cross-Model Token Intelligence"));
        assert!(md.contains("Target Model"));
        assert!(md.contains("gpt-4o"));
        assert!(md.contains("claude-3-5-sonnet"));
    }
}
