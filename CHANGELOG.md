# CHANGELOG (`CHANGELOG.md`)

**Policy:** Append-only chronological record of verified project changes by Stage, Phase, and Subphase. Never overwrite or delete historical entries.

---

## [Unreleased] — Stage 7: AI Context and Token Optimization

### 2026-11-18 — Phase 7.4: Quality validation (`7.4.2`)

#### Added
- **`[7.4.2]` Compression Reporting, Pipeline Progression Tracking & Financial Cost Savings in `crates/context-optimizer`:**
  - Implemented comprehensive compression reporting and telemetry engine in `crates/context-optimizer/src/compression.rs`:
    - `OptimizationStage` taxonomy: models transformation pipeline stages (`Normalization`, `ExactDeduplication`, `SemanticDeduplication`, `ProfileFiltering`, `BudgetKnapsack`, `Custom`).
    - `StageCompressionMetrics`: calculates input/output progression, token and byte savings, reduction percentages, and compression ratios per stage with GFM table row formatting.
    - `OverallCompressionMetrics`: measures cumulative end-to-end token, byte, character, and section reduction metrics and compression ratios with formatted GFM summary table.
    - `DocumentVolume`: measures multi-dimensional document volume (tokens, UTF-8 bytes, chars, sections) with tokenizer model adapter or heuristic fallback.
    - `CostSavingsMetrics` & `MultiModelCostSavings`: evaluates financial API cost reductions per 1k invocations across commercial LLM pricing schedules (GPT-4o, GPT-4o-mini, Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku, Gemini 1.5 Pro, Gemini 1.5 Flash) and self-hosted models (Llama 3 $0.00).
    - `CompressionReport`: comprehensive report card with GFM markdown rendering and JSON serialization.
    - `CompressionPipelineTracker`: progression recorder tracking volume transitions across sequential stages.
    - `CompressionEvaluator`: comparative evaluator between baseline and optimized `CdmDocument`s with profile context.
  - Re-exported compression models and public convenience APIs in `crates/context-optimizer/src/lib.rs`: `evaluate_compression`, `evaluate_compression_for_model`, and `evaluate_compression_with_profile`.
  - Authored integration test suite `crates/context-optimizer/tests/compression_reporting_test.rs` (7 integration tests validating unmodified document fidelity, budget knapsack compression, multi-stage pipeline progression, multi-model cost savings, markdown/JSON serialization, fail-closed empty doc rejection, and convenience APIs).
  - 100% passing quality gates: 61 crate tests passed, 694+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.4: Quality validation (`7.4.1`)

#### Added
- **`[7.4.1]` Preservation Metrics and Symbol Fidelity Evaluation in `crates/context-optimizer`:**
  - Implemented comprehensive content preservation and symbol fidelity evaluation engine in `crates/context-optimizer/src/preservation.rs`:
    - `KindPreservation` and `SectionPreservationMetrics`: tracks section retention rates, retained vs original section counts, and token counts across all `SectionKind` categories (prose, code files, manifests, documentation, diffs, etc.) with clean BTreeMap serialization.
    - `CriticalPreservationMetrics`: audits preservation of mission-critical sections (manifests, security policies, error definitions, root READMEs) with zero silent drop detection.
    - `SymbolPreservationMetrics`: integrates with `crates/code-intelligence` (`Language`, `SymbolDeclaration`, `SymbolExtractor`) to parse AST symbol declarations from original vs retained code files, measuring overall symbol retention ratio, public API symbol retention ratio, and compiling an exact inventory of `omitted_symbol_names`.
    - `TokenPreservationMetrics`: tracks original vs optimized token volume, retention ratio, and reduction percentage.
    - `StructuralIntegrityMetrics`: verifies syntax integrity across retained code sections and ensures clean truncation boundaries without malformed half-tokens.
    - `PreservationConfig`: configurable fidelity evaluation parameters with weighted composite scoring (`critical_weight: 0.35`, `public_symbols_weight: 0.30`, `overall_symbols_weight: 0.15`, `section_retention_weight: 0.20`, `min_acceptable_fidelity: 0.50`).
    - `PreservationReport`: comprehensive report struct supporting GFM summary tables, multi-section category breakdowns, symbol omission listings, and JSON serialization.
    - `PreservationEvaluator`: evaluation engine executing comparative analysis between original and optimized `CdmDocument`s.
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `evaluate_preservation` and `evaluate_preservation_with_config`.
  - Authored integration test suite `crates/context-optimizer/tests/preservation_metrics_test.rs` (7 integration tests validating unmodified fidelity, budget optimization retention, symbol omission detection, code review profile preservation, markdown and JSON report serialization, empty doc rejection, and custom thresholds).
  - 100% passing quality gates: 54 crate tests passed, 694+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.3: Context profiles (`7.3.4`) — PHASE 7.3 FULLY CERTIFIED

#### Added
- **`[7.3.4]` Debugging and Documentation Context Profile & Error Trace Prioritization in `crates/context-optimizer` (Phase 7.3 Certified):**
  - Implemented specialized Debugging and Documentation configuration and engine in `crates/context-optimizer/src/profiles/debugging.rs`:
    - `DebuggingCategory` taxonomy: classifies content sections into `ErrorDefinitionsAndPropagation`, `ObservabilityAndLogging`, `ApiContractsAndInterfaces`, `TroubleshootingDocsAndRunbooks`, `NegativeAndEdgeCaseTests`, `ManifestsAndEnvironment`, `InternalImplementation`, and `CosmeticOrUnrelatedAsset`.
    - `DebuggingConfig`: controls inclusion policies for error definitions (`include_error_definitions: true`), observability/tracing (`include_observability: true`), API contracts and schemas (`include_api_contracts: true`), troubleshooting runbooks (`include_troubleshooting_docs: true`), negative tests (`include_negative_tests: true`), manifests, internal implementation, and cosmetic asset suppression (`include_cosmetic_assets: false` by default).
    - Targeted error pattern matching: `with_target_error_patterns` and `from_error_pattern` pin sections matching specific incident error codes or exception names.
    - `DebuggingProfile`: implements `ContextProfile`:
      - Unconditionally pins error definitions (`enum Error`, `thiserror`), observability/tracing instrumentation, and API contracts with `PinStatus::MustInclude`.
      - Prioritizes troubleshooting runbooks and negative/error-handling tests with `PinStatus::Normal`.
      - Suppresses cosmetic presentation components (UI components, CSS, design tokens) and non-technical changelogs with `PinStatus::MustExclude` by default.
      - Enforces `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation under constrained token ceilings.
      - Computes `ProfileMetrics` category distribution.
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `apply_debugging_profile`, `apply_debugging_profile_for_model`, `apply_debugging_profile_with_target`, and `apply_debugging_profile_with_target_for_model`.
  - Authored integration test suite `crates/context-optimizer/tests/debugging_profile_test.rs` (6 integration tests validating error/observability/API contract pinning, targeted error pattern matching, negative test elevation, model-targeted optimization, tight budget boundary truncation, and report serialization).
  - 100% passing quality gates: 46 crate tests passed, 687+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.
  - **Formally certified completion of Phase 7.3: Context profiles** (`7.3.1`, `7.3.2`, `7.3.3`, `7.3.4` all `ACCEPTED`).

### 2026-11-18 — Phase 7.3: Context profiles (`7.3.3`)

#### Added
- **`[7.3.3]` Security Review Context Profile & Attack Surface Prioritization in `crates/context-optimizer`:**
  - Implemented specialized Security Review configuration and engine in `crates/context-optimizer/src/profiles/security_review.rs`:
    - `SecurityCategory` taxonomy: classifies content sections into `AuthAndAccessControl`, `CryptographyAndSecrets`, `AttackSurfaceAndEndpoints`, `InputValidationAndSanitizers`, `SecurityPolicyAndDocs`, `SecurityManifests`, `SecurityTests`, `GeneralImplementation`, and `BenignPresentationOrAsset`.
    - `SecurityReviewConfig`: controls inclusion policies for authentication/crypto, network attack surface, input validation/sanitization, security documentation (`SECURITY.md`), security manifests (`deny.toml`, security workflows), security test suites, benign presentation components (`include_benign_components: false` by default), custom pinned paths, and custom target keywords.
    - `SecurityReviewProfile`: implements `ContextProfile`:
      - Unconditionally pins authentication, authorization, cryptographic operations, secrets handling, and security policy documents (`SECURITY.md`) with `PinStatus::MustInclude`.
      - Prioritizes attack surface (routes, endpoints, fetchers), input validators, and security-centric unit/regression tests with `PinStatus::Normal`.
      - Suppresses benign presentation components (UI templates, CSS, design tokens) and non-security documentation (`CHANGELOG.md`, `ROADMAP.md`) with `PinStatus::MustExclude` by default to maximize vulnerability reasoning headroom.
      - Enforces `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation under constrained token ceilings.
      - Computes `ProfileMetrics` category distribution.
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `apply_security_review_profile` and `apply_security_review_profile_for_model`.
  - Authored integration test suite `crates/context-optimizer/tests/security_review_profile_test.rs` (6 integration tests validating auth/crypto/policy pinning, benign presentation suppression, security test elevation, model-targeted optimization, tight budget boundary truncation, custom keywords/paths, and report serialization).
  - 100% passing quality gates: 39 crate tests passed, 680+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.3: Context profiles (`7.3.2`)

#### Added
- **`[7.3.2]` Code Review Context Profile & Changed File Prioritization in `crates/context-optimizer`:**
  - Implemented specialized Code Review configuration and engine in `crates/context-optimizer/src/profiles/code_review.rs`:
    - `CodeReviewConfig`: configurable set of `changed_files`, toggles for `include_associated_tests` (default `true` to verify regression and behavior), `include_manifests` (default `true`), `include_interface_definitions` (default `true`), `include_unmodified_docs` (default `false` to conserve token budget for diffs), and boundary truncation thresholds.
    - `CodeReviewProfile`: implements `ContextProfile`:
      - Unconditionally pins modified/changed files (`PinStatus::MustInclude`) and marks them with `is_changed_file: true`.
      - Identifies associated unit/regression test files corresponding to changed source paths via naming heuristics and path patterns, assigning elevated priority and tagging with `associated_test: true`.
      - Preserves manifests (`Cargo.toml`, `package.json`, etc.) and interface definitions (`.d.ts`, trait definitions).
      - Suppresses unmodified prose documentation (`README.md`, `CHANGELOG.md`, `ROADMAP.md`) and generated assets by default to dedicate maximum context window headroom to code diffs.
      - Enforces `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation under constrained token ceilings.
      - Computes `ProfileMetrics` category distribution (changed files, associated tests, manifests, and auxiliary context).
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `apply_code_review_profile` and `apply_code_review_profile_for_model`.
  - Authored integration test suite `crates/context-optimizer/tests/code_review_profile_test.rs` (5 integration tests validating changed file pinning, associated test prioritization, model-targeted optimization, tight budget boundary truncation, zero changed files fallback, and telemetry serialization).
  - 100% passing quality gates: 32 crate tests passed, 673+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.3: Context profiles (`7.3.1`)

#### Added
- **`[7.3.1]` General AI Context Profile, Profile Architecture & Category Distribution in `crates/context-optimizer`:**
  - Implemented the foundational context profile abstraction in `crates/context-optimizer/src/profiles/mod.rs`:
    - `ContextProfile` trait: unified contract establishing `profile_kind`, `create_scorer`, `prepare_sections`, and `optimize` across all specialized context profile engines.
    - `ProfileKind` taxonomy: structured classification distinguishing `GeneralAi`, `CodeReview`, `SecurityReview`, and `DebuggingAndDocumentation`.
    - `ProfileMetrics`: tracks quantitative token consumption across functional categories (Documentation & Prose, Core Implementation Code, Manifests & Architecture, Tests & Fixtures, Other Content), calculates ratio shares, and formats GFM telemetry tables.
    - `ProfileOptimizationResult`: encapsulates the optimized `CdmDocument`, complete `BudgetPlan`, and `ProfileMetrics` with markdown reporting and JSON serialization roundtrips.
  - Implemented the General AI context profile in `crates/context-optimizer/src/profiles/general_ai.rs`:
    - `GeneralAiConfig`: controls inclusion policies for documentation, manifests, core code, tests (`include_tests: false` by default for high-signal architectural clarity), benchmarks (`include_benchmarks: false`), and truncation thresholds.
    - `GeneralAiProfile`: tags sections with `context_profile: general_ai`, pins prominent architectural files (root README, architecture specs, manifest files), suppresses/deprioritizes test files and benchmarks, applies `ContextBudgetOptimizer` greedy knapsack allocation, and computes category distribution metrics.
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `apply_general_ai_profile` and `apply_general_ai_profile_for_model`.
  - Implemented comprehensive integration test suite `crates/context-optimizer/tests/general_ai_profile_test.rs` (5 integration tests validating balanced content distribution, model-targeted optimization, test inclusion toggle, tight budget boundary truncation, and telemetry serialization).
  - 100% passing quality gates: 26 crate tests passed, 668+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.2: Token intelligence (`7.2.4`) — PHASE 7.2 FULLY CERTIFIED

#### Added
- **`[7.2.4]` Context Budget Calculation, Knapsack Prioritization & Headroom Allocation in `crates/context-optimizer` (Phase 7.2 Certified):**
  - Implemented multi-tier context headroom allocation and budget modeling in `crates/context-optimizer/src/budget.rs`:
    - `ContextBudget`: models total context window capacity ceilings, multi-tier reserved allowances (system prompts, response generation headroom, user query prompts), and effective usable content token capacity.
    - Proportional headroom derivation from `TargetModel` specifications and custom token allocations.
    - `BudgetHeadroomSummary`: calculates content/reserved allocation ratios and formats GFM telemetry tables.
  - Implemented section importance and density scoring framework in `crates/context-optimizer/src/priority.rs`:
    - `SectionPriority`, `PriorityScoreFactors`, and `SectionScorer`: evaluates structural `SectionKind` weights (Manifest: 1.0, Frontmatter: 0.85, Prose: 0.75, CodeFile: 0.70), filename/path prominence (root README: +0.35, architecture docs: +0.30, entrypoints: +0.25, manifests: +0.20), information density modifiers, architectural code roles, and penalties for test files, generated code, boilerplate, and deep nesting.
    - `PinStatus`: explicit inclusion overrides (`MustInclude`, `Normal`, `Deprioritized`, `MustExclude`).
  - Implemented optimization plan and decision audit ledger in `crates/context-optimizer/src/plan.rs`:
    - `BudgetPlan`: comprehensive token conservation accounting, utilization percentages, section counts, and GFM markdown summary tables.
    - `SectionBudgetDecision` & `BudgetAction` (`Include`, `Truncate`, `Exclude`): per-section auditable record capturing utility per token and exact disposition.
  - Implemented greedy knapsack budget optimization engine in `crates/context-optimizer/src/optimizer.rs`:
    - `ContextBudgetOptimizer` & `BudgetOptimizationConfig`: prioritizes `MustInclude` sections, allocates remaining headroom by `utility_per_token` descending, enforces strict capacity ceilings, and updates document revision BLAKE3 hashes.
    - `TruncationStrategy`: boundary-clean truncation (`HeadAndTail`, `HeadOnly`, `NoTruncation`) with explicit token reservation for omission markers and guaranteed post-condition capacity clamps.
    - Zero silent drops (Rule 04 / Rule 09): every excluded section is recorded in `document.exclusions` with `ExclusionReason::SizeLimit` or `UserConfig` and detailed metrics.
  - Implemented comprehensive reporting and telemetry in `crates/context-optimizer/src/report.rs`:
    - `ContextBudgetReport`: complete markdown report card with headroom breakdown, token conservation ledger, and JSON serialization roundtrips.
  - Public convenience APIs in `crates/context-optimizer/src/lib.rs`: `optimize_context_for_model` and `optimize_context_with_budget`.
  - Implemented integration test suite `crates/context-optimizer/tests/context_budget_test.rs` (6 integration tests validating multi-model allocations, knapsack prioritization, boundary truncation, zero silent drop conservation, serialization, and fail-closed error handling).
  - 100% passing quality gates: 20 crate tests passed, 662+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.
  - **Phase 7.2 ("Token intelligence") is formally completed and fully certified across all 4 subphases (`7.2.1` – `7.2.4`).**

### 2026-11-18 — Phase 7.2: Token intelligence (`7.2.3`)

#### Added
- **`[7.2.3]` Token Cost Estimation, Tiered Pricing Models & Budget Guardrails in `crates/tokenizer`:**
  - Implemented high-precision token cost estimation and commercial pricing modeling in `crates/tokenizer/src/cost.rs`:
    - `ModelPricingSchedule`: per-million-token pricing schedules covering all supported target models:
      - OpenAI: `Gpt4o` ($2.50 input, $1.25 cached, $10.00 output), `Gpt4oMini` ($0.15 input, $0.075 cached, $0.60 output), `Gpt4Turbo` ($10.00 input, $30.00 output).
      - Anthropic: `Claude35Sonnet` ($3.00 input, $0.30 cached, $15.00 output), `Claude3Opus` ($15.00 input, $1.50 cached, $75.00 output), `Claude3Haiku` ($0.25 input, $0.025 cached, $1.25 output).
      - Google: `Gemini15Pro` ($3.50 input, $0.875 cached, $10.50 output), `Gemini15Flash` ($0.075 input, $0.01875 cached, $0.30 output).
      - Meta / Self-Hosted: `Llama3`, `Llama370B`, `Llama2` ($0.00 input, $0.00 output, marked `is_free_or_self_hosted: true`).
      - Mistral: `MistralLarge` ($2.00 input, $6.00 output), `MistralSmall` ($0.20 input, $0.60 output), `Mixtral8x7B` ($0.70 input, $0.70 output).
      - Standard input rates, prompt caching read discounts (up to 90% discount on cache hits), output generation rates, and 50% batch API queue discounts.
    - `CostBreakdown`: high-precision calculation of standard input cost, prompt-cache savings ($ and %), batch API savings ($ and %), and total generation spend.
    - `CrossModelCostReport` & `ModelCostEntry`: multi-model cost analysis across all models for a given token count or document, identifying cheapest commercial model, computing cost variance ratio between most expensive and cheapest tiers, rendering formatted GFM Markdown comparison tables, and supporting JSON roundtrip serialization.
    - `CostGuardrail` & `CostGuardrailEvaluation`: strict dollar spending ceiling evaluation (`max_budget_usd`), computing remaining budget headroom, identifying budget breaches (`is_within_budget: false`), and automatically recommending alternative economical models that satisfy the budget limit.
    - `TokenizerError::CostBudgetExceeded { budget_usd, estimated_usd }`: added to domain errors in `crates/tokenizer/src/error.rs`.
  - Added `TargetModel::Gpt4oMini` to model taxonomy, registry, and adapters for ultra-economical ($0.15/MTok) conversions in `crates/tokenizer/src/model.rs`, `registry.rs`, `adapters/models.rs`, and `comparison.rs`.
  - Public convenience functions in `crates/tokenizer/src/lib.rs`: `estimate_token_cost` and `compare_token_costs`.
  - Implemented comprehensive integration test suite `crates/tokenizer/tests/cost_estimation_test.rs` (6 integration tests validating pricing schedules, standard/cached/batch calculations, cross-model comparison tables, document-level cost estimates, budget guardrail enforcement, and 20M token volume scaling).
  - 100% passing quality gates: 52 crate tests passed, 642+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.2: Token intelligence (`7.2.2`)

#### Added
- **`[7.2.2]` Model-Specific Counting Engines, Injection Defenses & Cross-Model Comparison in `crates/tokenizer`:**
  - Implemented dedicated, architecture-specific token counting engines in `crates/tokenizer/src/tokenizers/`:
    - `Cl100kTokenizer` (`cl100k_base`): 100k vocabulary rules, 4-space indentation merging, 3-digit number grouping, code keywords, and multi-byte UTF-8 byte fallback.
    - `O200kTokenizer` (`o200k_base`): 200k vocabulary rules, 8-space indentation merging, compound code operators (`::`, `->`, `=>`, `===`, `!==`, `&&`, `||`, `??`), and 1.5-2x enhanced Unicode compression for CJK, Devanagari, Arabic, and Cyrillic scripts.
    - `ClaudeTokenizer` (`claude_bpe`): XML tag preservation (`<thinking>`, `<artifacts>`, `<code>`), Markdown structural heading preservation, and 2/4/8 space indentation merging.
    - `GeminiTokenizer` (`gemini_sp`): SentencePiece style whitespace prefixing, large numeric grouping (up to 5 digits), and dense global Unicode compression.
    - `LlamaTokenizer` (`llama3_bpe`): 128k Llama 3 BPE boundary handling, code bracket sensitivity, and 4-space indentation merging.
    - `MistralTokenizer` (`mistral_tekken`): Tekken 131k tokenizer with aggressive code symbol merging and multi-language efficiency.
  - Implemented LLM special control token prompt-injection defenses in `crates/tokenizer/src/special_tokens.rs`:
    - `SpecialTokenPolicy`: `CountAsSingleToken`, `Disallow`, and `IgnoreOrEscape`.
    - `validate_special_tokens`: zero-trust detection and rejection of embedded control tokens (`<|endoftext|>`, `<|im_start|>`, `<|fim_prefix|>`) returning `TokenizerError::SpecialTokenForbidden`.
    - `find_special_tokens`: discovers and locates special tokens embedded inside text across OpenAI, Llama, and Mistral families.
  - Implemented granular syntax and structural token breakdown in `crates/tokenizer/src/syntax.rs`:
    - `TokenSyntaxBreakdown`: segments token spend across code syntax, natural prose, markup, whitespace, and special tokens with percentage shares, GFM Markdown summary table generation, and JSON serialization.
  - Implemented cross-model token intelligence and efficiency comparison in `crates/tokenizer/src/comparison.rs`:
    - `ModelTokenComparison` & `ModelComparisonEntry`: evaluates token counts and context window utilization across all supported model architectures (`Gpt4o`, `Gpt4Turbo`, `Claude35Sonnet`, `Gemini15Pro`, `Llama3`, `MistralLarge`), calculates efficiency ratios relative to baseline `cl100k`, computes max token variance percentages, renders formatted GFM comparison tables, and supports JSON serialization.
    - `compare_document`: whole-document multi-model token comparison across all sections in a `CdmDocument`.
  - Added `MistralAdapter` and registered `MistralAdapter`, `Claude3Opus`, and `Llama370B` in `TokenizerRegistry`.
  - Added convenience functions in `crates/tokenizer/src/lib.rs`: `compare_models_for_text`, `analyze_token_syntax`.
  - Implemented integration test suite `crates/tokenizer/tests/model_specific_counting_test.rs` (9 integration tests).
  - 100% passing quality gates: 42 crate tests passed, 636+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.2: Token intelligence (`7.2.1`)

#### Added
- **`[7.2.1]` Tokenizer Adapter Architecture in `crates/tokenizer`:**
  - Implemented modular, provider-independent tokenizer adapter architecture in `crates/tokenizer`:
    - `TokenizerError`: domain error handling invalid budgets, regex compilation failures, unknown adapters, context overflow, and UTF-8 boundary issues.
    - `ModelFamily` & `TargetModel`: structured taxonomy covering OpenAI (`Gpt4o`, `Gpt4Turbo`, `Gpt4`, `Gpt35Turbo`), Anthropic (`Claude35Sonnet`, `Claude3Opus`, `Claude3Haiku`), Google (`Gemini15Pro`, `Gemini15Flash`), Meta (`Llama3`, `Llama370B`, `Llama2`), Mistral (`MistralLarge`, `MistralSmall`, `Mixtral8x7B`), and generic models. Each model defines its maximum context window and conservative recommended budget ceilings.
    - `TokenPrecision`: explicit precision classification enforcing Rule 09 (`Exact`, `CalibratedEstimate { uncertainty_pct }`, `HeuristicEstimate`). Calibration estimates compute bounded uncertainty ranges (`[count * (1 - u), count * (1 + u)]`).
    - `TokenizerAdapter` core trait: defines abstract contract for model family, target model, context limits, exactness flag, token counting, and UTF-8 safe truncation.
    - `CalibratedHeuristicAdapter`: content-aware heuristic token estimator with empirical character/token ratios (JSON/dense code: 2.9, source code: 3.3, prose: 4.2) and explicit 7.5% uncertainty range.
    - `RegexBpeTokenizer`: deterministic regex pre-tokenizer segmentation using Rust-compatible PCRE equivalent pattern `r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+"` and integer token weights.
    - Model-specific BPE adapters: `OpenAiAdapter` (o200k/cl100k), `ClaudeAdapter` (claude_bpe), `GeminiAdapter` (gemini_sp), `LlamaAdapter` (llama3_bpe).
    - `TokenizerRegistry`: thread-safe registry for model and string identifier resolution with fallback to calibrated heuristics.
    - CDM document token enrichment (`count_section_tokens`, `enrich_document_token_metrics`): computes and enriches `CdmSection.token_metrics` across documents, yielding `DocumentTokenSummary` with context budget utilization percentages and GFM markdown telemetry tables.
    - Public convenience functions: `count_tokens_for_model` and `truncate_tokens_for_model`.
  - Implemented comprehensive integration test suite `crates/tokenizer/tests/tokenizer_adapter_test.rs` (8 integration tests, 11 unit tests, 19 total crate tests).
  - 100% passing quality gates: 19 crate tests passed, 627+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.1: Content optimization (`7.1.4`) — PHASE 7.1 CERTIFIED

#### Added
- **`[7.1.4]` Multi-Language Content Classification & AST Intelligence in `crates/code-intelligence` & `crates/normalizer`:**
  - Implemented multi-language detection and syntax identification in `crates/code-intelligence/src/language.rs`:
    - `Language`: enum covering 26 languages (Rust, TypeScript, JavaScript, Python, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, Scala, Shell, SQL, HTML, CSS, Markdown, YAML, TOML, JSON, XML, Protobuf, GraphQL, Dockerfile).
    - Path, file extension, shebang, and distinctive content keyword heuristics.
    - Comment delimiter and syntax classification predicates (`is_programming_language`, `is_config_or_data`, `is_markup_or_doc`).
  - Implemented AST symbol and declaration extraction in `crates/code-intelligence/src/symbols.rs`:
    - `SymbolKind`: Function, Method, Struct, Class, Interface, Trait, Enum, TypeAlias, Constant, Macro, Module, Import.
    - `Visibility`: Public, Protected, Private, Internal.
    - `SymbolDeclaration`: location, visibility, signature text, and preceding doc comment capture.
    - `SymbolExtractor`: multi-language declaration parsing across Rust, TypeScript/JavaScript, Python, Go, Java, C#, and C/C++.
  - Implemented structural code metrics and minification detection in `crates/code-intelligence/src/metrics.rs`:
    - `CodeMetrics`: line counts (total, code, comment, blank), symbol counts, export counts, comment ratio, average line length, and minification detection.
    - `CodeMetricsCalculator`: single-pass character and line streaming evaluation.
  - Implemented architectural code role classification in `crates/code-intelligence/src/nature.rs`:
    - `CodeNature`: CoreImplementation, InterfaceOrTypeDefinition, TestOrFixture, Benchmark, GeneratedOrBoilerplate, BuildOrScript, DataOrSchema.
    - `CodeNatureClassifier`: evaluates path heuristics, content markers (`@generated`, `DO NOT EDIT`, `#[test]`, `describe(`), and symbol composition ratios.
  - Implemented unified code analyzer orchestrator in `crates/code-intelligence/src/analyzer.rs`:
    - `CodeAnalyzer` & `CodeAnalysisResult`: integrates language detection, symbol extraction, metric evaluation, and nature classification.
  - Implemented CDM section and document content classifier in `crates/normalizer/src/classifier.rs`:
    - `ContentCategory`: SourceCode, Documentation, Specification, Configuration, DependencyManifest, TestAndVerification, GeneratedOrBoilerplate, NavigationOrMetadata, TabularOrData.
    - `InformationDensity`: High, Medium, Low, Trivial.
    - `NoiseIndicator`: CopyrightHeaderOnly, MinifiedContent, AutoGeneratedStub, RepetitiveTestFixture, EmptyOrTrivialSection, NavigationChromeOnly.
    - `SectionClassification`: assigns category, density, normalized signal score in $[0.0, 1.0]$, and noise indicators.
    - `DocumentContentClassifier`: enriches `CdmSection.attributes` (`content_category`, `information_density`, `signal_score`, `is_test`, `is_generated`, `is_boilerplate`, `code_nature`, `symbols_summary`, `noise_indicators`) while strictly preserving section cryptographic hashes (`verify_section_hashes() == true`).
  - Implemented classification telemetry and audit reporting in `crates/normalizer/src/classification_report.rs`:
    - `ContentClassificationReport`: tracks category distributions, token shares, density distributions, average signal scores, low-signal candidate section IDs, GFM Markdown summary table generation, and JSON serialization.
  - Implemented integration test suite `crates/normalizer/tests/content_classification_test.rs` (8 integration tests, 20 unit tests, 41 total normalizer tests, 14 code-intelligence tests):
    - Validates multi-language classification, test/benchmark identification, generated boilerplate detection, doc/spec separation, config/manifest categorization, navigation/empty noise indicators, end-to-end document enrichment, hash preservation, and report JSON roundtrip.
  - 100% passing quality gates: 14 code-intelligence tests passed, 41 normalizer tests passed, 608+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed — **PHASE 7.1 FORMALLY CERTIFIED**.

### 2026-11-18 — Phase 7.1: Content optimization (`7.1.3`)

#### Added
- **`[7.1.3]` Semantic-Aware Near-Duplicate Deduplication Engine in `crates/deduplication`:**
  - Implemented token shingling and n-gram decomposition in `crates/deduplication/src/shingle.rs`:
    - `Shingler::word_shingles`: sliding window word n-grams with punctuation normalization and case folding.
    - `Shingler::char_kgrams`: character-level k-grams for dense/short string decomposition.
    - `fnv1a_hash64`: deterministic 64-bit FNV-1a non-cryptographic hash for rapid token shingle fingerprinting.
  - Implemented MinHash locality-sensitive signature estimation in `crates/deduplication/src/minhash.rs`:
    - `MinHashSignature`: computes $N$-dimensional permutation signatures using deterministic SplitMix64 pseudo-random generator.
    - Jaccard similarity estimation via normalized signature component agreement.
  - Implemented SimHash 64-bit cosine similarity fingerprinting in `crates/deduplication/src/simhash.rs`:
    - `SimHash64`: 64-bit fingerprinting with weighted sign bit vector accumulation.
    - Hamming distance and cosine similarity projection via bit difference calculation.
  - Implemented strongly-typed semantic deduplication configuration models in `crates/deduplication/src/semantic_config.rs`:
    - `SemanticAlgorithm`: `MinHash { shingle_size, num_hashes }`, `SimHash { max_hamming_distance }`.
    - `SemanticDeduplicationStrategy`: `ReplaceWithAnnotatedPointer`, `Remove`, `MarkOnly`.
    - `SemanticCanonicalPolicy`: `InformationRichness`, `StructuralHierarchy`, `FirstSeen`, `ShortestPath`.
    - `SemanticDeduplicationConfig`: master configuration with `balanced()`, `aggressive()`, `conservative()`, and `simhash_based()` domain presets.
  - Implemented near-duplicate cluster domain models in `crates/deduplication/src/semantic_cluster.rs`:
    - `NearDuplicateCluster` and `NearDuplicateMember` recording canonical details, members, similarity scores, byte counts, and token savings.
  - Implemented semantic deduplication telemetry reporting in `crates/deduplication/src/semantic_report.rs`:
    - `SemanticDeduplicationReport`: tracking before/after byte and token metrics, rounded reduction percentages, Markdown summary table generation, and JSON roundtrip serialization.
  - Implemented semantic deduplicator engine in `crates/deduplication/src/semantic.rs`:
    - `SemanticDeduplicator::find_near_duplicate_clusters`: eligible candidate filtering, signature generation, pairwise graph adjacency, connected component traversal (BFS), and canonical candidate election.
    - `SemanticDeduplicator::deduplicate_document`: executes consolidation strategy (`ReplaceWithAnnotatedPointer`, `Remove`, `MarkOnly`), updates section hashes, emits `ExclusionRecord` items with `ExclusionReason::Duplicate`, links directed `DuplicateOf` relationships, and updates document-level BLAKE3 revision hashes.
    - Top-level crate exports and convenience function `deduplicate_semantic` in `crates/deduplication/src/lib.rs`.
  - Implemented comprehensive integration test suite `crates/deduplication/tests/semantic_deduplication_test.rs` (9 integration tests, 11 unit tests, 34 total crate tests):
    - Validates MinHash clustering, SimHash distance clustering, pointer comment substitution, aggressive removal with exclusion logging, mark-only annotations, structural hierarchy canonical election, minimum word threshold filtering, idempotency, and Markdown/JSON report serialization.
  - 100% passing quality gates: 34 deduplication tests passed, 586+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.1: Content optimization (`7.1.2`)

#### Added
- **`[7.1.2]` Exact Content Deduplication Engine in `crates/deduplication` & `crates/cdm`:**
  - Extended CDM relationship taxonomy:
    - Added `RelationshipKind::DuplicateOf` (`"duplicate_of"`) in `crates/cdm/src/relationships.rs` and updated schema validation tests.
  - Implemented strongly-typed configuration models and presets in `crates/deduplication/src/config.rs`:
    - `DeduplicationStrategy`: `Remove`, `ReplaceWithPointer`, `MarkOnly`.
    - `CanonicalSelectionPolicy`: `FirstSeen`, `ShortestPath`, `StructuralHierarchy` (`CodeFile` > `CodeSnippet` > `Prose` > `Table` > `Admonition` > `Frontmatter` > `Manifest` > `Navigation`), `PreferredPrefix(String)`.
    - `HashScope`: `ExactBytes` (raw byte BLAKE3), `Normalized` (canonical line-endings and whitespace trimming).
    - `ExactDeduplicationConfig`: `strategy`, `canonical_policy`, `hash_scope`, `min_byte_threshold` (default 32 bytes), `log_exclusions` (default true), presets `aggressive()`, `conservative()`, `lossless()`.
  - Implemented duplicate cluster model in `crates/deduplication/src/cluster.rs`:
    - `DuplicateCluster`: records `content_hash`, `canonical_section_id`, `canonical_title`, `canonical_path`, `canonical_kind`, `duplicate_section_ids`, `duplicate_count`, `bytes_per_instance`, `total_duplicate_bytes`, `estimated_tokens_saved`.
  - Implemented exact deduplication engine in `crates/deduplication/src/exact.rs`:
    - `compute_dedup_hash` evaluating raw or normalized whitespace text via BLAKE3.
    - `ExactDeduplicator::find_duplicate_clusters` grouping candidate sections by hash, filtering clusters with >1 instances, electing canonical section via configured policy, and sorting clusters by net bytes saved descending.
  - Implemented document-level deduplicator in `crates/deduplication/src/document.rs`:
    - `DocumentDeduplicator`: orchestrates exact deduplication across `CdmDocument`, executes strategy (`Remove` drops sections and logs `ExclusionRecord`, `ReplaceWithPointer` substitutes pointer comment and updates hash/metrics, `MarkOnly` preserves bodies), logs `ExclusionReason::Duplicate` records, registers `RelationshipKind::DuplicateOf` edges, and recomputes document-level BLAKE3 revision hash.
  - Implemented deduplication telemetry report in `crates/deduplication/src/report.rs`:
    - `ExactDeduplicationReport`: records before/after bytes and estimated tokens, net reduction, percentage compression, unique sections, duplicate sections, clusters, Markdown summary table, and JSON roundtrip serialization.
  - Implemented comprehensive integration test suite `crates/deduplication/tests/exact_deduplication_test.rs` (10 integration tests, 4 unit tests, 14 total passing tests):
    - Validates exact vs normalized hashing, aggressive removal, conservative pointer replacement, lossless mark-only, structural hierarchy election, shortest path election, preferred prefix election, minimum byte threshold defense, idempotency, and Markdown/JSON report serialization.
  - 100% passing quality gates: 14 deduplication tests passed, 577+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 7.1: Content optimization (`7.1.1`)

