//! # Model-Specific Tokenizer Adapters
//!
//! Strongly-typed adapters for OpenAI, Anthropic, Google, Meta, and Mistral architectures.
//! Delegates directly to dedicated model counting engines (`O200kTokenizer`, `Cl100kTokenizer`,
//! `ClaudeTokenizer`, `GeminiTokenizer`, `LlamaTokenizer`, `MistralTokenizer`).

use crate::adapter::TokenizerAdapter;
use crate::model::{ModelFamily, TargetModel};
use crate::precision::{TokenCountResult, TruncationResult};
use crate::tokenizers::{
    Cl100kTokenizer, ClaudeTokenizer, GeminiTokenizer, LlamaTokenizer, MistralTokenizer,
    O200kTokenizer,
};

/// Adapter for OpenAI models (GPT-4o, GPT-4 Turbo).
#[derive(Debug, Clone, Copy)]
pub struct OpenAiAdapter {
    target_model: TargetModel,
}

impl Default for OpenAiAdapter {
    fn default() -> Self {
        Self {
            target_model: TargetModel::Gpt4o,
        }
    }
}

impl OpenAiAdapter {
    /// Constructs an OpenAI adapter for a specific target model.
    #[must_use]
    pub const fn for_model(target_model: TargetModel) -> Self {
        Self { target_model }
    }
}

impl TokenizerAdapter for OpenAiAdapter {
    fn name(&self) -> &'static str {
        match self.target_model {
            TargetModel::Gpt4o => "o200k_base",
            _ => "cl100k_base",
        }
    }

    fn model_family(&self) -> ModelFamily {
        ModelFamily::OpenAi
    }

    fn target_model(&self) -> TargetModel {
        self.target_model
    }

    fn is_exact(&self) -> bool {
        true
    }

    fn count_tokens(&self, text: &str) -> TokenCountResult {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };

        let count = match self.target_model {
            TargetModel::Gpt4o => O200kTokenizer::count(text),
            _ => Cl100kTokenizer::count(text),
        };

        TokenCountResult::exact(count, self.name(), char_count, word_count, line_count)
    }

    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult {
        let initial_count = self.count_tokens(text);
        if initial_count.count <= max_tokens {
            return TruncationResult {
                truncated_text: text.to_string(),
                original_tokens: initial_count.count,
                resulting_tokens: initial_count.count,
                was_truncated: false,
                omitted_tokens: 0,
            };
        }

        let (truncated_text, resulting_tokens) = match self.target_model {
            TargetModel::Gpt4o => O200kTokenizer::truncate(text, max_tokens),
            _ => Cl100kTokenizer::truncate(text, max_tokens),
        };

        TruncationResult {
            truncated_text,
            original_tokens: initial_count.count,
            resulting_tokens,
            was_truncated: true,
            omitted_tokens: initial_count.count.saturating_sub(resulting_tokens),
        }
    }
}

/// Adapter for Anthropic Claude models (Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku).
#[derive(Debug, Clone, Copy)]
pub struct ClaudeAdapter {
    target_model: TargetModel,
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self {
            target_model: TargetModel::Claude35Sonnet,
        }
    }
}

impl ClaudeAdapter {
    /// Constructs a Claude adapter for a specific target model.
    #[must_use]
    pub const fn for_model(target_model: TargetModel) -> Self {
        Self { target_model }
    }
}

impl TokenizerAdapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude_bpe"
    }

    fn model_family(&self) -> ModelFamily {
        ModelFamily::Anthropic
    }

    fn target_model(&self) -> TargetModel {
        self.target_model
    }

    fn is_exact(&self) -> bool {
        true
    }

    fn count_tokens(&self, text: &str) -> TokenCountResult {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };
        let count = ClaudeTokenizer::count(text);

        TokenCountResult::exact(count, self.name(), char_count, word_count, line_count)
    }

    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult {
        let initial_count = self.count_tokens(text);
        if initial_count.count <= max_tokens {
            return TruncationResult {
                truncated_text: text.to_string(),
                original_tokens: initial_count.count,
                resulting_tokens: initial_count.count,
                was_truncated: false,
                omitted_tokens: 0,
            };
        }

        let (truncated_text, resulting_tokens) = ClaudeTokenizer::truncate(text, max_tokens);

        TruncationResult {
            truncated_text,
            original_tokens: initial_count.count,
            resulting_tokens,
            was_truncated: true,
            omitted_tokens: initial_count.count.saturating_sub(resulting_tokens),
        }
    }
}

/// Adapter for Google Gemini models (Gemini 1.5 Pro, Gemini 1.5 Flash).
#[derive(Debug, Clone, Copy)]
pub struct GeminiAdapter {
    target_model: TargetModel,
}

impl Default for GeminiAdapter {
    fn default() -> Self {
        Self {
            target_model: TargetModel::Gemini15Pro,
        }
    }
}

