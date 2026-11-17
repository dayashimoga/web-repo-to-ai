# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 2 — BACKEND AND PROCESSING FOUNDATION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 2.3 — Persistence` (`IN_PROGRESS`)  
**Active Subphase:** `2.3.4 — Implement durable job leasing, heartbeat, and recovery in crates/storage` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `2.3.3 — Implement job state machine, transitions, and history in crates/storage` (`ACCEPTED`)  
**Next Upcoming Subphase:** `2.3.4 — Implement durable job leasing, heartbeat, and recovery in crates/storage`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Phase 2.1 (`2.1.1` – `2.1.3`), and Phase 2.2 (`2.2.1` – `2.2.4`) are 100% complete, verified, certified, and accepted. Stage 2 ("Backend and Processing Foundation") is advancing through Phase 2.3: **Subphase `2.3.3` ("Implement job state machine, transitions, and history in crates/storage") has been successfully implemented, verified, and accepted**. The job state machine (`JobStateMachine`) strictly enforces legal transitions, phase invariants, and terminal state immutability (`Completed`, `Failed`, `Cancelled`). Database migration `20261117000002_job_state_transitions.sql` establishes the indexed audit trail table `job_state_transitions` with cascade deletion. `JobRepository` provides transactional row-level locked transitions (`SELECT ... FOR UPDATE`), progress updates, and chronological transition history retrieval. 109/109 automated Rust tests pass across all workspace crates (22 in `crates/storage`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly. Subphase `2.3.4` ("Implement durable job leasing, heartbeat, and recovery in crates/storage") is queued as the next eligible subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 39 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.1.3`, `2.2.1` – `2.2.4`, `2.3.1` – `2.3.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 158 | `2.3.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
