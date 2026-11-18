//! # Context Optimization Compression Reporting and Telemetry
//!
//! Provides comprehensive reporting on token, byte, character, and section reduction
//! across all optimization stages (normalization, exact deduplication, semantic deduplication,
//! profile filtering, and knapsack budget truncation). Calculates compression ratios,
//! space savings percentages, and multi-model financial API cost reductions.

use crate::error::ContextOptimizerError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_tokenizer::cost::CostBreakdown;
use web_repo_to_ai_tokenizer::{
    CalibratedHeuristicAdapter, TargetModel, TokenizerAdapter, TokenizerRegistry,
};

/// Stage or transformation phase in the context optimization pipeline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationStage {
    /// Structural normalization (whitespace collapse, CRLF to LF, markdown fence rectangularization).
    Normalization,
    /// Exact deduplication (elimination or aliasing of byte-identical content).
    ExactDeduplication,
    /// Semantic deduplication (near-duplicate cluster aggregation via MinHash/SimHash).
    SemanticDeduplication,
    /// Profile-based category filtering (suppression of cosmetic assets, tests, or auxiliary docs).
    ProfileFiltering,
    /// Knapsack token budget enforcement (boundary truncation and size-limit exclusions).
    BudgetKnapsack,
    /// Custom or external transformation stage.
    Custom(String),
}

impl OptimizationStage {
    /// Returns a human-readable title for markdown tables and telemetry.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Normalization => "Structural Normalization",
            Self::ExactDeduplication => "Exact Deduplication",
            Self::SemanticDeduplication => "Semantic Deduplication",
            Self::ProfileFiltering => "Profile Filtering",
            Self::BudgetKnapsack => "Budget Knapsack Truncation",
            Self::Custom(name) => name.as_str(),
        }
    }
}

impl fmt::Display for OptimizationStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detailed compression and conservation metrics for an individual optimization stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageCompressionMetrics {
    /// Stage identifier.
    pub stage: OptimizationStage,
    /// Token count entering this stage.
    pub input_tokens: usize,
    /// Token count exiting this stage.
    pub output_tokens: usize,
    /// Net tokens saved in this stage.
    pub token_savings: usize,
    /// Percentage of input tokens eliminated in this stage (0.0 - 100.0).
    pub token_reduction_pct: f64,
    /// Stage token compression ratio (`input_tokens / output_tokens`).
    pub token_compression_ratio: f64,
    /// UTF-8 byte count entering this stage.
    pub input_bytes: usize,
    /// UTF-8 byte count exiting this stage.
    pub output_bytes: usize,
    /// Net UTF-8 bytes saved in this stage.
    pub byte_savings: usize,
    /// Percentage of input bytes eliminated in this stage (0.0 - 100.0).
    pub byte_reduction_pct: f64,
    /// Stage byte compression ratio (`input_bytes / output_bytes`).
    pub byte_compression_ratio: f64,
    /// Section count entering this stage.
    pub input_sections: usize,
    /// Section count exiting this stage.
    pub output_sections: usize,
    /// Net sections removed or excluded in this stage.
    pub section_reduction: usize,
}

