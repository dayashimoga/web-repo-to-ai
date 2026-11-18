# PROJECT TODO TRACKER (`TODO.md`)

**Policy:** Append-only historical task tracker. Never delete or overwrite completed items; mark completed tasks with `[x]` and their completion date/subphase.

---

## Stage 0 — Project Bootstrap and Engineering Foundation (`COMPLETED` / `ACCEPTED`)

### Phase 0.1 — Repository and governance initialization
- [x] **[0.1.1]** Inspect environment, repository status, hardware budget, Podman machine limits, and available CLI tools *(Completed 2026-11-16)*
- [x] **[0.1.2]** Initialize Git repository (`main`), `.gitignore`, `.gitattributes`, `.editorconfig`, `LICENSE`, `README.md`, and 27 monorepo directories *(Completed 2026-11-16, commit `58e0f80`)*
- [x] **[0.1.3]** Create `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md`, and baseline `docs/*.md` specifications *(Completed 2026-11-16, commit `6372d8a`)*
- [x] **[0.1.4]** Create `AGENTS.md`, 9 specialized agent definitions, 10 `.agent/skills/*/SKILL.md`, 10 `.agent/rules/*.md`, and 10 `.agent/workflows/*.md` *(Completed 2026-11-16, commit `f2aa8e9`)*
- [x] **[0.1.5]** Create `ROADMAP.md`, `docs/ROADMAP.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1` *(Completed 2026-11-16)*

