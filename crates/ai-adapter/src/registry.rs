//! Thread-safe runtime registry and lifecycle manager for AI provider adapters.

use crate::disabled::DisabledAiAdapter;
use crate::error::AiAdapterError;
use crate::models::{
    AiHealthStatus, AiModelDescriptor, AiProviderType, ChatCompletionRequest,
    ChatCompletionResponse, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
};
use crate::traits::AiAdapter;
use std::sync::{Arc, RwLock};

/// Global or service-scoped registry holding the currently active AI provider adapter.
///
/// Defaults to [`DisabledAiAdapter`] to guarantee zero external network dependencies
/// or API key requirements unless explicitly opted in by user configuration.
#[derive(Debug, Clone)]
pub struct AiAdapterRegistry {
    active_adapter: Arc<RwLock<Arc<dyn AiAdapter>>>,
}

impl Default for AiAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AiAdapterRegistry {
    /// Create a new registry initialized with the zero-key [`DisabledAiAdapter`].
    pub fn new() -> Self {
        Self {
            active_adapter: Arc::new(RwLock::new(Arc::new(DisabledAiAdapter::new()))),
        }
    }

    /// Construct a registry initialized with a specific adapter.
    pub fn with_adapter(adapter: Arc<dyn AiAdapter>) -> Self {
        Self {
            active_adapter: Arc::new(RwLock::new(adapter)),
        }
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

    /// Reset the active adapter to the zero-key [`DisabledAiAdapter`].
    pub fn disable(&self) {
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
