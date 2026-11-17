# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 3.1 — Secure acquisition` (`ACCEPTED` & `CERTIFIED`) / `Phase 3.2 — Static extraction` (`IN_PROGRESS`)  
**Active Subphase:** `3.2.2 — Article extraction and readability heuristics in crates/web-extractor` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `3.2.1 — DOM parsing and boilerplate removal in crates/web-extractor` (`ACCEPTED`)  
**Next Upcoming Subphase:** `3.2.2 — Article extraction and readability heuristics in crates/web-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), and Phase 3.1 (`3.1.1` – `3.1.4`, 4 subphases) are 100% complete, verified, certified, and accepted. **Subphase `3.2.1` ("DOM parsing and boilerplate removal in crates/web-extractor") is now 100% complete, verified, and accepted.** `crates/web-extractor` integrates pure-safe-Rust parser `tl` (v0.7.8, MIT, zero dependencies), `DocumentMetadata` extracting titles, meta tags, and OpenGraph/Twitter attributes with full fallback cascades, `BoilerplateCleaner` and `BoilerplatePolicy` providing fine-grained stripping of scripts, styles, tracking, ads, cookie banners, navigation, footers, and sidebars while strictly preserving semantic `<main>`/`<article>` structures, preformatted `<pre><code>` blocks, and `<table>` content, and `LinkResolver` resolving and canonicalizing relative links while neutralizing `javascript:` XSS schemes. 219/219 automated Rust tests pass across all workspace crates (58 in `crates/web-extractor`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `3.2.2` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 49 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.2.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 148 | `3.2.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
