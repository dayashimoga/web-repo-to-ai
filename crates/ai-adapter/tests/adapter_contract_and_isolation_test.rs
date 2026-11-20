//! Polymorphic contract verification, network isolation, and concurrency stress tests for AI adapters.
//!
//! Enforces Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT:
//! "Zero Mandatory External AI / Paid Dependencies: All core document extraction,
//! normalization, deduplication, chunking, and export logic execute locally on CPU
//! without AI assistance. AI provider adapters serve solely as optional, opt-in
//! post-processing extensions. Core extraction MUST ALWAYS succeed without AI keys."

use axum::extract::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Barrier;
use web_repo_to_ai_ai_adapter::{
    AiAdapter, AiAdapterError, AiAdapterRegistry, AiClientConfig, AiOptInPolicy, AiProviderType,
    ChatCompletionRequest, ChatMessage, DisabledAiAdapter, DocumentSummarizationRequest,
    EmbeddingRequest, MockAiAdapter, OllamaAiAdapter, OpenAiAiAdapter, ResilientAiAdapter,
    SectionEnrichmentRequest, SummaryFocus, ZeroKeyFallbackEngine,
};

/// Helper spawning a delay-inducing server to test timeout isolation.
struct DelayServer {
    addr: SocketAddr,
}

impl DelayServer {
    async fn spawn(delay_ms: u64) -> Self {
        let app = Router::new()
            .route(
                "/api/tags",
                get(move || async move {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    Json(json!({ "models": [] }))
                }),
            )
            .route(
                "/models",
                get(move || async move {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    Json(json!({ "data": [] }))
                }),
            );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self { addr }
    }
}

/// Helper spawning a configurable HTTP status error server for error taxonomy verification.
struct ErrorServer {
    addr: SocketAddr,
    auth_header_seen: Arc<AtomicBool>,
}

impl ErrorServer {
    async fn spawn(status: StatusCode, body: &'static str) -> Self {
        let auth_header_seen = Arc::new(AtomicBool::new(false));
        let auth_seen_c = Arc::clone(&auth_header_seen);

        let app = Router::new()
            .route(
                "/chat/completions",
                post(move |headers: HeaderMap| {
                    let auth_seen = Arc::clone(&auth_seen_c);
                    async move {
                        if headers.get("authorization").is_some() {
                            auth_seen.store(true, Ordering::SeqCst);
                        }
                        (status, body).into_response()
                    }
                }),
            )
            .route(
                "/models",
                get(move || async move { (status, body).into_response() }),
            );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            addr,
            auth_header_seen,
        }
    }
}

#[tokio::test]
async fn test_polymorphic_contract_all_adapters() {
    // 1. Disabled adapter
    let disabled: Arc<dyn AiAdapter> = Arc::new(DisabledAiAdapter::new());
    assert_eq!(disabled.provider_type(), AiProviderType::Disabled);
    assert!(!disabled.is_enabled());
    let disabled_health = disabled.health_check().await.unwrap();
    assert!(!disabled_health.is_healthy);
    assert_eq!(disabled_health.provider, AiProviderType::Disabled);
    assert!(disabled.list_models().await.unwrap().is_empty());

    // 2. Mock adapter
    let mock: Arc<dyn AiAdapter> = Arc::new(MockAiAdapter::new());
    assert_eq!(mock.provider_type(), AiProviderType::Mock);
    assert!(mock.is_enabled());
    let mock_health = mock.health_check().await.unwrap();
    assert!(mock_health.is_healthy);
    assert_eq!(mock_health.provider, AiProviderType::Mock);
    assert!(!mock.list_models().await.unwrap().is_empty());

    // 3. Ollama adapter (configured against offline local endpoint)
    let ollama: Arc<dyn AiAdapter> =
        Arc::new(OllamaAiAdapter::new("http://127.0.0.1:11434").unwrap());
    assert_eq!(ollama.provider_type(), AiProviderType::Ollama);
    assert!(ollama.is_enabled());

    // 4. OpenAI adapter
    let openai: Arc<dyn AiAdapter> =
        Arc::new(OpenAiAiAdapter::for_openai("sk-test-key-12345678").unwrap());
    assert_eq!(openai.provider_type(), AiProviderType::OpenAiCompatible);
    assert!(openai.is_enabled());

    // 5. Resilient adapter (wrapping mock)
    let resilient: Arc<dyn AiAdapter> = Arc::new(ResilientAiAdapter::new(
        mock.clone(),
        AiOptInPolicy::local_only(),
    ));
    assert_eq!(resilient.provider_type(), AiProviderType::Mock);
    assert!(resilient.is_enabled());
    let resilient_health = resilient.health_check().await.unwrap();
    assert!(resilient_health.is_healthy);
}

