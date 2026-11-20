//! Local Ollama AI provider adapter.
//!
//! Connects directly to a local or on-premises Ollama daemon (default `http://localhost:11434`)
//! for chat completions and vector embeddings without cloud egress.

use crate::client::{map_http_response_error, map_reqwest_error, AiClientConfig};
use crate::error::AiAdapterError;
use crate::models::{
    AiCapabilities, AiHealthStatus, AiModelDescriptor, AiProviderType, ChatChoice,
    ChatCompletionRequest, ChatCompletionResponse, ChatFinishReason, ChatMessage,
    ChatResponseFormat, EmbeddingRequest, EmbeddingResponse, TokenUsage,
};
use crate::traits::AiAdapter;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// AI adapter implementation for the local Ollama daemon.
#[derive(Debug, Clone)]
pub struct OllamaAiAdapter {
    config: AiClientConfig,
    client: Client,
    default_chat_model: String,
    default_embed_model: String,
    enabled: bool,
}

impl OllamaAiAdapter {
    /// Construct a new Ollama adapter for the given endpoint base URL.
    pub fn new(base_url: impl Into<String>) -> Result<Self, AiAdapterError> {
        let config = AiClientConfig::new(base_url);
        let client = config.build_client()?;
        Ok(Self {
            config,
            client,
            default_chat_model: "llama3.2".to_string(),
            default_embed_model: "nomic-embed-text".to_string(),
            enabled: true,
        })
    }

    /// Construct adapter targeting default localhost daemon (`http://localhost:11434`).
    pub fn with_defaults() -> Result<Self, AiAdapterError> {
        Self::new("http://localhost:11434")
    }

    /// Construct from custom `AiClientConfig`.
    pub fn with_config(config: AiClientConfig) -> Result<Self, AiAdapterError> {
        let client = config.build_client()?;
        Ok(Self {
            config,
            client,
            default_chat_model: "llama3.2".to_string(),
            default_embed_model: "nomic-embed-text".to_string(),
            enabled: true,
        })
    }

    /// Override the default chat model.
    pub fn with_chat_model(mut self, model: impl Into<String>) -> Self {
        self.default_chat_model = model.into();
        self
    }

    /// Override the default embedding model.
    pub fn with_embed_model(mut self, model: impl Into<String>) -> Self {
        self.default_embed_model = model.into();
        self
    }

    /// Toggle enabled status.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

// Wire structures for Ollama REST API

#[derive(Serialize)]
struct OllamaChatPayload<'a> {
    model: &'a str,
    messages: Vec<OllamaWireMessage<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaChatOptions>,
}

#[derive(Serialize)]
struct OllamaWireMessage<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize, Default)]
struct OllamaChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    #[serde(default)]
    prompt_eval_count: Option<usize>,
    #[serde(default)]
    eval_count: Option<usize>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaModelEntry>,
}

#[derive(Deserialize)]
struct OllamaModelEntry {
    name: String,
    #[serde(default)]
    details: Option<OllamaModelDetails>,
}

#[derive(Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    family: Option<String>,
    #[serde(default)]
    parameter_size: Option<String>,
}

#[derive(Serialize)]
struct OllamaBatchEmbedPayload<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize)]
struct OllamaBatchEmbedResponse {
    model: String,
    embeddings: Vec<Vec<f32>>,
    #[serde(default)]
    prompt_eval_count: Option<usize>,
}

#[derive(Serialize)]
struct OllamaLegacyEmbedPayload<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct OllamaLegacyEmbedResponse {
    embedding: Vec<f32>,
}

#[async_trait]
impl AiAdapter for OllamaAiAdapter {
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Ollama
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        if !self.enabled {
            return Ok(AiHealthStatus {
                is_healthy: false,
                provider: AiProviderType::Ollama,
                status_message: "Ollama adapter is disabled".to_string(),
                available_models_count: 0,
                latency_ms: 0,
            });
        }

