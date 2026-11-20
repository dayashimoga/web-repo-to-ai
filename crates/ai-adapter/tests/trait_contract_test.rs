//! Tests verifying the AI adapter trait contract, domain models, and error taxonomy.

use web_repo_to_ai_ai_adapter::{
    AiAdapterError, AiHealthStatus, AiModelDescriptor, AiProviderType, ChatChoice,
    ChatCompletionRequest, ChatCompletionResponse, ChatFinishReason, ChatMessage,
    ChatResponseFormat, ChatRole, DocumentSummarizationRequest, DocumentSummarizationResponse,
    EmbeddingRequest, EmbeddingResponse, SectionEnrichmentRequest, SectionEnrichmentResponse,
    SummaryFocus, TokenUsage,
};

#[test]
fn test_chat_message_constructors() {
    let sys = ChatMessage::system("You are an assistant");
    assert_eq!(sys.role, ChatRole::System);
    assert_eq!(sys.content, "You are an assistant");
    assert!(sys.name.is_none());

    let user = ChatMessage::user("Hello AI");
    assert_eq!(user.role, ChatRole::User);
    assert_eq!(user.content, "Hello AI");

    let asst = ChatMessage::assistant("Hello human");
    assert_eq!(asst.role, ChatRole::Assistant);

    let tool = ChatMessage::tool("Result: 42", "calc");
    assert_eq!(tool.role, ChatRole::Tool);
    assert_eq!(tool.name.as_deref(), Some("calc"));
}

#[test]
fn test_chat_completion_request_builder() {
    let req = ChatCompletionRequest::new(
        "gpt-4o",
        vec![
            ChatMessage::system("system instructions"),
            ChatMessage::user("user query"),
        ],
    )
    .with_temperature(0.7)
    .with_max_tokens(1024)
    .with_json_format()
    .with_stop_sequence("END")
    .with_seed(42);

    assert_eq!(req.model, "gpt-4o");
    assert_eq!(req.messages.len(), 2);
    assert_eq!(req.temperature, Some(0.7));
    assert_eq!(req.max_tokens, Some(1024));
    assert_eq!(req.response_format, Some(ChatResponseFormat::JsonObject));
    assert_eq!(req.stop_sequences, vec!["END"]);
    assert_eq!(req.seed, Some(42));
}

#[test]
fn test_token_usage_arithmetic() {
    let usage = TokenUsage::new(150, 50);
    assert_eq!(usage.prompt_tokens, 150);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 200);

    // Saturating add on extreme bounds
    let extreme = TokenUsage::new(usize::MAX, 10);
    assert_eq!(extreme.total_tokens, usize::MAX);
}

#[test]
fn test_chat_completion_response_helpers() {
    let choice = ChatChoice {
        index: 0,
        message: ChatMessage::assistant("Generated response"),
        finish_reason: ChatFinishReason::Stop,
    };

    let resp = ChatCompletionResponse {
        id: "cmpl-123".into(),
        model: "llama3".into(),
        choices: vec![choice],
        usage: TokenUsage::new(10, 20),
        created_at: chrono::Utc::now(),
        latency_ms: 45,
    };

    assert_eq!(resp.primary_text(), Some("Generated response"));
    assert_eq!(
        resp.primary_message(),
        Some(&ChatMessage::assistant("Generated response"))
    );
}

#[test]
fn test_embedding_request_and_response_helpers() {
    let req = EmbeddingRequest::single("text-embed-3", "Hello world").with_dimensions(512);
    assert_eq!(req.model, "text-embed-3");
    assert_eq!(req.inputs, vec!["Hello world"]);
    assert_eq!(req.dimensions, Some(512));

    let resp = EmbeddingResponse {
        model: "text-embed-3".into(),
        embeddings: vec![vec![0.1, 0.2, 0.3]],
        usage: TokenUsage::new(2, 0),
        dimensions: 3,
        latency_ms: 12,
    };

    assert_eq!(resp.primary_vector(), Some(&vec![0.1, 0.2, 0.3]));
}

