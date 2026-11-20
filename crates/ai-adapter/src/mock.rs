//! High-fidelity in-memory mock AI adapter for testing and offline development.

use crate::error::AiAdapterError;
use crate::models::{
    AiCapabilities, AiHealthStatus, AiModelDescriptor, AiProviderType, ChatChoice,
    ChatCompletionRequest, ChatCompletionResponse, ChatFinishReason, ChatMessage, EmbeddingRequest,
    EmbeddingResponse, TokenUsage,
};
use crate::traits::AiAdapter;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Call recorder for assertions in unit and integration test suites.
#[derive(Debug, Default)]
pub struct MockCallTracker {
    chat_calls: Mutex<Vec<ChatCompletionRequest>>,
    embedding_calls: Mutex<Vec<EmbeddingRequest>>,
    total_prompt_tokens: AtomicUsize,
    total_completion_tokens: AtomicUsize,
}

impl MockCallTracker {
    /// Create an empty call tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of total chat completion calls recorded.
    pub fn chat_call_count(&self) -> usize {
        self.chat_calls.lock().unwrap().len()
    }

    /// Number of total embedding calls recorded.
    pub fn embedding_call_count(&self) -> usize {
        self.embedding_calls.lock().unwrap().len()
    }

    /// Cloned snapshot of all chat completion requests.
    pub fn chat_requests(&self) -> Vec<ChatCompletionRequest> {
        self.chat_calls.lock().unwrap().clone()
    }

    /// Cloned snapshot of all embedding requests.
    pub fn embedding_requests(&self) -> Vec<EmbeddingRequest> {
        self.embedding_calls.lock().unwrap().clone()
    }

    /// Total prompt tokens consumed across all recorded operations.
    pub fn prompt_tokens(&self) -> usize {
        self.total_prompt_tokens.load(Ordering::SeqCst)
    }

    /// Total completion tokens produced across all recorded operations.
    pub fn completion_tokens(&self) -> usize {
        self.total_completion_tokens.load(Ordering::SeqCst)
    }

    /// Clear all recorded history and token counters.
    pub fn reset(&self) {
        self.chat_calls.lock().unwrap().clear();
        self.embedding_calls.lock().unwrap().clear();
        self.total_prompt_tokens.store(0, Ordering::SeqCst);
        self.total_completion_tokens.store(0, Ordering::SeqCst);
    }
}

/// In-memory mock adapter supporting scripted responses, error injection,
/// deterministic pseudo-embeddings, and call tracking.
#[derive(Debug, Clone)]
pub struct MockAiAdapter {
    enabled: bool,
    provider_type: AiProviderType,
    models: Vec<AiModelDescriptor>,
    scripted_chat_responses: HashMap<String, String>,
    default_chat_response: Option<String>,
    scripted_embeddings: HashMap<String, Vec<f32>>,
    default_embedding_dimension: usize,
    injected_error: Option<AiAdapterError>,
    simulated_latency_ms: u64,
    call_tracker: Arc<MockCallTracker>,
}

impl Default for MockAiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockAiAdapter {
    /// Create a standard mock adapter with default test models.
    pub fn new() -> Self {
        let models = vec![
            AiModelDescriptor {
                id: "mock-chat".to_string(),
                provider: AiProviderType::Mock,
                display_name: "Mock Chat 3.2".to_string(),
                context_window_tokens: 128_000,
                max_output_tokens: 4_096,
                capabilities: AiCapabilities::chat_only(),
                description: Some("In-memory mock chat model".to_string()),
            },
            AiModelDescriptor {
                id: "mock-embed".to_string(),
                provider: AiProviderType::Mock,
                display_name: "Mock Embedding v1".to_string(),
                context_window_tokens: 8_192,
                max_output_tokens: 0,
                capabilities: AiCapabilities::embeddings_only(),
                description: Some("In-memory deterministic embedding model".to_string()),
            },
        ];

        Self {
            enabled: true,
            provider_type: AiProviderType::Mock,
            models,
            scripted_chat_responses: HashMap::new(),
            default_chat_response: None,
            scripted_embeddings: HashMap::new(),
            default_embedding_dimension: 384,
            injected_error: None,
            simulated_latency_ms: 0,
            call_tracker: Arc::new(MockCallTracker::new()),
        }
    }