        let start = Instant::now();
        let url = self.config.url("/api/tags");

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !resp.status().is_success() {
            let err = map_http_response_error(resp).await;
            return Err(err);
        }

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        Ok(AiHealthStatus {
            is_healthy: true,
            provider: AiProviderType::Ollama,
            status_message: format!(
                "Ollama daemon operational at {} ({} models loaded)",
                self.config.base_url,
                tags.models.len()
            ),
            available_models_count: tags.models.len(),
            latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Ollama adapter is disabled".into(),
            ));
        }

        let url = self.config.url("/api/tags");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if !resp.status().is_success() {
            return Err(map_http_response_error(resp).await);
        }

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        let descriptors = tags
            .models
            .into_iter()
            .map(|m| {
                let is_embed = m.name.contains("embed");
                let caps = if is_embed {
                    AiCapabilities::embeddings_only()
                } else {
                    AiCapabilities::chat_only()
                };

                let desc_text = m.details.as_ref().map(|d| {
                    format!(
                        "Family: {}, Size: {}",
                        d.family.as_deref().unwrap_or("unknown"),
                        d.parameter_size.as_deref().unwrap_or("unknown")
                    )
                });

                AiModelDescriptor {
                    id: m.name.clone(),
                    provider: AiProviderType::Ollama,
                    display_name: m.name,
                    context_window_tokens: 32_768,
                    max_output_tokens: 4_096,
                    capabilities: caps,
                    description: desc_text,
                }
            })
            .collect();

        Ok(descriptors)
    }

    async fn complete_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Ollama adapter is disabled".into(),
            ));
        }

        if request.messages.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Messages list cannot be empty".to_string(),
            ));
        }

        let start = Instant::now();
        let url = self.config.url("/api/chat");

        let wire_messages: Vec<OllamaWireMessage<'_>> = request
            .messages
            .iter()
            .map(|m| OllamaWireMessage {
                role: match m.role {
                    crate::models::ChatRole::System => "system",
                    crate::models::ChatRole::User => "user",
                    crate::models::ChatRole::Assistant => "assistant",
                    crate::models::ChatRole::Tool => "tool",
                },
                content: &m.content,
            })
            .collect();

        let format = match request.response_format {
            Some(ChatResponseFormat::JsonObject) => Some("json"),
            _ => None,
        };

        let options = Some(OllamaChatOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            num_predict: request.max_tokens,
            stop: request.stop_sequences,
            seed: request.seed,
        });

        let target_model = if request.model.is_empty() || request.model == "default" {
            &self.default_chat_model
        } else {
            &request.model
        };

        let payload = OllamaChatPayload {
            model: target_model,
            messages: wire_messages,
            stream: false,
            format,
            options,
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if !resp.status().is_success() {
            return Err(map_http_response_error(resp).await);
        }

        let body: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        let prompt_tokens = body.prompt_eval_count.unwrap_or(0);
        let completion_tokens = body.eval_count.unwrap_or(0);
        let usage = TokenUsage::new(prompt_tokens, completion_tokens);
        let latency_ms = start.elapsed().as_millis() as u64;

        let choice = ChatChoice {
            index: 0,
            message: ChatMessage::assistant(body.message.content),
            finish_reason: ChatFinishReason::Stop,
        };

        Ok(ChatCompletionResponse {
            id: format!("ollama-cmpl-{}", uuid::Uuid::new_v4()),
            model: body.model,
            choices: vec![choice],
            usage,
            created_at: Utc::now(),
            latency_ms,
        })
    }

    async fn generate_embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Ollama adapter is disabled".into(),
            ));
        }

        if request.inputs.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Embedding inputs cannot be empty".to_string(),
            ));
        }

        let start = Instant::now();
        let target_model = if request.model.is_empty() || request.model == "default" {
            &self.default_embed_model
        } else {
            &request.model
        };

        // Try modern Ollama /api/embed batch endpoint first
        let batch_url = self.config.url("/api/embed");
        let batch_payload = OllamaBatchEmbedPayload {
            model: target_model,
            input: &request.inputs,
        };

        let resp = self
            .client
            .post(&batch_url)
            .json(&batch_payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if resp.status().is_success() {
            let body: OllamaBatchEmbedResponse = resp
                .json()
                .await
                .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

            let dimensions = body.embeddings.first().map(|v| v.len()).unwrap_or(0);
            let prompt_tokens = body.prompt_eval_count.unwrap_or(0);
            let usage = TokenUsage::new(prompt_tokens, 0);
            let latency_ms = start.elapsed().as_millis() as u64;

            return Ok(EmbeddingResponse {
                model: body.model,
                embeddings: body.embeddings,
                usage,
                dimensions,
                latency_ms,
            });
        }

        // If batch /api/embed returned 404, fall back to legacy /api/embeddings in loop
        if resp.status() == StatusCode::NOT_FOUND {
            let legacy_url = self.config.url("/api/embeddings");
            let mut embeddings = Vec::with_capacity(request.inputs.len());

            for input in &request.inputs {
                let payload = OllamaLegacyEmbedPayload {
                    model: target_model,
                    prompt: input,
                };

                let legacy_resp = self
                    .client
                    .post(&legacy_url)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(map_reqwest_error)?;

                if !legacy_resp.status().is_success() {
                    return Err(map_http_response_error(legacy_resp).await);
                }

                let embed_data: OllamaLegacyEmbedResponse = legacy_resp
                    .json()
                    .await
                    .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

                embeddings.push(embed_data.embedding);
            }

            let dimensions = embeddings.first().map(|v| v.len()).unwrap_or(0);
            let usage = TokenUsage::new(0, 0);
            let latency_ms = start.elapsed().as_millis() as u64;

            return Ok(EmbeddingResponse {
                model: target_model.to_string(),
                embeddings,
                usage,
                dimensions,
                latency_ms,
            });
        }

        // Other error status
        Err(map_http_response_error(resp).await)
    }
}
