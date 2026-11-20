//! Integration tests for strict opt-in guardrails, secret redaction, and zero-key CPU fallback guarantees.
//!
//! Enforces Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT:
//! "Zero Mandatory External AI / Paid Dependencies: All core document extraction,
//! normalization, deduplication, chunking, and export logic execute locally on CPU
//! without AI assistance. AI provider adapters serve solely as optional, opt-in
//! post-processing extensions. Core extraction MUST ALWAYS succeed without AI keys."

use std::sync::Arc;
use web_repo_to_ai_ai_adapter::{
    mask_secret, redact_secrets_from_text, AiAdapter, AiAdapterError, AiAdapterRegistry,
    AiClientConfig, AiExecutionMode, AiOptInPolicy, AiProviderType, ChatCompletionRequest,
    ChatMessage, DocumentSummarizationRequest, EmbeddingRequest, MockAiAdapter, ResilientAiAdapter,
    SectionEnrichmentRequest, SummaryFocus, ZeroKeyFallbackEngine,
};

fn with_test_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
    let prev: Vec<(&str, Option<String>)> = vars
        .iter()
        .map(|(k, _)| (*k, std::env::var(k).ok()))
        .collect();
    for (k, v) in vars {
        match v {
            Some(val) => std::env::set_var(k, val),
            None => std::env::remove_var(k),
        }
    }
    f();
    for (k, prev_val) in prev {
        match prev_val {
            Some(val) => std::env::set_var(k, val),
            None => std::env::remove_var(k),
        }
    }
}

#[test]
fn test_guardrail_policy_disabled_by_default() {
    let policy = AiOptInPolicy::default();
    assert_eq!(policy.execution_mode, AiExecutionMode::Disabled);
    assert!(!policy.is_enabled());
    assert!(policy.allow_zero_key_fallback);

    // Any endpoint validation must fail with ProviderDisabled
    let err = policy
        .validate_endpoint("http://localhost:11434", false)
        .unwrap_err();
    assert!(matches!(err, AiAdapterError::ProviderDisabled(_)));

    let err2 = policy
        .validate_endpoint("https://api.openai.com/v1", true)
        .unwrap_err();
    assert!(matches!(err2, AiAdapterError::ProviderDisabled(_)));
}

