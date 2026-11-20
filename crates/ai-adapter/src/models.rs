//! Domain models and schema definitions for provider-independent AI adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported AI provider categories and backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderType {
    /// No AI provider configured or enabled (zero-key fallback).
    Disabled,
    /// In-memory mock adapter for deterministic unit and integration tests.
    Mock,
    /// Local Ollama daemon running on localhost:11434 without cloud egress.
    Ollama,
    /// OpenAI-compatible REST API (OpenAI, LocalAI, vLLM, LM Studio, etc.).
    OpenAiCompatible,
    /// Anthropic-compatible Claude API.
    AnthropicCompatible,
    /// Custom local or on-premises inference runtime.
    CustomLocal,
}

impl std::fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Mock => write!(f, "mock"),
            Self::Ollama => write!(f, "ollama"),
            Self::OpenAiCompatible => write!(f, "openai_compatible"),
            Self::AnthropicCompatible => write!(f, "anthropic_compatible"),
            Self::CustomLocal => write!(f, "custom_local"),
        }
    }
}

/// Feature capabilities supported by a given AI model or backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiCapabilities {
    /// Whether the model supports chat completions.
    pub supports_chat: bool,
    /// Whether the model supports generating vector embeddings.
    pub supports_embeddings: bool,
    /// Whether token streaming responses are supported.
    pub supports_streaming: bool,
    /// Whether multimodal vision/image input is supported.
    pub supports_vision: bool,
    /// Whether JSON schema / structured output mode is supported.
    pub supports_json_mode: bool,
    /// Whether function calling / tool use is supported.
    pub supports_tools: bool,
}

impl Default for AiCapabilities {
    fn default() -> Self {
        Self {
            supports_chat: true,
            supports_embeddings: false,
            supports_streaming: false,
            supports_vision: false,
            supports_json_mode: false,
            supports_tools: false,
        }
    }
}

impl AiCapabilities {
    /// Capabilities for a standard text chat model.
    pub fn chat_only() -> Self {
        Self {
            supports_chat: true,
            supports_embeddings: false,
            supports_streaming: true,
            supports_vision: false,
            supports_json_mode: true,
            supports_tools: false,
        }
    }

    /// Capabilities for a pure dense embedding model.
    pub fn embeddings_only() -> Self {
        Self {
            supports_chat: false,
            supports_embeddings: true,
            supports_streaming: false,
            supports_vision: false,
            supports_json_mode: false,
            supports_tools: false,
        }
    }

    /// Full capabilities enabled.
    pub fn full() -> Self {
        Self {
            supports_chat: true,
            supports_embeddings: true,
            supports_streaming: true,
            supports_vision: true,
            supports_json_mode: true,
            supports_tools: true,
        }
    }
}

/// Metadata descriptor for an AI model provided by an adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModelDescriptor {
    /// Unique model identifier (e.g. `"llama3.2"`, `"gpt-4o-mini"`, `"nomic-embed-text"`).
    pub id: String,
    /// Provider type hosting or servicing this model.
    pub provider: AiProviderType,
    /// Human-readable display label.
    pub display_name: String,
    /// Maximum context window in tokens (prompt + completion).
    pub context_window_tokens: usize,
    /// Maximum output generation tokens.
    pub max_output_tokens: usize,
    /// Capability flags.
    pub capabilities: AiCapabilities,
    /// Optional description or parameter size note.
    pub description: Option<String>,
}

impl AiModelDescriptor {
    /// Construct a basic chat model descriptor.
    pub fn new_chat(
        id: impl Into<String>,
        provider: AiProviderType,
        display_name: impl Into<String>,
        context_window_tokens: usize,
        max_output_tokens: usize,
    ) -> Self {
        Self {
            id: id.into(),
            provider,
            display_name: display_name.into(),
            context_window_tokens,
            max_output_tokens,
            capabilities: AiCapabilities::chat_only(),
            description: None,
        }
    }

    /// Construct a basic embedding model descriptor.
    pub fn new_embedding(
        id: impl Into<String>,
        provider: AiProviderType,
        display_name: impl Into<String>,
        context_window_tokens: usize,
    ) -> Self {
        Self {
            id: id.into(),
            provider,
            display_name: display_name.into(),
            context_window_tokens,
            max_output_tokens: 0,
            capabilities: AiCapabilities::embeddings_only(),
            description: None,
        }
    }
}