### Phase 0.2 — GitHub and CI foundation
- [x] **[0.2.1]** Create or connect GitHub remote repository (`web-repo-to-ai`) and push `main` branch *(Completed 2026-11-17, https://github.com/dayashimoga/web-repo-to-ai)*
- [x] **[0.2.2]** Configure branch protection and repository settings where authorized *(Completed 2026-11-17, linear history, Dependabot alerts/fixes, main protection)*
- [x] **[0.2.3]** Create executable GitHub Actions CI workflows (`ci.yml`, `build.yml`, `release.yml`, `container.yml`) *(Completed 2026-11-17, workflows created & validated on GitHub runners)*
- [x] **[0.2.4]** Configure linting, formatting, and test checks for Rust and Frontend *(Completed 2026-11-17, rustfmt.toml, .prettierrc.json, check-all.ps1/sh)*
- [x] **[0.2.5]** Configure security and dependency scanning workflows (`security.yml`, `dependency-review.yml`, `codeql.yml`, `deny.toml`) *(Completed 2026-11-17, deny.toml & security workflows active)*

### Phase 0.3 — Containerized development foundation (`COMPLETED` / `ACCEPTED`)
- [x] **[0.3.1]** Create Podman development configuration (`deploy/podman-compose.yml` and multi-stage Containerfiles) *(Completed 2026-11-17, podman-compose.yml & 4 Containerfiles)*
- [x] **[0.3.2]** Enforce the 30 GB storage ceiling (`scripts/check-resources.*`, `scripts/clean-caches.*`) *(Completed 2026-11-17, storage monitoring & safe cache pruning)*
- [x] **[0.3.3]** Create development environment startup, test, and recovery scripts (`scripts/dev-*.*`) *(Completed 2026-11-17, cross-platform dev lifecycle scripts)*
- [x] **[0.3.4]** Validate reproducible container builds (`scripts/verify-container-builds.*`) *(Completed 2026-11-17, multi-stage audit, unprivileged users, healthchecks)*
- [x] **[0.3.5]** Document container setup, resource management, and recovery in `docs/DEVELOPMENT.md` and `docs/DEPLOYMENT.md` *(Completed 2026-11-17, env vars table, disaster recovery runbook, hardening)*

---

## Stage 1 — Frontend and Product Experience (`COMPLETED` / `ACCEPTED`)

### Phase 1.1 — Design system (`COMPLETED` / `ACCEPTED`)
- [x] **[1.1.1]** Scaffold SvelteKit + Svelte 5 + TypeScript + Tailwind CSS application in `apps/web` and establish visual design token system *(Completed 2026-11-17, brand/accent/surface palettes, token budget indicators, tokens.css, Tailwind theme)*
- [x] **[1.1.2]** Implement typography scale, spacing system, and accessible color contrast utilities *(Completed 2026-11-17, Heading/Text/Code/Mono, Container/Stack/Grid, color-utils, docs/DESIGN_SYSTEM.md)*
- [x] **[1.1.3]** Build reusable accessible UI component primitives (Button, Input, Toggle, Badge, Modal, Tooltip) *(Completed 2026-11-17, Button/Input/Toggle/Badge/Modal/Tooltip, ARIA roles, 37 passing tests)*

### Phase 1.2 — Main website (`COMPLETED` / `ACCEPTED`)
- [x] **[1.2.1]** Implement landing page with conversion flow *(Completed 2026-11-17, Header, Hero, WorkflowSteps, SupportedSources, FeatureHighlights, Footer, 46 passing tests)*
- [x] **[1.2.2]** Implement URL input with instant source detection *(Completed 2026-11-17, UrlInput, url-detector, SSRF pre-check, 63 passing tests)*
- [x] **[1.2.3]** Implement conversion settings panel *(Completed 2026-11-17, ConversionSettings, model presets, budget slider, content filters, 69 passing tests)*
- [x] **[1.2.4]** Implement responsive layouts (mobile, tablet, desktop) *(Completed 2026-11-17, mobile touch targets >=44px, overflow containment, 76 passing tests, Phase 1.2 certified)*

### Phase 1.3 — Conversion workspace (`COMPLETED` / `ACCEPTED`)
- [x] **[1.3.1]** Build conversion job workspace *(Completed 2026-11-17, JobWorkspace.svelte, job domain model, completeness invariant matrix, 4-stage pipeline tracker, 90 passing tests)*
- [x] **[1.3.2]** Build syntax-highlighted content preview panels *(Completed 2026-11-17, PreviewPanel.svelte, 5 export formats, safe syntax highlighter, line numbers gutter, copy/download, 107 passing tests)*
- [x] **[1.3.3]** Build token statistics interface and budget bar *(Completed 2026-11-17, TokenBudgetBar.svelte, context utilization math, savings percentage, dual-layer progress bar, 122 passing tests)*
- [x] **[1.3.4]** Build format selection and download controls *(Completed 2026-11-17, DownloadControls.svelte, SHA-256 integrity, batch downloads, checksums manifest, 133 passing tests, Phase 1.3 certified)*

### Phase 1.4 — UX validation (`COMPLETED` / `ACCEPTED`)
- [x] **[1.4.1]** Execute keyboard navigation and accessibility audits *(Completed 2026-11-17, WCAG 2.1 AA/AAA compliance, 8 audit sections, 152 passing tests)*
- [x] **[1.4.2]** Execute mobile, tablet, and desktop viewport tests *(Completed 2026-11-17, multi-viewport certification suite, 6 test sections, 173 passing tests)*
- [x] **[1.4.3]** Execute UI performance and visual regression verification *(Completed 2026-11-17, bundle size budgets, algorithmic micro-benchmarks, design token regression, 184 passing tests, Phase 1.4 and Stage 1 Certified)*

---

## Stage 2 — Backend and Processing Foundation (`COMPLETED` / `ACCEPTED`)

### Phase 2.1 — Rust architecture (`COMPLETED` / `ACCEPTED`)
- [x] **[2.1.1]** Create Rust workspace and crate structure *(Completed 2026-11-17, root Cargo.toml, 17 workspace member crates/services, deny.toml 0.20+, rustfmt, 17/17 tests passing, all 5 gates pass)*
- [x] **[2.1.2]** Implement typed domain errors, results, and configuration loader in `crates/core` *(Completed 2026-11-17, CoreError, ErrorCode, ApiErrorEnvelope, strongly-typed UUID v7 domain IDs, AppConfig with env/validation, Clock/Identifiable traits, 18 passing tests, 34 workspace tests)*
- [x] **[2.1.3]** Set up structured logging, OpenTelemetry tracing, and Prometheus metrics in `crates/observability` *(Completed 2026-11-17, secret redaction engine, W3C TraceContext propagation, atomic Prometheus AppMetrics, LoggingConfig with JSON/compact tracing subscriber, 16 passing tests, 50 workspace tests, Phase 2.1 Certified)*

### Phase 2.2 — API (`COMPLETED` / `ACCEPTED`)
- [x] **[2.2.1]** Implement Axum service and router scaffolding *(Completed 2026-11-17, AppState, CORS/timeout/trace middleware, health/metrics routes, RFC 7807 404 fallback, graceful shutdown, 9 passing tests, all 5 gates pass)*
- [x] **[2.2.2]** Implement request validation and parameter parsing *(Completed 2026-11-17, DTOs for sources/jobs/query/params, ValidatedJson/ValidatedQuery/ValidatedPath extractors, RFC 7807 validation error mapping, 28 passing tests, all 5 gates pass)*
- [x] **[2.2.3]** Implement API error contracts and RFC 7807 problem details *(Completed 2026-11-17, ApiProblem responder, application/problem+json media type, MethodNotAllowed, RequestBodyLimitLayer 413, correlation integrity, 33 passing tests, all 5 gates pass)*
- [x] **[2.2.4]** Implement health, readiness, and metrics endpoints *(Completed 2026-11-17, liveness probe /health, deep readiness probe /ready auditing storage writeability and database URL, Prometheus /metrics exposition with dynamic uptime/build info/HTTP metrics, 37 passing tests, Phase 2.2 Certified)*

### Phase 2.3 — Persistence (`COMPLETED` / `ACCEPTED`)
- [x] **[2.3.1]** Design and implement PostgreSQL relational schema in crates/storage *(Completed 2026-11-17, DDL schema for jobs/artifacts/installations/cache, SQLx FromRow entity models, 10 passing storage tests, 97 workspace tests)*
- [x] **[2.3.2]** Implement database migrations and schema validation in crates/storage *(Completed 2026-11-17, embedded SQLx migrator, MigrationReport, validate_schema catalog verification, DatabaseManager, 17 passing storage tests, 104 workspace tests)*
- [x] **[2.3.3]** Implement job state machine, transitions, and history in crates/storage *(Completed 2026-11-17, JobStateMachine transition validation, terminal state immutability, job_state_transitions migration, JobRepository row-level locked transitions, 22 passing storage tests, 109 workspace tests)*
- [x] **[2.3.4]** Implement durable job leasing, heartbeat, and recovery in crates/storage *(Completed 2026-11-17, FOR UPDATE SKIP LOCKED worker queue polling, heartbeat lease extension, voluntary lease release, automated stale recovery, LeaseManager, 28 passing storage tests, 115 workspace tests, Phase 2.3 Certified)*

### Phase 2.4 — Storage (`COMPLETED` / `ACCEPTED`)
- [x] **[2.4.1]** Implement local filesystem artifact storage in crates/storage *(Completed 2026-11-17, LocalFsStorage, atomic staging writes, path traversal guards, layout helpers, storage_capacity accounting, 36 passing storage tests, 123 workspace tests)*
- [x] **[2.4.2]** Implement pluggable storage backend trait in crates/storage *(Completed 2026-11-17, StorageBackend trait, DynStorage, AsyncReadStream, LocalFsStorage impl, InMemoryStorage impl, create_storage_backend factory, 41 passing storage tests, 128 workspace tests)*
- [x] **[2.4.3]** Implement artifact retention and ephemeral cleanup policies in crates/storage *(Completed 2026-11-17, RetentionPolicy, CleanupReport, ArtifactRepository with FOR UPDATE SKIP LOCKED, RetentionManager with expired DB sweep, staging file purge, and watermark capacity enforcement, 48 passing storage tests, 135 workspace tests)*
- [x] **[2.4.4]** Implement artifact checksum validation and verification in crates/storage *(Completed 2026-11-17, ArtifactDigests dual SHA-256 and BLAKE3, streaming digester in 64 KB buffers, ArtifactVerifier, checksums.sha256 manifest generator and parser, 55 passing storage tests, 142 workspace tests, Phase 2.4 & Stage 2 Certified)*

---

## Stage 3: Universal Webpage Extraction (`COMPLETED` / `ACCEPTED` — **STAGE 3 CERTIFIED**)

### Phase 3.1 — Secure acquisition (`COMPLETED` / `CERTIFIED`)
- [x] **[3.1.1]** URL parsing, normalization, and validation in crates/url-security and crates/web-extractor *(Completed 2026-11-17, UrlSecurityError, UrlSecurityPolicy, is_blocked_ip, is_blocked_host, parse_alternative_ipv4, UrlNormalizer, ValidatedUrl, WebTarget, WebPageType, 16 passing url-security tests, 7 passing web-extractor tests, 163 workspace tests)*
- [x] **[3.1.2]** DNS resolution, IP pinning, and DNS-rebinding protection in crates/url-security *(Completed 2026-11-17, DnsResolver trait, SystemDnsResolver, MockDnsResolver, PinnedTarget, SecureDnsResolver with RwLock caching, all-record SSRF validation, anti-rebinding tests, 21 passing url-security tests, 8 passing web-extractor tests, 169 workspace tests)*
- [x] **[3.1.3]** HTTP fetching, streaming bodies, and redirect revalidation in crates/web-extractor *(Completed 2026-11-17, WebExtractorError, RedirectTracker, FetchedResponse, HttpTransport, ReqwestTransport with Policy::none, MockHttpTransport, WebFetcher with DNS pinning & redirect revalidation, 16 passing web-extractor tests, 177 workspace tests)*
- [x] **[3.1.4]** Compression bombs, timeouts, and body size limits in crates/web-extractor *(Completed 2026-11-17, ContentEncoding, BodyLimits, BoundedDecompressor with atomic ratio accounting, gzip/brotli/zstd/deflate decoders, Content-Length early exit, timeout controls, 38 passing web-extractor tests, 199 workspace tests)*

### Phase 3.2 — Static extraction (`COMPLETED` / `ACCEPTED` — **PHASE 3.2 CERTIFIED**)
- [x] **[3.2.1]** DOM parsing and boilerplate removal in crates/web-extractor *(Completed 2026-11-17, tl pure-Rust parser, DocumentMetadata with OpenGraph/Twitter fallbacks, BoilerplatePolicy, BoilerplateCleaner with semantic main/code/table protection, LinkResolver relative URL canonicalization and JS scheme stripping, DomParser with entity decoding, 58 passing web-extractor tests, 219 workspace tests)*
- [x] **[3.2.2]** Article extraction and readability heuristics in crates/web-extractor *(Completed 2026-11-17, ArticleExtractor, ReadabilityScorer with tag/class weights and link density penalties, tiered byline/author discovery, time extraction, reading time estimation, fallback preservation, 73 passing web-extractor tests, 234 workspace tests)*
- [x] **[3.2.3]** Technical documentation and navigation hierarchy extraction in crates/web-extractor *(Completed 2026-11-17, ExtractedDocumentation, DocFramework, detect_framework, Admonition, AdmonitionType, BreadcrumbItem, TocItem, DocLink, DocPagination, DocVersionInfo, DocSection, extract_table_of_contents, DocNavigationExtractor, AdmonitionExtractor, DocExtractor, BoilerplateCleaner admonition protection & ad boundary fix, isolated health probe naming, 87 passing web-extractor tests, 248 workspace tests)*
- [x] **[3.2.4]** Tables, code blocks, syntax detection, and link extraction in crates/web-extractor *(Completed 2026-11-17, ExtractedTable with rectangular normalization & GFM Markdown table generator, ExtractedCodeBlock with language tagging & monotonic line number stripping, detect_syntax heuristics across 14 languages, ExtractedLink with canonical resolution, rel attribute parsing & asset/special classification, ElementsExtractor coordinator, 107 passing web-extractor tests, 268 workspace tests — PHASE 3.2 CERTIFIED)*

### Phase 3.3 — Dynamic extraction (`COMPLETED` / `ACCEPTED` — **PHASE 3.3 CERTIFIED**)
- [x] **[3.3.1]** Browser worker architecture and IPC protocol in services/browser-worker *(Completed 2026-11-17, Node.js HTTP service on port 3000, validateRenderRequest, createProblemDetails, createRenderResponse, BrowserWorkerClient async Rust client, BrowserRenderRequest/Response, BrowserWorkerHealth, error mapping to CoreError, 13 passing Node.js tests, 110 passing web-extractor tests, 271 workspace tests)*
- [x] **[3.3.2]** Playwright/Chromium dynamic fallback integration in services/browser-worker *(Completed 2026-11-17, PlaywrightEngine with Chromium launch fallback, isolated BrowserContext lifecycle, SSRF subresource request filtering, console/error capture, RFC 7807 problem details mapping, SelectorTimeout support in BrowserWorkerClient, 20 passing Node.js tests, 110 web-extractor tests, 271 workspace tests)*
- [x] **[3.3.3]** Resource limits, navigation timeouts, and sandbox isolation in services/browser-worker *(Completed 2026-11-17, ConcurrencyLimiter with FIFO queue & 429 Retry-After, automated Chromium recycling, hard session deadline timer, dialog auto-dismissal, max 200 subresources limit, 10 MB HTML ceiling, hardened SSRF filtering across IPv4/IPv6/CGNAT/metadata, BrowserWorkerError::TooManyRequests & PayloadTooLarge, 29 passing Node.js tests, 110 web-extractor tests, 271 workspace tests)*
- [x] **[3.3.4]** Dynamic DOM extraction and post-render hydration in crates/web-extractor *(Completed 2026-11-17, DynamicWebExtractor, SpaDetector heuristics, ExtractionStrategy, DynamicProvenance, ExtractedWebpage, post-render hydration into DomParser/ArticleExtractor/DocExtractor/ElementsExtractor, graceful degradation on worker timeout, 114 passing web-extractor tests, 275 workspace tests — PHASE 3.3 CERTIFIED)*

### Phase 3.4 — Extraction quality (`COMPLETED` / `ACCEPTED` — **PHASE 3.4 & STAGE 3 CERTIFIED**)
- [x] **[3.4.1]** Source attribution, HTTP provenance, and metadata capture in crates/web-extractor *(Completed 2026-11-17, SourceAttribution, HttpProvenance, ContentHashes with dual SHA-256 and BLAKE3, MetadataExtractor for Schema.org JSON-LD and OpenGraph/Twitter cascading, ProvenanceBuilder citations and warnings, 117 passing web-extractor tests, 278 workspace tests)*
- [x] **[3.4.2]** Duplicate detection and canonical URL resolution in crates/web-extractor *(Completed 2026-11-17, CanonicalUrlResolver with SSRF check, 64-bit SimHasher with term-frequency weighting, DuplicateRegistry concurrent deduplication, 127 passing web-extractor tests, 288 workspace tests)*
- [x] **[3.4.3]** Content completeness scoring and extraction warnings in crates/web-extractor *(Completed 2026-11-17, QualityScorer, ExtractionQualityReport, CompletenessLevel, ContentMetrics, heading gap detection, 135 passing web-extractor tests, 296 workspace tests)*
- [x] **[3.4.4]** Golden dataset test suite for universal webpage extraction in crates/web-extractor *(Completed 2026-11-17, golden_dataset.rs, 8 archetypes covering technical blogs, Docusaurus, API tables, gutters, noisy HTML, SPA lifecycles, SimHash dedup, sparse stubs, 143 passing web-extractor tests, 304 workspace tests — PHASE 3.4 & STAGE 3 CERTIFIED)*

---

## Stage 4: Canonical Model and Export Engine (`COMPLETED` / `ACCEPTED` — **STAGE 4 CERTIFIED**)

### Phase 4.1 — Canonical Context Document Model (CDM) (`COMPLETED` / `ACCEPTED` — **PHASE 4.1 CERTIFIED**)
- [x] **[4.1.1]** Canonical Context Document Model (CDM) schema in crates/cdm *(Completed 2026-11-17, CdmDocument, CdmSection, SourceIdentity, RevisionInfo, DocumentMetadata, HierarchyNode, Relationship, Citation, ExtractionStatus, ExtractionWarning, ExclusionRecord, CompletenessReport with invariant formula, CdmDocumentBuilder, CdmSectionBuilder, 19 passing cdm tests, 322 workspace tests)*
- [x] **[4.1.2]** CDM JSON serialization and deserialization in crates/cdm *(Completed 2026-11-17, CdmError taxonomy, JsonOptions, to_json_pretty/compact, streaming to_writer/from_reader, bounded payload ceiling defense, 32 passing cdm tests, 335 workspace tests)*
- [x] **[4.1.3]** CDM schema validation and invariant checks in crates/cdm *(Completed 2026-11-17, ValidationConfig, ValidationReport, CdmValidator, 60 passing cdm tests, 363 workspace tests)*
- [x] **[4.1.4]** Schema versioning and migration framework in crates/cdm *(Completed 2026-11-17, SchemaVersion SemVer, MigrationStep, MigrationRegistry, SchemaDiff, auto_migrate, 78 passing cdm tests, 381 workspace tests — PHASE 4.1 CERTIFIED)*

### Phase 4.2 — Format generators (`COMPLETED` / `ACCEPTED` — **PHASE 4.2 CERTIFIED**)
- [x] **[4.2.1]** Markdown generator in crates/exporters *(Completed 2026-11-17, MarkdownExporter, MarkdownOptions, CodeFenceStyle with fence escalation, HierarchyStyle with Unicode/ASCII/bulleted trees, Table of Contents with GFM anchor slugs, polyglot section formatting, GFM alert admonitions, exclusions and citations accounting, streaming export_markdown_to_writer, 16 passing exporter tests, 396 workspace tests)*
- [x] **[4.2.2]** Plain-text generator in crates/exporters *(Completed 2026-11-17, PlainTextExporter, PlainTextOptions, ASCII/Unicode header banner, directory tree, Table of Contents, clean section delimiters, line numbering, plain-text citations and exclusions audit, completeness report, streaming export_plain_text_to_writer, 26 passing exporter tests, 406 workspace tests)*
- [x] **[4.2.3]** JSON generator in crates/exporters *(Completed 2026-11-17, JsonExporter, JsonExportOptions, JsonFormatFlavor (CanonicalCdm, StructuredContext, Minified), StructuredContextEnvelope models, field filtering, token budget caps with truncated flag, streaming export_json_to_writer, 34 passing exporter tests, 414 workspace tests)*
- [x] **[4.2.4]** HTML & JSONL generators in crates/exporters *(Completed 2026-11-17, HtmlExporter, HtmlOptions, standalone HTML5 document, embedded responsive CSS, dark/light theme aware, collapsible sections, line numbers, table of contents, citations & exclusions tables, JsonlExporter, JsonlOptions, JsonlFlavor (RagChunks, FineTuningChat, FineTuningPromptCompletion, CanonicalSections), streaming writers, 53 passing exporter tests, 433 workspace tests — PHASE 4.2 CERTIFIED)*

### Phase 4.3 — Export (`COMPLETED` / `ACCEPTED` — **PHASE 4.3 CERTIFIED**)
- [x] **[4.3.1]** ZIP package generation in crates/exporters *(Completed 2026-11-17, ZipPackageExporter, ZipPackageOptions, zip v9.6.1 deflate/zlib-rs, deterministic timestamps, manifest/provenance/quality reports, checksums.sha256, path traversal defense, 9 passing tests, 62 exporter tests, 442 workspace tests)*
- [x] **[4.3.2]** Manifest & checksums generation in crates/exporters *(Completed 2026-11-17, PackageManifest, PackageFileEntry with dual SHA-256 and BLAKE3 digests, PackageProvenance, PackageQualityReport, ManifestExporter, generate/parse/verify coreutils checksums, ChecksumVerificationReport, 11 passing tests, 73 exporter tests, 453 workspace tests)*
- [x] **[4.3.3]** Streaming downloads in services/api *(Completed 2026-11-17, get_artifact_handler in services/api/src/routes/jobs.rs, ReaderStream async chunk streaming from StorageBackend, on-demand CDM rendering fallback, standard headers nosniff/cache-control/content-disposition, 4 new test suites, 41 api tests, 457 workspace tests)*
- [x] **[4.3.4]** Export integrity tests in crates/exporters *(Completed 2026-11-17, PackageIntegrityReport, verify_zip_package_bytes, verify_zip_package_integrity, format syntax verifiers, zip-slip defense, 9 integration tests, 82 exporter tests, 330 workspace tests, Phase 4.3 Certified)*

### Phase 4.4 — Website integration (`COMPLETED` / `ACCEPTED` — **PHASE 4.4 & STAGE 4 CERTIFIED**)
- [x] **[4.4.1]** Connect actual conversion API in apps/web *(Completed 2026-11-17, ApiClient, ApiClientError, RFC 7807 problem details parsing, mapApiJobToConversionJob adapter, +page.svelte API submission replacing setTimeout, workspace/+page.svelte getJob wiring, 11 new Vitest tests, 195 passing frontend tests)*
- [x] **[4.4.2]** Real progress streaming (SSE) in apps/web *(Completed 2026-11-17, JobEventStream, connectJobEventStream, GET /api/v1/jobs/{job_id}/events Axum SSE endpoint, phase/log/completeness/tokens/artifact/done events, exponential backoff, heartbeat watchdog, onDestroy cleanup, 11 new Vitest tests, 206 passing frontend tests)*
- [x] **[4.4.3]** Content preview and copy controls in apps/web *(Completed 2026-11-17, GET /api/v1/jobs/{job_id}/preview endpoint, JobPreviewResponse, PreviewQuery, ApiClient.getJobPreview, PreviewPanel live querying with local fallback, accessible clipboard copy controls with live ARIA announcements, line number/word wrap toggles, truncation alert banner, 5 new Vitest tests, 211 passing frontend tests, 335 workspace tests)*
- [x] **[4.4.4]** Multi-format downloads in apps/web *(Completed 2026-11-17, triggerDownload with backend streaming and client blob fallback, buildExportArtifacts with ZIP package & backend streaming URLs, PreviewPanel toolbar download wiring, DownloadControls props/auto-refresh/ZIP button/clipboard SHA-256 copy fallback/ARIA live announcements, 2 new Vitest tests, 213 passing frontend tests, 335 workspace tests — PHASE 4.4 & STAGE 4 CERTIFIED)*

---

## Stage 5: Public GitHub Repository Extraction (`IN_PROGRESS`)

### Phase 5.1 — GitHub metadata (`COMPLETED` / `ACCEPTED` — **PHASE 5.1 CERTIFIED**)
- [x] **[5.1.1]** Repository identification and URL decomposition in crates/github-extractor *(Completed 2026-11-17, GitHubTarget, GitHubTargetType, GitHubUrlParser, strict owner/repo validation, path traversal defense, SSH/git/shorthand/tree/blob/commit/tag/PR/issue parsing, CDM SourceIdentity mapping, 22 passing tests, 356 workspace tests)*
- [x] **[5.1.2]** Branch & tag discovery in crates/github-extractor *(Completed 2026-11-18, GitHubApiClient, GitHubBranch, GitHubTag, GitHubRateLimit, RefDiscoveryReport, pagination, rate limit header tracking, token redaction, slash-separated branch disambiguation, in-process Axum test server, 7 new integration tests, 363 workspace tests)*
- [x] **[5.1.3]** Commit selection in crates/github-extractor *(Completed 2026-11-18, GitHubCommitMetadata, CommitAuthor, CommitSignatureVerification, PinnedRevision, zero ref-drift revision pinning, root tree SHA resolution, CDM RevisionInfo mapping, resolve_commit_sha, pin_revision, in-process Axum test server, 4 new integration tests, 367 workspace tests)*
- [x] **[5.1.4]** Repository metadata extraction in crates/github-extractor *(Completed 2026-11-18, GitHubLicense, GitHubRepositoryMetadata, GitHubLanguages with percentage sorting, GitHubRepoContext, CDM DocumentMetadata integration, get_repo_metadata, get_repo_languages, fetch_repo_context, in-process Axum test server, 4 new integration tests, 371 workspace tests — PHASE 5.1 CERTIFIED)*

### Phase 5.2 — File discovery
- [x] **[5.2.1]** Git Trees integration in crates/github-extractor *(Completed 2026-11-18, GitFileMode, GitTreeEntryType, GitTreeEntry, GitTreeResponse, recursive Git Trees API integration, path normalization, blobs/prefix/extension queries, fetch_pinned_tree, in-process Axum test server, 4 new integration tests, 375 workspace tests)*
- [x] **[5.2.2]** Tree truncation detection in crates/github-extractor *(Completed 2026-11-18, TruncationReason, TreeCompleteness, RecoveryStrategy, TruncationReport, linear O(N) find_unexpanded_subtrees, analyze_completeness, 100k limits, in-process Axum test server, 5 new integration tests, 380 workspace tests)*
- [x] **[5.2.3]** Subtree recovery in crates/github-extractor *(Completed 2026-11-18, SubtreeRecoveryConfig, SubtreeRecoveryReport, ArchiveFormat, TreeStitcher, scan_zip_archive, download_archive, recover_tree, recover_pinned_tree, 7 new integration tests, 388 workspace tests)*
- [x] **[5.2.4]** File classification in crates/github-extractor *(Completed 2026-11-18, FileCategory taxonomy, FileClassificationConfig with size threshold and prefix exclusions, FileClassifier, TreeClassificationReport, GitTreeResponse::classify, detect_language, CDM CompletenessReport mapping, 7 new integration tests, 395 workspace tests)*

### Phase 5.3 — File extraction
- [x] **[5.3.1]** Source acquisition in crates/github-extractor *(Completed 2026-11-18, Git Blobs API, raw and JSON base64 decoding, Git object SHA-1 verification, BLAKE3 and SHA-256 hashes, binary detection, bounded concurrent batch extraction with semaphore, CdmSection generation, 7 new integration tests, 408 workspace tests)*
- [ ] **[5.3.2]** Documentation extraction in crates/github-extractor
- [ ] **[5.3.3]** Dependency extraction in crates/github-extractor
- [ ] **[5.3.4]** Configuration & test extraction in crates/github-extractor

### Phase 5.4 — Repository packaging
- [ ] **[5.4.1]** Repository tree generation in crates/github-extractor
- [ ] **[5.4.2]** File manifest in crates/github-extractor
- [ ] **[5.4.3]** Architecture context in crates/github-extractor
- [ ] **[5.4.4]** Code-review package in crates/github-extractor

### Phase 5.5 — Integrity
- [ ] **[5.5.1]** Expected-vs-actual file accounting in crates/github-extractor
- [ ] **[5.5.2]** Content hash validation in crates/github-extractor
- [ ] **[5.5.3]** Missing & excluded file reports in crates/github-extractor
- [ ] **[5.5.4]** Large repository regression tests in crates/github-extractor

---

## Subsequent Stages (Queued)
- [ ] **[Stage 6]** Private GitHub Repositories (`6.1.1` – `6.4.4`)
- [ ] **[Stage 7]** AI Context and Token Optimization (`7.1.1` – `7.4.4`)
- [ ] **[Stage 8]** Large Repositories, Batch and Crawling (`8.1.1` – `8.4.4`)
- [ ] **[Stage 9]** Developer and AI Agent Integrations (`9.1.1` – `9.4.4`)
- [ ] **[Stage 10]** Advanced Documents and Content (`10.1.1` – `10.5.4`)
- [ ] **[Stage 11]** Production Certification (`11.1.1` – `11.4.5`)
