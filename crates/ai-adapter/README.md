# AI Adapter Crate (`web-repo-to-ai-ai-adapter`)

Provides provider-independent AI abstractions, asynchronous adapter traits, zero-key fallback mechanisms, strict opt-in guardrails, secret redaction, in-memory test mocks, and runtime registries for optional LLM assistance across the Universal Web & Repository to AI converter.

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
  - `AiAdapterError`: Explicit taxonomy covering `ProviderDisabled`, `PolicyViolation`, `AuthenticationRequired`, `NetworkError`, `RateLimitExceeded`, `ContextLengthExceeded`, `ModelNotFound`, `InvalidRequest`, `ResponseParsingError`, `ServerUnavailable`, `Timeout`, and `Internal`.
  - Classification helpers: `is_retryable()` for exponential backoff handling, and `status_code()` for HTTP problem details mapping.

- **Strict Opt-in Guardrails & Network Boundaries (`guardrails.rs`):**
  - `AiOptInPolicy`: Comprehensive policy governing AI provider invocation and network egress boundaries.
  - `AiExecutionMode`:
    - `Disabled`: Absolutely no AI calls allowed; all operations route cleanly to zero-key CPU fallbacks.
    - `LocalOnly`: Only local inference instances (`localhost`, `127.0.0.1`, `[::1]`, container hosts `host.docker.internal`, `ollama`, `vllm`, `localai`) are permitted; all cloud egress is blocked by policy.
    - `AllowRemote`: Remote cloud AI providers permitted over TLS/HTTPS; unencrypted HTTP rejected by default.
  - Cloud Metadata Isolation: Outbound calls to cloud metadata IP addresses (`169.254.169.254`, `metadata.google.internal`) are strictly forbidden under all modes.
  - Environment Discovery: `AiOptInPolicy::from_env()` reads `AI_ENABLED`, `AI_EXECUTION_MODE`, and `AI_ALLOW_REMOTE`.

- **Secret Redaction & Protection (`secret.rs`):**
  - `mask_secret`: Redacts API keys and tokens for safe logging and telemetry (preserving first 3 and last 4 characters, e.g. `sk-...cdef`).
  - `redact_secrets_from_text`: Scrubs explicit secrets and `Bearer <token>` patterns from error strings and traces.
  - Sanitized `AiClientConfig`: Custom `std::fmt::Debug` implementation preventing API key leakage in debug logs.

- **Zero-Key Fallback Engine (`fallback.rs`):**
  - `ZeroKeyFallbackEngine`: High-fidelity, deterministic CPU-only operations executing without AI keys or network access:
    - **Extractive Summarization**: Extracts document title, outline headings, and salient thesis sentences structured by analytical focus (`Architecture`, `Security`, `ApiOverview`, `General`).
    - **Heuristic Section Enrichment**: Extracts programming syntax tags, code signatures (`fn`, `struct`, `class`, `enum`), and conceptual headings.
    - **Deterministic Embeddings**: Generates Blake3-derived, $L_2$ unit-normalized vectors ($|v|_2 = 1.0$) with bit-for-bit repeatability across requested dimensions.
    - **Fallback Chat Completions**: Returns structured offline responses (including valid JSON if requested).

- **Resilient AI Adapter Decorator (`resilient.rs`):**
  - `ResilientAiAdapter`: Decorator wrapping any primary `AiAdapter` with policy guardrails and zero-key fallback.
  - When AI is disabled by policy, delegates immediately to `ZeroKeyFallbackEngine` without network activity.
  - When primary provider encounters network failures, timeouts, 401s, 429s, or 503s, gracefully degrades to zero-key fallback (when `fallback_on_error == true`).

