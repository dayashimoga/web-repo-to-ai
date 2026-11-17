//! # Tokenizer & Budget Estimator (`web-repo-to-ai-tokenizer`)
//!
//! Provides token counting and budget estimation across target LLM architectures:
//! - Exact BPE tokenization (tiktoken / GPT-4o / Claude / Gemini)
//! - Fast calibrated heuristic estimators with explicit confidence bounds
//! - Token count validation and safety margins

#![warn(missing_docs)]

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
}