impl GeminiAdapter {
    /// Constructs a Gemini adapter for a specific target model.
    #[must_use]
    pub const fn for_model(target_model: TargetModel) -> Self {
        Self { target_model }
    }
}

impl TokenizerAdapter for GeminiAdapter {
    fn name(&self) -> &'static str {
        "gemini_sp"
    }

    fn model_family(&self) -> ModelFamily {
        ModelFamily::Google
    }

    fn target_model(&self) -> TargetModel {
        self.target_model
    }

    fn is_exact(&self) -> bool {
        true
    }

    fn count_tokens(&self, text: &str) -> TokenCountResult {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };
        let count = GeminiTokenizer::count(text);

        TokenCountResult::exact(count, self.name(), char_count, word_count, line_count)
    }

    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult {
        let initial_count = self.count_tokens(text);
        if initial_count.count <= max_tokens {
            return TruncationResult {
                truncated_text: text.to_string(),
                original_tokens: initial_count.count,
                resulting_tokens: initial_count.count,
                was_truncated: false,
                omitted_tokens: 0,
            };
        }

        let (truncated_text, resulting_tokens) = GeminiTokenizer::truncate(text, max_tokens);

        TruncationResult {
            truncated_text,
            original_tokens: initial_count.count,
            resulting_tokens,
            was_truncated: true,
            omitted_tokens: initial_count.count.saturating_sub(resulting_tokens),
        }
    }
}

/// Adapter for Meta Llama models (Llama 3, Llama 3.1).
#[derive(Debug, Clone, Copy)]
pub struct LlamaAdapter {
    target_model: TargetModel,
}

impl Default for LlamaAdapter {
    fn default() -> Self {
        Self {
            target_model: TargetModel::Llama3,
        }
    }
}

impl LlamaAdapter {
    /// Constructs a Llama adapter for a specific target model.
    #[must_use]
    pub const fn for_model(target_model: TargetModel) -> Self {
        Self { target_model }
    }
}

impl TokenizerAdapter for LlamaAdapter {
    fn name(&self) -> &'static str {
        "llama3_bpe"
    }

    fn model_family(&self) -> ModelFamily {
        ModelFamily::Meta
    }

    fn target_model(&self) -> TargetModel {
        self.target_model
    }

    fn is_exact(&self) -> bool {
        true
    }

    fn count_tokens(&self, text: &str) -> TokenCountResult {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };
        let count = LlamaTokenizer::count(text);

        TokenCountResult::exact(count, self.name(), char_count, word_count, line_count)
    }

    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult {
        let initial_count = self.count_tokens(text);
        if initial_count.count <= max_tokens {
            return TruncationResult {
                truncated_text: text.to_string(),
                original_tokens: initial_count.count,
                resulting_tokens: initial_count.count,
                was_truncated: false,
                omitted_tokens: 0,
            };
        }

        let (truncated_text, resulting_tokens) = LlamaTokenizer::truncate(text, max_tokens);

        TruncationResult {
            truncated_text,
            original_tokens: initial_count.count,
            resulting_tokens,
            was_truncated: true,
            omitted_tokens: initial_count.count.saturating_sub(resulting_tokens),
        }
    }
}

/// Adapter for Mistral models (Mistral Large, Mistral Small, Mixtral).
#[derive(Debug, Clone, Copy)]
pub struct MistralAdapter {
    target_model: TargetModel,
}

impl Default for MistralAdapter {
    fn default() -> Self {
        Self {
            target_model: TargetModel::MistralLarge,
        }
    }
}

impl MistralAdapter {
    /// Constructs a Mistral adapter for a specific target model.
    #[must_use]
    pub const fn for_model(target_model: TargetModel) -> Self {
        Self { target_model }
    }
}

impl TokenizerAdapter for MistralAdapter {
    fn name(&self) -> &'static str {
        "mistral_tekken"
    }

    fn model_family(&self) -> ModelFamily {
        ModelFamily::Mistral
    }

    fn target_model(&self) -> TargetModel {
        self.target_model
    }

    fn is_exact(&self) -> bool {
        true
    }

    fn count_tokens(&self, text: &str) -> TokenCountResult {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };
        let count = MistralTokenizer::count(text);

        TokenCountResult::exact(count, self.name(), char_count, word_count, line_count)
    }

    fn truncate_tokens(&self, text: &str, max_tokens: usize) -> TruncationResult {
        let initial_count = self.count_tokens(text);
        if initial_count.count <= max_tokens {
            return TruncationResult {
                truncated_text: text.to_string(),
                original_tokens: initial_count.count,
                resulting_tokens: initial_count.count,
                was_truncated: false,
                omitted_tokens: 0,
            };
        }

        let (truncated_text, resulting_tokens) = MistralTokenizer::truncate(text, max_tokens);

        TruncationResult {
            truncated_text,
            original_tokens: initial_count.count,
            resulting_tokens,
            was_truncated: true,
            omitted_tokens: initial_count.count.saturating_sub(resulting_tokens),
        }
    }
}
