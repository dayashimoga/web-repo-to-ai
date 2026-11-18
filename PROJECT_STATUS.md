# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-18  
**Current Stage:** `STAGE 6 — PRIVATE GITHUB REPOSITORIES` (`IN_PROGRESS`)  
**Current Phase:** `Phase 6.2 — Credential management` (`IN_PROGRESS`)  
**Active Subphase:** `None (Phase 6.2 Subphase 6.2.3 ACCEPTED; awaiting user confirmation for Subphase 6.2.4)`  
**Last Completed Subphase:** `6.2.3 — Revocation in crates/github-extractor & services/api` (`ACCEPTED`)  
**Next Upcoming Subphase:** `6.2.4 — Secret redaction in crates/observability & crates/github-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), Stage 5 (`5.1.1` – `5.5.4`, 20 subphases), Phase 6.1 (`6.1.1` – `6.1.4`, 4 subphases), Subphase `6.2.1` ("Secure token storage"), Subphase `6.2.2` ("Token refresh & expiration"), and Subphase `6.2.3` ("Revocation") are 100% complete, verified, and accepted across 103 subphases. Subphase `6.2.3` implemented upstream GitHub App installation token revocation (`DELETE /installation/token`), coordinated storage/cache zeroization, webhook-driven uninstallation/suspension lifecycle synchronization, and tenant-scoped REST API revocation endpoints in `services/api`.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 103 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1` – `6.1.4`, `6.2.1`, `6.2.2`, `6.2.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 94 | `6.2.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
