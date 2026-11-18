//! # Tokenizer Adapter Trait Abstraction
//!
//! Defines the unified interface for exact BPE and calibrated token counting engines.

use crate::model::{ModelFamily, TargetModel};
use crate::precision::{TokenCountResult, TruncationResult};

/// Common contract for all LLM tokenizer adapters.
pub trait TokenizerAdapter: Send + Sync {
    /// Canonical name of this tokenizer adapter (e.g. `o200k_base`, `cl100k_base`, `claude_bpe`).
    fn name(&self) -> &'static str;

    /// Associated provider or model family.
    fn model_family(&self) -> ModelFamily;

    /// Primary target model supported by this adapter.
    fn target_model(&self) -> TargetModel;

    /// Returns true if this adapter produces mathematically exact BPE token counts.
    fn is_exact(&self) -> bool;

    /// Maximum context window limit in tokens.
    fn context_window_limit(&self) -> usize {
        self.target_model().context_window_limit()
    }

    /// Recommended input budget ceiling in tokens.
    fn recommended_budget_ceiling(&self) -> usize {
        self.target_model().recommended_budget_ceiling()
    }

    /// Evaluates the token count for a text slice.
    fn count_tokens(&self, text: &str) -> TokenCountResult;

    /// Truncates a text slice to fit within `max_tokens` without breaking UTF-8 boundaries.
    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult;
}