#[tokio::test]
async fn test_isolation_network_timeout_enforcement() {
    // Server delays 1500ms
    let delay_server = DelayServer::spawn(1500).await;
    let url = format!("http://{}", delay_server.addr);

    // Client with 1s timeout
    let config = AiClientConfig::new(&url).with_timeout_secs(1);
    let openai_adapter = OpenAiAiAdapter::with_config(config).unwrap();

    // Call models catalog: must time out cleanly
    let err = openai_adapter.list_models().await.unwrap_err();
    assert!(matches!(
        err,
        AiAdapterError::Timeout(_) | AiAdapterError::NetworkError(_)
    ));
    assert!(err.is_retryable());
    assert!(err.status_code().is_some());
}

#[tokio::test]
async fn test_isolation_connection_refused_immediate_failure() {
    // Pick an ephemeral port and close it immediately
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let closed_addr = listener.local_addr().unwrap();
    drop(listener);

    let url = format!("http://{closed_addr}");
    let ollama = OllamaAiAdapter::new(&url).unwrap();

    let err = ollama.health_check().await.unwrap_err();
    assert!(matches!(err, AiAdapterError::NetworkError(_)));
    assert!(err.is_retryable());
    assert_eq!(err.status_code(), Some(500));
}

#[tokio::test]
async fn test_isolation_simulated_network_blackout_and_resilient_recovery() {
    // Unreachable endpoint
    let closed_url = "http://127.0.0.1:1";
    let primary = Arc::new(OpenAiAiAdapter::new(closed_url, Some("sk-test".to_string())).unwrap());

    let policy = AiOptInPolicy::local_only();
    let resilient = ResilientAiAdapter::new(primary, policy).with_fallback_on_error(true);

    // Health check returns degraded with zero-key fallback active
    let health = resilient.health_check().await.unwrap();
    assert!(!health.is_healthy);
    assert!(health
        .status_message
        .to_ascii_lowercase()
        .contains("zero-key fallback active"));

    // Chat completion seamlessly degrades to zero-key CPU fallback
    let chat_req = ChatCompletionRequest::new("gpt-4o", vec![ChatMessage::user("offline query")]);
    let chat_resp = resilient.complete_chat(chat_req).await.unwrap();
    assert_eq!(chat_resp.model, "deterministic-cpu-fallback");
    assert!(!chat_resp.choices.is_empty());

    // Embeddings seamlessly degrade to Blake3 zero-key vectors
    let emb_req = EmbeddingRequest::new("text-embed", vec!["offline text".to_string()]);
    let emb_resp = resilient.generate_embeddings(emb_req).await.unwrap();
    assert_eq!(emb_resp.embeddings.len(), 1);
    assert_eq!(emb_resp.dimensions, 384);

    // Document summarization seamlessly degrades to extractive CPU fallback
    let sum_req = DocumentSummarizationRequest::new("doc-off", "Title", "# Heading\nContent line.");
    let sum_resp = resilient.summarize_document(sum_req).await.unwrap();
    assert_eq!(sum_resp.model, "extractive-cpu-fallback");
    assert!(!sum_resp.summary.is_empty());
}

#[tokio::test]
async fn test_polymorphic_error_taxonomy_mapping() {
    // 1. 401 Unauthorized
    let server_401 = ErrorServer::spawn(StatusCode::UNAUTHORIZED, "Invalid bearer token").await;
    let adapter_401 = OpenAiAiAdapter::for_local(format!("http://{}", server_401.addr)).unwrap();
    let err_401 = adapter_401.list_models().await.unwrap_err();
    assert!(matches!(err_401, AiAdapterError::AuthenticationRequired(_)));
    assert!(!err_401.is_retryable());
    assert_eq!(err_401.status_code(), Some(401));

    // 2. 404 Not Found
    let server_404 = ErrorServer::spawn(StatusCode::NOT_FOUND, "Model does not exist").await;
    let adapter_404 = OpenAiAiAdapter::for_local(format!("http://{}", server_404.addr)).unwrap();
    let err_404 = adapter_404.list_models().await.unwrap_err();
    assert!(matches!(err_404, AiAdapterError::ModelNotFound { .. }));
    assert!(!err_404.is_retryable());
    assert_eq!(err_404.status_code(), Some(404));

    // 3. 503 Server Unavailable
    let server_503 = ErrorServer::spawn(StatusCode::SERVICE_UNAVAILABLE, "Capacity exceeded").await;
    let adapter_503 = OpenAiAiAdapter::for_local(format!("http://{}", server_503.addr)).unwrap();
    let err_503 = adapter_503.list_models().await.unwrap_err();
    assert!(matches!(err_503, AiAdapterError::ServerUnavailable { .. }));
    assert!(err_503.is_retryable());
    assert_eq!(err_503.status_code(), Some(503));
}

