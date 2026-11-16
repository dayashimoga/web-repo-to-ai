# Functional & Non-Functional Requirements Specification

**Project:** `web-repo-to-ai`  
**Version:** `1.0.0`  
**Status:** Baseline Approved (Stage 0)

---

## 1. Functional Requirements

### 1.1 Webpage Extraction (`FR-WEB`)

- **FR-WEB-01 (HTTP Fetching):** Fetch publicly accessible `http://` and `https://` URLs using asynchronous streaming HTTP with configurable timeouts, user-agent identification, gzip/brotli/deflate decompression, and strict byte-size caps.
- **FR-WEB-02 (Static HTML Parsing):** Parse HTML documents using `html5ever`/`scraper` into a structured DOM tree resilient to malformed markup.
- **FR-WEB-03 (Article & Documentation Extraction):** Extract primary content using readability heuristics and technical documentation layout selectors (removing navigation chrome, cookie banners, ads, and boilerplate footers while preserving `<main>`, `<article>`, headings, callouts, and sidebars where semantically relevant).
- **FR-WEB-04 (Tables, Code & Links):** Preserve HTML tables as structured Markdown/CDM tables, preserve `<pre><code>` blocks with detected language tags, and normalize relative hyperlinks to canonical absolute URLs.
- **FR-WEB-05 (Dynamic Rendering Fallback):** Automatically detect JavaScript-dependent shell pages (or honor explicit user toggle) and delegate rendering to `services/browser-worker` (Playwright + Chromium) with bounded wait conditions and DOM snapshot extraction.
- **FR-WEB-06 (Access Policy & Error Transparency):** Respect configurable robots/access policies and report HTTP errors (`401`, `403`, `404`, `429`, `5xx`), CAPTCHAs, paywalls, or blocked origins clearly without fabricating content.

### 1.2 Public GitHub Repository Extraction (`FR-GH-PUB`)

- **FR-GH-PUB-01 (URL & Target Resolution):** Parse GitHub URLs pointing to a repository root, branch, tag, commit SHA, directory path (`/tree/...`), or individual file (`/blob/...`).
- **FR-GH-PUB-02 (Metadata & Ref Discovery):** Retrieve repository metadata (default branch, license, description, topics, star/fork counts, latest commit SHA) and discover branches and tags.
- **FR-GH-PUB-03 (Complete Tree Acquisition):** Fetch the complete repository tree via GitHub Git Trees API (`?recursive=1`). Detect `truncated: true` responses and recover all subtrees via targeted subtree traversal or shallow Git archive/fetch fallback.
- **FR-GH-PUB-04 (File Classification & Filtering):** Classify every discovered path into `source_code`, `documentation`, `configuration`, `dependency_manifest`, `test_fixture`, `generated_or_lockfile`, `binary`, or `oversized`. Support user-configurable include/exclude globs and folder/file selection.
- **FR-GH-PUB-05 (Completeness Accounting):** Generate a verifiable extraction manifest where:
  $$\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$$
  No repository job may transition to `COMPLETED` if any eligible file is unaccounted for.

### 1.3 Private GitHub Repository Integration (`FR-GH-PRIV`)

- **FR-GH-PRIV-01 (GitHub App Authorization):** Authenticate private repository access via a GitHub App requesting only `Contents: Read-only` and `Metadata: Read-only` permissions.
- **FR-GH-PRIV-02 (OAuth & Installation Security):** Validate OAuth `state` with cryptographic nonces tied to the user session; exchange installation IDs for short-lived installation access tokens server-side only.
- **FR-GH-PRIV-03 (Encrypted Credential Storage):** Encrypt stored tokens at rest using authenticated encryption (`AES-256-GCM`) with keys injected via environment/secret store. Support immediate token revocation and expiration cleanup.
- **FR-GH-PRIV-04 (Strict Tenant Isolation & Redaction):** Enforce session/user ownership checks on all private repository jobs and artifacts; scan logs, errors, and exports to redact any token patterns (`ghp_`, `ghs_`, `ghu_`, `github_pat_`).

### 1.4 Canonical Context Document Model (`FR-CDM`)

- **FR-CDM-01 (Schema Completeness):** Represent every extracted source in a versioned `CdmDocument` (`v1.0`) containing:
  - `schema_version`, `document_id`, `source_identity`, `source_type`, `canonical_url`, `retrieved_at`, `revision` (commit SHA, branch/tag, ETag/Last-Modified)
  - `content_type`, `metadata`, `hierarchy`, `sections` (with `original_path`, `line_range`, `byte_range`, `language`, `content_hash`, `body`)
  - `relationships` (imports, links, symbol references), `citations`
  - `extraction_status` (`Completed`, `Partial`, `Failed`, `Unsupported`), `warnings`, `exclusions`, and `completeness_summary`.
- **FR-CDM-02 (Schema Versioning & Validation):** Validate invariants prior to serialization and maintain backward-compatible schema migration tests.

### 1.5 AI Optimization & Token Intelligence (`FR-OPT`)

