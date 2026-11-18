# PROJECT ROADMAP (`ROADMAP.md`)

**Project:** `web-repo-to-ai`  
**Execution Hierarchy:** `Stage → Phase → Subphase → Tasks → Verification`  
**Detailed Specification:** [docs/ROADMAP.md](./docs/ROADMAP.md)  
**Live Execution Ledger:** [IMPLEMENTATION_LEDGER.md](./IMPLEMENTATION_LEDGER.md)

---

## Stage Overview & Status Summary

| Stage | Title | Primary Deliverable | Phases | Subphases | Current Status |
| :--- | :--- | :--- | :---: | :---: | :--- |
| **STAGE 0** | Project Bootstrap & Engineering Foundation | Working repository, governance, agents/skills/rules/workflows, CI, and Podman dev foundation | `0.1` – `0.3` | 15 (`0.1.1` – `0.3.5`) | `COMPLETED` / `ACCEPTED` |
| **STAGE 1** | Frontend & Product Experience | Responsive SvelteKit + Svelte 5 website foundation and conversion workspace | `1.1` – `1.4` | 14 (`1.1.1` – `1.4.3`) | `COMPLETED` / `ACCEPTED` |
| **STAGE 2** | Backend & Processing Foundation | Operational Rust Axum API, PostgreSQL job leasing, and artifact storage | `2.1` – `2.4` | 15 (`2.1.1` – `2.4.4`) | `COMPLETED` / `ACCEPTED` |
| **STAGE 3** | Universal Webpage Extraction | SSRF-guarded HTTP static & Playwright dynamic URL-to-clean-content extraction | `3.1` – `3.4` | 16 (`3.1.1` – `3.4.4`) | `COMPLETED` / `ACCEPTED` |
| **STAGE 4** | Canonical Model & Export Engine | Versioned CDM (`v1.0`), Markdown/TXT/JSON/HTML/JSONL/ZIP exporters, and end-to-end web integration | `4.1` – `4.4` | 16 (`4.1.1` – `4.4.4`) | `COMPLETED` / `ACCEPTED` |
| **STAGE 5** | Public GitHub Repository Extraction | Complete public GitHub repo conversion with 100% file completeness accounting | `5.1` – `5.5` | 20 (5 completed) | `IN_PROGRESS` |
| **STAGE 6** | Private GitHub Repositories | Secure read-only GitHub App integration, encrypted tokens, and tenant isolation | `6.1` – `6.4` | 16 (`6.1.1` – `6.4.4`) | `NOT_STARTED` |
| **STAGE 7** | AI Context & Token Optimization | Structural normalization, deduplication, exact/estimated tokenizers, and context profiles | `7.1` – `7.4` | 16 (`7.1.1` – `7.4.4`) | `NOT_STARTED` |
| **STAGE 8** | Large Repositories, Batch & Crawling | Multi-URL batching, bounded crawling, content-addressed caching, and streaming scale | `8.1` – `8.4` | 16 (`8.1.1` – `8.4.4`) | `NOT_STARTED` |
| **STAGE 9** | Developer & AI Agent Integrations | Authenticated REST API, Rust CLI, MCP server, and browser extension foundation | `9.1` – `9.4` | 16 (`9.1.1` – `9.4.4`) | `NOT_STARTED` |
| **STAGE 10** | Advanced Documents & Content | Modular PDF extraction, Tesseract OCR, structured docs, RAG chunks, and optional AI adapters | `10.1` – `10.5` | 20 (`10.1.1` – `10.5.4`) | `NOT_STARTED` |
| **STAGE 11** | Production Certification | Comprehensive security, performance, reliability, SBOM, and production release certification | `11.1` – `11.4` | 17 (`11.1.1` – `11.4.5`) | `NOT_STARTED` |

---

## Complete Subphase Index

### STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION
- **Phase 0.1 — Repository and governance initialization**
  - `0.1.1` Inspect environment, repository status and available tools (`ACCEPTED`)
  - `0.1.2` Initialize monorepo and Git configuration (`ACCEPTED`)
  - `0.1.3` Create engineering contract and requirements (`ACCEPTED`)
  - `0.1.4` Create agent skills, rules and workflows (`ACCEPTED`)
  - `0.1.5` Create stage/phase/subphase roadmap and ledger (`ACCEPTED`)
- **Phase 0.2 — GitHub and CI foundation**
  - `0.2.1` Create or connect GitHub repository (`ACCEPTED`)
  - `0.2.2` Configure branch protection and repository settings where authorized (`ACCEPTED`)
  - `0.2.3` Create GitHub Actions CI workflows (`ACCEPTED`)
  - `0.2.4` Configure linting, formatting and test checks (`ACCEPTED`)
  - `0.2.5` Configure security and dependency scanning (`ACCEPTED`)