    /// Access the shared call tracker.
    pub fn tracker(&self) -> Arc<MockCallTracker> {
        Arc::clone(&self.call_tracker)
    }

    /// Toggle adapter enabled status.
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        self
    }

    /// Set simulated execution latency in milliseconds.
    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.simulated_latency_ms = latency_ms;
        self
    }

    /// Set an error to inject into all subsequent calls.
    pub fn with_injected_error(mut self, error: AiAdapterError) -> Self {
        self.injected_error = Some(error);
        self
    }

    /// Clear injected error.
    pub fn clear_injected_error(&mut self) -> &mut Self {
        self.injected_error = None;
        self
    }

    /// Script a specific chat response when prompt contains a keyword or substring.
    pub fn with_scripted_chat(
        mut self,
        match_substring: impl Into<String>,
        response: impl Into<String>,
    ) -> Self {
        self.scripted_chat_responses
            .insert(match_substring.into(), response.into());
        self
    }

    /// Set fallback default chat response.
    pub fn with_default_chat_response(mut self, response: impl Into<String>) -> Self {
        self.default_chat_response = Some(response.into());
        self
    }

    /// Script a specific embedding vector for an exact input string.
    pub fn with_scripted_embedding(mut self, input: impl Into<String>, vector: Vec<f32>) -> Self {
        self.scripted_embeddings.insert(input.into(), vector);
        self
    }

    /// Set default embedding dimension for synthetic vectors.
    pub fn with_embedding_dimension(mut self, dimension: usize) -> Self {
        self.default_embedding_dimension = dimension;
        self
    }

    /// Override available models list.
    pub fn with_models(mut self, models: Vec<AiModelDescriptor>) -> Self {
        self.models = models;
        self
    }

    /// Generate deterministic pseudo-random unit-normalized embedding vector derived from Blake3 hash.
    fn generate_deterministic_embedding(&self, input: &str, dimensions: usize) -> Vec<f32> {
        let hash = blake3::hash(input.as_bytes());
        let hash_bytes = hash.as_bytes();

        let mut vector = Vec::with_capacity(dimensions);
        let mut sum_sq = 0.0f32;

        for i in 0..dimensions {
            let byte_a = hash_bytes[i % 32];
            let byte_b = hash_bytes[(i * 7 + 13) % 32];
            let raw_val = (((byte_a as u16) << 8 | (byte_b as u16)) as f32) / 65535.0f32;
            let centered = raw_val * 2.0 - 1.0;
            sum_sq += centered * centered;
            vector.push(centered);
        }

        // L2 normalize
        let norm = sum_sq.sqrt().max(1e-12);
        for val in &mut vector {
            *val /= norm;
        }

        vector
    }
}

