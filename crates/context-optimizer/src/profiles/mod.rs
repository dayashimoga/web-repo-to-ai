//! # Context Profiles (`web-repo-to-ai-context-optimizer::profiles`)
//!
//! Provides specialized context extraction profiles tailoring Canonical Context Document Model
//! ([`CdmDocument`]) representations to specific AI reasoning tasks:
//! - [`GeneralAiProfile`] ([`ProfileKind::GeneralAi`]): Balanced conceptual and architectural context.
//! - `CodeReviewProfile` ([`ProfileKind::CodeReview`]): Change-focused diffs and touched symbols (Phase 7.3.2).
//! - `SecurityReviewProfile` ([`ProfileKind::SecurityReview`]): Attack surface, auth, and crypto paths (Phase 7.3.3).
//! - `DebuggingProfile` ([`ProfileKind::DebuggingAndDocumentation`]): Error propagation, logs, and API docs (Phase 7.3.4).

pub mod code_review;
pub mod general_ai;
pub mod security_review;

pub use code_review::{CodeReviewConfig, CodeReviewProfile};
pub use general_ai::{GeneralAiConfig, GeneralAiProfile};
pub use security_review::{SecurityCategory, SecurityReviewConfig, SecurityReviewProfile};

use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::plan::BudgetPlan;
use crate::priority::SectionScorer;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};

/// Taxonomy of task-specific context optimization profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    /// Balanced conceptual and architectural context for general reasoning.
    GeneralAi,
    /// Targeted context for reviewing code changes and pull requests.
    CodeReview,
    /// Security-centric context analyzing attack surface, auth, and crypto.
    SecurityReview,
    /// Debugging context emphasizing error paths, logs, and API contracts.
    DebuggingAndDocumentation,
}

impl ProfileKind {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GeneralAi => "general_ai",
            Self::CodeReview => "code_review",
            Self::SecurityReview => "security_review",
            Self::DebuggingAndDocumentation => "debugging_and_documentation",
        }
    }
}

/// Token allocation distribution metrics across content categories for a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileMetrics {
    /// The profile type evaluated.
    pub profile_kind: ProfileKind,
    /// Tokens spent on architectural and prose documentation.
    pub documentation_tokens: usize,
    /// Tokens spent on core implementation source code.
    pub core_code_tokens: usize,
    /// Tokens spent on manifests, configurations, and directory trees.
    pub manifest_tokens: usize,
    /// Tokens spent on test suites and fixtures.
    pub test_tokens: usize,
    /// Tokens spent on other content categories.
    pub other_tokens: usize,
    /// Total tokens across all categories.
    pub total_tokens: usize,
    /// Ratio of documentation tokens to total tokens (0.0 - 1.0).
    pub documentation_ratio: f64,
    /// Ratio of core source code tokens to total tokens (0.0 - 1.0).
    pub core_code_ratio: f64,
    /// Ratio of manifest tokens to total tokens (0.0 - 1.0).
    pub manifest_ratio: f64,
}

