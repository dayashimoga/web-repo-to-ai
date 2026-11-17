# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 1.2 — Main website` (`IN_PROGRESS`)  
**Active Subphase:** None (Awaiting user approval for `1.2.2`)  
**Last Completed Subphase:** `1.2.1 — Landing page` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `1.2.2 — URL input & source detection` (Phase 1.2)

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases) and Phase 1.1 (`1.1.1` – `1.1.3`, 3 subphases) are certified and accepted. Phase 1.2 ("Main website") Subphase `1.2.1` ("Landing page") is completed and accepted:
- **Navigation & Semantic Landmarks:** Authored sticky semantic `Header.svelte` with branding, stage badge, desktop & mobile navigation drawers, and accessible theme switcher; authored semantic `Footer.svelte` with architecture specifications links, compliance badges, and MIT license.
- **Production Hero & Value Proposition:** Authored `Hero.svelte` communicating the core product proposition, supporting 6 multi-format exporter pills (`.md`, `.json`, `.xml`, `.txt`, `.jsonl`, `.zip`), and displaying hardware/cost operational bounds (CPU-only 16 GB, <= 30 GB Podman, $0 / zero API keys).
- **Pipeline & Ingestion Matrix:** Authored `WorkflowSteps.svelte` showcasing the 3-step canonical pipeline (Acquisition & SSRF Defense, CDM Canonicalization & Optimization, Multi-Format Export & Token Fit); authored `SupportedSources.svelte` covering all 4 input modalities (Static Documentation, Dynamic JavaScript SPAs, Public GitHub Repositories, Private Repositories via GitHub Apps).
- **Architectural Highlights & Conversion Entry:** Authored `FeatureHighlights.svelte` detailing 6 core pillars (SSRF defense, AST code intelligence, zero mandatory paid APIs, 30 GB storage ceiling, deterministic token budgeting, privacy-first zero retention); assembled root `+page.svelte` with interactive conversion entry and dynamic source detection.
- **Automated Verification:** Verified 46 automated tests across 5 test suites (`tokens`, `color-utils`, `typography`, `ui-components`, `landing-page`) with 0 failures; `svelte-check` reports 0 errors and 0 warnings; all 5 monorepo verification gates in `scripts/check-all.*` passed.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 19 | `0.1.1` – `0.3.5`, `1.1.1` – `1.2.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 178 | `1.2.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
