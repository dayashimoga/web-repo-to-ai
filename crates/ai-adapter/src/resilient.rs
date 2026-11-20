//! Resilient AI provider adapter wrapping primary providers with policy guardrails and zero-key fallback.

use crate::error::AiAdapterError;
use crate::fallback::ZeroKeyFallbackEngine;
use crate::guardrails::AiOptInPolicy;
use crate::models::{
    AiHealthStatus, AiModelDescriptor, AiProviderType, ChatCompletionRequest,
    ChatCompletionResponse, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
};
use crate::traits::AiAdapter;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::warn;

/// Resilient decorator wrapping an [`AiAdapter`] implementation with opt-in guardrails and
/// guaranteed zero-key CPU fallbacks.
///
/// Ensures compliance with Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT:
/// Document extraction and conversion pipelines always succeed without external keys or network.
#[derive(Debug, Clone)]
pub struct ResilientAiAdapter {
    primary: Arc<dyn AiAdapter>,
    policy: AiOptInPolicy,
    fallback_engine: ZeroKeyFallbackEngine,
    fallback_on_error: bool,
}

impl ResilientAiAdapter {
    /// Construct a new resilient adapter with default fallback enabled.
    pub fn new(primary: Arc<dyn AiAdapter>, policy: AiOptInPolicy) -> Self {
        Self {
            primary,
            policy,
            fallback_engine: ZeroKeyFallbackEngine::new(),
            fallback_on_error: true,
        }
    }

    /// Configure whether transient and non-retryable provider errors automatically degrade to zero-key fallbacks.
    #[must_use]
    pub fn with_fallback_on_error(mut self, enabled: bool) -> Self {
        self.fallback_on_error = enabled;
        self
    }

    /// Update active policy guardrails.
    #[must_use]
    pub fn with_policy(mut self, policy: AiOptInPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Get reference to active policy.
    #[must_use]
    pub fn policy(&self) -> &AiOptInPolicy {
        &self.policy
    }

    /// Get reference to the underlying primary adapter.
    #[must_use]
    pub fn primary(&self) -> &Arc<dyn AiAdapter> {
        &self.primary
    }

    /// Fallback models catalog.
    fn fallback_models(&self) -> Vec<AiModelDescriptor> {
        vec![
            AiModelDescriptor::new_chat(
                "extractive-cpu-fallback",
                AiProviderType::Disabled,
                "Extractive CPU Fallback Summarizer",
                128_000,
                4096,
            ),
            AiModelDescriptor::new_chat(
                "heuristic-cpu-fallback",
                AiProviderType::Disabled,
                "Heuristic CPU Fallback Enricher",
                128_000,
                4096,
            ),
            AiModelDescriptor::new_embedding(
                "deterministic-cpu-embeddings",
                AiProviderType::Disabled,
                "Blake3 Deterministic CPU Embeddings",
                128_000,
            ),
        ]
    }
}

#[async_trait]
impl AiAdapter for ResilientAiAdapter {
    fn provider_type(&self) -> AiProviderType {
        if self.policy.is_enabled() {
            self.primary.provider_type()
        } else {
            AiProviderType::Disabled
        }
    }

    fn is_enabled(&self) -> bool {
        self.policy.is_enabled() && self.primary.is_enabled()
    }

    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(AiHealthStatus::healthy(
                AiProviderType::Disabled,
                "AI features disabled by policy; zero-key fallback active",
            ));
        }

        match self.primary.health_check().await {
            Ok(status) => Ok(status),
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Primary AI adapter health check failed; zero-key fallback active"
                    );
                    Ok(AiHealthStatus::degraded(
                        self.primary.provider_type(),
                        format!("Primary provider error: {err}. Zero-key fallback active."),
                    ))
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(self.fallback_models());
        }

        match self.primary.list_models().await {
            Ok(mut models) => {
                models.extend(self.fallback_models());
                Ok(models)
            }
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Failed to list primary models; returning fallback catalog"
                    );
                    Ok(self.fallback_models())
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn complete_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(self.fallback_engine.fallback_chat(&request));
        }

        match self.primary.complete_chat(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Primary AI chat completion failed; degrading to zero-key CPU fallback"
                    );
                    Ok(self.fallback_engine.fallback_chat(&request))
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn generate_embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(self.fallback_engine.fallback_embeddings(&request));
        }

        match self.primary.generate_embeddings(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Primary AI embeddings generation failed; degrading to Blake3 zero-key fallback"
                    );
                    Ok(self.fallback_engine.fallback_embeddings(&request))
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn summarize_document(
        &self,
        request: DocumentSummarizationRequest,
    ) -> Result<DocumentSummarizationResponse, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(self.fallback_engine.fallback_summarize(&request));
        }

        match self.primary.summarize_document(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Primary AI document summarization failed; degrading to extractive CPU fallback"
                    );
                    Ok(self.fallback_engine.fallback_summarize(&request))
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn enrich_section(
        &self,
        request: SectionEnrichmentRequest,
    ) -> Result<SectionEnrichmentResponse, AiAdapterError> {
        if !self.policy.is_enabled() {
            return Ok(self.fallback_engine.fallback_enrich_section(&request));
        }

        match self.primary.enrich_section(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if self.fallback_on_error {
                    warn!(
                        error = %err,
                        "Primary AI section enrichment failed; degrading to heuristic CPU fallback"
                    );
                    Ok(self.fallback_engine.fallback_enrich_section(&request))
                } else {
                    Err(err)
                }
            }
        }
    }
}