impl StageCompressionMetrics {
    /// Calculates stage compression metrics from input and output measurements.
    #[must_use]
    pub fn calculate(
        stage: OptimizationStage,
        input_tokens: usize,
        output_tokens: usize,
        input_bytes: usize,
        output_bytes: usize,
        input_sections: usize,
        output_sections: usize,
    ) -> Self {
        let token_savings = input_tokens.saturating_sub(output_tokens);
        let token_reduction_pct = if input_tokens > 0 {
            ((token_savings as f64 / input_tokens as f64 * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let token_compression_ratio = if output_tokens > 0 {
            ((input_tokens as f64 / output_tokens as f64) * 100.0).round() / 100.0
        } else if input_tokens > 0 {
            f64::INFINITY
        } else {
            1.0
        };

        let byte_savings = input_bytes.saturating_sub(output_bytes);
        let byte_reduction_pct = if input_bytes > 0 {
            ((byte_savings as f64 / input_bytes as f64 * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let byte_compression_ratio = if output_bytes > 0 {
            ((input_bytes as f64 / output_bytes as f64) * 100.0).round() / 100.0
        } else if input_bytes > 0 {
            f64::INFINITY
        } else {
            1.0
        };

        let section_reduction = input_sections.saturating_sub(output_sections);

        Self {
            stage,
            input_tokens,
            output_tokens,
            token_savings,
            token_reduction_pct,
            token_compression_ratio,
            input_bytes,
            output_bytes,
            byte_savings,
            byte_reduction_pct,
            byte_compression_ratio,
            input_sections,
            output_sections,
            section_reduction,
        }
    }

    /// Formats this stage metric as a GitHub Flavored Markdown table row.
    #[must_use]
    pub fn to_markdown_row(&self) -> String {
        let token_ratio_str = if self.token_compression_ratio.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}x", self.token_compression_ratio)
        };
        let byte_ratio_str = if self.byte_compression_ratio.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}x", self.byte_compression_ratio)
        };

        format!(
            "| {} | `{}` → `{}` | `-{} ({:.1}%)` | `{}` | `{}` → `{}` | `-{} ({:.1}%)` | `{}` | `{}` → `{}` |",
            self.stage,
            self.input_tokens,
            self.output_tokens,
            self.token_savings,
            self.token_reduction_pct,
            token_ratio_str,
            self.input_bytes,
            self.output_bytes,
            self.byte_savings,
            self.byte_reduction_pct,
            byte_ratio_str,
            self.input_sections,
            self.output_sections,
        )
    }
}

/// Cumulative end-to-end compression metrics across all optimization stages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverallCompressionMetrics {
    /// Original token count before optimization.
    pub original_tokens: usize,
    /// Final optimized token count.
    pub optimized_tokens: usize,
    /// Total net tokens saved.
    pub total_token_savings: usize,
    /// Overall token space savings percentage (0.0 - 100.0).
    pub token_reduction_pct: f64,
    /// Overall token compression ratio (e.g. 2.5x).
    pub token_compression_ratio: f64,
    /// Original UTF-8 byte count.
    pub original_bytes: usize,
    /// Final optimized UTF-8 byte count.
    pub optimized_bytes: usize,
    /// Total net bytes saved.
    pub total_byte_savings: usize,
    /// Overall byte space savings percentage (0.0 - 100.0).
    pub byte_reduction_pct: f64,
    /// Overall byte compression ratio (e.g. 2.1x).
    pub byte_compression_ratio: f64,
    /// Original character count.
    pub original_chars: usize,
    /// Final optimized character count.
    pub optimized_chars: usize,
    /// Total character count reduction.
    pub total_char_savings: usize,
    /// Original section count.
    pub original_sections: usize,
    /// Retained section count in final document.
    pub retained_sections: usize,
    /// Net sections excluded or dropped.
    pub total_sections_removed: usize,
}

impl OverallCompressionMetrics {
    /// Calculates cumulative compression metrics from measured document volumes.
    #[must_use]
    pub fn from_volumes(original: &DocumentVolume, optimized: &DocumentVolume) -> Self {
        Self::calculate(
            original.tokens,
            optimized.tokens,
            original.bytes,
            optimized.bytes,
            original.chars,
            optimized.chars,
            original.sections,
            optimized.sections,
        )
    }

