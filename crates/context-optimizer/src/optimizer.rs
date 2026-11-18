//! # Context Budget Optimizer Engine
//!
//! Applies greedy knapsack importance prioritization and deterministic boundary truncation
//! to fit [`CdmDocument`] content into strict LLM context limits with zero silent drops.

use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::plan::{BudgetAction, BudgetPlan, SectionBudgetDecision};
use crate::priority::{PinStatus, SectionPriority, SectionScorer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, ExclusionReason, ExclusionRecord, SectionTokenMetrics,
};
use web_repo_to_ai_tokenizer::{
    CalibratedHeuristicAdapter, TargetModel, TokenizerAdapter, TokenizerRegistry,
};

/// Truncation strategy applied when a section must be shortened to fit budget headroom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruncationStrategy {
    /// Preserves the beginning (head) and end (tail) with a contextual omission marker in the middle.
    HeadAndTail,
    /// Preserves only the beginning (head) of the content up to the allocated token limit.
    HeadOnly,
    /// Disallows truncation entirely; sections must either be included wholly or excluded.
    NoTruncation,
}

/// Configuration settings controlling context budget optimization.
#[derive(Debug, Clone)]
pub struct BudgetOptimizationConfig {
    /// Context window limits and headroom allocations.
    pub budget: ContextBudget,
    /// Whether individual sections may be truncated to fit remaining headroom.
    pub allow_truncation: bool,
    /// Strategy to use when truncating sections.
    pub truncation_strategy: TruncationStrategy,
    /// Minimum tokens required to justify truncating a section rather than excluding it entirely.
    pub min_retained_tokens: usize,
    /// Section importance and priority scoring engine.
    pub scorer: SectionScorer,
}

impl BudgetOptimizationConfig {
    /// Creates a configuration with default settings for a given [`ContextBudget`].
    #[must_use]
    pub fn new(budget: ContextBudget) -> Self {
        Self {
            budget,
            allow_truncation: true,
            truncation_strategy: TruncationStrategy::HeadAndTail,
            min_retained_tokens: 50,
            scorer: SectionScorer::new(),
        }
    }

    /// Creates a configuration bound directly to a target model architecture.
    #[must_use]
    pub fn for_model(model: TargetModel) -> Self {
        Self::new(ContextBudget::from_model(model))
    }

    /// Sets whether truncation is allowed.
    #[must_use]
    pub const fn with_allow_truncation(mut self, allow: bool) -> Self {
        self.allow_truncation = allow;
        self
    }

    /// Sets the truncation strategy.
    #[must_use]
    pub const fn with_truncation_strategy(mut self, strategy: TruncationStrategy) -> Self {
        self.truncation_strategy = strategy;
        self
    }

    /// Sets the minimum retained token floor for truncation.
    #[must_use]
    pub const fn with_min_retained_tokens(mut self, min: usize) -> Self {
        self.min_retained_tokens = min;
        self
    }

    /// Sets the section importance scorer.
    #[must_use]
    pub fn with_scorer(mut self, scorer: SectionScorer) -> Self {
        self.scorer = scorer;
        self
    }
}

/// Internal item tracking a scored section during budget allocation.
struct ScoredCandidate<'a> {
    original_index: usize,
    section: &'a CdmSection,
    priority: SectionPriority,
    tokens: usize,
    utility_per_token: f64,
}

/// Context budget optimizer engine.
#[derive(Debug, Clone)]
pub struct ContextBudgetOptimizer {
    config: BudgetOptimizationConfig,
}

impl ContextBudgetOptimizer {
    /// Creates a new optimizer instance with the specified configuration.
    #[must_use]
    pub const fn new(config: BudgetOptimizationConfig) -> Self {
        Self { config }
    }

    /// Creates an optimizer configured for a target model.
    #[must_use]
    pub fn for_model(model: TargetModel) -> Self {
        Self::new(BudgetOptimizationConfig::for_model(model))
    }

