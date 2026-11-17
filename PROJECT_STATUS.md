# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION`  
**Current Phase:** `Phase 0.2 — GitHub and CI foundation` (`IN_PROGRESS`)  
**Last Completed Subphase:** `0.2.3 — Create GitHub Actions CI workflows` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `0.2.4 — Configure linting, formatting and test checks across Rust and Frontend`

---

## 1. Executive Summary

Phase `0.1` (`0.1.1` through `0.1.5`) and Subphases `0.2.1` – `0.2.3` have been implemented, verified, and pushed to the remote GitHub repository:
- **Environment & Tooling (`0.1.1`):** Inspected Windows host (14 logical cores, 16.64 GB RAM, 33.51 GB free disk), verified `git 2.55.0`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (WSL machine running with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager account (`dayashimoga`).
- **Monorepo & Git (`0.1.2`):** Initialized Git repository (`main`), configured LF line-ending rules, and created all 27 monorepo directories (`apps/web`, `services/{api,worker,browser-worker}`, 15 shared `crates/*`, `docs/`, `deploy/`, `scripts/`, `.agent/*`, `.github/workflows`).
- **Engineering Governance & Docs (`0.1.3`):** Created `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md` (`ADR-001`–`ADR-007`), and all 11 baseline specifications in `docs/`.
- **Agents, Skills, Rules & Workflows (`0.1.4`):** Created `AGENTS.md`, registered and persisted 9 specialized agent roles (`.agent/agents/*.md`), 10 domain skills (`.agent/skills/*/SKILL.md`), 10 engineering rules (`.agent/rules/*.md`), and 10 reusable workflows (`.agent/workflows/*.md`).
- **Roadmap & Ledgers (`0.1.5`):** Created `ROADMAP.md`, `docs/ROADMAP.md` (12 stages, 48 phases, 197 subphases), `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and verified all deliverables with `scripts/verify-bootstrap.ps1`.
- **GitHub Repository & Push (`0.2.1`):** Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai`, configured remote `origin`, pushed `main` branch with upstream tracking, and verified commit SHA alignment (`git ls-remote origin`).
- **Repository Governance & Protection (`0.2.2`):** Configured repository settings (linear history via squash/rebase merges, merge commits disabled, automatic branch pruning on merge, Dependabot vulnerability alerts and automated security fixes) and active branch protection on `main` (force pushes blocked, deletions blocked, conversation resolution required). Documented in `docs/DEVELOPMENT.md`.
- **GitHub Actions CI/CD Workflows (`0.2.3`):** Created executable workflows (`ci.yml`, `build.yml`, `container.yml`, `release.yml`) with least-privilege permissions, concurrency controls, standalone artifact uploads (frontend build, backend binaries, checksums), 30 GB container ceiling audit, CycloneDX SBOM generation, and `scripts/validate-workflows.py` validation.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 8 | `0.1.1`, `0.1.2`, `0.1.3`, `0.1.4`, `0.1.5`, `0.2.1`, `0.2.2`, `0.2.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 189 | `0.2.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
