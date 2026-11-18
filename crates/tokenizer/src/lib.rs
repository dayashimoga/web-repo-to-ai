//! # Tokenizer & Budget Estimator (`web-repo-to-ai-tokenizer`)
//!
//! Provides calibrated BPE tokenization, model-specific context window estimators,
//! and honest precision accounting across target LLM architectures:
//! - [`TokenizerAdapter`] trait abstraction for pluggable model encoders.
//! - Exact BPE tokenization ([`RegexBpeTokenizer`]) for OpenAI, Anthropic, Google, and Meta architectures.
//! - Content-aware heuristic estimation ([`CalibratedHeuristicAdapter`]) with explicit $\pm 7.5\%$ uncertainty bounds.
//! - Centralized adapter factory and lookup ([`TokenizerRegistry`]).
//! - Whole-document token enrichment and budget utilization ([`enrich_document_token_metrics`], [`DocumentTokenSummary`]).

#![warn(missing_docs)]

pub mod adapter;
pub mod adapters;
pub mod document;
pub mod error;
pub mod model;
pub mod precision;
pub mod registry;

pub use adapter::TokenizerAdapter;
pub use adapters::{
    CalibratedHeuristicAdapter, ClaudeAdapter, GeminiAdapter, LlamaAdapter, OpenAiAdapter,
    RegexBpeTokenizer,
};
pub use document::{count_section_tokens, enrich_document_token_metrics, DocumentTokenSummary};
pub use error::TokenizerError;
pub use model::{ModelFamily, TargetModel};
pub use precision::{TokenCountResult, TokenPrecision, TruncationResult};
pub use registry::TokenizerRegistry;

use std::sync::Arc;

/// Convenience function: Counts tokens in a string using a target model's default adapter.
#[must_use]
pub fn count_tokens_for_model(text: &str, model: TargetModel) -> TokenCountResult {
    let registry = TokenizerRegistry::default();
    let adapter = registry
        .get_for_model(model)
        .unwrap_or_else(|| Arc::new(CalibratedHeuristicAdapter::default()));
    adapter.count_tokens(text)
}

/// Convenience function: Truncates a string to fit within `max_tokens` using a target model's default adapter.
#[must_use]
pub fn truncate_tokens_for_model(
    text: &str,
    max_tokens: usize,
    model: TargetModel,
) -> TruncationResult {
    let registry = TokenizerRegistry::default();
    let adapter = registry
        .get_for_model(model)
        .unwrap_or_else(|| Arc::new(CalibratedHeuristicAdapter::default()));
    adapter.truncate_tokens(text, max_tokens)
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
    fn test_tokenizer_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-tokenizer");
    }

    #[test]
    fn test_count_tokens_for_model_convenience() {
        let text = "Modern asynchronous distributed architecture.";
        let res = count_tokens_for_model(text, TargetModel::Gpt4o);
        assert!(res.count > 0);
        assert!(res.precision.is_exact());
    }

    #[test]
    fn test_truncate_tokens_for_model_convenience() {
        let text = "One two three four five six seven eight nine ten.";
        let res = truncate_tokens_for_model(text, 5, TargetModel::Gpt4o);
        assert!(res.was_truncated);
        assert!(res.resulting_tokens <= 5);
    }
}