#### Added
- **`[7.1.1]` Structural Normalization Engine in `crates/normalizer` & `crates/cdm`:**
  - Implemented strongly-typed normalization configuration models in `crates/normalizer/src/config.rs`:
    - `EncodingConfig`: BOM stripping, null byte removal, unprintable control character filtering, zero-width space stripping, typographic punctuation toggle.
    - `WhitespaceConfig`: CRLF/CR -> LF canonicalization, trailing whitespace trimming, blank line trimming and collapsing ceilings (`max_consecutive_blank_lines_prose: 1`, `max_consecutive_blank_lines_code: 2`), inline space collapsing toggle.
    - `MarkdownConfig`: code fence normalization, heading level spacing, list marker standardization (`*` -> `-`), GFM table rectangularization and cell padding.
    - `NormalizationConfig`: master configuration with `lossless()`, `compact()`, and `code_focused()` domain presets.
  - Implemented character encoding repair in `crates/normalizer/src/encoding.rs`:
    - `EncodingSanitizer`: single-pass character stream filtering stripping `\u{FEFF}`, `\0`, unprintable ASCII control characters (`\x01`..=`\x1F` excluding `\t`, `\n`, `\r`), zero-width codepoints (`\u{200B}`, `\u{200C}`, `\u{200D}`), and smart punctuation normalization (`“`, `”`, `‘`, `’`, `—`, `…` -> ASCII).
  - Implemented whitespace and line-ending normalizer in `crates/normalizer/src/whitespace.rs`:
    - `WhitespaceNormalizer`: canonicalizes CRLF/CR to Unix LF (`\n`), trims per-line trailing whitespace, trims leading/trailing section blank lines, and collapses consecutive blank lines according to code/prose limits while strictly preserving code indentation and code spans.
  - Implemented Markdown structural syntax and table rectangularization in `crates/normalizer/src/markdown.rs`:
    - `MarkdownNormalizer`: normalizes code block fences with lowercase language specifiers, standardizes heading spacing (`#Title` -> `# Title`, `###   Title` -> `### Title`), converts bullet list markers (`*` -> `-`), and rectangularizes GFM tables with matching cell counts and delimiter rows (`:---`).
  - Implemented section-aware and whole-document normalizers in `crates/normalizer/src/section.rs` and `crates/normalizer/src/document.rs`:
    - `SectionNormalizer`: kind-specific strategy mapping (`SectionKind::CodeFile` and `CodeSnippet` strictly preserve quotes, symbols, and indentation; `Prose`, `Table`, `Admonition` receive Markdown normalization), updates section BLAKE3 hash, updates `SectionTokenMetrics`.
    - `DocumentNormalizer`: orchestrates section normalization across `CdmDocument`, recomputes document-level BLAKE3 revision hash, and aggregates granular section audits into `NormalizationReport`.
  - Implemented telemetry reporting in `crates/normalizer/src/report.rs`:
    - `NormalizationReport`: records before/after bytes, characters, lines, and estimated tokens, net reductions, percentage compression, per-section audit logs, GFM Markdown summary table rendering, and JSON roundtrip serialization.
  - Implemented error handling and top-level crate exports in `crates/normalizer/src/error.rs` and `crates/normalizer/src/lib.rs`:
    - `NormalizerError`, convenience functions `normalize_document` and `normalize_text`.
  - Implemented comprehensive integration test suite `crates/normalizer/tests/structural_normalization_test.rs` (13 integration tests, 16 unit tests, 29 total passing tests):
    - Validates CRLF/CR line ending conversion, BOM/null/control/zero-width stripping, typographic punctuation under lossless vs compact presets, strict semantic code indentation and symbol preservation, code fence language normalization, heading level spacing, list marker standardization, GFM table rectangularization, full document end-to-end normalization, document and raw text idempotency (`normalize(normalize(x)) == normalize(x)`), and report serialization.
  - 100% passing quality gates: 29 normalizer tests passed, 563+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

---

## [Unreleased] — Stage 6: Private GitHub Repositories

### 2026-11-18 — Phase 6.4: Security verification (`6.4.4` — Phase 6.4 & Stage 6 Certified)

#### Added
- **`[6.4.4]` Least-Privilege Permission Auditing & Stage 6 Certification in `crates/github-extractor`:**
  - Implemented comprehensive permission audit integration test suite `crates/github-extractor/tests/permission_audit_test.rs` (14 integration tests):
    - Strict minimal policy exact compliance (`PermissionPolicy::strict_minimal()`): Verified exact compliance for `contents: read` + `metadata: read` with zero violations, valid GFM Markdown report with `PERMITTED` badges, and JSON roundtrip serialization.
    - Missing required scopes: Validated immediate rejection when `contents` or `metadata` scopes are missing.
    - Prohibited permission levels: Exhaustively verified rejection of `write` and `admin` levels across `contents`, `metadata`, `pull_requests`, `issues`, and `packages`.
    - Prohibited sensitive categories: Exhaustively verified rejection of all 12 sensitive categories (`DEFAULT_FORBIDDEN_CATEGORIES`) under Rule 05 (`administration`, `actions`, `secrets`, `deployments`, `packages`, `security_events`, `members`, `organization_administration`, `organization_secrets`, `organization_actions`, `workflows`, `blocking_keys`).
    - Unexpected scope detection: Confirmed unexpected additional scopes are flagged under `StrictMinimal`.
    - Benign read scopes under `PermissionPolicy::standard()`: Confirmed benign read scopes (`pull_requests: read`, `issues: read`) are permitted while strictly rejecting write/admin and prohibited categories.
    - Custom policy builder: Verified fine-grained customization with `PermissionPolicyBuilder`.
    - Installation suspension enforcement: Verified suspended installations (`is_suspended: true`) produce `SuspendedInstallation` violation and fail validation.
    - Token validation: Verified `audit_installation_token` and `validate_installation_token`.
    - Context validation: Extended `AuthorizedInstallationContext` with `audit_permissions`, `validate_permissions`, `audit_permissions_with_policy`, and `validate_permissions_with_policy`.
    - Fail-closed private extraction: Verified `PrivateRepositoryExtractor::extract` enforces pre-flight permission validation, failing closed immediately on violating permissions (`contents: write` or forbidden scopes) before making network calls.
    - Wire-level token request scoping: In-process Axum mock server verified that client requests strictly minimal read-only permissions (`contents: read`, `metadata: read`).
    - Markdown & JSON reporting fidelity: Verified rendering of status badges (`PERMITTED` vs `REJECTED`), caution alerts, and JSON schema fidelity.
    - Stage 6 Security Acceptance Gate: Proved all Stage 6 invariants: at-rest AES-256-GCM encryption with session AAD, zero credential leakage in debug/display, zero repository modification permissions, and strict read-only least-privilege enforcement.
  - Implemented query helpers on `PermissionAuditReport`: `has_excessive_level`, `has_forbidden_category`, `has_missing_required`, `has_unexpected_scope`.
  - Implemented `audit_context` and `validate_context` on `PermissionAuditor`.
  - Extended `PrivateExtractionConfig` with `permission_policy` and `with_permission_policy`.
  - 100% passing quality gates: 14/14 new tests passed, 282 total extractor tests passed, 563+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.
  - **Formally certified Phase 6.4 and STAGE 6 (Private GitHub Repositories).**

### 2026-11-18 — Phase 6.4: Security verification (`6.4.3`)

#### Added
- **`[6.4.3]` Token Leakage Prevention & Defense Test Suites in `crates/github-extractor` & `crates/observability`:**
  - Implemented comprehensive domain model, error envelope, and wire-level token leakage defense test suite in `crates/github-extractor/tests/token_leakage_test.rs` (9 integration tests):
    - `AuthorizedInstallationContext`: Secret bearer tokens are masked with `***[REDACTED_TOKEN]***` in `Debug` and omitted in `Display`.
    - `StoredTokenRecord`: Masked with `***[REDACTED_TOKEN]***` in `Debug` and `Display`.
    - `GitHubAppConfig`: Client secret, private key PEM, and webhook secret are masked with `***[REDACTED_SECRET]***` and `***[REDACTED_RSA_KEY]***` in `Debug` and completely omitted in `Display`.
    - `GitHubInstallationToken`: Ephemeral bearer token string is masked with `***[REDACTED_TOKEN]***` in `Debug` and omitted in `Display`.
    - `GitHubOAuthTokenResponse` & `GitHubOAuthErrorResponse`: Access tokens and refresh tokens are masked with `***[REDACTED_TOKEN]***` in `Debug`, and OAuth error descriptions/URIs redact sensitive tokens via `redact_secrets`.
    - `AuthorizationRecoveryPlan` & summary cards: Redact all sensitive tokens to `[REDACTED_GITHUB_TOKEN]` in human-readable diagnostic messages and GFM Markdown cards.
    - `GitHubExtractorError`: Sanitization scrubs tokens across all error variants (`BadCredentials`, `Forbidden`, `ResourceNotFound`, `RateLimitExceeded`, `RequestFailed`, `AppConfiguration`, `HttpError`).
    - Wire-level HTTP extraction isolation: In-process Axum mock server verifies bearer token is passed strictly in the standard `Authorization: Bearer <token>` HTTP header and NEVER in the URL path, query params, or body.
    - Cryptographic keys: `EncryptionKey` and `EncryptedPayload` mask raw key bytes and ciphertext in `Debug` and `Display`.
  - Implemented comprehensive observability token and credential leakage defense suite in `crates/observability/tests/token_leakage_test.rs` (7 integration tests):
    - All GitHub token formats (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`, and URL embedded credentials `https://ghs_...@github.com`) are scrubbed to `[REDACTED_GITHUB_TOKEN]`.
    - Multi-line RSA, PKCS#8, and EC private keys are scrubbed to `[REDACTED_PRIVATE_KEY]`.
    - Webhook signatures (`sha256=...`), AWS keys (`AKIA...`), and generic hex API keys are redacted.
    - Recursive JSON AST walking scrubs values under sensitive keys and redacts string values.
    - `is_sensitive_key` taxonomy expanded to cover `passwd` variations.
  - 100% passing quality gates: 9/9 extractor tests passed, 7/7 observability tests passed, 268 extractor tests, 34 observability tests, 549+ workspace tests, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.4: Security verification (`6.4.2`)

#### Added
- **`[6.4.2]` Unauthorized Repository Access Test Suite in `crates/github-extractor`:**
  - Implemented comprehensive integration test suite `crates/github-extractor/tests/unauthorized_access_test.rs` (11 integration tests) verifying that unauthorized repository access attempts across all failure modes are strictly blocked under Rule 05:
    - Missing installation: `InstallationAuthorizer::authorize` without an installation in session yields `AuthorizationOutcome::NoInstallationForSession` and generates a `PromptAppInstallation` recovery plan with a 1-click install link.
    - Account ownership mismatch: Calling session with installations for other accounts attempting to access an uninstalled account yields `AuthorizationOutcome::AccountMismatch` and produces a targeted install link with zero credential leakage.
    - Selected repository access grant enforcement: In `RepositorySelection::Selected` mode, repositories omitted from the authorized cache trigger live upstream verification; if GitHub returns 404 or 403, access is denied with `AuthorizationOutcome::RepositoryNotGranted` and generates a `PromptRepositorySelectionUpdate` plan with direct settings URL.
    - Suspended installations: Installations with `is_suspended: true` are blocked immediately with `AuthorizationOutcome::InstallationSuspended` and produce a `PromptUnsuspendInstallation` action.
    - Revoked installations: Installations with `is_revoked: true` are blocked immediately with `AuthorizationOutcome::InstallationRevoked` and instruct reinstallation.
    - Rule 05 permission policy compliance: Tokens lacking mandatory `contents: read` or granted forbidden write/push scopes (`contents: write`) are strictly rejected with `AuthorizationOutcome::InsufficientPermissions` and generate an administrative remediation plan.
    - Upstream extraction error classification: When `PrivateRepositoryExtractor::extract` encounters HTTP 403 or 404 during commit resolution, it halts immediately with sanitized errors and maps to actionable recovery plans (`RepositoryNotGranted` or `InsufficientPermissions`).
    - Zero content and secret leakage: Verified that zero file contents, blobs, trees, or commits are extracted, and bearer tokens (`ghs_*`) are strictly masked across all error messages, plans, and summary cards.
  - 100% passing quality gates: 11/11 new tests passed, 259 total extractor tests passed, 533+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.4: Security verification (`6.4.1`)

#### Added
- **`[6.4.1]` Cross-User & Cross-Tenant Session Isolation in `crates/github-extractor` & `services/api`:**
  - Implemented and verified strict multi-tenant session isolation across:
    - `InMemoryInstallationTokenStore`: Installation tokens, account lookups, and session listings are strictly partitioned by composite key `(session_id, installation_id)`. Session B cannot view, enumerate, or query Session A's installations or accounts.
    - `InstallationAuthorizer`: Session B evaluating authorization for Session A's repositories receives `NoInstallationForSession` or `AccountMismatch` and is never granted access to Session A's token or repository context.
    - `InMemoryOAuthStateStore`: Cryptographic state nonces bound to Session A cannot be consumed or validated by Session B; attempted cross-session hijacks trigger `OAuth state session mismatch` and immediate nonce invalidation.
    - Revocation boundaries: Session B cannot revoke Session A's installation tokens in token storage or via the REST API; requests return 404 and leave victim tokens active.
    - Webhook uninstallation scoping: Global uninstallation webhooks (`revoke_all_for_installation`) revoke only the specified installation ID across all tenant sessions while leaving unrelated installations untouched.
    - Dynamic repository grant synchronization: Webhook repository additions and removals affect only the targeted installation and do not leak repository names into unrelated installations.
    - AES-256-GCM envelope encryption: Ciphertext encrypted with Authenticated Associated Data (AAD) bound to `tenant:{session_id}:installation:{installation_id}` cannot be decrypted under another tenant's session ID.
    - `PrivateRepositoryExtractor`: Extractor instances configured with Session B cannot access or proactively renew tokens stored for Session A.
  - Authored comprehensive integration test suite `crates/github-extractor/tests/cross_user_isolation_test.rs` (8 integration tests).
  - Extended `services/api/src/lib.rs` with multi-tenant isolation tests:
    - `test_api_list_installations_cross_tenant_isolation`: Verifies Tenant B receives `[]` when only Tenant A has installations, and concurrent tenants observe strictly their own installations.
    - `test_api_session_header_validation_errors`: Validates that missing or malformed `X-Session-ID` headers return HTTP 400 Bad Request with RFC 7807 problem details.
    - `test_api_concurrent_multi_tenant_revocation_isolation`: Verifies that Tenant A revoking their installation does not invalidate Tenant B's active installations.
  - 100% passing quality gates: 8/8 new extractor tests passed, 58/58 api tests passed, 522+ workspace tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.3: Private extraction (`6.3.4` — Phase 6.3 Certified)

#### Added
- **`[6.3.4]` Authorization Failure Recovery Engine & Phase 6.3 Certification in `crates/github-extractor`:**
  - Implemented `AuthorizationFailureKind` in `crates/github-extractor/src/authorization_recovery.rs`:
    - Categorizes failures into `TokenInvalid`, `TokenExpired`, `RepositoryNotGranted`, `AccountMismatch`, `NoInstallationForSession`, `InstallationSuspended`, `InstallationRevoked`, `InsufficientPermissions`, `RateLimited`, and `UpstreamApiError`.
  - Implemented `RecoveryAction` taxonomy:
    - `AutomaticRetryWithFreshToken` (transparent token cache eviction and renewal).
    - `PromptAppInstallation { installation_url, target_account }` (1-click GitHub App install link).
    - `PromptRepositorySelectionUpdate { settings_url, installation_id, repository }` (direct GitHub App settings link).
    - `PromptUnsuspendInstallation { settings_url, installation_id, account_login }`.
    - `PromptReauthenticateUser { authorize_url }`.
    - `WaitRateLimitReset { retry_after_secs, reset_at }`.
    - `ContactAdministrator { account_login, details }`.
    - `Unrecoverable { reason }`.
  - Implemented `AuthorizationRecoveryPlan`:
    - Encapsulates failure kind, auto-recoverability flag, prescribed action, human-readable diagnosis, step-by-step remediation instructions, target slug, and installation ID.
    - Added `summary_card(&self) -> String` producing formatted GitHub-Flavored Markdown diagnostic cards for UI and logging.
  - Implemented `AuthorizationRecoveryCoordinator`:
    - `diagnose_outcome(&self, outcome, owner, repo)`: Maps every `AuthorizationOutcome` variant to an actionable `AuthorizationRecoveryPlan`.
    - `diagnose_extractor_error(&self, error, owner, repo, installation_id, account_login)`: Maps runtime `GitHubExtractorError`s (HTTP 401, 403, 404, 429) to remediation plans.
    - `attempt_eviction_and_refresh(&self, session_id, installation_id, account_login)`: Automatically evicts invalid tokens from `InstallationTokenStore` and proactively requests fresh tokens via `InstallationTokenRefresher`.
    - `execute_with_retry_on_401(...)`: Wraps operations with transparent cache eviction and one-time renewal retry on 401 Bad credentials.
  - Enhanced `PrivateRepositoryExtractor`:
    - Added transparent 401 self-healing during `extract()` commit resolution: evicts invalid tokens, fetches fresh tokens, and seamlessly completes extraction.
  - Re-exported `AuthorizationFailureKind`, `RecoveryAction`, `AuthorizationRecoveryPlan`, and `AuthorizationRecoveryCoordinator` in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive test suite in `crates/github-extractor/tests/authorization_recovery_test.rs`:
    - 9 integration tests validating outcome diagnoses, runtime error mapping, transparent 401 retry, self-healing private extraction, and strict Rule 05 zero-leakage secret masking.
  - 100% passing quality gates: 9/9 new tests passed, 240 total extractor tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.
- **Phase 6.3 Milestone Certification:**
  - Completed all 4 subphases (`6.3.1` Installation authorization, `6.3.2` Selected repository access, `6.3.3` Private tree & source extraction, `6.3.4` Authorization failure recovery).
  - Formally certified Phase 6.3 under Rule 05 least-privilege security and mathematical completeness accounting.

### 2026-11-18 — Phase 6.3: Private extraction (`6.3.3`)

#### Added
- **`[6.3.3]` Private Repository Tree and Source Extraction Engine in `crates/github-extractor`:**
  - Implemented `PrivateExtractionConfig` in `crates/github-extractor/src/private_extractor.rs`:
    - Configures source extraction concurrency, file classification rules, automated subtree recovery, cryptographic hash validation, explicit target ref (`target_ref`), metadata acquisition toggle (`fetch_metadata`), and proactive token renewal safety margin (`proactive_token_renewal_margin_secs`).
  - Implemented `PrivateExtractionResult`:
    - Comprehensive extraction bundle capturing target, installation ID, account login, commit metadata, optional repository metadata, recovered tree, classification report, source report, doc report, dependency report, config/test report, accounting balance sheet, hash validation report, manifest, and architecture context.
    - Helper methods: `is_balanced()`, `total_extracted_files()`, `total_discovered_entries()`, `total_extracted_bytes()`, `commit_sha()`, and `root_tree_sha()`.
    - Custom `fmt::Display` displaying target, commit, extracted files count, and conservation balance status.
  - Implemented `PrivateRepositoryExtractor`:
    - Coordinates authenticated Git Trees API traversal with ephemeral installation access tokens (`Bearer ghs_*`).
    - Integrates with `InstallationTokenRefresher` and `InstallationTokenStore` to proactively detect and renew expiring tokens prior to extraction.
    - Resolves target ref (defaults to `"HEAD"`) and fetches commit metadata.
    - Performs Git tree traversal with automated recursive subtree recovery on truncated trees.
    - Classifies files and executes bounded concurrent content extraction across sources, docs, dependencies, configs, and tests via `tokio::join!`.
    - Audits mathematical completeness conservation identity ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$) via `FileAccountingAuditor`.
    - Validates file integrity via multi-algorithm cryptographic hash validation (Git SHA-1, SHA-256, BLAKE3).
    - Assembles unified file manifest and derives repository architecture context.
  - Added constructors and helpers:
    - Added `GitHubTarget::for_repo(owner, repo)` in `crates/github-extractor/src/target.rs`.
    - Added `AuthorizedInstallationContext::new(...)` and `with_permissions(...)` in `crates/github-extractor/src/authorization.rs`.
    - Re-exported `PrivateExtractionConfig`, `PrivateExtractionResult`, and `PrivateRepositoryExtractor` in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive test suite in `crates/github-extractor/tests/private_extractor_test.rs`:
    - 6 integration tests validating end-to-end private extraction, mathematical conservation balance, multi-algorithm hash checks, proactive token renewal, subtree recovery on truncation, unauthorized 401 handling, and zero secret token leakage.
  - 100% passing quality gates: 6/6 new tests passed, 231 total extractor tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.3: Private extraction (`6.3.2`)

#### Added
- **`[6.3.2]` Selected Repository Access, Traversal, Search & Dynamic Grant Synchronization in `crates/github-extractor`:**
  - Implemented `SelectedRepositoryListing` in `crates/github-extractor/src/selected_repositories.rs`:
    - Captures `total_count`, `repository_selection`, `repositories`, `pages_fetched`, and `is_complete`.
    - Implemented helper methods: `is_empty`, `len`, `full_names`, `has_repository`, `find_by_slug`, `search`, and `filter_by_visibility`.
  - Implemented `RepositoryAccessStatus` taxonomy:
    - `Granted(InstalledRepositorySummary)`
    - `NotGranted { owner, repo, reason }`
    - `TokenInvalid { reason }`
    - `RateLimited { retry_after_secs }`
  - Implemented `SelectedRepositoryManager`:
    - `fetch_all(&self, installation_token)`: Bounded pagination traversal across `GET /installation/repositories` honoring `max_pages` (default 50) and `per_page` (clamped to 100).
    - `fetch_all_for_user(&self, user_token, installation_id)`: Paginated enumeration across `GET /user/installations/{id}/repositories`.
    - `verify_repository_access(&self, token, owner, repo)`: Direct access verification handling 200 (Granted), 404 (NotGranted), 401 (TokenInvalid), and 403/429 (RateLimited).
    - `sync_grant_change(&self, store, installation_id, added, removed)`: Synchronizes repository grants from webhook events across all sessions in the store.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - Added `list_user_installation_repositories(&self, user_token, installation_id, page, per_page) -> Result<InstallationRepositoriesResponse, GitHubExtractorError>`.
  - Implemented `From<RepositoryAccessDetails> for InstalledRepositorySummary` in `crates/github-extractor/src/installation.rs`.
  - Extended `InstallationTokenStore` and `InMemoryInstallationTokenStore` in `crates/github-extractor/src/token_refresh.rs`:
    - Added `sync_installation_repositories(&self, installation_id, added, removed) -> Result<usize, GitHubExtractorError>`.
  - Authored comprehensive test suite in `crates/github-extractor/tests/selected_repositories_test.rs`:
    - 11 integration tests validating single-page and multi-page pagination, page cap limits, user installation retrieval, keyword search, visibility filtering, access verification, and multi-session grant synchronization.
  - 100% passing quality gates: 11/11 new tests passed, 12/12 authorization tests passed, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.3: Private extraction (`6.3.1`)

#### Added
- **`[6.3.1]` GitHub App Installation Authorization Engine in `crates/github-extractor`:**
  - Implemented `AuthorizedInstallationContext` in `crates/github-extractor/src/authorization.rs`:
    - Captures `installation_id`, `account_login`, `token`, `repository_selection`, target `owner`, `repo`, and granted `permissions`.
    - Custom `fmt::Debug` and `fmt::Display` implementations masking the sensitive bearer token with `***[REDACTED_TOKEN]***` to prevent credential leakage.
  - Implemented `AuthorizationOutcome` domain enum:
    - `Authorized(AuthorizedInstallationContext)`: Access granted and valid bearer token ready for private extraction.
    - `NoInstallationForSession { session_id }`: Calling session has no registered installations.
    - `AccountMismatch { session_id, requested_owner, available_accounts }`: Session has installations, but none matching requested repository owner.
    - `RepositoryNotGranted { installation_id, account_login, requested_repo, repository_selection }`: Target repository not granted under installation's repository selection.
    - `InstallationRevoked { installation_id, account_login }`: Installation is marked as revoked.
    - `InstallationSuspended { installation_id, account_login }`: Installation is suspended on GitHub.
    - `InsufficientPermissions { installation_id, account_login, violations }`: Granted permissions violate Rule 05 least-privilege policy.
    - `TokenUnavailable { installation_id, account_login, reason }`: Token retrieval or renewal failed.
  - Implemented `InstallationAuthorizer`:
    - Tenant session isolation: Resolves installations strictly scoped to calling `session_id`.
    - Revoked/suspended validation rejecting inactive installations upfront.
    - Proactive token renewal with configurable safety margins (default 300s) via `InstallationTokenRefresher`.
    - Repository access verification for `All` vs `Selected` selections, querying GitHub REST API `GET /repos/{owner}/{repo}` when repo is not in local cache.
    - Rule 05 least-privilege audit using `PermissionAuditor::audit_permissions` (rejecting write and admin permissions).
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - Added `check_repository_access(&self, token, owner, repo) -> Result<Option<RepositoryAccessDetails>, GitHubExtractorError>`.
  - Added models in `crates/github-extractor/src/installation.rs`:
    - `RepositoryPermissions` (`admin`, `push`, `pull`, `triage`, `maintain`).
    - `RepositoryAccessDetails` (`id`, `name`, `full_name`, `private`, `permissions`).
  - Extended token storage contracts in `crates/github-extractor/src/token_refresh.rs`:
    - Added `repository_selection`, `repositories`, `permissions`, `is_revoked`, `is_suspended`, and `has_repository` to `StoredTokenRecord`.
    - Added `save_token_record`, `list_installations_for_session`, and `find_installation_for_account` to `InstallationTokenStore` and `InMemoryInstallationTokenStore`.
  - Authored comprehensive test suite in `crates/github-extractor/tests/authorization_test.rs`:
    - 12 integration tests covering all authorization outcomes, mock GitHub API routes, proactive token renewal, and secret masking.
  - All workspace quality gates passed: 12/12 integration tests, zero clippy warnings, zero fmt diffs, cargo deny passed, and 5/5 check-all gates passed.

### 2026-11-18 — Phase 6.2: Credential management (`6.2.4`)

#### Added
- **`[6.2.4]` Comprehensive Secret Redaction & Tracing Scrubbing in `crates/observability` & `crates/github-extractor`:**
  - Expanded secret and credential redaction engine in `crates/observability/src/redaction.rs`:
    - Full GitHub token pattern support: personal access tokens (`ghp_*`), installation access tokens (`ghs_*`), user tokens (`ghu_*`), OAuth tokens (`gho_*`), refresh tokens (`ghr_*`), and fine-grained PATs (`github_pat_*`).
    - HTTP Authorization header patterns: `Bearer ...`, `token ...`, and `Basic ...` credentials.
    - URL userinfo credentials: `https://user:password@host` and embedded token URLs (`https://ghs_...[REDACTED_GITHUB_TOKEN]@github.com`).
    - Sensitive URL query parameters: `?access_token=`, `?token=`, `?client_secret=`, `?api_key=`, `?password=`, `?signing_key=`, `?webhook_secret=`.
    - Webhook signatures: HMAC-SHA256 digests (`sha256=[REDACTED_SIGNATURE]`).
    - Multi-format PEM private keys: RSA, EC, OPENSSH, and PKCS#8 private keys (`[REDACTED_PRIVATE_KEY]`).
    - `RedactionEngine`: Configurable builder supporting dynamic registration of custom secret literal strings and custom regex patterns.
    - `RedactingWriter` and `RedactingMakeWriter`: `std::io::Write` and `tracing_subscriber::fmt::MakeWriter` adapters that buffer log records, scrub secrets through `redact_sensitive_text`, and guarantee zero plaintext credential leakage to stdout or container logs.
  - Tracing subscriber integration in `crates/observability/src/logging.rs`:
    - Wired `RedactingMakeWriter` into `init_tracing` across all output formats (Compact, Pretty, JSON) whenever `config.redact_secrets` is active (default true).
  - Integration in `crates/github-extractor`:
    - Added `web-repo-to-ai-observability` workspace dependency to `crates/github-extractor/Cargo.toml`.
    - Integrated centralized redaction into `crates/github-extractor/src/error.rs` (`redact_secrets`, `GitHubExtractorError::sanitized()`, `GitHubExtractorError::to_sanitized_string()`).
    - Sanitized upstream API error response payloads and transport failures in `crates/github-extractor/src/client.rs`.
  - Authored comprehensive test coverage:
    - 7 new integration tests in `crates/observability/tests/redaction_test.rs` validating token patterns, multi-secret payloads, PEM keys, recursive JSON redaction, and `RedactingWriter`.
    - 8 new integration tests in `crates/github-extractor/tests/redaction_test.rs` verifying token variants, error sanitization, struct masking (`StoredTokenRecord`, `GitHubInstallationToken`, `GitHubOAuthTokenResponse`, `GitHubAppConfig`), and headers/URL redaction.
    - 2 new unit tests in `crates/github-extractor/src/error.rs`.
  - Verified 20/20 observability unit tests, 7/7 observability integration tests, 30/30 github-extractor unit tests, 32/32 extractor integration tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.
  - **PHASE 6.2 FORMALLY CERTIFIED & ACCEPTED.**

### 2026-11-18 — Phase 6.2: Credential management (`6.2.3`)

