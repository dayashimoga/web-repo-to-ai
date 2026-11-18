//! # Context Optimization Exclusion Explanations and Audit Telemetry
//!
//! Provides structured, human-readable, and machine-actionable rationales for every
//! omitted content section. Identifies the originating subsystem (budget knapsack,
//! profile filtering, deduplication, security), assesses impact severity, and generates
//! actionable remediation advice to re-include content if desired.

use crate::error::ContextOptimizerError;
use crate::plan::{BudgetAction, BudgetPlan};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use web_repo_to_ai_cdm::{CdmDocument, ExclusionReason, ExclusionRecord, SectionKind};
use web_repo_to_ai_tokenizer::TargetModel;

/// Impact severity level of omitting a section from the AI context window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionImpact {
    /// Minimal impact (e.g. cosmetic styling, non-technical prose, duplicate files).
    Low,
    /// Moderate impact (e.g. auxiliary test suites, non-core implementation files).
    Medium,
    /// High impact (e.g. core business logic, API schemas, error propagation paths).
    High,
    /// Critical impact (e.g. manifests, security policies, authentication modules).
    Critical,
}

impl ExclusionImpact {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Formats this impact level as a visual markdown badge.
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Low => "🟢 `LOW`",
            Self::Medium => "🟡 `MEDIUM`",
            Self::High => "🟠 `HIGH`",
            Self::Critical => "🔴 `CRITICAL`",
        }
    }
}

impl fmt::Display for ExclusionImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The pipeline stage or mechanism that triggered the exclusion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionSource {
    /// Excluded by context headroom limit / greedy knapsack capacity.
    BudgetHeadroom,
    /// Excluded by active context profile rules (e.g. cosmetic asset suppression).
    ProfileRule(String),
    /// Excluded by exact or semantic deduplication.
    Deduplication,
    /// Excluded by standard file ignore patterns (.gitignore, .aiignore).
    IgnorePattern,
    /// Excluded due to security restrictions (SSRF, secret leak).
    SecurityPolicy,
    /// Explicit caller or job configuration setting.
    UserConfiguration,
    /// Unknown or generic reason.
    Other(String),
}

impl ExclusionSource {
    /// Returns a human-readable title.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::BudgetHeadroom => "Context Budget Headroom",
            Self::ProfileRule(profile) => profile.as_str(),
            Self::Deduplication => "Content Deduplication",
            Self::IgnorePattern => "Ignore Pattern Rules",
            Self::SecurityPolicy => "Security Policy Enforcement",
            Self::UserConfiguration => "User Job Configuration",
            Self::Other(name) => name.as_str(),
        }
    }
}

impl fmt::Display for ExclusionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Concrete remediation recommendation to re-include an omitted section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediationAdvice {
    /// Short actionable recommendation title.
    pub title: String,
    /// Detailed instruction or configuration change.
    pub action: String,
    /// Specific parameter or CLI flag if applicable.
    pub parameter: Option<String>,
    /// Estimated additional token capacity needed (if budget-related).
    pub additional_tokens_needed: Option<usize>,
}

