//! # Code Analyzer Orchestrator
//!
//! Orchestrates language identification, AST symbol parsing, complexity metrics,
//! and code nature classification in a unified pipeline.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::language::Language;
use crate::metrics::{CodeMetrics, CodeMetricsCalculator};
use crate::nature::{CodeNature, CodeNatureClassifier};
use crate::symbols::{SymbolDeclaration, SymbolExtractor};

/// Comprehensive structural analysis result for a code segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    /// Resolved programming language.
    pub language: Language,
    /// Classified architectural nature of the code.
    pub nature: CodeNature,
    /// Quantitative complexity and structural metrics.
    pub metrics: CodeMetrics,
    /// Extracted declarations (functions, types, traits, interfaces).
    pub symbols: Vec<SymbolDeclaration>,
    /// Compact summary string of declared symbols (e.g. `fn:4,struct:2`).
    pub symbols_summary: String,
    /// True if code is machine-generated or boilerplate.
    pub is_generated: bool,
    /// True if code represents automated test suites or fixtures.
    pub is_test: bool,
    /// True if code is minified.
    pub is_minified: bool,
}

/// Orchestrator for analyzing code content.
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    /// Analyzes a code segment using optional path and language hints.
    #[must_use]
    pub fn analyze(
        path: Option<&str>,
        language_hint: Option<&str>,
        content: &str,
    ) -> CodeAnalysisResult {
        // 1. Resolve language
        let language = if let Some(hint) = language_hint {
            let parsed = Language::from_identifier(hint);
            if matches!(parsed, Language::Unknown(_)) {
                Language::detect(path, content)
            } else {
                parsed
            }
        } else {
            Language::detect(path, content)
        };

        // 2. Extract symbols
        let symbols = SymbolExtractor::extract(&language, content);

        // 3. Compute metrics
        let metrics = CodeMetricsCalculator::compute(&language, content, &symbols);

        // 4. Classify nature
        let nature = CodeNatureClassifier::classify(path, &language, content, &symbols);

        // 5. Generate symbol summary
        let mut symbol_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        for sym in &symbols {
            *symbol_counts.entry(sym.kind.as_str()).or_default() += 1;
        }

        let symbols_summary = if symbol_counts.is_empty() {
            "none".to_string()
        } else {
            symbol_counts
                .iter()
                .map(|(kind, count)| format!("{kind}:{count}"))
                .collect::<Vec<_>>()
                .join(",")
        };

        let is_generated = nature.is_generated_or_boilerplate() || metrics.is_minified;
        let is_test = nature.is_test();
        let is_minified = metrics.is_minified;

        CodeAnalysisResult {
            language,
            nature,
            metrics,
            symbols,
            symbols_summary,
            is_generated,
            is_test,
            is_minified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_analyzer_rust_file() {
        let rust_code = "\
pub struct Config {
    pub max_items: usize,
}

pub fn load_config() -> Config {
    Config { max_items: 100 }
}
";
        let res = CodeAnalyzer::analyze(Some("src/config.rs"), Some("rust"), rust_code);
        assert_eq!(res.language, Language::Rust);
        assert_eq!(res.nature, CodeNature::CoreImplementation);
        assert_eq!(res.symbols.len(), 2);
        assert_eq!(res.symbols_summary, "function:1,struct:1");
        assert!(!res.is_test);
        assert!(!res.is_generated);
        assert!(!res.is_minified);
    }
}
