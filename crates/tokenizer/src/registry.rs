//! # Tokenizer Registry & Factory
//!
//! Centralized repository of thread-safe tokenizer adapters for resolving adapters
//! by target model, family, or canonical identifier.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::adapter::TokenizerAdapter;
use crate::adapters::{
    CalibratedHeuristicAdapter, ClaudeAdapter, GeminiAdapter, LlamaAdapter, MistralAdapter,
    OpenAiAdapter,
};
use crate::model::{ModelFamily, TargetModel};

/// Central registry holding available tokenizer adapter implementations.
#[derive(Clone)]
pub struct TokenizerRegistry {
    adapters_by_model: BTreeMap<TargetModel, Arc<dyn TokenizerAdapter>>,
    adapters_by_name: BTreeMap<String, Arc<dyn TokenizerAdapter>>,
}

impl Default for TokenizerRegistry {
    fn default() -> Self {
        let mut registry = Self {
            adapters_by_model: BTreeMap::new(),
            adapters_by_name: BTreeMap::new(),
        };

        // 1. OpenAI
        registry.register(Arc::new(OpenAiAdapter::for_model(TargetModel::Gpt4o)));
        registry.register(Arc::new(OpenAiAdapter::for_model(TargetModel::Gpt4Turbo)));

        // 2. Anthropic Claude
        registry.register(Arc::new(ClaudeAdapter::for_model(
            TargetModel::Claude35Sonnet,
        )));
        registry.register(Arc::new(ClaudeAdapter::for_model(TargetModel::Claude3Opus)));
        registry.register(Arc::new(ClaudeAdapter::for_model(
            TargetModel::Claude3Haiku,
        )));

        // 3. Google Gemini
        registry.register(Arc::new(GeminiAdapter::for_model(TargetModel::Gemini15Pro)));
        registry.register(Arc::new(GeminiAdapter::for_model(
            TargetModel::Gemini15Flash,
        )));

        // 4. Meta Llama
        registry.register(Arc::new(LlamaAdapter::for_model(TargetModel::Llama3)));
        registry.register(Arc::new(LlamaAdapter::for_model(TargetModel::Llama370B)));

        // 5. Mistral
        registry.register(Arc::new(MistralAdapter::for_model(
            TargetModel::MistralLarge,
        )));
        registry.register(Arc::new(MistralAdapter::for_model(
            TargetModel::Mixtral8x7B,
        )));

        // 6. Calibrated heuristic fallback
        registry.register(Arc::new(CalibratedHeuristicAdapter::default()));

        registry
    }
}

impl TokenizerRegistry {
    /// Creates a new empty tokenizer registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapters_by_model: BTreeMap::new(),
            adapters_by_name: BTreeMap::new(),
        }
    }

    /// Registers a tokenizer adapter into the registry.
    pub fn register(&mut self, adapter: Arc<dyn TokenizerAdapter>) {
        self.adapters_by_model
            .insert(adapter.target_model(), Arc::clone(&adapter));
        self.adapters_by_name
            .insert(adapter.name().to_string(), Arc::clone(&adapter));
    }

    /// Retrieves an adapter specifically for the given [`TargetModel`].
    #[must_use]
    pub fn get_for_model(&self, model: TargetModel) -> Option<Arc<dyn TokenizerAdapter>> {
        self.adapters_by_model.get(&model).cloned().or_else(|| {
            // Fallback by family
            self.adapters_by_model
                .values()
                .find(|a| a.model_family() == model.model_family())
                .cloned()
        })
    }

    /// Retrieves an adapter by its canonical name string (e.g. `o200k_base`, `claude_bpe`).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn TokenizerAdapter>> {
        let clean = name.trim().to_ascii_lowercase();
        self.adapters_by_name.get(&clean).cloned().or_else(|| {
            if let Some(target) = TargetModel::from_name(name) {
                self.get_for_model(target)
            } else {
                None
            }
        })
    }

    /// Retrieves the default adapter for a [`ModelFamily`].
    #[must_use]
    pub fn default_for_family(&self, family: ModelFamily) -> Arc<dyn TokenizerAdapter> {
        let preferred_model = match family {
            ModelFamily::OpenAi => TargetModel::Gpt4o,
            ModelFamily::Anthropic => TargetModel::Claude35Sonnet,
            ModelFamily::Google => TargetModel::Gemini15Pro,
            ModelFamily::Meta => TargetModel::Llama3,
            ModelFamily::Mistral | ModelFamily::Generic => TargetModel::GenericFallback,
        };

        self.get_for_model(preferred_model)
            .unwrap_or_else(|| Arc::new(CalibratedHeuristicAdapter::default()))
    }

    /// Returns a list of all registered adapters.
    #[must_use]
    pub fn all_adapters(&self) -> Vec<Arc<dyn TokenizerAdapter>> {
        self.adapters_by_name.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_resolution() {
        let registry = TokenizerRegistry::default();

        let gpt4o_adapter = registry
            .get_for_model(TargetModel::Gpt4o)
            .expect("gpt4o adapter");
        assert_eq!(gpt4o_adapter.name(), "o200k_base");
        assert!(gpt4o_adapter.is_exact());

        let claude_adapter = registry.get_by_name("claude_bpe").expect("claude adapter");
        assert_eq!(claude_adapter.model_family(), ModelFamily::Anthropic);

        let fallback = registry.default_for_family(ModelFamily::Generic);
        assert_eq!(fallback.name(), "calibrated_heuristic");
        assert!(!fallback.is_exact());
    }
}
