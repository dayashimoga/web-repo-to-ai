//! Tests verifying OllamaAiAdapter wire protocol, request serialization, and error mapping.

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use web_repo_to_ai_ai_adapter::{
    AiAdapter, AiAdapterError, AiProviderType, ChatCompletionRequest, ChatMessage,
    EmbeddingRequest, OllamaAiAdapter,
};

struct MockOllamaServer {
    addr: SocketAddr,
    chat_calls: Arc<AtomicUsize>,
    batch_embed_calls: Arc<AtomicUsize>,
    legacy_embed_calls: Arc<AtomicUsize>,
}

impl MockOllamaServer {
    async fn spawn(simulate_no_batch_embed: bool, simulate_error_500: bool) -> Self {
        let chat_calls = Arc::new(AtomicUsize::new(0));
        let batch_embed_calls = Arc::new(AtomicUsize::new(0));
        let legacy_embed_calls = Arc::new(AtomicUsize::new(0));

        let chat_calls_c = Arc::clone(&chat_calls);
        let batch_calls_c = Arc::clone(&batch_embed_calls);
        let legacy_calls_c = Arc::clone(&legacy_embed_calls);

        let app = Router::new()
            .route(
                "/api/tags",
                get(move || async move {
                    if simulate_error_500 {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response();
                    }
                    Json(json!({
                        "models": [
                            {
                                "name": "llama3.2:latest",
                                "model": "llama3.2:latest",
                                "details": { "family": "llama", "parameter_size": "3B" }
                            },
                            {
                                "name": "nomic-embed-text:latest",
                                "model": "nomic-embed-text:latest",
                                "details": { "family": "nomic", "parameter_size": "137M" }
                            }
                        ]
                    }))
                    .into_response()
                }),
            )
            .route(
                "/api/chat",
                post(move |Json(payload): Json<Value>| {
                    let calls = Arc::clone(&chat_calls_c);
                    async move {
                        if simulate_error_500 {
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Inference crashed")
                                .into_response();
                        }
                        calls.fetch_add(1, Ordering::SeqCst);

                        let format = payload.get("format").and_then(|f| f.as_str());
                        let response_text = if format == Some("json") {
                            "{\"result\": \"json_ok\"}"
                        } else {
                            "Hello from mock Ollama daemon!"
                        };

                        Json(json!({
                            "model": payload["model"].as_str().unwrap_or("llama3.2"),
                            "created_at": "2026-11-20T12:00:00Z",
                            "message": {
                                "role": "assistant",
                                "content": response_text
                            },
                            "done": true,
                            "prompt_eval_count": 14,
                            "eval_count": 32
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/api/embed",
                post(move |Json(payload): Json<Value>| {
                    let calls = Arc::clone(&batch_calls_c);
                    async move {
                        if simulate_no_batch_embed {
                            return (StatusCode::NOT_FOUND, "Endpoint not found").into_response();
                        }
                        calls.fetch_add(1, Ordering::SeqCst);

                        let inputs = payload["input"].as_array().expect("array input");
                        let embeddings: Vec<Vec<f32>> = inputs
                            .iter()
                            .map(|_| vec![0.1f32, 0.2f32, 0.3f32])
                            .collect();

                        Json(json!({
                            "model": payload["model"].as_str().unwrap_or("nomic-embed-text"),
                            "embeddings": embeddings,
                            "prompt_eval_count": 8
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/api/embeddings",
                post(move |Json(_payload): Json<Value>| {
                    let calls = Arc::clone(&legacy_calls_c);
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Json(json!({
                            "embedding": [0.4f32, 0.5f32, 0.6f32]
                        }))
                        .into_response()
                    }
                }),
            );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            addr,
            chat_calls,
            batch_embed_calls,
            legacy_embed_calls,
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

#[tokio::test]
async fn test_ollama_health_check_success() {
    let server = MockOllamaServer::spawn(false, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    assert_eq!(adapter.provider_type(), AiProviderType::Ollama);
    assert!(adapter.is_enabled());

    let health = adapter.health_check().await.expect("health check");
    assert!(health.is_healthy);
    assert_eq!(health.provider, AiProviderType::Ollama);
    assert_eq!(health.available_models_count, 2);
    assert!(health.status_message.contains("Ollama daemon operational"));
}

#[tokio::test]
async fn test_ollama_list_models() {
    let server = MockOllamaServer::spawn(false, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let models = adapter.list_models().await.expect("list models");
    assert_eq!(models.len(), 2);

    let chat_m = models.iter().find(|m| m.id == "llama3.2:latest").unwrap();
    assert!(chat_m.capabilities.supports_chat);
    assert!(!chat_m.capabilities.supports_embeddings);
    assert!(chat_m.description.as_deref().unwrap().contains("llama"));

    let embed_m = models
        .iter()
        .find(|m| m.id == "nomic-embed-text:latest")
        .unwrap();
    assert!(!embed_m.capabilities.supports_chat);
    assert!(embed_m.capabilities.supports_embeddings);
}

#[tokio::test]
async fn test_ollama_complete_chat() {
    let server = MockOllamaServer::spawn(false, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let req = ChatCompletionRequest::new("llama3.2", vec![ChatMessage::user("Hi")]);
    let resp = adapter.complete_chat(req).await.expect("chat response");

    assert_eq!(resp.model, "llama3.2");
    assert_eq!(resp.primary_text(), Some("Hello from mock Ollama daemon!"));
    assert_eq!(resp.usage.prompt_tokens, 14);
    assert_eq!(resp.usage.completion_tokens, 32);
    assert_eq!(resp.usage.total_tokens, 46);
    assert_eq!(server.chat_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_ollama_complete_chat_json_format() {
    let server = MockOllamaServer::spawn(false, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let req = ChatCompletionRequest::new("llama3.2", vec![ChatMessage::user("JSON pls")])
        .with_json_format();

    let resp = adapter.complete_chat(req).await.expect("chat json");
    assert_eq!(resp.primary_text(), Some("{\"result\": \"json_ok\"}"));
}

#[tokio::test]
async fn test_ollama_batch_embeddings() {
    let server = MockOllamaServer::spawn(false, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let req = EmbeddingRequest::new(
        "nomic-embed-text",
        vec!["Text A".to_string(), "Text B".to_string()],
    );

    let resp = adapter.generate_embeddings(req).await.expect("embeddings");
    assert_eq!(resp.embeddings.len(), 2);
    assert_eq!(resp.dimensions, 3);
    assert_eq!(resp.embeddings[0], vec![0.1, 0.2, 0.3]);
    assert_eq!(server.batch_embed_calls.load(Ordering::SeqCst), 1);
    assert_eq!(server.legacy_embed_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_ollama_legacy_embeddings_fallback() {
    // Server simulates 404 on /api/embed, falling back to /api/embeddings
    let server = MockOllamaServer::spawn(true, false).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let req = EmbeddingRequest::new(
        "nomic-embed-text",
        vec!["Text 1".to_string(), "Text 2".to_string()],
    );

    let resp = adapter
        .generate_embeddings(req)
        .await
        .expect("embeddings fallback");
    assert_eq!(resp.embeddings.len(), 2);
    assert_eq!(resp.dimensions, 3);
    assert_eq!(resp.embeddings[0], vec![0.4, 0.5, 0.6]);
    assert_eq!(server.legacy_embed_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_ollama_server_error_mapping() {
    let server = MockOllamaServer::spawn(false, true).await;
    let adapter = OllamaAiAdapter::new(server.url()).expect("create adapter");

    let req = ChatCompletionRequest::new("llama3.2", vec![ChatMessage::user("Hi")]);
    let err = adapter.complete_chat(req).await.unwrap_err();

    match err {
        AiAdapterError::ServerUnavailable {
            status_code,
            message,
        } => {
            assert_eq!(status_code, Some(500));
            assert!(message.contains("Inference crashed"));
        }
        other => panic!("Expected ServerUnavailable, got {:?}", other),
    }
}

#[tokio::test]
async fn test_ollama_disabled_state() {
    let adapter = OllamaAiAdapter::new("http://localhost:11434")
        .expect("adapter")
        .with_enabled(false);

    assert!(!adapter.is_enabled());

    let health = adapter.health_check().await.expect("health");
    assert!(!health.is_healthy);

    let chat_err = adapter
        .complete_chat(ChatCompletionRequest::new(
            "llama",
            vec![ChatMessage::user("Hi")],
        ))
        .await
        .unwrap_err();
    assert!(matches!(chat_err, AiAdapterError::ProviderDisabled(_)));
}
