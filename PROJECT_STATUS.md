# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 3.1 & Phase 3.2` (`ACCEPTED` & `CERTIFIED`) / `Phase 3.3 — Dynamic extraction` (`NOT_STARTED`)  
**Active Subphase:** `3.3.1 — Browser worker architecture and IPC protocol in services/browser-worker` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `3.2.4 — Tables, code blocks, syntax detection, and link extraction in crates/web-extractor` (`ACCEPTED` — **PHASE 3.2 CERTIFIED**)  
**Next Upcoming Subphase:** `3.3.1 — Browser worker architecture and IPC protocol in services/browser-worker`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Phase 3.1 (`3.1.1` – `3.1.4`, 4 subphases), and Phase 3.2 (`3.2.1` – `3.2.4`, 4 subphases) are 100% complete, verified, certified, and accepted. **Phase 3.2: Static Extraction is formally completed and certified.** `crates/web-extractor` delivers robust DOM parsing (`tl`), boilerplate cleaning, metadata harvesting, readability scoring and article extraction, documentation generator framework recognition (Docusaurus, mdBook, Sphinx, MkDocs, Rustdoc, VitePress, Nextra, GitBook), on-page heading hierarchy/ToC extraction, breadcrumbs and pagination, callout/admonition parsing, HTML table normalization into rectangular matrices with GFM Markdown generation, `<pre><code>` code block extraction with line-number gutter stripping and syntax detection heuristics, and hyperlink canonicalization/classification. 268/268 automated Rust tests pass across all workspace crates (107 in `crates/web-extractor`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `3.3.1` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 52 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.2.4` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 145 | `3.3.1` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