- **Phase 0.3 — Containerized development foundation**
  - `0.3.1` Create Podman development configuration (`ACCEPTED`)
  - `0.3.2` Enforce the 30 GB storage ceiling (`ACCEPTED`)
  - `0.3.3` Create development environment scripts (`ACCEPTED`)
  - `0.3.4` Validate reproducible container builds (`ACCEPTED`)
  - `0.3.5` Document setup and recovery (`ACCEPTED`)

### STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE (`COMPLETED` / `ACCEPTED`)
- **Phase 1.1 — Design system:** `1.1.1` Visual design tokens (`ACCEPTED`) | `1.1.2` Typography, spacing & color systems (`ACCEPTED`) | `1.1.3` Reusable accessible components (`ACCEPTED`)
- **Phase 1.2 — Main website:** `1.2.1` Landing page (`ACCEPTED`) | `1.2.2` URL input & source detection (`ACCEPTED`) | `1.2.3` Conversion settings (`ACCEPTED`) | `1.2.4` Responsive layouts (`ACCEPTED`)
- **Phase 1.3 — Conversion workspace:** `1.3.1` Job workspace (`ACCEPTED`) | `1.3.2` Preview panels (`ACCEPTED`) | `1.3.3` Token statistics interface (`ACCEPTED`) | `1.3.4` Output & download controls (`ACCEPTED`)
- **Phase 1.4 — UX validation:** `1.4.1` Keyboard & accessibility testing (`ACCEPTED`) | `1.4.2` Mobile/tablet/desktop testing (`ACCEPTED`) | `1.4.3` UI performance & visual regression testing (`ACCEPTED` — **STAGE 1 CERTIFIED**)

### STAGE 2 — BACKEND AND PROCESSING FOUNDATION (`COMPLETED` / `ACCEPTED`)
- **Phase 2.1 — Rust architecture:** `2.1.1` Create Rust workspace (`ACCEPTED`) | `2.1.2` Error & configuration systems (`ACCEPTED`) | `2.1.3` Structured logging & tracing (`ACCEPTED` — **PHASE 2.1 CERTIFIED**)
- **Phase 2.2 — API:** `2.2.1` Implement Axum service (`ACCEPTED`) | `2.2.2` Request validation (`ACCEPTED`) | `2.2.3` API error contracts (`ACCEPTED`) | `2.2.4` Health & readiness endpoints (`ACCEPTED` — **PHASE 2.2 CERTIFIED**)
- **Phase 2.3 — Persistence:** `2.3.1` PostgreSQL schema (`ACCEPTED`) | `2.3.2` Migrations (`ACCEPTED`) | `2.3.3` Job state machine (`ACCEPTED`) | `2.3.4` Durable job leasing & recovery (`ACCEPTED` — **PHASE 2.3 CERTIFIED**)
- **Phase 2.4 — Storage:** `2.4.1` Local artifact storage (`ACCEPTED`) | `2.4.2` Storage abstraction (`ACCEPTED`) | `2.4.3` Retention & cleanup (`ACCEPTED`) | `2.4.4` Checksum validation (`ACCEPTED` — **PHASE 2.4 & STAGE 2 CERTIFIED**)

### STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION (`COMPLETED` / `ACCEPTED`)
- **Phase 3.1 — Secure acquisition:** `3.1.1` URL parsing & validation (`ACCEPTED`) | `3.1.2` DNS & SSRF protection (`ACCEPTED`) | `3.1.3` HTTP fetching & redirects (`ACCEPTED`) | `3.1.4` Compression, timeouts & size limits (`ACCEPTED` — **PHASE 3.1 CERTIFIED**)
- **Phase 3.2 — Static extraction:** `3.2.1` DOM parsing (`ACCEPTED`) | `3.2.2` Article extraction (`ACCEPTED`) | `3.2.3` Documentation extraction (`ACCEPTED`) | `3.2.4` Tables, links & code extraction (`ACCEPTED` — **PHASE 3.2 CERTIFIED**)
- **Phase 3.3 — Dynamic extraction:** `3.3.1` Browser worker architecture (`ACCEPTED`) | `3.3.2` Playwright integration (`ACCEPTED`) | `3.3.3` Resource & navigation limits (`ACCEPTED`) | `3.3.4` Dynamic DOM extraction (`ACCEPTED` — **PHASE 3.3 CERTIFIED**)
- **Phase 3.4 — Extraction quality:** `3.4.1` Source provenance (`ACCEPTED`) | `3.4.2` Duplicate detection (`ACCEPTED`) | `3.4.3` Extraction completeness (`ACCEPTED`) | `3.4.4` Golden dataset regression testing (`ACCEPTED` — **PHASE 3.4 & STAGE 3 CERTIFIED**)

### STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE
- **Phase 4.1 — CDM:** `4.1.1` Define schema (`ACCEPTED`) | `4.1.2` Serialization (`ACCEPTED`) | `4.1.3` Validation (`ACCEPTED`) | `4.1.4` Schema versioning (`ACCEPTED` — **PHASE 4.1 CERTIFIED**)
- **Phase 4.2 — Format generators:** `4.2.1` Markdown generator (`ACCEPTED`) | `4.2.2` Plain-text generator (`ACCEPTED`) | `4.2.3` JSON generator (`ACCEPTED`) | `4.2.4` HTML & JSONL generators (`ACCEPTED` — **PHASE 4.2 CERTIFIED**)
- **Phase 4.3 — Export:** `4.3.1` ZIP package generation (`ACCEPTED`) | `4.3.2` Manifest & checksums (`ACCEPTED`) | `4.3.3` Streaming downloads (`ACCEPTED`) | `4.3.4` Export integrity tests (`ACCEPTED` — **PHASE 4.3 CERTIFIED**)
- **Phase 4.4 — Website integration:** `4.4.1` Connect actual conversion API (`ACCEPTED`) | `4.4.2` Real progress (SSE) (`ACCEPTED`) | `4.4.3` Preview & copy (`ACCEPTED`) | `4.4.4` Downloads (`ACCEPTED` — **PHASE 4.4 & STAGE 4 CERTIFIED**)

### STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION
- **Phase 5.1 — GitHub metadata:** `5.1.1` Repository identification (`ACCEPTED`) | `5.1.2` Branch & tag discovery (`ACCEPTED`) | `5.1.3` Commit selection (`ACCEPTED`) | `5.1.4` Repository metadata extraction (`ACCEPTED` — **PHASE 5.1 CERTIFIED**)
- **Phase 5.2 — File discovery:** `5.2.1` Git Trees integration (`ACCEPTED`) | `5.2.2` Tree truncation detection (`ACCEPTED`) | `5.2.3` Subtree recovery (`ACCEPTED`) | `5.2.4` File classification (`ACCEPTED` — **PHASE 5.2 CERTIFIED**)
- **Phase 5.3 — File extraction:** `5.3.1` Source acquisition (`ACCEPTED`) | `5.3.2` Documentation extraction (`ACCEPTED`) | `5.3.3` Dependency extraction (`ACCEPTED`) | `5.3.4` Configuration & test extraction (`ACCEPTED` — **PHASE 5.3 CERTIFIED**)
- **Phase 5.4 — Repository packaging:** `5.4.1` Repository tree generation (`ACCEPTED`) | `5.4.2` File manifest (`ACCEPTED`) | `5.4.3` Architecture context (`NEXT`) | `5.4.4` Code-review package
- **Phase 5.5 — Integrity:** `5.5.1` Expected-vs-actual file accounting | `5.5.2` Content hash validation | `5.5.3` Missing & excluded file reports | `5.5.4` Large repository regression tests

### STAGE 6 — PRIVATE GITHUB REPOSITORIES
- **Phase 6.1 — GitHub App:** `6.1.1` App configuration documentation | `6.1.2` OAuth & installation flow | `6.1.3` Minimum permissions | `6.1.4` Callback security
- **Phase 6.2 — Credential management:** `6.2.1` Secure token storage | `6.2.2` Token refresh & expiration | `6.2.3` Revocation | `6.2.4` Secret redaction
- **Phase 6.3 — Private extraction:** `6.3.1` Installation authorization | `6.3.2` Selected repository access | `6.3.3` Private tree & source extraction | `6.3.4` Authorization failure recovery
- **Phase 6.4 — Security verification:** `6.4.1` Cross-user isolation | `6.4.2` Unauthorized repository access tests | `6.4.3` Token leakage tests | `6.4.4` Permission audit