    /// Optimizes a [`CdmDocument`] to fit strictly within the configured context budget.
    ///
    /// Returns the optimized document (with excluded sections auditable in `exclusions`,
    /// updated cryptographic hashes, and token metrics) alongside the complete [`BudgetPlan`].
    pub fn optimize(
        &self,
        document: &CdmDocument,
    ) -> Result<(CdmDocument, BudgetPlan), ContextOptimizerError> {
        if document.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        self.config.budget.validate()?;

        // Resolve tokenizer adapter
        let adapter: Arc<dyn TokenizerAdapter> =
            if let Some(model) = self.config.budget.target_model {
                let registry = TokenizerRegistry::default();
                registry
                    .get_for_model(model)
                    .unwrap_or_else(|| Arc::new(CalibratedHeuristicAdapter::default()))
            } else {
                Arc::new(CalibratedHeuristicAdapter::default())
            };

        // Score and measure each section
        let mut scored_candidates: Vec<ScoredCandidate<'_>> =
            Vec::with_capacity(document.sections.len());
        let mut original_document_tokens = 0;

        for (idx, sec) in document.sections.iter().enumerate() {
            let tokens = adapter.count_tokens(&sec.body).count;
            original_document_tokens += tokens;

            let priority = self.config.scorer.score(sec);
            let utility_per_token = (priority.score * 1000.0) / (tokens.max(1) as f64);

            scored_candidates.push(ScoredCandidate {
                original_index: idx,
                section: sec,
                priority,
                tokens,
                utility_per_token,
            });
        }

        // Partition candidates into MustInclude, Normal, and MustExclude
        let mut must_include: Vec<ScoredCandidate<'_>> = Vec::new();
        let mut normal_candidates: Vec<ScoredCandidate<'_>> = Vec::new();
        let mut must_exclude: Vec<ScoredCandidate<'_>> = Vec::new();

        for candidate in scored_candidates {
            match candidate.priority.pin_status {
                PinStatus::MustInclude => must_include.push(candidate),
                PinStatus::Normal | PinStatus::Deprioritized => normal_candidates.push(candidate),
                PinStatus::MustExclude => must_exclude.push(candidate),
            }
        }

        // Sort MustInclude by priority score descending
        must_include.sort_by(|a, b| {
            b.priority
                .score
                .partial_cmp(&a.priority.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sort Normal/Deprioritized candidates by utility per token descending (Knapsack greedy)
        normal_candidates.sort_by(|a, b| {
            b.utility_per_token
                .partial_cmp(&a.utility_per_token)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let available_budget = self.config.budget.available_content_tokens();
        let mut remaining_budget = available_budget;
        let mut cumulative_tokens = 0;

        let mut included_sections: Vec<(usize, CdmSection)> = Vec::new();
        let mut exclusions = document.exclusions.clone();
        let mut decisions: Vec<SectionBudgetDecision> = Vec::new();

        let mut included_count = 0;
        let mut truncated_count = 0;
        let mut excluded_count = 0;

        // Phase 1: Allocate MustInclude sections
        for candidate in must_include {
            let sec = candidate.section;
            let tokens = candidate.tokens;

            if tokens <= remaining_budget {
                // Section fits completely
                remaining_budget -= tokens;
                cumulative_tokens += tokens;
                included_count += 1;

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Include,
                });

                included_sections.push((candidate.original_index, sec.clone()));
            } else if self.config.allow_truncation
                && self.config.truncation_strategy != TruncationStrategy::NoTruncation
                && remaining_budget >= self.config.min_retained_tokens
            {
                // Truncate MustInclude section to fit remaining budget
                let truncated_sec = self.truncate_section(sec, remaining_budget, adapter.as_ref());
                let retained_tokens = adapter.count_tokens(&truncated_sec.body).count;
                let omitted = tokens.saturating_sub(retained_tokens);

                remaining_budget = remaining_budget.saturating_sub(retained_tokens);
                cumulative_tokens += retained_tokens;
                truncated_count += 1;

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens: retained_tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Truncate {
                        original_tokens: tokens,
                        retained_tokens,
                        truncated_tokens: omitted,
                        reason: "MustInclude section truncated to fit available budget capacity"
                            .to_string(),
                    },
                });

                included_sections.push((candidate.original_index, truncated_sec));
            } else {
                // Section does not fit even with truncation
                excluded_count += 1;
                let path_str = sec
                    .original_path
                    .clone()
                    .unwrap_or_else(|| sec.section_id.clone());
                let details = format!(
                    "MustInclude section exceeded available budget headroom (required {} tokens, {} available)",
                    tokens, remaining_budget
                );

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Exclude {
                        tokens,
                        reason: ExclusionReason::SizeLimit,
                        details: details.clone(),
                    },
                });

                exclusions.push(ExclusionRecord::new(
                    path_str,
                    ExclusionReason::SizeLimit,
                    None,
                    Some(details),
                    Some(sec.body.len() as u64),
                ));
            }
        }