#[test]
fn test_guardrail_policy_local_only_mode() {
    let policy = AiOptInPolicy::local_only();
    assert_eq!(policy.execution_mode, AiExecutionMode::LocalOnly);
    assert!(policy.is_enabled());

    // Local loopback and container endpoints succeed
    assert!(policy
        .validate_endpoint("http://localhost:11434", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://127.0.0.1:8000", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://[::1]:11434", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://host.docker.internal:11434", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://host.containers.internal:11434", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://ollama:11434", false)
        .is_ok());
    assert!(policy
        .validate_endpoint("http://localai.local:8080", false)
        .is_ok());

    // Remote cloud endpoints are strictly blocked
    let err = policy
        .validate_endpoint("https://api.openai.com/v1", true)
        .unwrap_err();
    assert!(matches!(err, AiAdapterError::PolicyViolation(_)));
    assert!(err
        .to_string()
        .contains("blocked by LocalOnly execution policy"));

    let err2 = policy
        .validate_endpoint("https://api.anthropic.com/v1", true)
        .unwrap_err();
    assert!(matches!(err2, AiAdapterError::PolicyViolation(_)));
}

#[test]
fn test_guardrail_policy_cloud_metadata_defense() {
    // Even when remote cloud calls are permitted, cloud metadata endpoints are strictly blocked
    let policy = AiOptInPolicy::allow_remote();

    let err_aws = policy
        .validate_endpoint("http://169.254.169.254/latest/meta-data", false)
        .unwrap_err();
    assert!(matches!(err_aws, AiAdapterError::PolicyViolation(_)));
    assert!(err_aws.to_string().contains("cloud metadata"));

    let err_gcp = policy
        .validate_endpoint("http://metadata.google.internal/computeMetadata/v1", false)
        .unwrap_err();
    assert!(matches!(err_gcp, AiAdapterError::PolicyViolation(_)));
    assert!(err_gcp.to_string().contains("cloud metadata"));
}

#[test]
fn test_guardrail_policy_from_env_discovery() {
    // 1. When AI_ENABLED is unset or false
    with_test_env(&[("AI_ENABLED", None), ("AI_EXECUTION_MODE", None)], || {
        let p = AiOptInPolicy::from_env();
        assert!(!p.is_enabled());
        assert_eq!(p.execution_mode, AiExecutionMode::Disabled);
    });

    // 2. When AI_ENABLED=true and AI_EXECUTION_MODE=local_only
    with_test_env(
        &[
            ("AI_ENABLED", Some("true")),
            ("AI_EXECUTION_MODE", Some("local_only")),
        ],
        || {
            let p = AiOptInPolicy::from_env();
            assert!(p.is_enabled());
            assert_eq!(p.execution_mode, AiExecutionMode::LocalOnly);
        },
    );

    // 3. When AI_ENABLED=1 and AI_ALLOW_REMOTE=true
    with_test_env(
        &[
            ("AI_ENABLED", Some("1")),
            ("AI_EXECUTION_MODE", None),
            ("AI_ALLOW_REMOTE", Some("true")),
        ],
        || {
            let p = AiOptInPolicy::from_env();
            assert!(p.is_enabled());
            assert_eq!(p.execution_mode, AiExecutionMode::AllowRemote);
        },
    );
}

#[test]
fn test_secret_redaction_and_debug_formatting() {
    let secret_key = "sk-proj-9876543210fedcba";
    let config = AiClientConfig::new("https://api.openai.com/v1").with_api_key(secret_key);

    // Verify debug formatting masks secret
    let debug_str = format!("{config:?}");
    assert!(
        !debug_str.contains(secret_key),
        "Cleartext API key leaked in Debug output!"
    );
    assert!(debug_str.contains("sk-...dcba"));

    // Verify masked_api_key helper
    assert_eq!(config.masked_api_key(), Some("sk-...dcba".to_string()));

    // Verify short secret masking
    assert_eq!(mask_secret("12345"), "[REDACTED]");
    assert_eq!(mask_secret(""), "");
}

#[test]
fn test_secret_scrubbing_from_error_messages() {
    let raw_key = "sk-live-secret-ai-token-123456";
    let error_text =
        format!("HTTP 401 Unauthorized for Authorization: Bearer {raw_key} at api.openai.com");

    let sanitized = redact_secrets_from_text(&error_text, &[raw_key]);
    assert!(
        !sanitized.contains(raw_key),
        "Raw secret leaked in sanitized error text!"
    );
    assert!(sanitized.contains("sk-...3456"));
}

#[test]
fn test_zero_key_extractive_summarization_fidelity() {
    let engine = ZeroKeyFallbackEngine::new();
    let sample_doc = r#"# Distributed Job Queue Architecture
This document details the high-throughput job queue and worker scheduling architecture.
Tasks are dispatched through atomic state transitions in PostgreSQL.

## Core Dispatcher
The core dispatcher polls pending tasks with adaptive backoff intervals.
Job concurrency is bounded by CPU core counts to prevent memory exhaustion.

## Failure Recovery
Interrupted tasks are automatically recovered upon worker heartbeat timeout.
Idempotent task tokens ensure exactly-once side effect execution.
"#;

    let req = DocumentSummarizationRequest::new(
        "doc-queue",
        "Distributed Job Queue Architecture",
        sample_doc,
    )
    .with_focus(SummaryFocus::Architecture);

    let resp = engine.fallback_summarize(&req);
    assert_eq!(resp.model, "extractive-cpu-fallback");
    assert!(resp.summary.contains("Distributed Job Queue Architecture"));
    assert!(resp.summary.contains("high-throughput job queue"));
    assert!(resp.key_topics.contains(&"architecture".to_string()));
    assert!(resp.token_usage.total_tokens > 0);
}

#[test]
fn test_zero_key_deterministic_embeddings_norm_and_repeatability() {
    let engine = ZeroKeyFallbackEngine::new();
    let text = "Universal URL and repository extractor for AI models";

    let req = EmbeddingRequest::new("cpu-embeddings", vec![text.to_string()]).with_dimensions(384);
    let resp1 = engine.fallback_embeddings(&req);
    let resp2 = engine.fallback_embeddings(&req);

    assert_eq!(resp1.embeddings.len(), 1);
    let vec1 = &resp1.embeddings[0];
    let vec2 = &resp2.embeddings[0];

    assert_eq!(vec1.len(), 384);
    // Bit-for-bit repeatability
    assert_eq!(vec1, vec2);

    // Euclidean L2 norm == 1.0 +- 1e-5
    let norm: f32 = vec1.iter().map(|v| v * v).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5);
}

#[test]
fn test_zero_key_section_enrichment() {
    let engine = ZeroKeyFallbackEngine::new();
    let code = r#"
pub struct WorkerPool {
    workers: Vec<WorkerThread>,
}

impl WorkerPool {
    pub fn new(capacity: usize) -> Self {
        Self { workers: Vec::with_capacity(capacity) }
    }
}
"#;
    let req = SectionEnrichmentRequest::new("CodeDeclaration", code)
        .with_title("WorkerPool Declaration")
        .with_source("src/pool.rs", "rust");

    let resp = engine.fallback_enrich_section(&req);
    assert_eq!(resp.model, "heuristic-cpu-fallback");
    assert!(resp.suggested_title.contains("WorkerPool"));
    assert!(resp.tags.contains(&"rust".to_string()));
    assert!(resp.tags.contains(&"data-structure".to_string()));
    assert!(resp.tags.contains(&"function".to_string()));
}

