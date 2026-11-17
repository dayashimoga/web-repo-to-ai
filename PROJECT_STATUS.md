# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 1.2 — Main website` (`COMPLETED` / `ACCEPTED`)  
**Upcoming Phase:** `Phase 1.3 — Conversion workspace`  
**Active Subphase:** None (Phase 1.2 certified; awaiting user approval for `1.3.1`)  
**Last Completed Subphase:** `1.2.4 — Responsive layouts (mobile, tablet, desktop)` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `1.3.1 — Build conversion job workspace` (Phase 1.3)

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases) and Phase 1.1 (`1.1.1` – `1.1.3`, 3 subphases) are certified and accepted. **Phase 1.2: Main website** is now 100% complete and certified across all 4 subphases (`1.2.1` – `1.2.4` all `ACCEPTED`):
- **Landing Page & Semantic Navigation (`1.2.1`):** Authored sticky semantic `Header.svelte` with theme switcher and mobile menu, semantic `Footer.svelte` with architectural specs and compliance badges, production `Hero.svelte` with 6 format pills, `WorkflowSteps.svelte` 3-step pipeline, `SupportedSources.svelte` 4-source matrix, and `FeatureHighlights.svelte` 6-pillar cards.
- **URL Input & Source Detection (`1.2.2`):** Authored `url-detector.ts` with zero-trust client SSRF defense and real-time GitHub, sitemap, and web documentation classification, coupled with accessible `UrlInput.svelte` form with ARIA live region, dynamic status badges, and AST preview chips.
- **Conversion Settings & Model Presets (`1.2.3`):** Authored `ConversionSettings.svelte` with 7 calibrated AI model presets (Gemini 2M, Claude 200k, GPT-4o 128k, Local 32k/128k), context budget slider, and toggles for boilerplate stripping, comment removal, test exclusion, and Chromium rendering fallback.
- **Responsive Layouts & Cross-Viewport Audit (`1.2.4`):** Verified horizontal overflow containment (`overflow-x-hidden`, `max-w-full`), responsive grid collapsing, and WCAG AA touch targets ($\ge 44 \times 44\text{px}$) across mobile (320px+), tablet (768px+), and desktop (1280px+). 76 automated tests pass across 9 test suites; all 5 monorepo verification gates passed.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 22 | `0.1.1` – `0.3.5`, `1.1.1` – `1.2.4` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 175 | `1.3.1` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