/// Chat message roles adhering to canonical conversation formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    /// System instruction establishing persona and constraints.
    System,
    /// User query or document context.
    User,
    /// Assistant response.
    Assistant,
    /// Tool/function output response.
    Tool,
}

impl std::fmt::Display for ChatRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::Tool => write!(f, "tool"),
        }
    }
}

/// Individual message in a chat completion prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message author.
    pub role: ChatRole,
    /// Message textual content.
    pub content: String,
    /// Optional participant or tool name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    /// Create a system instruction message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            name: None,
        }
    }

    /// Create a user prompt message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            name: None,
        }
    }

    /// Create an assistant response message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            name: None,
        }
    }

    /// Create a tool output message.
    pub fn tool(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Tool,
            content: content.into(),
            name: Some(name.into()),
        }
    }
}

/// Requested format of the chat completion output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatResponseFormat {
    /// Plain unstructured text or markdown.
    Text,
    /// Valid JSON object format.
    JsonObject,
}

/// Request parameters for a chat completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Target model identifier.
    pub model: String,
    /// Sequence of conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (typically 0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Nucleus sampling probability cutoff (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Maximum tokens to generate in completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    /// Stop sequences that halt generation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    /// Requested response format (Text or JsonObject).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ChatResponseFormat>,
    /// Deterministic seed for reproducible generation if supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

impl ChatCompletionRequest {
    /// Create a new chat completion request with default parameters.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop_sequences: Vec::new(),
            response_format: None,
            seed: None,
        }
    }

    /// Set sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set maximum tokens.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set response format to JSON object.
    pub fn with_json_format(mut self) -> Self {
        self.response_format = Some(ChatResponseFormat::JsonObject);
        self
    }

    /// Add a stop sequence.
    pub fn with_stop_sequence(mut self, stop: impl Into<String>) -> Self {
        self.stop_sequences.push(stop.into());
        self
    }

    /// Set deterministic seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

/// Reason why completion generation terminated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatFinishReason {
    /// Model hit natural stop sequence or EOS token.
    Stop,
    /// Generation halted due to max_tokens limit.
    Length,
    /// Content was flagged or filtered by provider.
    ContentFilter,
    /// Model emitted tool calls.
    ToolCalls,
    /// Other or unmapped reason.
    Unknown(String),
}

/// An individual choice in a chat completion response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Index of this choice (usually 0).
    pub index: usize,
    /// Generated assistant message.
    pub message: ChatMessage,
    /// Termination condition.
    pub finish_reason: ChatFinishReason,
}

/// Token accounting metrics for an AI operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in input prompt / messages.
    pub prompt_tokens: usize,
    /// Number of tokens generated in completion.
    pub completion_tokens: usize,
    /// Sum of prompt and completion tokens.
    pub total_tokens: usize,
}

impl TokenUsage {
    /// Construct token usage record with automatic total calculation.
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
        }
    }
}

/// Successful response from a chat completion operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique completion identifier.
    pub id: String,
    /// Model that produced the completion.
    pub model: String,
    /// Generated choices.
    pub choices: Vec<ChatChoice>,
    /// Token usage accounting.
    pub usage: TokenUsage,
    /// UTC timestamp of response creation.
    pub created_at: DateTime<Utc>,
    /// Execution roundtrip latency in milliseconds.
    pub latency_ms: u64,
}

impl ChatCompletionResponse {
    /// Retrieve the primary generated message if available.
    pub fn primary_message(&self) -> Option<&ChatMessage> {
        self.choices.first().map(|c| &c.message)
    }

    /// Retrieve the text content of the primary choice.
    pub fn primary_text(&self) -> Option<&str> {
        self.primary_message().map(|m| m.content.as_str())
    }
}

/// Request parameters for generating vector embeddings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Target embedding model.
    pub model: String,
    /// Text inputs to embed.
    pub inputs: Vec<String>,
    /// Optional output dimension truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
}

impl EmbeddingRequest {
    /// Create embedding request for multiple text inputs.
    pub fn new(model: impl Into<String>, inputs: Vec<String>) -> Self {
        Self {
            model: model.into(),
            inputs,
            dimensions: None,
        }
    }

    /// Create embedding request for a single text input.
    pub fn single(model: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            inputs: vec![input.into()],
            dimensions: None,
        }
    }

    /// Truncate or specify output dimensions.
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }
}

