# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-19  
**Current Stage:** `STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING` (`IN_PROGRESS`)  
**Last Completed Stage:** `STAGE 7 — AI CONTEXT AND TOKEN OPTIMIZATION` (`100% COMPLETED & FORMALLY CERTIFIED`)  
**Last Completed Phase:** `Phase 8.1 — Batch URLs` (`100% COMPLETED & FORMALLY CERTIFIED`)  
**Active Subphase:** `None (Subphase 8.2.2 completed and verified; awaiting user confirmation to proceed to 8.2.3)`  
**Last Completed Subphase:** `8.2.2 — Internal link discovery in crates/web-extractor` (`ACCEPTED`)  
**Next Upcoming Subphase:** `8.2.3 — Crawl-depth & domain restrictions in crates/web-extractor`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), Stage 5 (`5.1.1` – `5.5.4`, 20 subphases), Stage 6 (`6.1.1` – `6.4.4`, 16 subphases), and Stage 7 (`7.1.1` – `7.4.4`, 16 subphases) are 100% complete, verified, and accepted across 128 subphases. In Stage 8 ("Large Repositories, Batch and Crawling"), Phase 8.1 ("Batch URLs", subphases `8.1.1` – `8.1.4`) is 100% complete, verified, and formally certified. In Phase 8.2 ("Website crawling"), Subphases `8.2.1` ("Sitemap discovery in crates/web-extractor") and `8.2.2` ("Internal link discovery in crates/web-extractor") are fully implemented and verified. Subphase `8.2.2` delivered: domain models (`DiscoveredInternalLink`, `InternalLinkDiscoveryConfig`, `PageLinkDiscoveryReport`), automated `InternalLinkDiscoverer` engine with HTML `<base href="...">` resolution, top-down DOM parent-map hierarchy indexing for semantic ancestor traversal, content vs boilerplate navigation classification (`is_content_link`), pagination link detection (`is_pagination`), surrounding sentence context snippet extraction (up to 140 chars), URL normalization (in-page anchor fragment stripping `#section` and marketing tracking query parameter stripping `utm_*`, `fbclid`, etc.), asset exclusion (`.pdf`, `.zip`, etc.), subdomain policies, `rel="nofollow"` policies, Zero-Trust SSRF rejection via `ValidatedUrl`, and a deterministic 3-tier priority sort (primary content links first, pagination transitions second, boilerplate navigation third). Verified with 171 web-extractor tests (14 internal link discovery integration tests), zero clippy warnings, zero fmt diffs, cargo deny clean, and 100% passing quality gates (5/5).

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 134 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1` – `6.4.4`, `7.1.1` – `7.4.4`, `8.1.1` – `8.1.4`, `8.2.1`, `8.2.2` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 63 | `8.2.3` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