impl RemediationAdvice {
    /// Derives actionable remediation advice based on exclusion reason and metadata.
    #[must_use]
    pub fn derive(
        reason: ExclusionReason,
        tokens: usize,
        path: Option<&str>,
        details: Option<&str>,
    ) -> Self {
        let path_str = path.unwrap_or("target");
        let details_str = details.unwrap_or("");

        match reason {
            ExclusionReason::SizeLimit => Self {
                title: "Expand context token budget or upgrade model".to_string(),
                action: format!(
                    "The item required {} tokens, exceeding remaining headroom. Allocate at least +{} additional tokens or select a larger context model (e.g. Claude 3.5 Sonnet 200k or Gemini 1.5 Pro 2M).",
                    tokens, tokens
                ),
                parameter: Some("--token-budget".to_string()),
                additional_tokens_needed: Some(tokens),
            },
            ExclusionReason::UserConfig => {
                let lower = details_str.to_ascii_lowercase();
                if lower.contains("cosmetic") {
                    Self {
                        title: "Enable cosmetic asset inclusion".to_string(),
                        action: "Set `include_cosmetic_assets = true` in your profile configuration to retain UI templates, CSS, and styling.".to_string(),
                        parameter: Some("include_cosmetic_assets".to_string()),
                        additional_tokens_needed: Some(tokens),
                    }
                } else if lower.contains("test") {
                    Self {
                        title: "Enable test suite inclusion".to_string(),
                        action: "Set `include_tests = true` or `include_associated_tests = true` in your profile configuration to include unit and integration tests.".to_string(),
                        parameter: Some("include_tests".to_string()),
                        additional_tokens_needed: Some(tokens),
                    }
                } else if lower.contains("doc") || lower.contains("unmodified") {
                    Self {
                        title: "Enable unmodified documentation inclusion".to_string(),
                        action: "Set `include_unmodified_docs = true` in your profile configuration to retain prose documentation.".to_string(),
                        parameter: Some("include_unmodified_docs".to_string()),
                        additional_tokens_needed: Some(tokens),
                    }
                } else {
                    Self {
                        title: "Pin section to force inclusion".to_string(),
                        action: format!(
                            "Add '{}' to `pinned_paths` in your profile configuration to unconditionally include it.",
                            path_str
                        ),
                        parameter: Some("pinned_paths".to_string()),
                        additional_tokens_needed: Some(tokens),
                    }
                }
            }
            ExclusionReason::Duplicate => Self {
                title: "Switch deduplication strategy to mark-only".to_string(),
                action: "This content is byte-identical to another included section. Set deduplication strategy to 'mark_only' if duplicate bodies are required.".to_string(),
                parameter: Some("strategy = DeduplicationStrategy::MarkOnly".to_string()),
                additional_tokens_needed: Some(tokens),
            },
            ExclusionReason::RulePattern => Self {
                title: "Whitelist path in ignore patterns".to_string(),
                action: format!(
                    "Path '{}' matches an active ignore rule (.gitignore or .aiignore). Add '!{}' to .aiignore to whitelist.",
                    path_str, path_str
                ),
                parameter: Some(".aiignore".to_string()),
                additional_tokens_needed: None,
            },
            ExclusionReason::SecurityRestricted => Self {
                title: "Audit security policy restriction".to_string(),
                action: "This item was blocked by security boundaries (SSRF defense, secret leak prevention, or sensitive environment file). Verify safety before permitting.".to_string(),
                parameter: None,
                additional_tokens_needed: None,
            },
            ExclusionReason::BinaryFile => Self {
                title: "Non-text binary asset".to_string(),
                action: "Binary content (images, executables, compiled archives) cannot be embedded directly in textual LLM context windows.".to_string(),
                parameter: None,
                additional_tokens_needed: None,
            },
            ExclusionReason::ExcludedExtension => Self {
                title: "Add file extension to allowed list".to_string(),
                action: format!(
                    "The extension for '{}' is excluded by default (e.g. minified assets, lockfiles). Add it to allowed extensions in extraction config.",
                    path_str
                ),
                parameter: Some("allowed_extensions".to_string()),
                additional_tokens_needed: Some(tokens),
            },
            ExclusionReason::UnsupportedType => Self {
                title: "Unsupported document format".to_string(),
                action: "The document format is currently unsupported by extraction adapters.".to_string(),
                parameter: None,
                additional_tokens_needed: None,
            },
        }
    }
}

/// Structured, human-readable and machine-actionable rationale for an omitted section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionExplanation {
    /// Section or path identifier.
    pub target: String,
    /// Path of the file (if available).
    pub path: Option<String>,
    /// Kind of section (code file, manifest, prose, etc.).
    pub kind: Option<String>,
    /// Primary reason enum.
    pub reason: ExclusionReason,
    /// Subsystem that triggered the exclusion.
    pub source: ExclusionSource,
    /// Detailed diagnostic explanation in plain English.
    pub rationale: String,
    /// Impact severity on AI understanding.
    pub impact: ExclusionImpact,
    /// Token weight of the omitted section.
    pub token_count: usize,
    /// UTF-8 byte size of the omitted section.
    pub byte_size: usize,
    /// Section priority score before omission (if available).
    pub priority_score: Option<f64>,
    /// Actionable remediation advice to include this content if desired.
    pub remediation: RemediationAdvice,
}