    /// Calculates cumulative compression metrics.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn calculate(
        original_tokens: usize,
        optimized_tokens: usize,
        original_bytes: usize,
        optimized_bytes: usize,
        original_chars: usize,
        optimized_chars: usize,
        original_sections: usize,
        retained_sections: usize,
    ) -> Self {
        let total_token_savings = original_tokens.saturating_sub(optimized_tokens);
        let token_reduction_pct = if original_tokens > 0 {
            ((total_token_savings as f64 / original_tokens as f64 * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let token_compression_ratio = if optimized_tokens > 0 {
            ((original_tokens as f64 / optimized_tokens as f64) * 100.0).round() / 100.0
        } else if original_tokens > 0 {
            f64::INFINITY
        } else {
            1.0
        };

        let total_byte_savings = original_bytes.saturating_sub(optimized_bytes);
        let byte_reduction_pct = if original_bytes > 0 {
            ((total_byte_savings as f64 / original_bytes as f64 * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let byte_compression_ratio = if optimized_bytes > 0 {
            ((original_bytes as f64 / optimized_bytes as f64) * 100.0).round() / 100.0
        } else if original_bytes > 0 {
            f64::INFINITY
        } else {
            1.0
        };

        let total_char_savings = original_chars.saturating_sub(optimized_chars);
        let total_sections_removed = original_sections.saturating_sub(retained_sections);

        Self {
            original_tokens,
            optimized_tokens,
            total_token_savings,
            token_reduction_pct,
            token_compression_ratio,
            original_bytes,
            optimized_bytes,
            total_byte_savings,
            byte_reduction_pct,
            byte_compression_ratio,
            original_chars,
            optimized_chars,
            total_char_savings,
            original_sections,
            retained_sections,
            total_sections_removed,
        }
    }

    /// Renders cumulative compression metrics as a formatted GFM Markdown table.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let token_ratio_str = if self.token_compression_ratio.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}x", self.token_compression_ratio)
        };
        let byte_ratio_str = if self.byte_compression_ratio.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}x", self.byte_compression_ratio)
        };

        format!(
            "| Dimension | Original | Optimized | Net Savings | Reduction % | Ratio |\n\
             | :--- | :---: | :---: | :---: | :---: | :---: |\n\
             | **Tokens** | `{}` | `{}` | `{}` | `{:.1}%` | `{}` |\n\
             | **Bytes (UTF-8)** | `{}` | `{}` | `{}` | `{:.1}%` | `{}` |\n\
             | **Characters** | `{}` | `{}` | `{}` | `{:.1}%` | — |\n\
             | **Sections** | `{}` | `{}` | `{}` | `{:.1}%` | — |",
            self.original_tokens,
            self.optimized_tokens,
            self.total_token_savings,
            self.token_reduction_pct,
            token_ratio_str,
            self.original_bytes,
            self.optimized_bytes,
            self.total_byte_savings,
            self.byte_reduction_pct,
            byte_ratio_str,
            self.original_chars,
            self.optimized_chars,
            self.total_char_savings,
            if self.original_chars > 0 {
                ((self.total_char_savings as f64 / self.original_chars as f64 * 100.0) * 10.0)
                    .round()
                    / 10.0
            } else {
                0.0
            },
            self.original_sections,
            self.retained_sections,
            self.total_sections_removed,
            if self.original_sections > 0 {
                ((self.total_sections_removed as f64 / self.original_sections as f64 * 100.0)
                    * 10.0)
                    .round()
                    / 10.0
            } else {
                0.0
            },
        )
    }
}

/// Estimated API dollar cost savings for a specific LLM model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostSavingsMetrics {
    /// Target model.
    pub model: TargetModel,
    /// Estimated cost of unoptimized prompt (USD).
    pub unoptimized_cost_usd: f64,
    /// Estimated cost of optimized prompt (USD).
    pub optimized_cost_usd: f64,
    /// Net financial savings per prompt call (USD).
    pub cost_savings_usd: f64,
    /// Financial savings percentage (0.0 - 100.0).
    pub savings_pct: f64,
    /// Estimated prompt caching savings on optimized prompt (USD).
    pub cached_cost_savings_usd: Option<f64>,
    /// Estimated batch queue savings on optimized prompt (USD).
    pub batch_cost_savings_usd: f64,
    /// Whether model is self-hosted / $0.00.
    pub is_free_or_self_hosted: bool,
}

impl CostSavingsMetrics {
    /// Calculates financial cost savings between original and optimized token volumes.
    #[must_use]
    pub fn calculate(original_tokens: usize, optimized_tokens: usize, model: TargetModel) -> Self {
        let unoptimized = CostBreakdown::calculate(original_tokens, model, 0);
        let optimized = CostBreakdown::calculate(optimized_tokens, model, 0);

        let cost_savings_usd =
            ((unoptimized.standard_cost_usd - optimized.standard_cost_usd).max(0.0) * 100_000.0)
                .round()
                / 100_000.0;

        let savings_pct = if unoptimized.standard_cost_usd > 0.0 {
            ((cost_savings_usd / unoptimized.standard_cost_usd * 100.0) * 10.0).round() / 10.0
        } else {
            0.0
        };

        let cached_cost_savings_usd = optimized.cached_cost_usd.map(|cached_opt| {
            ((unoptimized.standard_cost_usd - cached_opt).max(0.0) * 100_000.0).round() / 100_000.0
        });

        let batch_cost_savings_usd =
            ((unoptimized.standard_cost_usd - optimized.batch_cost_usd).max(0.0) * 100_000.0)
                .round()
                / 100_000.0;

        Self {
            model,
            unoptimized_cost_usd: unoptimized.standard_cost_usd,
            optimized_cost_usd: optimized.standard_cost_usd,
            cost_savings_usd,
            savings_pct,
            cached_cost_savings_usd,
            batch_cost_savings_usd,
            is_free_or_self_hosted: unoptimized.is_free_or_self_hosted,
        }
    }

