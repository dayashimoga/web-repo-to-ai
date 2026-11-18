//! # Model Families & Target Architectures
//!
//! Strongly-typed target LLM models, families, and context window limits.

use serde::{Deserialize, Serialize};

/// High-level provider or architecture family for target language models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    /// OpenAI models (GPT-4o, GPT-4 Turbo, o1).
    OpenAi,
    /// Anthropic models (Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku).
    Anthropic,
    /// Google models (Gemini 1.5 Pro, Gemini 1.5 Flash, Gemini 2.0).
    Google,
    /// Meta models (Llama 3, Llama 3.1, Llama 3.2).
    Meta,
    /// Mistral models (Mistral Large, Codestral).
    Mistral,
    /// Generic or fallback model family.
    Generic,
}

impl ModelFamily {
    /// Returns the canonical snake_case string identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::Meta => "meta",
            Self::Mistral => "mistral",
            Self::Generic => "generic",
        }
    }
}

/// Target language model for context budgeting and token counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetModel {
    /// OpenAI GPT-4o (`o200k_base` vocabulary, 128k context window).
    Gpt4o,
    /// OpenAI GPT-4 Turbo (`cl100k_base` vocabulary, 128k context window).
    Gpt4Turbo,
    /// Anthropic Claude 3.5 Sonnet (200k context window).
    Claude35Sonnet,
    /// Anthropic Claude 3 Opus (200k context window).
    Claude3Opus,
    /// Anthropic Claude 3 Haiku (200k context window).
    Claude3Haiku,
    /// Google Gemini 1.5 Pro (2,000,000 token context window).
    Gemini15Pro,
    /// Google Gemini 1.5 Flash (1,000,000 token context window).
    Gemini15Flash,
    /// Meta Llama 3 / 3.1 8B (128k token context window).
    Llama3,
    /// Meta Llama 3 / 3.1 70B (128k token context window).
    Llama370B,
    /// Mistral Large (128k token context window, Tekken 131k tokenizer).
    MistralLarge,
    /// Mixtral 8x7B (32k token context window).
    Mixtral8x7B,
    /// Generic fallback model (32k token conservative baseline).
    GenericFallback,
}