### STAGE 7 — AI CONTEXT AND TOKEN OPTIMIZATION
- **Phase 7.1 — Content optimization:** `7.1.1` Structural normalization | `7.1.2` Exact duplicate removal | `7.1.3` Semantic-aware deduplication | `7.1.4` Content classification
- **Phase 7.2 — Token intelligence:** `7.2.1` Tokenizer adapter architecture | `7.2.2` Model-specific counting | `7.2.3` Cost estimation | `7.2.4` Context-budget calculation
- **Phase 7.3 — Context profiles:** `7.3.1` General AI | `7.3.2` Code review | `7.3.3` Security review | `7.3.4` Debugging & documentation
- **Phase 7.4 — Quality validation:** `7.4.1` Preservation metrics | `7.4.2` Compression reporting | `7.4.3` Exclusion explanations | `7.4.4` Golden comparison tests

### STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING
- **Phase 8.1 — Batch URLs:** `8.1.1` Multi-URL submission | `8.1.2` Bounded parallel execution | `8.1.3` Individual result tracking | `8.1.4` Batch export
- **Phase 8.2 — Website crawling:** `8.2.1` Sitemap discovery | `8.2.2` Internal link discovery | `8.2.3` Crawl-depth & domain restrictions | `8.2.4` Duplicate & loop prevention
- **Phase 8.3 — Incremental processing:** `8.3.1` HTTP validators | `8.3.2` Git commit/blob caching | `8.3.3` Content-addressed storage | `8.3.4` Changed-file processing
- **Phase 8.4 — Scalability:** `8.4.1` Adaptive concurrency | `8.4.2` Backpressure | `8.4.3` Streaming large exports | `8.4.4` Resumable processing

### STAGE 9 — DEVELOPER AND AI AGENT INTEGRATIONS
- **Phase 9.1 — REST API:** `9.1.1` API documentation | `9.1.2` Authentication & authorization | `9.1.3` Rate limits | `9.1.4` API integration tests
- **Phase 9.2 — CLI:** `9.2.1` URL conversion commands | `9.2.2` Repository conversion commands | `9.2.3` Configuration | `9.2.4` Download & verification
- **Phase 9.3 — MCP:** `9.3.1` MCP server | `9.3.2` Conversion tools | `9.3.3` Resource retrieval | `9.3.4` Security & protocol testing
- **Phase 9.4 — Browser integration:** `9.4.1` Browser extension foundation | `9.4.2` Current-page conversion | `9.4.3` Result handoff | `9.4.4` Extension security

### STAGE 10 — ADVANCED DOCUMENTS AND CONTENT
- **Phase 10.1 — PDF extraction:** `10.1.1` PDF parser & layout extraction | `10.1.2` Page/section provenance & table extraction | `10.1.3` Scanned/encrypted PDF detection & limits | `10.1.4` PDF golden accuracy validation
- **Phase 10.2 — Image OCR:** `10.2.1` Isolated Tesseract OCR adapter | `10.2.2` Image preprocessing & language selection | `10.2.3` Confidence scoring & provenance attribution | `10.2.4` OCR accuracy & timeout tests
- **Phase 10.3 — Structured documents:** `10.3.1` OpenAPI/JSON Schema/YAML extraction | `10.3.2` CSV/TSV & tabular data normalization | `10.3.3` Jupyter Notebook (`.ipynb`) cell extraction | `10.3.4` Structured document validation tests
- **Phase 10.4 — RAG chunk generation:** `10.4.1` Semantic & AST-boundary chunking | `10.4.2` Citation & cross-reference metadata injection | `10.4.3` JSONL & vector-ingest schema exports | `10.4.4` Chunk determinism & retrieval validation
- **Phase 10.5 — Optional local and external AI provider adapters:** `10.5.1` Provider-independent AI adapter trait | `10.5.2` Local Ollama / OpenAI-compatible adapter | `10.5.3` Strict opt-in guardrails & zero-key fallback | `10.5.4` Adapter contract & isolation tests

### STAGE 11 — PRODUCTION CERTIFICATION
- **Phase 11.1 — Security:** `11.1.1` Threat model review | `11.1.2` SSRF & network security | `11.1.3` OAuth & access-control audit | `11.1.4` Dependency & secrets audit
- **Phase 11.2 — Performance:** `11.2.1` Extraction benchmarks | `11.2.2` Large repository benchmarks | `11.2.3` Memory & concurrency testing | `11.2.4` Regression benchmarks
- **Phase 11.3 — Reliability:** `11.3.1` Interrupted-job recovery | `11.3.2` Retry & idempotency tests | `11.3.3` Storage cleanup verification | `11.3.4` Load & failure testing
- **Phase 11.4 — Release:** `11.4.1` Complete documentation | `11.4.2` Production build | `11.4.3` Release artifacts & SBOM | `11.4.4` Deployment verification | `11.4.5` Final certification report