    /// Formats this model cost metric as a Markdown table row.
    #[must_use]
    pub fn to_markdown_row(&self) -> String {
        if self.is_free_or_self_hosted {
            format!(
                "| `{}` | `$0.00` | `$0.00` | `$0.00` | `0.0%` | *(Self-Hosted / Open-Weights)* |",
                self.model.as_str()
            )
        } else {
            let cached_str = match self.cached_cost_savings_usd {
                Some(cached_savings) => format!("`+${:.4}`", cached_savings),
                None => "—".to_string(),
            };

            format!(
                "| `{}` | `${:.4}` | `${:.4}` | `-${:.4}` | `{:.1}%` | {} |",
                self.model.as_str(),
                self.unoptimized_cost_usd,
                self.optimized_cost_usd,
                self.cost_savings_usd,
                self.savings_pct,
                cached_str,
            )
        }
    }
}

/// Cross-model financial cost savings comparison across major LLM architectures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiModelCostSavings {
    /// Cost savings metrics per model.
    pub models: Vec<CostSavingsMetrics>,
}

impl MultiModelCostSavings {
    /// Calculates cost savings across representative commercial and self-hosted models.
    #[must_use]
    pub fn calculate(original_tokens: usize, optimized_tokens: usize) -> Self {
        let key_models = [
            TargetModel::Gpt4o,
            TargetModel::Gpt4oMini,
            TargetModel::Claude35Sonnet,
            TargetModel::Claude3Opus,
            TargetModel::Claude3Haiku,
            TargetModel::Gemini15Pro,
            TargetModel::Gemini15Flash,
            TargetModel::Llama3,
        ];

        let models = key_models
            .into_iter()
            .map(|model| CostSavingsMetrics::calculate(original_tokens, optimized_tokens, model))
            .collect();

        Self { models }
    }

    /// Renders the multi-model financial cost table as GitHub Flavored Markdown.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "| Target Model | Unoptimized Cost | Optimized Cost | Net Savings | Savings % | With Prompt Cache |\n\
             | :--- | :---: | :---: | :---: | :---: | :---: |\n",
        );
        for m in &self.models {
            out.push_str(&m.to_markdown_row());
            out.push('\n');
        }
        out
    }
}

