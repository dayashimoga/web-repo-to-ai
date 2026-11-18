//! # Token Cost Intelligence & Provider Pricing Models
//!
//! Provides model-specific pricing schedules, input/output cost calculations,
//! prompt caching discounts, batch API pricing, and dollar budget guardrails.

use crate::error::TokenizerError;
use crate::model::TargetModel;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::CdmDocument;

/// Provider pricing schedule for a target LLM architecture (in USD per 1,000,000 tokens).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPricingSchedule {
    /// Associated target model.
    pub model: TargetModel,
    /// Cost per 1,000,000 standard input tokens in USD.
    pub input_cost_per_million: f64,
    /// Cost per 1,000,000 prompt-cached read input tokens in USD (if supported).
    pub cached_input_cost_per_million: Option<f64>,
    /// Cost per 1,000,000 generated output tokens in USD.
    pub output_cost_per_million: f64,
    /// Batch API discount percentage (typically 50.0% for batch queues).
    pub batch_discount_pct: f64,
    /// True if the model runs self-hosted (zero API fees, e.g. Llama 3 via Ollama/vLLM).
    pub is_free_or_self_hosted: bool,
}

impl ModelPricingSchedule {
    /// Returns the standard published pricing schedule for `model`.
    #[must_use]
    pub fn for_model(model: TargetModel) -> Self {
        match model {
            TargetModel::Gpt4o => Self {
                model,
                input_cost_per_million: 2.50,
                cached_input_cost_per_million: Some(1.25),
                output_cost_per_million: 10.00,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Gpt4oMini => Self {
                model,
                input_cost_per_million: 0.15,
                cached_input_cost_per_million: Some(0.075),
                output_cost_per_million: 0.60,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Gpt4Turbo => Self {
                model,
                input_cost_per_million: 10.00,
                cached_input_cost_per_million: None,
                output_cost_per_million: 30.00,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Claude35Sonnet => Self {
                model,
                input_cost_per_million: 3.00,
                cached_input_cost_per_million: Some(0.30),
                output_cost_per_million: 15.00,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Claude3Opus => Self {
                model,
                input_cost_per_million: 15.00,
                cached_input_cost_per_million: Some(1.50),
                output_cost_per_million: 75.00,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Claude3Haiku => Self {
                model,
                input_cost_per_million: 0.25,
                cached_input_cost_per_million: Some(0.025),
                output_cost_per_million: 1.25,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Gemini15Pro => Self {
                model,
                input_cost_per_million: 1.25,
                cached_input_cost_per_million: Some(0.3125),
                output_cost_per_million: 5.00,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Gemini15Flash => Self {
                model,
                input_cost_per_million: 0.075,
                cached_input_cost_per_million: Some(0.01875),
                output_cost_per_million: 0.30,
                batch_discount_pct: 50.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Llama3 | TargetModel::Llama370B => Self {
                model,
                input_cost_per_million: 0.0,
                cached_input_cost_per_million: Some(0.0),
                output_cost_per_million: 0.0,
                batch_discount_pct: 0.0,
                is_free_or_self_hosted: true,
            },
            TargetModel::MistralLarge => Self {
                model,
                input_cost_per_million: 2.00,
                cached_input_cost_per_million: None,
                output_cost_per_million: 6.00,
                batch_discount_pct: 0.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::Mixtral8x7B => Self {
                model,
                input_cost_per_million: 0.24,
                cached_input_cost_per_million: None,
                output_cost_per_million: 0.24,
                batch_discount_pct: 0.0,
                is_free_or_self_hosted: false,
            },
            TargetModel::GenericFallback => Self {
                model,
                input_cost_per_million: 0.0,
                cached_input_cost_per_million: None,
                output_cost_per_million: 0.0,
                batch_discount_pct: 0.0,
                is_free_or_self_hosted: true,
            },
        }
    }
}

/// Detailed financial cost estimation for processing context through a target model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Target model analyzed.
    pub model: TargetModel,
    /// Evaluated input tokens.
    pub input_tokens: usize,
    /// Estimated output generation tokens.
    pub estimated_output_tokens: usize,
    /// Standard on-demand execution cost in USD.
    pub standard_cost_usd: f64,
    /// Estimated cost with prompt-cache hits in USD (if supported).
    pub cached_cost_usd: Option<f64>,
    /// Estimated cost with batch processing discount in USD.
    pub batch_cost_usd: f64,
    /// Total dollar savings from prompt caching.
    pub cache_savings_usd: f64,
    /// Percentage savings from prompt caching.
    pub cache_savings_pct: f64,
    /// Total dollar savings from batch execution.
    pub batch_savings_usd: f64,
    /// Percentage savings from batch execution.
    pub batch_savings_pct: f64,
    /// True if model is free or self-hosted.
    pub is_free_or_self_hosted: bool,
}

impl CostBreakdown {
    /// Calculates the cost breakdown for a given input token volume and output allowance.
    #[must_use]
    pub fn calculate(
        input_tokens: usize,
        model: TargetModel,
        estimated_output_tokens: usize,
    ) -> Self {
        let schedule = ModelPricingSchedule::for_model(model);

        if schedule.is_free_or_self_hosted {
            return Self {
                model,
                input_tokens,
                estimated_output_tokens,
                standard_cost_usd: 0.0,
                cached_cost_usd: Some(0.0),
                batch_cost_usd: 0.0,
                cache_savings_usd: 0.0,
                cache_savings_pct: 0.0,
                batch_savings_usd: 0.0,
                batch_savings_pct: 0.0,
                is_free_or_self_hosted: true,
            };
        }

        let in_cost = (input_tokens as f64 * schedule.input_cost_per_million) / 1_000_000.0;
        let out_cost =
            (estimated_output_tokens as f64 * schedule.output_cost_per_million) / 1_000_000.0;
        let standard_cost_usd = ((in_cost + out_cost) * 100_000.0).round() / 100_000.0;

        let (cached_cost_usd, cache_savings_usd, cache_savings_pct) = match schedule
            .cached_input_cost_per_million
        {
            Some(cached_rate) => {
                let cached_in_cost = (input_tokens as f64 * cached_rate) / 1_000_000.0;
                let cached_total = ((cached_in_cost + out_cost) * 100_000.0).round() / 100_000.0;
                let savings = (standard_cost_usd - cached_total).max(0.0);
                let pct = if standard_cost_usd > 0.0 {
                    ((savings / standard_cost_usd * 100.0) * 10.0).round() / 10.0
                } else {
                    0.0
                };
                (Some(cached_total), savings, pct)
            }
            None => (None, 0.0, 0.0),
        };

        let batch_mult = (100.0 - schedule.batch_discount_pct) / 100.0;
        let batch_cost_usd = ((standard_cost_usd * batch_mult) * 100_000.0).round() / 100_000.0;
        let batch_savings_usd = (standard_cost_usd - batch_cost_usd).max(0.0);
        let batch_savings_pct = schedule.batch_discount_pct;

        Self {
            model,
            input_tokens,
            estimated_output_tokens,
            standard_cost_usd,
            cached_cost_usd,
            batch_cost_usd,
            cache_savings_usd,
            cache_savings_pct,
            batch_savings_usd,
            batch_savings_pct,
            is_free_or_self_hosted: false,
        }
    }
}

/// Multi-model cost comparison across target architectures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossModelCostReport {
    /// Input tokens evaluated.
    pub input_tokens: usize,
    /// Estimated output tokens.
    pub estimated_output_tokens: usize,
    /// Cost breakdowns for each evaluated model.
    pub breakdowns: Vec<CostBreakdown>,
    /// Most cost-effective paid commercial model.
    pub cheapest_paid_model: Option<TargetModel>,
    /// Most expensive commercial model.
    pub most_expensive_model: Option<TargetModel>,
    /// Price ratio between the most expensive and cheapest paid commercial models.
    pub price_ratio_expensive_vs_cheapest: f64,
}

impl CrossModelCostReport {
    /// Compares costs across target models (or standard supported models if empty).
    #[must_use]
    pub fn compare(
        input_tokens: usize,
        estimated_output_tokens: usize,
        models: &[TargetModel],
    ) -> Self {
        let models_to_eval = if models.is_empty() {
            TargetModel::supported_models()
        } else {
            models
        };

        let mut breakdowns = Vec::new();
        let mut min_paid_cost = f64::MAX;
        let mut max_cost = 0.0;
        let mut cheapest_paid = None;
        let mut most_expensive = None;

        for &model in models_to_eval {
            let breakdown = CostBreakdown::calculate(input_tokens, model, estimated_output_tokens);
            if !breakdown.is_free_or_self_hosted {
                if breakdown.standard_cost_usd < min_paid_cost {
                    min_paid_cost = breakdown.standard_cost_usd;
                    cheapest_paid = Some(model);
                }
                if breakdown.standard_cost_usd > max_cost {
                    max_cost = breakdown.standard_cost_usd;
                    most_expensive = Some(model);
                }
            }
            breakdowns.push(breakdown);
        }

        let price_ratio_expensive_vs_cheapest = if min_paid_cost > 0.0 && max_cost >= min_paid_cost
        {
            ((max_cost / min_paid_cost) * 10.0).round() / 10.0
        } else {
            1.0
        };

        Self {
            input_tokens,
            estimated_output_tokens,
            breakdowns,
            cheapest_paid_model: cheapest_paid,
            most_expensive_model: most_expensive,
            price_ratio_expensive_vs_cheapest,
        }
    }

    /// Compares cost across models for a [`CdmDocument`].
    #[must_use]
    pub fn compare_document(
        doc: &CdmDocument,
        estimated_output_tokens: usize,
        models: &[TargetModel],
    ) -> Self {
        let total_chars: usize = doc.sections.iter().map(|s| s.body.chars().count()).sum();
        // Calibrated baseline: ~3.5 chars/token
        let estimated_tokens = (total_chars as f64 / 3.5).ceil() as usize;
        Self::compare(estimated_tokens, estimated_output_tokens, models)
    }

    /// Renders a human-readable GitHub Flavored Markdown cost table.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let mut md = String::new();
        md.push_str("### Cross-Model Inference Cost Intelligence\n\n");
        md.push_str(&format!(
            "- **Evaluated Input Tokens:** `{}` | **Estimated Output Tokens:** `{}`\n\n",
            self.input_tokens, self.estimated_output_tokens
        ));
        md.push_str(
            "| Target Model | Standard Cost | Cached Read Cost | Batch API Cost | Model Tier |\n",
        );
        md.push_str("| :--- | :---: | :---: | :---: | :--- |\n");

        for b in &self.breakdowns {
            let cached_str = match b.cached_cost_usd {
                Some(cost) if cost > 0.0 => {
                    format!("${cost:.4} (-{}%)", b.cache_savings_pct.round())
                }
                Some(_) => "$0.0000".to_string(),
                None => "N/A".to_string(),
            };

            let std_str = if b.is_free_or_self_hosted {
                "**$0.0000** (Free)".to_string()
            } else {
                format!("${:.4}", b.standard_cost_usd)
            };

            let batch_str = if b.is_free_or_self_hosted {
                "$0.0000".to_string()
            } else {
                format!(
                    "${:.4} (-{}%)",
                    b.batch_cost_usd,
                    b.batch_savings_pct.round()
                )
            };

            let tier_marker = if b.is_free_or_self_hosted {
                "Self-Hosted / Open-Source"
            } else if Some(b.model) == self.cheapest_paid_model {
                "**Cheapest Commercial**"
            } else if Some(b.model) == self.most_expensive_model {
                "Flagship / Frontier"
            } else {
                "Standard Commercial"
            };

            md.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                b.model.as_str(),
                std_str,
                cached_str,
                batch_str,
                tier_marker
            ));
        }

        if let (Some(cheapest), Some(most_exp)) =
            (self.cheapest_paid_model, self.most_expensive_model)
        {
            md.push_str(&format!(
                "\n> [!TIP] Cost Optimization Insight\n> Switching from `{}` to `{}` yields a **{}x cost reduction** while retaining complete repository context.\n",
                most_exp.as_str(), cheapest.as_str(), self.price_ratio_expensive_vs_cheapest
            ));
        }

        md
    }
}

/// Dollar budget guardrail enforcing monetary spending ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CostGuardrail {
    /// Maximum allowable cost in USD for a single conversion or inference task.
    pub max_budget_usd: f64,
}

/// Result of evaluating a cost against a [`CostGuardrail`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostGuardrailEvaluation {
    /// Cost is strictly within configured budget.
    WithinBudget {
        /// Dollar headroom remaining under budget ceiling.
        remaining_budget_usd: f64,
    },
    /// Cost exceeds configured budget limit.
    BudgetExceeded {
        /// Dollar amount exceeding the ceiling.
        excess_usd: f64,
        /// Recommended more economical model architectures that fit within budget.
        suggested_alternatives: Vec<TargetModel>,
    },
}

impl CostGuardrail {
    /// Constructs a cost guardrail with a given dollar ceiling.
    #[must_use]
    pub const fn new(max_budget_usd: f64) -> Self {
        Self { max_budget_usd }
    }

    /// Evaluates `breakdown` against the guardrail.
    #[must_use]
    pub fn evaluate(&self, breakdown: &CostBreakdown) -> CostGuardrailEvaluation {
        if breakdown.standard_cost_usd <= self.max_budget_usd {
            let remaining = ((self.max_budget_usd - breakdown.standard_cost_usd) * 100_000.0)
                .round()
                / 100_000.0;
            CostGuardrailEvaluation::WithinBudget {
                remaining_budget_usd: remaining.max(0.0),
            }
        } else {
            let excess = ((breakdown.standard_cost_usd - self.max_budget_usd) * 100_000.0).round()
                / 100_000.0;

            // Find alternative models that fit within budget
            let mut alternatives = Vec::new();
            for &candidate in TargetModel::supported_models() {
                if candidate != breakdown.model {
                    let alt_cost = CostBreakdown::calculate(
                        breakdown.input_tokens,
                        candidate,
                        breakdown.estimated_output_tokens,
                    );
                    if alt_cost.standard_cost_usd <= self.max_budget_usd {
                        alternatives.push(candidate);
                    }
                }
            }

            CostGuardrailEvaluation::BudgetExceeded {
                excess_usd: excess,
                suggested_alternatives: alternatives,
            }
        }
    }

    /// Validates `breakdown` and returns an error if the budget is breached.
    ///
    /// # Errors
    /// Returns [`TokenizerError::CostBudgetExceeded`] if cost exceeds `max_budget_usd`.
    pub fn check_or_err(&self, breakdown: &CostBreakdown) -> Result<(), TokenizerError> {
        if breakdown.standard_cost_usd > self.max_budget_usd {
            Err(TokenizerError::CostBudgetExceeded {
                budget_usd: self.max_budget_usd,
                estimated_usd: breakdown.standard_cost_usd,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_breakdown_calculation() {
        // 100k input tokens + 2k output tokens on GPT-4o
        let cost = CostBreakdown::calculate(100_000, TargetModel::Gpt4o, 2_000);

        // 100k * $2.50 / 1M = $0.25; 2k * $10.00 / 1M = $0.02 -> $0.27
        assert_eq!(cost.standard_cost_usd, 0.27);
        // Cached read: 100k * $1.25 / 1M = $0.125 + $0.02 = $0.145
        assert_eq!(cost.cached_cost_usd, Some(0.145));
        assert!(cost.cache_savings_usd > 0.0);
        assert_eq!(cost.batch_savings_pct, 50.0);
    }

    #[test]
    fn test_self_hosted_llama_is_free() {
        let cost = CostBreakdown::calculate(1_000_000, TargetModel::Llama3, 5_000);
        assert!(cost.is_free_or_self_hosted);
        assert_eq!(cost.standard_cost_usd, 0.0);
    }

    #[test]
    fn test_cross_model_cost_report() {
        let report = CrossModelCostReport::compare(50_000, 1_000, &[]);
        assert!(!report.breakdowns.is_empty());
        assert!(report.price_ratio_expensive_vs_cheapest > 1.0);

        let md = report.to_markdown_table();
        assert!(md.contains("Cross-Model Inference Cost Intelligence"));
        assert!(md.contains("Standard Cost"));
    }

    #[test]
    fn test_cost_guardrail_budget_check() {
        let guardrail = CostGuardrail::new(0.10); // $0.10 ceiling

        let cheap = CostBreakdown::calculate(10_000, TargetModel::Gpt4oMini, 500);
        assert!(matches!(
            guardrail.evaluate(&cheap),
            CostGuardrailEvaluation::WithinBudget { .. }
        ));
        assert!(guardrail.check_or_err(&cheap).is_ok());

        let expensive = CostBreakdown::calculate(500_000, TargetModel::Claude3Opus, 5_000);
        let eval = guardrail.evaluate(&expensive);
        assert!(matches!(
            eval,
            CostGuardrailEvaluation::BudgetExceeded { .. }
        ));
        if let CostGuardrailEvaluation::BudgetExceeded {
            suggested_alternatives,
            ..
        } = eval
        {
            assert!(
                suggested_alternatives.contains(&TargetModel::Gpt4oMini)
                    || suggested_alternatives.contains(&TargetModel::Gemini15Flash)
            );
        }

        assert!(guardrail.check_or_err(&expensive).is_err());
    }
}
