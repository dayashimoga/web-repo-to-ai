# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION` (`IN_PROGRESS`)  
**Current Phase:** `Phase 3.1 — Secure acquisition` (`IN_PROGRESS`)  
**Active Subphase:** `3.1.2 — DNS resolution, IP pinning, and DNS-rebinding protection in crates/url-security` (`NOT_STARTED` / `Next Eligible Subphase`)  
**Last Completed Subphase:** `3.1.1 — URL parsing, normalization, and validation in crates/url-security and crates/web-extractor` (`ACCEPTED`)  
**Next Upcoming Subphase:** `3.1.2 — DNS resolution, IP pinning, and DNS-rebinding protection in crates/url-security`

---

## 1. Executive Summary

Stage 0 (`0.1.1` – `0.3.5`, 15 subphases), Stage 1 (`1.1.1` – `1.4.3`, 15 subphases), and Stage 2 (`2.1.1` – `2.4.4`, 15 subphases) are 100% complete, verified, certified, and accepted. **Stage 3 ("Universal Webpage Extraction") is officially in progress with Subphase `3.1.1` ("URL parsing, normalization, and validation in crates/url-security and crates/web-extractor") successfully implemented, verified, and accepted.** `crates/url-security` establishes zero-trust WHATWG URL parsing, query canonicalization with tracking parameter stripping (`utm_*`, `fbclid`, `gclid`), default-port stripping (`:80`, `:443`), path duplicate-slash collapse, and rigorous SSRF defenses covering IPv4 loopback, RFC 1918 private, link-local / cloud metadata (`169.254.169.254`), CGNAT, multicast/reserved, IPv6 unique-local, link-local, IPv4-mapped IPv6, 6to4/Teredo tunnels, and alternative integer encodings (POSIX `inet_aton` decimal/hex/octal). `crates/web-extractor` integrates `ValidatedUrl` into `WebTarget` with heuristic classification (`Documentation`, `Article`, `FileResource`, `GenericWeb`), relative link resolution, and subpath detection. 163/163 automated Rust tests pass across all workspace crates (16 in `crates/url-security`, 7 in `crates/web-extractor`), 184/184 frontend tests pass in `apps/web`, and all 5 monorepo verification gates pass cleanly with zero failures. Subphase `3.1.2` is queued as the next active subphase.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 45 | `0.1.1` – `0.3.5`, `1.1.1` – `1.4.3`, `2.1.1` – `2.4.4`, `3.1.1` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 152 | `3.1.2` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