/// Comprehensive compression and token savings report for context optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressionReport {
    /// UTC timestamp of report generation.
    pub generated_at: DateTime<Utc>,
    /// Name or identifier of context profile applied (if any).
    pub profile_name: Option<String>,
    /// Target model (if evaluated for a specific architecture).
    pub target_model: Option<TargetModel>,
    /// Cumulative overall compression metrics.
    pub overall: OverallCompressionMetrics,
    /// Stage-by-stage progression breakdown.
    pub stages: Vec<StageCompressionMetrics>,
    /// Multi-model financial cost savings analysis.
    pub cost_savings: MultiModelCostSavings,
    /// Contextual notes, warnings, or optimization takeaways.
    pub notes: Vec<String>,
}

impl CompressionReport {
    /// Renders a full GitHub Flavored Markdown summary report card.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();

        out.push_str("## Context Optimization Compression Report\n\n");
        out.push_str(&format!(
            "**Generated At:** `{}`  \n**Profile:** `{}`  \n**Target Model:** `{}`  \n**Token Compression Ratio:** `{:.2}x` (`{:.1}%` reduction)\n\n",
            self.generated_at.to_rfc3339(),
            self.profile_name.as_deref().unwrap_or("None (Direct)"),
            self.target_model.map(|m| m.as_str()).unwrap_or("General / Heuristic"),
            self.overall.token_compression_ratio,
            self.overall.token_reduction_pct,
        ));

        out.push_str("### 1. Cumulative Optimization Summary\n\n");
        out.push_str(&self.overall.to_markdown_table());
        out.push_str("\n\n");

        if !self.stages.is_empty() {
            out.push_str("### 2. Stage-by-Stage Optimization Progression\n\n");
            out.push_str(
                "| Stage | Token Progression | Token Savings | Token Ratio | Byte Progression | Byte Savings | Byte Ratio | Sections |\n\
                 | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n",
            );
            for s in &self.stages {
                out.push_str(&s.to_markdown_row());
                out.push('\n');
            }
            out.push('\n');
        }

        out.push_str("### 3. Estimated Financial API Cost Savings (Per 1k Invocations)\n\n");
        out.push_str(&self.cost_savings.to_markdown_table());
        out.push('\n');

        if !self.notes.is_empty() {
            out.push_str("### 4. Optimization Notes\n\n");
            for note in &self.notes {
                out.push_str(&format!("- {}\n", note));
            }
            out.push('\n');
        }

        out
    }

    /// Serializes the report to a formatted JSON string.
    pub fn to_json(&self) -> Result<String, ContextOptimizerError> {
        serde_json::to_string_pretty(self).map_err(ContextOptimizerError::Serialization)
    }

    /// Deserializes a report from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ContextOptimizerError> {
        serde_json::from_str(json).map_err(ContextOptimizerError::Serialization)
    }
}

/// Measurement of document volume (tokens, UTF-8 bytes, characters, and sections).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocumentVolume {
    /// Total token count.
    pub tokens: usize,
    /// Total UTF-8 byte count.
    pub bytes: usize,
    /// Total character count.
    pub chars: usize,
    /// Total section count.
    pub sections: usize,
}

impl DocumentVolume {
    /// Measures the volume of a [`CdmDocument`].
    #[must_use]
    pub fn measure(doc: &CdmDocument, model: Option<TargetModel>) -> Self {
        let registry = TokenizerRegistry::default();
        let adapter: Option<std::sync::Arc<dyn TokenizerAdapter>> =
            model.and_then(|m| registry.get_for_model(m));
        let fallback = CalibratedHeuristicAdapter::default();

        let mut tokens = 0;
        let mut bytes = 0;
        let mut chars = 0;

        for s in &doc.sections {
            bytes += s.body.len();
            chars += s.body.chars().count();
            if let Some(ref a) = adapter {
                tokens += a.count_tokens(&s.body).count;
            } else if let Some(ref metrics) = s.token_metrics {
                if let Some(exact) = metrics.exact_tokens {
                    tokens += exact;
                } else {
                    tokens += fallback.count_tokens(&s.body).count;
                }
            } else {
                tokens += fallback.count_tokens(&s.body).count;
            }
        }

        Self {
            tokens,
            bytes,
            chars,
            sections: doc.sections.len(),
        }
    }
}