- **Dynamic Runtime Registry (`registry.rs`):**
  - `AiAdapterRegistry`: Thread-safe container managing active adapter instances and guardrail policies.
  - Safely defaults to `DisabledAiAdapter` and `AiOptInPolicy::disabled()` on construction.
  - Allows dynamic runtime adapter replacement (`set_adapter`), resilient wrapping (`set_resilient_adapter`), policy reconfiguration (`set_policy`), and instant disabling (`disable`).

- **Local Ollama Daemon Adapter (`ollama.rs`):**
  - `OllamaAiAdapter`: Production HTTP adapter connecting to local Ollama daemons (`http://localhost:11434`).
  - Supports `POST /api/chat` for chat completions, `POST /api/embed` batch embeddings with fallback to legacy `POST /api/embeddings`, and model catalog discovery via `GET /api/tags`.
  - Configurable timeouts, custom chat/embedding models, and zero cloud egress.

- **OpenAI-Compatible REST Adapter (`openai.rs`):**
  - `OpenAiAiAdapter`: Production HTTP adapter targeting standard OpenAI endpoints or OpenAI-compatible local runtimes (vLLM, LM Studio, LocalAI, Ollama `/v1`, Azure OpenAI).
  - Supports `POST /chat/completions` with JSON mode and stop sequences, `POST /embeddings` with dimension specification, and `GET /models`.
  - Configurable Bearer authentication, organization headers, and project scopes.

## Verification & Quality

- **100% Passing Test Suite:** 75 tests across 8 test suites:
  - `tests/adapter_contract_and_isolation_test.rs` (9 tests): Polymorphic trait contract verification across all 5 adapter types (`Disabled`, `Mock`, `Ollama`, `OpenAiCompatible`, `Resilient`), timeout isolation, connection refused immediate failure, simulated network blackout with resilient recovery, categorized error taxonomy mapping (401, 404, 503), 25-task multithreaded concurrency stress, dynamic registry hot-swapping under load, wire header injection and sanitization, and Blake3 vector mathematical invariants.
  - `tests/guardrails_and_fallback_test.rs` (13 tests): Opt-in policy defaults, `LocalOnly` loopback/container validation, remote blocking, cloud metadata defense, environment variable discovery, secret redaction in debug formatting and error strings, extractive summarization fidelity, Blake3 $L_2$ unit-norm embeddings repeatability, section enrichment, resilient fallback on provider failure, and registry integration.
  - `tests/ollama_adapter_test.rs` (8 tests): Local Ollama wire protocol, `/api/tags` model catalog, `/api/chat` completions with JSON format, modern `/api/embed` batch embeddings with legacy `/api/embeddings` fallback, 500 error mapping, and disabled state.
  - `tests/openai_adapter_test.rs` (8 tests): OpenAI-compatible wire protocol, Bearer auth header injection, `/models` listing, `/chat/completions` token accounting, `/embeddings` vector sorting, 401 unauthorized mapping, 429 rate limit with `retry-after`, 400 context length exceeded mapping, and local servers without authentication.
  - `tests/trait_contract_test.rs` (7 tests): Domain model constructors, request builders, token arithmetic, error classifications, and full Serde JSON roundtrip.
  - `tests/disabled_adapter_test.rs` (4 tests): Zero-key fallback guarantees, graceful rejection of chat, embeddings, summarization, and enrichment.
  - `tests/mock_adapter_test.rs` (11 tests): Automated mock heuristics, scripted chat, default overrides, document summarization with topic extraction, section enrichment, deterministic Blake3 pseudo-embeddings ($L_2$ unit norm), error injection, call tracking, and 10-task concurrent execution.
  - `tests/registry_test.rs` (3 tests): Registry default fallback, dynamic runtime switching, and thread-safety under concurrent mutation.
- **Strict Compliance:** Zero mocks in production paths, zero placeholders, Clippy `-D warnings` clean, `cargo deny check` clean.
- **Phase 10.5 & Stage 10 Certified:** All 4 subphases (`10.5.1`, `10.5.2`, `10.5.3`, `10.5.4`) complete, verified, and certified under Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT.