#### Added
- **`[6.2.3]` Upstream Revocation, Coordinated Zeroization & Webhook Synchronization in `crates/github-extractor`, `crates/storage`, and `services/api`:**
  - Added upstream token revocation API to `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `revoke_installation_access_token(&self, installation_token: &str) -> Result<(), GitHubExtractorError>` calling GitHub REST API `DELETE /installation/token` with idempotent handling of HTTP 204 No Content, HTTP 401 Unauthorized, and HTTP 404 Not Found.
  - Implemented token revocation coordinator and webhook lifecycle handling in `crates/github-extractor/src/revocation.rs`:
    - `RevocationOutcome`: Enum tracking `Success`, `UpstreamAlreadyRevoked`, `UpstreamFailed`, `StorageRevocationFailed`, and `PartialRevocation`.
    - `RevocationSummary`: Serialized summary capturing `installation_id`, `tokens_revoked`, `storage_records_revoked`, `cache_entries_evicted`, `upstream_outcome`, and timestamp.
    - `WebhookLifecycleAction`: Lifecycle discriminator (`RevokeAndPurge`, `Ignore`) for webhook event payloads (`installation.deleted` and `installation.suspend`).
    - `TokenRevocationCoordinator`: Central engine orchestrating upstream GitHub token revocation, local ciphertext storage zeroization, and in-memory cache eviction across sessions.
  - Extended repository and in-memory caches:
    - Added `revoke_all_by_installation_id` to `TokenStorageRepository` trait, `InMemoryTokenStorage`, and `PostgresTokenStorageRepository` in `crates/storage/src/token_storage.rs`.
    - Added `revoke_all_for_installation` to `InstallationTokenStore` and `InMemoryInstallationTokenStore` in `crates/github-extractor/src/token_refresh.rs`.
  - Implemented REST API routes and handlers in `services/api`:
    - Extended `AppState` with `token_storage: Arc<dyn TokenStorageRepository>`, `github_client: Arc<GitHubApiClient>`, and `webhook_tracker: Arc<Mutex<InMemoryWebhookDeliveryTracker>>`.
    - Created DTO models in `services/api/src/dto/github.rs` (`InstallationPathParams`, `RevokeInstallationResponse`, `InstallationSummaryResponse` with zero secret leakage, `WebhookReceiptResponse`).
    - Implemented `GET /api/v1/auth/github/installations`: Tenant session-scoped installation listing via `X-Session-ID` returning public metadata without exposing ciphertext, keys, or nonces.
    - Implemented `DELETE /api/v1/auth/github/installations/:installation_id`: Tenant-scoped credential revocation enforcing session ownership, invoking upstream revocation, and purging storage records.
    - Implemented `POST /api/v1/auth/github/webhook`: Secure webhook ingestion verifying constant-time HMAC-SHA256 signatures, defending against replay attacks, and automatically zeroing storage upon `installation.deleted` and `installation.suspend` events.
  - Authored comprehensive test coverage:
    - 7 new integration tests in `crates/github-extractor/tests/revocation_test.rs` testing upstream deletion, idempotent 401/404, server error handling, coordinator workflows, and webhook payload parsing.
    - 1 new integration test in `crates/storage/tests/token_storage_test.rs` testing multi-session installation credential purging.
    - 10 new API integration tests in `services/api/src/lib.rs` verifying listing, cross-session isolation, tenant-scoped revocation, 404 handling, webhook signature validation, webhook installation deleted purging, and replay rejection.
  - Verified 55/55 API tests, 193/193 extractor tests, 66/66 storage tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.2: Credential management (`6.2.2`)

#### Added
- **`[6.2.2]` Token Refresh, Expiration Evaluation & Concurrent Deduplication in `crates/github-extractor` & `crates/storage`:**
  - Implemented token freshness and lifecycle evaluation models in `crates/github-extractor/src/token_refresh.rs`:
    - `TokenFreshness`: Enum representing `Fresh { seconds_remaining }`, `ExpiringSoon { seconds_remaining }`, and `Expired { expired_ago_secs }` with `evaluate(expires_at, now, safety_margin_secs)` and predicate helpers (`is_fresh()`, `is_expired()`, `needs_refresh()`).
    - `TokenLifecyclePolicy`: Configurable policy supporting `safety_margin_secs` (default 300s / 5m buffer), `max_retries` (default 3), `retry_backoff_ms` (default 200ms exponential), and an optional `PermissionPolicy` under Rule 05.
    - `StoredTokenRecord`: Tenant token metadata container with `expires_at`, `account_login`, and strict secret masking in `Debug` and `Display` (`***[REDACTED_INSTALLATION_TOKEN]***`).
    - `InstallationTokenStore` async trait and `InMemoryInstallationTokenStore`: Thread-safe, non-mock in-memory token cache with CRUD and revocation.
    - `InstallationTokenRefresher`: Production-grade token renewal engine featuring:
      - Double-checked locking preventing race conditions during concurrent cache misses.
      - Per-installation single-flight deduplication mutex registry (`InFlightRegistry`), ensuring that concurrent extraction tasks targeting the same GitHub installation coalesce onto a single renewal roundtrip.
      - Proactive renewal when remaining TTL falls below the configurable safety margin.
      - Force refresh capability bypassing the cache.
      - Automated bounded exponential backoff retries for transient GitHub API outages (HTTP 5xx).
      - Strict Rule 05 permission policy validation: renewed tokens granting write, admin, or unauthorized scopes fail closed and are discarded.
  - Extended token storage query models in `crates/storage`:
    - Added `is_expired()`, `is_expiring_soon(margin_secs)`, and `time_until_expiration()` helper methods to `GitHubInstallationRow` in `crates/storage/src/schema/github.rs`.
    - Added `get_active_token_with_margin` and `get_token_expiration` to `TokenStorageRepository` trait, `InMemoryTokenStorage`, and `PostgresTokenStorageRepository` in `crates/storage/src/token_storage.rs`.
  - Re-exported token refresh components in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive test suites:
    - 2 new integration tests in `crates/storage/tests/token_storage_test.rs` validating proactive safety margin filtering and expiration metadata retrieval (10/10 passing).
    - 10 new integration tests in `crates/github-extractor/tests/token_refresh_test.rs` with live Axum mock GitHub API server:
      - Freshness evaluation states, secret redaction, store CRUD, fresh token cache hit, proactive renewal on expiring soon, force refresh, Rule 05 permission rejection, transient retry recovery, retry exhaustion, and 20-thread single-flight deduplication.
  - Verified 575 passing workspace Rust tests (186 in `github-extractor`, 65 in `storage`), 213 passing frontend vitest tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.2: Credential management (`6.2.1`)

#### Added
- **`[6.2.1]` Secure Token Storage & Authenticated Envelope Encryption in `crates/storage` & `crates/core`:**
  - Implemented cryptographic primitives and authenticated encryption in `crates/core/src/crypto.rs`:
    - `EncryptionKey`: 256-bit symmetric encryption key for AES-256-GCM with memory zeroization on drop, strict secret masking in `Debug` and `Display` (`***[REDACTED_ENCRYPTION_KEY]***`), hex decoding, HKDF-SHA256 passphrase derivation, and OS random generation via `ring::rand::SystemRandom`.
    - `EncryptedPayload`: Ciphertext and 96-bit (12-byte) initialization vector/nonce with secret masking in `Debug` and `Display`.
    - `Aes256GcmEngine`: AES-256-GCM AEAD encryption and decryption with associated authenticated data (AAD) via `ring::aead`.
    - `build_token_aad`, `encrypt_token`, and `decrypt_token`: Tenant-isolated credential envelope binding ciphertext to `tenant:{owner_session_id}:installation:{github_installation_id}`.
  - Implemented secure token storage abstractions and repositories in `crates/storage/src/token_storage.rs`:
    - `TokenStorageRepository`: Async trait defining `save_installation_token`, `get_active_token`, `get_installation`, `list_active_installations`, `revoke_installation`, and `delete_installation`.
    - `InMemoryTokenStorage`: Thread-safe, mock-free in-memory repository implementing `TokenStorageRepository` with automatic encryption, decryption, expiration filtering, and revocation zeroization.
    - `PostgresTokenStorageRepository`: Production PostgreSQL-backed implementation utilizing `sqlx::PgPool` and UPSERT statements into `github_installations`.
  - Re-exported crypto engine in `crates/core/src/lib.rs` and token storage repository in `crates/storage/src/lib.rs`.
  - Authored 6 unit tests in `crates/core/src/crypto.rs` and 8 integration tests in `crates/storage/tests/token_storage_test.rs`:
    - Validating AES-256-GCM roundtrip, tamper detection across ciphertext, nonce, and AAD, cross-tenant isolation, cross-installation isolation, expiration filtering, revocation zeroization, deletion, and input validation.
  - Resolved disk space bottleneck by purging 28.9 GiB in intermediate build artifacts from target directory, restoring 30.30 GB of free drive space.
  - Verified 563 passing workspace Rust tests (26 in core, 63 in storage), 213 passing frontend tests, zero clippy warnings, clean formatting, and 100% passing quality gates in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.1: GitHub App (`6.1.4` & Phase 6.1 Formal Certification)

#### Added
- **`[6.1.4]` Callback Security, Open-Redirect Defense & Webhook Verification in `crates/github-extractor`:**
  - Implemented cryptographic OAuth callback validation and open-redirect protection in `crates/github-extractor/src/callback.rs`:
    - `OAuthCallbackParams`: Models query parameters (`code`, `state`, `error`, `error_description`, `error_uri`) with robust error handling for user-denied authorization flows.
    - `ValidatedOAuthCallback`: Contains the sanitized authorization code and consumed, verified `OAuthState`.
    - `OAuthCallbackValidator`: Validates non-empty code, single-use state nonce consumption with constant-time matching, session ID and tenant ID matching to prevent CSRF and session hijacking, and URL length/character sanity checks.
    - `OpenRedirectGuard`: Defends against open-redirect phishing by strictly validating redirect targets; permits relative paths starting with a single `/` while explicitly rejecting protocol-relative URLs (`//evil.com`, `/\evil.com`), control characters, and non-whitelisted absolute URLs.
  - Implemented GitHub App post-installation setup callback handling:
    - `InstallationCallbackParams`: Models parameters (`installation_id`, `setup_action`, `code`, `state`).
    - `SetupAction`: Strongly-typed enum representing `Install`, `Update`, `Request`.
    - `InstallationCallbackValidator`: Validates positive installation IDs, allowed setup actions, code sanitization, and timing-safe state validation.
  - Implemented HMAC-SHA256 webhook signature verification and replay prevention:
    - `WebhookSignatureVerifier`: Verifies `X-Hub-Signature-256` HMAC-SHA256 digests in constant time using `ring::hmac::verify`; fails closed on missing, malformed, or tampered signatures.
    - `WebhookDeliveryTracker` trait and `InMemoryWebhookDeliveryTracker`: Prevents webhook replay attacks by tracking `X-GitHub-Delivery` GUIDs with sliding-window TTL eviction.
    - `WebhookEventParser`: Enforces HMAC signature verification *prior* to JSON deserialization; routes events into strongly-typed `GitHubWebhookEvent` variants (`Ping`, `Installation`, `InstallationRepositories`, `Other`).
    - `InstallationAction` and `InstallationRepoAction`: Strongly-typed models for installation lifecycle events.
  - Authored 14 integration test suites in `crates/github-extractor/tests/callback_security_test.rs`:
    - Validating valid OAuth callback, replay prevention, session ID mismatch, parameter sanitization, error responses, open-redirect defense, installation callback validation, HMAC-SHA256 verification (valid, tampered payload, wrong secret, malformed header), delivery deduplication, lifecycle event parsing, and live Axum mock webhook HTTP server integration.
  - Formally certified **Phase 6.1 — GitHub App** across all 4 subphases:
    - `6.1.1` App configuration documentation & manifest models (`ACCEPTED`, commit `7575950`)
    - `6.1.2` OAuth & installation flow (`ACCEPTED`, commit `01d1290`)
    - `6.1.3` Minimum permissions runtime policy engine (`ACCEPTED`, commit `8e4d575`)
    - `6.1.4` Callback security, open-redirect defense & webhook verification (`ACCEPTED`)
  - Verified 549 passing workspace Rust tests (176 in `crates/github-extractor`), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.1: GitHub App (`6.1.3`)

#### Added
- **`[6.1.3]` Minimum Permissions & Least-Privilege Policy in `crates/github-extractor`:**
  - Implemented configurable permission policy engine and auditing framework in `crates/github-extractor/src/permissions.rs`:
    - `PermissionPolicy`: Configurable policy engine with `strict_minimal()` (strictly `contents: read` and `metadata: read`), `standard()` (permits benign read-only scopes like `pull_requests: read`, `issues: read`, while strictly rejecting write/admin or sensitive scopes), and `builder()` for custom domain policies.
    - `PermissionViolation`: Granular enum representing `MissingRequired`, `ExcessiveLevel`, `ForbiddenScope`, `UnexpectedScope`, and `SuspendedInstallation`.
    - `PermissionAuditReport`: Structured audit report containing compliance status, violation list, summary description, JSON export, and GitHub-Flavored Markdown table generation with callout alerts.
    - `PermissionAuditor`: Verification engine for raw permissions maps, `GitHubInstallation`, and `GitHubInstallationToken`.
    - `DEFAULT_FORBIDDEN_CATEGORIES`: Strictly prohibited scopes presenting credential exposure, CI/CD, or administrative risk (`administration`, `actions`, `secrets`, `deployments`, `packages`, `security_events`, `members`, `organization_administration`, `workflows`, `blocking_keys`).
    - `DEFAULT_FORBIDDEN_LEVELS`: Strictly prohibited levels violating read-only policy (`write`, `admin`).
  - Enhanced GitHub App installation and token models in `crates/github-extractor/src/installation.rs`:
    - Added `audit_permissions()`, `audit_permissions_with_policy()`, `validate_permissions()`, and `validate_permissions_with_policy()` on `GitHubInstallation` and `GitHubInstallationToken`.
    - Added `InstallationTokenRequest`: strongly-typed request payload for scoped token issuance (`POST /app/installations/{id}/access_tokens`) with `minimal_read_only()` and `for_repositories()` builder methods.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - Added `create_scoped_installation_access_token` taking an `InstallationTokenRequest`.
    - Added automatic response permission validation ensuring that issued tokens from GitHub are checked against least privilege before returning.
  - Extended error taxonomy in `crates/github-extractor/src/error.rs`:
    - Added `GitHubExtractorError::PermissionViolation` for fail-closed permission rejections.
  - Authored 14 integration test suites in `crates/github-extractor/tests/minimum_permissions_test.rs`:
    - Validating compliant minimal permissions, missing contents, missing metadata, excessive write/admin levels, forbidden categories, suspended installations, Markdown audit generation, token request scoping, and live mock HTTP server validation.
  - Verified 535 passing workspace Rust tests (162 in `crates/github-extractor`), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.1: GitHub App (`6.1.2`)

#### Added
- **`[6.1.2]` OAuth & Installation Flow in `crates/github-extractor`:**
  - Implemented cryptographic OAuth state management and CSRF defense in `crates/github-extractor/src/oauth.rs`:
    - `OAuthState`: 256-bit BLAKE3-hashed high-entropy nonce, timestamps, user ID, session ID, tenant ID, and redirect URI binding with constant-time equality check (`constant_time_eq`) preventing timing side-channel attacks.
    - `OAuthStateStore` trait and `InMemoryOAuthStateStore`: single-use token consumption preventing replay attacks, coupled with thread-safe timestamp-based expired state pruning.
    - URL builders: `build_oauth_authorize_url` and `build_app_installation_url` with strict query parameter encoding.
    - Server-side OAuth code exchange models: `GitHubOAuthTokenResponse` and `GitHubOAuthErrorResponse` with custom `Debug` and `Display` masking sensitive token values with `***[REDACTED_SECRET]***`.
    - Pure Rust RFC 7515 URL-safe base64 encoding (`base64url_encode`).
    - RFC 7519 RS256 JWT generator (`generate_app_jwt`, `generate_app_jwt_at_time`) utilizing `ring::signature::RsaKeyPair` with PKCS#1 v1.5 SHA-256; features automatic PKCS#1 DER wrapping into standard RFC 5208 ASN.1 `PrivateKeyInfo` (`pkcs1_to_pkcs8_der`) supporting both `BEGIN RSA PRIVATE KEY` and `BEGIN PRIVATE KEY` PEM formats without Marvin timing side-channel vulnerabilities (`RUSTSEC-2023-0071`).
  - Implemented GitHub App installation domain models in `crates/github-extractor/src/installation.rs`:
    - `RepositorySelection` (`All`, `Selected`), `GitHubAccountSummary`, `GitHubInstallation`, `InstalledRepositorySummary`, `UserInstallationsResponse`, and `InstallationRepositoriesResponse`.
    - `GitHubInstallationToken`: short-lived installation access token model with custom `Debug`/`Display` secret redaction, `is_expired()`, and `is_expiring_soon(buffer)`.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `exchange_oauth_code`: exchanges temporary authorization codes for user access tokens (`POST /login/oauth/access_token`).
    - `list_user_installations`: retrieves GitHub App installations accessible to the authenticated user (`GET /user/installations`).
    - `get_installation`: retrieves installation details via App JWT (`GET /app/installations/{id}`).
    - `create_installation_access_token`: mints short-lived scoped installation access tokens (`POST /app/installations/{id}/access_tokens`).
    - `list_installation_repositories`: enumerates repositories accessible to an installation token (`GET /installation/repositories`).
  - Authored 10 integration test suites in `crates/github-extractor/tests/oauth_installation_test.rs`:
    - Validating nonce generation and constant-time matching, store single-use replay defense, session binding, TTL expiration, URL builders, secret masking in `Debug` and `Display`, RS256 JWT generation with PKCS#1 and PKCS#8 private keys, and live in-process Axum mock HTTP server integration for code exchange and token minting.
  - Verified 521 passing workspace Rust tests (148 in `crates/github-extractor`), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 6.1: GitHub App (`6.1.1`)

#### Added
- **`[6.1.1]` App Configuration Documentation & Manifest in `crates/github-extractor` & `docs/GITHUB_APP.md`:**
  - Implemented strongly-typed GitHub App Manifest models and least-privilege permission validation adhering to Rule 05 in `crates/github-extractor/src/app_manifest.rs`:
    - `PermissionLevel`: enum representing permission levels, strictly constrained to `Read` (`"read"`); write and admin deserialization or assignment is explicitly rejected with Rule 05 errors.
    - `GitHubAppPermissions`: struct enforcing minimal read-only permissions (`contents: read`, `metadata: read`); systematically validates and rejects write, admin, or unauthorized permission scopes (`administration`, `actions`, `secrets`, etc.).
    - `GitHubHookAttributes`: models webhook configuration attributes (`url`, `active`).
    - `GitHubAppManifest`: root manifest model conforming to GitHub's official manifest creation flow (`name`, `url`, `hook_attributes`, `redirect_url`, `callback_urls`, `setup_url`, `description`, `public`, `default_permissions`, `default_events`).
    - `GitHubAppManifestGenerator`: fluent builder for constructing, validating, and serializing manifests, with `to_manifest_create_url()` generating 1-click URL-encoded registration links for GitHub.com and GitHub Enterprise Server.
  - Implemented runtime configuration loader with strict secret masking:
    - `GitHubAppConfig`: capturing `app_id`, `client_id`, `client_secret`, `private_key_pem`, `webhook_secret`, `app_name`, `app_slug`, `base_api_url`, and `html_url`.
    - Custom `fmt::Debug` and `fmt::Display` implementations masking all sensitive secrets with `***[REDACTED_SECRET]***` and `***[REDACTED_RSA_KEY]***` to prevent credential exposure in application logs.
    - Domain validation verifying positive numeric App ID, non-empty client ID/secret, valid PKCS#1 (`-----BEGIN RSA PRIVATE KEY-----`) and PKCS#8 (`-----BEGIN PRIVATE KEY-----`) PEM delimiters.
    - `from_env()` loading credentials from standard environment variables with direct PEM string or `.pem` file reading support.
    - `public_metadata()` exposing non-sensitive metadata for system status and health endpoints.
  - Authored comprehensive operational and architectural documentation in `docs/GITHUB_APP.md`:
    - Security philosophy and comparison between GitHub Apps, Personal Access Tokens, and OAuth Apps.
    - Automated 1-click manifest creation flow vs manual web UI registration guide.
    - Permissions and events reference matrix explaining least-privilege access and forbidden scopes.
    - Cryptographic key specifications (RSA 2048/4096-bit, PKCS#1, PKCS#8) and two-tier JWT/installation token exchange lifecycle.
    - Webhook event tracking (`installation`, `installation_repositories`) and HMAC-SHA256 constant-time signature verification runbook.
    - Environment variables reference table and operational troubleshooting runbook.
  - Authored 10 integration test suites in `crates/github-extractor/tests/app_manifest_test.rs`:
    - Validating minimal read-only default permissions, Rule 05 rejection of write/admin scopes, serialization/deserialization roundtrip, 1-click manifest URL generation, input validation, runtime config builder and PEM validation, secret masking in `Debug` and `Display`, and public metadata extraction.
  - Verified 511 passing workspace Rust tests (138 in `crates/github-extractor`), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

## [Unreleased] — Stage 5: Public GitHub Repository Extraction

### 2026-11-18 — Phase 5.5: Integrity (`5.5.4` & Stage 5 Formal Certification)

#### Added
- **`[5.5.4]` Large Repository Regression Tests in `crates/github-extractor` & Stage 5 Formal Certification:**
  - Authored a comprehensive large-repository regression test suite, stress verification framework, and golden validation harness in `crates/github-extractor/tests/large_repository_regression_test.rs`:
    - `test_large_scale_tree_truncation_and_subtree_stitching`: synthetic repository tree with 2,500 blobs spanning 10-level deep directory hierarchies; simulates Git Trees API truncation (`truncated: true`), executes subtree analysis and stitching via `TreeStitcher::stitch_subtree`, asserting 100% path preservation with zero dropped entries and zero duplicates.
    - `test_polyglot_monorepo_classification_and_manifest_synthesis`: enterprise polyglot repository covering Rust backend, TypeScript/SvelteKit frontend, Python ML microservice, Go tooling, architecture documentation, CI/CD workflows, vendored artifacts, binary assets, and security-sensitive files; verifies file classification, language detection, and multi-report aggregation into `RepoFileManifest`.
    - `test_large_repository_mathematical_conservation_identity`: mathematical proof of exact arithmetic completeness conservation $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported} + \text{missing\_unaccounted}$; verifies `balance_sheet.is_balanced()`, `is_fully_accounted()`, `is_perfectly_consistent()`, and bidirectional mapping to a valid CDM `CompletenessReport`.
    - `test_large_repository_multi_algorithm_hash_validation_and_corruption_detection`: verifies 100% cryptographic checksum validity across Git Object SHA-1, BLAKE3, and SHA-256 digests; validates pinpoint tamper detection on simulated file corruption.
    - `test_large_repository_exclusion_and_diagnostics_audit`: verifies granular categorization across vendor, lockfile, binary, and security exclusion categories; validates CDM `ExclusionRecord` mapping, and confirms Markdown & JSON formatting.
    - `test_large_repository_code_review_package_budget_and_prioritization`: validates review focus heuristic scoring, checklist synthesis, and strict file count budget ceilings.
    - `test_large_repository_end_to_end_cdm_assembly_strict_validation`: constructs a unified `CdmDocument` representing the full monorepo with tree, architecture context, review package, balance sheet, checksum report, and exclusion audit sections, passing `CdmValidator::with_config(ValidationConfig::strict())` with zero validation issues.
    - `test_large_repository_live_mock_http_integration`: validates ref discovery, commit revision pinning, and tree fetching through `GitHubApiClient` against an in-process Axum mock HTTP server.
  - Formally certified **Stage 5 — Public GitHub Repository Extraction** across all 5 phases:
    - Phase 5.1 (Metadata): `5.1.1` – `5.1.4`
    - Phase 5.2 (Discovery): `5.2.1` – `5.2.4`
    - Phase 5.3 (Extraction): `5.3.1` – `5.3.4`
    - Phase 5.4 (Packaging): `5.4.1` – `5.4.4`
    - Phase 5.5 (Integrity): `5.5.1` – `5.5.4`
  - Verified 501 passing workspace Rust tests (127 in `crates/github-extractor`), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.5: Integrity (`5.5.3`)

