//! # Code Intelligence (`web-repo-to-ai-code-intelligence`)
//!
//! Provides structural code analysis:
//! - Multi-language syntax identification ([`Language`])
//! - AST symbol, signature, and declaration extraction ([`SymbolDeclaration`], [`SymbolExtractor`])
//! - Quantitative complexity and comment metrics ([`CodeMetrics`], [`CodeMetricsCalculator`])
//! - Architectural code role classification ([`CodeNature`], [`CodeNatureClassifier`])
//! - Unified code analysis pipeline ([`CodeAnalyzer`], [`CodeAnalysisResult`])

#![warn(missing_docs)]

pub mod analyzer;
pub mod error;
pub mod language;
pub mod metrics;
pub mod nature;
pub mod symbols;

pub use analyzer::{CodeAnalysisResult, CodeAnalyzer};
pub use error::CodeIntelligenceError;
pub use language::Language;
pub use metrics::{CodeMetrics, CodeMetricsCalculator};
pub use nature::{CodeNature, CodeNatureClassifier};
pub use symbols::{SymbolDeclaration, SymbolExtractor, SymbolKind, Visibility};

/// Convenience function: Analyzes a source code snippet or file.
#[must_use]
pub fn analyze_code(
    path: Option<&str>,
    language_hint: Option<&str>,
    content: &str,
) -> CodeAnalysisResult {
    CodeAnalyzer::analyze(path, language_hint, content)
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_intelligence_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-code-intelligence");
    }

    #[test]
    fn test_analyze_code_convenience_function() {
        let ts_code = "export interface Session { id: string; }";
        let res = analyze_code(Some("types/session.ts"), None, ts_code);
        assert_eq!(res.language, Language::TypeScript);
        assert_eq!(res.symbols.len(), 1);
        assert_eq!(res.symbols[0].name, "Session");
    }
}