#[tokio::test]
async fn test_concurrent_multithreaded_stress() {
    let registry = Arc::new(AiAdapterRegistry::new());
    let mock = Arc::new(MockAiAdapter::new());
    registry.set_policy(AiOptInPolicy::local_only());
    registry.set_resilient_adapter(mock.clone(), true);

    let num_tasks = 25;
    let barrier = Arc::new(Barrier::new(num_tasks));
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let reg = Arc::clone(&registry);
        let bar = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            bar.wait().await;

            // Task 1: Chat completion
            let chat_req =
                ChatCompletionRequest::new("mock", vec![ChatMessage::user(format!("ping {i}"))]);
            let chat_resp = reg.complete_chat(chat_req).await.unwrap();
            assert!(chat_resp.primary_text().is_some());

            // Task 2: Vector embeddings
            let emb_req = EmbeddingRequest::new("mock", vec![format!("text {i}")]);
            let emb_resp = reg.generate_embeddings(emb_req).await.unwrap();
            assert_eq!(emb_resp.embeddings.len(), 1);

            // Task 3: Document summarization
            let sum_req = DocumentSummarizationRequest::new(
                format!("doc-{i}"),
                format!("Title {i}"),
                format!("# Heading\nBody text for document {i}."),
            );
            let sum_resp = reg.summarize_document(sum_req).await.unwrap();
            assert!(!sum_resp.summary.is_empty());

            // Task 4: Section enrichment
            let enrich_req =
                SectionEnrichmentRequest::new("Code", format!("pub fn worker_{i}() {{}}"))
                    .with_title(format!("Worker {i}"));
            let enrich_resp = reg.enrich_section(enrich_req).await.unwrap();
            assert!(!enrich_resp.suggested_title.is_empty());
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Each task performed: complete_chat (1) + summarize_document (1 chat) + enrich_section (1 chat) = 3 chat calls
    assert_eq!(mock.tracker().chat_call_count(), num_tasks * 3);
    assert_eq!(mock.tracker().embedding_call_count(), num_tasks);
}

#[tokio::test]
async fn test_hot_swapping_registry_under_concurrent_load() {
    let registry = Arc::new(AiAdapterRegistry::new());
    let num_workers = 15;
    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    // Spawn concurrent readers
    for _ in 0..num_workers {
        let reg = Arc::clone(&registry);
        let stop = Arc::clone(&stop_signal);

        handles.push(tokio::spawn(async move {
            while !stop.load(Ordering::Relaxed) {
                let req = ChatCompletionRequest::new("model", vec![ChatMessage::user("test")]);
                let _ = reg.complete_chat(req).await;
                tokio::task::yield_now().await;
            }
        }));
    }

    // Dynamic coordinator rapidly hot-swaps adapters
    for iteration in 0..10 {
        if iteration % 2 == 0 {
            registry.set_adapter(Arc::new(MockAiAdapter::new()));
        } else {
            registry.disable();
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    stop_signal.store(true, Ordering::Relaxed);
    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::test]
async fn test_isolation_header_injection_and_sanitization() {
    let server = ErrorServer::spawn(StatusCode::OK, r#"{"choices":[]}"#).await;
    let url = format!("http://{}", server.addr);

    let config = AiClientConfig::new(&url).with_api_key("sk-super-secret-token-12345");
    let adapter = OpenAiAiAdapter::with_config(config).unwrap();

    // Execute chat completion to verify header presence on the wire
    let req = ChatCompletionRequest::new("model", vec![ChatMessage::user("hello")]);
    let _ = adapter.complete_chat(req).await;

    // Verify wire server observed Authorization header
    assert!(server.auth_header_seen.load(Ordering::SeqCst));
}

#[test]
fn test_zero_key_fallback_contract_invariants() {
    let engine = ZeroKeyFallbackEngine::new();

    // Invariant 1: Vector dimensions and unit norm
    for dim in [64, 128, 256, 384, 512, 768, 1536] {
        let req = EmbeddingRequest::new("test", vec!["invariant input text".to_string()])
            .with_dimensions(dim);
        let resp = engine.fallback_embeddings(&req);

        assert_eq!(resp.dimensions, dim);
        assert_eq!(resp.embeddings[0].len(), dim);

        let norm: f32 = resp.embeddings[0].iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-5,
            "Vector norm {norm} diverged from unit length for dimension {dim}"
        );
    }

    // Invariant 2: Extractive summary structure
    let doc =
        "# System Architecture\nThis is the core pipeline abstract.\n\n## Component A\nDetails.";
    let req = DocumentSummarizationRequest::new("doc-inv", "System Architecture", doc)
        .with_focus(SummaryFocus::Architecture);
    let resp = engine.fallback_summarize(&req);

    assert_eq!(resp.model, "extractive-cpu-fallback");
    assert!(resp.summary.contains("System Architecture"));
    assert!(!resp.key_topics.is_empty());
}
