//! # Error Types for Context Optimization
//!
//! Strongly-typed domain errors emitted during budget calculation, headroom
//! evaluation, section prioritization, and context optimization.

use thiserror::Error;
use web_repo_to_ai_tokenizer::TokenizerError;

/// Core error taxonomy for context optimization operations.
#[derive(Debug, Error)]
pub enum ContextOptimizerError {
    /// The specified budget is structurally invalid.
    #[error("Invalid context budget: {reason}")]
    InvalidBudget {
        /// Explanation of why the budget is invalid.
        reason: String,
    },

    /// The available content tokens budget is smaller than the minimum required threshold.
    #[error(
        "Context budget too small: available content capacity is {available_tokens} tokens, but minimum required is {minimum_required}"
    )]
    BudgetTooSmall {
        /// Effective content tokens available after headroom deduction.
        available_tokens: usize,
        /// Minimum tokens required to include essential document metadata or sections.
        minimum_required: usize,
    },

    /// Attempted to optimize a document with zero sections.
    #[error("Document is empty: no content sections available for optimization")]
    DocumentEmpty,

    /// An upstream tokenizer error occurred during token counting or truncation.
    #[error("Tokenizer error: {0}")]
    Tokenizer(#[from] TokenizerError),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
