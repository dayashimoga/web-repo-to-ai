# CHANGELOG (`CHANGELOG.md`)

**Policy:** Append-only chronological record of verified project changes by Stage, Phase, and Subphase. Never overwrite or delete historical entries.

---

## [Unreleased] — Stage 1: Frontend and Product Experience

---

## Stage 0: Project Bootstrap and Engineering Foundation (`COMPLETED` / `ACCEPTED`)

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.5`)

#### Added
- **`[0.3.5]` Setup Documentation, Disaster Recovery Runbooks & Stage 0 Certification:**
  - Authored comprehensive Environment Variables Reference in `docs/DEPLOYMENT.md` detailing all 19 configuration variables from `deploy/.env.example` (PostgreSQL, Axum API, Worker, Playwright, SvelteKit, and GitHub App credentials).
  - Authored PostgreSQL Logical Backup & Restore Runbook (`pg_dump`/`pg_restore`) for both Windows PowerShell and POSIX bash with clean table recreation and volume copying.
  - Authored Artifact Storage Volume Backup & Restore Runbook for `deploy_artifact_data`.
  - Authored Emergency Cold Disaster Recovery Runbook guaranteeing complete system recovery from cold workstation restart or storage corruption in under 5 minutes.
  - Authored Production Hardening Guidelines covering Caddy automatic HTTPS reverse proxy configuration with TLS headers, secret manager injection, block-level encryption at rest, AES-256-GCM token encryption, read-only container filesystems, dropped Linux capabilities, and container resource limits.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 223/223 automated checks.
  - Formally certified completion of **Stage 0: Project Bootstrap and Engineering Foundation** (Phases `0.1`, `0.2`, `0.3`, 15 subphases `0.1.1` – `0.3.5` all `ACCEPTED`).

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.3`)

#### Added
- **`[0.3.3]` Development Environment Lifecycle Scripts:**
  - Authored `scripts/dev-up.ps1` and `scripts/dev-up.sh` for starting the compose stack, ensuring `deploy/.env` initialization, verifying the 30 GB storage ceiling prior to launch, supporting service subsets (`-Services`) and image rebuilding (`-Build`), and awaiting database healthcheck readiness.
  - Authored `scripts/dev-down.ps1` and `scripts/dev-down.sh` for service teardown with interactive protection for persistent database volumes (`postgres_data`, `artifact_data`) and post-teardown cache pruning (`-PruneCaches`).
  - Authored `scripts/dev-logs.ps1` and `scripts/dev-logs.sh` supporting service filtering, line tailing, and real-time log streaming (`-Follow`).
  - Authored `scripts/dev-restart.ps1` and `scripts/dev-restart.sh` for safe service restarts and healthcheck validation.
  - Authored `scripts/dev-status.ps1` and `scripts/dev-status.sh` reporting runtime container state, healthcheck status, exposed ports, and storage budget compliance in table and structured JSON formats.
  - Documented complete developer lifecycle workflow in `docs/DEVELOPMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 213/213 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.2: GitHub and CI Foundation (`0.2.1` – `0.2.5`)

#### Added
- **`[0.2.5]` Security & Dependency Scanning:**
  - Configured `deny.toml` with strict RustSec advisory checks, permissive license allowlists (MIT, Apache-2.0, BSD, ISC, Unicode, PostgreSQL, CC0, Zlib, Unlicense), copyleft rejection, and registry restrictions.
  - Authored `.github/workflows/security.yml` executing secret leakage scans, `cargo-deny` checks, and `npm audit`.
  - Authored `.github/workflows/dependency-review.yml` gating PRs on dependency vulnerabilities and license compliance.
  - Authored `.github/workflows/codeql.yml` running automated CodeQL static security analysis.
  - Extended verification suite (`scripts/verify-bootstrap.ps1` & `scripts/check-all.ps1`) to 187 automated checks.
- **`[0.2.4]` Linting, Formatting & Quality Check Runners:**
  - Configured `rustfmt.toml` enforcing Rust 2021 edition, 100 character width, Unix LF line endings, import reordering, and comment formatting.
  - Configured `.prettierrc.json` and `.prettierignore` for web, JSON, Markdown, and TypeScript styling.
  - Authored cross-platform monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `scripts/verify-bootstrap.ps1` with lint/format configuration assertions.
- **`[0.2.3]` GitHub Actions CI/CD Workflows:**
  - Authored `.github/workflows/ci.yml` with concurrency controls, bootstrap integrity checks, and conditional Rust and SvelteKit test gates.
  - Authored `.github/workflows/build.yml` with separate artifact uploads for frontend build bundle, backend binaries, and SHA-256 checksums.
  - Authored `.github/workflows/container.yml` auditing Containerfile specifications and enforcing the 30 GB storage ceiling.
  - Authored `.github/workflows/release.yml` with automated release gate verification, CycloneDX SBOM generation, SHA-256 checksums, and GitHub Release asset publishing.
  - Created `scripts/validate-workflows.py` for automated PyYAML validation and integrated it into `scripts/verify-bootstrap.ps1`.
- **`[0.2.2]` Branch Protection & Repository Governance:**
  - Configured repository merge strategy via GitHub REST API: disabled merge commits (`allow_merge_commit: false`), enabled squash merge (`allow_squash_merge: true`), and enabled rebase merge (`allow_rebase_merge: true`) to enforce linear history.
  - Enabled automatic head branch pruning on PR merge (`delete_branch_on_merge: true`).
  - Enabled Dependabot vulnerability alerts and automated security fixes.
  - Enforced branch protection on `main`: blocked force pushes (`allow_force_pushes: false`), blocked branch deletions (`allow_deletions: false`), and required PR conversation resolution (`required_conversation_resolution: true`).
  - Documented repository governance and protection rules in `docs/DEVELOPMENT.md`.
- **`[0.2.1]` GitHub Remote Repository & Push:**
  - Verified Git Credential Manager OAuth scopes (`gist, repo, workflow`) for authenticated user `dayashimoga`.
  - Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai` via GitHub REST API with issues enabled.
  - Configured Git remote `origin` pointing to `https://github.com/dayashimoga/web-repo-to-ai.git`.
  - Pushed `main` branch with upstream tracking (`git push -u origin main`).
  - Verified remote tracking and commit SHA alignment (`98cc1decf23f270aef970877eb3a4b574b86d822 refs/heads/main`).

