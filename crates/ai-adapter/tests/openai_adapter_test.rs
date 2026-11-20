//! Tests verifying OpenAiAiAdapter wire protocol, Bearer auth, schemas, and error mapping.

use axum::extract::Json;
use axum::http::{HeaderMap, StatusCode};
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
    EmbeddingRequest, OpenAiAiAdapter,
};

struct MockOpenAiServer {
    addr: SocketAddr,
    chat_calls: Arc<AtomicUsize>,
    embed_calls: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct ServerFlags {
    expected_token: String,
    simulate_rate_limit: bool,
    simulate_context_overflow: bool,
}

impl MockOpenAiServer {
    async fn spawn(flags: ServerFlags) -> Self {
        let chat_calls = Arc::new(AtomicUsize::new(0));
        let embed_calls = Arc::new(AtomicUsize::new(0));

        let chat_calls_c = Arc::clone(&chat_calls);
        let embed_calls_c = Arc::clone(&embed_calls);
        let flags_c = flags.clone();

        let app = Router::new()
            .route(
                "/models",
                get(move |headers: HeaderMap| {
                    let flags = flags_c.clone();
                    async move {
                        // Check auth
                        if !flags.expected_token.is_empty() {
                            let auth = headers.get("authorization").and_then(|h| h.to_str().ok());
                            let expected = format!("Bearer {}", flags.expected_token);
                            if auth != Some(&expected) {
                                return (
                                    StatusCode::UNAUTHORIZED,
                                    Json(json!({ "error": { "message": "Invalid API key" } })),
                                )
                                    .into_response();
                            }
                        }

                        Json(json!({
                            "object": "list",
                            "data": [
                                { "id": "gpt-4o-mini", "object": "model", "owned_by": "system" },
                                { "id": "text-embedding-3-small", "object": "model", "owned_by": "system" }
                            ]
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/chat/completions",
                post(move |headers: HeaderMap, Json(payload): Json<Value>| {
                    let calls = Arc::clone(&chat_calls_c);
                    let flags = flags.clone();
                    async move {
                        // Check auth
                        if !flags.expected_token.is_empty() {
                            let auth = headers.get("authorization").and_then(|h| h.to_str().ok());
                            let expected = format!("Bearer {}", flags.expected_token);
                            if auth != Some(&expected) {
                                return (
                                    StatusCode::UNAUTHORIZED,
                                    Json(json!({ "error": { "message": "Invalid API key" } })),
                                )
                                    .into_response();
                            }
                        }

                        if flags.simulate_rate_limit {
                            let mut resp_headers = axum::http::HeaderMap::new();
                            resp_headers.insert("retry-after", "45".parse().unwrap());
                            return (
                                StatusCode::TOO_MANY_REQUESTS,
                                resp_headers,
                                Json(json!({ "error": { "message": "Rate limit exceeded" } })),
                            )
                                .into_response();
                        }

                        if flags.simulate_context_overflow {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(json!({
                                    "error": {
                                        "message": "This model's maximum context length is 128000 tokens. However, you requested 130000 tokens."
                                    }
                                })),
                            )
                                .into_response();
                        }

                        calls.fetch_add(1, Ordering::SeqCst);

                        Json(json!({
                            "id": "chatcmpl-mock-12345",
                            "object": "chat.completion",
                            "created": 1732100000,
                            "model": payload["model"].as_str().unwrap_or("gpt-4o-mini"),
                            "choices": [
                                {
                                    "index": 0,
                                    "message": {
                                        "role": "assistant",
                                        "content": "Mock OpenAI response message"
                                    },
                                    "finish_reason": "stop"
                                }
                            ],
                            "usage": {
                                "prompt_tokens": 15,
                                "completion_tokens": 30,
                                "total_tokens": 45
                            }
                        }))
                        .into_response()
                    }
                }),
            )
            .route(
                "/embeddings",
                post(move |_headers: HeaderMap, Json(payload): Json<Value>| {
                    let calls = Arc::clone(&embed_calls_c);
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let inputs = payload["input"].as_array().expect("array input");

                        let data: Vec<Value> = inputs
                            .iter()
                            .enumerate()
                            .map(|(idx, _)| {
                                json!({
                                    "object": "embedding",
                                    "index": idx,
                                    "embedding": [0.05, -0.05, 0.99]
                                })
                            })
                            .collect();

                        Json(json!({
                            "object": "list",
                            "data": data,
                            "model": payload["model"].as_str().unwrap_or("text-embedding-3-small"),
                            "usage": {
                                "prompt_tokens": 6,
                                "total_tokens": 6
                            }
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
            embed_calls,
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

#[tokio::test]
async fn test_openai_health_check_success() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "sk-test123".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::new(server.url(), Some("sk-test123".into())).expect("adapter");

    assert_eq!(adapter.provider_type(), AiProviderType::OpenAiCompatible);
    assert!(adapter.is_enabled());

    let health = adapter.health_check().await.expect("health check");
    assert!(health.is_healthy);
    assert_eq!(health.provider, AiProviderType::OpenAiCompatible);
    assert_eq!(health.available_models_count, 2);
    assert!(health.status_message.contains("OpenAI-compatible"));
}

#[tokio::test]
async fn test_openai_auth_failure_401() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "correct-token".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    // Provide incorrect token
    let adapter = OpenAiAiAdapter::new(server.url(), Some("wrong-token".into())).expect("adapter");

    let err = adapter.health_check().await.unwrap_err();
    assert!(matches!(err, AiAdapterError::AuthenticationRequired(_)));
}

#[tokio::test]
async fn test_openai_list_models() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "sk-key".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::new(server.url(), Some("sk-key".into())).expect("adapter");
    let models = adapter.list_models().await.expect("list models");

    assert_eq!(models.len(), 2);
    let chat_m = models.iter().find(|m| m.id == "gpt-4o-mini").unwrap();
    assert!(chat_m.capabilities.supports_chat);
    assert!(!chat_m.capabilities.supports_embeddings);

    let embed_m = models
        .iter()
        .find(|m| m.id == "text-embedding-3-small")
        .unwrap();
    assert!(embed_m.capabilities.supports_embeddings);
}

#[tokio::test]
async fn test_openai_complete_chat_and_token_usage() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "sk-key".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::new(server.url(), Some("sk-key".into())).expect("adapter");

    let req = ChatCompletionRequest::new("gpt-4o-mini", vec![ChatMessage::user("Hello")])
        .with_temperature(0.5)
        .with_max_tokens(100);

    let resp = adapter.complete_chat(req).await.expect("chat response");

    assert_eq!(resp.id, "chatcmpl-mock-12345");
    assert_eq!(resp.model, "gpt-4o-mini");
    assert_eq!(resp.primary_text(), Some("Mock OpenAI response message"));
    assert_eq!(resp.usage.prompt_tokens, 15);
    assert_eq!(resp.usage.completion_tokens, 30);
    assert_eq!(resp.usage.total_tokens, 45);
    assert_eq!(server.chat_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_openai_generate_embeddings() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::for_local(server.url()).expect("adapter");

    let req = EmbeddingRequest::new(
        "text-embedding-3-small",
        vec!["Input doc 1".to_string(), "Input doc 2".to_string()],
    );

    let resp = adapter.generate_embeddings(req).await.expect("embeddings");
    assert_eq!(resp.embeddings.len(), 2);
    assert_eq!(resp.dimensions, 3);
    assert_eq!(resp.embeddings[0], vec![0.05, -0.05, 0.99]);
    assert_eq!(server.embed_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_openai_rate_limit_429_with_retry_after() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "sk-key".into(),
        simulate_rate_limit: true,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::new(server.url(), Some("sk-key".into())).expect("adapter");

    let req = ChatCompletionRequest::new("gpt-4o-mini", vec![ChatMessage::user("Hi")]);
    let err = adapter.complete_chat(req).await.unwrap_err();

    match err {
        AiAdapterError::RateLimitExceeded {
            retry_after_secs,
            message,
        } => {
            assert_eq!(retry_after_secs, Some(45));
            assert!(message.contains("Rate limit exceeded"));
        }
        other => panic!("Expected RateLimitExceeded, got {:?}", other),
    }
}

#[tokio::test]
async fn test_openai_context_length_exceeded_error() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "sk-key".into(),
        simulate_rate_limit: false,
        simulate_context_overflow: true,
    })
    .await;

    let adapter = OpenAiAiAdapter::new(server.url(), Some("sk-key".into())).expect("adapter");

    let req = ChatCompletionRequest::new("gpt-4o-mini", vec![ChatMessage::user("Huge text")]);
    let err = adapter.complete_chat(req).await.unwrap_err();

    match err {
        AiAdapterError::ContextLengthExceeded { message, .. } => {
            assert!(message.contains("maximum context length"));
        }
        other => panic!("Expected ContextLengthExceeded, got {:?}", other),
    }
}

#[tokio::test]
async fn test_openai_local_endpoint_without_key() {
    let server = MockOpenAiServer::spawn(ServerFlags {
        expected_token: "".into(), // Local vLLM/LM Studio server without auth
        simulate_rate_limit: false,
        simulate_context_overflow: false,
    })
    .await;

    let adapter = OpenAiAiAdapter::for_local(server.url()).expect("adapter");
    let health = adapter.health_check().await.expect("health check");
    assert!(health.is_healthy);
    assert_eq!(health.available_models_count, 2);
}
