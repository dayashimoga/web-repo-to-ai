# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 2 — BACKEND AND PROCESSING FOUNDATION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 2.4 — Storage` (`NOT_STARTED`)  
**Active Subphase:** `2.4.1 — Implement local filesystem artifact storage in crates/storage` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `2.3.4 — Implement durable job leasing, heartbeat, and recovery in crates/storage` (`ACCEPTED` — **PHASE 2.3 CERTIFIED**)  
**Next Upcoming Subphase:** `2.4.1 — Implement local filesystem artifact storage in crates/storage`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Phase 2.1 (`2.1.1` – `2.1.3`), Phase 2.2 (`2.2.1` – `2.2.4`), and now **Phase 2.3 (`2.3.1` – `2.3.4`)** are 100% complete, verified, certified, and accepted. Stage 2 ("Backend and Processing Foundation") has achieved formal completion and certification of **Phase 2.3: Persistence**: Subphase `2.3.4` ("Implement durable job leasing, heartbeat, and recovery in crates/storage") delivered lock-free distributed worker queue polling via PostgreSQL `SELECT ... FOR UPDATE SKIP LOCKED` (`acquire_next_job`), periodic heartbeat lease extensions with phase/progress recording (`heartbeat_job`), lease validity verification (`is_lease_valid`), voluntary graceful lease relinquishment (`release_lease`), and automated background stale lease recovery (`recover_stale_jobs`). The full persistence layer now covers production PostgreSQL DDL schema, embedded SQLx migrations (`MIGRATOR`), runtime catalog introspection (`validate_schema`), deterministic job state machine (`JobStateMachine`), and transactional repository operations with row-level locking. 115/115 automated Rust tests pass across all workspace member crates (28 in `crates/storage`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly. Subphase `2.4.1` ("Implement local filesystem artifact storage in crates/storage") is queued as the next eligible subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 40 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.1.3`, `2.2.1` – `2.2.4`, `2.3.1` – `2.3.4` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 157 | `2.4.1` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
