# PROJECT STATUS (`PROJECT_STATUS.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Last Updated:** 2026-11-17  
**Current Stage:** `STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION`  
**Current Phase:** `Phase 0.3 — Containerized development foundation` (`IN_PROGRESS`)  
**Last Completed Subphase:** `0.3.3 — Create development environment scripts` (`ACCEPTED`)  
**Next Eligible Subphase (Awaiting Explicit User Approval):** `0.3.4 — Validate reproducible container builds`

---

## 1. Executive Summary

Phase `0.1` (`0.1.1` – `0.1.5`), Phase `0.2` (`0.2.1` – `0.2.5`), and Subphases `0.3.1` – `0.3.3` have been implemented, verified, and pushed to the remote GitHub repository:
- **Environment & Tooling (`0.1.1`):** Inspected Windows host (14 logical cores, 16.64 GB RAM, 33.51 GB free disk), verified `git 2.55.0`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (WSL machine running with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager account (`dayashimoga`).
- **Monorepo & Git (`0.1.2`):** Initialized Git repository (`main`), configured LF line-ending rules, and created all 27 monorepo directories (`apps/web`, `services/{api,worker,browser-worker}`, 15 shared `crates/*`, `docs/`, `deploy/`, `scripts/`, `.agent/*`, `.github/workflows`).
- **Engineering Governance & Docs (`0.1.3`):** Created `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md` (`ADR-001`–`ADR-007`), and all 11 baseline specifications in `docs/`.
- **Agents, Skills, Rules & Workflows (`0.1.4`):** Created `AGENTS.md`, registered and persisted 9 specialized agent roles (`.agent/agents/*.md`), 10 domain skills (`.agent/skills/*/SKILL.md`), 10 engineering rules (`.agent/rules/*.md`), and 10 reusable workflows (`.agent/workflows/*.md`).
- **Roadmap & Ledgers (`0.1.5`):** Created `ROADMAP.md`, `docs/ROADMAP.md` (12 stages, 48 phases, 197 subphases), `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and verified all deliverables with `scripts/verify-bootstrap.ps1`.
- **GitHub Repository & Push (`0.2.1`):** Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai`, configured remote `origin`, pushed `main` branch with upstream tracking, and verified commit SHA alignment (`git ls-remote origin`).
- **Repository Governance & Protection (`0.2.2`):** Configured repository settings (linear history via squash/rebase merges, merge commits disabled, automatic branch pruning on merge, Dependabot vulnerability alerts and automated security fixes) and active branch protection on `main` (force pushes blocked, deletions blocked, conversation resolution required). Documented in `docs/DEVELOPMENT.md`.
- **GitHub Actions CI/CD Workflows (`0.2.3`):** Created executable workflows (`ci.yml`, `build.yml`, `container.yml`, `release.yml`) with least-privilege permissions, concurrency controls, standalone artifact uploads (frontend build, backend binaries, checksums), 30 GB container ceiling audit, CycloneDX SBOM generation, and verified execution on live GitHub runners.
- **Linting, Formatting & Quality Check Runners (`0.2.4`):** Created `rustfmt.toml` (Rust 2021, 100 max width, Unix LF), `.prettierrc.json`, `.prettierignore`, and cross-platform quality audit scripts (`scripts/check-all.ps1` and `scripts/check-all.sh`). All 4 verification gates passed cleanly with 0 failures.
- **Security & Dependency Scanning (`0.2.5`):** Created `deny.toml` (RustSec vulnerability enforcement, permissive license whitelist), `.github/workflows/security.yml` (secret scan, cargo-deny, npm audit), `.github/workflows/dependency-review.yml` (PR dependency license gate), and `.github/workflows/codeql.yml` (CodeQL static security analysis). Verified across 187 checks.
- **Podman Development Foundation (`0.3.1`):** Created `deploy/podman-compose.yml` (5 services, 2 isolated networks, 2 persistent volumes, PostgreSQL 16 Alpine healthcheck), multi-stage Containerfiles (`Containerfile.api`, `Containerfile.worker`, `Containerfile.browser-worker`, `Containerfile.web`), and `deploy/.env.example`. Validated via `podman-compose config` and 194 automated checks.
- **Storage Ceiling Enforcement (`0.3.2`):** Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to enforce the 30 GB container storage ceiling, host RAM, CPU, and disk space with structured JSON and non-zero exit on breach; created `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning protecting persistent volumes; integrated Resource Budget Gate 5 into `scripts/check-all.*`; updated `docs/DEPLOYMENT.md`; verified 199/199 bootstrap checks.
- **Development Lifecycle Scripts (`0.3.3`):** Authored complete cross-platform lifecycle utilities across Windows PowerShell and POSIX bash (`scripts/dev-up.*`, `scripts/dev-down.*`, `scripts/dev-logs.*`, `scripts/dev-restart.*`, `scripts/dev-status.*`); enforced pre-flight storage checks, protected data volumes during teardown, supported service subsets and image rebuilds; documented usage in `docs/DEVELOPMENT.md`; verified 213/213 checks across all gates.

---

## 2. Subphase Status Counter

| Status | Count | Subphases |
| :--- | :---: | :--- |
| `ACCEPTED` | 13 | `0.1.1` – `0.1.5`, `0.2.1` – `0.2.5`, `0.3.1` – `0.3.3` |
| `VERIFIED` | 0 | — |
| `IMPLEMENTED` | 0 | — |
| `IN_PROGRESS` | 0 | — |
| `BLOCKED` | 0 | — |
| `NOT_STARTED` | 184 | `0.3.4` – `11.4.5` |
| **Total** | **197** | `Stage 0` – `Stage 11` |

---

## 3. Environment Notes & Known External Dependencies

1. **GitHub Remote Repository (`0.2.1` `ACCEPTED`):**
   - Remote repository is live at `https://github.com/dayashimoga/web-repo-to-ai`.
   - Local `main` branch tracks `origin/main`.
2. **Database & Browser Services:**
   - `psql` and `sqlx` CLI are not installed directly on the Windows host; per Section 4 ("No host-wide software installations for local development when a containerized alternative exists"), PostgreSQL and Playwright/Chromium will run via Podman containers configured in Phase `0.3`.
