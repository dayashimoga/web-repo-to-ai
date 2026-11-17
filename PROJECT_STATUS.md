# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 1.2 — Main website` (`IN_PROGRESS`)  
**Active Subphase:** None (Awaiting user approval for `1.2.4`)  
**Last Completed Subphase:** `1.2.3 — Conversion settings panel` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `1.2.4 — Responsive layouts (mobile, tablet, desktop)` (Phase 1.2)

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Phase 1.1 (`1.1.1` – `1.1.3`, 3 subphases), and Phase 1.2 Subphases `1.2.1` – `1.2.3` are certified and accepted:
- **Conversion Settings & Calibrated AI Presets (`settings.ts`):** Authored strong TypeScript settings contracts and calibrated AI model presets for Gemini 1.5 Pro (2M window), Gemini 1.5 Flash (1M window), Claude 3.5 Sonnet (200k window), GPT-4o (128k window), and Local LLMs (32k/128k windows), backed by budget and file size validators.
- **Accessible Settings Component (`ConversionSettings.svelte`):** Authored collapsible disclosure component with ARIA landmarks (`aria-expanded`, `aria-controls`, `role="region"`), interactive preset selector buttons, dynamic token budget range slider with tabular numbers and ratio indicator, accessible content normalization toggles (`stripBoilerplate`, `stripCodeComments`, `excludeTests`, `semanticDeduplication`), dynamic Chromium rendering fallback switch, and file size boundary inputs.
- **Automated Verification:** Verified 69 automated tests across 8 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings; production build bundled cleanly with `@sveltejs/adapter-node`; all 5 monorepo verification gates in `scripts/check-all.*` passed.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 21 | `0.1.1` – `0.3.5`, `1.1.1` – `1.2.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 176 | `1.2.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
