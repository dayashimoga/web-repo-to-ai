# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 1.2 — Main website` (`IN_PROGRESS`)  
**Active Subphase:** None (Awaiting user approval for `1.2.3`)  
**Last Completed Subphase:** `1.2.2 — URL input & source detection` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `1.2.3 — Conversion settings panel` (Phase 1.2)

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases) and Phase 1.1 (`1.1.1` – `1.1.3`, 3 subphases) are certified and accepted. Phase 1.2 Subphases `1.2.1` ("Landing page") and `1.2.2` ("URL input & source detection") are completed and accepted:
- **URL Classification & SSRF Pre-validation Engine (`url-detector.ts`):** Authored deterministic URL classifier detecting standard public GitHub repositories, branch/tree subpaths, file blobs, XML sitemaps, and generic web documentation; implements client-side zero-trust SSRF pre-check against loopback (`127.0.0.0/8`, `::1`), RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata (`169.254.169.254`), and non-HTTP protocols (`file:`, `javascript:`, `data:`, `ftp:`).
- **Dedicated Accessible URL Input Component (`UrlInput.svelte`):** Authored accessible search form component (`role="search"`) with dynamic status badge, ARIA live region (`role="status"`, `aria-live="polite"`), leading icons matching detected source modalities, clear control (`aria-label="Clear URL input"`), error alert banner (`role="alert"`), and GitHub AST preview chips.
- **Automated Verification:** Verified 63 automated tests across 7 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings; production build bundled cleanly with `@sveltejs/adapter-node`; all 5 monorepo verification gates in `scripts/check-all.*` passed.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 20 | `0.1.1` – `0.3.5`, `1.1.1` – `1.2.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 177 | `1.2.3` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
