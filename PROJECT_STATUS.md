# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 4.4 — Export frontend` (`IN_PROGRESS`)  
**Active Subphase:** `None (awaiting approval for 4.4.4)`  
**Last Completed Subphase:** `4.4.3 — Content preview and copy controls in apps/web` (`ACCEPTED`)  
**Next Upcoming Subphase:** `4.4.4 — Multi-format downloads in apps/web`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), and Stage 3 (`3.1.1` – `3.4.4`, 16 subphases) are 100% complete, verified, certified, and accepted. In Stage 4, Phase 4.1 ("Canonical Context Document Model (CDM)"), Phase 4.2 ("Format generators"), and Phase 4.3 ("Export") are 100% complete, verified, certified, and accepted across 12 subphases (`4.1.1` through `4.3.4`). Phase 4.4 ("Export frontend") is active with Subphases `4.4.1`, `4.4.2`, and `4.4.3` accepted: Subphase `4.4.3` ("Content preview and copy controls in apps/web") implemented `GET /api/v1/jobs/{job_id}/preview`, connected live preview querying with local fallback degradation, accessible clipboard copy controls with live ARIA announcements, line number/word wrap toggles, truncation alerts, and 211 passing Vitest tests. Subphase `4.4.4` ("Multi-format downloads in apps/web") is queued next.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 75 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.1.4`, `4.2.1` – `4.2.4`, `4.3.1` – `4.3.4`, `4.4.1`, `4.4.2`, `4.4.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 122 | `4.4.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
