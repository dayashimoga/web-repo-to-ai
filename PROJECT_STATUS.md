# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 3.1 & Phase 3.2` (`ACCEPTED` & `CERTIFIED`) / `Phase 3.3 — Dynamic extraction` (`IN_PROGRESS`)  
**Active Subphase:** `3.3.4 — Dynamic DOM extraction and post-render hydration in crates/web-extractor` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `3.3.3 — Resource limits, navigation timeouts, and sandbox isolation in services/browser-worker` (`ACCEPTED`)  
**Next Upcoming Subphase:** `3.3.4 — Dynamic DOM extraction and post-render hydration in crates/web-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Phase 3.1 (`3.1.1` – `3.1.4`, 4 subphases), and Phase 3.2 (`3.2.1` – `3.2.4`, 4 subphases) are 100% complete, verified, certified, and accepted. **Subphases `3.3.1` ("Browser worker architecture and IPC protocol in services/browser-worker"), `3.3.2` ("Playwright/Chromium dynamic fallback integration in services/browser-worker"), and `3.3.3` ("Resource limits, navigation timeouts, and sandbox isolation in services/browser-worker") are now 100% complete, verified, and accepted.** `services/browser-worker` enforces strict concurrency limits (`ConcurrencyLimiter` with bounded FIFO queue and 429 `Retry-After: 5`), hard session deadline timeouts, automated Chromium recycling (after 200 renders or 768 MB RSS), modal dialog auto-dismissal, subresource request limits (max 200), 10 MB HTML size ceilings (HTTP 413), and hardened subresource SSRF defenses (blocking loopback, RFC 1918, CGNAT, link-local, cloud metadata, and private IPv6 ranges). `crates/web-extractor`'s `BrowserWorkerClient` maps 429 to `TooManyRequests` and 413 to `PayloadTooLarge`. 271/271 automated Rust tests pass across all workspace crates (110 in `crates/web-extractor`), 29/29 Node.js tests pass in `services/browser-worker` across 7 test suites, 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `3.3.4` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 55 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.3.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 142 | `3.3.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
