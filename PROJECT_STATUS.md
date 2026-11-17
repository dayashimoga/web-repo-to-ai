# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE` (`IN_PROGRESS`)  
**Current Phase:** `Phase 4.1 — Canonical Context Document Model (CDM)` (`IN_PROGRESS`)  
**Active Subphase:** `4.1.3 — CDM schema validation and invariant checks in crates/cdm` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `4.1.2 — CDM JSON serialization and deserialization in crates/cdm` (`ACCEPTED`)  
**Next Upcoming Subphase:** `4.1.3 — CDM schema validation and invariant checks in crates/cdm`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), and Stage 3 (`3.1.1` – `3.4.4`, 16 subphases) are 100% complete, verified, certified, and accepted. Subphase `4.1.1` ("Canonical Context Document Model (CDM) schema in crates/cdm") is complete and accepted. **Subphase `4.1.2` ("CDM JSON serialization and deserialization in crates/cdm") is complete, verified, and accepted.** `crates/cdm` implements canonical JSON serialization and deserialization: `to_json_pretty`, `to_json_compact`, streaming `to_writer` / `from_reader`, bounded reader payload defenses (`PayloadTooLarge` with 64 MiB default ceiling), strongly-typed error taxonomy (`CdmError` mapped to `CoreError`), deterministic BTreeMap key sorting, and lossless roundtrip fidelity across all CDM schema primitives. 335/335 automated Rust workspace tests pass (32 in `crates/cdm`), 29/29 Node.js tests pass in `services/browser-worker`, 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `4.1.3` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 62 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1`, `4.1.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 135 | `4.1.3` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
