//! # Quality Validation & Preservation Metrics
//!
//! Evaluates content fidelity, AST symbol preservation, structural integrity, and
//! token compression ratios between pre- and post-optimization [`CdmDocument`] representations.

use crate::error::ContextOptimizerError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};
use web_repo_to_ai_code_intelligence::{Language, SymbolDeclaration, SymbolExtractor, Visibility};

/// Preservation breakdown for an individual [`SectionKind`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KindPreservation {
    /// Number of sections present in the original document.
    pub original_count: usize,
    /// Number of sections preserved in the optimized document.
    pub retained_count: usize,
    /// Ratio of preserved sections to original sections in this kind (0.0 to 1.0).
    pub retention_ratio: f64,
}

/// Section retention metrics across all document section kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionPreservationMetrics {
    /// Total sections in the original uncompressed document.
    pub original_total: usize,
    /// Total sections retained in the optimized document.
    pub retained_total: usize,
    /// Overall section retention ratio (0.0 to 1.0).
    pub retention_ratio: f64,
    /// Detailed preservation breakdown by [`SectionKind`].
    pub by_kind: BTreeMap<String, KindPreservation>,
}

/// Preservation metrics for mission-critical sections (manifests, security modules, diffs, errors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticalPreservationMetrics {
    /// Total critical sections identified in the original document.
    pub critical_original: usize,
    /// Total critical sections preserved in the optimized document.
    pub critical_retained: usize,
    /// Retention ratio for critical content (0.0 to 1.0). Target: 1.0.
    pub critical_retention_ratio: f64,
    /// Paths or identifiers of critical sections omitted due to severe budget constraints.
    pub critical_omissions: Vec<String>,
}

/// AST symbol and interface declaration preservation metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolPreservationMetrics {
    /// Total AST symbols extracted from original source code sections.
    pub symbols_original_count: usize,
    /// Total AST symbols retained in optimized source code sections.
    pub symbols_retained_count: usize,
    /// Overall symbol preservation ratio (0.0 to 1.0).
    pub symbol_retention_ratio: f64,
    /// Publicly exported / accessible symbols in the original source code.
    pub public_symbols_original: usize,
    /// Publicly exported / accessible symbols preserved in the optimized code.
    pub public_symbols_retained: usize,
    /// Ratio of preserved public interface symbols (0.0 to 1.0).
    pub public_symbol_retention_ratio: f64,
    /// Names of symbols that were omitted or truncated away.
    pub omitted_symbol_names: Vec<String>,
}

/// Token reduction and conservation metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenPreservationMetrics {
    /// Estimated or exact tokens in the original document.
    pub original_tokens: usize,
    /// Tokens in the optimized document.
    pub retained_tokens: usize,
    /// Ratio of retained tokens to original tokens (0.0 to 1.0).
    pub token_retention_ratio: f64,
    /// Percentage reduction in token volume.
    pub token_reduction_pct: f64,
}

/// Structural and syntax integrity indicators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralIntegrityMetrics {
    /// Whether all retained code sections possess valid structural boundaries and UTF-8 encodings.
    pub syntax_intact: bool,
    /// Count of corrupt or malformed sections (0 in any passing optimization).
    pub corrupt_sections_count: usize,
    /// Whether all truncated sections contain valid omission markers.
    pub clean_truncation_boundaries: bool,
    /// Number of sections that were shortened via boundary truncation.
    pub truncated_sections_count: usize,
}

/// Configuration parameters for composite fidelity evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreservationConfig {
    /// Weight assigned to critical section retention in composite score.
    pub critical_weight: f64,
    /// Weight assigned to public symbol preservation in composite score.
    pub public_symbols_weight: f64,
    /// Weight assigned to overall symbol preservation in composite score.
    pub overall_symbols_weight: f64,
    /// Weight assigned to general section retention in composite score.
    pub section_retention_weight: f64,
    /// Minimum composite fidelity score required for quality acceptance (0.0 to 1.0).
    pub min_acceptable_fidelity: f64,
}