impl ExclusionExplanation {
    /// Formats this explanation as a GitHub Flavored Markdown table row.
    #[must_use]
    pub fn to_markdown_row(&self) -> String {
        let path_display = self.path.as_deref().unwrap_or(&self.target);
        let kind_str = self.kind.as_deref().unwrap_or("unknown");
        let score_str = match self.priority_score {
            Some(score) => format!("{:.1}", score),
            None => "—".to_string(),
        };

        format!(
            "| `{}` | `{}` | `{}` | {} | {} | `{}` | `{}` | `{}` | **{}**: {} |",
            path_display,
            kind_str,
            self.reason.as_str(),
            self.source,
            self.impact.badge(),
            self.token_count,
            self.byte_size,
            score_str,
            self.remediation.title,
            self.remediation.action,
        )
    }
}

/// Aggregated summary for a category of exclusions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionCategorySummary {
    /// Category or reason name.
    pub category: String,
    /// Primary reason enum.
    pub reason: ExclusionReason,
    /// Number of items excluded in this category.
    pub item_count: usize,
    /// Total tokens omitted.
    pub total_tokens: usize,
    /// Total UTF-8 bytes omitted.
    pub total_bytes: usize,
    /// Percentage of total excluded tokens represented by this category.
    pub token_share_pct: f64,
    /// Primary remediation recommendation for this category.
    pub primary_remediation: String,
}

impl ExclusionCategorySummary {
    /// Formats this category summary as a Markdown table row.
    #[must_use]
    pub fn to_markdown_row(&self) -> String {
        format!(
            "| **{}** | `{}` | `{}` | `{:.1}%` | `{}` | {} |",
            self.category,
            self.item_count,
            self.total_tokens,
            self.token_share_pct,
            self.total_bytes,
            self.primary_remediation,
        )
    }
}

/// Comprehensive diagnostic report detailing all excluded context sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionReport {
    /// UTC timestamp of report generation.
    pub generated_at: DateTime<Utc>,
    /// Profile name if evaluated under a specific context profile.
    pub profile_name: Option<String>,
    /// Target model if evaluated for a specific context window.
    pub target_model: Option<TargetModel>,
    /// Total number of excluded sections/items.
    pub total_exclusions: usize,
    /// Total omitted tokens across all exclusions.
    pub total_omitted_tokens: usize,
    /// Total omitted UTF-8 bytes across all exclusions.
    pub total_omitted_bytes: usize,
    /// Highest impact level encountered in the report.
    pub highest_impact: ExclusionImpact,
    /// Count of critical or high-impact exclusions.
    pub high_impact_count: usize,
    /// Categorical breakdown of exclusions.
    pub categories: Vec<ExclusionCategorySummary>,
    /// Detailed explanations for each individual exclusion.
    pub explanations: Vec<ExclusionExplanation>,
    /// Summary recommendations to optimize or reconfigure context inclusion.
    pub recommendations: Vec<String>,
}

impl ExclusionReport {
    /// Renders a full GitHub Flavored Markdown diagnostic report card.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();

        out.push_str("## Context Optimization Exclusion Report\n\n");

        if self.total_exclusions == 0 {
            out.push_str("✅ **All context sections were retained — zero omissions.**\n\n");
            out.push_str(&format!(
                "**Generated At:** `{}`  \n**Profile:** `{}`  \n**Target Model:** `{}`\n\n",
                self.generated_at.to_rfc3339(),
                self.profile_name.as_deref().unwrap_or("None (Direct)"),
                self.target_model
                    .map(|m| m.as_str())
                    .unwrap_or("General / Heuristic"),
            ));
            return out;
        }

        out.push_str(&format!(
            "**Generated At:** `{}`  \n**Profile:** `{}`  \n**Target Model:** `{}`  \n**Total Excluded:** `{}` items (`{}` tokens, `{}` bytes)  \n**Highest Impact:** {}  \n**High/Critical Impact Items:** `{}`\n\n",
            self.generated_at.to_rfc3339(),
            self.profile_name.as_deref().unwrap_or("None (Direct)"),
            self.target_model.map(|m| m.as_str()).unwrap_or("General / Heuristic"),
            self.total_exclusions,
            self.total_omitted_tokens,
            self.total_omitted_bytes,
            self.highest_impact.badge(),
            self.high_impact_count,
        ));

