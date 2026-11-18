//! # Context Budget Modeling and Headroom Allocation
//!
//! Models LLM context window boundaries, total capacity ceilings, and multi-tier
//! reserved headroom allocations (system prompts, task queries, generation headroom).

use crate::error::ContextOptimizerError;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_tokenizer::TargetModel;

/// Comprehensive specification of LLM context limits and reserved headroom tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    /// Target LLM model architecture, if bound to a specific model.
    pub target_model: Option<TargetModel>,
    /// Total context window capacity ceiling in tokens.
    pub total_tokens: usize,
    /// Reserved token allowance for system prompt, role instructions, and safety directives.
    pub reserved_system_tokens: usize,
    /// Reserved token allowance for LLM output generation response headroom.
    pub reserved_output_tokens: usize,
    /// Reserved token allowance for user query, task specification, or downstream prompt additions.
    pub reserved_query_tokens: usize,
}

impl ContextBudget {
    /// Creates a new custom context budget with explicit token allocations.
    #[must_use]
    pub const fn new(
        total_tokens: usize,
        reserved_system_tokens: usize,
        reserved_output_tokens: usize,
        reserved_query_tokens: usize,
    ) -> Self {
        Self {
            target_model: None,
            total_tokens,
            reserved_system_tokens,
            reserved_output_tokens,
            reserved_query_tokens,
        }
    }

    /// Derives a balanced context budget directly from a [`TargetModel`]'s full context window.
    #[must_use]
    pub fn from_model(model: TargetModel) -> Self {
        let window = model.context_window_limit();
        let (sys, out, query) = Self::calculate_default_headroom(window);
        Self {
            target_model: Some(model),
            total_tokens: window,
            reserved_system_tokens: sys,
            reserved_output_tokens: out,
            reserved_query_tokens: query,
        }
    }

    /// Derives a conservative context budget using a [`TargetModel`]'s recommended safe budget.
    #[must_use]
    pub fn from_model_recommended(model: TargetModel) -> Self {
        let budget = model.recommended_budget_ceiling();
        let (sys, out, query) = Self::calculate_default_headroom(budget);
        Self {
            target_model: Some(model),
            total_tokens: budget,
            reserved_system_tokens: sys,
            reserved_output_tokens: out,
            reserved_query_tokens: query,
        }
    }

    /// Creates a context budget with proportionate headroom allocations for a specified total capacity.
    #[must_use]
    pub fn with_total_tokens(total: usize) -> Self {
        let (sys, out, query) = Self::calculate_default_headroom(total);
        Self {
            target_model: None,
            total_tokens: total,
            reserved_system_tokens: sys,
            reserved_output_tokens: out,
            reserved_query_tokens: query,
        }
    }

    /// Helper computing proportionate headroom allowances based on total token budget.
    fn calculate_default_headroom(total: usize) -> (usize, usize, usize) {
        if total >= 100_000 {
            // Large context windows (GPT-4o, Claude 3.5 Sonnet, Gemini)
            (2_000, 4_096, 1_500)
        } else if total >= 32_000 {
            // Mid-size context windows (Mixtral 8x7B, 32k models)
            (1_500, 3_000, 1_000)
        } else if total >= 8_000 {
            // Standard 8k context windows (Llama 3, GPT-4 8k)
            (800, 1_500, 500)
        } else if total >= 2_000 {
            // Small context windows
            (200, 500, 150)
        } else {
            // Micro budgets: scale down to 10% system, 25% output, 10% query
            let sys = (total * 10 / 100).max(1);
            let out = (total * 25 / 100).max(1);
            let query = (total * 10 / 100).max(1);
            (sys, out, query)
        }
    }

    /// Sets the target model binding.
    #[must_use]
    pub const fn with_target_model(mut self, model: TargetModel) -> Self {
        self.target_model = Some(model);
        self
    }

    /// Overrides the reserved system tokens allowance.
    #[must_use]
    pub const fn with_system_reservation(mut self, tokens: usize) -> Self {
        self.reserved_system_tokens = tokens;
        self
    }