impl Default for PreservationConfig {
    fn default() -> Self {
        Self {
            critical_weight: 0.35,
            public_symbols_weight: 0.30,
            overall_symbols_weight: 0.15,
            section_retention_weight: 0.20,
            min_acceptable_fidelity: 0.50,
        }
    }
}

impl PreservationConfig {
    /// Creates a new default preservation evaluation configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum acceptable composite fidelity threshold.
    #[must_use]
    pub const fn with_min_acceptable_fidelity(mut self, threshold: f64) -> Self {
        self.min_acceptable_fidelity = threshold;
        self
    }
}

/// Comprehensive report on content preservation, symbol fidelity, and compression quality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreservationReport {
    /// Section retention breakdown.
    pub section_metrics: SectionPreservationMetrics,
    /// Critical section fidelity breakdown.
    pub critical_metrics: CriticalPreservationMetrics,
    /// AST symbol preservation metrics.
    pub symbol_metrics: SymbolPreservationMetrics,
    /// Token volume conservation metrics.
    pub token_metrics: TokenPreservationMetrics,
    /// Structural and syntax integrity indicators.
    pub structural_metrics: StructuralIntegrityMetrics,
    /// Weighted composite information fidelity score in range `[0.0, 1.0]`.
    pub fidelity_score: f64,
    /// Whether the optimization satisfies the configured minimum quality threshold.
    pub is_acceptable: bool,
}

impl PreservationReport {
    /// Formats a GitHub Flavored Markdown summary table of core preservation metrics.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        format!(
            "| Preservation Dimension | Original | Retained | Fidelity Ratio (%) |\n\
             | :--- | :---: | :---: | :---: |\n\
             | **Critical Content** | `{}` | `{}` | **{:.1}%** |\n\
             | **Public API Symbols** | `{}` | `{}` | **{:.1}%** |\n\
             | **All AST Symbols** | `{}` | `{}` | {:.1}% |\n\
             | **Total Sections** | `{}` | `{}` | {:.1}% |\n\
             | **Token Footprint** | `{}` | `{}` | **-{:.1}%** (Reduction) |\n\
             | **Composite Fidelity** | — | — | **{:.1}%** ({}) |\n",
            self.critical_metrics.critical_original,
            self.critical_metrics.critical_retained,
            self.critical_metrics.critical_retention_ratio * 100.0,
            self.symbol_metrics.public_symbols_original,
            self.symbol_metrics.public_symbols_retained,
            self.symbol_metrics.public_symbol_retention_ratio * 100.0,
            self.symbol_metrics.symbols_original_count,
            self.symbol_metrics.symbols_retained_count,
            self.symbol_metrics.symbol_retention_ratio * 100.0,
            self.section_metrics.original_total,
            self.section_metrics.retained_total,
            self.section_metrics.retention_ratio * 100.0,
            self.token_metrics.original_tokens,
            self.token_metrics.retained_tokens,
            self.token_metrics.token_reduction_pct,
            self.fidelity_score * 100.0,
            if self.is_acceptable {
                "✅ `ACCEPTED`"
            } else {
                "⚠️ `BELOW_THRESHOLD`"
            },
        )
    }

    /// Formats a complete multi-section GitHub Flavored Markdown report.
    #[must_use]
    pub fn to_markdown_report(&self) -> String {
        let mut out = String::new();
        out.push_str("## Quality Validation & Information Preservation Report\n\n");
        out.push_str(&format!(
            "**Composite Fidelity:** `{:.1}%` | **Status:** {} | **Syntax Intact:** {}\n\n",
            self.fidelity_score * 100.0,
            if self.is_acceptable {
                "✅ `QUALITY_ACCEPTED`"
            } else {
                "⚠️ `BELOW_MINIMUM_THRESHOLD`"
            },
            if self.structural_metrics.syntax_intact {
                "✅ `VERIFIED`"
            } else {
                "❌ `CORRUPTION_DETECTED`"
            }
        ));

        out.push_str("### 1. High-Level Preservation Summary\n\n");
        out.push_str(&self.to_markdown_table());
        out.push('\n');

        out.push_str("### 2. Preservation Breakdown by Section Kind\n\n");
        out.push_str("| Section Kind | Original | Retained | Preservation (%) |\n");
        out.push_str("| :--- | :---: | :---: | :---: |\n");
        for (kind, m) in &self.section_metrics.by_kind {
            out.push_str(&format!(
                "| `{:?}` | `{}` | `{}` | {:.1}% |\n",
                kind,
                m.original_count,
                m.retained_count,
                m.retention_ratio * 100.0
            ));
        }
        out.push('\n');

        if !self.critical_metrics.critical_omissions.is_empty() {
            out.push_str("### 3. Critical Section Omission Audit\n\n");
            for path in &self.critical_metrics.critical_omissions {
                out.push_str(&format!("- ⚠️ Omitted critical path: `{path}`\n"));
            }
            out.push('\n');
        }

        if !self.symbol_metrics.omitted_symbol_names.is_empty() {
            out.push_str("### 4. Omitted Symbol Declarations (Sample)\n\n");
            for name in self.symbol_metrics.omitted_symbol_names.iter().take(15) {
                out.push_str(&format!("- 🔸 Symbol omitted: `{name}`\n"));
            }
            if self.symbol_metrics.omitted_symbol_names.len() > 15 {
                out.push_str(&format!(
                    "- *... and {} additional symbols omitted to fit budget.*\n",
                    self.symbol_metrics.omitted_symbol_names.len() - 15
                ));
            }
            out.push('\n');
        }

        out
    }

    /// Serializes the preservation report to formatted JSON.
    pub fn to_json(&self) -> Result<String, ContextOptimizerError> {
        serde_json::to_string_pretty(self).map_err(ContextOptimizerError::Serialization)
    }

    /// Deserializes a preservation report from JSON.
    pub fn from_json(json: &str) -> Result<Self, ContextOptimizerError> {
        serde_json::from_str(json).map_err(ContextOptimizerError::Serialization)
    }
}

