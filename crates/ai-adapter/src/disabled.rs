//! Disabled AI adapter providing zero-key fallback with zero external dependencies.

use crate::error::AiAdapterError;
use crate::models::{
    AiHealthStatus, AiModelDescriptor, AiProviderType, ChatCompletionRequest,
    ChatCompletionResponse, EmbeddingRequest, EmbeddingResponse,
};
use crate::traits::AiAdapter;
use async_trait::async_trait;

/// Null object implementation representing a disabled AI subsystem.
///
/// Ensures the entire extraction, normalization, deduplication, and export
/// pipelines operate without any external API keys or local LLM runtimes,
/// satisfying Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT.
#[derive(Debug, Default, Clone, Copy)]
pub struct DisabledAiAdapter;

impl DisabledAiAdapter {
    /// Create a new disabled adapter instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AiAdapter for DisabledAiAdapter {
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Disabled
    }

    fn is_enabled(&self) -> bool {
        false
    }

    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        Ok(AiHealthStatus {
            is_healthy: false,
            provider: AiProviderType::Disabled,
            status_message: "AI provider is disabled (zero-key fallback active)".to_string(),
            available_models_count: 0,
            latency_ms: 0,
        })
    }

    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        Ok(Vec::new())
    }

    async fn complete_chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError> {
        Err(AiAdapterError::ProviderDisabled(
            "AI provider is not configured or enabled. The platform operates cleanly in standalone zero-key mode.".to_string(),
        ))
    }

    async fn generate_embeddings(
        &self,
        _request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError> {
        Err(AiAdapterError::ProviderDisabled(
            "AI embeddings are disabled. Zero-key fallback active.".to_string(),
        ))
    }
}