impl TargetModel {
    /// Returns the canonical snake_case identifier for this target model.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gpt4o => "gpt-4o",
            Self::Gpt4Turbo => "gpt-4-turbo",
            Self::Claude35Sonnet => "claude-3-5-sonnet",
            Self::Claude3Opus => "claude-3-opus",
            Self::Claude3Haiku => "claude-3-haiku",
            Self::Gemini15Pro => "gemini-1-5-pro",
            Self::Gemini15Flash => "gemini-1-5-flash",
            Self::Llama3 => "llama-3",
            Self::Llama370B => "llama-3-70b",
            Self::MistralLarge => "mistral-large",
            Self::Mixtral8x7B => "mixtral-8x7b",
            Self::GenericFallback => "generic-fallback",
        }
    }

    /// Resolves a target model from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let clean = name.trim().to_ascii_lowercase();
        match clean.as_str() {
            "gpt-4o" | "gpt4o" | "o200k" | "o200k_base" => Some(Self::Gpt4o),
            "gpt-4-turbo" | "gpt4-turbo" | "cl100k" | "cl100k_base" | "gpt-4" => {
                Some(Self::Gpt4Turbo)
            }
            "claude-3-5-sonnet" | "claude-3.5-sonnet" | "claude-sonnet" | "claude" => {
                Some(Self::Claude35Sonnet)
            }
            "claude-3-opus" | "claude-opus" => Some(Self::Claude3Opus),
            "claude-3-haiku" | "claude-haiku" => Some(Self::Claude3Haiku),
            "gemini-1-5-pro" | "gemini-1.5-pro" | "gemini-pro" | "gemini" => {
                Some(Self::Gemini15Pro)
            }
            "gemini-1-5-flash" | "gemini-1.5-flash" | "gemini-flash" => Some(Self::Gemini15Flash),
            "llama-3" | "llama-3.1" | "llama3" => Some(Self::Llama3),
            "llama-3-70b" | "llama-3.1-70b" | "llama3-70b" => Some(Self::Llama370B),
            "mistral-large" | "mistral" | "tekken" => Some(Self::MistralLarge),
            "mixtral-8x7b" | "mixtral" => Some(Self::Mixtral8x7B),
            "generic" | "fallback" | "default" => Some(Self::GenericFallback),
            _ => None,
        }
    }

    /// Associated provider or model family.
    #[must_use]
    pub const fn model_family(self) -> ModelFamily {
        match self {
            Self::Gpt4o | Self::Gpt4Turbo => ModelFamily::OpenAi,
            Self::Claude35Sonnet | Self::Claude3Opus | Self::Claude3Haiku => ModelFamily::Anthropic,
            Self::Gemini15Pro | Self::Gemini15Flash => ModelFamily::Google,
            Self::Llama3 | Self::Llama370B => ModelFamily::Meta,
            Self::MistralLarge | Self::Mixtral8x7B => ModelFamily::Mistral,
            Self::GenericFallback => ModelFamily::Generic,
        }
    }

    /// Maximum total context window in tokens supported by the model architecture.
    #[must_use]
    pub const fn context_window_limit(self) -> usize {
        match self {
            Self::Gpt4o | Self::Gpt4Turbo | Self::Llama3 | Self::Llama370B | Self::MistralLarge => {
                128_000
            }
            Self::Claude35Sonnet | Self::Claude3Opus | Self::Claude3Haiku => 200_000,
            Self::Gemini15Pro => 2_000_000,
            Self::Gemini15Flash => 1_000_000,
            Self::Mixtral8x7B | Self::GenericFallback => 32_000,
        }
    }

    /// Maximum output generation tokens supported by the model.
    #[must_use]
    pub const fn max_output_tokens(self) -> usize {
        match self {
            Self::Gpt4o | Self::Gpt4Turbo => 4_096,
            Self::Claude35Sonnet => 8_192,
            Self::Claude3Opus | Self::Claude3Haiku => 4_096,
            Self::Gemini15Pro | Self::Gemini15Flash => 8_192,
            Self::Llama3 | Self::Llama370B => 4_096,
            Self::MistralLarge => 8_192,
            Self::Mixtral8x7B | Self::GenericFallback => 4_096,
        }
    }

    /// Recommended input budget ceiling reserving head-room for system prompts and model generation.
    #[must_use]
    pub fn recommended_budget_ceiling(self) -> usize {
        let total = self.context_window_limit();
        let max_output = self.max_output_tokens();
        // Reserve 10% safety margin plus max output
        let reserve = max_output + (total / 10);
        total.saturating_sub(reserve)
    }

    /// Canonical tokenizer adapter name associated with this target model.
    #[must_use]
    pub const fn default_tokenizer_name(self) -> &'static str {
        match self {
            Self::Gpt4o => "o200k_base",
            Self::Gpt4Turbo => "cl100k_base",
            Self::Claude35Sonnet | Self::Claude3Opus | Self::Claude3Haiku => "claude_bpe",
            Self::Gemini15Pro | Self::Gemini15Flash => "gemini_sp",
            Self::Llama3 | Self::Llama370B => "llama3_bpe",
            Self::MistralLarge | Self::Mixtral8x7B => "mistral_tekken",
            Self::GenericFallback => "calibrated_heuristic",
        }
    }

    /// Returns a slice of all standard supported production models for comparative analysis.
    #[must_use]
    pub const fn supported_models() -> &'static [TargetModel] {
        &[
            TargetModel::Gpt4o,
            TargetModel::Gpt4Turbo,
            TargetModel::Claude35Sonnet,
            TargetModel::Gemini15Pro,
            TargetModel::Llama3,
            TargetModel::MistralLarge,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_model_resolution() {
        assert_eq!(TargetModel::from_name("gpt-4o"), Some(TargetModel::Gpt4o));
        assert_eq!(
            TargetModel::from_name("CL100K_BASE"),
            Some(TargetModel::Gpt4Turbo)
        );
        assert_eq!(
            TargetModel::from_name("claude-3-5-sonnet"),
            Some(TargetModel::Claude35Sonnet)
        );
        assert_eq!(
            TargetModel::from_name("gemini-1.5-pro"),
            Some(TargetModel::Gemini15Pro)
        );
        assert_eq!(TargetModel::from_name("unknown-xyz"), None);
    }

    #[test]
    fn test_context_window_and_budget_ceilings() {
        let gpt4o = TargetModel::Gpt4o;
        assert_eq!(gpt4o.context_window_limit(), 128_000);
        assert!(gpt4o.recommended_budget_ceiling() < 128_000);
        assert!(gpt4o.recommended_budget_ceiling() > 100_000);

        let gemini = TargetModel::Gemini15Pro;
        assert_eq!(gemini.context_window_limit(), 2_000_000);
        assert!(gemini.recommended_budget_ceiling() > 1_500_000);
    }
}