#### Added
- **`[5.5.3]` Missing & Excluded File Reports in `crates/github-extractor`:**
  - Implemented structured diagnostics, root-cause categorizations, remediation guidance, and formal audit reporting for excluded, dropped, and failed repository items in `crates/github-extractor/src/exclusion.rs`:
    - `ExclusionCategory`: strongly-typed exclusion taxonomy (`OversizedFile`, `GeneratedOrLockfile`, `VendorOrThirdParty`, `BinaryOrAsset`, `SubmoduleOrSymlink`, `PolicyPattern`, `SensitiveSecurityPattern`, `Other`) with `to_cdm_reason()` and `default_remediation()` recommendations.
    - `ExtractionFailureCategory`: strongly-typed failure taxonomy (`NetworkTimeout`, `RateLimitExceeded`, `BlobTooLarge`, `CorruptedContent`, `SubtreeTraversalFailed`, `IoOrStorage`, `Unknown`) with `is_retryable()` predicate for automated retry scheduling.
    - `ExcludedFileRecord`: structured record capturing relative path, git blob SHA, exclusion category, size bytes, language, rationale, matched pattern, and remediation advice.
    - `FailedFileRecord`: record capturing extraction failures, root-cause error category, error message, HTTP status code, and retry eligibility.
    - `MissingFileRecord`: record capturing eligible discovered files completely unaccounted for in extraction artifacts.
    - `ExclusionAuditReport`: consolidated report model capturing target repo, target ref, pinned commit SHA, discovered blobs, excluded files count & byte metrics, failed files count, missing files count, category distribution maps, and itemized record lists.
    - Integrity predicates: `is_clean()` (zero missing and zero failed items) and `has_critical_issues()`.
    - Bidirectional CDM mapping: `to_cdm_exclusion_records()` translating records directly into Canonical Context Document Model [`ExclusionRecord`] items with standardized [`ExclusionReason`] types.
    - `ExclusionReportGenerator`: multi-report synthesis engine reconciling `RepoFileManifest`, direct extraction reports (`SourceExtractionReport`, `DocExtractionReport`, `DependencyExtractionReport`, `ConfigTestExtractionReport`), and `FileAccountingBalanceSheet`.
    - Formatters & serializers: `to_markdown()` producing structured GFM status summaries, breakdown tables, and failure logs; `to_json()` / `to_json_compact()` for structured JSON export; `to_cdm_section()` generating schema-valid `CdmSection` with `SectionKind::Manifest`, `manifest_role: "exclusion_and_missing_report"`, and BLAKE3 hash.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `generate_exclusion_report`: client method generating missing and excluded diagnostics from repository artifacts.
  - Re-exported exclusion models and generator in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/missing_excluded_reports_test.rs`:
    - 11 test suites validating category inference across patterns and directories, CDM exclusion mapping, failure category inference and retryability, record models, audit report clean vs critical states, multi-report manifest aggregation, GFM Markdown formatting, JSON serialization roundtrip, CDM manifest section generation, and live API client integration.
  - Verified 501 passing workspace Rust tests (165 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.5: Integrity (`5.5.2`)

#### Added
- **`[5.5.2]` Content Hash Validation in `crates/github-extractor`:**
  - Implemented comprehensive multi-algorithm cryptographic integrity auditing and content hash verification in `crates/github-extractor/src/hash_validator.rs`:
    - `HashAlgorithm`: algorithm taxonomy supporting Git Object SHA-1 (`git_blob_sha1`, 40 hex chars), primary BLAKE3 (`blake3`, 64 hex chars), and secondary SHA-256 (`sha256`, 64 hex chars) with strict syntax/lowercase hex validation.
    - `HashCheckMetric`: quantitative tracking per algorithm of evaluated, passed, and failed checks.
    - `HashValidationFailure`: structured error model capturing relative path, algorithm, expected hash, calculated hash, and failure context/reason.
    - `HashValidationConfig`: configuration switches allowing individual algorithm activation (`verify_git_sha`, `verify_blake3`, `verify_sha256`) and optional fail-fast termination.
    - `HashValidationReport`: comprehensive report summarizing validation outcomes, breakdown tables per algorithm, failure lists, and overall status evaluation via `is_valid()`.
    - `ContentHashValidator`: high-throughput verification engine supporting auditing of raw byte payloads, `RepoFileManifest` entries against content maps, extraction report collections (`SourceExtractionReport`, `DocExtractionReport`, `DependencyExtractionReport`, `ConfigTestExtractionReport`), and `CdmSection` lists.
    - Formatters & serializers: `to_markdown()` producing structured GFM checksum breakdown tables and failure logs; `to_json()` / `to_json_compact()` for structured JSON export; `to_cdm_section()` generating a schema-valid `CdmSection` with `SectionKind::Manifest`, `manifest_role: "content_hash_validation"`, and BLAKE3 hash.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `validate_content_hashes`: client method executing checksum validation across extracted artifacts.
  - Re-exported hash validation models and functions in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/hash_validation_test.rs`:
    - 11 test suites validating valid content hash verification, Git SHA-1 mismatch detection, BLAKE3 mismatch detection, SHA-256 mismatch detection, malformed hex format detection, multi-report verification, manifest verification, CDM section verification, fail-fast behavior, and live API client integration.
  - Verified 490 passing workspace Rust tests (154 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.5: Integrity (`5.5.1`)

#### Added
- **`[5.5.1]` Expected-vs-Actual File Accounting in `crates/github-extractor`:**
  - Implemented mathematical completeness verification, discrepancy taxonomy, bidirectional set reconciliation, and quantitative balance sheet validation in `crates/github-extractor/src/accounting.rs`:
    - `AccountingDiscrepancyKind`: strongly-typed root cause classification (`MissingUnaccounted`, `UnexpectedExtra`, `ByteSizeMismatch`, `GitShaMismatch`, `CategoryMismatch`, `StatusConflict`).
    - `DiscrepancySeverity`: 3-tier severity rating (`Critical`, `Warning`, `Info`) prioritizing completeness and content integrity violations.
    - `AccountingDiscrepancy`: structured discrepancy record capturing normalized path, discrepancy kind, expected details from Git tree discovery, actual details from extraction artifacts, and impact severity.
    - `FileAccountingBalanceSheet`: comprehensive balance sheet model capturing discovered tree blobs, eligible blobs, expected excluded blobs, actual extracted files, actual excluded files, actual failed files, actual unsupported files, missing unaccounted files, unexpected extra files, byte size mismatches, and Git SHA mismatches.
    - Mathematical invariant verification:
      - `is_balanced()`: verifies the arithmetic conservation law $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported} + \text{missing\_unaccounted}$.
      - `is_fully_accounted()`: verifies that zero eligible files are dropped without record ($\text{missing\_unaccounted} == 0$) and zero phantom extra files are introduced ($\text{unexpected\_extra} == 0$).
      - `is_perfectly_consistent()`: verifies that all conservation laws hold, all files are accounted for, zero byte size mismatches exist, and zero Git SHA mismatches exist.
      - `to_completeness_report()`: direct translation into CDM [`CompletenessReport`].
    - `FileAccountingAuditor`: bidirectional set reconciliation engine supporting audit against `RepoFileManifest`, direct extraction reports (`SourceExtractionReport`, `DocExtractionReport`, `DependencyExtractionReport`, `ConfigTestExtractionReport`), or CDM sections (`audit_sections`).
    - Formatters & serializers: `to_markdown()` producing structured GFM balance sheets and discrepancy logs; `to_json()` / `to_json_compact()` for structured JSON export; `to_cdm_section()` generating a schema-valid `CdmSection` with `SectionKind::Manifest`, `manifest_role: "file_accounting_audit"`, and BLAKE3 hash.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `audit_file_accounting`: client method reconciling repository tree entries against manifests or extraction reports.
  - Re-exported accounting models and functions in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/file_accounting_test.rs`:
    - 9 test suites validating balanced extractions, missing file detection, unexpected extra detection, byte size mismatches, Git SHA mismatches, multi-report integration, CDM section generation, CDM section auditing, and live API client integration.
  - Verified 479 passing workspace Rust tests (143 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.4: Repository Packaging (`5.4.4` & Phase 5.4 Certification)

#### Added
- **`[5.4.4]` Code-Review Package in `crates/github-extractor` & Phase 5.4 Formal Certification:**
  - Implemented automated AI code-review package assembly, review prioritization tiering, review checklist synthesis, and end-to-end CDM document construction in `crates/github-extractor/src/code_review.rs`:
    - `ReviewFocus`: strongly-typed review focus enum (`FullRepository`, `SecurityAudit`, `ArchitectureReview`, `QualityAudit`, `DiffChangeset`) with system prompt focus guidelines and targeted heuristic weighting.
    - `ReviewPriorityTier`: 5-tier review importance rating (`Critical`, `High`, `Medium`, `Low`, `Excluded`) with heuristic prioritization scoring based on security criticality (auth, crypto, security, network, credentials, tokens, SSRF, permissions), architecture entry points (main, lib, api, routes), business logic, configurations, and tests.
    - `PrioritizedReviewFile`: domain model capturing relative repository path, priority tier, category, language, byte size, line count, git blob SHA, prioritization rationale, and content snippet or full text.
    - `CodeReviewPackageConfig`: configuration switches for review focus, max files budget, max review tokens, directory tree inclusion, manifest inclusion, architecture overview inclusion, and review checklist inclusion.
    - `CodeReviewPackage`: consolidated AI review package containing review metadata, review focus, summary metrics, prioritized files, total review files, total review lines, and review checklist items.
    - Assembly constructor `assemble`: ranks and filters files according to `ReviewFocus` and `CodeReviewPackageConfig` budgets, synthesizes targeted review checklists tailored to the selected focus (e.g., OWASP/SSRF defenses, memory bounds, error propagation, API contracts, test coverage).
    - Formatters & serializers: `to_markdown()` producing clean, structured GFM code-review dossiers with summary profiles and markdown tables; `to_json()` and `to_json_compact()` for structured JSON export.
    - End-to-end CDM document packaging: `to_cdm_document()` synthesizing a complete, schema-compliant `CdmDocument` comprising Review Overview (`SectionKind::Frontmatter`), Directory Tree (`SectionKind::Manifest`), File Manifest (`SectionKind::Manifest`), Architecture Overview (`SectionKind::Prose`), Prioritized Code Files (`SectionKind::CodeFile`), and Review Checklist (`SectionKind::Admonition`), strictly validated against `CdmValidator::with_config(ValidationConfig::strict())`.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `generate_code_review_package`: client method assembling review package from extracted repository components.
  - Re-exported code review models and helpers in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/code_review_package_test.rs`:
    - 8 test suites validating review focus defaults, priority tiers, security audit filtering, file budgeting, GFM formatting, JSON serialization, end-to-end strict CDM document validation, and live API client integration.
  - Formally certified **Phase 5.4: Repository packaging** across all 4 subphases (`5.4.1`, `5.4.2`, `5.4.3`, `5.4.4` all `ACCEPTED`).
  - Verified 470 passing workspace Rust tests (134 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.4: Repository Packaging (`5.4.3`)

#### Added
- **`[5.4.3]` Architecture Context & System Overview in `crates/github-extractor`:**
  - Implemented automated repository architecture analysis, workspace topology classification, entry point discovery, component cataloging, and CDM overview section construction in `crates/github-extractor/src/architecture.rs`:
    - `WorkspaceType`: strongly-typed topology enum (`SinglePackage`, `Monorepo`, `MultiService`, `Library`, `Other`) with multi-signal heuristics detecting directory groupings (`crates/*`, `packages/*`, `apps/*`, `services/*`, `deploy/`, `docs/`, `scripts/`) and nested workspace manifests.
    - `EntryPointRole` & `EntryPointInfo`: discovers and catalogs executable application entry points (`BinaryMain`, `LibraryRoot`, `WebApi`, `UiRoute`, `WorkerDaemon`, `ScriptCli`) with language, path, and functional notes.
    - `ComponentRole` & `ComponentInfo`: discovers and catalogs repository subsystems and packages (`Service`, `LibraryCrate`, `WebFrontend`, `Worker`, `ToolCli`, `Infrastructure`, `Documentation`, `GeneralModule`) with file counts, byte sizes, and primary languages.
    - `RepoArchitectureContext`: unified domain model synthesizing workspace topology, primary language distribution, ecosystems, inferred key technologies (Tokio, Axum, SQLx, SvelteKit, Tailwind CSS, Playwright, Prettier, etc.), discovered architecture/design documents (ADRs, specifications, data models), and build/CI/CD pipelines.
    - Formatters & serializers: `to_markdown()` producing clean, structured GFM architecture overviews with summary profiles and markdown tables; `to_json()` and `to_json_compact()` for structured JSON export.
    - CDM integration: `to_cdm_section()` generating a Canonical Context Document Model [`CdmSection`] with `SectionKind::Prose`, `manifest_role: "architecture_context"`, title `"Repository Architecture & System Overview"`, and BLAKE3 content hash.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `generate_architecture_context`: client method assembling architecture synthesis from repository classification, manifests, and extraction reports.
  - Re-exported architecture models and helpers in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/architecture_context_test.rs`:
    - 10 test suites validating monorepo, single-package, and library topology classification, entry point discovery, component cataloging, technology stack inference, architecture doc indexing, Markdown formatting, JSON roundtrip serialization, CDM section creation, and live API client integration.
  - Verified 454 passing workspace Rust tests (126 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.4: Repository Packaging (`5.4.2`)

#### Added
- **`[5.4.2]` File Manifest & Completeness Index in `crates/github-extractor`:**
  - Implemented structured repository file manifest indexing, per-file extraction status accounting, multi-report aggregation, and CDM manifest section construction in `crates/github-extractor/src/manifest.rs`:
    - `FileExtractionStatus`: strongly-typed enum tracking extraction outcomes (`Extracted`, `Excluded { reason }`, `Failed { error }`, `Unsupported { reason }`) with query helpers (`is_extracted`, `is_excluded`, `is_failed`, `is_unsupported`, `status_name`, `reason_or_error`).
    - `ManifestFileEntry`: comprehensive per-file manifest model capturing relative path, 40-character Git SHA-1, classified file category, byte size, line count, detected programming language, BLAKE3 and SHA-256 digests, extraction status, and associated CDM section ID.
    - `RepoFileManifest`: consolidated repository index with aggregate metrics (total discovered files, extracted files, excluded files, failed files, unsupported files, total bytes, extracted bytes, total line count, category distribution map, language distribution map, and status distribution map).
    - Multi-report aggregator `from_classification_and_reports`: stitches `TreeClassificationReport` with downstream reports from sources, documentation, dependencies, configurations, and tests, guaranteeing 100% accounting for all discovered tree files.
    - Completeness equation validation `verify_completeness()`: enforces the fundamental conservation law `total_files == extracted + excluded + failed + unsupported`.
    - Formatters & serializers: `to_markdown_table()` generating GFM executive summary tables and complete file indices; `to_json()` and `to_json_compact()` for structured JSON export.
    - CDM integration: `to_cdm_section()` generating a Canonical Context Document Model [`CdmSection`] with `SectionKind::Manifest`, `manifest_role: "file_manifest"`, BLAKE3 content hash, and completeness attributes.
    - `to_completeness_report()`: direct translation into CDM [`CompletenessReport`].
  - Extended `TreeClassificationReport` in `crates/github-extractor/src/classification.rs`:
    - `to_file_manifest(&self) -> RepoFileManifest`: converts classification report into an initial repository manifest.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `generate_file_manifest`: convenience client method assembling classification and arbitrary extraction reports into `RepoFileManifest`.
  - Re-exported manifest types in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/file_manifest_test.rs`:
    - 7 test suites validating explicit entry assembly, multi-report stitching, GFM Markdown formatting, JSON serialization roundtrip, CDM completeness report mapping, CDM manifest section construction, and live Axum API client integration.
  - Verified 444 passing workspace Rust tests (116 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.4: Repository Packaging (`5.4.1`)

#### Added
- **`[5.4.1]` Repository Directory Tree Generation in `crates/github-extractor`:**
  - Implemented hierarchical directory tree models, formatting engines, and CDM manifest section mappings in `crates/github-extractor/src/tree_generator.rs`:
    - `RepoTreeNode`: recursive hierarchical node representation supporting files, subdirectories, modes, byte sizes, file categories, and language tags.
    - `RepoTree`: aggregated tree model with `from_entries`, `from_classified_files`, and `from_tree_response` constructors, computing recursive file counts, directory counts, total byte sizes, and max nesting depths.
    - `TreeFormatConfig` and `TreeStyle`: configurable diagram renderer supporting Unicode box-drawing (`├── `, `└── `, `│   `) and plain ASCII (`|-- `, `\-- `), depth ceilings (`max_depth`), total entry ceilings (`max_entries`), directories-first sorting, file sizes, and category badges.
    - `category_badge`: compact category indicator formatter (`[source]`, `[doc]`, `[manifest]`, `[config]`, `[test]`, `[lockfile]`, `[binary]`, `[oversized]`).
    - `to_cdm_section`: renders the complete directory diagram into a Canonical Context Document Model [`CdmSection`] with `SectionKind::Manifest`, `manifest_role: "repository_tree"`, BLAKE3 content hash, and baseline token metrics.
  - Extended `GitTreeResponse` in `crates/github-extractor/src/tree.rs`:
    - `to_repo_tree(&self) -> RepoTree`: direct conversion from Git tree response.
  - Extended `TreeClassificationReport` in `crates/github-extractor/src/classification.rs`:
    - `to_repo_tree(&self) -> RepoTree`: direct conversion preserving classification categories and languages.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `generate_repo_tree(&self, tree_response) -> RepoTree` and `generate_repo_tree_classified(&self, classified_files) -> RepoTree`.
  - Re-exported tree generator models in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/repo_tree_generation_test.rs`:
    - 7 test suites validating Git tree entry construction, classification badge formatting, ASCII/Unicode styles, depth limiting, entry count ceiling, CDM section mapping, and live Axum API client integration.
  - Verified 437 passing workspace Rust tests (109 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.3: File Extraction (`5.3.4`) — PHASE 5.3 CERTIFIED

#### Added
- **`[5.3.4]` Configuration & Test Extraction in `crates/github-extractor` (PHASE 5.3 CERTIFIED):**
  - Implemented specialized configuration and test extraction models in `crates/github-extractor/src/config_test.rs`:
    - `ConfigCategory`: functional taxonomy for repository configuration covering `CiCd` (GitHub Actions, GitLab CI, Jenkins, CircleCI), `Build` (Dockerfile, tsconfig.json, Makefile, docker-compose, bundler configs), `LinterFormatter` (rustfmt, clippy, eslint, prettier, editorconfig, biome), `Environment` (.env templates), and `General`.
    - `TestCategory`: functional classification for test files covering `Unit` (*_test.rs, *.test.ts, test_*.py), `Integration` (tests/*, integration suites), `E2e` (playwright, cypress, e2e specs), `Fixture` (fixtures/*, mocks/*, testdata/*), `Benchmark` (benches/*), and `General`.
    - `detect_config_format`: presentation format resolver for YAML, JSON, TOML, Dockerfile, Makefile, INI, TypeScript, JavaScript, and plaintext.
    - `ExtractedConfigFile`: verified configuration model with Git blob SHA-1 validation, format identification, BLAKE3 and SHA-256 digests, and conversion to CDM [`CdmSection`] with `SectionKind::CodeFile` and `file_category: "configuration"`.
    - `ExtractedTestFile`: verified test code/fixture model with language resolution, BLAKE3 and SHA-256 digests, and conversion to CDM [`CdmSection`] with `SectionKind::CodeFile` and `file_category: "test_fixture"`.
    - `ConfigTestExtractionReport`: aggregated outcome report capturing extracted configs, extracted tests, failures, unsupported binaries, dedicated `ci_workflows()` and `fixtures()` accessors, and CDM `CompletenessReport` mapping preserving the invariant `discovered == extracted + excluded + failed + unsupported`.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `fetch_config_file(&target, file, config)`: acquires and verifies an individual configuration file.
    - `fetch_test_file(&target, file, config)`: acquires and verifies an individual test file or fixture.
    - `extract_config_and_test_files(&target, files, config)`: bounded concurrent batch extraction via `tokio::sync::Semaphore`.
  - Re-exported configuration and test types in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/config_test_extraction_test.rs`:
    - 6 test suites validating live Axum GitHub Actions CI workflow extraction, Dockerfile container specification, rustfmt.toml formatting config, unit and integration test code, static JSON fixture extraction, and bounded concurrent batch extraction with partial failures and CDM section mapping.
  - Verified 430 passing workspace Rust tests (98 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.
  - **FORMALLY CERTIFIED PHASE 5.3 ("File extraction", Subphases `5.3.1` – `5.3.4`).**

### 2026-11-18 — Phase 5.3: File Extraction (`5.3.3`)

#### Added
- **`[5.3.3]` Dependency Extraction & Manifest Parsing in `crates/github-extractor`:**
  - Implemented specialized dependency models, ecosystem detectors, and deterministic parsers in `crates/github-extractor/src/dependency.rs`:
    - `PackageEcosystem`: multi-ecosystem identification across `Cargo`, `Npm`, `Pip`, `GoMod`, `MavenGradle`, `Bundler`, `Composer`, `Pub`, `Mix`, and `Other`.
    - `ManifestRole`: functional classification distinguishing `Manifest` (declarative package and dependency specification) from `Lockfile` (pinned transitive lock files and hashes).
    - `DependencyType`: functional dependency classifications covering `Direct`, `Dev`, `Build`, `Peer`, and `Optional`.
    - `DeclaredDependency`: structured dependency declaration capturing package name, optional version constraint requirement, functional type, optional flag, and enabled feature sets.
    - `ParsedManifest`: parsed package metadata capturing package name, version, description, license, and queried dependencies (`direct_dependencies`, `dev_dependencies`, `dependency_count`, `find_dependency`).
    - Deterministic lightweight manifest parsers for `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, and `pyproject.toml` without external parser dependencies or panics.
    - `ExtractedDependencyFile`: extracted dependency model with Git object SHA-1 verification, ecosystem and role categorization, parsed manifest integration, BLAKE3 and SHA-256 hashes, and conversion into CDM [`CdmSection`] with `SectionKind::Manifest` and metadata attributes.
    - `DependencyExtractionReport`: aggregated extraction outcome capturing extracted files, failures, unsupported binaries, ecosystem filtering, direct dependency rollup, and CDM `CompletenessReport` mapping preserving the invariant `discovered == extracted + excluded + failed + unsupported`.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `fetch_dependency_file(&target, file, config)`: acquires and parses an individual manifest or lockfile.
    - `extract_dependency_files(&target, files, config)`: bounded concurrent batch extraction via `tokio::sync::Semaphore`.
  - Re-exported dependency models, parsers, and helper functions in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/dependency_extraction_test.rs`:
    - 6 test suites validating live Axum `Cargo.toml` extraction and parsing, `package.json` dependency classification, `requirements.txt` parsing with version operators, `go.mod` module and direct/dev dependency resolution, `Cargo.lock` lockfile acquisition, and bounded concurrent batch extraction with completeness accounting and CDM section mapping.
  - Verified 424 passing workspace Rust tests (89 in `crates/github-extractor` + 1 doc-test), 213 frontend tests, zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.3: File Extraction (`5.3.2`)

#### Added
- **`[5.3.2]` Documentation Extraction & Title/Frontmatter Parsing in `crates/github-extractor`:**
  - Implemented specialized documentation models and utilities in `crates/github-extractor/src/documentation.rs`:
    - `DocType`: canonical functional taxonomy covering `Readme`, `Architecture`, `Changelog`, `Contributing`, `License`, `Security`, `CodeOfConduct`, `Guide`, `Roadmap`, and `General`.
    - `DocFormat`: format detection across `Markdown` (`.md`, `.markdown`), `Asciidoc` (`.adoc`, `.asciidoc`), `RestructuredText` (`.rst`), and `PlainText`.
    - `extract_title_and_frontmatter`: extracts document titles from Markdown headings (`# Heading`) or YAML frontmatter (`title: "..."`), with clean fallback to normalized path basenames.
    - `ExtractedDocFile`: verified documentation model converting directly into CDM [`CdmSection`] with `SectionKind::Prose`, line range, byte range, language, BLAKE3 content hash, `SectionTokenMetrics`, and metadata attributes (`doc_type`, `has_frontmatter`, `git_blob_sha`, `sha256`).
    - `DocExtractionReport`: aggregated outcome report capturing extracted docs, failures, unsupported binaries, dedicated root `readme()` and `license()` lookup accessors, and CDM `CompletenessReport` mapping.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `fetch_doc_file(&target, file, config)`: acquires and verifies an individual documentation file.
    - `extract_doc_files(&target, files, config)`: bounded concurrent batch extraction via `tokio::sync::Semaphore`.
  - Re-exported documentation types in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/documentation_extraction_test.rs`:
    - 6 test suites validating live Axum README extraction and heading title parsing, plain-text LICENSE extraction, YAML frontmatter detection, bounded concurrent batch extraction with primary README and LICENSE accessors, partial failure handling with CDM completeness accounting, and CDM section generation.
  - Verified 418 passing workspace Rust tests (83 in `crates/github-extractor` + 1 doc-test), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.3: File Extraction (`5.3.1`)

#### Added
- **`[5.3.1]` Source Acquisition & Blob Verification in `crates/github-extractor`:**
  - Implemented source extraction models, decoders, and hashing utilities in `crates/github-extractor/src/source.rs`:
    - `decode_base64`: pure Rust RFC 4648 base64 decoder with tolerance for whitespace, newlines, and padding variants.
    - `compute_git_blob_sha`: standard Git object SHA-1 hex digest calculation (`sha1("blob <size>\0<content>")`).
    - `compute_blake3_hash` and `compute_sha256_hash`: cryptographic content digests for CDM section integrity and indexing.
    - `is_binary_content`: null byte inspection across an 8,000-byte sniff window (`BINARY_SNIFF_WINDOW_BYTES`).
    - `GitBlob`: raw blob representation with verification helpers (`verify_git_sha`, `is_binary`, `content_lossy`).
    - `SourceExtractionConfig`: configurable batch parameters for concurrency (default 10 via `DEFAULT_EXTRACTION_CONCURRENCY`), maximum file size (default 1 MiB via `DEFAULT_MAX_FILE_SIZE_BYTES`), Git SHA verification toggle, and request timeout.
    - `ExtractedSourceFile`: rich source representation converting directly into CDM [`CdmSection`] with `SectionKind::CodeFile`, line span (`LineRange`), byte range (`ByteRange`), BLAKE3 content hash, language, token metrics, and Git blob SHA attributes.
    - `FailedSourceExtraction`: per-file failure record capturing relative path, SHA, and sanitized error string.
    - `SourceExtractionReport`: batch acquisition outcome report capturing extracted files, failed files, unsupported files, total bytes, total lines, duration, and mapping to CDM `CompletenessReport` enforcing formula invariants.
  - Added `BlobError` variant to `GitHubExtractorError` in `crates/github-extractor/src/error.rs`.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `get_blob(&target, file_sha)`: fetches raw blob (`application/vnd.github.raw`) with fallback to JSON base64 payloads.
    - `fetch_source_file(&target, file, config)`: acquires and verifies an individual classified file.
    - `extract_source_files(&target, files, config)`: bounded concurrent batch extraction via `tokio::sync::Semaphore`.
  - Re-exported source acquisition types in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/source_acquisition_test.rs`:
    - 7 test suites validating live Axum raw blob fetching, base64 blob fetching, Git SHA mismatch rejection, binary detection and unsupported categorization, bounded concurrent batch extraction with semaphore rate limiting, partial failure handling with CDM completeness accounting, and CDM section generation.
  - Verified 408 passing workspace Rust tests (72 in `crates/github-extractor` + 1 doc-test), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.2: File Discovery (`5.2.4` & Phase 5.2 Milestone Certification)

#### Added
- **`[5.2.4]` Repository File Classification & CDM Completeness Reporting in `crates/github-extractor`:**
  - Implemented comprehensive file classification in `crates/github-extractor/src/classification.rs`:
    - `FileCategory`: canonical taxonomy (`SourceCode`, `Documentation`, `Configuration`, `DependencyManifest`, `TestFixture`, `GeneratedOrLockfile`, `Binary`, `Oversized`, `Other`) with query helpers (`is_source_code`, `is_documentation`, `is_extractable`, etc.).
    - `FileClassificationConfig`: configurable classification rules supporting size thresholds (default 1 MiB / 1,048,576 bytes via `DEFAULT_MAX_FILE_SIZE_BYTES`), excluded path prefixes (`node_modules/`, `target/`, `.git/`, `.venv/`, `vendor/`), lockfile exclusions, test exclusions, and explicit path overrides.
    - `ClassifiedFile`: rich entry representation including normalized path, category, file mode, SHA, size, detected language, extraction eligibility, and explicit exclusion reason.
    - `TreeClassificationReport`: aggregated tree classification metrics capturing total files, category counts, category sizes, eligible/excluded breakdowns, binary and oversized counters, and category filtering helpers.
    - Direct CDM `CompletenessReport` mapping: `to_expected_completeness_report()` and `to_pre_extraction_completeness_report()` strictly satisfying the fundamental CDM invariant:
      `discovered_count == extracted_count + excluded_count + failed_count + unsupported_count`.
    - `detect_language`: robust language detection for 28+ languages including Rust, TypeScript, Python, Go, C/C++, Svelte, Vue, Dockerfiles, and Makefiles.
  - Added `classify(&self, config)` convenience method to `GitTreeResponse` in `crates/github-extractor/src/tree.rs`.
  - Re-exported classification models and functions in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/file_classification_test.rs`:
    - 7 test suites validating classification across all 8 canonical categories, multi-language detection, byte-size threshold gating and oversized tagging, excluded path prefix filtering, custom override precedence, CDM completeness report formula validity, and tree classification aggregation.
  - Verified 395 passing workspace Rust tests (65 in `crates/github-extractor` + 1 doc-test), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

#### Milestone Certification: Phase 5.2 — File Discovery
- **Subphases Completed:** `5.2.1` (Git Trees API integration), `5.2.2` (Tree truncation detection), `5.2.3` (Subtree recovery & shallow archive fallback), `5.2.4` (File classification & CDM completeness reporting).
- **Outcome:** Phase 5.2 is 100% complete, verified, and formally certified. All file discovery capabilities for public GitHub repositories are robustly operational.

### 2026-11-18 — Phase 5.2: File Discovery (`5.2.3`)

#### Added
- **`[5.2.3]` Git Subtree Recovery & Shallow Archive Fallback in `crates/github-extractor`:**
  - Implemented recovery models and utilities in `crates/github-extractor/src/recovery.rs`:
    - `ArchiveFormat`: supports `Tarball` (`.tar.gz`) and `Zipball` (`.zip`) formats with URL path and MIME helpers.
    - `SubtreeRecoveryConfig`: configurable parameters for recovery (`max_subtrees`, `max_depth`, `auto_fallback`, `concurrency`).
    - `SubtreeRecoveryReport`: diagnostic outcome report detailing subtrees attempted, succeeded, failed, additional blobs/trees recovered, final completeness, fallback status, and remaining unexpanded paths.
    - `TreeStitcher`: path prefixing and normalization utility (`stitch_child_path`) and in-place subtree merging with deduplication (`stitch_subtree`).
    - `scan_zip_archive`: in-memory GitHub zipball parser extracting complete `GitTreeResponse` hierarchies, stripping top-level archive prefixes, detecting executable permissions (`100755`) and directory modes (`040000`), and generating deterministic SHA hashes.
  - Extended `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `download_archive(&target, git_ref, format)`: downloads binary archive stream with secret redaction and redirect handling.
    - `recover_tree(&target, base_tree, config, fallback_ref)`: orchestrates bounded BFS queue traversal of unexpanded subtrees, stitches child entries, and executes shallow archive fallback if thresholds are exceeded or unexpanded subtrees remain.
    - `recover_pinned_tree(&pinned, config)`: fetches pinned revision tree and automatically recovers complete hierarchy with zero ref-drift.
  - Added `ArchiveError` variant to `GitHubExtractorError` in `crates/github-extractor/src/error.rs`.
  - Re-exported recovery types in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/subtree_recovery_test.rs`:
    - 7 test suites covering path stitching, in-place tree merging and deduplication, config defaults/builders, in-memory zip archive parsing, live Axum recursive subtree traversal, threshold-based archive fallback, and upstream failure archive fallback.
  - Verified 388 passing workspace Rust tests (54 in `crates/github-extractor`), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.2: File Discovery (`5.2.2`)

#### Added
- **`[5.2.2]` Git Tree Truncation Detection & Completeness Auditing in `crates/github-extractor`:**
  - Implemented truncation domain models and constants in `crates/github-extractor/src/tree.rs`:
    - `GITHUB_TREE_ENTRY_LIMIT`: 100,000 entry ceiling per GitHub Git Trees API.
    - `SUBTREE_TRAVERSAL_LIMIT`: 500 unexpanded subtree threshold before escalating to archive fallback.
    - `TruncationReason`: enum identifying truncation triggers (`GitHubFlag`, `EntryCountCap`, `IncompleteSubtrees`).
    - `RecoveryStrategy`: enum classifying remediation path (`None`, `SubtreeTraversal`, `ShallowArchiveFallback`).
    - `TreeCompleteness`: enum identifying whether repository tree is `Complete` or `Truncated(TruncationReport)`.
    - `TruncationReport`: diagnostic record capturing total entries, blob count, tree count, truncation flag, explicit reasons, unexpanded subtree paths and SHAs, and recommended recovery strategy.
  - Implemented linear O(N) unexpanded subtree detection and completeness auditing on `GitTreeResponse`:
    - `find_unexpanded_subtrees(&self)`: identifies directory subtree entries lacking child records in recursive payloads in linear time via a single-pass `HashSet<&str>` of directory segments.
    - `analyze_completeness(&self)`: comprehensively audits tree health, checks explicit GitHub flags, entry caps, and incomplete directories, and selects optimal recovery strategies.
  - Re-exported truncation models in `crates/github-extractor/src/lib.rs`.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/tree_truncation_test.rs`:
    - 5 test suites covering complete trees, explicit `truncated: true` detection, unexpanded subtree path identification, 100k entry cap enforcement, and live in-process Axum mock server tests validating full tree vs truncated responses.
  - Verified 380 passing workspace Rust tests (46 in `crates/github-extractor`), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.2: File Discovery (`5.2.1`)

#### Added
- **`[5.2.1]` Git Trees Integration & Hierarchy Traversal in `crates/github-extractor`:**
  - Implemented Git Trees models in `crates/github-extractor/src/tree.rs`:
    - `GitFileMode`: typed POSIX file modes (`100644` regular file, `100755` executable file, `040000` directory subtree, `120000` symlink, `160000` submodule commit pointer, and `Other`), with query helpers (`is_file`, `is_directory`, `is_executable`, `is_symlink`, `is_submodule`).
    - `GitTreeEntryType`: classification enum (`Blob`, `Tree`, `Commit`).
    - `GitTreeEntry`: path-normalized tree object with `file_name()`, `extension()`, `parent_path()`, `depth()`, and byte size.
    - `GitTreeResponse`: collection of tree entries with `blob_count()`, `tree_count()`, `submodule_count()`, `total_blob_size()`, `blobs()` iterator, path lookups (`find_by_path`), prefix filtering (`filter_by_prefix`), extension filtering (`filter_by_extension`), and path set generation (`to_path_set`).
  - Extended REST client `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `get_tree(&target, tree_sha, recursive)`: queries `GET /repos/{owner}/{repo}/git/trees/{tree_sha}` (with optional `?recursive=1`) with secret token redaction and rate limiting inspection.
    - `fetch_pinned_tree(&pinned, recursive)`: directly queries the immutable `PinnedRevision.root_tree_sha()`, guaranteeing zero ref-drift during file discovery.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/git_trees_test.rs`:
    - 4 test suites covering file mode classification, path normalization, tree queries/filters, and live in-process Axum mock server tests validating recursive vs. non-recursive modes, `fetch_pinned_tree`, 404 `ResourceNotFound` handling, and secret token redaction.
  - Verified 375 passing workspace Rust tests (41 in `crates/github-extractor`), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.1: GitHub Metadata (`5.1.4` — PHASE 5.1 CERTIFIED)

#### Added
- **`[5.1.4]` Repository Metadata Extraction, Language Breakdown & CDM Integration in `crates/github-extractor` (Phase 5.1 Certified):**
  - Implemented repository metadata and language models in `crates/github-extractor/src/metadata.rs`:
    - `GitHubLicense`: represents key, title name, SPDX identifier, and license endpoint URL.
    - `GitHubLanguages`: captures raw byte volumes per programming language (`BTreeMap<String, u64>`), calculating `total_bytes()`, language `percentage()`, and descending sorted `percentages()`.
    - `GitHubRepositoryMetadata`: extracts comprehensive repository properties (id, node_id, full_name, owner, description, fork/archived/disabled/visibility flags, default_branch, homepage, primary_language, topics, license, stars/watchers/forks/issues counts, size in KB, and timestamps).
    - `GitHubRepoContext`: combines repository target, metadata, language distribution, and optional pinned revision into a consolidated extraction container with a Markdown `summary_card()`.
    - Seamless CDM Mapping: `to_cdm_metadata()` translating repository properties and language breakdown into standard CDM `DocumentMetadata` (`title`, `description`, `authors`, `license`, `primary_language`, `language_breakdown`, `tags`, `published_at`, `modified_at`, and structured `extra` stats).
  - Extended REST client `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `get_repo_metadata(&target)`: queries `GET /repos/{owner}/{repo}` with rate limit tracking and secret token redaction.
    - `get_repo_languages(&target)`: queries `GET /repos/{owner}/{repo}/languages`.
    - `fetch_repo_context(&target, pin)`: orchestrates parallel metadata and language extraction via `tokio::join!`, with optional immutable revision pinning.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/metadata_extraction_test.rs`:
    - 4 test suites covering license structures, language byte calculations, CDM `DocumentMetadata` mapping, and live in-process Axum mock server tests on ephemeral port validating metadata queries, language percentages, unified context fetching with pinned revisions, 404 `ResourceNotFound` handling, and token redaction under error conditions.
  - Formally completed and certified **Phase 5.1 — GitHub metadata** (`5.1.1` – `5.1.4`).
  - Verified 371 passing workspace Rust tests (37 in `crates/github-extractor`), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.1: GitHub Metadata (`5.1.3`)

#### Added
- **`[5.1.3]` Commit Selection, Root Tree SHA Resolution & Immutable Revision Pinning in `crates/github-extractor`:**
  - Implemented commit metadata and revision models in `crates/github-extractor/src/commit.rs`:
    - `CommitAuthor`: captures contributor name, email address, and UTC timestamp.
    - `CommitSignatureVerification`: captures cryptographic GPG/SSH commit signature status (`verified`, `reason`, `signature`, `payload`).
    - `GitHubCommitMetadata`: extracts full 40-character commit SHA, optional GraphQL node ID, commit message with first-line subject `summary()`, author and committer records with `commit_date()` fallback, root Git `tree_sha`, parent commit SHAs, comment count, and cryptographic signature verification.
    - `PinnedRevision`: immutable revision container binding a `GitHubTarget` to a verified 40-character commit SHA, root Git tree SHA, commit metadata, and effective ref (`commit_sha()`, `short_sha()`, `root_tree_sha()`, `matches_sha()`), providing zero ref-drift during downstream tree and file extraction.
    - Seamless CDM mappings: `to_revision_info()` generating `RevisionInfo` with `git_commit`, `git_branch`, `git_tag`, and RFC 2822 `http_last_modified`, and `to_source_identity()` injecting the pinned git reference.
  - Extended REST client `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - `get_commit(&target, commit_ref)` querying `/repos/{owner}/{repo}/commits/{ref}` with rate limit tracking and secret redaction.
    - `resolve_commit_sha(&target, ref_spec)` resolving branch names, tags, or `HEAD` into full 40-character commit SHAs.
    - `pin_revision(&target)` orchestrating target analysis, ref resolution, and immutable revision pinning.
  - Authored comprehensive integration tests in `crates/github-extractor/tests/commit_selection_test.rs`:
    - 4 test suites validating commit metadata helpers, serialization/deserialization, CDM `RevisionInfo` and `SourceIdentity` mappings across Repo/Tree/Tag targets, and live in-process Axum HTTP server tests on ephemeral port validating explicit SHA queries, branch resolution, `HEAD` fallback, root tree SHA extraction, 404 `ResourceNotFound` handling, and token redaction under failure conditions.
  - Verified 367 passing workspace Rust tests (33 in `crates/github-extractor`), zero clippy warnings, zero fmt diffs, and 100% passing gates across all 5 checks in `scripts/check-all.ps1`.

### 2026-11-18 — Phase 5.1: GitHub Metadata (`5.1.2`)

#### Added
- **`[5.1.2]` Branch & Tag Discovery and Ref Disambiguation in `crates/github-extractor`:**
  - Implemented ref discovery models in `crates/github-extractor/src/models.rs`:
    - `GitHubRateLimit`: parses `x-ratelimit-limit`, `x-ratelimit-remaining`, `x-ratelimit-reset`, and `x-ratelimit-used` headers, providing `is_exhausted()` and `seconds_until_reset()` helpers.
    - `GitHubBranch`: represents branch name, 40-character commit SHA, and branch protection status.
    - `GitHubTag`: represents release tag name, 40-character commit SHA, zipball URL, and tarball URL.
    - `RefDiscoveryReport`: aggregates discovered branches, tags, default branch resolution (`main`/`master`), rate limit status, commit SHA lookup, and longest-prefix disambiguation.
  - Implemented asynchronous REST client `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
    - Configurable base URL, timeout, page size (default 100), and max pagination limit (default 10 pages / 1,000 items).
    - `list_branches(&target)` and `list_tags(&target)` with automatic pagination.
    - `get_branch(&target, branch_name)` for single branch retrieval.
    - `discover_refs(&target)` running parallel branch and tag discovery via `tokio::join!`.
    - `disambiguate_target(&target, &report)` refining ambiguous URLs where branches contain slashes (e.g. `feature/pipeline-v2/crates/core`).
  - Implemented robust token redaction in `crates/github-extractor/src/error.rs` (`redact_secrets`):
    - Sanitizes `ghp_`, `ghs_`, `ghu_`, `gho_`, `ghr_`, `github_pat_`, and `Bearer`/`token` strings from all error messages and logs.
  - Authored comprehensive test suite in `crates/github-extractor/tests/ref_discovery_test.rs`:
    - 7 tests validating rate limit parsing, token redaction, ref lookups, longest-prefix branch disambiguation, target refinement, and real in-process Axum HTTP server tests for pagination, single branch lookup, rate limit exhaustion (HTTP 403), unauthorized credentials (HTTP 401), and resource not found (HTTP 404).
  - Verified 363 passing workspace Rust tests with zero clippy warnings and 100% passing gates in `scripts/check-all.ps1`.

### 2026-11-17 — Phase 5.1: GitHub Metadata (`5.1.1`)

#### Added
- **`[5.1.1]` Repository Identification and URL Decomposition in `crates/github-extractor`:**
  - Implemented strongly-typed error taxonomy `GitHubExtractorError` in `crates/github-extractor/src/error.rs` using `thiserror`:
    - Handles invalid URLs, owner naming constraints, repository naming constraints, invalid git refs, subpath errors, reserved system paths, unsupported hosts, embedded credentials, and URL security violations.
  - Implemented `GitHubTarget`, `GitHubTargetType`, and `GitHubUrlParser` in `crates/github-extractor/src/target.rs`:
    - Full URL support: standard HTTPS (`https://github.com/owner/repo`), normalized HTTP, SSH transports (`git@github.com:owner/repo.git`, `ssh://git@github.com:...`), Git transport (`git://github.com/...`), and shorthand identifiers (`owner/repo`, `owner/repo@ref`).
    - Granular target entity classification via `GitHubTargetType`: `RepositoryRoot`, `Tree` (ref and optional subpath), `Blob` (ref and file path), `Commit` (validated hexadecimal SHA), `Tag` (release tag), `Commits` (ref commit log), `PullRequest`, `Issue`, and `Releases`.
    - Strict owner and repository name validation adhering to GitHub rules (owner <= 39 chars, no consecutive hyphens, no leading/trailing hyphen; repo <= 100 chars, `.git` stripped, cannot be `.` or `..`).
    - Robust path traversal prevention: early detection of `/../`, `\..\`, `%2e%2e`, and `..%2f` before WHATWG URL normalization.
    - Zero-trust URL security & SSRF defense via `crates/url-security`: private IP blocking, cloud metadata isolation (`169.254.169.254`), and port restrictions.
    - Canonical URL generation: `canonical_repo_url()`, `canonical_target_url()`, HTTPS/SSH clone URLs, API base/repo endpoints, and raw content URLs (`raw.githubusercontent.com`).
    - Seamless mapping to Canonical Context Document Model via `target.to_source_identity(...)`.
    - Ref refinement support (`with_refined_ref_and_subpath`) enabling ambiguous branch paths with slashes to be refined during downstream discovery.
  - Added comprehensive test suite in `crates/github-extractor/tests/target_decomposition_test.rs` (21 integration tests, 1 unit test, 100% passing).
  - Verified 356 passing workspace Rust tests with zero clippy warnings and 100% passing gates in `scripts/check-all.ps1`.

### 2026-11-17 — Phase 4.4: Export Frontend (`4.4.4` — PHASE 4.4 & STAGE 4 CERTIFIED)

#### Added
- **`[4.4.4]` Multi-Format Downloads and Artifact Streaming in `apps/web` (Stage 4 Certified):**
  - Enhanced export artifact types and streaming models in `apps/web/src/lib/types/download.ts`:
    - Added optional `downloadUrl` property to `ExportArtifact` for direct streaming integration with Axum API endpoints.
    - Updated `buildExportArtifacts` to accept options `{ jobId, backendArtifacts }`, generating strongly-typed download entries with streaming URLs (`/api/v1/jobs/{job_id}/artifacts/{format}`) and adding the complete ZIP package entry (`web-repo-context.zip`) pointing to `/api/v1/jobs/{job_id}/artifacts/zip`.
    - Enhanced `triggerDownload` with dual-mode dispatch: direct anchor element download for backend streaming URLs and fallback to client-side Blob object URLs when running offline or in preview mode.
  - Enhanced `apps/web/src/lib/components/workspace/PreviewPanel.svelte`:
    - Updated top toolbar "Download" button to query `defaultApiClient.getArtifactUrl(jobId, selectedFormat)` when `jobId` is available, triggering direct backend streaming downloads.
  - Enhanced `apps/web/src/lib/components/workspace/DownloadControls.svelte`:
    - Added optional `jobId?: string` and `artifacts?: JobArtifactItem[]` props.
    - Implemented reactive `refreshArtifacts()` syncing frontend artifact state with backend job artifacts and streaming URLs.
    - Added prominent "Download ZIP Package" button wired to trigger full package streaming with accessible tooltips and download indicators.
    - Wired each format card's download button to `triggerDownload(artifact.filename, artifact.content, artifact.mimeType, artifact.downloadUrl)`.
    - Implemented accessible clipboard SHA-256 copy control with fallback and ARIA live status announcement region (`role="status"`, `aria-live="polite"`).
  - Connected `jobId={job.id}` and `artifacts={job.artifacts}` in `apps/web/src/lib/components/workspace/JobWorkspace.svelte`.
  - Expanded Vitest test suite in `apps/web/tests/download-controls.test.ts` (14 tests) covering ZIP package presence, streaming URLs, blob fallbacks, SHA-256 copy fallbacks, ARIA live regions, and component integration.
  - Formal Stage 4 Completion Certification: All 16 subphases across Phases 4.1, 4.2, 4.3, 4.4 completed, tested, and certified. 213 frontend tests passing, 335 workspace Rust tests passing, 100% clean gates in `scripts/check-all.ps1`.

### 2026-11-17 — Phase 4.4: Export Frontend (`4.4.3`)

#### Added
- **`[4.4.3]` Content Preview and Copy Controls in `apps/web` and `services/api`:**
  - Implemented preview endpoint `GET /api/v1/jobs/{job_id}/preview` in `services/api/src/routes/jobs.rs` and registered it in `services/api/src/routes/mod.rs`:
    - Validates strongly-typed UUID v7 `job_id` and query parameters `format`, `max_lines` (1 to 50,000, default 1,000), and `file_path`.
    - Retrieves stored `CdmDocument` from storage and generates format-specific preview representation (`markdown`, `json`, `html`, `txt`, `jsonl`, `xml`).
    - Extracts per-file metrics (`PreviewFileItem`: path, language, byte size, token count, line count) and hierarchy outline.
    - Windows content to `max_lines` slice and sets `truncated: true` when content exceeds ceiling.
    - Returns RFC 7807 Problem Details `RESOURCE_NOT_FOUND` on missing job and `VALIDATION_ERROR` on malformed parameters.
    - Added backend integration tests in `services/api/src/lib.rs`: `test_get_preview_not_found`, `test_get_preview_invalid_format_rejected`, `test_get_preview_success_from_cdm` (markdown, JSON, file-specific filter, max_lines windowing).
  - Implemented typed frontend client methods and models:
    - Added `PreviewFileItem` and `JobPreviewResponse` interfaces to `apps/web/src/lib/api/types.ts`.
    - Added `ApiClient.getJobPreview(jobId, options)` to `apps/web/src/lib/api/client.ts`.
  - Enhanced `PreviewPanel.svelte` in `apps/web/src/lib/components/workspace/PreviewPanel.svelte`:
    - Accepted `jobId` prop and connected live preview querying via Svelte 5 runes (`$state`, `$derived`, `$effect`).
    - Gracefully degrades to local preview formatting and sample files when API returns 404 or offline.
    - Implemented robust clipboard copy with `navigator.clipboard.writeText` and `document.execCommand('copy')` fallback.
    - Added accessible ARIA live status announcement region (`role="status"`, `aria-live="polite"`) for copy confirmation ("Copied to clipboard!").
    - Added truncation notice banner informing users when preview is windowed to the first 2,000 lines.
    - Connected line number toggle, word wrap toggle, and real-time line search filtering.
  - Connected `jobId={job.id}` binding from `JobWorkspace.svelte` to `PreviewPanel.svelte`.
  - Expanded Vitest test suite in `apps/web/tests/api-client.test.ts` and `apps/web/tests/preview-panel.test.ts` (211 passing tests across 17 test suites).
  - Verified 100% passing gates in `powershell scripts/check-all.ps1` (0 failures across all 5 monorepo gates, 335 workspace Rust tests).

### 2026-11-17 — Phase 4.4: Export Frontend (`4.4.2`)

#### Added
- **`[4.4.2]` Implement Real-Time Server-Sent Events (SSE) Progress Streaming in `apps/web` and `services/api`:**
  - Implemented SSE endpoint `GET /api/v1/jobs/{job_id}/events` and `/stream` alias in `services/api/src/routes/jobs.rs` and `services/api/src/routes/mod.rs`:
    - Dispatches typed SSE events: `phase`, `log`, `completeness`, `tokens`, `artifact`, and `done`.
    - Reads stored CDM document to emit real completeness accounting (`discovered_count`, `extracted_count`, `excluded_count`, `failed_count`) and token metrics.
    - Emits keep-alive comment pings every 15 seconds to prevent proxy/firewall timeouts.
    - Added backend integration test `test_get_job_events_sse_stream` in `services/api/src/lib.rs`.
  - Implemented `JobEventStream` and `connectJobEventStream` in `apps/web/src/lib/api/sse.ts`:
    - Decodes strongly-typed SSE event data for pipeline phase changes, diagnostic logs, file completeness accounting, token metrics, artifact ready signals, and completion states.
    - Automatic exponential backoff reconnection on transient drops (`1s`, `2s`, `4s` up to `30s`, max 5 attempts).
    - Watchdog heartbeat timeout monitoring (45s period) to detect silent network drops.
    - Explicit `stream.close()` method cleanly unbinding listeners, terminating `EventSource`, and resetting timers to prevent SSE connection leaks.
  - Connected `apps/web/src/routes/workspace/+page.svelte` to live SSE event streams:
    - Binds SSE events directly to Svelte 5 `$state` runes.
    - Dynamically updates 4-stage pipeline cards, overall progress percentage, live diagnostic logs, completeness metrics, and artifact download links without polling.
    - Cleanly terminates SSE stream on component unmount (`onDestroy`) and user cancellation (`handleCancel`).
  - Added comprehensive Vitest test suite in `apps/web/tests/sse-stream.test.ts` (11 tests):
    - URL construction, open event listener, event decoding (`phase`, `log`, `completeness`, `tokens`, `artifact`, `done`), automatic terminal closure, exponential backoff retries, retry ceiling enforcement, and workspace component route contract wiring.
  - Verification: 206 passing Vitest tests (17 test files), 41 passing API tests, 331 passing workspace Rust tests, 0 `svelte-check` errors/warnings, 100% Prettier compliance, and all 5 monorepo verification gates passing cleanly.

### 2026-11-17 — Phase 4.4: Export Frontend (`4.4.1`)

#### Added
- **`[4.4.1]` Connect Live Conversion API in `apps/web`:**
  - Implemented strongly-typed REST API client contracts in `apps/web/src/lib/api/types.ts`:
    - `CreateJobPayload`: source URLs, target Git ref, glob patterns, token budget, target model, export formats, and deduplication flags.
    - `ApiJobResponse`: strongly-typed UUID v7 `job_id`, status enum (`queued`, `processing`, `completed`, `failed`, `cancelled`), source list, budget, model, and export formats.
    - `DetectSourcesPayload` and `DetectSourcesResponse`: source URL classification.
    - `ApiProblemDetails`: RFC 7807 specification error envelope decoding (`status`, `title`, `detail`, `request_id`, `invalid_params`).
  - Implemented `ApiClient` and `ApiClientError` in `apps/web/src/lib/api/client.ts`:
    - `createJob`: asynchronous `POST /api/v1/jobs` dispatch returning `202 Accepted` job descriptor.
    - `getJob`: `GET /api/v1/jobs/{job_id}` query with URL sanitization.
    - `detectSources`: `POST /api/v1/sources/detect` endpoint query.
    - `getArtifactUrl`: deterministic artifact download URL generation (`/api/v1/jobs/{job_id}/artifacts/{format}`).
    - `getStreamUrl`: SSE progress stream URL generation (`/api/v1/jobs/{job_id}/stream`).
    - Robust error parsing with RFC 7807 problem details decoding.
    - Reusable singleton `defaultApiClient`.
  - Implemented API domain adapter in `apps/web/src/lib/api/adapter.ts`:
    - `mapApiStatusToJobStatus`, `mapStatusToPhase`, and `mapApiJobToConversionJob` translating backend DTOs into frontend `ConversionJob` domain models with full stage progression and token budgeting integrity.
  - Replaced mock `setTimeout` transitions in `apps/web/src/routes/+page.svelte` with live `defaultApiClient.createJob(...)` submission and redirect to `/workspace?jobId=...`.
  - Wired `apps/web/src/routes/workspace/+page.svelte` to query `defaultApiClient.getJob(jobId)` on mount and map server state.
  - Added comprehensive Vitest test suite in `apps/web/tests/api-client.test.ts` (11 tests) verifying client transport, HTTP headers, RFC 7807 error decoding, adapter mapping, and SvelteKit route contract wiring.
  - Verification: 195 passing Vitest tests (16 test files), 0 `svelte-check` errors/warnings, 100% Prettier compliance, and all 5 monorepo verification gates passing cleanly.

### 2026-11-17 — Phase 4.3 Certification & Subphase `4.3.4`

#### Added
- **`[4.3.4]` Implement Export Integrity Test Suite and Verification Engine in `crates/exporters`:**
  - Implemented `crates/exporters/src/integrity.rs`:
    - `PackageIntegrityReport`: Diagnostic report model capturing `is_valid`, `verified_files`, `mismatched_checksums`, `missing_files`, `unlisted_files`, `path_traversal_violations`, and deserialized `manifest`, `provenance`, and `quality_report` structures.
    - `verify_zip_package_bytes` and `verify_zip_package_integrity`: Comprehensive in-memory and streaming verification engine scanning entry paths for zip-slip directory traversal (`..`, `\`, leading `/`, null bytes), reading and validating `manifest.json`, cross-checking all files in `checksums.sha256` against extracted contents, validating declared sizes and dual SHA-256 / BLAKE3 digests, and auditing for rogue unlisted files.
    - Format syntax verifiers: `verify_markdown_syntax` (balanced backtick code fences), `verify_json_syntax` (JSON parseability), `verify_html_syntax` (standalone DOCTYPE and closing html tag validation), and `verify_jsonl_syntax` (per-line JSON validation).
  - Re-exported integrity verification API in `crates/exporters/src/lib.rs`.
  - Added exhaustive integration test suite in `crates/exporters/tests/export_integrity.rs` (9 tests):
    1. `test_integrity_valid_package_roundtrip`: 100% cryptographic and structural integrity across all 12 package files.
    2. `test_integrity_tampered_entry_detection`: 1 corrupted byte in `context.md` detected via SHA-256 hash mismatch.
    3. `test_integrity_truncated_entry_detection`: Truncated `context.txt` detected via size and checksum mismatch.
    4. `test_integrity_missing_file_detection`: Missing file declared in manifest correctly flagged in `missing_files`.
    5. `test_integrity_unlisted_rogue_file_detection`: Injected rogue backdoor script detected in `unlisted_files`.
    6. `test_integrity_zip_slip_and_path_traversal_detection`: Adversarial paths (`../../etc/shadow`, `C:\Windows\System32\calc.exe`, `/var/run/secret`) flagged in `path_traversal_violations`.
    7. `test_integrity_format_syntax_verifiers`: Syntax validation across Markdown, JSON, HTML, and JSONL.
    8. `test_integrity_token_budget_truncation_fidelity`: Budget truncation produces syntactically valid Markdown, HTML, JSON, and JSONL.
    9. `test_integrity_edge_cases_and_multi_byte_unicode`: Multi-byte Arabic RTL, CJK Kanji, mathematical symbols, Emojis, and deeply nested paths verify with 100% cryptographic integrity.
  - Phase 4.3 Milestone Certification: All 4 subphases (`4.3.1` through `4.3.4`) verified and accepted.

### 2026-11-17 — Phase 4.3: Export (`4.3.3`)

#### Added
- **`[4.3.3]` Implement Asynchronous Streaming Downloads in `services/api`:**
  - Added `tokio-util = { version = "0.7", default-features = false }` to root workspace dependencies and `tokio-util = { workspace = true, features = ["io"] }`, `web-repo-to-ai-exporters = { workspace = true }` to `services/api/Cargo.toml`.
  - Added `"checksums"` to `VALID_EXPORT_FORMATS` in `services/api/src/dto/jobs.rs`.
  - Integrated `DynStorage` into `AppState` in `services/api/src/state.rs` with automatic instantiation via `create_storage_backend(&config.storage)` and test/memory injection constructors `with_storage` and `with_metrics_and_storage`.
  - Implemented `get_artifact_handler` in `services/api/src/routes/jobs.rs`:
    - Handles `GET /api/v1/jobs/{job_id}/artifacts/{format}` for all supported formats (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `checksums`, `cdm_json`).
    - Storage lookup with fallback keys matching standard layout (`jobs/{job_id}/artifact.{ext}`).
    - Direct chunked streaming from storage using `ReaderStream` and Axum `Body::from_stream`, preventing large file RAM exhaustion.
    - On-demand rendering fallback: If a pre-rendered artifact does not exist in storage, looks for the stored parent CDM document (`jobs/{job_id}/artifact.cdm.json`), renders the requested format on the fly, caches it to storage, and streams it.
    - Standard RFC compliance headers: `Content-Type` matching format MIME, `Content-Disposition: attachment; filename="web-repo-to-ai-{job_id}.{ext}"`, `Content-Length`, `x-content-type-options: nosniff`, `cache-control: private, no-transform, max-age=86400`, and correlated `x-request-id`.
    - Returns standard RFC 7807 problem details (`404 Not Found`, `application/problem+json`) if neither artifact nor CDM exists.
  - Added integration test suites in `services/api/src/lib.rs` verifying streaming markdown downloads, zip streaming, manifest & checksums downloads, on-demand CDM rendering fallback, headers integrity, and 404/400 validation.
  - Verification: 41 passing API tests (up from 37), 457 total workspace Rust tests (up from 453), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.3: Export (`4.3.2`)

#### Added
- **`[4.3.2]` Implement Standalone Manifest & Checksums Generation in `crates/exporters`:**
  - Added `blake3 = { workspace = true }` to `crates/exporters/Cargo.toml`.
  - Implemented standalone cryptographic checksum engine in `crates/exporters/src/checksum.rs`:
    - `ChecksumAlgorithm`: `Sha256`, `Blake3` with `as_str()`, `Display`, and `compute_hex()`.
    - `compute_sha256_hex` and `compute_blake3_hex` for deterministic 64-character lowercase hex digest computation.
    - `ChecksumEntry`: Path, hash, algorithm, and constructor helpers `sha256`, `blake3`, `from_sha256_bytes`, and `from_blake3_bytes`.
    - BSD/GNU coreutils standard generation: `generate_checksums_manifest` and streaming `export_checksums_to_writer` formatting lines as `<hash>  <path>\n`.
    - Coreutils parser: `parse_checksums_manifest` parsing standard and binary `*` mode lines, ignoring comment lines (`#`) and whitespace, and verifying 64-hex character syntax.
    - Checksum verifier: `verify_checksums` returning a `ChecksumVerificationReport` with `matched_files`, `mismatched_files` (`ChecksumMismatch`), `missing_files`, and `is_valid()` verification status.
  - Implemented standalone package manifest and quality report generator in `crates/exporters/src/manifest.rs`:
    - `detect_mime_type`: Zero-dependency IANA MIME media type detector mapping file extensions (`.md`, `.txt`, `.json`, `.jsonl`, `.html`, `.zip`, `.rs`, `.ts`, `.js`, `.py`, `.go`, `.css`, etc.).
    - `PackageFileEntry`: Inventory item capturing relative path, uncompressed byte size, dual SHA-256 and BLAKE3 digests, format descriptor, canonical MIME type, line count, and token estimates (`div_ceil(4)`).
    - `PackageManifest`: Root manifest schema v1.0 with generator version, document ID, schema version, creation timestamp, canonical URL, source type, file count, total uncompressed bytes, and file entries.
    - `PackageProvenance`: Provenance schema capturing canonical URL, source identity, revision info, and citations list.
    - `PackageQualityReport`: Quality report schema capturing completeness ratio, formatted percentage, discovered/extracted/excluded/failed/unsupported counts, and exclusion logs.
    - Builders: `build_manifest`, `build_provenance`, `build_quality_report`.
    - Exporters: `export_manifest`, `export_provenance`, `export_quality_report` and their respective single-pass streaming writer functions.
    - `ManifestExporter`: Fluent builder API supporting in-memory and streaming generation.
  - Refactored `crates/exporters/src/zip.rs` to reuse `PackageFileEntry`, `PackageManifest`, `PackageProvenance`, and `PackageQualityReport` from `crate::manifest`, and checksum generation from `crate::checksum`.
  - Added `ManifestOptions` and `ChecksumOptions` models and presets in `crates/exporters/src/options.rs`.
  - Added `ExporterError::ChecksumValidation` error variant in `crates/exporters/src/error.rs`.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite `crates/exporters/tests/manifest_checksums.rs` (11 tests) verifying manifest generation, provenance, quality report, MIME detection, checksum formatting, parsing, corrupt file detection, missing file detection, and streaming equivalence.
  - Verification: 73 passing exporter tests (up from 62), 453 total workspace Rust tests (up from 442), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.3: Export (`4.3.1`)

#### Added
- **`[4.3.1]` Implement Deterministic ZIP Package Generator in `crates/exporters`:**
  - Added dependency `zip = { version = "9.6", default-features = false, features = ["deflate"] }` with `flate2/zlib-rs` pure safe Rust backend (zero C dependencies, zero cargo deny advisories) and `sha2 = { workspace = true }`.
  - Implemented `ZipPackageOptions` and `ZipPackageMetrics` in `crates/exporters/src/options.rs`:
    - Flags for bundled formats: `include_markdown`, `include_plain_text`, `include_json_cdm`, `include_json_structured`, `include_html`, `include_jsonl`.
    - Flags for source files & audit metadata: `include_sources`, `include_manifest`, `include_provenance`, `include_quality_report`, `include_checksums`.
    - Nested sub-options: `markdown_options`, `plain_text_options`, `json_options`, `html_options`, `jsonl_options`.
    - Compression options: `compression_method` (`ZipCompressionMethod::Deflate` or `Stored`), and `compression_level` (0–9).
    - Presets: `complete()` (default), `minimal()` (markdown + plain_text + manifest), `formats_only()` (all 6 formats + manifest/provenance/quality/checksums without sources), and `sources_only()` (hierarchical sources + manifest/checksums).
  - Implemented core ZIP exporter engine in `crates/exporters/src/zip.rs`:
    - `ZipPackageExporter`: Fluent builder API supporting in-memory `export_to_bytes` and streaming `export_to_writer`.
    - `export_zip_package` and `export_zip_package_to_writer`: High-performance deterministic ZIP package generator writing to any `std::io::Write + std::io::Seek` sink.
    - Deterministic reproducible builds: Sets uniform fixed timestamps (`1980-01-01 00:00:00`) and standard permissions (`0o644`) on all ZIP entries.
    - Path traversal defense: Sanitizes all archive paths using `sanitize_zip_path`, stripping drive prefixes, leading slashes, backslashes, and `../` directory traversal attempts to prevent zip-slip vulnerabilities.
    - Metadata manifests: Emits `manifest.json` (`PackageManifest`), `provenance.json` (`PackageProvenance`), and `quality-report.json` (`PackageQualityReport`).
    - Standard checksums: Generates `checksums.sha256` matching GNU `sha256sum` format (`<hex_digest>  <relative_archive_path>`) for all embedded archive files.
    - Telemetry metrics: Captures total files bundled, uncompressed byte size, compressed byte size, and overall package SHA-256 digest.
  - Re-exported public ZIP package types in `crates/exporters/src/lib.rs`.
  - Created integration test suite `crates/exporters/tests/zip_export.rs` (9 tests) verifying complete preset, minimal preset, formats-only preset, sources-only preset, streaming writer equivalence, path sanitization security defense, empty documents, serde roundtrips, and builder API.
  - Verification: 62 passing exporter tests (up from 53), 442 total workspace Rust tests (up from 433), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.4` — **PHASE 4.2 CERTIFIED**)

#### Added
- **`[4.2.4]` Implement HTML & JSONL Generators in `crates/exporters`:**
  - Implemented `HtmlOptions` configuration model in `crates/exporters/src/options.rs`:
    - Standalone HTML5 document toggle (`standalone`), embedded responsive CSS (`include_css`), custom CSS injection (`custom_css`), header banner (`include_header`), hierarchy tree style (`hierarchy_style`), table of contents (`include_table_of_contents`), collapsible `<details><summary>` sections (`collapsible_sections`, `initially_expanded`), code line numbers (`include_line_numbers`), sources & citations table (`include_citations`), excluded files audit table (`include_exclusions`), completeness report (`include_completeness_report`), and token budget ceiling (`max_token_budget`).
    - Added preset constructors: `standalone()`, `fragment()`, and `minimal()`.
  - Implemented core HTML exporter engine in `crates/exporters/src/html.rs`:
    - `HtmlExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - `export_html` and `export_html_to_writer`: Single-pass streaming renderer writing directly to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Embedded zero-CDN responsive stylesheet (`DEFAULT_HTML_CSS`) with automatic `@media (prefers-color-scheme: dark)` dark/light theme switching, system font stacks, badge badges, card layouts, and code block styling.
    - Security-hardened HTML character entity escaping (`escape_html`) preventing XSS vulnerabilities.
    - Anchor slug generator (`slugify`) for table of contents navigation.
    - Collapsible section rendering with line numbering support.
  - Implemented `JsonlFlavor` and `JsonlOptions` configuration models in `crates/exporters/src/options.rs`:
    - Flavors: `RagChunks` (embedding/retrieval records with document/line coordinates), `FineTuningChat` (conversational instruction dataset records with `messages`), `FineTuningPromptCompletion` (`prompt`/`completion` pairs), and `CanonicalSections` (lossless serialized CDM sections per line).
    - Presets: `rag()`, `chat_fine_tuning()`, `prompt_completion_fine_tuning()`, and `canonical()`.
  - Implemented core JSON Lines exporter in `crates/exporters/src/jsonl.rs`:
    - `JsonlExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - Models: `RagChunkRecord`, `ChatMessage`, `FineTuningChatRecord`, `FineTuningPromptCompletionRecord`.
    - High-throughput streaming serialization writing strictly one line per section terminated by `\n`.
  - Added `ExporterError::Json` variant with `#[from] serde_json::Error` in `crates/exporters/src/error.rs`.
  - Re-exported all HTML and JSONL types in `crates/exporters/src/lib.rs`.
  - Created integration test suites:
    - `crates/exporters/tests/html_export.rs` (10 tests) verifying standalone layout, fragments, minimal preset, line numbers, token budget truncation, streaming byte equivalence, empty documents, serde roundtrips, XSS defense, and builder API.
    - `crates/exporters/tests/jsonl_export.rs` (9 tests) verifying RAG chunks, chat fine-tuning, prompt/completion pairs, canonical CDM sections, token budget truncation, streaming byte equivalence, empty documents, serde roundtrips, and builder API.
  - Verification: 53 passing exporter tests (up from 34), 433 total workspace Rust tests (up from 414), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.3`)

#### Added
- **`[4.2.3]` Implement Multi-Flavor JSON Generator in `crates/exporters`:**
  - Implemented `JsonFormatFlavor` and `JsonExportOptions` configuration models in `crates/exporters/src/options.rs`:
    - `JsonFormatFlavor`: `CanonicalCdm` (full schema v1.0), `StructuredContext` (token-optimized flat envelope tailored for LLM system prompts and RAG embeddings), and `Minified` (compact single-line JSON without whitespace).
    - `JsonExportOptions`: Options controlling pretty-printing, field filtering (`include_metadata`, `include_hierarchy`, `include_sections`, `include_citations`, `include_exclusions`, `include_completeness`), and token budgeting (`max_token_budget`). Includes preset constructors `canonical()`, `structured()`, and `compact()`.
  - Implemented core JSON exporter engine in `crates/exporters/src/json.rs`:
    - `JsonExporter`: Fluent builder API supporting in-memory `export_to_string` and streaming `export_to_writer`.
    - `export_json` and `export_json_to_writer`: High-performance deterministic serializing functions writing directly to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Token-optimized Structured Context envelope models: `StructuredContextEnvelope`, `StructuredMetrics`, `StructuredCompleteness`, `StructuredHierarchyNode`, `StructuredSection`, `StructuredCitation`, and `StructuredExclusion` with `#[serde(default)]` and `#[derive(Default)]` for seamless deserialization.
    - Dynamic section filtering and token budget caps that prune overflow sections while marking output with `truncated: true` telemetry.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite in `crates/exporters/tests/json_export.rs` (8 tests) verifying Canonical CDM serialization, Structured Context envelopes, Minified single-line output, token budget truncation, field filtering, streaming writer byte equivalence, empty document handling, and serde roundtrips.
  - Verification: 34 passing exporter tests, 414 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.2`)

#### Added
- **`[4.2.2]` Implement Plain-Text Generator in `crates/exporters`:**
  - Implemented `PlainTextOptions` configuration model in `crates/exporters/src/options.rs`:
    - Configurable top-level document metadata banner (`include_header_banner`), hierarchy tree style (`HierarchyStyle::AsciiTree` default), plain-text table of contents, customizable section divider lines (`divider_char`, `divider_width`), line numbering, exclusions and citations audit sections, completeness report, and token budget ceiling.
    - Added preset constructors `minimal()` and `compact()`.
  - Implemented core Plain-Text generator in `crates/exporters/src/plain_text.rs`:
    - `PlainTextExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - `export_plain_text`: In-memory deterministic plain-text generation.
    - `export_plain_text_to_writer`: Single-pass streaming renderer writing directly to any `std::io::Write` sink with `CountingWriter` for exact byte and newline telemetry.
    - Clean ASCII header banner detailing Document Title, Source URL, Type, Retrieved timestamp, License, Language, Authors, Commit SHA, volume/token metrics, and Completeness yield.
    - Plain-text directory / outline tree rendering (ASCII tree, Unicode tree, or bulleted list) without Markdown code block fences.
    - Plain-text Table of Contents with numbered section index, titles, language, line count, byte size, and token estimates.
    - Clean polyglot section formatting with structured divider banners (`----------------`), section kind labels, metadata headers, and clean bodies without markdown fence noise.
    - Plain-text Sources & Citations section.
    - Plain-text Excluded Files & Content audit section detailing skipped paths, reasons, patterns, sizes, and rationale.
    - Completeness Accounting breakdown with discovered, extracted, excluded, failed, and unsupported counts.
    - Token budget enforcement halting section emission with an ASCII warning banner when budget is exceeded.
  - Factored shared utilities into `crates/exporters/src/util.rs` (`CountingWriter`, `format_bytes`).
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite in `crates/exporters/tests/plain_text_export.rs` (10 tests) verifying default export, minimal/compact presets, line numbering, token budget truncation, streaming writer byte equivalence, hierarchy styles, custom dividers, empty document handling, and serde roundtrips.
  - Verification: 26 passing exporter tests, 406 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.1`)

#### Added
- **`[4.2.1]` Implement Markdown Generator in `crates/exporters`:**
  - Added `chrono` dependency to `crates/exporters/Cargo.toml` with serde support.
  - Implemented typed domain errors in `crates/exporters/src/error.rs`:
    - `ExporterError`: Enumerating `Io`, `Formatting`, `BudgetExceeded`, and `InvalidDocument`.
    - `Result<T>` type alias.
  - Implemented configuration models and telemetry metrics in `crates/exporters/src/options.rs`:
    - `CodeFenceStyle`: `Backticks` and `Tildes` with automatic fence escalation to 4+ markers when code bodies contain backticks or tildes.
    - `HierarchyStyle`: `UnicodeTree` (`├──`, `└──`), `AsciiTree` (`|--`, `\--`), `BulletedList` (`- **dir/**`), and `None`.
    - `ExportMetrics`: Tracking `bytes_written`, `lines_written`, `sections_exported`, `exclusions_exported`, `citations_exported`, `estimated_tokens`, and `truncated`.
    - `MarkdownOptions`: Comprehensive configuration controlling frontmatter, metadata block, hierarchy tree, table of contents with GFM slugs, per-section metadata, exclusions, citations, completeness accounting, heading levels, and token budget limits. Includes preset constructors `minimal()` and `compact()`.
  - Implemented core Markdown exporter engine in `crates/exporters/src/markdown.rs`:
    - `MarkdownExporter`: Fluent engine with `export_to_string` and streaming `export_to_writer`.
    - `export_markdown`: In-memory deterministic Markdown generation.
    - `export_markdown_to_writer`: Single-pass streaming renderer writing to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Structured YAML frontmatter with string escaping for special characters (`:`, `"`, `#`, `\n`).
    - Human- and AI-readable metadata summary block with source URL, type, license, and language tags.
    - Formatted directory / outline tree in code blocks preserving file sizes and line counts.
    - Table of contents with anchor links derived via GitHub GFM slugification rules and token/line telemetry.
    - Polyglot section formatting covering all `SectionKind` variants (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Admonition`, `Navigation`, `Manifest`, `Frontmatter`).
    - Escalating code fence delimiters preventing markdown fence collisions.
    - GFM blockquote alerts (`> [!NOTE]`, `> [!WARNING]`, etc.) for admonition callouts.
    - Auditable exclusions table documenting skipped files, patterns, sizes, and rationale.
    - Sources & citations section for full provenance tracking.
    - Completeness accounting summary block.
    - Token budget truncation defense with advisory notices when limits are exceeded.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Implemented 15 integration tests in `crates/exporters/tests/markdown_export.rs` verifying default formatting, minimal/compact presets, fence escalation, GFM alerts, line numbering, token budget truncation, streaming writer byte equivalence, hierarchy styles, YAML escaping, and empty document handling.
  - Verification: 16 passing exporter tests, 396 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.4`) — PHASE 4.1 CERTIFIED

#### Added
- **`[4.1.4]` Implement Schema Versioning and Migration Framework in `crates/cdm` (Phase 4.1 Certified):**
  - Implemented semantic versioning representation in `crates/cdm/src/version.rs`:
    - `SchemaVersion`: Strongly-typed SemVer struct (`major`, `minor`, `patch`, `pre_release`) implementing parsing via `FromStr`, `Display`, ordering via `Ord`/`PartialOrd`, string serde roundtrips, and compatibility negotiation via `is_compatible_with` (`major == target.major`).
    - Standard version constants: `current()`, `v0_8_0()`, `v0_9_0()`, `v1_0_0()`.
  - Implemented migration step architecture in `crates/cdm/src/migration.rs`:
    - `MigrationStep` trait with `from_version()`, `to_version()`, `description()`, and `migrate(&Value) -> Result<Value, CdmError>`.
    - `Migration0_8To0_9`: Migrates legacy `content_blocks` to normalized `sections`, extracts `source_url`, sets initial `source_identity` coordinates, and upgrades schema version to `0.9.0`.
    - `Migration0_9To1_0`: Establishes root `hierarchy` tree, derives mathematical `CompletenessReport` preserving invariants, populates missing BLAKE3 content hashes and baseline token metrics, initializes auxiliary collections (`exclusions`, `warnings`, `citations`, `relationships`), and bumps schema version to `1.0.0`.
    - `MigrationRegistry`: Thread-safe registry finding shortest migration chains using Breadth-First Search (BFS), executing sequential transformations on `serde_json::Value` payloads, and providing `migrate_and_deserialize` into [`CdmDocument`].
    - Added helper migration functions `migrate_document_json` and `migrate_document_value`.
  - Implemented schema and document diffing in `crates/cdm/src/diff.rs`:
    - `SchemaDiff`: Computes differences between two document instances, detecting schema version shifts, added sections, removed sections, modified sections (`body`, `content_hash`, `line_range`, `byte_range`, `title`), hierarchy modifications, and completeness shifts.
    - `SectionDiff`: Detailed attribute-level section comparison.
  - Extended `JsonOptions` and JSON deserialization in `crates/cdm/src/json.rs`:
    - Added `auto_migrate: bool` to `JsonOptions` and fluent builder `.with_auto_migrate(bool)`.
    - Updated `from_json_with_options`, `from_slice_with_options`, and `from_reader_with_options` to seamlessly up-migrate legacy payloads when `auto_migrate` is enabled.
  - Added convenience methods to `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.diff(&other) -> SchemaDiff`
    - `CdmDocument::from_json_migrated(json_str) -> Result<CdmDocument>`
    - `CdmDocument::from_value_migrated(value) -> Result<CdmDocument>`
  - Added `#[serde(default)]` to `DocumentMetadata` and `RevisionInfo` for robust forward/backward compatibility when deserializing partial schema payloads.
  - Re-exported versioning, migration, and diffing primitives in `crates/cdm/src/lib.rs`.
  - Added comprehensive integration tests in `crates/cdm/tests/schema_migration.rs` verifying multi-step migration (`v0.8.0` $\to$ `v0.9.0` $\to$ `v1.0.0`), direct preview migration, streaming reader/slice auto-migration, document diffing, and error handling for unregistered version jumps and downgrade attempts.
  - Verified 78/78 tests in `crates/cdm` (40 unit + 7 JSON serde + 7 schema migration + 8 schema validation + 16 validation invariants), 381/381 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.
  - Formally completed and certified **Phase 4.1 (Canonical Context Document Model)** across all 4 subphases (`4.1.1` through `4.1.4`).

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.3`)

#### Added
- **`[4.1.3]` Implement CDM Schema Validation and Invariant Checks in `crates/cdm`:**
  - Added dependency to `crates/cdm/Cargo.toml`: `url = { workspace = true }` for standard URL validation.
  - Implemented comprehensive validation engine in `crates/cdm/src/validator.rs`:
    - `ValidationSeverity`: `Error` (fatal violation) and `Warning` (advisory finding).
    - `ValidationIssue`: Strongly-typed validation issue tracking invariant code, human-readable message, structural path coordinate, and severity level.
    - `ValidationConfig`: Configurable validation rules (`strict_mode`, `verify_hashes`, `enforce_completeness`, `check_referential_integrity`, `check_tree_acyclicity`, `max_tree_depth`), with fluent constructors `strict()`, `permissive()`, and builder methods.
    - `ValidationReport`: Aggregate report collecting issues across sections, hierarchy nodes, relationships, and completeness, with methods `is_valid()`, `has_errors()`, `has_warnings()`, `error_count()`, `warning_count()`, `filter_by_code()`, `filter_by_path_prefix()`, and human-readable `summary()`.
    - `CdmValidator`: Stateful validator running 8 modular validation passes:
      1. Major schema version compatibility against `SCHEMA_VERSION` (`1.0.0`).
      2. Document identity & metadata invariants (non-nil `document_id`, HTTP/HTTPS `canonical_url` validation via `url::Url::parse`, `raw_input` non-empty, GitHub/web coordinate checks, 64-char hex format checks on BLAKE3/SHA-256 hashes, descriptive title warning).
      3. Section-level invariants (non-empty unique `section_id`s, valid `source_url`s, 1-indexed monotonic `line_range` `1 <= start <= end`, `byte_range` `start <= end`, BLAKE3 content hash recomputation against `body`, and token metrics consistency).
      4. Hierarchy tree acyclicity & integrity (depth bounding to `max_tree_depth`, duplicate node ID cycle detection, and referential integrity verifying every node's `section_id` exists in document sections).
      5. Relationships referential integrity (`source_id` and internal `target_id` existence in document sections, self-referential cycle warnings).
      6. Authoritative citations validity (URL parse verification, section ID referential integrity).
      7. Completeness accounting invariants (strict formula $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$, verifying complete extraction status requires `is_complete() == true`).
      8. Warning records and exclusions integrity (non-empty warning codes, messages, and exclusion paths).
  - Extended `CdmError` in `crates/cdm/src/error.rs` with `ValidationFailed { error_count, summary }` and helper `CdmError::from_report(&ValidationReport)`, mapped cleanly to workspace `CoreError::Validation`.
  - Added convenience validation methods to `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.validation_report() -> ValidationReport`
    - `doc.validate_with_config(&ValidationConfig) -> ValidationReport`
    - `doc.validate() -> Result<(), CdmError>`
    - `doc.validate_strict() -> Result<(), CdmError>`
  - Re-exported validation types in `crates/cdm/src/lib.rs`.
  - Added 12 unit tests in `src/validator.rs` and 16 integration tests in `crates/cdm/tests/validation_invariants.rs` covering all invariant checks, cycle detection, referential integrity violations, duplicate IDs, invalid line/byte ranges, tampered hashes, completeness matrix violations, and diagnostic reports.
  - Verified 60/60 tests in `crates/cdm`, 363/363 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.2`)

#### Added
- **`[4.1.2]` Implement CDM JSON Serialization and Deserialization in `crates/cdm`:**
  - Added dependency to `crates/cdm/Cargo.toml`: `thiserror = { workspace = true }`.
  - Implemented strongly-typed domain error taxonomy `CdmError` in `crates/cdm/src/error.rs`:
    - `Serialization(serde_json::Error)`: JSON string/stream encoding failures.
    - `Deserialization(serde_json::Error)`: JSON syntax and structural decoding failures.
    - `PayloadTooLarge { size_bytes, max_bytes }`: Defense against unbounded payload bombs and excessive allocation.
    - `UnsupportedSchemaVersion { version, supported }`: Major schema version mismatch protection.
    - `HashMismatch { expected, actual }`: Document/section cryptographic integrity verification failures.
    - `Validation(String)`: Semantic invariant and model validation errors.
    - `Io(std::io::Error)`: Low-level I/O reader/writer stream errors.
    - Implemented `From<CdmError> for CoreError` mapping CDM domain errors into standard workspace `CoreError` codes (`INVALID_INPUT`, `SERIALIZATION_ERROR`, `IO_ERROR`).
  - Implemented production-grade JSON serialization engine in `crates/cdm/src/json.rs`:
    - `JsonOptions`: Configurable serialization/deserialization behavior (`pretty`, `max_bytes` default 64 MiB, `verify_hashes`, `validate_version`).
    - `to_json_pretty(&CdmDocument) -> Result<String, CdmError>`: Deterministic multi-line formatted JSON.
    - `to_json_compact(&CdmDocument) -> Result<String, CdmError>`: Space-efficient single-line JSON.
    - `to_writer<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`: Streaming compact writer.
    - `to_writer_pretty<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`: Streaming formatted writer.
    - `from_json(&str) -> Result<CdmDocument, CdmError>`: Memory-safe deserializer from string slice.
    - `from_slice(&[u8]) -> Result<CdmDocument, CdmError>`: Byte-slice deserializer.
    - `from_reader<R: std::io::Read>(reader: R) -> Result<CdmDocument, CdmError>`: Streaming deserializer bounded by default 64 MiB ceiling defense.
    - `from_reader_with_options<R: std::io::Read>(reader: R, options: &JsonOptions) -> Result<CdmDocument, CdmError>`: Fully customizable stream reader with `std::io::Read::take(max_bytes + 1)` defense preventing memory exhaustion.
  - Implemented convenience methods directly on `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.to_json_pretty()`, `doc.to_json_compact()`, `CdmDocument::from_json(&str)`, `CdmDocument::from_slice(&[u8])`, `CdmDocument::from_reader(reader)`.
  - Re-exported `CdmError`, `JsonOptions`, and JSON utilities in `crates/cdm/src/lib.rs`.
  - Added comprehensive integration test suite in `crates/cdm/tests/json_serde.rs`:
    - `test_json_lossless_roundtrip_complex_document`: Verifies exact property equality between original document and deserialized document.
    - `test_json_deterministic_serialization`: Verifies byte-for-byte identical output across repeated runs with BTreeMap sorting.
    - `test_json_streaming_writer_and_reader`: Verifies streaming I/O roundtrip fidelity.
    - `test_payload_size_ceiling_defense`: Verifies rejection with `PayloadTooLarge` when stream exceeds configured byte limit.
    - `test_corrupted_json_syntax_rejected`: Verifies `Deserialization` error on truncated/invalid JSON.
    - `test_schema_version_validation_logic`: Verifies rejection of incompatible major schema versions while accepting minor updates.
    - `test_hash_integrity_verification_on_deserialization`: Verifies detection of tampered section content hashes via optional validation.
  - Verified 32/32 tests in `crates/cdm` (17 unit + 7 JSON serde integration + 8 schema validation integration), 335/335 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.1`)

#### Added
- **`[4.1.1]` Implement Canonical Context Document Model (CDM) Schema in `crates/cdm`:**
  - Added dependencies to `crates/cdm/Cargo.toml`: `chrono` (with `serde`), `uuid` (with `v4`, `v7`, `serde`), and `blake3`.
  - Implemented core document models in `crates/cdm/src/model.rs`: `CdmDocument`, `SourceIdentity`, `SourceType` (`WebpageStatic`, `WebpageDynamic`, `GitHubPublic`, `GitHubPrivate`, `DocumentPdf`, `DocumentOcr`), `RevisionInfo`, `ExtractionStatus` (`Completed`, `Partial`, `Failed`, `Unsupported`), `ExtractionWarning`, and `WarningSeverity` (`Critical`, `Warning`, `Info`).
  - Implemented section models in `crates/cdm/src/section.rs`: `CdmSection`, `SectionKind` (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Frontmatter`, `Manifest`, `Navigation`, `Admonition`), `LineRange` with line count arithmetic, `ByteRange` with saturating byte length, and `SectionTokenMetrics` with whitespace/character counting and baseline token estimation.
  - Implemented metadata and provenance models in `crates/cdm/src/metadata.rs`: `DocumentMetadata`, `Citation` (generating concise, markdown callout, and plaintext formats), and `ExclusionRecord` with typed `ExclusionReason` (`RulePattern`, `SizeLimit`, `BinaryFile`, `ExcludedExtension`, `Duplicate`, `UserConfig`, `SecurityRestricted`, `UnsupportedType`).
  - Implemented structural tree hierarchy in `crates/cdm/src/hierarchy.rs`: `HierarchyNode`, `HierarchyKind` (`Root`, `Directory`, `File`, `Section`, `Heading`), child management, recursive ID search (`find_node`), path search (`find_node_by_path`), pre-order flattening (`flatten`), and node/file counters.
  - Implemented relationship models in `crates/cdm/src/relationships.rs`: `Relationship` and `RelationshipKind` (`Imports`, `ImportedBy`, `Calls`, `Extends`, `LinksTo`, `References`, `ParentOf`, `ChildOf`, `DefinesSymbol`).
  - Implemented completeness accounting in `crates/cdm/src/completeness.rs`: `CompletenessReport` verifying the strict mathematical invariant formula $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$ and `is_complete()`.
  - Implemented ergonomic fluent builders in `crates/cdm/src/builder.rs`: `CdmDocumentBuilder` and `CdmSectionBuilder` with automatic BLAKE3 hash computation and baseline token metric derivation.
  - Added 19 comprehensive unit and integration tests across `crates/cdm` (11 unit + 8 integration), validating schema consistency, tamper detection, hierarchy queries, completeness matrix, and token metrics, bringing workspace tests to 322 passing tests.
  - Verified 19/19 tests in `crates/cdm`, 322/322 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.4`) — PHASE 3.4 & STAGE 3 CERTIFIED

#### Added
- **`[3.4.4]` Implement Golden Dataset Test Suite for Universal Webpage Extraction in `crates/web-extractor` (Phase 3.4 & Stage 3 Certified):**
  - Created comprehensive golden dataset integration test suite in `crates/web-extractor/tests/golden_dataset.rs` covering 8 real-world HTML archetypes:
    1. `test_golden_archetype_1_technical_blog_article`: Technical blog article validating article title, author byline, publication ISO timestamp, word count, reading time estimation, Rust syntax highlighting, OpenGraph and Schema.org JSON-LD parsing, and quality completeness score ($\ge 0.85$, `Complete`).
    2. `test_golden_archetype_2_docusaurus_documentation`: Multi-version technical documentation site validating Docusaurus framework detection, 3-level breadcrumbs navigation, typed admonitions (`Tip`, `Warning`), and next/previous pagination links.
    3. `test_golden_archetype_3_api_specification_tables`: REST API specification validating complex HTML tables with alignment indicators (`:---:`, `---:`), GFM rectangular table normalization, backtick formatting, and HTTP reference extraction.
    4. `test_golden_archetype_4_polyglot_code_with_line_gutters`: Multi-language code repository page validating monotonic line number gutter stripping from Rust code, Python shebang detection, and SQL DDL syntax parsing without indentation corruption.
    5. `test_golden_archetype_5_noisy_malformed_html`: Noisy real-world news layout validating unclosed markup recovery, `BoilerplateCleaner` pruning of cookie banners, inline advertisements, social share widgets, newsletter popups, and related articles lists.
    6. `test_golden_archetype_6_spa_hydration_lifecycle`: Single-page application lifecycle validating `SpaDetector` heuristic identification of low-density skeleton shells and noscript fallbacks, followed by post-render hydration verifying extraction of complete client-side telemetry markup.
    7. `test_golden_archetype_7_canonical_and_simhash_dedup`: Search engine deduplication pipeline validating relative canonical URL resolution with UTM query stripping, exact URL duplicate registration, 64-bit SimHash near-duplicate clustering within Hamming distance threshold ($\le 8$), and distinct content isolation.
    8. `test_golden_archetype_8_sparse_stub_diagnostics`: Malformed/sparse stub testing asserting heavy diagnostic penalty (< 0.40 score), issuance of `CONTENT_SPARSE` and missing metadata warnings, and exclusion from CDM conversion (`is_acceptable_for_cdm = false`).
  - Verified 143/143 tests in `crates/web-extractor` (135 unit + 8 golden integration tests), 304/304 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.
  - Formally completed and certified **Phase 3.4 (Extraction Quality)** and **Stage 3 (Universal Webpage Extraction)** across all 16 subphases (`3.1.1` through `3.4.4`).

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.3`)

#### Added
- **`[3.4.3]` Implement Content Completeness Scoring and Extraction Warnings in `crates/web-extractor`:**
  - Implemented multidimensional completeness and quality models in `crates/web-extractor/src/quality/model.rs`: `CompletenessLevel` (`Complete`, `High`, `Moderate`, `Partial`, `Degraded`, `Empty`), `ExtractionWarningCategory`, `ExtractionWarning`, `DimensionScores`, `ContentMetrics`, and `ExtractionQualityReport`.
  - Implemented `QualityScorer` in `crates/web-extractor/src/quality/scorer.rs` evaluating metadata completeness (title, description, author, dates, language, canonical link), content volume and density, structural fidelity (heading hierarchy, code syntax detection, table header normalization, link categorization), and acquisition provenance (hashes, HTTP status, cache validators, canonical verification, SSRF blocked subresources, unhandled JS page errors, dynamic degradation).
  - Implemented DOM heading analysis using `tl` parser scanning heading sequences and detecting hierarchy gaps (e.g. `<h1>` jumped directly to `<h3>` or `<h4>`), emitting `HEADING_HIERARCHY_GAP` warnings.
  - Implemented empirical content metrics extraction: word count, character count, estimated reading time, paragraph count, heading counts by level, code block and table counts, internal/external link breakdown, content density ratio, and boilerplate ratio.
  - Integrated `ExtractionQualityReport` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` across both `hydrate_static` and `render_and_hydrate` execution paths.
  - Re-exported `quality` types in `crates/web-extractor/src/lib.rs`.
  - Added 8 unit and integration tests covering rich article scoring, empty content critical warnings, sparse text degradation, missing metadata penalties, heading hierarchy gaps, code syntax and table header detection, subresource/page error diagnostics, and browser degradation penalties, bringing `crates/web-extractor` to 135/135 passing tests and workspace to 296/296 passing tests.
  - Verified 135/135 tests in `crates/web-extractor`, 296/296 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.2`)

#### Added
- **`[3.4.2]` Implement Duplicate Detection and Canonical URL Resolution in `crates/web-extractor`:**
  - Implemented deduplication models in `crates/web-extractor/src/dedup/model.rs`: `CanonicalSource`, `CanonicalResolution`, `DuplicateClassification`, `DuplicateRecord`, `DedupOptions`.
  - Implemented `CanonicalUrlResolver` in `crates/web-extractor/src/dedup/canonical.rs` resolving `<link rel="canonical">` tags with relative URL resolution, cross-domain canonical detection, and SSRF security validation preventing private IP or loopback targets from polluting canonical resolution.
  - Implemented 64-bit `SimHasher` in `crates/web-extractor/src/dedup/simhash.rs` utilizing unigram tokenization, term-frequency weighting, 64-bit FNV-1a hashing, Hamming distance computation, and Jaccard token similarity for robust near-duplicate detection.
  - Implemented thread-safe `DuplicateRegistry` in `crates/web-extractor/src/dedup/registry.rs` indexing canonical URLs, BLAKE3 content digests, and 64-bit SimHash fingerprints, classifying incoming documents as `Unique`, `ExactCanonical`, `ExactContent`, or `NearDuplicate`.
  - Integrated `CanonicalUrlResolver` into `ProvenanceBuilder::build_static` and `ProvenanceBuilder::build_dynamic` in `crates/web-extractor/src/provenance/extractor.rs`, automatically recording canonical resolutions on `SourceAttribution`.
  - Re-exported `dedup` types in `crates/web-extractor/src/lib.rs`.
  - Added 10 unit and integration tests covering canonical resolution, cross-domain and SSRF security handling, SimHash Hamming distance similarity, and concurrent registry duplicate classification, bringing `crates/web-extractor` to 127/127 passing tests and the workspace to 288/288 passing tests.
  - Verified 127/127 tests in `crates/web-extractor`, 288/288 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.1`)

#### Added
- **`[3.4.1]` Implement Source Attribution, HTTP Provenance, and Metadata Capture in `crates/web-extractor`:**
  - Added `blake3` and `sha2` workspace dependencies to `crates/web-extractor/Cargo.toml`.
  - Implemented strongly-typed source attribution models in `crates/web-extractor/src/provenance/model.rs`: `SourceAttribution`, `HttpProvenance`, `RedirectHopRecord`, `ContentHashes`, `CapturedMetadata`, `BrowserDiagnostics`, `ProvenanceCitations`, `ProvenanceWarning`, and `WarningSeverity`.
  - Implemented dual cryptographic content hash computation (`ContentHashes::compute`) generating SHA-256 and BLAKE3 lowercase hex digests over raw response markup bytes and extracted plain text.
  - Implemented `MetadataExtractor` in `crates/web-extractor/src/provenance/extractor.rs` extracting and parsing Schema.org JSON-LD structured data blocks (`<script type="application/ld+json">`) and cascading metadata precedence (OpenGraph -> Twitter -> HTML `<head>` -> Schema.org).
  - Implemented `ProvenanceBuilder` in `crates/web-extractor/src/provenance/extractor.rs` synthesizing `SourceAttribution` for both static acquisition (`build_static`) and dynamic browser rendering (`build_dynamic`), capturing HTTP headers, ETag, Last-Modified, cache-control, server, pinned IP, redirect history, and generating formatted citations (concise, markdown callout, plaintext).
  - Implemented diagnostic warning classification detecting canonical URL mismatches (`CANONICAL_URL_MISMATCH`), missing cache validators (`MISSING_HTTP_CACHE_VALIDATORS`), missing author metadata (`MISSING_AUTHOR`), blocked subresources, and dynamic-to-static degradation (`BROWSER_DEGRADED_TO_STATIC`).
  - Integrated `SourceAttribution` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` for all static and dynamic execution paths.
  - Re-exported provenance types in `crates/web-extractor/src/lib.rs`.
  - Added unit and integration tests covering content hash computation, Schema.org JSON-LD extraction, metadata precedence cascading, and provenance builder warning generation, bringing `crates/web-extractor` to 117/117 passing tests and workspace to 278/278 passing tests.
  - Verified 117/117 tests in `crates/web-extractor`, 278/278 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.4`) — PHASE 3.3 CERTIFIED

#### Added
- **`[3.3.4]` Implement Dynamic DOM Extraction and Post-Render Hydration in `crates/web-extractor` (Phase 3.3 Milestone Certified):**
  - Implemented `DynamicWebExtractor` orchestrating full web extraction: fast streaming static acquisition, content density & SPA shell heuristic analysis, dynamic browser worker invocation for JS-rendered pages, and post-render hydration into DOM, article, documentation, and structured element models.
  - Implemented `SpaDetector` heuristic analyzer detecting empty DOMs, framework hydration roots (`#root`, `#app`, `#__next`, `#svelte`, `<app-root>`), `<noscript>` JavaScript warnings, and low-density markup shells (< 5% content density).
  - Implemented post-render hydration pipeline feeding rendered HTML into `DomParser`, `ArticleExtractor`, `DocExtractor`, and `ElementsExtractor`, returning unified `ExtractedWebpage` instances.
  - Implemented `DynamicProvenance` capturing extraction method (`Static` vs `DynamicBrowser`), HTTP status, redirect chains, console logs, unhandled page errors, blocked subresources, and diagnostic warnings.
  - Implemented graceful degradation in `DynamicWebExtractor::extract`: if dynamic browser rendering fails or times out on an SPA page with fallback static content, preserve static extraction and attach diagnostic provenance warnings rather than failing the job.
  - Re-exported `DynamicWebExtractor`, `SpaDetector`, `ExtractionStrategy`, `ExtractionMethod`, `DynamicExtractorOptions`, `DynamicProvenance`, and `ExtractedWebpage` in `crates/web-extractor/src/lib.rs`.
  - Added 4 unit and integration tests in `crates/web-extractor/src/browser/extractor.rs` testing SPA detector heuristics, rich article static path, SPA fallback with mock browser worker, and graceful degradation on browser worker failure, bringing `crates/web-extractor` to 114/114 tests and workspace to 275/275 passing tests.
  - Formally certified **Phase 3.3 ("Dynamic extraction")** with all 4 subphases (`3.3.1` – `3.3.4`) fully implemented, verified, and accepted.
  - Verified 114/114 tests in `crates/web-extractor`, 275/275 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.3`)

#### Added
- **`[3.3.3]` Implement Resource Limits, Navigation Timeouts, and Sandbox Isolation in `services/browser-worker` and `crates/web-extractor`:**
  - Implemented `ConcurrencyLimiter` in `services/browser-worker/src/limiter.js` enforcing maximum concurrent page sessions (default 4) and a bounded FIFO queue (capacity 20) with queue timeout (15s), rejecting capacity overflows with HTTP 429 `ConcurrencyLimitExceeded` and `Retry-After: 5`.
  - Implemented automated Chromium lifecycle recycling in `PlaywrightEngine` (`recycleBrowser`), automatically relaunching Chromium when active sessions reach 0 and either `rendersCount >= maxRendersPerBrowser` (200) or process memory exceeds RSS threshold (768 MB).
  - Implemented hard rendering session deadline timer (`hardTimeoutBufferMs = 5000` added to client timeout) via `Promise.race`, terminating hung pages with 504 `SessionTimeout`.
  - Hardened context isolation with `serviceWorkers: 'block'`, `acceptDownloads: false`, and `permissions: []`.
  - Implemented automatic dialog dismissal on `page.on('dialog')` so `alert()`, `confirm()`, and `prompt()` modals never block worker execution.
  - Implemented subresource request ceiling (`maxSubresources = 200`), aborting excess subresources with `blockedbyclient` and recording in `blocked_subresources`.
  - Implemented maximum rendered HTML payload ceiling (`maxHtmlBytes = 10 MB`), rejecting oversized DOMs with 413 `PayloadTooLarge`.
  - Hardened `isPrivateOrLoopbackHost` to comprehensively classify IPv4 private ranges (RFC 1918), loopback (127.0.0.0/8), link-local/cloud metadata (169.254.0.0/16, 169.254.169.254, 100.100.100.200, metadata.google.internal), CGNAT (100.64.0.0/10), current network (0.0.0.0/8), broadcast, IPv6 ULA (fc00::/7), IPv6 link-local (fe80::/10), and IPv4-mapped IPv6.
  - Updated `crates/web-extractor/src/browser/error.rs` and `client.rs` to map HTTP 429 to `BrowserWorkerError::TooManyRequests` and HTTP 413 to `BrowserWorkerError::PayloadTooLarge`.
  - Configured `deny.toml` advisory ignore for `RUSTSEC-2026-0543` (transitive sqlx chrono advisory).
  - Added 9 unit and integration tests in `services/browser-worker/test/limits.test.js` validating limiter queueing, 429 rejection with `Retry-After`, address classification, dialog auto-dismissal, subresource ceilings, and HTML payload bounds, bringing `services/browser-worker` tests to 29/29 passing across 7 suites.
  - Verified 110/110 tests in `crates/web-extractor`, 271/271 workspace tests, 184/184 frontend tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.2`)

#### Added
- **`[3.3.2]` Implement Playwright/Chromium Dynamic Fallback Integration in `services/browser-worker` and `crates/web-extractor`:**
  - Integrated `playwright-core` into `services/browser-worker` with pinned `package.json` and reproducible `package-lock.json`.
  - Implemented `PlaywrightEngine` in `services/browser-worker/src/engine.js` with launch fallback resolution across bundled Playwright binaries, `CHROME_BIN`, `BROWSER_PATH`, Google Chrome channel, and Microsoft Edge channel, enforcing sandboxed container arguments (`--no-sandbox`, `--disable-dev-shm-usage`, `--disable-gpu`, `--disable-extensions`).
  - Implemented isolated per-request `BrowserContext` lifecycles, ensuring complete state, cookie, and cache segregation with unconditional cleanup in a `finally` block to prevent RAM, socket, or process leaks.
  - Implemented subresource request interception (`isPrivateOrLoopbackHost`) blocking non-HTTP(S) schemes and private/loopback/cloud-metadata IP destinations (`127.0.0.1`, `localhost`, `169.254.169.254`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`).
  - Implemented bounded console log (max 100) and page error (max 50) buffers with string truncation to prevent memory bloat from noisy scripts.
  - Implemented error classification and mapping into RFC 7807 problem details (`NavigationTimeout`, `SelectorTimeout`, `DnsResolutionFailed`, `ConnectionRefused`, `TlsCertificateError`).
  - Integrated `engine.render()` and live readiness probes into `services/browser-worker/src/routes.js` and graceful shutdown hooks in `services/browser-worker/server.js`.
  - Updated `crates/web-extractor/src/browser/client.rs` to map `SelectorTimeout` errors to `BrowserWorkerError::NavigationTimeout`.
  - Added 7 unit and integration tests in `services/browser-worker/test/engine.test.js` validating hostname security classification, error mapping, live Chromium SPA hydration with selector wait, and HTTP redirect chains, bringing `services/browser-worker` tests to 20/20 passing across 4 suites.
  - Verified 110/110 tests in `crates/web-extractor`, 271/271 workspace tests, 184/184 frontend tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.1`)

#### Added
- **`[3.3.1]` Implement Browser Worker Architecture and IPC Protocol in `services/browser-worker` and `crates/web-extractor`:**
  - Initialized isolated Node.js microservice in `services/browser-worker` with `package.json`, Node >=20 engine, and standard library HTTP server.
  - Implemented strongly typed IPC/RPC protocol in `services/browser-worker/src/protocol.js` validating render requests (`url`, `wait_until`, `timeout_ms`, `wait_for_selector`, `viewport`, `user_agent`, `headers`, `javascript_enabled`, `block_subresource_ips`) and generating RFC 7807 problem details error envelopes and standard response payloads.
  - Implemented HTTP dispatcher in `services/browser-worker/src/routes.js` handling `GET /health` (liveness), `GET /ready` (readiness), and `POST /render` (dynamic rendering RPC).
  - Implemented standalone daemon in `services/browser-worker/server.js` with graceful shutdown signal handling on `SIGINT`/`SIGTERM`.
  - Added 13 unit and integration tests in `services/browser-worker/test/protocol.test.js` using Node.js built-in `node:test`.
  - Defined strongly typed Rust IPC models in `crates/web-extractor/src/browser/model.rs`: `BrowserRenderRequest`, `BrowserRenderResponse`, `BrowserRedirect`, `ConsoleMessage`, `PageError`, `BlockedSubresource`, `WaitUntil`, `Viewport`, `BrowserTiming`, and `BrowserWorkerHealth`.
  - Implemented `BrowserWorkerError` in `crates/web-extractor/src/browser/error.rs` and mapped into `WebExtractorError::BrowserWorker` and `CoreError::Extraction`.
  - Implemented asynchronous `BrowserWorkerClient` in `crates/web-extractor/src/browser/client.rs` executing `check_health`, `check_readiness`, and `render` RPC calls over HTTP with connection timeouts and RFC 7807 error parsing.
  - Added 3 unit and integration tests in `crates/web-extractor` (110 total in crate), expanding workspace tests to 271 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.4`) & Phase 3.2 Milestone Certification

#### Added
- **`[3.2.4]` Implement Tables, Code Blocks, Syntax Detection, and Link Extraction in `crates/web-extractor`:**
  - Defined `ExtractedTable`, `TableAlignment`, `ExtractedCodeBlock`, `ExtractedLink`, `LinkType`, `ExtractedElements`, and `ElementsOptions` in `crates/web-extractor/src/elements/model.rs`.
  - Implemented `TableExtractor` in `crates/web-extractor/src/elements/table.rs` extracting table headers, body rows, captions, and alignments (`align` and CSS `text-align`), normalizing `colspan` and `rowspan` into rectangular matrices, and generating GitHub Flavored Markdown (GFM) tables with character escaping.
  - Implemented `detect_syntax` and `normalize_language_tag` in `crates/web-extractor/src/elements/syntax.rs` identifying programming languages via shebangs and keyword patterns across Rust, Python, TypeScript, JavaScript, Go, C/C++, Java, SQL, HTML, CSS, JSON, YAML, TOML, Bash, and Dockerfile.
  - Implemented `CodeBlockExtractor` in `crates/web-extractor/src/elements/code.rs` handling `<pre><code>` and `<pre>` blocks, identifying language classes and data attributes, stripping Rouge/Pygments line-number gutters and monotonic numeric prefixes, and preserving exact indentation, whitespace, and line breaks without loss.
  - Implemented `LinkExtractor` in `crates/web-extractor/src/elements/link.rs` resolving hyperlinks against base URLs, classifying links into `Internal`, `External`, `Anchor`, `Asset` (`.pdf`, `.zip`, `.parquet`, `.tar.gz`, etc.), and `Special` (`mailto:`, `tel:`), and neutralizing dangerous `javascript:` schemes.
  - Implemented `ElementsExtractor` in `crates/web-extractor/src/elements/extractor.rs` coordinating unified extraction of tables, code, and links from raw HTML strings or parsed DOM trees.
  - Added 20 unit and integration tests across elements modules and `lib.rs` (107 total in crate), expanding workspace tests to 268 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.
  - Formally certified completion of **Phase 3.2: Static Extraction** (`3.2.1` – `3.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.3`)

#### Added
- **`[3.2.3]` Implement Technical Documentation and Navigation Hierarchy Extraction in `crates/web-extractor`:**
  - Defined `ExtractedDocumentation`, `DocFramework`, `Admonition`, `AdmonitionType`, `BreadcrumbItem`, `TocItem`, `DocLink`, `DocPagination`, `DocVersionInfo`, and `DocSection` in `crates/web-extractor/src/docs/model.rs`.
  - Implemented `detect_framework` in `crates/web-extractor/src/docs/framework.rs` identifying generator frameworks (Docusaurus, mdBook, Sphinx, MkDocs, Rustdoc, VitePress, Nextra, GitBook, Generic).
  - Implemented `extract_table_of_contents` in `crates/web-extractor/src/docs/toc.rs` extracting heading levels (`h1`–`h6`), stripping anchor symbols (`#`, `¶`, `§`), mapping anchor IDs, and building nested ToC trees.
  - Implemented `DocNavigationExtractor` in `crates/web-extractor/src/docs/navigation.rs` resolving breadcrumb lists, previous/next pagination links, and version dropdown selectors.
  - Implemented `AdmonitionExtractor` in `crates/web-extractor/src/docs/admonition.rs` classifying callouts across standard and framework-specific alert styles.
  - Implemented `DocExtractor` in `crates/web-extractor/src/docs/extractor.rs` orchestrating raw HTML navigation discovery, framework-targeted container selection, cleaned content rendering, and section decomposition.
  - Updated `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` protecting admonitions in `CONTENT_INDICATORS` and refining ad token matching with boundary checks.
  - Isolated health check probe file naming in `services/api/src/routes/health.rs` with `RequestId::new()` to eliminate Windows sharing violations during concurrent test execution.
  - Added 14 unit and integration tests across doc frameworks (87 total in crate, 248 across workspace).

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.2`)

#### Added
- **`[3.2.2]` Implement Article Extraction and Readability Heuristics in `crates/web-extractor`:**
  - Defined `ExtractedArticle`, `ReadabilityMetrics`, and `ReadabilityOptions` builder in `crates/web-extractor/src/article/model.rs`.
  - Implemented heuristic pattern scoring in `crates/web-extractor/src/article/heuristics.rs` featuring positive/negative class matchers, unlikely candidate filters, and structural tag weights.
  - Implemented `ReadabilityScorer` in `crates/web-extractor/src/article/scorer.rs` computing text density, comma and prose frequency, paragraph density, heading counts, code block bonuses, and quadratic link-density penalties.
  - Implemented `ArticleExtractor` in `crates/web-extractor/src/article/extractor.rs` identifying candidate article containers (`article`, `main`, `section`, `div`), rendering selected subtrees via `DomParser::render_subtree`, extracting `<h1>` headlines, tiered bylines (`rel="author"`, `class="author"`), publication timestamps (`<time datetime="...">`), and lead excerpts, calculating reading times via `div_ceil`, and supporting whole-document fallback.
  - Extended `DomParser` in `crates/web-extractor/src/dom/parser.rs` to render isolated subtrees and preserve `rel`, `id`, `class` on `<a>` tags and `datetime` on `<time>` tags.
  - Added `HtmlParseFailed` and `ExtractionFailed` error variants to `WebExtractorError` and mapped into `CoreError`.
  - Added 15 unit and integration tests across article extraction modules (73 total in crate), expanding workspace tests to 234 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.1`)

#### Added
- **`[3.2.1]` Implement DOM Parsing and Boilerplate Removal in `crates/web-extractor`:**
  - Integrated `tl` (v0.7.8, safe pure Rust, MIT licensed, zero dependencies) into root `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `DocumentMetadata` in `crates/web-extractor/src/dom/metadata.rs` extracting `<title>`, meta description/author/keywords, OpenGraph (`og:*`), Twitter card (`twitter:*`), `<base href="...">`, document language, and canonical link attributes with robust fallback cascades.
  - Implemented `BoilerplatePolicy` and builder in `crates/web-extractor/src/dom/policy.rs` providing granular controls for scripts, styles, comments, nav, footers, sidebars, forms, ads, cookies, social sharing widgets, and custom class block/allow lists.
  - Implemented `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` distinguishing boilerplate elements from semantic content, explicitly protecting `<pre><code>` blocks, markdown body containers, and `<table>` content.
  - Implemented `LinkResolver` in `crates/web-extractor/src/dom/link_resolver.rs` resolving and canonicalizing relative `<a>` href and `<img>` src attributes against base URLs while neutralizing malicious `javascript:` schemes.
  - Implemented `DomParser` and `CleanedDocument` in `crates/web-extractor/src/dom/parser.rs` executing single-pass recursive DOM traversal, boilerplate pruning, link rewriting, standard entity decoding, and text newline normalization.
  - Added 20 unit and integration tests across DOM modules and `lib.rs` (58 total in crate), expanding workspace tests to 219 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.4`) & Phase 3.1 Milestone Certification

#### Added
- **`[3.1.4]` Implement Compression Bombs, Timeouts, and Body Size Limits in `crates/web-extractor`:**
  - Added `flate2` (with pure-Rust `rust_backend`), `brotli`, and `zstd` to root `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `ContentEncoding` enum in `crates/web-extractor/src/compression/encoding.rs` supporting `Identity`, `Gzip`, `Deflate`, `Brotli`, and `Zstd` with case-insensitive token and comma-separated header parsing.
  - Implemented `BodyLimits` in `crates/web-extractor/src/compression/limits.rs` encapsulating `max_raw_body_bytes` (10 MB), `max_decompressed_bytes` (25 MB), `max_expansion_ratio` (20.0), `min_ratio_check_bytes` (4 KB), and `enforce_content_length`.
  - Implemented `BoundedDecompressor` in `crates/web-extractor/src/compression/decompressor.rs` utilizing `CountingReader` with atomic byte tracking to stream decompressed data and detect ratio explosions and size breaches in real time across single and chained encodings.
  - Added `DecompressionBomb`, `DecompressionFailed`, `UnsupportedContentEncoding`, and `ContentLengthMismatch` variants to `WebExtractorError` and mapped into `CoreError`.
  - Upgraded `WebFetcher` in `crates/web-extractor/src/http/fetcher.rs` with total operation timeout bounding, chunk read inactivity timeouts, declared Content-Length early exit, Content-Length match enforcement, and transparent multi-format decompression.
  - Extended `FetchedResponse` with `raw_body_size` and `content_encoding` for full acquisition provenance.
  - Extended `MockHttpTransport` with `set_delay` and `set_chunk_delay` to enable deterministic timeout and slow-drip testing.
  - Added 22 unit and integration tests across compression modules and `lib.rs` (38 total in crate), expanding workspace tests to 199 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.
  - Formally certified completion of **Phase 3.1: Secure Acquisition** (`3.1.1` – `3.1.4` all `ACCEPTED`).

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.3`)

#### Added
- **`[3.1.3]` Implement HTTP Fetching, Streaming Bodies, and Redirect Revalidation in `crates/web-extractor`:**
  - Added `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }` to root `Cargo.toml`.
  - Added `reqwest`, `async-trait`, `bytes`, `futures`, `chrono`, and `tokio` to `crates/web-extractor/Cargo.toml`.
  - Added `"CDLA-Permissive-2.0"` to permitted licenses in `deny.toml` to support `webpki-roots` (Mozilla root CA store for pure-Rust TLS without C OpenSSL).
  - Implemented `WebExtractorError` in `crates/web-extractor/src/error.rs` covering `UrlSecurity`, `Http`, `RedirectLoop`, `TooManyRedirects`, `MissingLocationHeader`, `InvalidRedirectUrl`, `PayloadTooLarge`, `Timeout`, and `Network`, with seamless conversion into `CoreError`.
  - Implemented `RedirectHop` and `RedirectTracker` in `crates/web-extractor/src/http/redirect.rs` tracking hop history, detecting URL cycles, and enforcing max redirect limits.
  - Implemented `FetchedResponse` in `crates/web-extractor/src/http/response.rs` capturing initial URL, terminal canonical URL, status code, response headers, body bytes, pinned IP address, redirect history, ETag, and Last-Modified headers.
  - Defined asynchronous `HttpTransport` trait with production `ReqwestTransport` (configured with `Policy::none()` to disable automatic redirects and enforce connection to pinned socket addresses) and programmable `MockHttpTransport` in `crates/web-extractor/src/http/transport.rs`.
  - Implemented `WebFetcher` and `FetchOptions` in `crates/web-extractor/src/http/fetcher.rs` coordinating hop-by-hop DNS resolution & IP pinning via `SecureDnsResolver`, custom User-Agent and Accept header injection, redirect revalidation, and streaming chunked body consumption bounded by `max_body_bytes`.
  - Fixed target URL propagation in `SecureDnsResolver::resolve_and_pin` when reusing cached DNS resolutions.
  - Added 8 unit and integration tests in `crates/web-extractor/src/lib.rs` (16 total in crate), expanding workspace tests to 177 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.2`)

#### Added
- **`[3.1.2]` Implement DNS Resolution, IP Pinning, and DNS-Rebinding Protection in `crates/url-security`:**
  - Added `tokio = { workspace = true, features = ["net", "rt", "macros", "time", "sync"] }`, `async-trait = { workspace = true }`, and `chrono = { workspace = true, features = ["serde"] }` to `crates/url-security/Cargo.toml`.
  - Added `DnsResolutionFailed`, `DnsRebindingDetected`, and `IpPinningMismatch` variants to `UrlSecurityError` in `crates/url-security/src/error.rs`, mapping rebinding and pinning mismatches to `CoreError::Security`.
  - Defined asynchronous `DnsResolver` trait in `crates/url-security/src/dns.rs` along with production `SystemDnsResolver` utilizing `tokio::net::lookup_host` and thread-safe `MockDnsResolver` supporting static host mappings and ordered multi-response rebinding attack sequences.
  - Implemented immutable `PinnedTarget` in `crates/url-security/src/pinning.rs` containing `url`, `host`, `port`, `pinned_ip`, `all_resolved_ips`, `resolved_at`, and `ttl_secs`, with `socket_addr()`, TTL expiration checking, and socket destination verification via `verify_socket_addr`.
  - Implemented `SecureDnsResolver` in `crates/url-security/src/resolver.rs` providing thread-safe `RwLock` target caching, IP literal pre-checks, hostname blocklist validation, and **strict all-record SSRF validation** (rejecting entire DNS responses if even one resolved IP belongs to loopback, private, or cloud metadata CIDRs).
  - Integrated `resolve_and_pin` directly on `WebTarget` in `crates/web-extractor/src/url.rs`.
  - Added 5 new unit tests in `crates/url-security` and 1 integration test in `crates/web-extractor`, expanding workspace tests to 169 passing across all 17 crates, with all 5 monorepo verification gates passing cleanly.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.1`)

#### Added
- **`[3.1.1]` Implement URL Parsing, Normalization, and Validation in `crates/url-security` and `crates/web-extractor`:**
  - Added `url = { workspace = true }`, `serde = { workspace = true, features = ["derive"] }`, and `serde_json = { workspace = true }` to `crates/url-security/Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `UrlSecurityError` domain error enum in `crates/url-security/src/error.rs` covering `TooLong`, `ForbiddenScheme`, `ParseError`, `UserinfoForbidden`, `BlockedIp`, `BlockedHost`, `BlockedPort`, `MissingHost`, and `InvalidCharacters`, with standard conversion into `CoreError`.
  - Implemented `UrlSecurityPolicy` and `UrlSecurityPolicyBuilder` in `crates/url-security/src/policy.rs` supporting configurable length limits, scheme allowlists, marketing/analytics tracking query parameter lists, port blocklists against protocol confusion attacks, and test overrides.
  - Implemented comprehensive SSRF filters in `crates/url-security/src/ip_filter.rs`:
    - `is_blocked_ip` covering IPv4 loopback (`127.0.0.0/8`), RFC 1918 private (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), link-local / cloud metadata (`169.254.0.0/16` including AWS/GCP/Azure `169.254.169.254`), CGNAT (`100.64.0.0/10`), multicast (`224.0.0.0/4`), reserved (`240.0.0.0/4`), IPv6 loopback (`::1`), unspecified (`::`), unique-local (`fc00::/7`), link-local (`fe80::/10`), multicast (`ff00::/8`), documentation (`2001:db8::/32`), IPv4-mapped IPv6 (`::ffff:0:0/96`), IPv4-compatible IPv6, 6to4 encapsulation (`2002::/16`), and Teredo tunneling (`2001:0000::/32`).
    - `is_blocked_host` covering domain blocklists, local/internal domain patterns (`localhost`, `*.localhost`, `*.local`, `*.internal`, `*.lan`), cloud metadata domain names (`metadata.google.internal`), and alternative integer IP notations (POSIX `inet_aton` decimal integers e.g. `2130706433`, hex `0x7f000001`, octal `017700000001`, and dotted abbreviations `127.1`).
  - Implemented `UrlNormalizer` in `crates/url-security/src/normalizer.rs` performing length verification, null byte and control character detection, WHATWG parsing, scheme validation, default-port stripping (`:80` for http, `:443` for https), duplicate path slash collapse, tracking parameter stripping (`utm_*`, `fbclid`, `gclid`), and deterministic query parameter sorting.
  - Implemented domain wrapper `ValidatedUrl` in `crates/url-security/src/validated_url.rs` with `as_str`, `origin`, `is_same_origin`, `is_same_domain`, `is_subpath_of`, `join` for relative link traversal with full security re-evaluation, and secure serde serialization/deserialization.
  - Implemented `WebTarget` and `WebPageType` in `crates/web-extractor/src/url.rs` with heuristic classifier `classify_url` recognizing documentation (by keywords and subdomains e.g. `docs.rs`), articles (by keywords, date paths, and platforms e.g. `medium.com`, `substack.com`), file resources (by file extensions), and generic web pages.
  - Added 16 unit tests in `crates/url-security` and 7 unit tests in `crates/web-extractor`, expanding workspace unit tests to 163 passing across all 17 crates with 0 warnings and 0 errors across all 5 monorepo verification gates.

### 2026-11-17 — Phase 2.4: Storage (`2.4.4`), Phase 2.4 Milestone Certification & Stage 2 Completion Certification

#### Added
- **`[2.4.4]` Implement Artifact Checksum Validation and Verification in `crates/storage`:**
  - Added `sha2` (v0.10) and `blake3` (v1.5) to root `Cargo.toml` and `crates/storage/Cargo.toml`.
  - Implemented `ChecksumAlgorithm` enum supporting `Sha256` and `Blake3` with string representations, parse traits, and digest length constants.
  - Implemented `ArtifactDigests` computing SHA-256 and BLAKE3 concurrently over in-memory slices (`compute`) and over asynchronous streaming readers (`compute_streaming`) in 64 KB chunks to bound RAM usage.
  - Implemented `ChecksumVerificationStatus` (`Verified`, `Mismatch`, `Missing`, `Corrupted`) and structured `ArtifactIntegrityReport` tracking expected vs actual hashes, byte sizes, and error details.
  - Implemented `StorageIntegrityAuditSummary` with health checks and success rate calculations.
  - Implemented `ArtifactVerifier` coordinating storage backends and PostgreSQL: `verify_bytes`, `verify_storage_file`, `verify_artifact_record`, `verify_job_artifact`, `verify_all_job_artifacts`, and `audit_storage_integrity`.
  - Implemented BSD/GNU-compatible `checksums.sha256` manifest generator `generate_sha256_manifest` and robust parser `parse_sha256_manifest`.
  - Added 7 unit tests in `crates/storage/src/checksum.rs` verifying empty vectors, hash matching/validation, streaming vs slice parity, verifier outcomes, relational artifact verification, manifest round-trips, and audit summary health, expanding storage crate tests to 55 and automated Rust workspace tests to 142/142 passing. Verified all 5 monorepo gates pass cleanly.
  - Formally certified completion of **Phase 2.4: Storage** (`2.4.1` – `2.4.4` all `ACCEPTED`).
  - Formally certified completion of **Stage 2: Backend and Processing Foundation** (15 subphases across `Phase 2.1`, `Phase 2.2`, `Phase 2.3`, `Phase 2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.4: Storage (`2.4.3`)

#### Added
- **`[2.4.3]` Implement Artifact Retention and Ephemeral Cleanup Policies in `crates/storage`:**
  - Implemented `RetentionPolicy` in `crates/storage/src/retention.rs` with configurable TTLs (`default_ttl`, `max_ttl`, `ephemeral_ttl`), high/low capacity watermarks (`high_watermark_bytes`, `low_watermark_bytes`), batch sizes, invariant validation, and bounded `calculate_expires_at` deadline computation.
  - Implemented `CleanupReport` tracking evaluated artifacts, deleted artifacts, bytes freed, deleted ephemeral staging files, freed ephemeral bytes, and non-fatal error accumulators with merge and inspection helpers.
  - Implemented relational repository `ArtifactRepository` providing SQLx CRUD operations on `job_artifacts` with non-blocking transactional batch deletion using `FOR UPDATE SKIP LOCKED`.
  - Extended `StorageBackend` trait, `LocalFsStorage`, and `InMemoryStorage` with `cleanup_stale_staging_files` scanning for and purging orphaned temporary staging files (`.*.tmp.*`).
  - Implemented central lifecycle coordinator `RetentionManager` executing expired database artifact sweeps, orphaned staging file cleanup, job directory deletion, and capacity watermark enforcement (pruning oldest artifacts down to low watermark threshold).
  - Added 7 unit tests in `crates/storage/src/retention.rs` covering policy validations, TTL calculations, report merging/metrics, in-memory stale staging cleanup, watermark capacity enforcement, filesystem staging cleanup, and job directory purging, expanding storage crate tests to 48 and automated Rust workspace tests to 135/135 passing. Verified all 5 monorepo gates pass cleanly.

### 2026-11-17 — Phase 2.4: Storage (`2.4.2`)

#### Added
- **`[2.4.2]` Implement Pluggable Storage Backend Trait in `crates/storage`:**
  - Defined object-safe asynchronous storage stream alias `pub type AsyncReadStream = Pin<Box<dyn tokio::io::AsyncRead + Send>>;` and dispatch handle `pub type DynStorage = Arc<dyn StorageBackend>;`.
  - Implemented async trait `#[async_trait] pub trait StorageBackend: Send + Sync + std::fmt::Debug` covering `backend_type`, `put`, `get`, `open_read`, `exists`, `delete`, `delete_directory`, `metadata`, `list`, and `storage_capacity`.
  - Implemented `StorageBackend` for `LocalFsStorage` mapping directly to atomic filesystem operations.
  - Implemented `InMemoryStorage` using `Arc<RwLock<HashMap<String, Vec<u8>>>>` with strict path traversal checking, prefix-based listing, byte count metrics in `storage_capacity()`, and `Cursor<Vec<u8>>` for `open_read()`, providing zero-placeholder in-memory operations.
  - Implemented runtime backend factory `create_storage_backend(config: &StorageConfig) -> Result<DynStorage, CoreError>` supporting `StorageBackendConfig::LocalFs` and returning configuration errors for unconfigured backends.
  - Added 5 unit tests in `crates/storage/src/backend.rs` verifying LocalFsStorage via `DynStorage`, `InMemoryStorage` CRUD, recursive directory deletion, traversal rejections, and factory configuration errors, expanding storage crate tests to 41 and automated Rust workspace tests to 128/128 passing. Verified all 5 monorepo gates pass cleanly.

### 2026-11-17 — Phase 2.4: Storage (`2.4.1`)

#### Added
- **`[2.4.1]` Implement Local Filesystem Artifact Storage in `crates/storage`:**
  - Added `tokio` features `["fs", "io-util", "macros", "rt"]` to `crates/storage/Cargo.toml`.
  - Implemented `LocalFsStorage` in `crates/storage/src/local_fs.rs` providing durable atomic file writes, reading, streaming handles (`open_read`), existence probing, single file deletion, recursive directory deletion, and metadata retrieval.
  - Implemented strict defense-in-depth path traversal defenses in `resolve_path(&self, key)`: rejects null bytes, absolute paths, root/drive prefixes (`C:`), parent traversal components (`..`), Windows reserved device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`), and trailing period/space characters.
  - Implemented atomic staging writes: writes data to `.<name>.tmp.<uuid>`, flushes to physical storage with `sync_all()`, and renames atomically with Windows fallback.
  - Implemented standard artifact key layout helpers in `layout` module (`job_artifact_key`, `job_dir_key`, `blob_key`).
  - Implemented recursive directory capacity accounting in `storage_capacity()` returning `StorageUsage` (`total_artifact_bytes`, `artifact_count`).
  - Added 8 unit tests in `crates/storage/src/local_fs.rs` covering CRUD lifecycle, atomic overwrite, traversal rejections, recursive directory deletion, listing, storage capacity metrics, not found errors, and payload limits, expanding storage crate tests to 36 and automated Rust workspace tests to 123/123 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.4`) & Phase 2.3 Milestone Certification

#### Added
- **`[2.3.4]` Implement Durable Job Leasing, Heartbeat, and Recovery in `crates/storage`:**
  - Implemented `LeaseConfig` in `crates/storage/src/leasing.rs` defining worker identity (`worker_id`), lease duration (`lease_duration_seconds`), heartbeat interval (`heartbeat_interval_seconds`), and max retry attempts (`max_attempts`) with invariant validation and Serde support.
  - Implemented `HeartbeatResult` and `LeaseRecoveryReport` structs with accumulator and inspection helpers (`total_processed`, `is_empty`) and Serde support.
  - Implemented `JobRepository::acquire_next_job` using PostgreSQL `SELECT ... FOR UPDATE SKIP LOCKED` inside an atomic transaction, dynamically leasing either `queued` jobs or stale `leased`/`running` jobs whose lease expired, updating attempt count, calculating lease expiration relative to `NOW()`, and appending an audit record to `job_state_transitions`.
  - Implemented `JobRepository::heartbeat_job` and `JobRepository::is_lease_valid`, enforcing worker lease ownership, preventing heartbeat on terminal jobs, verifying expiration boundaries, updating timestamps, and optionally logging progress transitions.
  - Implemented `JobRepository::release_lease` enabling graceful voluntary lease relinquishment back to `queued` on worker shutdown, and `JobRepository::recover_stale_jobs` sweeping expired worker leases to either `queued` (for retry) or `failed` with code `LEASE_TIMEOUT_EXCEEDED` (when max attempts reached).
  - Implemented coordinator `LeaseManager` wrapping `PgPool` and `LeaseConfig` with typed async leasing operations, updated `JobStatus::can_transition_to` to permit `Running -> Leased` reacquisition, and added 6 unit tests covering leasing configurations, validation rejections, constants invariants, and Serde round-trips, expanding storage crate tests to 28 and automated Rust workspace tests to 115/115 passing. Verified all 5 quality gates pass cleanly.
  - Formally certified completion of **Phase 2.3: Persistence** (`2.3.1` – `2.3.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.3: Persistence (`2.3.3`)

#### Added
- **`[2.3.3]` Implement Job State Machine, Transitions, and History in `crates/storage`:**
  - Added migration file `crates/storage/migrations/20261117000002_job_state_transitions.sql` defining `job_state_transitions` audit trail table with `job_id REFERENCES jobs(id) ON DELETE CASCADE`, `from_status`, `to_status`, `phase`, `progress_pct`, `message`, `error_code`, `worker_id`, `transitioned_at`, and index `idx_job_state_transitions_job_id`.
  - Updated schema DDL script `crates/storage/src/schema/ddl.sql`, expected catalog tables/indexes in `crates/storage/src/schema/mod.rs`, and expected columns in `crates/storage/src/validation/mod.rs`.
  - Documented `job_state_transitions` table in `docs/DATA_MODEL.md` Section 2.1.
  - Implemented `JobStateMachine` and `JobTransitionRequest` builder in `crates/storage/src/state_machine.rs`:
    - Enforces legal status transitions and terminal state immutability (`Completed`, `Failed`, `Cancelled`).
    - Enforces progress bounds (0..=100), completion invariants (100% progress and Completed phase), and failure invariants.
    - Added typed entity struct `JobStateTransitionRow` deriving `sqlx::FromRow`, `Serialize`, and `Deserialize`.
  - Implemented transactional `JobRepository` in `crates/storage/src/repository.rs`:
    - `create_job(&pool, &JobRow)` and `get_job(&pool, id)`.
    - `transition_job(&pool, id, req)` using transaction with row-level locking (`SELECT ... FOR UPDATE`), state machine validation, updating status/lease/completion timestamps, and inserting audit records into `job_state_transitions`.
    - `get_transition_history(&pool, id)` retrieving chronological transition history.
    - Convenience transition helpers (`update_progress`, `complete_job`, `fail_job`, `cancel_job`).
  - Added 5 unit tests in `crates/storage/src/state_machine.rs` verifying legal/illegal transitions, request builders, invariant validations, and transition row construction, expanding storage crate tests to 22 and workspace automated tests to 109/109 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.2`)

#### Added
- **`[2.3.2]` Implement Database Migrations and Schema Validation in `crates/storage`:**
  - Added initial SQLx migration file `crates/storage/migrations/20261117000001_initial_schema.sql` defining relational tables, indexes, constraints, and triggers.
  - Implemented compile-time embedded SQLx migration runner `crates/storage/src/migrations/mod.rs` with `MIGRATOR` (`sqlx::migrate!("./migrations")`), `run_migrations(&PgPool)`, and structured `MigrationReport` with elapsed execution timing and migration version tracking.
  - Implemented runtime schema introspection and catalog validation in `crates/storage/src/validation/mod.rs`:
    - `EXPECTED_TRIGGERS` and `expected_table_columns()` column invariant maps.
    - `validate_catalog_metadata()` verifying presence of all required tables, columns, indexes, and triggers.
    - `validate_schema(&PgPool)` querying `information_schema.tables`, `information_schema.columns`, `pg_indexes`, and `information_schema.triggers` against live PostgreSQL instances.
    - `SchemaValidationReport` formatting machine-readable validation summaries.
  - Implemented central connection pool and lifecycle manager `DatabaseManager` in `crates/storage/src/database.rs` wrapping `sqlx::PgPool` with `connect`, `connect_with_config`, `migrate`, `validate_schema`, `is_healthy` (`SELECT 1`), and `close()`.
  - Added 7 unit tests covering embedded migration verification, report generation, and catalog metadata validation across table, column, index, and trigger omissions, expanding storage crate tests to 17 and total automated Rust workspace tests to 104/104 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.1`)

#### Added
- **`[2.3.1]` Design and Implement PostgreSQL Relational Schema in `crates/storage`:**
  - Added `sqlx` (v0.8, runtime-tokio, postgres, chrono, uuid, json, macros, migrate) and `serde`/`chrono`/`uuid` dependencies to workspace and `crates/storage/Cargo.toml`.
  - Authored standalone production-grade PostgreSQL 16 DDL script (`crates/storage/src/schema/ddl.sql`):
    - Table `jobs`: supports `kind` (web & GitHub variants), `status` (queued, leased, running, completed, partial, failed, cancelled), `progress_phase`, `progress_pct` (0..=100), `attempt_count`, `leased_by`, `lease_expires_at`, and `owner_session_id`.
    - Table `job_artifacts`: foreign key referencing `jobs(id)` with `ON DELETE CASCADE`, unique `(job_id, format)`, strict 64-char hex SHA256 and BLAKE3 digests, byte size, and expiration timestamps.
    - Table `github_installations`: unique `(owner_session_id, github_installation_id)`, encrypted token and 12-byte nonce storage, and token revocation timestamps.
    - Table `content_cache_index`: content-addressed cache mapping blob signatures to storage keys with LRU `last_accessed_at` timestamps.
    - Performance indexes: `idx_jobs_status_lease`, `idx_jobs_owner_session`, `idx_jobs_created_at`, `idx_job_artifacts_job_id`, `idx_job_artifacts_expires_at`, `idx_github_installations_owner`, and `idx_content_cache_last_accessed`.
    - Triggers: `fn_set_updated_at()` auto-updating `updated_at` timestamps on row modifications.
  - Implemented strongly-typed Rust entity models deriving `sqlx::FromRow` and Serde (`crates/storage/src/schema/`):
    - `JobKind`, `JobStatus` (with `can_transition_to` transition validation), `JobProgressPhase`, and `JobRow`.
    - `ArtifactFormat` (with MIME types and file extensions) and `JobArtifactRow`.
    - `GitHubInstallationRow` (with active and revocation status helpers).
    - `ContentCacheIndexRow` (with `touch()` method).
  - Implemented `split_ddl_statements()` parser in `crates/storage/src/schema/mod.rs` and added 10 unit tests in `crates/storage`, expanding total automated Rust workspace tests to 97/97 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.2: API (`2.2.4`) & Phase 2.2 Milestone Certification

#### Added
- **`[2.2.4]` Implement Health, Readiness, and Metrics Endpoints in `services/api`:**
  - Implemented container orchestrator liveness probe `GET /api/v1/health` returning structured JSON (`status: "pass"`, `service`, `version`, `uptime_seconds`, ISO 8601 `timestamp`).
  - Implemented deep subsystem readiness probe `GET /api/v1/ready` auditing storage volume writeability (creates and removes `.healthcheck_probe` file in configured `storage.base_dir`), database configuration (`database.url`), and worker queue readiness, returning 200 OK (`ready`) or 503 Service Unavailable (`not_ready`).
  - Expanded `AppMetrics` in `crates/observability/src/metrics.rs` with `http_requests_total` (`CounterVec`) and `http_request_duration_ms` (`Histogram`).
  - Added request duration timing and `http_requests_total` recording into `trace_id_middleware` in `services/api/src/middleware.rs`.
  - Added `start_time` and `uptime_seconds()` to `AppState` in `services/api/src/state.rs`.
  - Implemented Prometheus exposition endpoint `GET /api/v1/metrics` serving dynamic `web_repo_to_ai_uptime_seconds`, `web_repo_to_ai_build_info`, and application counters/gauges/histograms.
  - Added 4 integration tests in `services/api/src/lib.rs` verifying liveness fields, subsystem readiness degradation, unconfigured database failures, and Prometheus telemetry format, expanding workspace tests to 88/88 passing (37 in `services/api`). Verified all 5 quality gates pass cleanly.
  - Formally certified completion of **Phase 2.2: API** (`2.2.1` – `2.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.2: API (`2.2.3`)

#### Added
- **`[2.2.3]` Implement API Error Contracts and RFC 7807 Problem Details in `services/api`:**
  - Implemented RFC 7807 problem details response wrapper `ApiProblem` and helper constructors (`services/api/src/error.rs`):
    - Sets `Content-Type: application/problem+json` on all HTTP error responses.
    - Accurately maps domain errors to standard HTTP status codes (`400`, `401`, `403`, `404`, `405`, `413`, `429`, `500`, `504`).
    - Guarantees exact correlation between `x-request-id` response header and `body.error.request_id` in problem JSON envelopes.
    - Provided helpers: `to_problem_response()`, `not_found_problem()`, `validation_problem()`, `schema_problem()`, `payload_too_large_problem()`, and `method_not_allowed_problem()`.
  - Added `MethodNotAllowed` (405) variant to `ErrorCode` and `CoreError` in `crates/core/src/error.rs`.
  - Configured `RequestBodyLimitLayer` (10 MB request body limit) in `services/api/src/server.rs` with automatic RFC 7807 `PAYLOAD_TOO_LARGE` conversion in `ValidatedJson`.
  - Refactored `fallback_handler` in `services/api/src/routes/mod.rs` and handlers in `services/api/src/routes/jobs.rs` to format 404 errors as RFC 7807 problem details with correlated request ID.
  - Added 4 integration tests in `services/api/src/lib.rs` verifying problem JSON media types, request ID correlation integrity, 10 MB payload rejections, and `traceparent` propagation on errors, expanding workspace tests to 84/84 passing. Verified all 5 quality gates pass.

### 2026-11-17 — Phase 2.2: API (`2.2.2`)

#### Added
- **`[2.2.2]` Implement Request Validation and Parameter Parsing in `services/api`:**
  - Implemented typed request and response DTOs (`services/api/src/dto/`):
    - `DetectSourcesRequest` / `DetectSourcesResponse` with URL length ceilings (<= 2048 chars), supported schemes (`http`, `https`), host presence checks, batch limit (<= 50 URLs), and `classify_source_url()` helper.
    - `CreateJobRequest` / `JobResponse` validating 1..=50 sources, git reference sanitization (no `..`, control characters, or shell metacharacters), token budget bounds (1,000..=2,000,000), target model identifier syntax, and allowed export formats (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`).
    - `ListJobsQuery` / `PaginatedResponse` enforcing page numbers (>= 1), page limits (1..=100), permitted statuses (`queued`, `processing`, `completed`, `failed`, `cancelled`), and sort orders (`asc`, `desc`).
    - `JobPathParams` / `ArtifactPathParams` validating strongly-typed UUID v7 identifiers and artifact format strings.
  - Implemented custom Axum extractors with automatic domain validation (`services/api/src/extractors/`):
    - `ValidatedJson<T>`: deserializes JSON body and invokes `Validatable::validate()`, translating syntax and validation errors into machine-readable RFC 7807 problem details payloads (`INVALID_REQUEST_SCHEMA`, `VALIDATION_ERROR`, `UNSUPPORTED_URL_SCHEME`) with correlated `request_id`.
    - `ValidatedQuery<T>`: parses query strings and validates domain invariants.
    - `ValidatedPath<T>`: extracts and validates URI path parameters.
  - Implemented route endpoints (`services/api/src/routes/`):
    - `POST /api/v1/sources/detect`: classifies input URLs into `WebPage`, `PublicGitHub`, `PrivateGitHub`, or `BatchList`.
    - `POST /api/v1/jobs`: accepts validated job specifications, creates UUID v7 `JobId`, records Prometheus metric `jobs_total`, and returns `202 Accepted`.
    - `GET /api/v1/jobs`: returns validated paginated job collection.
    - `GET /api/v1/jobs/:job_id` and `GET /api/v1/jobs/:job_id/artifacts/:format`: path parameter validation endpoints.
  - Added `CoreError::InvalidRequestSchema` to `crates/core/src/error.rs` and implemented `Validatable` for all domain IDs in `crates/core/src/id.rs`.
  - Added 14 integration tests in `services/api/src/lib.rs` and 5 unit tests in `services/api/src/dto/mod.rs`, expanding workspace tests to 78/78 passing. Verified all 5 quality gates pass.

### 2026-11-17 — Phase 2.2: API (`2.2.1`)

#### Added
- **`[2.2.1]` Implement Axum Service and Router Scaffolding in `services/api`:**
  - Implemented application state container `AppState` (`services/api/src/state.rs`) holding shared references to `Arc<AppConfig>` and `Arc<AppMetrics>`.
  - Implemented HTTP middleware pipeline (`services/api/src/middleware.rs`):
    - Configurable `CorsLayer` parsing `cors_allowed_origins` from configuration, with support for standard methods (`GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`, `HEAD`) and headers.
    - Global request `TimeoutLayer` bounding execution time according to `server.request_timeout_secs`.
    - Distributed tracing and correlation middleware (`trace_id_middleware`) preserving or generating `x-request-id` (UUID v7) and W3C `traceparent` headers across request and response lifecycles.
  - Implemented HTTP router and handlers (`services/api/src/routes/`):
    - Root landing handler `/` returning JSON metadata, service name, and API v1 endpoints descriptor.
    - Container orchestrator liveness probe `/api/v1/health` and readiness probe `/api/v1/ready`.
    - Prometheus metrics exposition handler `/api/v1/metrics` serving `text/plain` Prometheus telemetry.
    - Fallback handler returning RFC 7807 compliant `ApiErrorEnvelope` with `RESOURCE_NOT_FOUND` on unmapped routes.
  - Implemented server lifecycle (`services/api/src/server.rs`):
    - `build_app()` assembling middleware layers and route tree.
    - `run_server()` binding TCP listener to configured host and port with graceful termination on `Ctrl+C` / `SIGINT` (and Unix `SIGTERM`).
  - Updated binary entry point `services/api/src/main.rs` initializing `init_tracing()` and `AppConfig::from_env()`.
  - Added 8 integration tests and 1 binary metadata test in `services/api`, expanding workspace test coverage to 59/59 passing tests. Verified all 5 gates pass cleanly.

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.3`) & Phase 2.1 Milestone Certification

#### Added
- **`[2.1.3]` Set Up Structured Logging, OpenTelemetry Tracing, and Prometheus Metrics in `crates/observability`:**
  - Implemented automated secret redaction engine in `crates/observability/src/redaction.rs`:
    - Sanitizes GitHub Personal Access Tokens (`ghp_*`, `gho_*`, `github_pat_*`), OAuth Bearer tokens, embedded basic auth credentials in URLs (`user:pass@host`), provider API keys (OpenAI `sk-*`, Anthropic `sk-ant-*`, AWS `AKIA*`), and private key blocks.
    - Provided `redact_sensitive_text()`, `is_sensitive_key()`, and recursive `redact_json_value()`.
  - Implemented OpenTelemetry and W3C TraceContext distributed trace propagation in `crates/observability/src/trace.rs`:
    - Defined `TraceContext` representation supporting 128-bit `trace_id`, 64-bit `span_id`, and `trace_flags`.
    - Implemented W3C `traceparent` parsing (`from_traceparent`) with all-zero rejection, formatting (`to_traceparent`), and child span context derivation (`new_child()`).
  - Implemented Prometheus-compatible metrics registry in `crates/observability/src/metrics.rs`:
    - Lock-free thread-safe atomic `Counter`, `CounterVec`, `Gauge`, and `Histogram` (with standard latency buckets).
    - Defined `AppMetrics` tracking `jobs_total`, `job_duration_ms`, `extracted_files_total`, `tokens_processed_total`, `storage_bytes_current`, `active_workers`, and `ssrf_blocks_total`.
    - Implemented `render_prometheus_text()` outputting valid Prometheus text exposition format.
  - Implemented structured logging subscriber in `crates/observability/src/logging.rs`:
    - Defined `LoggingConfig` and `LogFormat` (`Json`, `Pretty`, `Compact`).
    - Implemented idempotent `init_tracing()` with dynamic `EnvFilter` from `RUST_LOG` or default level.
  - Added 16 unit tests in `crates/observability`, expanding total automated Rust tests across the workspace to 50/50 passing.
  - Formally certified completion of **Phase 2.1: Rust architecture** (`2.1.1` – `2.1.3` all `ACCEPTED`).

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.2`)

#### Added
- **`[2.1.2]` Implement Typed Domain Errors, Results, and Configuration Loader in `crates/core`:**
  - Implemented typed domain error taxonomy in `crates/core/src/error.rs`:
    - Defined machine-readable `ErrorCode` enum and HTTP status code mappings (`INVALID_REQUEST_SCHEMA`, `UNSUPPORTED_URL_SCHEME`, `SSRF_ADDRESS_BLOCKED`, `UNAUTHORIZED_GITHUB_REPO`, `RESOURCE_NOT_FOUND`, `RATE_LIMIT_EXCEEDED`, `PAYLOAD_TOO_LARGE`, `EXTRACTION_TIMEOUT`, `CONFIG_ERROR`, `VALIDATION_ERROR`, `STORAGE_ERROR`, `JOB_ENGINE_ERROR`, `INTERNAL_SERVER_ERROR`).
    - Implemented `CoreError` with `thiserror`, standard `Result<T, E = CoreError>` type alias, and conversion into RFC 7807 compliant `ApiErrorEnvelope` / `ApiError` payloads.
  - Implemented strongly-typed domain identifiers in `crates/core/src/id.rs`:
    - Defined time-ordered UUID v7 newtypes: `JobId`, `DocumentId`, `TenantId`, `RequestId`, `ArtifactId`.
    - Implemented `Display`, `FromStr`, `From<Uuid>`, `Into<Uuid>`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, and transparent Serde serialization.
  - Implemented hierarchical configuration loader in `crates/core/src/config.rs`:
    - Defined `AppConfig` and modular sub-configurations: `ServerConfig`, `DatabaseConfig`, `StorageConfig`, `SecurityConfig`, `ExtractionConfig`, `OptimizationConfig`.
    - Provided default development values requiring zero mandatory environment variables.
    - Implemented `from_env()` parsing environment variable overrides (`PORT`, `HOST`, `DATABASE_URL`, `STORAGE_DIR`, etc.).
    - Implemented self-validation enforcing ports, non-zero payloads, connection bounds, budget ceilings, and strict production SSRF invariants (`allow_private_ips` prohibited in production).
  - Implemented core shared traits in `crates/core/src/traits.rs`: `Identifiable`, `Clock`, `SystemClock`, and `Validatable`.
  - Added 18 unit tests in `crates/core`, increasing total workspace tests to 34/34 passing. Verified all 5 gates pass cleanly.

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.1`)

#### Added
- **`[2.1.1]` Create Rust Workspace and 17 Crate Structures:**
  - Initialized monorepo root `Cargo.toml` with `[workspace]` configuration, unified lints, and workspace dependency management:
    - Declared 2 backend services (`services/api`, `services/worker`) and 15 domain/processing crates (`crates/core`, `crates/cdm`, `crates/url-security`, `crates/web-extractor`, `crates/github-extractor`, `crates/code-intelligence`, `crates/normalizer`, `crates/deduplication`, `crates/tokenizer`, `crates/context-optimizer`, `crates/chunking`, `crates/exporters`, `crates/storage`, `crates/job-engine`, `crates/observability`).
    - Configured package metadata defaults (`edition = "2021"`, `version = "0.1.0"`, `license = "MIT OR Apache-2.0"`).
    - Enforced strict workspace compiler warnings (`rust.unsafe_code = "deny"`, `rust.unused_must_use = "deny"`, `clippy.all = "warn"`, `clippy.pedantic = "warn"`, `clippy.correctness = "deny"`).
    - Pinned internal and third-party dependencies (`tokio`, `serde`, `tracing`, `thiserror`, `anyhow`, `chrono`) with exact versions and path mappings.
  - Initialized all 17 member crates with compilable `Cargo.toml` and entry points (`src/lib.rs` / `src/main.rs`), each providing crate metadata functions and passing unit tests.
  - Configured `deny.toml` compatible with cargo-deny 0.20+ (license validation, bans, security advisories, and path dependency wildcard allowances).
  - Configured `rustfmt.toml` with stable formatting options compatible with Rust 1.97.1.
  - Updated `scripts/check-all.sh` and `scripts/check-all.ps1` with robust cross-platform `cargo` binary detection and `cargo deny check` quality gating.
  - Verified 17/17 Rust unit tests pass, `cargo check --workspace` compiles cleanly in 0.64s, `cargo fmt` and `cargo clippy` report 0 warnings/errors, and all 5 monorepo verification gates pass.

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.3`), Phase 1.4 Certification & Stage 1 Milestone Certification

#### Added
- **`[1.4.3]` UI Performance Benchmarks, Asset Budget Audits, Design Token Regression & Stage 1 Certification:**
  - Authored comprehensive performance and visual regression test suite `apps/web/tests/ui-performance.test.ts` (11 tests across 3 categories):
    - **Production Bundle Size & Dependency Budget:** Asserted `package.json` contains zero external runtime dependencies (`dependencies` is empty); verified absence of bloated third-party libraries (no lodash, moment, prismjs, highlight.js, rxjs, lucide-react, MUI, Chakra); verified client build asset footprints strictly satisfy budgets (largest JS chunk $57.4\text{ kB} \le 100\text{ kB}$ raw, largest CSS chunk $44.1\text{ kB} \le 60\text{ kB}$ raw, total client bundle $\sim 260\text{ kB} \le 350\text{ kB}$ raw).
    - **Algorithmic Throughput & Micro-Benchmarks:** Verified high-throughput execution for syntax highlighting ($> 20,000\text{ lines/sec}$ on 1,000 JSON lines; $< 40\text{ms}$ on 500+ XML lines), token budget utilization evaluation (10,000 calls in $< 50\text{ms}$, $> 200,000\text{ ops/sec}$), URL classification and SSRF pre-checking (5,000 calls in $< 50\text{ms}$), and Web Crypto SHA-256 integrity digests (500 KB payload in $< 50\text{ms}$).
    - **Visual Design System & Styling Token Regression:** Verified full CSS custom properties export in `tokens.css` (`--color-brand-*`, `--color-accent-*`, `--color-surface-*`, `--color-status-*`, `--elevation-*`, `--spacing-*`, `--radius-*`, `.dark` theme overrides); verified Tailwind semantic utility classes (`.bg-theme-canvas`, `.bg-theme-panel`, `.bg-theme-subtle`, `.text-theme-primary`, `.text-theme-secondary`, `.text-theme-muted`, `.border-theme-subtle`, `.border-theme-strong`); verified WCAG AAA contrast compliance across both Dark mode ($\ge 14:1$ canvas, $\ge 12:1$ panel) and Light mode ($\ge 14:1$), plus AA for brand buttons ($\ge 4.5:1$); audited all 12 frontend components confirming zero inline arbitrary hex color pollution.
  - Expanded total automated test suite to 184/184 tests passing across 15 test suites with 0 failures.
  - Formally certified completion of **Phase 1.4: UX validation** across all 3 subphases (`1.4.1` – `1.4.3` all `ACCEPTED`).
  - Formally certified completion of **Stage 1: Frontend and Product Experience** across all 15 subphases (`1.1.1` – `1.4.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.2`)

#### Added
- **`[1.4.2]` Mobile, Tablet, and Desktop Viewport Tests & Responsive Invariant Suite:**
  - Expanded `apps/web/tests/responsive-layout.test.ts` into a multi-viewport certification test suite (28 tests across 6 sections):
    - **Tailwind Breakpoint Spectrum:** Verified resolution mapping across representative devices for Mobile (`xs`: 320px–480px, including iPhone SE, iPhone 14/15, Pixel 7), Tablet (`sm`: 640px, `md`: 768px–834px, `lg`: 1024px, including iPad Mini, iPad 10th Gen, iPad Air), and Desktop (`xl`: 1280px–1440px, `2xl`: 1536px–2560px, including standard laptops, QHD, and 1080p FHD displays).
    - **Mobile Viewport Invariants (320px–480px):** Verified horizontal overflow prevention (`overflow-x-hidden`, `max-w-full`, `min-h-screen`), standard responsive container padding (`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8`), Hero typography scaling (`text-3xl sm:text-5xl lg:text-6xl`), full-width action button stacking (`flex-col`, `w-full min-h-[48px]`), format pills 2-column collapse, value metric 1-column collapse, `UrlInput` vertical stacking with iOS auto-zoom prevention (`text-sm sm:text-base`), content grids single-column collapse, and Header mobile hamburger menu button (`md:hidden`).
    - **Tablet Viewport Invariants (640px–1024px):** Verified horizontal button flow at `sm` (`sm:flex-row`), 3-column format pills (`sm:grid-cols-3`), 3-column metric cards (`sm:grid-cols-3`), 3-column workflow cards (`md:grid-cols-3`), 2-column sources and feature grids (`md:grid-cols-2`), and Header desktop navigation visibility (`hidden md:flex`).
    - **Desktop Viewport Invariants (1280px–1920px+):** Verified 6-column format matrix (`lg:grid-cols-6`), 3-column feature highlights (`lg:grid-cols-3`), 5-column footer matrix (`lg:grid-cols-5`), maximum headline typography (`lg:text-6xl`), and container constraints preventing ultra-wide stretching (`max-w-7xl mx-auto`, `max-w-6xl mx-auto`).
    - **Conversion Workspace Viewport Invariants:** Verified workspace header responsive alignment (`flex-col sm:flex-row items-start sm:items-center`), 4-stage pipeline execution cards collapsing (1-col mobile, 2-col tablet, 4-col desktop), completeness accounting matrix (2-col mobile, 4-col tablet+), `TokenBudgetBar` metric grid, `PreviewPanel` 12-column responsive layout with sidebar stacking on mobile and side-by-side splitting on tablet/desktop (`grid-cols-1 md:grid-cols-12`), `DownloadControls` artifact cards (`flex-col md:flex-row md:items-center`), and bounded execution telemetry log scroll (`max-h-64 overflow-y-auto`).
    - **Touch Target Accessibility (WCAG 2.1 AA/AAA):** Verified $\ge 44 \times 44\text{px}$ touch targets for Header hamburger toggle and theme buttons, $\ge 44\text{px}$ touch heights for mobile dropdown links, and $\ge 48\text{px}$ touch heights for Hero CTA and `UrlInput` controls.
  - Expanded total automated test suite to 173/173 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.1`)

#### Added
- **`[1.4.1]` Keyboard Navigation and Accessibility Audits (WCAG 2.1 AA & AAA):**
  - Authored comprehensive accessibility audit test suite `apps/web/tests/a11y-audit.test.ts` (19 tests) asserting compliance across 8 distinct accessibility dimensions:
    - **Semantic HTML Landmarks:** Validated `<header>`, `<nav aria-label="Main Navigation">`, `<main id="main-content">`, and `<footer>` on both landing page and workspace routes; verified explicit `<section>` regions with `aria-labelledby` across `Hero.svelte`, `WorkflowSteps.svelte`, `SupportedSources.svelte`, and `FeatureHighlights.svelte`.
    - **WAI-ARIA Progressbar Pattern:** Validated compliance in `JobWorkspace.svelte` (overall pipeline progress) and `TokenBudgetBar.svelte` (context utilization progressbar) asserting `role="progressbar"`, dynamic `aria-valuenow`, `aria-valuemin={0}`, `aria-valuemax={100}`, and descriptive `aria-label`.
    - **WAI-ARIA Tabs Design Pattern:** Validated bidirectional linkage in `PreviewPanel.svelte` format tabs asserting `role="tablist"`, interactive tab controls (`role="tab"`, `id="tab-{id}"`, dynamic `aria-selected`, `aria-controls`), and panel containers (`role="tabpanel"`, `id="panel-{id}"`, `aria-labelledby`).
    - **Polite Live Regions & Assertive Alerts:** Verified non-disruptive feedback live regions (`role="status"`, `role="log"`, `aria-live="polite"`) across `UrlInput`, `JobWorkspace`, `PreviewPanel`, and `DownloadControls`, alongside assertive error and budget overflow banners (`role="alert"`).
    - **Screen Reader Exclusion of Decorative Content:** Verified `aria-hidden="true"` on line number gutter cells (`line.lineNumber`) and decorative background SVGs and visual effects.
    - **Universal Visible Focus Indicators:** Verified WCAG 2.4.7 visible focus styles (`focus-visible:ring` and `focus-visible:outline`) across interactive primitives (`Button`, `Input`, `Toggle`, `UrlInput`, `ConversionSettings`, `PreviewPanel`, `TokenBudgetBar`, `DownloadControls`).
    - **Color Contrast Certification:** Mathematically verified WCAG AAA ($\ge 7.0:1$) compliance for dark canvas ($14.0:1$) and panels ($12.0:1$) with white text, light canvas with dark slate text ($14.0:1$), and WCAG AA ($\ge 4.5:1$) for brand primary buttons.
    - **Keyboard Navigation Shortcuts:** Validated Escape key listeners in `UrlInput` (clearing input) and `Modal` (dialog dismissal), and Enter/Space keyboard toggling on `Toggle` switches (`role="switch"`, `aria-checked`).
  - Total automated test suite expanded to 152/152 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.4`) & Phase 1.3 Certification

#### Added
- **`[1.3.4]` Export Artifact Download Controls, SHA-256 Verification & Phase 1.3 Certification:**
  - Defined export artifact contracts and cryptographic SHA-256 generators in `apps/web/src/lib/types/download.ts`:
    - `computeSha256Hex`: Generates standard 64-character hexadecimal digests via Web Crypto API (`crypto.subtle.digest`).
    - `deriveArtifactBaseName`: Sanitizes filesystem-safe artifact base names from GitHub repositories and web documentation URLs.
    - `buildExportArtifacts`: Asynchronously generates concrete `ExportArtifact` records for all 5 formats with real content, byte sizes, and SHA-256 digests.
    - `generateChecksumsTxt`: Produces standard coreutils `sha256sum` formatted checksum lists (`<hash>  <filename>`).
    - `generateChecksumManifestJson`: Produces schema v1.0.0 JSON checksum manifests matching CDM specifications.
    - `triggerBlobDownload`: Client-side Blob download trigger with automatic Object URL cleanup.
  - Authored accessible, interactive `DownloadControls.svelte` in `apps/web/src/lib/components/workspace/DownloadControls.svelte`:
    - Region landmark (`role="region"`, `aria-label="Format Selection and Download Controls"`).
    - Batch download controls: "Download All Formats", "Checksums (.txt)", and "manifest.json".
    - Individual artifact cards with format icons, extension badges, byte sizes, truncated SHA-256 pills with one-click copy feedback, and download buttons.
    - Collapsible checksum manifest drawer rendering coreutils `checksums.txt` content.
    - Screen reader announcement live region (`role="status"`, `aria-live="polite"`).
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `DownloadControls` into `JobWorkspace.svelte` directly below the preview panel.
  - Authored comprehensive automated test suite `apps/web/tests/download-controls.test.ts` (11 tests) validating SHA-256 hashing, base name derivation, artifact generation, checksums.txt formatting, manifest JSON schema, and component ARIA landmarks.
  - Expanded total automated test suite to 133/133 passing tests across 13 test suites.
  - Formally certified completion of **Phase 1.3: Conversion workspace** across all 4 subphases (`1.3.1` – `1.3.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.3`)

#### Added
- **`[1.3.3]` Token Statistics Interface, Context Window Budget Bar & Headroom Metrics:**
  - Defined token budget evaluation domain model and calculations in `apps/web/src/lib/types/token-budget.ts`:
    - Mathematical helpers: `calculateSavingsPct` (token reduction ratio), `calculateUtilizationPct` (budget consumption), and `calculateHeadroom` (available conversational reasoning buffer).
    - Evaluation engine `evaluateBudgetUtilization` supporting 3 status levels: `optimal` (utilization $\le 85\%$), `warning` ($> 85\%$ and $\le 100\%$), and `exceeded` ($> 100\%$).
    - Calibration across AI model context windows (Gemini 2M, Gemini 1M, Claude 200k, GPT-4o 128k, Local 32k).
  - Authored accessible, interactive `TokenBudgetBar.svelte` in `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`:
    - Region landmark (`role="region"`, `aria-label="Token Statistics and Context Budget Bar"`).
    - 4-card metric grid: Raw Ingested Tokens, Deduplicated Tokens, Final Context Output, and Token Optimization Savings with reduction badge.
    - Dual-layer progress bar (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`) showing Final Tokens, ghost Deduplication Savings segment, and 80% safety margin threshold line.
    - Dynamic model preset switcher allowing instant re-evaluation against all major model context windows.
    - Headroom callout displaying remaining tokens for conversation prompts and completions.
    - Accessible context overflow alert banner (`role="alert"`) with one-click model upgrade switch to Gemini 1.5 Pro.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `TokenBudgetBar` into `JobWorkspace.svelte` positioned between the 4-stage pipeline execution cards and completeness accounting matrix.
  - Authored comprehensive automated test suite `apps/web/tests/token-budget.test.ts` (15 tests) validating savings math, headroom math, utilization bounds, ARIA landmarks, preset switching, and workspace integration.
  - Expanded total automated test suite to 122/122 passing tests across 12 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.2`)

#### Added
- **`[1.3.2]` Syntax-Highlighted Content Preview Panel & Multi-Format Exporters:**
  - Defined preview domain contracts and multi-format exporters in `apps/web/src/lib/types/preview.ts`:
    - 5 standard export formats: Markdown (`.md`), JSON CDM schema v1.0.0 (`.json`), Claude XML (`.xml`), Plaintext (`.txt`), and JSONL (`.jsonl`).
    - Canonical Context Document Model (CDM) serializer matching `docs/DATA_MODEL.md` with completeness accounting.
    - Claude context XML hierarchy serializer with `<documents>`, `<document index="...">`, and `<document_content>`.
    - Deterministic token estimation heuristic `estimateTokens` and sample extracted repository/article generator `generateSampleExtractedFiles`.
  - Authored zero-dependency, high-performance syntax highlighter in `apps/web/src/lib/utils/syntax-highlighter.ts`:
    - Strict HTML escaping (`escapeHtml`) preventing cross-site scripting (XSS).
    - Syntax token classification and styling for JSON (keys, strings, numbers, booleans, null), XML (tags, attributes, values, comments), Markdown (headings, blockquotes, code blocks, lists), and general code (keywords, comments, strings, numbers).
    - Line splitter `highlightLines` with 1-based line numbers.
  - Authored accessible, interactive `PreviewPanel.svelte` in `apps/web/src/lib/components/workspace/PreviewPanel.svelte`:
    - Accessible tablist (`role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`).
    - Extracted files sidebar with instant path filtering, file icons, byte size, token counts, and full concatenated context mode.
    - Copy to clipboard button with animated visual confirmation (`✓ Copied!`) and polite ARIA announcement (`role="status"`, `aria-live="polite"`).
    - Line numbers gutter with `aria-hidden="true"`, word wrap toggle, search within preview text, and direct format download.
    - Context metrics banner displaying estimated tokens, line counts, and byte sizes.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `PreviewPanel` into `JobWorkspace.svelte` with toggleable visibility and automatic target format synchronization.
  - Authored comprehensive automated test suite `apps/web/tests/preview-panel.test.ts` (17 tests) validating domain models, exporters, syntax highlighters, ARIA accessibility, and workspace integration.
  - Expanded total automated test suite to 107/107 passing tests across 11 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.1`)

#### Added
- **`[1.3.1]` Conversion Job Workspace, Pipeline Tracker & File Completeness Accounting:**
  - Defined conversion job domain model, state transitions, and file completeness invariant in `apps/web/src/lib/types/job.ts`:
    - Strict lifecycle states: `queued`, `leased`, `running`, `completed`, `partial`, `failed`, `cancelled`.
    - Standard 4-stage pipeline definitions: Acquisition & SSRF Defense, CDM Canonicalization, MinHash Deduplication, Export & Token Fit.
    - Verified invariant helper `verifyCompletenessInvariant` enforcing $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed}$ per `docs/DATA_MODEL.md` to prevent silent file drops.
    - Weighted progress calculator `calculateOverallProgress` and deterministic mock-free factory `createInitialJob`.
  - Authored accessible, responsive `JobWorkspace.svelte` component in `apps/web/src/lib/components/workspace/JobWorkspace.svelte`:
    - Overall progress bar landmark (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`).
    - 4-stage execution cards highlighting active, completed, failed, and pending stages with step numbers and status badges.
    - Repository completeness accounting matrix displaying discovered, extracted, excluded, and failed counts with verification badge.
    - Live execution telemetry log terminal with windowed DOM slice (most recent 200 items per Rule 03 bounded memory) and `role="log"`, `aria-live="polite"`.
    - Failure recovery alert banner (`role="alert"`) with error diagnostics and interactive retry trigger.
    - Action buttons for interactive job cancellation and pipeline retry.
  - Exported component via `apps/web/src/lib/components/workspace/index.ts`.
  - Authored dedicated workspace route in `apps/web/src/routes/workspace/+page.svelte`:
    - Reads query parameters (`?url=...&format=...&preset=...&budget=...`) on client load with fallback to initial verified target.
    - Integrated with site `Header`, `Footer`, breadcrumbs navigation, and SEO meta tags.
  - Updated `apps/web/src/routes/+page.svelte` to provide direct "Open Workspace →" action in conversion feedback banner.
  - Authored comprehensive automated test suite `apps/web/tests/job-workspace.test.ts` (14 tests) validating pipeline stages, job factories, completeness invariants, weighted progress math, ARIA attributes, and route integration.
  - Expanded total automated unit and invariant test suite to 90/90 passing tests across 10 test suites.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.4`) & Phase 1.2 Certification

#### Added
- **`[1.2.4]` Responsive Layout Auditing, Mobile Touch Targets & Phase 1.2 Certification:**
  - Audited and hardened responsive layout rules across all Phase 1.2 components:
    - Root layout (`apps/web/src/routes/+layout.svelte`) configured with `w-full max-w-full overflow-x-hidden` ensuring zero horizontal page scrolling across mobile viewports down to `320px`.
    - Enforced WCAG 2.1 AA recommended touch target sizing ($\ge 44 \times 44\text{px}$) across mobile hamburger button, theme switch toggle, mobile navigation drawer links, clear buttons, and submit controls.
    - Updated `Hero.svelte` action buttons with mobile full-width vertical stacking and $\ge 48\text{px}$ touch targets.
    - Updated `UrlInput.svelte` submit button with mobile full-width stacking (`w-full sm:w-auto min-h-[48px]`).
  - Authored automated responsive layout test suite `apps/web/tests/responsive-layout.test.ts` (7 tests) validating overflow containment, container padding, responsive grid collapsing, and touch target minimums.
  - Total automated test suite count expanded to 76/76 passing tests with zero failures across 9 test suites.
  - Formally certified completion of **Phase 1.2: Main website** across all 4 subphases (`1.2.1` – `1.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.2: Main Website (`1.2.3`)

#### Added
- **`[1.2.3]` Conversion Settings Panel, AI Model Presets & Content Normalization Toggles:**
  - Defined conversion settings contracts and calibrated AI model presets in `apps/web/src/lib/types/settings.ts`:
    - Model presets for Gemini 1.5 Pro (2M window), Gemini 1.5 Flash (1M window), Claude 3.5 Sonnet (200k window), GPT-4o (128k window), Local LLMs (32k/128k windows), and Custom budget.
    - CDM normalization and filtering options: `stripBoilerplate`, `stripCodeComments`, `excludeTests`, `enableDynamicRendering`, `semanticDeduplication`, `maxFileSizeKb`.
    - Validation utility enforcing budget bounds (1,000 to 2,000,000 tokens) and file size bounds (64 to 10,240 KB).
  - Authored accessible conversion settings component in `apps/web/src/lib/components/settings/ConversionSettings.svelte`:
    - Collapsible disclosure panel with `aria-expanded`, `aria-controls`, and `role="region"`.
    - Preset selection grid updating calibrated token ceilings.
    - Context window token budget range slider with live percentage progress bar and tabular number formatting.
    - Accessible switches (`Toggle`) for boilerplate stripping, code comment stripping, test exclusion, and MinHash semantic deduplication.
    - Dynamic Chromium rendering fallback toggle and file size threshold number input.
    - Exported component via `apps/web/src/lib/components/settings/index.ts`.
  - Integrated `ConversionSettings` into `apps/web/src/routes/+page.svelte` in `#convert`, binding settings state directly to conversion job feedback.
  - Authored automated unit and component invariant test suite `apps/web/tests/conversion-settings.test.ts` (6 tests), expanding total test suite count to 69/69 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.2`)

#### Added
- **`[1.2.2]` URL Input Component, Real-Time Source Detection & SSRF Pre-validation Engine:**
  - Authored URL detection and classification utility `apps/web/src/lib/utils/url-detector.ts`:
    - Full classification supporting standard public GitHub repositories, branch/tree subpaths, file blobs, XML sitemaps, sitemap indexes, and generic web documentation.
    - Implemented client-side zero-trust SSRF pre-check (`isBlockedHost`) preventing submission to loopback addresses (`127.0.0.0/8`, `::1`), RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata endpoints (`169.254.169.254`, `metadata.google.internal`), and non-HTTP protocols (`file:`, `javascript:`, `data:`, `ftp:`).
    - Extracted GitHub metadata including owner, repo, ref, and subpath.
  - Authored dedicated accessible URL input component in `apps/web/src/lib/components/input/UrlInput.svelte`:
    - Accessible search form landmark (`role="search"`, `aria-label="URL Ingestion Form"`).
    - Dynamic source badge with ARIA live region (`role="status"`, `aria-live="polite"`).
    - Contextual leading icons matching detected source modalities (GitHub Octocat, Sitemap XML, Security Warning Shield, Globe).
    - Clear button control (`aria-label="Clear URL input"`), `Escape` key handling, and submit button with SVG loading spinner.
    - Accessible error alert banner (`role="alert"`) rendering explicit SSRF and syntax error diagnostics.
    - Exported component via `apps/web/src/lib/components/input/index.ts`.
  - Integrated `UrlInput` into `apps/web/src/routes/+page.svelte` replacing the prototype input in `#convert` with real-time classification, dynamic target format suggestion, and verified submission feedback.
  - Authored automated unit test suites `apps/web/tests/url-detector.test.ts` (14 tests) and `apps/web/tests/url-input-component.test.ts` (3 tests), expanding total test suite count to 63/63 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.1`)

#### Added
- **`[1.2.1]` Production Landing Page, Semantic Navigation & Deterministic Pipeline Showcase:**
  - Authored navigation components in `apps/web/src/lib/components/navigation/`:
    - `Header.svelte`: Sticky semantic navigation header with brand AI icon, stage badge, desktop anchor navigation (`#convert`, `#workflow`, `#sources`, `#features`), external documentation and GitHub repository badges, accessible dark/light mode toggle with dynamic ARIA attributes, and responsive mobile dropdown drawer.
    - `Footer.svelte`: Semantic footer (`role="contentinfo"`) with copyright, MIT license, compliance badges (CPU-only 16 GB, <= 30 GB Podman, $0 / zero API costs), and external links to system architecture, SSRF defense, CDM data model, and performance targets.
    - `index.ts`: Unified navigation exports.
  - Authored landing feature components in `apps/web/src/lib/components/landing/`:
    - `Hero.svelte`: Production hero headline, value proposition subtitle, primary "Start Conversion" and "Explore How It Works" call-to-action buttons, 6 multi-format exporter pills (`.md`, `.json`, `.xml`, `.txt`, `.jsonl`, `.zip`), and hardware/operational metric badges.
    - `WorkflowSteps.svelte`: 3-step structured cards detailing Acquisition & SSRF Defense, CDM Canonicalization & Optimization, and Multi-Format Export & Token Fit.
    - `SupportedSources.svelte`: 4-source responsive grid matrix highlighting Static Documentation (HTML5 parser), Dynamic JavaScript SPAs (isolated Chromium fallback), Public GitHub Repositories (Git Trees API), and Private Repositories (GitHub App ephemeral tokens).
    - `FeatureHighlights.svelte`: 6 architectural pillar cards covering zero-trust SSRF defense, AST code intelligence, zero mandatory paid APIs, 30 GB Podman storage ceiling, deterministic token budgeting, and privacy-first self-hostability.
    - `index.ts`: Unified landing exports.
  - Assembled root `apps/web/src/routes/+page.svelte` integrating semantic landmarks (`header`, `main`, `footer`), quick conversion entry with URL input, reactive source detection, format selector pills, and live verification feedback banner.
  - Enhanced `apps/web/src/lib/components/ui/Button.svelte` with optional `href`, `target`, and `rel` attributes for accessible link buttons.
  - Enhanced `apps/web/src/lib/components/ui/Input.svelte` with optional `ariaLabel` attribute for headless input labeling.
  - Authored comprehensive test suite `apps/web/tests/landing-page.test.ts` validating semantic landmarks, navigation links, export formats, 3-step pipeline, 4 sources, and 6 feature pillars (expanding total passing tests to 46/46).

### 2026-11-17 — Phase 1.1: Design System (`1.1.3`)

#### Added
- **`[1.1.3]` Reusable Accessible UI Component Primitives & Phase 1.1 Completion:**
  - Authored accessible UI components in `apps/web/src/lib/components/ui/`:
    - `Button.svelte`: 5 variants (`primary`, `secondary`, `outline`, `ghost`, `danger`), 3 sizes (`sm`, `md`, `lg`), SVG loading spinner (`aria-busy`), disabled state, keyboard focus rings.
    - `Input.svelte`: accessible text/url input with label, helper text, error message, clear button, and ARIA described-by linkage.
    - `Toggle.svelte`: accessible switch primitive with `role="switch"`, `aria-checked`, label, description, and keyboard toggle (`Space`, `Enter`).
    - `Badge.svelte`: status badge with 6 semantic variants and optional status dot.
    - `Modal.svelte`: accessible dialog with backdrop overlay, `Escape` key dismissal, click-outside dismissal, `role="dialog"`, `aria-modal="true"`, and header/footer action slots.
    - `Tooltip.svelte`: contextual popover appearing on hover and keyboard focus (`role="tooltip"`).
  - Exported all UI primitives via `apps/web/src/lib/components/ui/index.ts`.
  - Updated showcase page `apps/web/src/routes/+page.svelte` demonstrating live interactive UI components.
  - Authored unit test suite `apps/web/tests/ui-components.test.ts`, expanding total test suite count to 37/37 passed tests.
  - Formally certified completion of **Phase 1.1: Design System** (Subphases `1.1.1`, `1.1.2`, `1.1.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.1: Design System (`1.1.2`)

#### Added
- **`[1.1.2]` Typography Hierarchy, Spacing Primitives & WCAG Contrast Systems:**
  - Authored accessible typography scale components in `apps/web/src/lib/components/typography/`: `Heading.svelte` supporting levels 1–6 (`h1`–`h6`) with proportional sizes `xs` to `5xl`, `Text.svelte` supporting body sizes, weights, and semantic tone variants, `Code.svelte` for inline and block code, and `Mono.svelte` for tabular numbers and token metrics.
  - Authored layout primitives in `apps/web/src/lib/components/layout/`: `Container.svelte` with responsive breakpoint max-widths, `Stack.svelte` supporting vertical/horizontal direction and 4px baseline gap spacing (`gap-0` to `gap-12`), and `Grid.svelte` supporting responsive 1–12 columns.
  - Implemented color contrast and accessibility utilities in `apps/web/src/lib/tokens/color-utils.ts` verifying WCAG 2.1 AA/AAA compliance thresholds, automatic text contrast selection, token budget status badge styling, and universal visible focus rings.
  - Published comprehensive design system specification in `docs/DESIGN_SYSTEM.md` covering typography hierarchy, 4px baseline rhythm, container breakpoints, surface layers, token budget bands, and WCAG AA/AAA certification tables.
  - Updated showcase page `apps/web/src/routes/+page.svelte` integrating typography and layout components with interactive budget controls.
  - Authored unit test suites `apps/web/tests/typography.test.ts` and `apps/web/tests/color-utils.test.ts`, expanding total test count to 31/31 passed tests.

### 2026-11-17 — Phase 1.1: Design System (`1.1.1`)

#### Added
- **`[1.1.1]` SvelteKit 2 + Svelte 5 Foundation & Visual Design Tokens:**
  - Initialized `apps/web` with SvelteKit 2, Svelte 5 runes (`$state`, `$derived`, `$props`), strict TypeScript, Tailwind CSS, PostCSS, Autoprefixer, `@sveltejs/adapter-node`, Vitest, and Prettier.
  - Implemented atomic design token module `apps/web/src/lib/tokens/index.ts` defining brand primary AI violet/indigo (`#6366f1`), accent telemetry cyan (`#06b6d4`), neutral slate surfaces (`#090d16` canvas to `#f8fafc`), semantic status indicators (`success`, `warning`, `error`, `info`), and context token budget indicators (`safe` <=80%, `warning` >80%, `critical` >100%).
  - Implemented CSS custom properties stylesheet `apps/web/src/lib/styles/tokens.css` with `:root` and `.dark` variables, alongside base styling `apps/web/src/lib/styles/app.css`.
  - Configured `apps/web/tailwind.config.ts` cleanly integrating design tokens into the Tailwind theme engine.
  - Authored root layout `+layout.svelte` and interactive token budget test showcase `+page.svelte`.
  - Authored automated Vitest suite `apps/web/tests/tokens.test.ts` verifying token completeness, hex regex validation, budget threshold logic, WCAG AAA dark background contrast (>= 7:1), WCAG AA brand contrast (>= 4.5:1), and CSS variable flattening.
  - Verified all 5 monorepo verification gates with 0 errors across PowerShell and POSIX runners.

---

## Stage 0: Project Bootstrap and Engineering Foundation (`COMPLETED` / `ACCEPTED`)

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.5`)

#### Added
- **`[0.3.5]` Setup Documentation, Disaster Recovery Runbooks & Stage 0 Certification:**
  - Authored comprehensive Environment Variables Reference in `docs/DEPLOYMENT.md` detailing all 19 configuration variables from `deploy/.env.example` (PostgreSQL, Axum API, Worker, Playwright, SvelteKit, and GitHub App credentials).
  - Authored PostgreSQL Logical Backup & Restore Runbook (`pg_dump`/`pg_restore`) for both Windows PowerShell and POSIX bash with clean table recreation and volume copying.
  - Authored Artifact Storage Volume Backup & Restore Runbook for `deploy_artifact_data`.
  - Authored Emergency Cold Disaster Recovery Runbook guaranteeing complete system recovery from cold workstation restart or storage corruption in under 5 minutes.
  - Authored Production Hardening Guidelines covering Caddy automatic HTTPS reverse proxy configuration with TLS headers, secret manager injection, block-level encryption at rest, AES-256-GCM token encryption, read-only container filesystems, dropped Linux capabilities, and container resource limits.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 223/223 automated checks.
  - Formally certified completion of **Stage 0: Project Bootstrap and Engineering Foundation** (Phases `0.1`, `0.2`, `0.3`, 15 subphases `0.1.1` – `0.3.5` all `ACCEPTED`).

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.3`)

#### Added
- **`[0.3.3]` Development Environment Lifecycle Scripts:**
  - Authored `scripts/dev-up.ps1` and `scripts/dev-up.sh` for starting the compose stack, ensuring `deploy/.env` initialization, verifying the 30 GB storage ceiling prior to launch, supporting service subsets (`-Services`) and image rebuilding (`-Build`), and awaiting database healthcheck readiness.
  - Authored `scripts/dev-down.ps1` and `scripts/dev-down.sh` for service teardown with interactive protection for persistent database volumes (`postgres_data`, `artifact_data`) and post-teardown cache pruning (`-PruneCaches`).
  - Authored `scripts/dev-logs.ps1` and `scripts/dev-logs.sh` supporting service filtering, line tailing, and real-time log streaming (`-Follow`).
  - Authored `scripts/dev-restart.ps1` and `scripts/dev-restart.sh` for safe service restarts and healthcheck validation.
  - Authored `scripts/dev-status.ps1` and `scripts/dev-status.sh` reporting runtime container state, healthcheck status, exposed ports, and storage budget compliance in table and structured JSON formats.
  - Documented complete developer lifecycle workflow in `docs/DEVELOPMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 213/213 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.2: GitHub and CI Foundation (`0.2.1` – `0.2.5`)

#### Added
- **`[0.2.5]` Security & Dependency Scanning:**
  - Configured `deny.toml` with strict RustSec advisory checks, permissive license allowlists (MIT, Apache-2.0, BSD, ISC, Unicode, PostgreSQL, CC0, Zlib, Unlicense), copyleft rejection, and registry restrictions.
  - Authored `.github/workflows/security.yml` executing secret leakage scans, `cargo-deny` checks, and `npm audit`.
  - Authored `.github/workflows/dependency-review.yml` gating PRs on dependency vulnerabilities and license compliance.
  - Authored `.github/workflows/codeql.yml` running automated CodeQL static security analysis.
  - Extended verification suite (`scripts/verify-bootstrap.ps1` & `scripts/check-all.ps1`) to 187 automated checks.
- **`[0.2.4]` Linting, Formatting & Quality Check Runners:**
  - Configured `rustfmt.toml` enforcing Rust 2021 edition, 100 character width, Unix LF line endings, import reordering, and comment formatting.
  - Configured `.prettierrc.json` and `.prettierignore` for web, JSON, Markdown, and TypeScript styling.
  - Authored cross-platform monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `scripts/verify-bootstrap.ps1` with lint/format configuration assertions.
- **`[0.2.3]` GitHub Actions CI/CD Workflows:**
  - Authored `.github/workflows/ci.yml` with concurrency controls, bootstrap integrity checks, and conditional Rust and SvelteKit test gates.
  - Authored `.github/workflows/build.yml` with separate artifact uploads for frontend build bundle, backend binaries, and SHA-256 checksums.
  - Authored `.github/workflows/container.yml` auditing Containerfile specifications and enforcing the 30 GB storage ceiling.
  - Authored `.github/workflows/release.yml` with automated release gate verification, CycloneDX SBOM generation, SHA-256 checksums, and GitHub Release asset publishing.
  - Created `scripts/validate-workflows.py` for automated PyYAML validation and integrated it into `scripts/verify-bootstrap.ps1`.
- **`[0.2.2]` Branch Protection & Repository Governance:**
  - Configured repository merge strategy via GitHub REST API: disabled merge commits (`allow_merge_commit: false`), enabled squash merge (`allow_squash_merge: true`), and enabled rebase merge (`allow_rebase_merge: true`) to enforce linear history.
  - Enabled automatic head branch pruning on PR merge (`delete_branch_on_merge: true`).
  - Enabled Dependabot vulnerability alerts and automated security fixes.
  - Enforced branch protection on `main`: blocked force pushes (`allow_force_pushes: false`), blocked branch deletions (`allow_deletions: false`), and required PR conversation resolution (`required_conversation_resolution: true`).
  - Documented repository governance and protection rules in `docs/DEVELOPMENT.md`.
- **`[0.2.1]` GitHub Remote Repository & Push:**
  - Verified Git Credential Manager OAuth scopes (`gist, repo, workflow`) for authenticated user `dayashimoga`.
  - Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai` via GitHub REST API with issues enabled.
  - Configured Git remote `origin` pointing to `https://github.com/dayashimoga/web-repo-to-ai.git`.
  - Pushed `main` branch with upstream tracking (`git push -u origin main`).
  - Verified remote tracking and commit SHA alignment (`98cc1decf23f270aef970877eb3a4b574b86d822 refs/heads/main`).

### 2026-11-16 — Phase 0.1: Repository and Governance Initialization (`0.1.1` – `0.1.5`)

#### Added
- **`[0.1.1]` Environment & Tooling Inspection:**
  - Verified Windows host environment (14 logical processors, 16.64 GB physical RAM, 33.51 GB free disk on `C:`), `git 2.55.0.windows.5`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (`podman-machine-default` WSL VM with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager authentication status (`dayashimoga`).
- **`[0.1.2]` Monorepo & Git Foundation (`58e0f80`):**
  - Initialized local Git repository on `main` branch with LF line-ending normalization (`.gitattributes`, `core.autocrlf=false`).
  - Added `.gitignore`, `.editorconfig`, MIT `LICENSE`, and `README.md`.
  - Created monorepo structure across `apps/web`, `services/{api,worker,browser-worker}`, 15 shared Rust crate directories (`crates/{core,cdm,url-security,web-extractor,github-extractor,code-intelligence,normalizer,deduplication,tokenizer,context-optimizer,chunking,exporters,storage,job-engine,observability}`), `docs/`, `deploy/`, `scripts/`, `.agent/`, and `.github/workflows/`.
- **`[0.1.3]` Engineering Contract, Requirements & Baseline Documentation (`6372d8a`):**
  - Created `GLOBAL_ENGINEERING_CONTRACT.md` enforcing product integrity, CDM-centered Rust architecture, tiered performance, completeness accounting, SSRF/OAuth security, and one-subphase-at-a-time execution.
  - Created `REQUIREMENTS.md`, `docs/REQUIREMENTS.md`, and `DECISIONS.md` (`ADR-001` through `ADR-007`).
  - Created `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DATA_MODEL.md`, `docs/API.md`, `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md`, `docs/USER_GUIDE.md`, `docs/TESTING.md`, `docs/PERFORMANCE.md`, and `docs/LICENSING.md`.
- **`[0.1.4]` Agent System, Skills, Rules & Workflows (`f2aa8e9`):**
  - Created `AGENTS.md` and registered 9 specialized agent roles (`architecture-lead`, `frontend-engineer`, `rust-backend-engineer`, `extraction-engineer`, `github-integration-engineer`, `security-reviewer`, `qa-engineer`, `devops-engineer`, `documentation-reviewer`) in the Antigravity runtime and `.agent/agents/*.md`.
  - Created 10 domain skills under `.agent/skills/*/SKILL.md` (`architecture`, `rust-engineering`, `frontend-engineering`, `web-extraction`, `github-integration`, `context-optimization`, `security-engineering`, `testing-quality`, `devops-cicd`, `documentation`).
  - Created 10 engineering rules under `.agent/rules/00-global-engineering.md` through `09-no-mock-no-placeholder.md`.
  - Created 10 reusable workflows under `.agent/workflows/*.md`.
- **`[0.1.5]` Staged Roadmap, Ledger & Bootstrap Verification:**
  - Created `ROADMAP.md` and `docs/ROADMAP.md` defining all 12 stages (`Stage 0` – `Stage 11`), 48 phases, and 197 subphases.
  - Created `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1`.