/// Preservation metrics evaluation engine.
pub struct PreservationEvaluator;

impl PreservationEvaluator {
    /// Evaluates preservation metrics between original and optimized [`CdmDocument`]s using default configuration.
    pub fn evaluate(
        original: &CdmDocument,
        optimized: &CdmDocument,
    ) -> Result<PreservationReport, ContextOptimizerError> {
        Self::evaluate_with_config(original, optimized, &PreservationConfig::default())
    }

    /// Evaluates preservation metrics between original and optimized [`CdmDocument`]s using a custom [`PreservationConfig`].
    pub fn evaluate_with_config(
        original: &CdmDocument,
        optimized: &CdmDocument,
        config: &PreservationConfig,
    ) -> Result<PreservationReport, ContextOptimizerError> {
        if original.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        // 1. Evaluate section preservation breakdown
        let original_total = original.sections.len();
        let retained_total = optimized.sections.len();
        let retention_ratio = if original_total > 0 {
            retained_total as f64 / original_total as f64
        } else {
            0.0
        };

        let mut orig_kind_counts: BTreeMap<String, usize> = BTreeMap::new();
        for s in &original.sections {
            *orig_kind_counts
                .entry(s.kind.as_str().to_string())
                .or_insert(0) += 1;
        }

        let mut ret_kind_counts: BTreeMap<String, usize> = BTreeMap::new();
        for s in &optimized.sections {
            *ret_kind_counts
                .entry(s.kind.as_str().to_string())
                .or_insert(0) += 1;
        }

        let mut by_kind = BTreeMap::new();
        for (kind_str, &orig_count) in &orig_kind_counts {
            let ret_count = ret_kind_counts.get(kind_str).copied().unwrap_or(0);
            let ratio = if orig_count > 0 {
                ret_count as f64 / orig_count as f64
            } else {
                0.0
            };
            by_kind.insert(
                kind_str.clone(),
                KindPreservation {
                    original_count: orig_count,
                    retained_count: ret_count,
                    retention_ratio: ratio,
                },
            );
        }

        let section_metrics = SectionPreservationMetrics {
            original_total,
            retained_total,
            retention_ratio,
            by_kind,
        };

        // 2. Evaluate critical section preservation
        let mut critical_original = 0;
        let mut critical_retained = 0;
        let mut critical_omissions = Vec::new();

        let retained_ids: HashSet<&str> = optimized
            .sections
            .iter()
            .map(|s| s.section_id.as_str())
            .collect();
        let retained_paths: HashSet<&str> = optimized
            .sections
            .iter()
            .filter_map(|s| s.original_path.as_deref())
            .collect();

        for s in &original.sections {
            if Self::is_critical_section(s) {
                critical_original += 1;
                let path = s.original_path.as_deref().unwrap_or("");
                let is_retained = retained_ids.contains(s.section_id.as_str())
                    || (!path.is_empty() && retained_paths.contains(path));

                if is_retained {
                    critical_retained += 1;
                } else {
                    critical_omissions.push(if !path.is_empty() {
                        path.to_string()
                    } else {
                        s.section_id.clone()
                    });
                }
            }
        }

        let critical_retention_ratio = if critical_original > 0 {
            critical_retained as f64 / critical_original as f64
        } else {
            1.0
        };

        let critical_metrics = CriticalPreservationMetrics {
            critical_original,
            critical_retained,
            critical_retention_ratio,
            critical_omissions,
        };

        // 3. Evaluate AST symbol preservation
        let orig_symbols = Self::extract_all_symbols(&original.sections);
        let ret_symbols = Self::extract_all_symbols(&optimized.sections);

        let symbols_original_count = orig_symbols.len();
        let symbols_retained_count = ret_symbols.len();
        let symbol_retention_ratio = if symbols_original_count > 0 {
            symbols_retained_count as f64 / symbols_original_count as f64
        } else {
            1.0
        };

        let public_symbols_original = orig_symbols
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .count();
        let public_symbols_retained = ret_symbols
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .count();
        let public_symbol_retention_ratio = if public_symbols_original > 0 {
            public_symbols_retained as f64 / public_symbols_original as f64
        } else {
            1.0
        };

        let ret_symbol_names: HashSet<&str> = ret_symbols.iter().map(|s| s.name.as_str()).collect();
        let mut omitted_symbol_names = Vec::new();
        for sym in &orig_symbols {
            if !ret_symbol_names.contains(sym.name.as_str())
                && !omitted_symbol_names.contains(&sym.name)
            {
                omitted_symbol_names.push(sym.name.clone());
            }
        }

        let symbol_metrics = SymbolPreservationMetrics {
            symbols_original_count,
            symbols_retained_count,
            symbol_retention_ratio,
            public_symbols_original,
            public_symbols_retained,
            public_symbol_retention_ratio,
            omitted_symbol_names,
        };

        // 4. Evaluate token metrics
        let original_tokens = Self::calculate_document_tokens(original);
        let retained_tokens = Self::calculate_document_tokens(optimized);

        let token_retention_ratio = if original_tokens > 0 {
            retained_tokens as f64 / original_tokens as f64
        } else {
            0.0
        };

        let token_reduction_pct = if original_tokens > 0 {
            (1.0 - token_retention_ratio).max(0.0) * 100.0
        } else {
            0.0
        };

        let token_metrics = TokenPreservationMetrics {
            original_tokens,
            retained_tokens,
            token_retention_ratio,
            token_reduction_pct,
        };

        // 5. Evaluate structural and syntax integrity
        let mut corrupt_sections_count = 0;
        let mut truncated_sections_count = 0;
        let mut clean_truncation_boundaries = true;

        for s in &optimized.sections {
            if s.section_id.is_empty() {
                corrupt_sections_count += 1;
            }

            if s.body.contains("[Truncated:") {
                truncated_sections_count += 1;
                if !s.body.contains("...]") {
                    clean_truncation_boundaries = false;
                }
            }
        }

        let structural_metrics = StructuralIntegrityMetrics {
            syntax_intact: corrupt_sections_count == 0,
            corrupt_sections_count,
            clean_truncation_boundaries,
            truncated_sections_count,
        };

        // 6. Compute composite information fidelity score
        let fidelity_score = (config.critical_weight * critical_retention_ratio)
            + (config.public_symbols_weight * public_symbol_retention_ratio)
            + (config.overall_symbols_weight * symbol_retention_ratio)
            + (config.section_retention_weight * retention_ratio);

        let is_acceptable =
            fidelity_score >= config.min_acceptable_fidelity && structural_metrics.syntax_intact;

        Ok(PreservationReport {
            section_metrics,
            critical_metrics,
            symbol_metrics,
            token_metrics,
            structural_metrics,
            fidelity_score,
            is_acceptable,
        })
    }

