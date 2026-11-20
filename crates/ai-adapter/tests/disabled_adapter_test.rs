//! Tests verifying DisabledAiAdapter's zero-key fallback behavior.

use web_repo_to_ai_ai_adapter::{
    AiAdapter, AiAdapterError, AiProviderType, ChatCompletionRequest, ChatMessage,
    DisabledAiAdapter, DocumentSummarizationRequest, EmbeddingRequest, SectionEnrichmentRequest,
};

#[tokio::test]
async fn test_disabled_adapter_lifecycle_and_flags() {
    let adapter = DisabledAiAdapter::new();

    assert!(!adapter.is_enabled());
    assert_eq!(adapter.provider_type(), AiProviderType::Disabled);

    let health = adapter.health_check().await.expect("health check");
    assert!(!health.is_healthy);
    assert_eq!(health.provider, AiProviderType::Disabled);
    assert!(health.status_message.contains("zero-key"));
    assert_eq!(health.available_models_count, 0);

    let models = adapter.list_models().await.expect("list models");
    assert!(models.is_empty());
}

#[tokio::test]
async fn test_disabled_adapter_rejects_chat_cleanly() {
    let adapter = DisabledAiAdapter::new();

    let req = ChatCompletionRequest::new("any-model", vec![ChatMessage::user("Hello")]);
    let err = adapter.complete_chat(req).await.unwrap_err();

    match err {
        AiAdapterError::ProviderDisabled(msg) => {
            assert!(msg.contains("zero-key"));
        }
        other => panic!("Expected ProviderDisabled error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_disabled_adapter_rejects_embeddings_cleanly() {
    let adapter = DisabledAiAdapter::new();

    let req = EmbeddingRequest::single("any-model", "Test text");
    let err = adapter.generate_embeddings(req).await.unwrap_err();

    match err {
        AiAdapterError::ProviderDisabled(msg) => {
            assert!(msg.contains("Zero-key"));
        }
        other => panic!("Expected ProviderDisabled error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_disabled_adapter_rejects_summarization_and_enrichment() {
    let adapter = DisabledAiAdapter::new();

    let sum_req = DocumentSummarizationRequest::new("doc-1", "Title", "Body");
    let sum_err = adapter.summarize_document(sum_req).await.unwrap_err();
    assert!(matches!(sum_err, AiAdapterError::ProviderDisabled(_)));

    let enrich_req = SectionEnrichmentRequest::new("CodeFile", "let x = 1;");
    let enrich_err = adapter.enrich_section(enrich_req).await.unwrap_err();
    assert!(matches!(enrich_err, AiAdapterError::ProviderDisabled(_)));
}