/// Multi-stage pipeline tracker recording metrics at each transformation transition.
#[derive(Debug, Clone)]
pub struct CompressionPipelineTracker {
    initial_volume: DocumentVolume,
    current_volume: DocumentVolume,
    stages: Vec<StageCompressionMetrics>,
    profile_name: Option<String>,
    target_model: Option<TargetModel>,
    notes: Vec<String>,
}

impl CompressionPipelineTracker {
    /// Initializes a new pipeline tracker with the unoptimized baseline document.
    pub fn new(initial_doc: &CdmDocument) -> Result<Self, ContextOptimizerError> {
        if initial_doc.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        let volume = DocumentVolume::measure(initial_doc, None);

        Ok(Self {
            initial_volume: volume,
            current_volume: volume,
            stages: Vec::new(),
            profile_name: None,
            target_model: None,
            notes: Vec::new(),
        })
    }

    /// Configures the target model used for token counting and financial cost calculations.
    #[must_use]
    pub fn with_target_model(mut self, model: TargetModel) -> Self {
        self.target_model = Some(model);
        self
    }

    /// Configures the name of the optimization profile.
    #[must_use]
    pub fn with_profile_name(mut self, name: impl Into<String>) -> Self {
        self.profile_name = Some(name.into());
        self
    }

    /// Appends an informational note to the final report.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Records a stage transition by measuring the document exiting the stage.
    pub fn record_stage(&mut self, stage: OptimizationStage, after_doc: &CdmDocument) {
        let after_volume = DocumentVolume::measure(after_doc, self.target_model);

        let stage_metrics = StageCompressionMetrics::calculate(
            stage,
            self.current_volume.tokens,
            after_volume.tokens,
            self.current_volume.bytes,
            after_volume.bytes,
            self.current_volume.sections,
            after_volume.sections,
        );

        self.stages.push(stage_metrics);
        self.current_volume = after_volume;
    }

    /// Generates the cumulative compression report.
    #[must_use]
    pub fn generate_report(&self) -> CompressionReport {
        let overall =
            OverallCompressionMetrics::from_volumes(&self.initial_volume, &self.current_volume);

        let cost_savings = MultiModelCostSavings::calculate(
            self.initial_volume.tokens,
            self.current_volume.tokens,
        );

        CompressionReport {
            generated_at: Utc::now(),
            profile_name: self.profile_name.clone(),
            target_model: self.target_model,
            overall,
            stages: self.stages.clone(),
            cost_savings,
            notes: self.notes.clone(),
        }
    }
}

/// Comparative compression evaluator measuring content reduction between two [`CdmDocument`]s.
pub struct CompressionEvaluator;

impl CompressionEvaluator {
    /// Compares an original document against an optimized document using heuristic token estimates.
    pub fn evaluate(
        original: &CdmDocument,
        optimized: &CdmDocument,
    ) -> Result<CompressionReport, ContextOptimizerError> {
        Self::evaluate_with_profile(original, optimized, None::<String>, None)
    }

    /// Compares an original document against an optimized document for a specific [`TargetModel`].
    pub fn evaluate_for_model(
        original: &CdmDocument,
        optimized: &CdmDocument,
        model: TargetModel,
    ) -> Result<CompressionReport, ContextOptimizerError> {
        Self::evaluate_with_profile(original, optimized, None::<String>, Some(model))
    }

