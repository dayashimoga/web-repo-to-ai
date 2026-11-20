# AI Adapter Crate (`web-repo-to-ai-ai-adapter`)

Provides provider-independent AI abstractions, asynchronous adapter traits, zero-key fallback mechanisms, in-memory test mocks, and runtime registries for optional LLM assistance across the Universal Web & Repository to AI converter.

## Core Philosophy: Zero Mandatory AI Dependencies

In strict compliance with **Section 3.4 of the `GLOBAL_ENGINEERING_CONTRACT.md`** (*"Zero Mandatory External AI / Paid Dependencies"*):
- All core document acquisition, HTML/code extraction, Canonical Context Document Model (CDM) normalization, token counting, deduplication, chunking, and export serialization execute **locally and deterministically on CPU**.
- The AI subsystem is **100% optional and opt-in**. If no AI provider is configured or if network access is restricted, the platform operates cleanly in zero-key fallback mode without warnings, degraded extraction quality, or runtime panics.

## Features & Architecture

- **Provider-Independent Abstraction (`traits.rs`):**
  - `AiAdapter`: Core asynchronous trait decoupling the monorepo from specific vendor APIs or wire protocols.
  - Implements chat completions, vector embeddings, connectivity health checks, model catalog enumeration, and high-level document analysis helpers.
  - Provides default prompt-engineering implementations for `summarize_document` and `enrich_section`.

- **Domain Models & Schemas (`models.rs`):**
  - **`AiProviderType`**: `Disabled`, `Mock`, `Ollama`, `OpenAiCompatible`, `AnthropicCompatible`, `CustomLocal`.
  - **`AiModelDescriptor`**: Model identity, context window limits, output token ceilings, and capability flags.
  - **`AiCapabilities`**: Capability flags (`supports_chat`, `supports_embeddings`, `supports_streaming`, `supports_vision`, `supports_json_mode`, `supports_tools`).
  - **Chat Completions**: `ChatMessage`, `ChatRole` (`System`, `User`, `Assistant`, `Tool`), `ChatCompletionRequest` (temperature, max tokens, response formats, stop sequences, seed), `ChatChoice`, `ChatFinishReason`, and `ChatCompletionResponse`.
  - **Vector Embeddings**: `EmbeddingRequest`, `EmbeddingResponse`, dimension specification, and multi-input batching.
  - **Pipeline Domain Helpers**: `DocumentSummarizationRequest` / `Response` (with analytical focuses: `General`, `Architecture`, `Security`, `ApiOverview`, `KeyInsights`) and `SectionEnrichmentRequest` / `Response`.
  - **Token Accounting**: `TokenUsage` tracking prompt tokens, completion tokens, and total usage.

- **Categorized Error Taxonomy (`error.rs`):**
  - `AiAdapterError`: Explicit taxonomy covering `ProviderDisabled`, `AuthenticationRequired`, `NetworkError`, `RateLimitExceeded`, `ContextLengthExceeded`, `ModelNotFound`, `InvalidRequest`, `ResponseParsingError`, `ServerUnavailable`, `Timeout`, and `Internal`.
  - Classification helpers: `is_retryable()` for exponential backoff handling, and `status_code()` for HTTP problem details mapping.

- **Zero-Key Fallback Adapter (`disabled.rs`):**
  - `DisabledAiAdapter`: Null object implementation active by default.
  - Returns `is_enabled() == false`, reports healthy zero-key status in `health_check`, and gracefully rejects AI invocations with `AiAdapterError::ProviderDisabled`.

- **In-Memory Mock Adapter (`mock.rs`):**
  - `MockAiAdapter`: Full-featured in-memory mock designed for offline development and deterministic unit/integration testing.
  - Supports programmable scripted responses, default canned fallbacks, automated heuristic answer synthesis, and deterministic Blake3-derived pseudo-embeddings ($L_2$ unit-normalized).
  - Configurable error injection (`with_injected_error`) and simulated network latency (`with_latency_ms`).
  - `MockCallTracker`: Thread-safe call history recording and cumulative token accounting.

- **Dynamic Runtime Registry (`registry.rs`):**
  - `AiAdapterRegistry`: Thread-safe container managing active adapter instances.
  - Safely defaults to `DisabledAiAdapter` on construction.
  - Allows dynamic runtime adapter replacement (`set_adapter`), instant disabling (`disable`), and transparent method delegation.

- **Local Ollama Daemon Adapter (`ollama.rs`):**
  - `OllamaAiAdapter`: Production HTTP adapter connecting to local Ollama daemons (`http://localhost:11434`).
  - Supports `POST /api/chat` for chat completions, `POST /api/embed` batch embeddings with fallback to legacy `POST /api/embeddings`, and model catalog discovery via `GET /api/tags`.
  - Configurable timeouts, custom chat/embedding models, and zero cloud egress.

- **OpenAI-Compatible REST Adapter (`openai.rs`):**
  - `OpenAiAiAdapter`: Production HTTP adapter targeting standard OpenAI endpoints or OpenAI-compatible local runtimes (vLLM, LM Studio, LocalAI, Ollama `/v1`, Azure OpenAI).
  - Supports `POST /chat/completions` with JSON mode and stop sequences, `POST /embeddings` with dimension specification, and `GET /models`.
  - Configurable Bearer authentication, organization headers, and project scopes.

## Verification & Quality

- **100% Passing Test Suite:** 41 tests across 6 test suites:
  - `tests/ollama_adapter_test.rs`: Local Ollama wire protocol, `/api/tags` model catalog, `/api/chat` completions with JSON format, modern `/api/embed` batch embeddings with legacy `/api/embeddings` fallback, 500 error mapping, and disabled state.
  - `tests/openai_adapter_test.rs`: OpenAI-compatible wire protocol, Bearer auth header injection, `/models` listing, `/chat/completions` token accounting, `/embeddings` vector sorting, 401 unauthorized mapping, 429 rate limit with `retry-after`, 400 context length exceeded mapping, and local servers without authentication.
  - `tests/trait_contract_test.rs`: Domain model constructors, request builders, token arithmetic, error classifications, and full Serde JSON roundtrip.
  - `tests/disabled_adapter_test.rs`: Zero-key fallback guarantees, graceful rejection of chat, embeddings, summarization, and enrichment.
  - `tests/mock_adapter_test.rs`: Automated mock heuristics, scripted chat, default overrides, document summarization with topic extraction, section enrichment, deterministic Blake3 pseudo-embeddings ($L_2$ unit norm), error injection, call tracking, and 10-task concurrent execution.
  - `tests/registry_test.rs`: Registry default fallback, dynamic runtime switching, and thread-safety under concurrent mutation.
- **Strict Compliance:** Zero mocks in production paths, zero placeholders, Clippy `-D warnings` clean, `cargo deny check` clean.
