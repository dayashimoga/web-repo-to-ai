//! Tests verifying MockAiAdapter's capabilities, call tracking, determinism, and error handling.

use std::sync::Arc;
use web_repo_to_ai_ai_adapter::{
    AiAdapter, AiAdapterError, AiProviderType, ChatCompletionRequest, ChatMessage,
    DocumentSummarizationRequest, EmbeddingRequest, MockAiAdapter, SectionEnrichmentRequest,
    SummaryFocus,
};

#[tokio::test]
async fn test_mock_chat_completion_default_heuristic() {
    let adapter = MockAiAdapter::new();
    assert!(adapter.is_enabled());
    assert_eq!(adapter.provider_type(), AiProviderType::Mock);

    let req =
        ChatCompletionRequest::new("mock-chat", vec![ChatMessage::user("Explain Rust traits")]);
    let resp = adapter.complete_chat(req).await.expect("complete chat");

    assert_eq!(resp.model, "mock-chat");
    assert!(!resp.choices.is_empty());
    let text = resp.primary_text().expect("primary text");
    assert!(text.contains("Mock response from model 'mock-chat'"));
    assert!(resp.usage.prompt_tokens > 0);
    assert!(resp.usage.completion_tokens > 0);

    let tracker = adapter.tracker();
    assert_eq!(tracker.chat_call_count(), 1);
    assert_eq!(tracker.embedding_call_count(), 0);
}

#[tokio::test]
async fn test_mock_chat_completion_scripted_response() {
    let adapter = MockAiAdapter::new()
        .with_scripted_chat("special prompt", "Custom scripted output for test");

    let req = ChatCompletionRequest::new(
        "mock-chat",
        vec![ChatMessage::user("Here is a special prompt for you")],
    );
    let resp = adapter.complete_chat(req).await.expect("complete chat");
    assert_eq!(resp.primary_text(), Some("Custom scripted output for test"));
}

#[tokio::test]
async fn test_mock_chat_completion_default_override() {
    let adapter = MockAiAdapter::new().with_default_chat_response("Universal canned response");

    let req =
        ChatCompletionRequest::new("mock-chat", vec![ChatMessage::user("Any random question")]);
    let resp = adapter.complete_chat(req).await.expect("complete chat");
    assert_eq!(resp.primary_text(), Some("Universal canned response"));
}

#[tokio::test]
async fn test_mock_empty_messages_validation_error() {
    let adapter = MockAiAdapter::new();
    let req = ChatCompletionRequest::new("mock-chat", vec![]);
    let err = adapter.complete_chat(req).await.unwrap_err();
    assert!(matches!(err, AiAdapterError::InvalidRequest(_)));
}

#[tokio::test]
async fn test_mock_document_summarization_with_topic_extraction() {
    let adapter = MockAiAdapter::new();

    let doc_content = "This document describes the Architecture of the storage engine.\n\
                       The storage engine uses an LSM tree, Write-Ahead Log (WAL), and Bloom filters.";

    let req =
        DocumentSummarizationRequest::new("doc-storage-01", "Storage Architecture", doc_content)
            .with_focus(SummaryFocus::Architecture)
            .with_max_tokens(500);

    let resp = adapter
        .summarize_document(req)
        .await
        .expect("summarize doc");

    assert!(!resp.summary.is_empty());
    assert!(!resp.key_topics.is_empty());
    assert!(resp.key_topics.contains(&"architecture".to_string()));
    assert!(resp.token_usage.total_tokens > 0);
}

#[tokio::test]
async fn test_mock_section_enrichment_with_metadata_parsing() {
    let adapter = MockAiAdapter::new();

    let code_content = "pub struct StorageEngine {\n    db_path: PathBuf,\n}\nimpl StorageEngine {\n    pub fn open() -> Self { ... }\n}";

    let req = SectionEnrichmentRequest::new("CodeFile", code_content)
        .with_source("crates/storage/src/engine.rs", "rust")
        .with_title("StorageEngine struct definition");

    let resp = adapter.enrich_section(req).await.expect("enrich section");

    assert!(!resp.suggested_title.is_empty());
    assert!(!resp.concise_description.is_empty());
    assert!(!resp.tags.is_empty());
    assert!(resp.tags.iter().any(|t| t == "rust" || t == "architecture"));
}