/// Successful response from an embedding generation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Model that generated the embeddings.
    pub model: String,
    /// Output vector representations (one per input text).
    pub embeddings: Vec<Vec<f32>>,
    /// Token usage accounting.
    pub usage: TokenUsage,
    /// Dimensionality of each output vector.
    pub dimensions: usize,
    /// Execution latency in milliseconds.
    pub latency_ms: u64,
}

impl EmbeddingResponse {
    /// Retrieve the primary embedding vector if available.
    pub fn primary_vector(&self) -> Option<&Vec<f32>> {
        self.embeddings.first()
    }
}

/// Health and connectivity check result for an AI provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiHealthStatus {
    /// Whether the provider is reachable and operational.
    pub is_healthy: bool,
    /// Provider type.
    pub provider: AiProviderType,
    /// Descriptive status or error message.
    pub status_message: String,
    /// Number of available models reported by provider.
    pub available_models_count: usize,
    /// Roundtrip ping latency in milliseconds.
    pub latency_ms: u64,
}

impl AiHealthStatus {
    /// Create a healthy status.
    pub fn healthy(provider: AiProviderType, status_message: impl Into<String>) -> Self {
        Self {
            is_healthy: true,
            provider,
            status_message: status_message.into(),
            available_models_count: 1,
            latency_ms: 0,
        }
    }

    /// Create a degraded status.
    pub fn degraded(provider: AiProviderType, status_message: impl Into<String>) -> Self {
        Self {
            is_healthy: false,
            provider,
            status_message: status_message.into(),
            available_models_count: 0,
            latency_ms: 0,
        }
    }
}

/// Focus area for document summarization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryFocus {
    /// General overview of document content and purpose.
    #[default]
    General,
    /// Architecture, crate structure, design patterns, and boundaries.
    Architecture,
    /// Security postures, authentication, threat models, and data flows.
    Security,
    /// REST/RPC API endpoints, request/response schemas, and error codes.
    ApiOverview,
    /// Key high-level insights and takeaways.
    KeyInsights,
}

/// Request for high-level document summarization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummarizationRequest {
    /// Document identifier or source URI.
    pub document_id: String,
    /// Title of the document.
    pub title: String,
    /// Text content or concatenated section bodies to summarize.
    pub content: String,
    /// Specific analytical focus.
    pub focus: SummaryFocus,
    /// Target maximum summary tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_summary_tokens: Option<usize>,
    /// Optional specific model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl DocumentSummarizationRequest {
    /// Create a new document summarization request.
    pub fn new(
        document_id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            title: title.into(),
            content: content.into(),
            focus: SummaryFocus::General,
            max_summary_tokens: None,
            model: None,
        }
    }

    /// Set summary focus.
    pub fn with_focus(mut self, focus: SummaryFocus) -> Self {
        self.focus = focus;
        self
    }

    /// Set max summary tokens.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_summary_tokens = Some(max_tokens);
        self
    }

    /// Set model override.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// Output summary and metadata from document summarization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummarizationResponse {
    /// Generated markdown summary.
    pub summary: String,
    /// Identified key topics or tags.
    pub key_topics: Vec<String>,
    /// Token usage.
    pub token_usage: TokenUsage,
    /// Model used.
    pub model: String,
    /// Latency in milliseconds.
    pub latency_ms: u64,
}

/// Request to enrich an individual document section with AI-generated metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionEnrichmentRequest {
    /// Current section title if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_title: Option<String>,
    /// Kind of section (e.g. "Prose", "CodeFile", "Table").
    pub section_kind: String,
    /// Section body content.
    pub content: String,
    /// File path if from a codebase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Programming or markup language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl SectionEnrichmentRequest {
    /// Create a new section enrichment request.
    pub fn new(section_kind: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            section_title: None,
            section_kind: section_kind.into(),
            content: content.into(),
            file_path: None,
            language: None,
            model: None,
        }
    }

    /// Set section title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.section_title = Some(title.into());
        self
    }

    /// Set file path and language.
    pub fn with_source(
        mut self,
        file_path: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        self.file_path = Some(file_path.into());
        self.language = Some(language.into());
        self
    }
}

/// Metadata generated from section enrichment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionEnrichmentResponse {
    /// Suggested or normalized descriptive title.
    pub suggested_title: String,
    /// 1-2 sentence concise description of section purpose.
    pub concise_description: String,
    /// Relevant classification tags.
    pub tags: Vec<String>,
    /// Token usage.
    pub token_usage: TokenUsage,
    /// Model used.
    pub model: String,
    /// Latency in milliseconds.
    pub latency_ms: u64,
}
