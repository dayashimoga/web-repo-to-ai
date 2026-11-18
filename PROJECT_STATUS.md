# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-18  
**Current Stage:** `STAGE 7 — AI CONTEXT AND TOKEN OPTIMIZATION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 7.1 — Content optimization` (`IN_PROGRESS`)  
**Active Subphase:** `None (Awaiting user confirmation to begin Subphase 7.1.2)`  
**Last Completed Subphase:** `7.1.1 — Structural normalization in crates/normalizer & crates/cdm` (`ACCEPTED`)  
**Next Upcoming Subphase:** `7.1.2 — Exact duplicate removal in crates/deduplication`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), Stage 2 (`2.1.1` – `2.4.4`, 15 subphases), Stage 3 (`3.1.1` – `3.4.4`, 16 subphases), Stage 4 (`4.1.1` – `4.4.4`, 16 subphases), Stage 5 (`5.1.1` – `5.5.4`, 20 subphases), and Stage 6 (`6.1.1` – `6.4.4`, 16 subphases) are 100% complete, verified, and accepted across 112 subphases. Subphase `7.1.1` ("Structural normalization in crates/normalizer & crates/cdm") implements deterministic character encoding sanitization (BOM stripping, null bytes, zero-width chars, unprintable controls, smart quotes), whitespace and line-ending standardization (CRLF/CR -> Unix LF, blank line limits, trailing whitespace stripping), Markdown syntax canonicalization (code fence languages, heading levels, list markers, GFM table rectangularization), kind-specific normalization with 100% semantic code fidelity preservation, document-level orchestration, cryptographic BLAKE3 content hash recalculation, and quantitative `NormalizationReport` generation with 29 passing tests.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 113 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` – `3.4.4`, `4.1.1` – `4.4.4`, `5.1.1` – `5.5.4`, `6.1.1` – `6.4.4`, `7.1.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 84 | `7.1.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