    /// Overrides the reserved generation output tokens allowance.
    #[must_use]
    pub const fn with_output_reservation(mut self, tokens: usize) -> Self {
        self.reserved_output_tokens = tokens;
        self
    }

    /// Overrides the reserved query prompt tokens allowance.
    #[must_use]
    pub const fn with_query_reservation(mut self, tokens: usize) -> Self {
        self.reserved_query_tokens = tokens;
        self
    }

    /// Computes the sum of all reserved headroom tokens.
    #[must_use]
    pub const fn total_reserved_tokens(&self) -> usize {
        self.reserved_system_tokens + self.reserved_output_tokens + self.reserved_query_tokens
    }

    /// Computes the effective content capacity available for source code, web pages, and documents.
    #[must_use]
    pub const fn available_content_tokens(&self) -> usize {
        self.total_tokens
            .saturating_sub(self.total_reserved_tokens())
    }

    /// Ratio of reserved headroom tokens to total context window capacity.
    #[must_use]
    pub fn headroom_ratio(&self) -> f64 {
        if self.total_tokens == 0 {
            0.0
        } else {
            self.total_reserved_tokens() as f64 / self.total_tokens as f64
        }
    }

    /// Ratio of available content tokens to total context window capacity.
    #[must_use]
    pub fn content_ratio(&self) -> f64 {
        if self.total_tokens == 0 {
            0.0
        } else {
            self.available_content_tokens() as f64 / self.total_tokens as f64
        }
    }

    /// Validates that the budget configuration is mathematically consistent and usable.
    pub fn validate(&self) -> Result<(), ContextOptimizerError> {
        if self.total_tokens == 0 {
            return Err(ContextOptimizerError::InvalidBudget {
                reason: "Total context window capacity must be greater than zero".to_string(),
            });
        }

        let reserved = self.total_reserved_tokens();
        if reserved >= self.total_tokens {
            return Err(ContextOptimizerError::InvalidBudget {
                reason: format!(
                    "Reserved headroom tokens ({reserved}) exceed or equal total context budget ({})",
                    self.total_tokens
                ),
            });
        }

        if self.available_content_tokens() == 0 {
            return Err(ContextOptimizerError::InvalidBudget {
                reason: "Available content capacity resolved to zero tokens".to_string(),
            });
        }

        Ok(())
    }

    /// Generates a structured summary report of budget headroom allocations.
    #[must_use]
    pub fn headroom_summary(&self) -> BudgetHeadroomSummary {
        let total = self.total_tokens;
        let available = self.available_content_tokens();

        BudgetHeadroomSummary {
            target_model: self.target_model,
            total_tokens: total,
            reserved_system_tokens: self.reserved_system_tokens,
            reserved_output_tokens: self.reserved_output_tokens,
            reserved_query_tokens: self.reserved_query_tokens,
            available_content_tokens: available,
            content_ratio: self.content_ratio(),
            headroom_ratio: self.headroom_ratio(),
        }
    }
}

/// Structured summary of budget headroom and allocation percentages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetHeadroomSummary {
    /// Target model, if bound.
    pub target_model: Option<TargetModel>,
    /// Total budget tokens.
    pub total_tokens: usize,
    /// System prompt reservation.
    pub reserved_system_tokens: usize,
    /// Generation output reservation.
    pub reserved_output_tokens: usize,
    /// User query reservation.
    pub reserved_query_tokens: usize,
    /// Usable content tokens.
    pub available_content_tokens: usize,
    /// Content fraction of total budget (0.0 - 1.0).
    pub content_ratio: f64,
    /// Headroom fraction of total budget (0.0 - 1.0).
    pub headroom_ratio: f64,
}

impl BudgetHeadroomSummary {
    /// Renders a GitHub Flavored Markdown summary table of the headroom allocation.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let model_str = self
            .target_model
            .map_or("Custom / Generic", TargetModel::as_str);