#[tokio::test]
async fn test_mock_deterministic_pseudo_embeddings_repeatability_and_unit_norm() {
    let adapter = MockAiAdapter::new().with_embedding_dimension(384);

    let text_a = "High-performance universal web repository to AI converter";
    let text_b = "Different unrelated text about something else";

    let req_a1 = EmbeddingRequest::single("mock-embed", text_a);
    let resp_a1 = adapter.generate_embeddings(req_a1).await.expect("embed a1");

    let req_a2 = EmbeddingRequest::single("mock-embed", text_a);
    let resp_a2 = adapter.generate_embeddings(req_a2).await.expect("embed a2");

    let req_b = EmbeddingRequest::single("mock-embed", text_b);
    let resp_b = adapter.generate_embeddings(req_b).await.expect("embed b");

    // 1. Bit-for-bit repeatability on identical input
    assert_eq!(resp_a1.embeddings[0], resp_a2.embeddings[0]);

    // 2. Different inputs yield distinct vectors
    assert_ne!(resp_a1.embeddings[0], resp_b.embeddings[0]);

    // 3. Dimensionality adherence
    assert_eq!(resp_a1.dimensions, 384);
    assert_eq!(resp_a1.embeddings[0].len(), 384);

    // 4. L2 unit norm constraint: ||v||_2 ~= 1.0 (+/- 1e-4)
    let sum_sq: f32 = resp_a1.embeddings[0].iter().map(|x| x * x).sum();
    let norm = sum_sq.sqrt();
    assert!(
        (norm - 1.0).abs() < 1e-4,
        "Expected unit norm ~1.0, got {}",
        norm
    );
}

#[tokio::test]
async fn test_mock_scripted_embeddings_override() {
    let custom_vector = vec![0.5, -0.5, std::f32::consts::FRAC_1_SQRT_2];
    let adapter =
        MockAiAdapter::new().with_scripted_embedding("special-term", custom_vector.clone());

    let req = EmbeddingRequest::single("mock-embed", "special-term");
    let resp = adapter.generate_embeddings(req).await.expect("embed");

    assert_eq!(resp.embeddings[0], custom_vector);
}

#[tokio::test]
async fn test_mock_error_injection_and_recovery() {
    let mut adapter = MockAiAdapter::new().with_injected_error(AiAdapterError::RateLimitExceeded {
        retry_after_secs: Some(15),
        message: "Simulated quota limit".into(),
    });

    let req = ChatCompletionRequest::new("mock-chat", vec![ChatMessage::user("Hi")]);
    let err = adapter.complete_chat(req.clone()).await.unwrap_err();
    assert!(matches!(err, AiAdapterError::RateLimitExceeded { .. }));

    // Clear error and recover
    adapter.clear_injected_error();
    let resp = adapter.complete_chat(req).await.expect("recovered chat");
    assert!(!resp.choices.is_empty());
}

#[tokio::test]
async fn test_mock_call_tracker_and_token_accounting() {
    let adapter = MockAiAdapter::new();
    let tracker = adapter.tracker();

    assert_eq!(tracker.chat_call_count(), 0);
    assert_eq!(tracker.embedding_call_count(), 0);
    assert_eq!(tracker.prompt_tokens(), 0);
    assert_eq!(tracker.completion_tokens(), 0);

    // Perform chat call
    let chat_req = ChatCompletionRequest::new("mock-chat", vec![ChatMessage::user("Hello AI")]);
    adapter.complete_chat(chat_req).await.expect("chat");

    assert_eq!(tracker.chat_call_count(), 1);
    assert!(tracker.prompt_tokens() > 0);
    assert!(tracker.completion_tokens() > 0);

    // Perform embedding call
    let embed_req = EmbeddingRequest::single("mock-embed", "Sample input");
    adapter.generate_embeddings(embed_req).await.expect("embed");

    assert_eq!(tracker.embedding_call_count(), 1);

    // Reset tracker
    tracker.reset();
    assert_eq!(tracker.chat_call_count(), 0);
    assert_eq!(tracker.embedding_call_count(), 0);
    assert_eq!(tracker.prompt_tokens(), 0);
    assert_eq!(tracker.completion_tokens(), 0);
}

#[tokio::test]
async fn test_mock_concurrent_multithreaded_execution() {
    let adapter = Arc::new(MockAiAdapter::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                let req = ChatCompletionRequest::new(
                    "mock-chat",
                    vec![ChatMessage::user(format!("Thread task {}", i))],
                );
                let resp = adapter_clone.complete_chat(req).await.expect("chat call");
                assert!(!resp.choices.is_empty());
            } else {
                let req = EmbeddingRequest::single("mock-embed", format!("Thread text {}", i));
                let resp = adapter_clone
                    .generate_embeddings(req)
                    .await
                    .expect("embed call");
                assert_eq!(resp.embeddings.len(), 1);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("task join");
    }

    let tracker = adapter.tracker();
    assert_eq!(tracker.chat_call_count(), 5);
    assert_eq!(tracker.embedding_call_count(), 5);
}
