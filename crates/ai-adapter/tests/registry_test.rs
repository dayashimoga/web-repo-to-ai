//! Tests verifying AiAdapterRegistry dynamic switching, fallback, and thread safety.

use std::sync::Arc;
use web_repo_to_ai_ai_adapter::{
    AiAdapterError, AiAdapterRegistry, AiProviderType, ChatCompletionRequest, ChatMessage,
    EmbeddingRequest, MockAiAdapter,
};

#[tokio::test]
async fn test_registry_initializes_with_disabled_fallback() {
    let registry = AiAdapterRegistry::new();

    assert!(!registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Disabled);

    let health = registry.health_check().await.expect("health");
    assert!(!health.is_healthy);
    assert_eq!(health.provider, AiProviderType::Disabled);

    let models = registry.list_models().await.expect("models");
    assert!(models.is_empty());

    let chat_err = registry
        .complete_chat(ChatCompletionRequest::new(
            "any",
            vec![ChatMessage::user("Hi")],
        ))
        .await
        .unwrap_err();
    assert!(matches!(chat_err, AiAdapterError::ProviderDisabled(_)));

    let embed_err = registry
        .generate_embeddings(EmbeddingRequest::single("any", "text"))
        .await
        .unwrap_err();
    assert!(matches!(embed_err, AiAdapterError::ProviderDisabled(_)));
}

#[tokio::test]
async fn test_registry_dynamic_adapter_switching() {
    let registry = AiAdapterRegistry::new();
    assert!(!registry.is_enabled());

    // Switch to active mock adapter
    let mock = Arc::new(MockAiAdapter::new());
    registry.set_adapter(mock.clone());

    assert!(registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Mock);

    // Call chat successfully through registry delegation
    let resp = registry
        .complete_chat(ChatCompletionRequest::new(
            "mock-chat",
            vec![ChatMessage::user("Hello")],
        ))
        .await
        .expect("chat call");
    assert!(!resp.choices.is_empty());
    assert_eq!(mock.tracker().chat_call_count(), 1);

    // Disable registry
    registry.disable();
    assert!(!registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Disabled);

    // Subsequent calls are rejected with ProviderDisabled
    let err = registry
        .complete_chat(ChatCompletionRequest::new(
            "mock-chat",
            vec![ChatMessage::user("Hello")],
        ))
        .await
        .unwrap_err();
    assert!(matches!(err, AiAdapterError::ProviderDisabled(_)));
}

#[tokio::test]
async fn test_registry_concurrent_thread_safety() {
    let registry = Arc::new(AiAdapterRegistry::new());
    let mock = Arc::new(MockAiAdapter::new());
    registry.set_adapter(mock.clone());

    let mut handles = Vec::new();

    for i in 0..20 {
        let reg = Arc::clone(&registry);
        let h = tokio::spawn(async move {
            if i == 10 {
                // One task toggles adapter temporarily
                reg.disable();
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                reg.set_adapter(Arc::new(MockAiAdapter::new()));
            } else {
                let req = ChatCompletionRequest::new(
                    "mock-chat",
                    vec![ChatMessage::user(format!("Req {}", i))],
                );
                // May succeed or return ProviderDisabled during the brief disable window
                let _ = reg.complete_chat(req).await;
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.await.expect("join handle");
    }

    // Must be in a valid operational state at the end
    assert!(registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Mock);
}
