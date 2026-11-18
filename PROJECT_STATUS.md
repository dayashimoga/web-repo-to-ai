# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-18  
**Current Stage:** `STAGE 6 — PRIVATE GITHUB REPOSITORIES` (`IN_PROGRESS`)  
**Current Phase:** `Phase 6.1 — GitHub App` (`IN_PROGRESS`)  
**Active Subphase:** `None` (Subphase `6.1.2` completed; awaiting explicit user instruction to begin `6.1.3`)  
**Last Completed Subphase:** `6.1.2 — OAuth & installation flow in crates/github-extractor` (`ACCEPTED`)  
**Next Upcoming Subphase:** `6.1.3 — Minimum permissions in crates/github-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), and Stage 5 (`5.1.1` – `5.5.4`, 20 subphases) are 100% complete, verified, certified, and accepted across 96 subphases. Stage 6 ("Private GitHub Repositories") is active: Phase 6.1 ("GitHub App") has completed Subphase `6.1.1` ("App configuration documentation & manifest") and Subphase `6.1.2` ("OAuth & installation flow") with cryptographic CSRF nonces, user authorization and app installation URL builders, server-side code exchange, RS256 JWT generation from PKCS#1/PKCS#8 keys, installation token exchange, and user/installation query APIs. Subphase `6.1.3` ("Minimum permissions") is queued next.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 98 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1`, `6.1.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 99 | `6.1.3` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
