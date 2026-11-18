# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-18  
**Current Stage:** `STAGE 6 — PRIVATE GITHUB REPOSITORIES` (`IN_PROGRESS`)  
**Current Phase:** `Phase 6.3 — Private extraction` (`IN_PROGRESS`)  
**Active Subphase:** `None (Awaiting user confirmation to begin Subphase 6.3.3)`  
**Last Completed Subphase:** `6.3.2 — Selected repository access in crates/github-extractor` (`ACCEPTED`)  
**Next Upcoming Subphase:** `6.3.3 — Private tree & source extraction in crates/github-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), Stage 5 (`5.1.1` – `5.5.4`, 20 subphases), Phase 6.1 (`6.1.1` – `6.1.4`, 4 subphases), and Phase 6.2 (`6.2.1` – `6.2.4`, 4 subphases) are 100% complete, verified, and accepted across 104 subphases. Subphases `6.3.1` and `6.3.2` are complete and verified, bringing the accepted subphase count to 106. Subphase `6.3.2` ("Selected repository access") implements paginated repository traversal across `GET /installation/repositories` and `GET /user/installations/{id}/repositories`, local search and visibility filtering, live access verification, and dynamic grant synchronization from webhook events into cached session token records.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 106 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1` – `6.1.4`, `6.2.1` – `6.2.4`, `6.3.1`, `6.3.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 91 | `6.3.3` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
