//! OpenAI-compatible AI provider adapter.
//!
//! Interacts with standard OpenAI endpoints or OpenAI-compatible local runtimes
//! (e.g. vLLM, LM Studio, LocalAI, Ollama `/v1`, Azure OpenAI).

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
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// AI adapter implementation for OpenAI and OpenAI-compatible REST endpoints.
#[derive(Debug, Clone)]
pub struct OpenAiAiAdapter {
    config: AiClientConfig,
    client: Client,
    default_chat_model: String,
    default_embed_model: String,
    enabled: bool,
}

impl OpenAiAiAdapter {
    /// Construct adapter for a custom OpenAI-compatible endpoint.
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self, AiAdapterError> {
        let mut config = AiClientConfig::new(base_url);
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        let client = config.build_client()?;
        Ok(Self {
            config,
            client,
            default_chat_model: "gpt-4o-mini".to_string(),
            default_embed_model: "text-embedding-3-small".to_string(),
            enabled: true,
        })
    }

    /// Construct targeting official OpenAI API (`https://api.openai.com/v1`).
    pub fn for_openai(api_key: impl Into<String>) -> Result<Self, AiAdapterError> {
        Self::new("https://api.openai.com/v1", Some(api_key.into()))
    }

    /// Construct targeting a local inference server (e.g. LM Studio, vLLM, LocalAI) without auth.
    pub fn for_local(base_url: impl Into<String>) -> Result<Self, AiAdapterError> {
        Self::new(base_url, None)
    }

    /// Construct from custom `AiClientConfig`.
    pub fn with_config(config: AiClientConfig) -> Result<Self, AiAdapterError> {
        let client = config.build_client()?;
        Ok(Self {
            config,
            client,
            default_chat_model: "gpt-4o-mini".to_string(),
            default_embed_model: "text-embedding-3-small".to_string(),
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

// Wire structures for OpenAI Chat Completions & Embeddings

#[derive(Serialize)]
struct OpenAiChatPayload<'a> {
    model: &'a str,
    messages: Vec<OpenAiWireMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<OpenAiResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Serialize)]
struct OpenAiWireMessage<'a> {
    role: &'static str,
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Serialize)]
struct OpenAiResponseFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChatChoice>,
    usage: Option<OpenAiTokenUsage>,
}

#[derive(Deserialize)]
struct OpenAiChatChoice {
    index: usize,
    message: OpenAiResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiTokenUsage {
    #[serde(default)]
    prompt_tokens: usize,
    #[serde(default)]
    completion_tokens: usize,
    #[serde(default)]
    total_tokens: usize,
}

#[derive(Deserialize)]
struct OpenAiModelsListResponse {
    #[serde(default)]
    data: Vec<OpenAiModelEntry>,
}

#[derive(Deserialize)]
struct OpenAiModelEntry {
    id: String,
    #[serde(default)]
    owned_by: Option<String>,
}

#[derive(Serialize)]
struct OpenAiEmbeddingPayload<'a> {
    model: &'a str,
    input: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    model: String,
    data: Vec<OpenAiEmbeddingData>,
    usage: Option<OpenAiEmbeddingUsage>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    index: usize,
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingUsage {
    #[serde(default)]
    prompt_tokens: usize,
}

#[async_trait]
impl AiAdapter for OpenAiAiAdapter {
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::OpenAiCompatible
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        if !self.enabled {
            return Ok(AiHealthStatus {
                is_healthy: false,
                provider: AiProviderType::OpenAiCompatible,
                status_message: "OpenAI-compatible adapter is disabled".to_string(),
                available_models_count: 0,
                latency_ms: 0,
            });
        }

        let start = Instant::now();
        let url = self.config.url("/models");

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !resp.status().is_success() {
            return Err(map_http_response_error(resp).await);
        }

        let list: OpenAiModelsListResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        Ok(AiHealthStatus {
            is_healthy: true,
            provider: AiProviderType::OpenAiCompatible,
            status_message: format!(
                "OpenAI-compatible endpoint operational at {} ({} models reported)",
                self.config.base_url,
                list.data.len()
            ),
            available_models_count: list.data.len(),
            latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "OpenAI-compatible adapter is disabled".into(),
            ));
        }

