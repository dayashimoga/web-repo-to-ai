# Detailed Staged Implementation Roadmap (`docs/ROADMAP.md`)

**Project:** `web-repo-to-ai`  
**Total Stages:** 12 (`Stage 0` – `Stage 11`)  
**Total Phases:** 48  
**Total Subphases:** 197  
**Execution Policy:** Strictly one subphase at a time (`Stage → Phase → Subphase → Tasks → Verification`).

---

## Stage Acceptance Gates & Deliverable Matrix

### STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION (`COMPLETED` / `ACCEPTED`)
- **Deliverable:** Working Git repository with all engineering governance, agent definitions, project documentation, CI workflows, and containerized development foundations.
- **Stage Acceptance Gate:** Repository, documentation, agents, rules, workflows, Podman config (<30 GB), and initial CI workflows are operational and verified.
- **Phases:**
  - `0.1` Repository and governance initialization (`0.1.1` – `0.1.5`) (`COMPLETED` / `ACCEPTED`)
  - `0.2` GitHub and CI foundation (`0.2.1` – `0.2.5`) (`COMPLETED` / `ACCEPTED`)
  - `0.3` Containerized development foundation (`0.3.1` – `0.3.5`) (`COMPLETED` / `ACCEPTED`)

### STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE (`COMPLETED` / `ACCEPTED`)
- **Deliverable:** Real responsive SvelteKit + Svelte 5 website foundation in `apps/web`.
- **Stage Acceptance Gate:** Real frontend, functional navigation, and validated responsive UI connected to tested API contracts with keyboard/WCAG AA accessibility.
- **Phases:**
  - `1.1` Design system (`1.1.1` – `1.1.3`) (`COMPLETED` / `ACCEPTED`)
  - `1.2` Main website (`1.2.1` – `1.2.4`) (`COMPLETED` / `ACCEPTED`)
  - `1.3` Conversion workspace (`1.3.1` – `1.3.4`) (`COMPLETED` / `ACCEPTED`)
  - `1.4` UX validation (`1.4.1` – `1.4.3`) (`COMPLETED` / `ACCEPTED`)

### STAGE 2 — BACKEND AND PROCESSING FOUNDATION (`COMPLETED` / `ACCEPTED — CERTIFIED`)
- **Deliverable:** Operational Rust Axum API, persistent PostgreSQL job management, and artifact storage infrastructure.
- **Stage Acceptance Gate:** Actual API, database schema/migrations, durable job leasing, and storage lifecycle verified via integration tests.
- **Phases:**
  - `2.1` Rust architecture (`2.1.1` [ACCEPTED], `2.1.2` [ACCEPTED], `2.1.3` [ACCEPTED — CERTIFIED])
  - `2.2` API (`2.2.1` [ACCEPTED], `2.2.2` [ACCEPTED], `2.2.3` [ACCEPTED], `2.2.4` [ACCEPTED — CERTIFIED])
  - `2.3` Persistence (`2.3.1` [ACCEPTED], `2.3.2` [ACCEPTED], `2.3.3` [ACCEPTED], `2.3.4` [ACCEPTED — CERTIFIED])
  - `2.4` Storage (`2.4.1` [ACCEPTED], `2.4.2` [ACCEPTED], `2.4.3` [ACCEPTED], `2.4.4` [ACCEPTED — STAGE 2 CERTIFIED])

### STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION (`COMPLETED` / `ACCEPTED — CERTIFIED`)
- **Deliverable:** Functional, SSRF-safe URL-to-clean-content conversion across static HTML and dynamic JS pages.
- **Stage Acceptance Gate:** Actual URLs produce validated, source-traceable output with full SSRF/DNS-rebinding protection and golden regression tests.
- **Phases:**
  - `3.1` Secure acquisition (`3.1.1` [ACCEPTED], `3.1.2` [ACCEPTED], `3.1.3` [ACCEPTED], `3.1.4` [ACCEPTED] — CERTIFIED)
  - `3.2` Static extraction (`3.2.1` [ACCEPTED], `3.2.2` [ACCEPTED], `3.2.3` [ACCEPTED], `3.2.4` [ACCEPTED — PHASE 3.2 CERTIFIED])
  - `3.3` Dynamic extraction (`3.3.1` [ACCEPTED], `3.3.2` [ACCEPTED], `3.3.3` [ACCEPTED], `3.3.4` [ACCEPTED — PHASE 3.3 CERTIFIED])
  - `3.4` Extraction quality (`3.4.1` [ACCEPTED], `3.4.2` [ACCEPTED], `3.4.3` [ACCEPTED], `3.4.4` [ACCEPTED — STAGE 3 CERTIFIED])

### STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE
- **Deliverable:** Versioned CDM (`crates/cdm`) and consistent multi-format output generators integrated end-to-end with `apps/web`.
- **Stage Acceptance Gate:** A user can enter a URL in the web UI and obtain verified downloadable AI-ready content (Markdown, TXT, JSON, HTML, JSONL, ZIP).
- **Phases:**
  - `4.1` CDM (`4.1.1` [ACCEPTED], `4.1.2` [ACCEPTED], `4.1.3` [ACCEPTED], `4.1.4` [ACCEPTED — PHASE 4.1 CERTIFIED])
  - `4.2` Format generators (`4.2.1` [ACCEPTED], `4.2.2` [ACCEPTED], `4.2.3` [ACCEPTED], `4.2.4` [ACCEPTED — PHASE 4.2 CERTIFIED])
  - `4.3` Export (`4.3.1` [ACCEPTED], `4.3.2` [ACCEPTED], `4.3.3` [ACCEPTED], `4.3.4` [ACCEPTED — PHASE 4.3 CERTIFIED])
  - `4.4` Website integration (`4.4.1` [ACCEPTED], `4.4.2` [ACCEPTED], `4.4.3` [ACCEPTED], `4.4.4` [ACCEPTED — PHASE 4.4 & STAGE 4 CERTIFIED])

### STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION (`COMPLETED` / `ACCEPTED` — **STAGE 5 CERTIFIED**)
- **Deliverable:** Complete public GitHub repository conversion across branches, tags, commits, and path filters.
- **Stage Acceptance Gate:** Real public repositories produce verifiable, complete packages where $\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$.
- **Phases:**
  - `5.1` GitHub metadata (`5.1.1` [ACCEPTED], `5.1.2` [ACCEPTED], `5.1.3` [ACCEPTED], `5.1.4` [ACCEPTED — PHASE 5.1 CERTIFIED])
  - `5.2` File discovery (`5.2.1` [ACCEPTED], `5.2.2` [ACCEPTED], `5.2.3` [ACCEPTED], `5.2.4` [ACCEPTED — PHASE 5.2 CERTIFIED])
  - `5.3` File extraction (`5.3.1` [ACCEPTED], `5.3.2` [ACCEPTED], `5.3.3` [ACCEPTED], `5.3.4` [ACCEPTED — PHASE 5.3 CERTIFIED])
  - `5.4` Repository packaging (`5.4.1` [ACCEPTED], `5.4.2` [ACCEPTED], `5.4.3` [ACCEPTED], `5.4.4` [ACCEPTED — PHASE 5.4 CERTIFIED])
  - `5.5` Integrity (`5.5.1` [ACCEPTED], `5.5.2` [ACCEPTED], `5.5.3` [ACCEPTED], `5.5.4` [ACCEPTED — PHASE 5.5 & STAGE 5 CERTIFIED])

### STAGE 6 — PRIVATE GITHUB REPOSITORIES
- **Deliverable:** Secure private repository conversion via read-only GitHub App authorization.
- **Stage Acceptance Gate:** Authorized private repositories can be extracted without exposing credentials, leaking tokens, or permitting repository modifications.
- **Phases:**
  - `6.1` GitHub App (`6.1.1` [ACCEPTED], `6.1.2` [ACCEPTED], `6.1.3` [ACCEPTED], `6.1.4` [ACCEPTED — PHASE 6.1 CERTIFIED])
  - `6.2` Credential management (`6.2.1` [ACCEPTED], `6.2.2` [ACCEPTED], `6.2.3` [ACCEPTED], `6.2.4` [ACCEPTED — PHASE 6.2 CERTIFIED])
  - `6.3` Private extraction (`6.3.1` [ACCEPTED], `6.3.2` [ACCEPTED], `6.3.3` [ACCEPTED], `6.3.4` [ACCEPTED — PHASE 6.3 CERTIFIED])
  - `6.4` Security verification (`6.4.1` [ACCEPTED], `6.4.2` [ACCEPTED], `6.4.3` [ACCEPTED], `6.4.4` [ACCEPTED — PHASE 6.4 & STAGE 6 CERTIFIED])