        // Phase 2: Allocate Normal and Deprioritized candidates by utility per token
        for candidate in normal_candidates {
            let sec = candidate.section;
            let tokens = candidate.tokens;

            if tokens <= remaining_budget {
                // Whole inclusion
                remaining_budget -= tokens;
                cumulative_tokens += tokens;
                included_count += 1;

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Include,
                });

                included_sections.push((candidate.original_index, sec.clone()));
            } else if self.config.allow_truncation
                && self.config.truncation_strategy != TruncationStrategy::NoTruncation
                && remaining_budget >= self.config.min_retained_tokens
                && candidate.priority.pin_status != PinStatus::Deprioritized
            {
                // Partial inclusion via clean truncation
                let truncated_sec = self.truncate_section(sec, remaining_budget, adapter.as_ref());
                let retained_tokens = adapter.count_tokens(&truncated_sec.body).count;
                let omitted = tokens.saturating_sub(retained_tokens);

                remaining_budget = remaining_budget.saturating_sub(retained_tokens);
                cumulative_tokens += retained_tokens;
                truncated_count += 1;

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens: retained_tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Truncate {
                        original_tokens: tokens,
                        retained_tokens,
                        truncated_tokens: omitted,
                        reason: "Section truncated to fit remaining content budget ceiling"
                            .to_string(),
                    },
                });

                included_sections.push((candidate.original_index, truncated_sec));
            } else {
                // Section excluded
                excluded_count += 1;
                let path_str = sec
                    .original_path
                    .clone()
                    .unwrap_or_else(|| sec.section_id.clone());
                let details = format!(
                    "Excluded to fit context budget of {} content tokens (section required {} tokens, priority score: {:.2})",
                    available_budget, tokens, candidate.priority.score
                );

                decisions.push(SectionBudgetDecision {
                    section_id: sec.section_id.clone(),
                    path: sec.original_path.clone(),
                    title: sec.title.clone(),
                    kind: sec.kind,
                    tokens,
                    cumulative_tokens,
                    priority: candidate.priority,
                    utility_per_token: candidate.utility_per_token,
                    action: BudgetAction::Exclude {
                        tokens,
                        reason: ExclusionReason::SizeLimit,
                        details: details.clone(),
                    },
                });

                exclusions.push(ExclusionRecord::new(
                    path_str,
                    ExclusionReason::SizeLimit,
                    None,
                    Some(details),
                    Some(sec.body.len() as u64),
                ));
            }
        }

        // Phase 3: Process MustExclude candidates
        for candidate in must_exclude {
            let sec = candidate.section;
            let tokens = candidate.tokens;
            excluded_count += 1;

            let path_str = sec
                .original_path
                .clone()
                .unwrap_or_else(|| sec.section_id.clone());
            let details = "Section excluded due to explicit pin=must_exclude attribute".to_string();

            decisions.push(SectionBudgetDecision {
                section_id: sec.section_id.clone(),
                path: sec.original_path.clone(),
                title: sec.title.clone(),
                kind: sec.kind,
                tokens,
                cumulative_tokens,
                priority: candidate.priority,
                utility_per_token: candidate.utility_per_token,
                action: BudgetAction::Exclude {
                    tokens,
                    reason: ExclusionReason::UserConfig,
                    details: details.clone(),
                },
            });

            exclusions.push(ExclusionRecord::new(
                path_str,
                ExclusionReason::UserConfig,
                None,
                Some(details),
                Some(sec.body.len() as u64),
            ));
        }

        // Sort included sections back into original document sequence to preserve document structure
        included_sections.sort_by_key(|(idx, _)| *idx);
        let final_sections: Vec<CdmSection> =
            included_sections.into_iter().map(|(_, s)| s).collect();

        // Recompute document cryptographic hash across all remaining sections
        let mut hasher = blake3::Hasher::new();
        for sec in &final_sections {
            hasher.update(sec.content_hash.as_bytes());
        }
        let new_revision_hash = hasher.finalize().to_hex().to_string();

        let mut optimized_doc = document.clone();
        optimized_doc.sections = final_sections;
        optimized_doc.exclusions = exclusions;
        optimized_doc.revision.content_hash_blake3 = Some(new_revision_hash);

        let fitted_tokens = cumulative_tokens;
        let budget_utilization_pct = if available_budget == 0 {
            0.0
        } else {
            (fitted_tokens as f64 / available_budget as f64) * 100.0
        };

        let plan = BudgetPlan {
            target_model: self.config.budget.target_model,
            total_budget_tokens: self.config.budget.total_tokens,
            available_content_tokens: available_budget,
            total_reserved_tokens: self.config.budget.total_reserved_tokens(),
            original_document_tokens,
            fitted_tokens,
            budget_utilization_pct,
            original_section_count: document.sections.len(),
            included_section_count: included_count,
            truncated_section_count: truncated_count,
            excluded_section_count: excluded_count,
            decisions,
        };

        Ok((optimized_doc, plan))
    }

    /// Truncates a section according to the configured truncation strategy.
    fn truncate_section(
        &self,
        section: &CdmSection,
        target_tokens: usize,
        adapter: &dyn TokenizerAdapter,
    ) -> CdmSection {
        let mut truncated = section.clone();

        let mut body = match self.config.truncation_strategy {
            TruncationStrategy::HeadOnly => {
                let marker = "\n\n// ... [Truncated to satisfy context budget ceiling] ...\n";
                let marker_tokens = adapter.count_tokens(marker).count;

                if target_tokens > marker_tokens + 10 {
                    let net_tokens = target_tokens - marker_tokens;
                    let trunc_res = adapter.truncate_tokens(&section.body, net_tokens);
                    format!("{}{}", trunc_res.truncated_text.trim_end(), marker)
                } else {
                    adapter
                        .truncate_tokens(&section.body, target_tokens)
                        .truncated_text
                }
            }
            TruncationStrategy::HeadAndTail => {
                let marker = "\n\n// ... [Truncated: content omitted to fit budget] ...\n\n";
                let marker_tokens = adapter.count_tokens(marker).count;

                if target_tokens > marker_tokens + 20 {
                    let net_tokens = target_tokens - marker_tokens;
                    let head_tokens = (net_tokens * 60 / 100).max(1);
                    let tail_tokens = net_tokens.saturating_sub(head_tokens);

                    let head_res = adapter.truncate_tokens(&section.body, head_tokens);

                    // Collect tail lines
                    let lines: Vec<&str> = section.body.lines().collect();
                    let mut tail_lines = Vec::new();
                    let mut current_tail_tokens = 0;

                    for line in lines.iter().rev() {
                        let line_tokens = adapter.count_tokens(line).count + 1; // + newline
                        if current_tail_tokens + line_tokens > tail_tokens {
                            break;
                        }
                        tail_lines.push(*line);
                        current_tail_tokens += line_tokens;
                    }
                    tail_lines.reverse();
                    let tail_text = tail_lines.join("\n");

                    format!(
                        "{}{}{}",
                        head_res.truncated_text.trim_end(),
                        marker,
                        tail_text.trim_start()
                    )
                } else {
                    adapter
                        .truncate_tokens(&section.body, target_tokens)
                        .truncated_text
                }
            }
            TruncationStrategy::NoTruncation => {
                // If NoTruncation, simply return direct truncation up to target_tokens
                adapter
                    .truncate_tokens(&section.body, target_tokens)
                    .truncated_text
            }
        };

        // Fail-safe post-condition: strictly guarantee resulting tokens <= target_tokens
        let final_count = adapter.count_tokens(&body).count;
        if final_count > target_tokens {
            let hard_trunc = adapter.truncate_tokens(&body, target_tokens);
            body = hard_trunc.truncated_text;
        }

        let resulting_tokens = adapter.count_tokens(&body).count;
        truncated.content_hash = CdmSection::compute_hash(&body);
        truncated.token_metrics = Some(SectionTokenMetrics {
            word_count: body.split_whitespace().count(),
            char_count: body.chars().count(),
            line_count: body.lines().count(),
            estimated_tokens: body.chars().count().div_ceil(4),
            exact_tokens: Some(resulting_tokens),
            tokenizer_name: Some(adapter.name().to_string()),
        });
        truncated.body = body;

        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, RevisionInfo,
        SectionKind, SourceIdentity, SourceType,
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

    fn make_section(
        id: &str,
        path: &str,
        kind: SectionKind,
        body: &str,
        attrs: Vec<(&str, &str)>,
    ) -> CdmSection {
        let mut attributes = BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }

        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(path.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!("https://github.com/org/repo/blob/main/{path}"),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: None,
            attributes,
        }
    }

    #[test]
    fn test_optimize_fits_within_budget_with_exclusion() {
        let sec1 = make_section(
            "s1",
            "README.md",
            SectionKind::Prose,
            "# Project Overview\n\nThis is a universal AI context generator.",
            vec![],
        );
        let sec2 = make_section(
            "s2",
            "src/main.rs",
            SectionKind::CodeFile,
            "fn main() {\n    println!(\"Running\");\n}\n",
            vec![("code_nature", "core_implementation")],
        );
        let sec3 = make_section(
            "s3",
            "tests/huge_test.rs",
            SectionKind::CodeFile,
            &"let x = 1;\n".repeat(500),
            vec![("is_test", "true")],
        );

        let doc = make_test_doc(vec![sec1, sec2, sec3]);

        // Budget with only 100 available content tokens, truncation disabled
        let budget = ContextBudget::new(200, 50, 40, 10);
        let config = BudgetOptimizationConfig::new(budget).with_allow_truncation(false);
        let optimizer = ContextBudgetOptimizer::new(config);

        let (optimized_doc, plan) = optimizer.optimize(&doc).expect("optimization succeeds");

        assert!(plan.is_within_budget());
        assert!(plan.fitted_tokens <= 100);
        assert_eq!(plan.original_section_count, 3);
        assert_eq!(plan.included_section_count, 2);
        assert_eq!(plan.truncated_section_count, 0);
        assert_eq!(plan.excluded_section_count, 1);
        assert_eq!(optimized_doc.exclusions.len(), 1);
        assert_eq!(
            optimized_doc.exclusions[0].reason,
            ExclusionReason::SizeLimit
        );
        assert!(optimized_doc.revision.content_hash_blake3.is_some());
    }

    #[test]
    fn test_optimize_fits_within_budget_with_truncation() {
        let sec1 = make_section(
            "s1",
            "README.md",
            SectionKind::Prose,
            "# Project Overview\n\nThis is a universal AI context generator.",
            vec![],
        );
        let sec2 = make_section(
            "s2",
            "src/main.rs",
            SectionKind::CodeFile,
            "fn main() {\n    println!(\"Running\");\n}\n",
            vec![("code_nature", "core_implementation")],
        );
        let sec3 = make_section(
            "s3",
            "src/utils.rs",
            SectionKind::CodeFile,
            &"pub fn helper() -> i32 { 42 }\n".repeat(200),
            vec![],
        );

        let doc = make_test_doc(vec![sec1, sec2, sec3]);

        // Budget with 100 available content tokens, truncation enabled
        let budget = ContextBudget::new(200, 50, 40, 10);
        let config = BudgetOptimizationConfig::new(budget).with_allow_truncation(true);
        let optimizer = ContextBudgetOptimizer::new(config);

        let (_optimized_doc, plan) = optimizer.optimize(&doc).expect("optimization succeeds");

        assert!(plan.is_within_budget());
        assert!(plan.fitted_tokens <= 100);
        assert_eq!(plan.original_section_count, 3);
        assert_eq!(plan.truncated_section_count, 1);
    }

    #[test]
    fn test_empty_document_rejected() {
        let empty_doc = make_test_doc(vec![]);
        let optimizer = ContextBudgetOptimizer::for_model(TargetModel::Gpt4o);
        assert!(matches!(
            optimizer.optimize(&empty_doc),
            Err(ContextOptimizerError::DocumentEmpty)
        ));
    }
}