#[async_trait]
impl AiAdapter for MockAiAdapter {
    fn provider_type(&self) -> AiProviderType {
        self.provider_type
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn health_check(&self) -> Result<AiHealthStatus, AiAdapterError> {
        if !self.enabled {
            return Ok(AiHealthStatus {
                is_healthy: false,
                provider: self.provider_type,
                status_message: "Mock adapter is disabled".to_string(),
                available_models_count: 0,
                latency_ms: 0,
            });
        }

        if let Some(ref err) = self.injected_error {
            return Err(err.clone());
        }

        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.simulated_latency_ms)).await;
        }

        Ok(AiHealthStatus {
            is_healthy: true,
            provider: self.provider_type,
            status_message: "Mock AI adapter operational (in-memory)".to_string(),
            available_models_count: self.models.len(),
            latency_ms: self.simulated_latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Mock adapter disabled".into(),
            ));
        }
        if let Some(ref err) = self.injected_error {
            return Err(err.clone());
        }
        Ok(self.models.clone())
    }

    async fn complete_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Mock AI adapter is disabled.".to_string(),
            ));
        }

        if let Some(ref err) = self.injected_error {
            return Err(err.clone());
        }

        if request.messages.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Messages list cannot be empty".to_string(),
            ));
        }

        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.simulated_latency_ms)).await;
        }

        // Record request in tracker
        self.call_tracker
            .chat_calls
            .lock()
            .unwrap()
            .push(request.clone());

        // Concatenate message content for inspection
        let combined_prompt = request
            .messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Determine response text
        let mut response_text = None;

        // 1. Scripted substring matches
        for (needle, scripted) in &self.scripted_chat_responses {
            if combined_prompt.contains(needle) {
                response_text = Some(scripted.clone());
                break;
            }
        }

        // 2. Default override
        if response_text.is_none() {
            if let Some(ref def) = self.default_chat_response {
                response_text = Some(def.clone());
            }
        }

        // 3. Automated heuristic synthesizer
        let text = response_text.unwrap_or_else(|| {
            if combined_prompt.contains("Enrich this section with metadata") {
                "Title: Core Module Architecture\n\
                 Description: Implementation of high-performance components and traits.\n\
                 Tags: rust, architecture, traits, core"
                    .to_string()
            } else if combined_prompt.contains("expert AI documentation analyst") {
                "This document specifies system components, architectures, and boundaries.\n\n\
                 Key Topics: architecture, traits, adapters, performance"
                    .to_string()
            } else {
                format!(
                    "Mock response from model '{}' for prompt of {} chars.",
                    request.model,
                    combined_prompt.len()
                )
            }
        });

        // Compute estimated token counts
        let prompt_tokens = (combined_prompt.len() / 4).max(1);
        let completion_tokens = (text.len() / 4).max(1);
        let usage = TokenUsage::new(prompt_tokens, completion_tokens);

        self.call_tracker
            .total_prompt_tokens
            .fetch_add(prompt_tokens, Ordering::SeqCst);
        self.call_tracker
            .total_completion_tokens
            .fetch_add(completion_tokens, Ordering::SeqCst);

        let choice = ChatChoice {
            index: 0,
            message: ChatMessage::assistant(text),
            finish_reason: ChatFinishReason::Stop,
        };

        Ok(ChatCompletionResponse {
            id: format!("mock-cmpl-{}", uuid::Uuid::new_v4()),
            model: request.model,
            choices: vec![choice],
            usage,
            created_at: Utc::now(),
            latency_ms: self.simulated_latency_ms,
        })
    }

    async fn generate_embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiAdapterError> {
        if !self.enabled {
            return Err(AiAdapterError::ProviderDisabled(
                "Mock AI adapter is disabled.".to_string(),
            ));
        }

        if let Some(ref err) = self.injected_error {
            return Err(err.clone());
        }

        if request.inputs.is_empty() {
            return Err(AiAdapterError::InvalidRequest(
                "Embedding inputs cannot be empty".to_string(),
            ));
        }

        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.simulated_latency_ms)).await;
        }

        // Record request in tracker
        self.call_tracker
            .embedding_calls
            .lock()
            .unwrap()
            .push(request.clone());

        let dim = request
            .dimensions
            .unwrap_or(self.default_embedding_dimension);

        let mut embeddings = Vec::with_capacity(request.inputs.len());
        let mut total_chars = 0;

        for input in &request.inputs {
            total_chars += input.len();
            if let Some(scripted) = self.scripted_embeddings.get(input) {
                embeddings.push(scripted.clone());
            } else {
                embeddings.push(self.generate_deterministic_embedding(input, dim));
            }
        }

        let prompt_tokens = (total_chars / 4).max(1);
        let usage = TokenUsage::new(prompt_tokens, 0);

        self.call_tracker
            .total_prompt_tokens
            .fetch_add(prompt_tokens, Ordering::SeqCst);

        Ok(EmbeddingResponse {
            model: request.model,
            embeddings,
            usage,
            dimensions: dim,
            latency_ms: self.simulated_latency_ms,
        })
    }
}