        out.push_str("### 1. Exclusion Categories Summary\n\n");
        out.push_str(
            "| Category | Items | Omitted Tokens | Token Share | Omitted Bytes | Recommended Remediation |\n\
             | :--- | :---: | :---: | :---: | :---: | :--- |\n",
        );
        for cat in &self.categories {
            out.push_str(&cat.to_markdown_row());
            out.push('\n');
        }
        out.push('\n');

        out.push_str("### 2. Itemized Exclusion Audit Trail\n\n");
        out.push_str(
            "| Target / Path | Kind | Reason | Source | Impact | Tokens | Bytes | Priority | Recommended Remediation |\n\
             | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |\n",
        );
        for exp in &self.explanations {
            out.push_str(&exp.to_markdown_row());
            out.push('\n');
        }
        out.push('\n');

        if !self.recommendations.is_empty() {
            out.push_str("### 3. Actionable Context Remediation Recommendations\n\n");
            for rec in &self.recommendations {
                out.push_str(&format!("- {}\n", rec));
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

/// Diagnostic engine constructing structured exclusion explanations.
pub struct ExclusionExplainer;

impl ExclusionExplainer {
    /// Assesses impact severity based on section path, kind, priority, and attributes.
    #[must_use]
    pub fn assess_impact(
        path: Option<&str>,
        kind: Option<SectionKind>,
        score: Option<f64>,
        details: Option<&str>,
    ) -> ExclusionImpact {
        let p = path.unwrap_or("").to_ascii_lowercase();
        let d = details.unwrap_or("").to_ascii_lowercase();

        // Critical: Manifests, security policies, core auth modules
        if kind == Some(SectionKind::Manifest)
            || p == "cargo.toml"
            || p == "package.json"
            || p.ends_with("security.md")
            || d.contains("mustinclude")
            || d.contains("critical")
        {
            return ExclusionImpact::Critical;
        }

        // High: Core APIs, error definitions, authentication, high priority score
        if p.contains("api")
            || p.contains("error")
            || p.contains("contract")
            || p.contains("auth")
            || p.contains("crypto")
            || score.is_some_and(|s| s >= 70.0)
        {
            return ExclusionImpact::High;
        }

        // Medium: Test suites, examples, secondary specs, moderate priority
        if p.contains("test")
            || p.contains("fixture")
            || p.contains("example")
            || p.contains("benchmark")
            || score.is_some_and(|s| s >= 40.0)
        {
            return ExclusionImpact::Medium;
        }

        // Low: Cosmetic assets, CSS, layout styling, non-technical docs, duplicates
        ExclusionImpact::Low
    }

    /// Generates structured explanations from a [`BudgetPlan`].
    #[must_use]
    pub fn explain_plan(
        plan: &BudgetPlan,
        profile_name: Option<impl Into<String>>,
    ) -> ExclusionReport {
        let profile = profile_name.map(Into::into);
        let mut explanations = Vec::new();

        for decision in &plan.decisions {
            if let BudgetAction::Exclude {
                tokens,
                reason,
                ref details,
            } = decision.action
            {
                let impact = Self::assess_impact(
                    decision.path.as_deref(),
                    Some(decision.kind),
                    Some(decision.priority.score),
                    Some(details),
                );

                let source = match reason {
                    ExclusionReason::SizeLimit => ExclusionSource::BudgetHeadroom,
                    ExclusionReason::UserConfig => {
                        if let Some(ref prof) = profile {
                            ExclusionSource::ProfileRule(prof.clone())
                        } else {
                            ExclusionSource::UserConfiguration
                        }
                    }
                    ExclusionReason::Duplicate => ExclusionSource::Deduplication,
                    ExclusionReason::RulePattern => ExclusionSource::IgnorePattern,
                    ExclusionReason::SecurityRestricted => ExclusionSource::SecurityPolicy,
                    _ => ExclusionSource::Other(reason.as_str().to_string()),
                };

                let remediation = RemediationAdvice::derive(
                    reason,
                    tokens,
                    decision.path.as_deref(),
                    Some(details),
                );

                explanations.push(ExclusionExplanation {
                    target: decision.section_id.clone(),
                    path: decision.path.clone(),
                    kind: Some(decision.kind.as_str().to_string()),
                    reason,
                    source,
                    rationale: details.clone(),
                    impact,
                    token_count: tokens,
                    byte_size: tokens * 4, // estimate when raw bytes not stored in plan
                    priority_score: Some(decision.priority.score),
                    remediation,
                });
            }
        }

        Self::compile_report(explanations, profile, plan.target_model)
    }

    /// Generates structured explanations by comparing original and optimized documents.
    pub fn explain_documents(
        original: &CdmDocument,
        optimized: &CdmDocument,
        profile_name: Option<impl Into<String>>,
        target_model: Option<TargetModel>,
    ) -> Result<ExclusionReport, ContextOptimizerError> {
        if original.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        let profile = profile_name.map(Into::into);
        let retained_ids: std::collections::HashSet<&str> = optimized
            .sections
            .iter()
            .map(|s| s.section_id.as_str())
            .collect();

        // Index exclusions from optimized document by path or target
        let mut exclusion_records: BTreeMap<&str, &ExclusionRecord> = BTreeMap::new();
        for exc in &optimized.exclusions {
            exclusion_records.insert(exc.original_path.as_str(), exc);
        }

        let mut explanations = Vec::new();

        for sec in &original.sections {
            if !retained_ids.contains(sec.section_id.as_str()) {
                let path = sec.original_path.as_deref().unwrap_or(&sec.section_id);
                let matched_rec = exclusion_records
                    .get(path)
                    .or_else(|| exclusion_records.get(sec.section_id.as_str()));

                let reason = matched_rec
                    .map(|r| r.reason)
                    .unwrap_or(ExclusionReason::SizeLimit);

                let details = matched_rec
                    .and_then(|r| r.details.clone())
                    .unwrap_or_else(|| {
                        "Section was omitted during context optimization to meet token budget headroom.".to_string()
                    });

                let source = match reason {
                    ExclusionReason::SizeLimit => ExclusionSource::BudgetHeadroom,
                    ExclusionReason::UserConfig => {
                        if let Some(ref prof) = profile {
                            ExclusionSource::ProfileRule(prof.clone())
                        } else {
                            ExclusionSource::UserConfiguration
                        }
                    }
                    ExclusionReason::Duplicate => ExclusionSource::Deduplication,
                    ExclusionReason::RulePattern => ExclusionSource::IgnorePattern,
                    ExclusionReason::SecurityRestricted => ExclusionSource::SecurityPolicy,
                    _ => ExclusionSource::Other(reason.as_str().to_string()),
                };

                let tokens = sec
                    .token_metrics
                    .as_ref()
                    .and_then(|m| m.exact_tokens)
                    .unwrap_or_else(|| sec.body.chars().count().div_ceil(4));

                let impact = Self::assess_impact(
                    sec.original_path.as_deref(),
                    Some(sec.kind),
                    None,
                    Some(&details),
                );

                let remediation = RemediationAdvice::derive(
                    reason,
                    tokens,
                    sec.original_path.as_deref(),
                    Some(&details),
                );

                explanations.push(ExclusionExplanation {
                    target: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    kind: Some(sec.kind.as_str().to_string()),
                    reason,
                    source,
                    rationale: details,
                    impact,
                    token_count: tokens,
                    byte_size: sec.body.len(),
                    priority_score: None,
                    remediation,
                });
            }
        }

        Ok(Self::compile_report(explanations, profile, target_model))
    }

    /// Generates structured explanations from document exclusion records directly.
    pub fn explain_document(
        document: &CdmDocument,
    ) -> Result<ExclusionReport, ContextOptimizerError> {
        let mut explanations = Vec::new();

        for exc in &document.exclusions {
            let reason = exc.reason;
            let details = exc.details.clone().unwrap_or_else(|| {
                format!("Item omitted due to exclusion reason: {}", reason.as_str())
            });

            let bytes = exc.byte_size.unwrap_or(0) as usize;
            let tokens = bytes.div_ceil(4);

            let source = match reason {
                ExclusionReason::SizeLimit => ExclusionSource::BudgetHeadroom,
                ExclusionReason::Duplicate => ExclusionSource::Deduplication,
                ExclusionReason::RulePattern => ExclusionSource::IgnorePattern,
                ExclusionReason::SecurityRestricted => ExclusionSource::SecurityPolicy,
                _ => ExclusionSource::Other(reason.as_str().to_string()),
            };

            let impact = Self::assess_impact(Some(&exc.original_path), None, None, Some(&details));
            let remediation =
                RemediationAdvice::derive(reason, tokens, Some(&exc.original_path), Some(&details));

            explanations.push(ExclusionExplanation {
                target: exc.original_path.clone(),
                path: Some(exc.original_path.clone()),
                kind: None,
                reason,
                source,
                rationale: details,
                impact,
                token_count: tokens,
                byte_size: bytes,
                priority_score: None,
                remediation,
            });
        }

        Ok(Self::compile_report(explanations, None, None))
    }

    fn compile_report(
        explanations: Vec<ExclusionExplanation>,
        profile_name: Option<String>,
        target_model: Option<TargetModel>,
    ) -> ExclusionReport {
        let total_exclusions = explanations.len();
        let total_omitted_tokens: usize = explanations.iter().map(|e| e.token_count).sum();
        let total_omitted_bytes: usize = explanations.iter().map(|e| e.byte_size).sum();

        let mut highest_impact = ExclusionImpact::Low;
        let mut high_impact_count = 0;

        for e in &explanations {
            if e.impact > highest_impact {
                highest_impact = e.impact;
            }
            if e.impact == ExclusionImpact::High || e.impact == ExclusionImpact::Critical {
                high_impact_count += 1;
            }
        }

        // Categorize by reason
        let mut category_map: BTreeMap<String, (ExclusionReason, usize, usize, usize, String)> =
            BTreeMap::new();
        for e in &explanations {
            let key = e.reason.as_str().to_string();
            let entry =
                category_map
                    .entry(key)
                    .or_insert((e.reason, 0, 0, 0, e.remediation.title.clone()));
            entry.1 += 1;
            entry.2 += e.token_count;
            entry.3 += e.byte_size;
        }

        let categories: Vec<ExclusionCategorySummary> = category_map
            .into_iter()
            .map(|(category, (reason, count, tokens, bytes, remediation))| {
                let token_share_pct = if total_omitted_tokens > 0 {
                    ((tokens as f64 / total_omitted_tokens as f64 * 100.0) * 10.0).round() / 10.0
                } else {
                    0.0
                };

                ExclusionCategorySummary {
                    category,
                    reason,
                    item_count: count,
                    total_tokens: tokens,
                    total_bytes: bytes,
                    token_share_pct,
                    primary_remediation: remediation,
                }
            })
            .collect();

        let mut recommendations = Vec::new();
        if high_impact_count > 0 {
            recommendations.push(format!(
                "Attention required: {} critical or high-impact sections were omitted. Review itemized audit trail to ensure mission-critical context is retained.",
                high_impact_count
            ));
        }

        for cat in &categories {
            if cat.reason == ExclusionReason::SizeLimit && cat.total_tokens > 0 {
                recommendations.push(format!(
                    "Budget expansion: {} items ({} tokens) exceeded token headroom. Consider increasing budget limit or switching to a larger model.",
                    cat.item_count, cat.total_tokens
                ));
            } else if cat.reason == ExclusionReason::UserConfig {
                recommendations.push(format!(
                    "Profile tuning: {} items were suppressed by profile rules. Adjust profile flags (e.g. include_tests, include_cosmetic_assets) to include them.",
                    cat.item_count
                ));
            }
        }

        ExclusionReport {
            generated_at: Utc::now(),
            profile_name,
            target_model,
            total_exclusions,
            total_omitted_tokens,
            total_omitted_bytes,
            highest_impact,
            high_impact_count,
            categories,
            explanations,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
        RevisionInfo, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn make_test_section(id: &str, path: &str, kind: SectionKind, body: &str) -> CdmSection {
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

    fn make_test_doc(sections: Vec<CdmSection>, exclusions: Vec<ExclusionRecord>) -> CdmDocument {
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
            exclusions,
            completeness: CompletenessReport::default(),
        }
    }

    #[test]
    fn test_exclusion_impact_assessment() {
        assert_eq!(
            ExclusionExplainer::assess_impact(
                Some("Cargo.toml"),
                Some(SectionKind::Manifest),
                None,
                None
            ),
            ExclusionImpact::Critical
        );
        assert_eq!(
            ExclusionExplainer::assess_impact(
                Some("src/auth.rs"),
                Some(SectionKind::CodeFile),
                Some(80.0),
                None
            ),
            ExclusionImpact::High
        );
        assert_eq!(
            ExclusionExplainer::assess_impact(
                Some("tests/unit_test.rs"),
                Some(SectionKind::CodeFile),
                Some(50.0),
                None
            ),
            ExclusionImpact::Medium
        );
        assert_eq!(
            ExclusionExplainer::assess_impact(
                Some("src/styles.css"),
                Some(SectionKind::Prose),
                Some(20.0),
                None
            ),
            ExclusionImpact::Low
        );
    }

    #[test]
    fn test_remediation_advice_derivation() {
        let budget_rem =
            RemediationAdvice::derive(ExclusionReason::SizeLimit, 1200, Some("src/big.rs"), None);
        assert_eq!(budget_rem.additional_tokens_needed, Some(1200));
        assert!(budget_rem.action.contains("Allocate at least +1200"));

        let cosmetic_rem = RemediationAdvice::derive(
            ExclusionReason::UserConfig,
            200,
            Some("src/button.svelte"),
            Some("Cosmetic asset suppressed"),
        );
        assert!(cosmetic_rem
            .action
            .contains("include_cosmetic_assets = true"));

        let dup_rem =
            RemediationAdvice::derive(ExclusionReason::Duplicate, 50, Some("src/dup.rs"), None);
        assert!(dup_rem.action.contains("mark_only"));
    }

    #[test]
    fn test_explain_documents_comparative() {
        let s1 = make_test_section(
            "s1",
            "Cargo.toml",
            SectionKind::Manifest,
            "[package]\nname = \"app\"",
        );
        let s2 = make_test_section(
            "s2",
            "src/auth.rs",
            SectionKind::CodeFile,
            "pub fn auth() -> bool { true }",
        );
        let s3 = make_test_section(
            "s3",
            "src/ui.css",
            SectionKind::Prose,
            ".button { color: red; }",
        );

        let orig = make_test_doc(vec![s1.clone(), s2.clone(), s3.clone()], vec![]);
        let opt = make_test_doc(
            vec![s1, s2],
            vec![ExclusionRecord::new(
                "src/ui.css",
                ExclusionReason::UserConfig,
                None,
                Some("Cosmetic presentation component excluded by profile".to_string()),
                Some(30),
            )],
        );

        let report = ExclusionExplainer::explain_documents(
            &orig,
            &opt,
            Some("SecurityReview"),
            Some(TargetModel::Claude35Sonnet),
        )
        .expect("Explanation succeeds");

        assert_eq!(report.total_exclusions, 1);
        assert_eq!(report.explanations[0].path.as_deref(), Some("src/ui.css"));
        assert_eq!(report.explanations[0].reason, ExclusionReason::UserConfig);
        assert_eq!(report.explanations[0].impact, ExclusionImpact::Low);
        assert_eq!(report.profile_name.as_deref(), Some("SecurityReview"));

        let md = report.to_markdown_summary();
        assert!(md.contains("Context Optimization Exclusion Report"));
        assert!(md.contains("SecurityReview"));
        assert!(md.contains("src/ui.css"));
        assert!(md.contains("include_cosmetic_assets = true"));
    }

    #[test]
    fn test_zero_exclusions_clean_report() {
        let s1 = make_test_section("s1", "src/main.rs", SectionKind::CodeFile, "fn main() {}");
        let doc = make_test_doc(vec![s1], vec![]);

        let report = ExclusionExplainer::explain_documents(&doc, &doc, None::<String>, None)
            .expect("Succeeds on identical");

        assert_eq!(report.total_exclusions, 0);
        assert_eq!(report.highest_impact, ExclusionImpact::Low);
        assert!(report.explanations.is_empty());
        assert!(report.categories.is_empty());

        let md = report.to_markdown_summary();
        assert!(md.contains("zero omissions"));
    }

    #[test]
    fn test_empty_document_fails_closed() {
        let empty = make_test_doc(vec![], vec![]);
        let err = ExclusionExplainer::explain_documents(&empty, &empty, None::<String>, None)
            .expect_err("Should fail closed");
        assert!(matches!(err, ContextOptimizerError::DocumentEmpty));
    }
}
