//! Provider-independent AI adapter trait and implementations for optional LLM assistance.
//!
//! Provides abstract interfaces, domain schemas, zero-key fallback implementations,
//! in-memory test mocks, and thread-safe runtime registries for AI integration.
//!
//! In accordance with Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT ("Zero Mandatory
//! External AI / Paid Dependencies"), all core document extraction, normalization,
//! deduplication, chunking, and export logic operate locally and deterministically
//! on CPU without AI assistance. AI provider adapters serve solely as optional,
//! opt-in post-processing extensions.

pub mod client;
pub mod disabled;
pub mod error;
pub mod mock;
pub mod models;
pub mod ollama;
pub mod openai;
pub mod registry;
pub mod traits;

// Re-exports
pub use client::AiClientConfig;
pub use disabled::DisabledAiAdapter;
pub use error::AiAdapterError;
pub use mock::{MockAiAdapter, MockCallTracker};
pub use models::{
    AiCapabilities, AiHealthStatus, AiModelDescriptor, AiProviderType, ChatChoice,
    ChatCompletionRequest, ChatCompletionResponse, ChatFinishReason, ChatMessage,
    ChatResponseFormat, ChatRole, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
    SummaryFocus, TokenUsage,
};
pub use ollama::OllamaAiAdapter;
pub use openai::OpenAiAiAdapter;
pub use registry::AiAdapterRegistry;
pub use traits::AiAdapter;