### STAGE 7 — AI CONTEXT AND TOKEN OPTIMIZATION
- **Deliverable:** Accurate, measurable, token-efficient context optimization across specialized engineering profiles.
- **Stage Acceptance Gate:** Measurable optimization with exact/labeled token counts, transparent information-preservation metrics, and exclusion reports.
- **Phases:**
  - `7.1` Content optimization (`7.1.1` [ACCEPTED], `7.1.2` [ACCEPTED], `7.1.3` [ACCEPTED], `7.1.4` [ACCEPTED] — **PHASE 7.1 CERTIFIED**)
  - `7.2` Token intelligence (`7.2.1` [ACCEPTED], `7.2.2` [ACCEPTED], `7.2.3` [ACCEPTED], `7.2.4` [ACCEPTED] — **PHASE 7.2 CERTIFIED**)
  - `7.3` Context profiles (`7.3.1` [ACCEPTED], `7.3.2` [ACCEPTED], `7.3.3` [ACCEPTED], `7.3.4` [ACCEPTED] — **PHASE 7.3 CERTIFIED**)
  - `7.4` Quality validation (`7.4.1` [ACCEPTED], `7.4.2` [ACCEPTED], `7.4.3` [ACCEPTED], `7.4.4` [ACCEPTED] — **PHASE 7.4 & STAGE 7 CERTIFIED**)

### STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING (`COMPLETED` / `ACCEPTED` — **STAGE 8 CERTIFIED**)
- **Deliverable:** Bounded parallel batch URL processing, website crawling, and content-addressed incremental caching.
- **Stage Acceptance Gate:** Reliable batch and large-source processing within bounded memory and CPU resources.
- **Phases:**
  - `8.1` Batch URLs (`8.1.1` [ACCEPTED], `8.1.2` [ACCEPTED], `8.1.3` [ACCEPTED], `8.1.4` [ACCEPTED — PHASE 8.1 CERTIFIED])
  - `8.2` Website crawling (`8.2.1`–`8.2.4` [ACCEPTED — PHASE 8.2 CERTIFIED])
  - `8.3` Incremental processing (`8.3.1`–`8.3.4` [ACCEPTED — PHASE 8.3 CERTIFIED])
  - `8.4` Scalability (`8.4.1` [ACCEPTED], `8.4.2` [ACCEPTED], `8.4.3` [ACCEPTED], `8.4.4` [ACCEPTED — PHASE 8.4 & STAGE 8 CERTIFIED])

### STAGE 9 — DEVELOPER AND AI AGENT INTEGRATIONS (`IN_PROGRESS`)
- **Deliverable:** Reusable developer-facing conversion platform (REST API, CLI, MCP server, browser extension foundation).
- **Stage Acceptance Gate:** Functioning integrations powered by the same production Rust extraction engine.
- **Phases:**
  - `9.1` REST API (`9.1.1` [ACCEPTED], `9.1.2` [ACCEPTED], `9.1.3` [ACCEPTED], `9.1.4` [ACCEPTED] — **PHASE 9.1 CERTIFIED**)
  - `9.2` CLI (`9.2.1` [ACCEPTED], `9.2.2` [ACCEPTED], `9.2.3` – `9.2.4`)
  - `9.3` MCP (`9.3.1` – `9.3.4`)
  - `9.4` Browser integration (`9.4.1` – `9.4.4`)

### STAGE 10 — ADVANCED DOCUMENTS AND CONTENT
- **Deliverable:** Extended document extraction capabilities (PDF, OCR, structured specs/notebooks, RAG chunks, optional AI adapters).
- **Stage Acceptance Gate:** Real document conversion with source attribution, documented format limitations, and validated output.
- **Phases:**
  - `10.1` PDF extraction (`10.1.1` – `10.1.4`)
  - `10.2` Image OCR (`10.2.1` – `10.2.4`)
  - `10.3` Structured documents (`10.3.1` – `10.3.4`)
  - `10.4` RAG chunk generation (`10.4.1` – `10.4.4`)
  - `10.5` Optional local and external AI provider adapters (`10.5.1` – `10.5.4`)

### STAGE 11 — PRODUCTION CERTIFICATION
- **Deliverable:** Verified, deployable production release with SBOMs, checksums, and certification report.
- **Stage Acceptance Gate:** All critical security, performance, reliability, and release gates proven with repeatable objective evidence.
- **Phases:**
  - `11.1` Security (`11.1.1` – `11.1.4`)
  - `11.2` Performance (`11.2.1` – `11.2.4`)
  - `11.3` Reliability (`11.3.1` – `11.3.4`)
  - `11.4` Release (`11.4.1` – `11.4.5`)
