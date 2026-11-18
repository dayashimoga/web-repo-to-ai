//! # Error Types for Tokenizer Architecture
//!
//! Strongly-typed errors encountered during tokenization, adapter resolution, and truncation.

use thiserror::Error;

/// Error type encountered in tokenizer operations.
#[derive(Debug, Error)]
pub enum TokenizerError {
    /// Requested model or tokenizer adapter was not found in registry.
    #[error("Unknown or unregistered tokenizer: {0}")]
    UnknownTokenizer(String),

    /// Requested target model is unsupported or invalid.
    #[error("Unsupported target model: {0}")]
    UnsupportedModel(String),

    /// Truncation could not satisfy constraints.
    #[error("Token truncation failed: {0}")]
    TruncationFailed(String),

    /// Character encoding or string manipulation failure.
    #[error("Encoding or UTF-8 error: {0}")]
    EncodingError(String),

    /// Forbidden special token detected when policy is Disallow.
    #[error("Forbidden special token encountered: {0}")]
    SpecialTokenForbidden(String),

    /// Estimated inference cost exceeds configured dollar budget ceiling.
    #[error("Cost budget exceeded: estimated ${estimated_usd:.4} > max ${budget_usd:.4}")]
    CostBudgetExceeded {
        /// Configured budget ceiling in USD.
        budget_usd: f64,
        /// Calculated estimated cost in USD.
        estimated_usd: f64,
    },
}