    /// Determines whether a section represents mission-critical context.
    fn is_critical_section(s: &CdmSection) -> bool {
        if s.kind == SectionKind::Manifest {
            return true;
        }

        if let Some(pin) = s.attributes.get("pin") {
            if pin == "must_include" || pin == "pinned" {
                return true;
            }
        }

        if s.attributes
            .get("is_changed_file")
            .is_some_and(|v| v == "true")
        {
            return true;
        }

        if s.attributes
            .get("security_relevance")
            .is_some_and(|v| v == "critical")
        {
            return true;
        }

        if s.attributes
            .get("debugging_relevance")
            .is_some_and(|v| v == "critical")
        {
            return true;
        }

        let path = s.original_path.as_deref().unwrap_or("");
        let lower = path.to_ascii_lowercase();
        lower.ends_with("readme.md")
            || lower.ends_with("security.md")
            || lower.ends_with("cargo.toml")
            || lower.ends_with("package.json")
    }

    /// Extracts all AST symbol declarations across a slice of sections.
    fn extract_all_symbols(sections: &[CdmSection]) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for s in sections {
            if s.kind != SectionKind::CodeFile && s.kind != SectionKind::CodeSnippet {
                continue;
            }

            let path = s.original_path.as_deref().unwrap_or("");
            let lang = Language::from_path(path).unwrap_or_else(|| {
                s.language
                    .as_deref()
                    .map(Language::from_identifier)
                    .unwrap_or(Language::Rust)
            });

            let sec_symbols = SymbolExtractor::extract(&lang, &s.body);
            symbols.extend(sec_symbols);
        }