#[test]
fn test_ai_adapter_error_classification() {
    let rate_limit = AiAdapterError::RateLimitExceeded {
        retry_after_secs: Some(30),
        message: "Too many requests".into(),
    };
    assert!(rate_limit.is_retryable());
    assert_eq!(rate_limit.status_code(), Some(429));

    let net_err = AiAdapterError::NetworkError("Connection refused".into());
    assert!(net_err.is_retryable());
    assert_eq!(net_err.status_code(), Some(500));

    let disabled = AiAdapterError::ProviderDisabled("Zero key".into());
    assert!(!disabled.is_retryable());
    assert_eq!(disabled.status_code(), Some(403));

    let auth = AiAdapterError::AuthenticationRequired("Missing key".into());
    assert!(!auth.is_retryable());
    assert_eq!(auth.status_code(), Some(401));

    let ctx_len = AiAdapterError::ContextLengthExceeded {
        requested_tokens: 130_000,
        max_tokens: 128_000,
        message: "Prompt too long".into(),
    };
    assert!(!ctx_len.is_retryable());
    assert_eq!(ctx_len.status_code(), Some(413));

    let not_found = AiAdapterError::ModelNotFound {
        model: "nonexistent".into(),
        message: "Model not found".into(),
    };
    assert!(!not_found.is_retryable());
    assert_eq!(not_found.status_code(), Some(404));
}

#[test]
fn test_serde_json_roundtrip_all_models() {
    // 1. Model descriptor
    let desc = AiModelDescriptor::new_chat(
        "test-model",
        AiProviderType::Ollama,
        "Test Model",
        32000,
        2048,
    );
    let desc_json = serde_json::to_string(&desc).expect("serialize desc");
    let desc_back: AiModelDescriptor = serde_json::from_str(&desc_json).expect("deserialize desc");
    assert_eq!(desc, desc_back);

    // 2. Chat completion request and response
    let chat_req = ChatCompletionRequest::new("model-1", vec![ChatMessage::user("Hi")]);
    let req_json = serde_json::to_string(&chat_req).expect("serialize req");
    let req_back: ChatCompletionRequest = serde_json::from_str(&req_json).expect("deserialize req");
    assert_eq!(chat_req, req_back);

    // 3. Document summarization request and response
    let sum_req = DocumentSummarizationRequest::new("doc-1", "Doc Title", "Body text")
        .with_focus(SummaryFocus::Architecture)
        .with_max_tokens(500);
    let sum_json = serde_json::to_string(&sum_req).expect("serialize sum_req");
    let sum_back: DocumentSummarizationRequest =
        serde_json::from_str(&sum_json).expect("deserialize sum_req");
    assert_eq!(sum_req, sum_back);

    let sum_resp = DocumentSummarizationResponse {
        summary: "Summary text".into(),
        key_topics: vec!["topic1".into(), "topic2".into()],
        token_usage: TokenUsage::new(100, 50),
        model: "model-1".into(),
        latency_ms: 120,
    };
    let sum_resp_json = serde_json::to_string(&sum_resp).expect("serialize sum_resp");
    let sum_resp_back: DocumentSummarizationResponse =
        serde_json::from_str(&sum_resp_json).expect("deserialize sum_resp");
    assert_eq!(sum_resp, sum_resp_back);

    // 4. Section enrichment request and response
    let enrich_req = SectionEnrichmentRequest::new("CodeFile", "fn main() {}")
        .with_source("src/main.rs", "rust")
        .with_title("Main Entrypoint");
    let enrich_json = serde_json::to_string(&enrich_req).expect("serialize enrich_req");
    let enrich_back: SectionEnrichmentRequest =
        serde_json::from_str(&enrich_json).expect("deserialize enrich_req");
    assert_eq!(enrich_req, enrich_back);

    let enrich_resp = SectionEnrichmentResponse {
        suggested_title: "Entrypoint".into(),
        concise_description: "Rust entry function".into(),
        tags: vec!["main".into(), "rust".into()],
        token_usage: TokenUsage::new(20, 10),
        model: "model-1".into(),
        latency_ms: 80,
    };
    let enrich_resp_json = serde_json::to_string(&enrich_resp).expect("serialize enrich_resp");
    let enrich_resp_back: SectionEnrichmentResponse =
        serde_json::from_str(&enrich_resp_json).expect("deserialize enrich_resp");
    assert_eq!(enrich_resp, enrich_resp_back);

    // 5. Health status
    let health = AiHealthStatus {
        is_healthy: true,
        provider: AiProviderType::Mock,
        status_message: "OK".into(),
        available_models_count: 5,
        latency_ms: 15,
    };
    let health_json = serde_json::to_string(&health).expect("serialize health");
    let health_back: AiHealthStatus =
        serde_json::from_str(&health_json).expect("deserialize health");
    assert_eq!(health, health_back);
}
