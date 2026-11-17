# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 2 — BACKEND AND PROCESSING FOUNDATION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 2.3 — Persistence` (`IN_PROGRESS`)  
**Active Subphase:** `2.3.2 — Implement database migrations and schema validation in crates/storage` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `2.3.1 — Design and implement PostgreSQL relational schema in crates/storage` (`ACCEPTED`)  
**Next Upcoming Subphase:** `2.3.2 — Implement database migrations and schema validation in crates/storage`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Phase 2.1 (`2.1.1` – `2.1.3`), and Phase 2.2 (`2.2.1` – `2.2.4`) are 100% complete, verified, certified, and accepted. Stage 2 ("Backend and Processing Foundation") is advancing through Phase 2.3: **Subphase `2.3.1` ("Design and implement PostgreSQL relational schema in crates/storage") has been successfully implemented, verified, and accepted**. The production PostgreSQL relational schema DDL (`ddl.sql`) is implemented with complete constraint checking, cascading foreign keys, performance indexes (`idx_jobs_status_lease`, `idx_jobs_owner_session`, `idx_jobs_created_at`, `idx_job_artifacts_job_id`, `idx_job_artifacts_expires_at`, `idx_content_cache_last_accessed`), and `update_timestamp` triggers. Strongly-typed Rust entity models (`JobRow`, `JobKind`, `JobStatus`, `JobProgressPhase`, `JobArtifactRow`, `ArtifactFormat`, `GitHubInstallationRow`, `ContentCacheIndexRow`) derive `sqlx::FromRow` and validate all domain invariants. 97/97 automated Rust tests pass across all workspace crates (10 in `crates/storage`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly. Subphase `2.3.2` ("Implement database migrations and schema validation in crates/storage") is queued as the next eligible subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 37 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.1.3`, `2.2.1` – `2.2.4`, `2.3.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 160 | `2.3.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