- **FR-OPT-01 (Context Profiles):** Provide deterministic transformation profiles:
  1. `lossless` (exact clean extraction)
  2. `compact` (whitespace/boilerplate normalization without semantic loss)
  3. `general-ai` (balanced structure, file tree, high-signal docs + code)
  4. `code-review` (focus on implementation, interfaces, diffs, tests, and call relationships)
  5. `security-review` (prioritize auth, network, crypto, config, dependencies, secrets/sinks, entrypoints)
  6. `debugging` (focus on error paths, state transitions, stack-relevant modules, configs)
  7. `documentation` (READMEs, architecture docs, public API signatures, usage examples)
  8. `rag` (self-contained semantic chunks with full provenance headers)
- **FR-OPT-02 (Deduplication):** Support exact content-hash (`BLAKE3`) deduplication and near-duplicate/boilerplate block deduplication with provenance cross-references.
- **FR-OPT-03 (Tokenizer Adapters):** Provide exact token counts for supported open tokenizers (e.g., OpenAI `o200k_base` / `cl100k_base` via `tiktoken-rs` or equivalent BPE tables) and clearly labeled heuristic/approximate token estimates (with explicit uncertainty bounds) for proprietary tokenizers.
- **FR-OPT-04 (Budget Prioritization & Chunking):** Rank sections by profile relevance when a maximum token budget is specified, split oversized sections along AST/heading boundaries with configurable chunk size and overlap, and report every budget exclusion.

### 1.6 Multi-Format Exports (`FR-EXP`)

- **FR-EXP-01 (Output Formats):** Generate AI-ready Markdown (`.md`), Plain Text (`.txt`), Canonical/Structured JSON (`.json`), Semantic HTML (`.html`), RAG-ready JSON Lines (`.jsonl`), and downloadable ZIP archives (`.zip`).
- **FR-EXP-02 (Package Provenance & Integrity):** Every ZIP package must include the primary converted outputs plus `manifest.json`, `provenance.json`, `quality-report.json`, and `checksums.sha256`.

### 1.7 Web Frontend (`FR-UI`)

- **FR-UI-01 (Unified Input & Detection):** Automatically classify pasted single URLs, multiple batch URLs, public GitHub URLs, and private GitHub URLs.
- **FR-UI-02 (Interactive Configuration):** Allow users to select branch/tag/commit, interactive file/folder tree filters, context profile, target model/tokenizer, token budget, chunking parameters, and output formats.
- **FR-UI-03 (Real-Time Progress & Workspace):** Stream job phase updates via Server-Sent Events (SSE), display virtualized file trees and syntax-highlighted output previews, show original vs. optimized token metrics, and provide one-click copy and streaming downloads.

### 1.8 Batch, Crawling & Incremental Processing (`FR-SCALE`)

- **FR-SCALE-01 (Batch Processing):** Accept multiple URLs in a single batch job with bounded parallel execution, per-URL status tracking, and consolidated or individual exports.
- **FR-SCALE-02 (Bounded Website Crawling):** Support optional sitemap (`sitemap.xml`) and internal link discovery constrained by same-domain/path-prefix rules, maximum depth, and maximum page count.
- **FR-SCALE-03 (Incremental & Content-Addressed Caching):** Cache HTTP validators (`ETag`, `Last-Modified`) and Git blob SHAs so repeated or updated conversions re-process only changed content.

### 1.9 Developer & Agent Integrations (`FR-INT`)

- **FR-INT-01 (REST API):** Versioned JSON REST API (`/api/v1/...`) with OpenAPI specification, API key / session authentication, rate limiting, and SSE progress streaming.
- **FR-INT-02 (CLI):** Cross-platform Rust CLI binary supporting URL and repository conversion, local output verification, and exit-code contracts.
- **FR-INT-03 (MCP Server):** Model Context Protocol server exposing conversion tools and CDM resource retrieval over stdio/SSE.
- **FR-INT-04 (Browser Extension):** Manifest V3 browser extension foundation for sending the active tab URL or DOM snapshot to a configured `web-repo-to-ai` instance.

---

## 2. Non-Functional Requirements

### 2.1 Security (`NFR-SEC`)
- **NFR-SEC-01:** Block 100% of SSRF attempts targeting loopback, RFC1918, link-local, cloud metadata (`169.254.169.254`), IPv6 mapped private addresses, and DNS rebinding responses across initial requests and all redirect hops.
- **NFR-SEC-02:** Never execute untrusted code from target repositories or webpages inside the API or worker processes.
- **NFR-SEC-03:** Enforce configurable request rate limits, maximum single-file size (default 10 MB), maximum repository archive size (default 250 MB), and job timeout (default 300s).

### 2.2 Performance & Resource Limits (`NFR-PERF`)
- **NFR-PERF-01:** Run full stack within a 16 GB RAM, CPU-only machine.
- **NFR-PERF-02:** Maintain total Podman container and volume usage strictly below **30 GB**.
- **NFR-PERF-03:** Stream large artifact downloads and ZIP exports with bounded memory overhead ($O(1)$ buffer relative to total archive size).

### 2.3 Reliability & Quality (`NFR-QUAL`)
- **NFR-QUAL-01:** PostgreSQL-backed job leasing with heartbeat renewals and automatic recovery of stalled/interrupted jobs on worker restart.
- **NFR-QUAL-02:** Minimum 95% meaningful test coverage across critical core crates (`cdm`, `url-security`, `web-extractor`, `github-extractor`, `normalizer`, `deduplication`, `tokenizer`, `context-optimizer`, `chunking`, `exporters`).
- **NFR-QUAL-03:** Zero unresolved critical or high-severity CVEs in production dependencies; automated SBOM generation on release.