    /// Compares an original document against an optimized document with profile identification and optional target model.
    pub fn evaluate_with_profile(
        original: &CdmDocument,
        optimized: &CdmDocument,
        profile_name: Option<impl Into<String>>,
        target_model: Option<TargetModel>,
    ) -> Result<CompressionReport, ContextOptimizerError> {
        if original.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        let orig_vol = DocumentVolume::measure(original, target_model);
        let opt_vol = DocumentVolume::measure(optimized, target_model);

        let overall = OverallCompressionMetrics::from_volumes(&orig_vol, &opt_vol);

        let cost_savings = MultiModelCostSavings::calculate(orig_vol.tokens, opt_vol.tokens);

        // Derive high-level pipeline stages from exclusions and truncation differences
        let mut stages = Vec::new();

        if orig_vol.tokens != opt_vol.tokens || orig_vol.bytes != opt_vol.bytes {
            let stage_kind = if profile_name.is_some() {
                OptimizationStage::ProfileFiltering
            } else {
                OptimizationStage::BudgetKnapsack
            };

            stages.push(StageCompressionMetrics::calculate(
                stage_kind,
                orig_vol.tokens,
                opt_vol.tokens,
                orig_vol.bytes,
                opt_vol.bytes,
                orig_vol.sections,
                opt_vol.sections,
            ));
        }

        let mut notes = Vec::new();
        if overall.token_reduction_pct >= 50.0 {
            notes.push(format!(
                "Significant token optimization achieved: {:.1}% reduction ({:.2}x compression ratio).",
                overall.token_reduction_pct, overall.token_compression_ratio
            ));
        }
        if !optimized.exclusions.is_empty() {
            notes.push(format!(
                "Document exclusions: {} sections logged in audit trail.",
                optimized.exclusions.len()
            ));
        }

        Ok(CompressionReport {
            generated_at: Utc::now(),
            profile_name: profile_name.map(Into::into),
            target_model,
            overall,
            stages,
            cost_savings,
            notes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
        RevisionInfo, SectionKind, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
        CdmDocument {
            schema_version: "1.0.0".to_string(),
            document_id: DocumentId::new(),
            source_identity: SourceIdentity::new(
                "https://github.com/org/repo",
                "https://github.com/org/repo",
            ),
            source_type: SourceType::GitHubPublic,
            canonical_url: "https://github.com/org/repo".to_string(),
            retrieved_at: Utc::now(),
            revision: RevisionInfo::default(),
            content_type: "application/vnd.github.v3+json".to_string(),
            metadata: DocumentMetadata::default(),
            hierarchy: HierarchyNode::root("root"),
            sections,
            relationships: Vec::new(),
            citations: Vec::new(),
            extraction_status: ExtractionStatus::Completed,
            warnings: Vec::new(),
            exclusions: Vec::new(),
            completeness: CompletenessReport::default(),
        }
    }

    fn make_section(id: &str, path: &str, kind: SectionKind, body: &str) -> CdmSection {
        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(path.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!("https://github.com/org/repo/blob/main/{path}"),
            line_range: None,
            byte_range: None,
            language: if path.ends_with(".rs") {
                Some("rust".to_string())
            } else {
                None
            },
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: None,
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn test_stage_compression_metrics_calculation() {
        let m = StageCompressionMetrics::calculate(
            OptimizationStage::Normalization,
            1000,
            800,
            5000,
            4000,
            10,
            10,
        );

        assert_eq!(m.stage, OptimizationStage::Normalization);
        assert_eq!(m.token_savings, 200);
        assert_eq!(m.token_reduction_pct, 20.0);
        assert_eq!(m.token_compression_ratio, 1.25);
        assert_eq!(m.byte_savings, 1000);
        assert_eq!(m.byte_reduction_pct, 20.0);
        assert_eq!(m.byte_compression_ratio, 1.25);
        assert_eq!(m.section_reduction, 0);

        let row = m.to_markdown_row();
        assert!(row.contains("Structural Normalization"));
        assert!(row.contains("1.25x"));
    }

    #[test]
    fn test_overall_compression_metrics_and_markdown() {
        let m = OverallCompressionMetrics::calculate(
            10_000, 2_500, 40_000, 10_000, 35_000, 8_750, 20, 8,
        );

        assert_eq!(m.total_token_savings, 7_500);
        assert_eq!(m.token_reduction_pct, 75.0);
        assert_eq!(m.token_compression_ratio, 4.0);
        assert_eq!(m.total_byte_savings, 30_000);
        assert_eq!(m.byte_reduction_pct, 75.0);
        assert_eq!(m.total_sections_removed, 12);

        let table = m.to_markdown_table();
        assert!(table.contains("Tokens"));
        assert!(table.contains("4.00x"));
        assert!(table.contains("75.0%"));
    }

    #[test]
    fn test_multi_model_cost_savings() {
        let savings = MultiModelCostSavings::calculate(100_000, 25_000);
        assert!(!savings.models.is_empty());

        let gpt4o = savings
            .models
            .iter()
            .find(|m| m.model == TargetModel::Gpt4o)
            .expect("GPT-4o present");

        assert_eq!(gpt4o.savings_pct, 75.0);
        assert!(gpt4o.cost_savings_usd > 0.0);

        let llama3 = savings
            .models
            .iter()
            .find(|m| m.model == TargetModel::Llama3)
            .expect("Llama 3 present");

        assert!(llama3.is_free_or_self_hosted);
        assert_eq!(llama3.cost_savings_usd, 0.0);

        let table = savings.to_markdown_table();
        assert!(table.contains("gpt-4o"));
        assert!(table.contains("llama-3"));
    }

    #[test]
    fn test_pipeline_tracker_progression() {
        let s1 = make_section(
            "s1",
            "src/main.rs",
            SectionKind::CodeFile,
            "fn main() { println!(\"hello\"); }",
        );
        let s2 = make_section(
            "s2",
            "src/lib.rs",
            SectionKind::CodeFile,
            "pub fn helper() -> usize { 42 }",
        );
        let s3 = make_section(
            "s3",
            "tests/test.rs",
            SectionKind::CodeFile,
            "assert_eq!(1, 1);",
        );
        let s4 = make_section("s4", "README.md", SectionKind::Prose, "# Readme doc");

        let doc0 = make_test_doc(vec![s1.clone(), s2.clone(), s3.clone(), s4]);

        let mut tracker = CompressionPipelineTracker::new(&doc0)
            .expect("Valid initial doc")
            .with_profile_name("CodeReview")
            .with_target_model(TargetModel::Claude35Sonnet)
            .with_note("Prioritized modified code files and tests.");

        // Stage 1: Normalization (retains same sections, slightly cleaned)
        let doc1 = doc0.clone();
        tracker.record_stage(OptimizationStage::Normalization, &doc1);

        // Stage 2: Profile Filtering (excludes README)
        let doc2 = make_test_doc(vec![s1, s2, s3]);
        tracker.record_stage(OptimizationStage::ProfileFiltering, &doc2);

        let report = tracker.generate_report();
        assert_eq!(report.stages.len(), 2);
        assert_eq!(report.overall.original_sections, 4);
        assert_eq!(report.overall.retained_sections, 3);
        assert_eq!(report.overall.total_sections_removed, 1);
        assert_eq!(report.profile_name.as_deref(), Some("CodeReview"));

        let md = report.to_markdown_summary();
        assert!(md.contains("CodeReview"));
        assert!(md.contains("claude-3-5-sonnet"));
    }

    #[test]
    fn test_empty_document_fails_closed() {
        let empty_doc = make_test_doc(vec![]);
        let err = CompressionPipelineTracker::new(&empty_doc).expect_err("Should fail closed");
        assert!(matches!(err, ContextOptimizerError::DocumentEmpty));

        let err2 = CompressionEvaluator::evaluate(&empty_doc, &empty_doc).expect_err("Should fail");
        assert!(matches!(err2, ContextOptimizerError::DocumentEmpty));
    }
}