impl ProfileMetrics {
    /// Computes content distribution metrics from a slice of included [`CdmSection`]s.
    #[must_use]
    pub fn from_sections(sections: &[CdmSection], profile_kind: ProfileKind) -> Self {
        let mut documentation_tokens = 0;
        let mut core_code_tokens = 0;
        let mut manifest_tokens = 0;
        let mut test_tokens = 0;
        let other_tokens = 0;

        for sec in sections {
            let tokens = sec
                .token_metrics
                .as_ref()
                .and_then(|m| m.exact_tokens)
                .unwrap_or_else(|| sec.body.chars().count().div_ceil(4));

            let path = sec.original_path.as_deref().unwrap_or("");
            let is_test = sec.attributes.get("is_test").is_some_and(|v| v == "true")
                || path.contains("/tests/")
                || path.contains("/test/");

            if is_test {
                test_tokens += tokens;
            } else {
                match sec.kind {
                    SectionKind::Prose | SectionKind::Admonition => {
                        documentation_tokens += tokens;
                    }
                    SectionKind::CodeFile | SectionKind::CodeSnippet => {
                        core_code_tokens += tokens;
                    }
                    SectionKind::Manifest | SectionKind::Frontmatter | SectionKind::Navigation => {
                        manifest_tokens += tokens;
                    }
                    SectionKind::Table => {
                        documentation_tokens += tokens;
                    }
                }
            }
        }

        let total_tokens =
            documentation_tokens + core_code_tokens + manifest_tokens + test_tokens + other_tokens;

        let (doc_ratio, code_ratio, man_ratio) = if total_tokens > 0 {
            (
                documentation_tokens as f64 / total_tokens as f64,
                core_code_tokens as f64 / total_tokens as f64,
                manifest_tokens as f64 / total_tokens as f64,
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        Self {
            profile_kind,
            documentation_tokens,
            core_code_tokens,
            manifest_tokens,
            test_tokens,
            other_tokens,
            total_tokens,
            documentation_ratio: doc_ratio,
            core_code_ratio: code_ratio,
            manifest_ratio: man_ratio,
        }
    }

    /// Renders a GitHub Flavored Markdown summary table of profile distribution.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        format!(
            "| Content Category | Tokens | Share (%) |\n\
             | :--- | :---: | :---: |\n\
             | Documentation & Prose | `{}` | {:.1}% |\n\
             | Core Implementation Code | `{}` | {:.1}% |\n\
             | Manifests & Architecture | `{}` | {:.1}% |\n\
             | Tests & Fixtures | `{}` | {:.1}% |\n\
             | Other Content | `{}` | {:.1}% |\n\
             | **Total Fitted Tokens** | `{}` | **100.0%** |\n",
            self.documentation_tokens,
            self.documentation_ratio * 100.0,
            self.core_code_tokens,
            self.core_code_ratio * 100.0,
            self.manifest_tokens,
            self.manifest_ratio * 100.0,
            self.test_tokens,
            if self.total_tokens > 0 {
                (self.test_tokens as f64 / self.total_tokens as f64) * 100.0
            } else {
                0.0
            },
            self.other_tokens,
            if self.total_tokens > 0 {
                (self.other_tokens as f64 / self.total_tokens as f64) * 100.0
            } else {
                0.0
            },
            self.total_tokens,
        )
    }
}

/// Comprehensive outcome of applying a context profile to a [`CdmDocument`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileOptimizationResult {
    /// Classification of the profile used.
    pub profile_kind: ProfileKind,
    /// Optimized document satisfying the profile and context budget.
    pub document: CdmDocument,
    /// Detailed knapsack allocation and audit plan.
    pub plan: BudgetPlan,
    /// Profile content category token distribution metrics.
    pub metrics: ProfileMetrics,
}

impl ProfileOptimizationResult {
    /// Renders a complete GitHub Flavored Markdown report of the profile optimization.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "## Context Profile Optimization: `{}`\n\n",
            self.profile_kind.as_str()
        ));
        out.push_str(&format!(
            "**Status:** {}  \n**Budget Utilization:** `{:.1}%` (`{}` / `{}` tokens)\n\n",
            if self.plan.is_within_budget() {
                "✅ `WITHIN_BUDGET`"
            } else {
                "⚠️ `BUDGET_EXCEEDED`"
            },
            self.plan.budget_utilization_pct,
            self.plan.fitted_tokens,
            self.plan.available_content_tokens,
        ));

        out.push_str("### 1. Token Distribution by Category\n\n");
        out.push_str(&self.metrics.to_markdown_table());
        out.push('\n');

        out.push_str("### 2. Section Budget Decisions\n\n");
        out.push_str(&self.plan.to_markdown_table());

        out
    }

    /// Serializes the result to a formatted JSON string.
    pub fn to_json(&self) -> Result<String, ContextOptimizerError> {
        serde_json::to_string_pretty(self).map_err(ContextOptimizerError::Serialization)
    }

    /// Deserializes a result from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ContextOptimizerError> {
        serde_json::from_str(json).map_err(ContextOptimizerError::Serialization)
    }
}

/// Abstract contract for task-specific context optimization profiles.
pub trait ContextProfile: Send + Sync {
    /// Returns the unique classification of this profile.
    fn profile_kind(&self) -> ProfileKind;

    /// Constructs the customized [`SectionScorer`] for this profile.
    fn create_scorer(&self) -> SectionScorer;

    /// Pre-processes or enriches sections prior to budget allocation.
    fn prepare_sections(&self, sections: &[CdmSection]) -> Vec<CdmSection>;

    /// Applies the profile to optimize a [`CdmDocument`] within a [`ContextBudget`].
    fn optimize(
        &self,
        document: &CdmDocument,
        budget: &ContextBudget,
    ) -> Result<ProfileOptimizationResult, ContextOptimizerError>;
}