        let url = self.config.url("/models");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if !resp.status().is_success() {
            return Err(map_http_response_error(resp).await);
        }

        let list: OpenAiModelsListResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        let descriptors = list
            .data
            .into_iter()
            .map(|m| {
                let is_embed = m.id.contains("embed");
                let caps = if is_embed {
                    AiCapabilities::embeddings_only()
                } else {
                    AiCapabilities::chat_only()
                };

                let desc_text = m.owned_by.as_ref().map(|o| format!("Owner: {}", o));

                AiModelDescriptor {
                    id: m.id.clone(),
                    provider: AiProviderType::OpenAiCompatible,
                    display_name: m.id,
                    context_window_tokens: 128_000,
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
                "OpenAI-compatible adapter is disabled".into(),
            ));
        }

        if request.messages.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Messages list cannot be empty".to_string(),
            ));
        }

        let start = Instant::now();
        let url = self.config.url("/chat/completions");

        let wire_messages: Vec<OpenAiWireMessage<'_>> = request
            .messages
            .iter()
            .map(|m| OpenAiWireMessage {
                role: match m.role {
                    crate::models::ChatRole::System => "system",
                    crate::models::ChatRole::User => "user",
                    crate::models::ChatRole::Assistant => "assistant",
                    crate::models::ChatRole::Tool => "tool",
                },
                content: &m.content,
                name: m.name.as_deref(),
            })
            .collect();

        let response_format = match request.response_format {
            Some(ChatResponseFormat::JsonObject) => Some(OpenAiResponseFormat {
                format_type: "json_object",
            }),
            _ => None,
        };

        let target_model = if request.model.is_empty() || request.model == "default" {
            &self.default_chat_model
        } else {
            &request.model
        };

        let payload = OpenAiChatPayload {
            model: target_model,
            messages: wire_messages,
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            stop: request.stop_sequences,
            response_format,
            seed: request.seed,
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

        let body: OpenAiChatResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        let usage = body
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: if u.total_tokens > 0 {
                    u.total_tokens
                } else {
                    u.prompt_tokens + u.completion_tokens
                },
            })
            .unwrap_or_default();

        let latency_ms = start.elapsed().as_millis() as u64;

        let choices = body
            .choices
            .into_iter()
            .map(|c| {
                let finish_reason = match c.finish_reason.as_deref() {
                    Some("stop") => ChatFinishReason::Stop,
                    Some("length") => ChatFinishReason::Length,
                    Some("content_filter") => ChatFinishReason::ContentFilter,
                    Some("tool_calls") => ChatFinishReason::ToolCalls,
                    Some(other) => ChatFinishReason::Unknown(other.to_string()),
                    None => ChatFinishReason::Stop,
                };

                ChatChoice {
                    index: c.index,
                    message: ChatMessage::assistant(c.message.content.unwrap_or_default()),
                    finish_reason,
                }
            })
            .collect();

        Ok(ChatCompletionResponse {
            id: body.id,
            model: body.model,
            choices,
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
                "OpenAI-compatible adapter is disabled".into(),
            ));
        }

        if request.inputs.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Embedding inputs cannot be empty".to_string(),
            ));
        }

        let start = Instant::now();
        let url = self.config.url("/embeddings");

        let target_model = if request.model.is_empty() || request.model == "default" {
            &self.default_embed_model
        } else {
            &request.model
        };

        let payload = OpenAiEmbeddingPayload {
            model: target_model,
            input: &request.inputs,
            dimensions: request.dimensions,
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

        let body: OpenAiEmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| AiAdapterError::ResponseParsingError(e.to_string()))?;

        let mut data = body.data;
        data.sort_by_key(|d| d.index);

        let embeddings: Vec<Vec<f32>> = data.into_iter().map(|d| d.embedding).collect();
        let dimensions = embeddings.first().map(|v| v.len()).unwrap_or(0);

        let usage = body
            .usage
            .map(|u| TokenUsage::new(u.prompt_tokens, 0))
            .unwrap_or_default();

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(EmbeddingResponse {
            model: body.model,
            embeddings,
            usage,
            dimensions,
            latency_ms,
        })
    }
}
