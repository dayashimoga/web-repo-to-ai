//! # Context Optimization Reporting and Telemetry
//!
//! Provides comprehensive audit reports, headroom breakdowns, and JSON/Markdown serialization
//! for context budget evaluation and token conservation tracking.

use crate::budget::{BudgetHeadroomSummary, ContextBudget};
use crate::error::ContextOptimizerError;
use crate::plan::BudgetPlan;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Comprehensive audit report summarizing context budget allocation and section decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetReport {
    /// Timestamp when this report was evaluated.
    pub generated_at: DateTime<Utc>,
    /// Headroom allocation summary (system, generation, query, content).
    pub headroom: BudgetHeadroomSummary,
    /// Detailed optimization plan and section audit ledger.
    pub plan: BudgetPlan,
}

impl ContextBudgetReport {
    /// Constructs a report from a [`BudgetPlan`] and its corresponding [`ContextBudget`].
    #[must_use]
    pub fn from_plan_and_budget(plan: BudgetPlan, budget: &ContextBudget) -> Self {
        Self {
            generated_at: Utc::now(),
            headroom: budget.headroom_summary(),
            plan,
        }
    }

    /// Renders a complete GitHub Flavored Markdown report card.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();

        out.push_str("## Context Budget Optimization Report\n\n");
        out.push_str(&format!(
            "**Generated At:** `{}`  \n**Status:** {}  \n**Budget Utilization:** `{:.1}%`\n\n",
            self.generated_at.to_rfc3339(),
            if self.plan.is_within_budget() {
                "✅ `WITHIN_BUDGET`"
            } else {
                "⚠️ `BUDGET_EXCEEDED`"
            },
            self.plan.budget_utilization_pct
        ));

        out.push_str("### 1. Context Headroom Allocations\n\n");
        out.push_str(&self.headroom.to_markdown_table());
        out.push('\n');

        out.push_str("### 2. Token Conservation Ledger\n\n");
        out.push_str(&format!(
            "| Metric | Value |\n\
             | :--- | :---: |\n\
             | Input Document Tokens | `{}` |\n\
             | Fitted Content Tokens | `{}` |\n\
             | Total Content Capacity | `{}` |\n\
             | Included Sections | `{}` / `{}` |\n\
             | Truncated Sections | `{}` |\n\
             | Excluded Sections | `{}` |\n\n",
            self.plan.original_document_tokens,
            self.plan.fitted_tokens,
            self.plan.available_content_tokens,
            self.plan.included_section_count,
            self.plan.original_section_count,
            self.plan.truncated_section_count,
            self.plan.excluded_section_count,
        ));

        out.push_str("### 3. Section Decisions\n\n");
        out.push_str(&self.plan.to_markdown_table());

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

#[cfg(test)]
mod tests {
    use super::*;
    use web_repo_to_ai_tokenizer::TargetModel;

    #[test]
    fn test_report_json_roundtrip_and_markdown() {
        let budget = ContextBudget::from_model(TargetModel::Gpt4o);
        let plan = BudgetPlan {
            target_model: Some(TargetModel::Gpt4o),
            total_budget_tokens: 128_000,
            available_content_tokens: 120_404,
            total_reserved_tokens: 7_596,
            original_document_tokens: 10_000,
            fitted_tokens: 10_000,
            budget_utilization_pct: 8.3,
            original_section_count: 5,
            included_section_count: 5,
            truncated_section_count: 0,
            excluded_section_count: 0,
            decisions: Vec::new(),
        };

        let report = ContextBudgetReport::from_plan_and_budget(plan, &budget);
        let json = report.to_json().expect("JSON serialization succeeds");
        let decoded = ContextBudgetReport::from_json(&json).expect("JSON deserialization succeeds");

        assert_eq!(report.plan.fitted_tokens, decoded.plan.fitted_tokens);
        assert_eq!(report.headroom.total_tokens, decoded.headroom.total_tokens);

        let md = report.to_markdown_summary();
        assert!(md.contains("WITHIN_BUDGET"));
        assert!(md.contains("Context Headroom Allocations"));
    }
}