#[tokio::test]
async fn test_resilient_adapter_delegates_to_zero_key_when_disabled() {
    let mock = Arc::new(MockAiAdapter::new());
    let policy = AiOptInPolicy::disabled();
    let resilient = ResilientAiAdapter::new(mock.clone(), policy);

    assert!(!resilient.is_enabled());
    assert_eq!(resilient.provider_type(), AiProviderType::Disabled);

    // Health check returns healthy zero-key fallback status
    let health = resilient.health_check().await.unwrap();
    assert!(health.is_healthy);
    assert_eq!(health.provider, AiProviderType::Disabled);
    assert!(health.status_message.contains("zero-key fallback"));

    // Chat completion uses zero-key fallback with zero calls to underlying mock
    let chat_req = ChatCompletionRequest::new("gpt-4o", vec![ChatMessage::user("hello")]);
    let chat_resp = resilient.complete_chat(chat_req).await.unwrap();
    assert_eq!(chat_resp.model, "deterministic-cpu-fallback");
    assert_eq!(mock.tracker().chat_call_count(), 0);

    // Embeddings use zero-key fallback
    let emb_req = EmbeddingRequest::new("text-embedding-3", vec!["test".to_string()]);
    let emb_resp = resilient.generate_embeddings(emb_req).await.unwrap();
    assert_eq!(emb_resp.embeddings.len(), 1);
    assert_eq!(mock.tracker().embedding_call_count(), 0);

    // Document summarization uses zero-key fallback
    let sum_req = DocumentSummarizationRequest::new("doc-1", "Title", "# Content\nSome text.");
    let sum_resp = resilient.summarize_document(sum_req).await.unwrap();
    assert_eq!(sum_resp.model, "extractive-cpu-fallback");
    assert_eq!(mock.tracker().chat_call_count(), 0);
}

#[tokio::test]
async fn test_resilient_adapter_seamless_fallback_on_primary_failure() {
    let mock = Arc::new(MockAiAdapter::new().with_injected_error(
        AiAdapterError::ServerUnavailable {
            status_code: Some(503),
            message: "Upstream LLM cluster overloaded".to_string(),
        },
    ));

    let policy = AiOptInPolicy::local_only();
    let resilient = ResilientAiAdapter::new(mock.clone(), policy).with_fallback_on_error(true);

    // Primary fails, but resilient adapter seamlessly degrades to zero-key fallback
    let chat_req = ChatCompletionRequest::new("gpt-4o", vec![ChatMessage::user("query")]);
    let chat_resp = resilient.complete_chat(chat_req).await.unwrap();
    assert_eq!(chat_resp.model, "deterministic-cpu-fallback");
    assert!(!chat_resp.choices.is_empty());

    // Embeddings also seamlessly degrade
    let emb_req = EmbeddingRequest::new("embed-model", vec!["dense text".to_string()]);
    let emb_resp = resilient.generate_embeddings(emb_req).await.unwrap();
    assert_eq!(emb_resp.embeddings.len(), 1);
    assert_eq!(emb_resp.dimensions, 384);

    // Document summarization seamlessly degrades
    let sum_req = DocumentSummarizationRequest::new("doc-fail", "Title", "# Fail test\nContent");
    let sum_resp = resilient.summarize_document(sum_req).await.unwrap();
    assert_eq!(sum_resp.model, "extractive-cpu-fallback");
    assert!(!sum_resp.summary.is_empty());
}

#[tokio::test]
async fn test_resilient_adapter_error_propagation_when_fallback_disabled() {
    let mock = Arc::new(MockAiAdapter::new().with_injected_error(
        AiAdapterError::AuthenticationRequired("Invalid key".to_string()),
    ));

    let policy = AiOptInPolicy::local_only();
    let resilient = ResilientAiAdapter::new(mock.clone(), policy).with_fallback_on_error(false);

    let chat_req = ChatCompletionRequest::new("gpt-4o", vec![ChatMessage::user("query")]);
    let err = resilient.complete_chat(chat_req).await.unwrap_err();
    assert!(matches!(err, AiAdapterError::AuthenticationRequired(_)));
}

#[tokio::test]
async fn test_registry_with_resilient_guardrails_integration() {
    let registry = AiAdapterRegistry::new();
    assert!(!registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Disabled);

    // Configure resilient mock adapter with local_only policy
    let mock = Arc::new(MockAiAdapter::new());
    registry.set_policy(AiOptInPolicy::local_only());
    registry.set_resilient_adapter(mock.clone(), true);

    assert!(registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Mock);

    // Chat completion executes through mock
    let req = ChatCompletionRequest::new("mock", vec![ChatMessage::user("test")]);
    let resp = registry.complete_chat(req).await.unwrap();
    assert!(resp.primary_text().is_some());
    assert_eq!(mock.tracker().chat_call_count(), 1);

    // Disable instantly resets to zero-key
    registry.disable();
    assert!(!registry.is_enabled());
    assert_eq!(registry.provider_type(), AiProviderType::Disabled);
}
