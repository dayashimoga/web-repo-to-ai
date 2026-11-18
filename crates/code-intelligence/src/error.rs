//! # Error Types for Code Intelligence
//!
//! Strongly-typed error domain for language detection, AST parsing, and symbol extraction.

use thiserror::Error;

/// Error type encountered during code intelligence analysis.
#[derive(Debug, Error)]
pub enum CodeIntelligenceError {
    /// Failure during AST symbol parsing or regex compilation.
    #[error("Symbol extraction failed: {0}")]
    SymbolExtraction(String),

    /// Language detection could not resolve an ambiguous source or payload.
    #[error("Language identification failed: {0}")]
    LanguageIdentification(String),

    /// Content payload was corrupted or invalid UTF-8.
    #[error("Invalid content: {0}")]
    InvalidContent(String),
}
