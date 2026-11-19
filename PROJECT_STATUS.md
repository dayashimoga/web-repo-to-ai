# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-19  
**Current Stage:** `STAGE 9 — DEVELOPER AND AI AGENT INTEGRATIONS` (`IN_PROGRESS`)  
**Last Completed Stage:** `STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING` (`100% COMPLETED & FORMALLY CERTIFIED`)  
**Last Completed Phase:** `Phase 8.4 — Scalability` (`100% COMPLETED & FORMALLY CERTIFIED`)  
**Active Subphase:** `None (Awaiting confirmation for Subphase 9.2.3)`  
**Last Completed Subphase:** `9.2.2 — Repository conversion commands (crates/cli: GitHub repository and local filesystem directory conversion commands)` (`ACCEPTED`)  
**Last Completed Phase:** `Phase 9.1 — REST API` (`100% COMPLETED`)  
**Next Upcoming Subphase:** `9.2.3 — Batch conversion and input file processing (crates/cli: batch conversion from input file, glob patterns, and directory scanning)`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), Stage 5 (`5.1.1` – `5.5.4`, 20 subphases), Stage 6 (`6.1.1` – `6.4.4`, 16 subphases), Stage 7 (`7.1.1` – `7.4.4`, 16 subphases), and Stage 8 (`8.1.1` – `8.4.4`, 16 subphases) are 100% complete, verified, and accepted across 144 subphases.

In Stage 9 ("Developer and AI Agent Integrations"):
- Phase 9.1 ("REST API") is 100% complete across all 4 subphases:
  - Subphase `9.1.1` delivered complete OpenAPI 3.1.0 specifications (`docs/openapi.json` and `docs/openapi.yaml`), embedded and served directly from `services/api` via `GET /api/v1/openapi.json`, `GET /api/v1/openapi.yaml`, and an interactive HTML documentation viewer at `GET /api/v1/docs` (with top-level alias `GET /docs`).
  - Subphase `9.1.2` delivered high-entropy cryptographic API key generation (`ring::rand::SystemRandom`, `wr_live_<64_hex_chars>`), BLAKE3 key hashing, `ApiKeyRecord` persistence (`ApiKeyRepository`, `InMemoryApiKeyRepository`, `PostgresApiKeyRepository`), SQL migration (`api_keys` table), `AuthCaller` and `RequireAuth` Axum extractors, API key management route handlers (`POST /api/v1/auth/keys`, `GET /api/v1/auth/keys`, `DELETE /api/v1/auth/keys/:key_id`), permission scope verification (`read`, `write`, `admin`), cross-tenant isolation enforcement (anti-enumeration with RFC 7807 404 responses), OpenAPI schemas and path definitions, technical documentation in `docs/API.md`, and integration test suites.
  - Subphase `9.1.3` delivered dual-tier token-bucket and sliding window rate limiting engine in `crates/core` (`RateLimitDecision`, `RateLimitTier`, `TokenBucket`, `SlidingWindowCounter`, `RateLimiter` trait, `InMemoryTieredRateLimiter` with bounded memory and idle bucket pruning), PostgreSQL rate limit persistence schema and store (`rate_limit_counters` table and `RateLimitStore` trait in `crates/storage`), Axum rate limiting middleware in `services/api` (`rate_limit_middleware`) with caller attribution (`Authorization` token hash, `X-Session-ID`, or client IP from forwarding headers), tiered ceilings (120 RPM / 30 burst for authenticated callers vs 60 RPM / 20 burst for anonymous callers, 10,000 for admins), standard header injection (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`), RFC 7807 problem details with `429 Too Many Requests` and `Retry-After` header on quota exhaustion, strict exemption of system health/docs/telemetry probes (`/`, `/docs`, `/api/v1/health`, `/api/v1/ready`, `/api/v1/metrics`, `/api/v1/openapi.*`, `/api/v1/docs`, and CORS preflights), OpenAPI 3.1 429 response schemas and header definitions across 19 operational routes, Section 12 in `docs/API.md`, and 6 comprehensive integration tests (`services/api/tests/rate_limit_test.rs`).
  - Subphase `9.1.4` delivered comprehensive end-to-end API integration tests in `services/api/tests/api_integration_test.rs` exercising single-job lifecycle (`POST /jobs`), CDM seeding and preview generation (`GET /jobs/:id/preview`), SSE real-time event streaming (`GET /jobs/:id/events` and `/stream`), streaming multi-format artifact downloads (`GET /jobs/:id/artifacts/:format` across markdown, txt, json, html, jsonl, zip, manifest, checksums), multi-URL parallel batch processing (`POST /jobs/batch`, `GET /jobs/batch/:id`, `GET /jobs/batch/:id/items`, `/items/:idx`), batch multi-format export and artifact listing, batch cancellation, source detection and classification (`POST /sources/detect`), telemetry and health probes (`/health`, `/ready`, `/metrics`, `/`), tenant boundary anti-enumeration isolation, and RFC 7807 validation error fidelity. Verified with 107 API tests, 570 workspace tests, zero clippy warnings, zero fmt diffs, cargo deny clean, and 100% passing quality gates (5/5).
- Phase 9.2 ("CLI") is in progress:
  - Subphase `9.2.1` delivered `crates/cli` implementing the `url` conversion command with dual-mode execution (local headless pipeline using `WebFetcher`, `ArticleExtractor`, `CdmDocumentBuilder`, `DocumentNormalizer`, `ContextBudgetOptimizer`, and `Multi-Format Exporters`; remote API client mode via `--api-url`), strict Unix stdout/stderr channel separation, deterministic exit codes (0: success, 1: validation error, 2: security/SSRF/network error, 3: extraction failure, 4: token budget exceeded), 8 multi-format targets (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `checksums`), `--json` execution envelope, token budgeting and `--no-optimize` flags, and 8 integration tests covering argument validation, SSRF loopback/private/metadata blocking, local multi-format conversion, token budgeting, and JSON metadata envelopes. Verified with 8 CLI tests, 578 workspace tests, zero clippy warnings, zero fmt diffs, cargo deny clean, and 100% passing quality gates (5/5).
  - Subphase `9.2.2` delivered repository conversion commands in `crates/cli` (`web-repo-to-ai repo <owner/repo|url|dir>`) supporting local filesystem recursive directory extraction (with standard ignore patterns, binary sniffing, and syntax language detection) and GitHub REST API ingestion (via `get_repo_metadata`, `resolve_commit_sha`, `get_tree`, `classify_tree`, and `extract_source_files`), subpath filtering and directory traversal prevention (exit code 2), token budget optimization via `ContextBudgetOptimizer` and `--no-optimize` flag, 8 export formats, `--json` execution envelopes, remote API client dispatch via `--api-url`, and 9 comprehensive integration tests (`crates/cli/tests/repo_command_test.rs`). Verified with 17 CLI tests, 587 workspace tests, zero clippy warnings, zero fmt diffs, cargo deny clean, and 100% passing quality gates (5/5).

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 150 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1` – `6.4.4`, `7.1.1` – `7.4.4`, `8.1.1` – `8.4.4`, `9.1.1`, `9.1.2`, `9.1.3`, `9.1.4`, `9.2.1`, `9.2.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 46 | `9.2.3` – `11.4.5` |
| **Total** | **196** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
