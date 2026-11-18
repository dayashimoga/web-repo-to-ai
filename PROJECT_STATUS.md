# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-18  
**Current Stage:** `STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 5.2 — File discovery` (`COMPLETED` — **PHASE 5.2 FORMALLY CERTIFIED**)  
**Active Subphase:** `None (awaiting approval for 5.3.1)`  
**Last Completed Subphase:** `5.2.4 — File classification in crates/github-extractor` (`ACCEPTED` — **PHASE 5.2 CERTIFIED**)  
**Next Upcoming Subphase:** `5.3.1 — Source acquisition in crates/github-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), and Stage 4 (`4.1.1` – `4.4.4`, 16 subphases) are 100% complete, verified, certified, and accepted across 76 subphases. Stage 5 ("Public GitHub Repository Extraction") has certified Phase 5.1 ("GitHub metadata", `5.1.1` – `5.1.4`) and Phase 5.2 ("File discovery", `5.2.1` – `5.2.4`): Git Trees API integration (`5.2.1`), Tree truncation detection (`5.2.2`), Subtree recovery & archive fallback (`5.2.3`), and File classification & CDM completeness reporting (`5.2.4`) are all accepted and formally certified. Phase 5.3 ("File extraction", `5.3.1` – `5.3.4`) is queued next, beginning with Subphase `5.3.1` ("Source acquisition").

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 84 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.2.4` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 113 | `5.3.1` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
