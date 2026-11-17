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

## Stage 2 — Backend and Processing Foundation (`IN_PROGRESS`)

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

### Phase 2.4 — Storage (`IN_PROGRESS`)
- [x] **[2.4.1]** Implement local filesystem artifact storage in crates/storage *(Completed 2026-11-17, LocalFsStorage, atomic staging writes, path traversal guards, layout helpers, storage_capacity accounting, 36 passing storage tests, 123 workspace tests)*
- [ ] **[2.4.2]** Implement pluggable storage backend trait in crates/storage
- [ ] **[2.4.3]** Implement artifact retention and ephemeral cleanup policies in crates/storage
- [ ] **[2.4.4]** Implement artifact checksum validation and verification in crates/storage

---

## Subsequent Stages (Queued)
- [ ] **[Stage 3]** Universal Webpage Extraction (`3.1.1` – `3.4.4`)
- [ ] **[Stage 4]** Canonical Model and Export Engine (`4.1.1` – `4.4.4`)
- [ ] **[Stage 5]** Public GitHub Repository Extraction (`5.1.1` – `5.5.4`)
- [ ] **[Stage 6]** Private GitHub Repositories (`6.1.1` – `6.4.4`)
- [ ] **[Stage 7]** AI Context and Token Optimization (`7.1.1` – `7.4.4`)
- [ ] **[Stage 8]** Large Repositories, Batch and Crawling (`8.1.1` – `8.4.4`)
- [ ] **[Stage 9]** Developer and AI Agent Integrations (`9.1.1` – `9.4.4`)
- [ ] **[Stage 10]** Advanced Documents and Content (`10.1.1` – `10.5.4`)
- [ ] **[Stage 11]** Production Certification (`11.1.1` – `11.4.5`)
