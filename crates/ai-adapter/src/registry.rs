//! Thread-safe runtime registry and lifecycle manager for AI provider adapters.

use crate::disabled::DisabledAiAdapter;
use crate::error::AiAdapterError;
use crate::guardrails::AiOptInPolicy;
use crate::models::{
    AiHealthStatus, AiModelDescriptor, AiProviderType, ChatCompletionRequest,
    ChatCompletionResponse, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
};
use crate::resilient::ResilientAiAdapter;
use crate::traits::AiAdapter;
use std::sync::{Arc, RwLock};

/// Global or service-scoped registry holding the currently active AI provider adapter.
///
/// Defaults to [`DisabledAiAdapter`] with [`AiOptInPolicy::disabled()`] to guarantee zero external
/// network dependencies or API key requirements unless explicitly opted in by user configuration.
#[derive(Debug, Clone)]
pub struct AiAdapterRegistry {
    active_adapter: Arc<RwLock<Arc<dyn AiAdapter>>>,
    policy: Arc<RwLock<AiOptInPolicy>>,
}

impl Default for AiAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AiAdapterRegistry {
    /// Create a new registry initialized with the zero-key [`DisabledAiAdapter`] and default disabled policy.
    pub fn new() -> Self {
        Self {
            active_adapter: Arc::new(RwLock::new(Arc::new(DisabledAiAdapter::new()))),
            policy: Arc::new(RwLock::new(AiOptInPolicy::disabled())),
        }
    }

    /// Construct a registry initialized with a specific adapter and default disabled policy.
    pub fn with_adapter(adapter: Arc<dyn AiAdapter>) -> Self {
        Self {
            active_adapter: Arc::new(RwLock::new(adapter)),
            policy: Arc::new(RwLock::new(AiOptInPolicy::disabled())),
        }
    }

    /// Set or update the registry's guardrail policy.
    pub fn with_policy(self, policy: AiOptInPolicy) -> Self {
        *self.policy.write().unwrap() = policy;
        self
    }

    /// Retrieve a cloned copy of the active policy.
    pub fn get_policy(&self) -> AiOptInPolicy {
        self.policy.read().unwrap().clone()
    }

    /// Dynamically update the active policy at runtime.
    pub fn set_policy(&self, policy: AiOptInPolicy) {
        *self.policy.write().unwrap() = policy;
    }

    /// Retrieve a cloned reference to the currently active adapter.
    pub fn get_adapter(&self) -> Arc<dyn AiAdapter> {
        self.active_adapter.read().unwrap().clone()
    }

    /// Dynamically switch or update the active adapter at runtime.
    pub fn set_adapter(&self, adapter: Arc<dyn AiAdapter>) {
        let mut guard = self.active_adapter.write().unwrap();
        *guard = adapter;
    }

    /// Register a primary adapter wrapped in a [`ResilientAiAdapter`] with the current registry policy.
    pub fn set_resilient_adapter(&self, primary: Arc<dyn AiAdapter>, fallback_on_error: bool) {
        let policy = self.get_policy();
        let resilient = Arc::new(
            ResilientAiAdapter::new(primary, policy).with_fallback_on_error(fallback_on_error),
        );
        self.set_adapter(resilient);
    }

    /// Reset the active adapter to the zero-key [`DisabledAiAdapter`] and policy to disabled.
    pub fn disable(&self) {
        self.set_policy(AiOptInPolicy::disabled());
        self.set_adapter(Arc::new(DisabledAiAdapter::new()));
    }

    /// Whether the active adapter is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.get_adapter().is_enabled()
    }

    /// Provider type of the active adapter.
    pub fn provider_type(&self) -> AiProviderType {
        self.get_adapter().provider_type()
    }

    /// Check health of the currently active adapter.
    pub async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        self.get_adapter().health_check().await
    }

    /// Enumerate models supported by the currently active adapter.
    pub async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        self.get_adapter().list_models().await
    }

    /// Execute a chat completion prompt via the active adapter.
    pub async fn complete_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError> {
        self.get_adapter().complete_chat(request).await
    }

    /// Generate vector embeddings via the active adapter.
    pub async fn generate_embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError> {
        self.get_adapter().generate_embeddings(request).await
    }

    /// Summarize a document via the active adapter.
    pub async fn summarize_document(
        &self,
        request: DocumentSummarizationRequest,
    ) -> Result<DocumentSummarizationResponse, AiAdapterError> {
        self.get_adapter().summarize_document(request).await
    }

    /// Enrich a document section via the active adapter.
    pub async fn enrich_section(
        &self,
        request: SectionEnrichmentRequest,
    ) -> Result<SectionEnrichmentResponse, AiAdapterError> {
        self.get_adapter().enrich_section(request).await
    }
}