        format!(
            "| Component | Tokens | Share (%) |\n\
             | :--- | :---: | :---: |\n\
             | **Target Model** | `{model_str}` | — |\n\
             | **Total Window** | `{}` | 100.0% |\n\
             | Reserved System | `{}` | {:.1}% |\n\
             | Reserved Output | `{}` | {:.1}% |\n\
             | Reserved Query | `{}` | {:.1}% |\n\
             | **Available Content** | `{}` | **{:.1}%** |\n",
            self.total_tokens,
            self.reserved_system_tokens,
            (self.reserved_system_tokens as f64 / self.total_tokens as f64) * 100.0,
            self.reserved_output_tokens,
            (self.reserved_output_tokens as f64 / self.total_tokens as f64) * 100.0,
            self.reserved_query_tokens,
            (self.reserved_query_tokens as f64 / self.total_tokens as f64) * 100.0,
            self.available_content_tokens,
            self.content_ratio * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_derivation_from_models() {
        let gpt4o_budget = ContextBudget::from_model(TargetModel::Gpt4o);
        assert_eq!(gpt4o_budget.total_tokens, 128_000);
        assert_eq!(gpt4o_budget.reserved_system_tokens, 2_000);
        assert_eq!(gpt4o_budget.reserved_output_tokens, 4_096);
        assert_eq!(gpt4o_budget.reserved_query_tokens, 1_500);
        assert_eq!(gpt4o_budget.total_reserved_tokens(), 7_596);
        assert_eq!(gpt4o_budget.available_content_tokens(), 120_404);
        assert!(gpt4o_budget.validate().is_ok());

        let llama3_budget = ContextBudget::from_model(TargetModel::Llama3);
        assert_eq!(llama3_budget.total_tokens, 128_000);
        assert_eq!(llama3_budget.reserved_system_tokens, 2_000);
        assert_eq!(llama3_budget.reserved_output_tokens, 4_096);
        assert_eq!(llama3_budget.reserved_query_tokens, 1_500);
        assert_eq!(llama3_budget.total_reserved_tokens(), 7_596);
        assert_eq!(llama3_budget.available_content_tokens(), 120_404);
        assert!(llama3_budget.validate().is_ok());
    }

    #[test]
    fn test_budget_recommended_derivation() {
        let gpt4o_rec = ContextBudget::from_model_recommended(TargetModel::Gpt4o);
        assert_eq!(gpt4o_rec.total_tokens, 111_104);
        assert!(gpt4o_rec.validate().is_ok());
    }

    #[test]
    fn test_budget_custom_overrides_and_validation() {
        let budget = ContextBudget::with_total_tokens(10_000)
            .with_system_reservation(500)
            .with_output_reservation(1000)
            .with_query_reservation(500);

        assert_eq!(budget.total_tokens, 10_000);
        assert_eq!(budget.total_reserved_tokens(), 2_000);
        assert_eq!(budget.available_content_tokens(), 8_000);
        assert!((budget.headroom_ratio() - 0.20).abs() < 1e-6);
        assert!((budget.content_ratio() - 0.80).abs() < 1e-6);
        assert!(budget.validate().is_ok());
    }

    #[test]
    fn test_invalid_budgets_rejected() {
        let zero_total = ContextBudget::new(0, 0, 0, 0);
        assert!(zero_total.validate().is_err());

        let headroom_exceeded = ContextBudget::new(1000, 500, 400, 200);
        assert!(headroom_exceeded.validate().is_err());

        let equal_headroom = ContextBudget::new(1000, 400, 400, 200);
        assert!(equal_headroom.validate().is_err());
    }

    #[test]
    fn test_headroom_summary_markdown_table() {
        let budget = ContextBudget::from_model(TargetModel::Claude35Sonnet);
        let summary = budget.headroom_summary();
        let table = summary.to_markdown_table();
        assert!(table.contains("claude-3-5-sonnet"));
        assert!(table.contains("200000"));
        assert!(table.contains("Available Content"));
    }
}