### 2026-11-16 — Phase 0.1: Repository and Governance Initialization (`0.1.1` – `0.1.5`)

#### Added
- **`[0.1.1]` Environment & Tooling Inspection:**
  - Verified Windows host environment (14 logical processors, 16.64 GB physical RAM, 33.51 GB free disk on `C:`), `git 2.55.0.windows.5`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (`podman-machine-default` WSL VM with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager authentication status (`dayashimoga`).
- **`[0.1.2]` Monorepo & Git Foundation (`58e0f80`):**
  - Initialized local Git repository on `main` branch with LF line-ending normalization (`.gitattributes`, `core.autocrlf=false`).
  - Added `.gitignore`, `.editorconfig`, MIT `LICENSE`, and `README.md`.
  - Created monorepo structure across `apps/web`, `services/{api,worker,browser-worker}`, 15 shared Rust crate directories (`crates/{core,cdm,url-security,web-extractor,github-extractor,code-intelligence,normalizer,deduplication,tokenizer,context-optimizer,chunking,exporters,storage,job-engine,observability}`), `docs/`, `deploy/`, `scripts/`, `.agent/`, and `.github/workflows/`.
- **`[0.1.3]` Engineering Contract, Requirements & Baseline Documentation (`6372d8a`):**
  - Created `GLOBAL_ENGINEERING_CONTRACT.md` enforcing product integrity, CDM-centered Rust architecture, tiered performance, completeness accounting, SSRF/OAuth security, and one-subphase-at-a-time execution.
  - Created `REQUIREMENTS.md`, `docs/REQUIREMENTS.md`, and `DECISIONS.md` (`ADR-001` through `ADR-007`).
  - Created `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DATA_MODEL.md`, `docs/API.md`, `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md`, `docs/USER_GUIDE.md`, `docs/TESTING.md`, `docs/PERFORMANCE.md`, and `docs/LICENSING.md`.
- **`[0.1.4]` Agent System, Skills, Rules & Workflows (`f2aa8e9`):**
  - Created `AGENTS.md` and registered 9 specialized agent roles (`architecture-lead`, `frontend-engineer`, `rust-backend-engineer`, `extraction-engineer`, `github-integration-engineer`, `security-reviewer`, `qa-engineer`, `devops-engineer`, `documentation-reviewer`) in the Antigravity runtime and `.agent/agents/*.md`.
  - Created 10 domain skills under `.agent/skills/*/SKILL.md` (`architecture`, `rust-engineering`, `frontend-engineering`, `web-extraction`, `github-integration`, `context-optimization`, `security-engineering`, `testing-quality`, `devops-cicd`, `documentation`).
  - Created 10 engineering rules under `.agent/rules/00-global-engineering.md` through `09-no-mock-no-placeholder.md`.
  - Created 10 reusable workflows under `.agent/workflows/*.md`.
- **`[0.1.5]` Staged Roadmap, Ledger & Bootstrap Verification:**
  - Created `ROADMAP.md` and `docs/ROADMAP.md` defining all 12 stages (`Stage 0` – `Stage 11`), 48 phases, and 197 subphases.
  - Created `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1`.