        symbols
    }

    /// Calculates total token volume across all sections in a document.
    fn calculate_document_tokens(doc: &CdmDocument) -> usize {
        doc.sections
            .iter()
            .map(|s| {
                s.token_metrics
                    .as_ref()
                    .and_then(|m| m.exact_tokens)
                    .unwrap_or_else(|| s.body.chars().count().div_ceil(4))
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, RevisionInfo,
        SourceIdentity, SourceType,
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
    fn test_preservation_metrics_identical_documents() {
        let s1 = make_section(
            "s1",
            "Cargo.toml",
            SectionKind::Manifest,
            "[package]\nname = \"repo\"",
            vec![],
        );
        let s2 = make_section(
            "s2",
            "src/lib.rs",
            SectionKind::CodeFile,
            "pub fn public_api() -> bool { true }\nfn internal_helper() {}",
            vec![],
        );

        let doc = make_test_doc(vec![s1, s2]);
        let report = PreservationEvaluator::evaluate(&doc, &doc).expect("Evaluation succeeds");

        assert_eq!(report.section_metrics.retention_ratio, 1.0);
        assert_eq!(report.critical_metrics.critical_retention_ratio, 1.0);
        assert_eq!(report.symbol_metrics.symbol_retention_ratio, 1.0);
        assert_eq!(report.symbol_metrics.public_symbol_retention_ratio, 1.0);
        assert_eq!(report.token_metrics.token_retention_ratio, 1.0);
        assert_eq!(report.token_metrics.token_reduction_pct, 0.0);
        assert!(report.structural_metrics.syntax_intact);
        assert_eq!(report.fidelity_score, 1.0);
        assert!(report.is_acceptable);
    }
}
