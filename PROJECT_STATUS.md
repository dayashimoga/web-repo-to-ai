# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 5.1 — GitHub metadata` (`IN_PROGRESS`)  
**Active Subphase:** `None (awaiting approval for 5.1.2)`  
**Last Completed Subphase:** `5.1.1 — Repository identification and URL decomposition` (`ACCEPTED`)  
**Next Upcoming Subphase:** `5.1.2 — Branch & tag discovery`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), and Stage 4 (`4.1.1` – `4.4.4`, 16 subphases) are 100% complete, verified, certified, and accepted across 76 subphases. Stage 5 ("Public GitHub Repository Extraction") is active in Phase 5.1 ("GitHub metadata"): Subphase `5.1.1` ("Repository identification and URL decomposition") implemented GitHub repository URL parsing, decomposition (owner, repo, branch, tag, commit SHA, tree/blob subpaths, SSH/git protocols, shorthand notations), normalization, path traversal sanitization, and SSRF validation in `crates/github-extractor` with 22 passing tests and 356 workspace tests. Subphase `5.1.2` ("Branch & tag discovery") is queued next.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 77 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 120 | `5.1.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
