# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 4.2 — Format generators` (`IN_PROGRESS`)  
**Active Subphase:** `4.2.2 — Plain-text generator in crates/exporters` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `4.2.1 — Markdown generator in crates/exporters` (`ACCEPTED`)  
**Next Upcoming Subphase:** `4.2.2 — Plain-text generator in crates/exporters`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), and Stage 3 (`3.1.1` – `3.4.4`, 16 subphases) are 100% complete, verified, certified, and accepted. Phase 4.1 ("Canonical Context Document Model (CDM)") is 100% complete, verified, certified, and accepted across all 4 subphases (`4.1.1` through `4.1.4`). **Phase 4.2 ("Format generators") is now active, with Subphase `4.2.1` ("Markdown generator in crates/exporters") COMPLETED and ACCEPTED.** Subphase `4.2.1` implements a deterministic, production-grade AI-ready Markdown export engine in `crates/exporters` with configurable frontmatter, metadata overview, directory tree / outline hierarchy, prose sections, escalating code fences, GFM tables, callout admonitions, provenance citations, auditable exclusion reporting, and streaming writer support. 396/396 automated Rust workspace tests pass (16 in `crates/exporters`), 29/29 Node.js tests pass in `services/browser-worker`, 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `4.2.2` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 65 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1`, `4.1.2`, `4.1.3`, `4.1.4`, `4.2.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 132 | `4.2.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
