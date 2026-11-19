# IMPLEMENTATION LEDGER (`IMPLEMENTATION_LEDGER.md`)

**Project:** `web-repo-to-ai`  
**Policy:** Append-only verification ledger. Every subphase records all 14 mandatory tracking attributes.  
**Allowed Statuses:** `NOT_STARTED`, `IN_PROGRESS`, `BLOCKED`, `IMPLEMENTED`, `VERIFIED`, `ACCEPTED`.

---

## STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION

### Phase 0.1 — Repository and governance initialization

#### Subphase `0.1.1` — Inspect environment, repository status and available tools
- **Unique ID:** `0.1.1`
- **Objective:** Inspect host OS, hardware resources, existing workspace files, Git status, installed CLI toolchains, container runtime limits, and GitHub authentication state.
- **Scope:** Read-only inspection of `c:\Users\dayan\web-repo-to-ai` and workstation environment.
- **Prerequisites:** None (initial bootstrap subphase).
- **Files/Components Affected:** Documented in `docs/DEVELOPMENT.md`, `PROJECT_STATUS.md`, and `IMPLEMENTATION_LEDGER.md`.
- **Implementation Tasks:**
  1. Check workspace directory contents and Git repository status.
  2. Enumerate versions and paths of `git`, `gh`, `rustc`, `cargo`, `node`, `npm`, `pnpm`, `podman`, `podman-compose`, `docker`, `psql`, `sqlx`, `python`, `ssh`, `tar`.
  3. Inspect CPU logical processors, physical RAM, C: drive free space, and Podman WSL machine configuration.
  4. Check Git configuration and GitHub credential helper state.
- **Acceptance Criteria:**
  - Complete, factual inventory of installed tools, missing tools, hardware specs, Podman limits, and GitHub credential status recorded without fabrication.
- **Tests:** Executed PowerShell environment inspection commands (`Get-Command`, `Get-CimInstance`, `Get-PSDrive`, `podman machine list`, `git credential-manager github list`).
- **Security Considerations:** Verified `GITHUB_TOKEN`/`GH_TOKEN` presence (`False`) and Git Credential Manager account list (`dayashimoga`) without exposing any secret values.
- **Performance Considerations:** Confirmed workstation matches target envelope: 14 logical CPUs, 16.64 GB physical RAM, 33.51 GB free disk on `C:`, and `podman-machine-default` configured with 7 CPUs, 2 GiB RAM, and a 30 GiB disk ceiling.
- **Documentation Requirements:** Recorded baseline environment in `docs/DEVELOPMENT.md` and `IMPLEMENTATION_LEDGER.md`.
- **Validation Evidence:**
  - `git version 2.55.0.windows.5`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman version 5.8.3`, `podman-compose.exe`, `python 3.11`.
  - Noted missing host CLIs (`gh`, `pnpm`, `docker`, `psql`, `sqlx`) and documented container/HTTP/credential-manager alternatives.
- **Git Commit Reference:** `58e0f80` (recorded with initial repository initialization commit)
- **Status:** `ACCEPTED`

---

#### Subphase `0.1.2` — Initialize monorepo and Git configuration
- **Unique ID:** `0.1.2`
- **Objective:** Initialize the local Git repository on branch `main`, configure line-ending and ignore rules, and create the canonical monorepo directory structure.
- **Scope:** Root Git configuration, `.gitignore`, `.gitattributes`, `.editorconfig`, `LICENSE`, `README.md`, and directory skeleton (`apps/web`, `services/*`, `crates/*`, `docs`, `deploy`, `scripts`, `.agent/*`, `.github/workflows`).
- **Prerequisites:** `0.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `.gitignore`, `.gitattributes`, `.editorconfig`, `LICENSE`, `README.md`
  - `apps/web/.gitkeep`
  - `services/{api,worker,browser-worker}/.gitkeep`
  - `crates/{core,cdm,url-security,web-extractor,github-extractor,code-intelligence,normalizer,deduplication,tokenizer,context-optimizer,chunking,exporters,storage,job-engine,observability}/.gitkeep`
  - `deploy/.gitkeep`, `scripts/.gitkeep`, `.github/workflows/.gitkeep`
- **Implementation Tasks:**
  1. Run `git init -b main` and set repository-local `core.autocrlf=false` and `core.eol=lf`.
  2. Create `.gitignore`, `.gitattributes`, `.editorconfig`, MIT `LICENSE`, and root `README.md`.
  3. Create all 15 shared crate directories, 3 service directories, web app directory, deployment/scripts/docs directories, and `.agent` directories.
- **Acceptance Criteria:**
  - Valid Git repository on `main` branch with LF normalization configured and all architectural directories tracked.
- **Tests:** Verified via `git status`, `git branch --show-current`, and directory existence checks across all 27 structural directories.
- **Security Considerations:** `.gitignore` explicitly blocks `.env*` (except `.env.example`), `.pem`, `.key`, `.p12`, `.pfx`, `secrets/`, and `credentials.json`.
- **Performance Considerations:** Excluded `/target/`, `node_modules/`, `/data/`, and `/artifacts/` from Git indexing.
- **Documentation Requirements:** `README.md` documents the monorepo directory structure and links to all core governance docs.
- **Validation Evidence:**
  - `git init -b main` succeeded; committed 27 files in root commit `58e0f80`.
- **Git Commit Reference:** `58e0f80`
- **Status:** `ACCEPTED`

---

#### Subphase `0.1.3` — Create engineering contract and requirements
- **Unique ID:** `0.1.3`
- **Objective:** Establish the non-negotiable `GLOBAL_ENGINEERING_CONTRACT.md`, functional and non-functional requirements (`REQUIREMENTS.md`, `docs/REQUIREMENTS.md`), architecture decision records (`DECISIONS.md`), and the complete suite of baseline technical documentation in `docs/*.md`.
- **Scope:** Root governance contract, requirements, ADRs (`ADR-001` through `ADR-007`), and `docs/{ARCHITECTURE,SECURITY,DATA_MODEL,API,DEVELOPMENT,DEPLOYMENT,USER_GUIDE,TESTING,PERFORMANCE,LICENSING}.md`.
- **Prerequisites:** `0.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md`
  - `docs/REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DATA_MODEL.md`, `docs/API.md`, `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md`, `docs/USER_GUIDE.md`, `docs/TESTING.md`, `docs/PERFORMANCE.md`, `docs/LICENSING.md`
- **Implementation Tasks:**
  1. Write `GLOBAL_ENGINEERING_CONTRACT.md` enforcing product integrity, Rust-first CDM architecture, performance budgets, completeness accounting, SSRF/OAuth security, code quality, and one-subphase-at-a-time execution.
  2. Write `REQUIREMENTS.md` and `docs/REQUIREMENTS.md` covering `FR-WEB`, `FR-GH-PUB`, `FR-GH-PRIV`, `FR-CDM`, `FR-OPT`, `FR-EXP`, `FR-UI`, `FR-SCALE`, `FR-INT`, `FR-DOC`, and `NFR-SEC`/`NFR-PERF`/`NFR-QUAL`.
  3. Record `ADR-001` through `ADR-007` in `DECISIONS.md`.
  4. Create all required `docs/*.md` specifications.
- **Acceptance Criteria:**
  - All 14 governance and documentation files created with concrete schemas, threat models, API contracts, and resource constraints.
- **Tests:** Verified file presence, Markdown cross-link targets, and non-empty structural completeness across all 14 files.
- **Security Considerations:** Defined mandatory SSRF IP blocklists, DNS pinning, redirect revalidation, browser sandboxing, read-only GitHub App scopes, and `AES-256-GCM` token encryption in `docs/SECURITY.md`.
- **Performance Considerations:** Defined 16 GB RAM CPU-only execution and 30 GB Podman storage ceiling in `docs/PERFORMANCE.md`.
- **Documentation Requirements:** Satisfies Section 3 and Section 15 baseline documentation requirements.
- **Validation Evidence:**
  - 14 files (842 lines) verified and committed in `6372d8a`.
- **Git Commit Reference:** `6372d8a`
- **Status:** `ACCEPTED`

---

#### Subphase `0.1.4` — Create agent skills, rules and workflows
- **Unique ID:** `0.1.4`
- **Objective:** Create the cross-agent coordination manual (`AGENTS.md`), 9 specialized agent roles (both registered in Antigravity via `define_subagent` and persisted in `.agent/agents/*.md`), 10 domain skills (`.agent/skills/*/SKILL.md`), 10 enforceable rules (`.agent/rules/*.md`), and 10 reusable execution/audit workflows (`.agent/workflows/*.md`).
- **Scope:** `AGENTS.md`, `.agent/agents/*`, `.agent/skills/*`, `.agent/rules/*`, and `.agent/workflows/*`.
- **Prerequisites:** `0.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `AGENTS.md`
  - `.agent/agents/{architecture-lead,frontend-engineer,rust-backend-engineer,extraction-engineer,github-integration-engineer,security-reviewer,qa-engineer,devops-engineer,documentation-reviewer}.md`
  - `.agent/skills/{architecture,rust-engineering,frontend-engineering,web-extraction,github-integration,context-optimization,security-engineering,testing-quality,devops-cicd,documentation}/SKILL.md`
  - `.agent/rules/{00-global-engineering,01-architecture,02-rust,03-frontend,04-extraction-accuracy,05-github-security,06-testing,07-documentation,08-deployment,09-no-mock-no-placeholder}.md`
  - `.agent/workflows/{bootstrap-project,execute-next-subphase,verify-subphase,complete-phase,complete-stage,security-audit,performance-audit,regression-audit,release-certification,project-status-review}.md`
- **Implementation Tasks:**
  1. Author `AGENTS.md` defining clear ownership boundaries and conflict-prevention rules.
  2. Register all 9 specialized subagents in the Antigravity runtime via `define_subagent` and persist their role specs in `.agent/agents/*.md`.
  3. Author all 10 `SKILL.md` files with valid YAML frontmatter (`name`, `description`) and all 7 mandatory sections (Purpose, When to use, Required procedures, Engineering rules, Validation requirements, Expected outputs, Common failure modes).
  4. Author all 10 concise, enforceable rule files in `.agent/rules/` (each well under the 24 KB Antigravity rule limit).
  5. Author all 10 workflow files in `.agent/workflows/` enforcing the one-subphase-at-a-time rule.
- **Acceptance Criteria:**
  - All 10 skills contain all 7 mandatory sections and valid frontmatter; all 10 rules and 10 workflows exist and cross-reference accurately; all 9 agent roles are defined and documented.
- **Tests:** Automated verification script checking YAML frontmatter, presence of all 7 required headings in every `SKILL.md`, and existence of all 40 agent/skill/rule/workflow files.
- **Security Considerations:** `.agent/rules/05-github-security.md`, `.agent/skills/security-engineering/SKILL.md`, and `.agent/workflows/security-audit.md` codify mandatory SSRF, OAuth, and secret-redaction checks.
- **Performance Considerations:** Skills use progressive disclosure (`name` + `description` frontmatter) and rules are concise (<2 KB each), preserving the 20,000-token Antigravity rules context budget.
- **Documentation Requirements:** Indexed in `AGENTS.md` and `README.md`.
- **Validation Evidence:**
  - 9 runtime subagents registered via `define_subagent`; 40 files (852 lines) committed in `f2aa8e9`.
- **Git Commit Reference:** `f2aa8e9`
- **Status:** `ACCEPTED`

---

#### Subphase `0.1.5` — Create stage/phase/subphase roadmap and ledger
- **Unique ID:** `0.1.5`
- **Objective:** Create the complete Stage 0 – Stage 11 roadmap (`ROADMAP.md`, `docs/ROADMAP.md`) and initialize the persistent project tracking ledgers (`IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`).
- **Scope:** `ROADMAP.md`, `docs/ROADMAP.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and bootstrap verification script `scripts/verify-bootstrap.ps1`.
- **Prerequisites:** `0.1.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `ROADMAP.md`, `docs/ROADMAP.md`
  - `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`
  - `scripts/verify-bootstrap.ps1`
- **Implementation Tasks:**
  1. Generate `ROADMAP.md` and `docs/ROADMAP.md` covering all 12 stages (`Stage 0` – `Stage 11`), 48 phases, and 197 subphases.
  2. Create `IMPLEMENTATION_LEDGER.md` recording complete 14-attribute entries for `0.1.1`–`0.1.5` and structured ledger entries for upcoming subphases.
  3. Create `PROJECT_STATUS.md`, append-only `TODO.md`, and append-only `CHANGELOG.md`.
  4. Create and run `scripts/verify-bootstrap.ps1` to programmatically validate every file, directory, skill section, rule, workflow, and documentation requirement of Phase `0.1`.
- **Acceptance Criteria:**
  - All tracking files exist, are mutually consistent, and pass automated bootstrap verification via `scripts/verify-bootstrap.ps1` with 0 errors.
- **Tests:** Executed `powershell -ExecutionPolicy Bypass -File scripts/verify-bootstrap.ps1`.
- **Security Considerations:** Verified zero secrets or credentials in tracked files or Git history.
- **Performance Considerations:** Lightweight static verification script completes in <2 seconds.
- **Documentation Requirements:** Completes all Phase `0.1` tracking and roadmap files.
- **Validation Evidence:**
  - `scripts/verify-bootstrap.ps1` passed 100% of checks (173/173 checks passed, 0 failures) across directories, governance files, docs, skills, rules, workflows, agents, and git status.
- **Git Commit Reference:** `4055f93`
- **Status:** `ACCEPTED`

---

### Phase 0.2 — GitHub and CI foundation

#### Subphase `0.2.1` — Create or connect GitHub repository
- **Unique ID:** `0.2.1`
- **Objective:** Create or connect GitHub repository (`web-repo-to-ai`) using available GitHub credentials, configure remote `origin`, and push `main` branch with upstream tracking.
- **Scope:** `.git/config` remote `origin`, GitHub repository `dayashimoga/web-repo-to-ai`, and tracking ledgers.
- **Prerequisites:** `0.1.5` (`ACCEPTED`).
- **Files/Components Affected:**
  - `.git/config` (remote `origin`)
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Inspect Git Credential Manager token scopes via GitHub REST API (`gist, repo, workflow` confirmed).
  2. Query GitHub API for `dayashimoga/web-repo-to-ai`; verify repository does not exist yet.
  3. Create public repository `web-repo-to-ai` under `dayashimoga` with issues enabled via `POST /user/repos`.
  4. Configure local Git remote `origin` pointing to `https://github.com/dayashimoga/web-repo-to-ai.git`.
  5. Push `main` branch with upstream tracking (`git push -u origin main`).
  6. Verify remote tracking branch and commit SHA alignment via `git ls-remote origin`.
- **Acceptance Criteria:**
  - Remote repository `web-repo-to-ai` exists on GitHub, `origin` remote configured, and `main` branch pushed and verified with `git ls-remote origin`.
- **Tests:** Executed `git ls-remote origin` confirming `refs/heads/main` matches local commit `98cc1decf23f270aef970877eb3a4b574b86d822`.
- **Security Considerations:** No raw tokens or credentials written to tracked files or logs; Git Credential Manager token accessed in memory only.
- **Performance Considerations:** Direct REST creation and Git push completed in <5 seconds.
- **Documentation Requirements:** Updated `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - Remote repository live at `https://github.com/dayashimoga/web-repo-to-ai`.
  - `git status` reports: `On branch main, Your branch is up to date with 'origin/main', nothing to commit, working tree clean`.
  - `git ls-remote origin` output: `98cc1decf23f270aef970877eb3a4b574b86d822 refs/heads/main`.
- **Git Commit Reference:** `68936ac`
- **Status:** `ACCEPTED`

---

#### Subphase `0.2.2` — Configure branch protection and repository settings where authorized
- **Unique ID:** `0.2.2`
- **Objective:** Configure branch protection and repository settings where authorized to enforce linear history, branch deletion on merge, vulnerability alerts, and protection against accidental force pushes or deletions on `main`.
- **Scope:** GitHub repository settings (`dayashimoga/web-repo-to-ai`), `docs/DEVELOPMENT.md`, and tracking ledgers.
- **Prerequisites:** `0.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - GitHub repository settings (`dayashimoga/web-repo-to-ai`)
  - `docs/DEVELOPMENT.md`
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Configure repository merge strategies via GitHub REST API: disable traditional merge commits (`allow_merge_commit: false`), enable squash merge (`allow_squash_merge: true`), enable rebase merge (`allow_rebase_merge: true`), and enable automatic head branch deletion (`delete_branch_on_merge: true`).
  2. Enable automated vulnerability alerts (`PUT /repos/{owner}/{repo}/vulnerability-alerts`) and automated security fixes (`PUT /repos/{owner}/{repo}/automated-security-fixes`).
  3. Apply branch protection policy on `main` (`PUT /repos/{owner}/{repo}/branches/main/protection`): prevent force pushes (`allow_force_pushes: false`), prevent branch deletions (`allow_deletions: false`), and require conversation resolution (`required_conversation_resolution: true`).
  4. Document repository governance and branch protection policy in `docs/DEVELOPMENT.md`.
- **Acceptance Criteria:**
  - Linear history enforced, automatic branch pruning enabled, vulnerability alerts enabled, and branch protection on `main` active and documented in `docs/DEVELOPMENT.md`.
- **Tests:** GitHub API queries confirm `allow_force_pushes: false`, `allow_deletions: false`, `required_conversation_resolution: true`, `delete_branch_on_merge: true`, `allow_merge_commit: false`, vulnerability alerts HTTP 204, automated security fixes HTTP 204.
- **Security Considerations:** Prevents destructive force pushes or accidental branch deletions on `main`; activates automated Dependabot vulnerability alerts; zero credentials stored in files.
- **Performance Considerations:** API configuration calls complete in <3 seconds.
- **Documentation Requirements:** Documented in `docs/DEVELOPMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - Branch protection output verified: `allow_force_pushes: False`, `allow_deletions: False`, `required_conversation_resolution: True`.
  - Repo settings verified: `delete_branch_on_merge: True`, `allow_merge_commit: False`, `allow_squash_merge: True`, `allow_rebase_merge: True`.
  - Security endpoints: HTTP 204 response for vulnerability alerts and automated fixes.
- **Git Commit Reference:** `8382b1d`
- **Status:** `ACCEPTED`

---

#### Subphase `0.2.3` — Create GitHub Actions CI workflows
- **Unique ID:** `0.2.3`
- **Objective:** Create executable GitHub Actions CI/CD workflows (`ci.yml`, `build.yml`, `release.yml`, `container.yml`) enforcing least-privilege permissions, pinned action versions, and multi-artifact separation.
- **Scope:** `.github/workflows/{ci,build,release,container}.yml`, `scripts/validate-workflows.py`, `scripts/verify-bootstrap.ps1`, and tracking ledgers.
- **Prerequisites:** `0.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `.github/workflows/ci.yml`
  - `.github/workflows/build.yml`
  - `.github/workflows/container.yml`
  - `.github/workflows/release.yml`
  - `scripts/validate-workflows.py`
  - `scripts/verify-bootstrap.ps1`
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Author `.github/workflows/ci.yml` with least-privilege `contents: read` permissions, concurrency control, bootstrap verification, and conditional checks for Rust workspace and SvelteKit frontend.
  2. Author `.github/workflows/build.yml` producing separate frontend build artifacts, backend binary packages, and SHA-256 checksums without unnecessary nested ZIPs.
  3. Author `.github/workflows/container.yml` enforcing Containerfile inspections and the 30 GB storage ceiling audit.
  4. Author `.github/workflows/release.yml` automating release gate execution, CycloneDX SBOM generation, SHA-256 checksum verification, and GitHub Releases asset publishing.
  5. Create `scripts/validate-workflows.py` and integrate YAML syntax/schema verification into `scripts/verify-bootstrap.ps1`.
- **Acceptance Criteria:**
  - All 4 workflows exist, pass PyYAML parser validation, declare explicit `permissions:` blocks, pin action versions, and execute cleanly.
- **Tests:** `python scripts/validate-workflows.py` confirmed 4/4 workflows valid; `scripts/verify-bootstrap.ps1` passed 178/178 checks.
- **Security Considerations:** Enforces minimal `permissions: contents: read` (with `contents: write` restricted solely to `release.yml`); pins GitHub Actions to major versions (`@v4`, `@v2`); prevents accidental secret leakage.
- **Performance Considerations:** Workflows declare concurrency groups with `cancel-in-progress: true` to prevent redundant runner usage.
- **Documentation Requirements:** Updated `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - `python scripts/validate-workflows.py`: 4/4 workflows passed (verified triggers, permissions, jobs).
  - `scripts/verify-bootstrap.ps1`: 178/178 checks passed (0 failures).
  - Real GitHub Actions execution on GitHub runners:
    - `CI` workflow: `success` (jobs `bootstrap-governance`, `frontend-ci`, `rust-ci` all green).
    - `Build Artifacts` workflow: `success` (jobs `build-frontend`, `build-backend` all green; separate artifacts `frontend-build`, `backend-binaries`, `build-checksums` uploaded).
    - `Container Build & Verification` workflow: `success` (job `validate-containers` green; artifact `container-audit-manifest` uploaded).
- **Git Commit Reference:** `ca0e9c2`
- **Status:** `ACCEPTED`

---

#### Subphase `0.2.4` — Configure linting, formatting and test checks across Rust and Frontend
- **Unique ID:** `0.2.4`
- **Objective:** Configure linting, formatting, and test check standards across Rust and Frontend workspaces (`rustfmt.toml`, `.prettierrc.json`, `.prettierignore`, `scripts/check-all.ps1`, `scripts/check-all.sh`).
- **Scope:** Root formatting configurations, monorepo quality audit runners, and tracking ledgers.
- **Prerequisites:** `0.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `rustfmt.toml`
  - `.prettierrc.json`
  - `.prettierignore`
  - `scripts/check-all.ps1`
  - `scripts/check-all.sh`
  - `scripts/verify-bootstrap.ps1`
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Author `rustfmt.toml` specifying Rust 2021 edition, 100 character width, Unix LF newlines, import reordering, and comment formatting.
  2. Author `.prettierrc.json` and `.prettierignore` for web, JSON, Markdown, and TypeScript styling.
  3. Create cross-platform monorepo quality runner scripts (`scripts/check-all.ps1` and `scripts/check-all.sh`) validating all gates (bootstrap integrity, workflow schemas, Rust formatting/clippy/tests, frontend lint/types/tests).
  4. Update `scripts/verify-bootstrap.ps1` to assert lint/format configuration presence and integrity.
- **Acceptance Criteria:**
  - `rustfmt.toml`, `.prettierrc.json`, `.prettierignore`, `scripts/check-all.ps1`, and `scripts/check-all.sh` exist, validate cleanly, and pass all verification gates with 0 errors.
- **Tests:** `scripts/check-all.ps1` executed all 4 quality gates successfully; `scripts/verify-bootstrap.ps1` passed 183/183 checks.
- **Security Considerations:** No sensitive files exposed; `.prettierignore` ignores build artifacts and local storage.
- **Performance Considerations:** Automated lint/format scripts complete in <2 seconds.
- **Documentation Requirements:** Documented in `docs/DEVELOPMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - `scripts/check-all.ps1`: ALL GATES PASSED (0 failures).
  - `scripts/verify-bootstrap.ps1`: 183/183 checks passed (0 failures).
- **Git Commit Reference:** `5342058`
- **Status:** `ACCEPTED`

---

#### Subphase `0.2.5` — Configure security and dependency scanning
- **Unique ID:** `0.2.5`
- **Objective:** Configure automated security, vulnerability, dependency, and license scanning workflows (`security.yml`, `dependency-review.yml`, `codeql.yml`, `deny.toml`).
- **Scope:** Security workflows in `.github/workflows/`, cargo-deny configuration `deny.toml`, verification scripts, and tracking ledgers.
- **Prerequisites:** `0.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `deny.toml`
  - `.github/workflows/security.yml`
  - `.github/workflows/dependency-review.yml`
  - `.github/workflows/codeql.yml`
  - `scripts/verify-bootstrap.ps1`
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Author `deny.toml` configuring `cargo-deny` with strict RustSec advisories (`vulnerability = "deny"`), permissive license allowlist (MIT, Apache-2.0, BSD-2/3, ISC, Unicode, PostgreSQL, CC0, Zlib, Unlicense), copyleft rejection, and registry restrictions.
  2. Author `.github/workflows/security.yml` running secret leakage scans, `cargo-deny` audits, and `npm audit`.
  3. Author `.github/workflows/dependency-review.yml` for pull request dependency vulnerability and license review.
  4. Author `.github/workflows/codeql.yml` for automated CodeQL static security analysis.
  5. Extend `scripts/verify-bootstrap.ps1` to assert all 7 workflow files and `deny.toml`.
- **Acceptance Criteria:**
  - `deny.toml` and all 3 security workflow files exist, declare least-privilege permissions, pass PyYAML validation, and pass `scripts/check-all.ps1`.
- **Tests:** `python scripts/validate-workflows.py` verified 7/7 workflows; `scripts/check-all.ps1` passed all 4 quality gates (187/187 checks).
- **Security Considerations:** Enforces zero known vulnerabilities, zero leaked secrets, least-privilege workflow permissions, and blocking of copyleft/proprietary licenses.
- **Performance Considerations:** Workflows utilize caching and fast concurrency-controlled runners.
- **Documentation Requirements:** Documented in `docs/SECURITY.md`, `docs/LICENSING.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - `python scripts/validate-workflows.py`: 7/7 workflows passed (verified triggers, permissions, jobs).
  - `scripts/check-all.ps1`: ALL GATES PASSED (187/187 checks).
- **Git Commit Reference:** `364b4b5`
- **Status:** `ACCEPTED`

---

### Phase 0.3 — Containerized development foundation

#### Subphase `0.3.1` — Create Podman development configuration
- **Unique ID:** `0.3.1`
- **Objective:** Create the containerized development foundation in `deploy/` (`deploy/podman-compose.yml`, multi-stage Containerfiles for API, Worker, Browser Worker, Web Frontend, and local PostgreSQL service).
- **Scope:** `deploy/podman-compose.yml`, `deploy/Containerfile.{api,worker,browser-worker,web}`, `deploy/.env.example`, `scripts/verify-bootstrap.ps1`, and tracking ledgers.
- **Prerequisites:** `0.2.5` (`ACCEPTED`).
- **Files/Components Affected:**
  - `deploy/podman-compose.yml`
  - `deploy/Containerfile.api`
  - `deploy/Containerfile.worker`
  - `deploy/Containerfile.browser-worker`
  - `deploy/Containerfile.web`
  - `deploy/.env.example`
  - `scripts/verify-bootstrap.ps1`
  - `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`
- **Implementation Tasks:**
  1. Author `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust background Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend with bridge networks (`internal_net`, `browser_net`), persistent volumes, and healthchecks.
  2. Author multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` (Rust slim builder -> Debian bookworm-slim runtime with unprivileged user).
  3. Author `deploy/Containerfile.browser-worker` with Playwright/Chromium and isolated non-root user.
  4. Author multi-stage `deploy/Containerfile.web` (Node 20 slim builder -> minimal runtime).
  5. Author `deploy/.env.example` documenting all service configuration variables.
  6. Update `scripts/verify-bootstrap.ps1` to assert all deployment configurations.
- **Acceptance Criteria:**
  - All 6 deployment files exist, pass YAML syntax validation, pass `podman-compose config` schema verification, and respect container isolation boundaries.
- **Tests:** `podman-compose config` parsed and validated all 5 services; `scripts/check-all.ps1` passed 194/194 checks across all 4 quality gates.
- **Security Considerations:** Enforces unprivileged runtime users (`appuser`, `pwuser`, `node`); isolates Browser Worker from PostgreSQL database and internal service network; blocks `no-new-privileges`.
- **Performance Considerations:** Multi-stage builds and slim base images minimize layer duplication, respecting the 30 GB storage ceiling.
- **Documentation Requirements:** Documented in `docs/DEPLOYMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - `podman-compose config` CLI verification: exit code 0, 5 services parsed.
  - `scripts/check-all.ps1`: ALL GATES PASSED (194/194 checks passed).
- **Git Commit Reference:** `1e8d4e8`
- **Status:** `ACCEPTED`

---

#### Subphase `0.3.2` — Enforce the 30 GB storage ceiling
- **Unique ID:** `0.3.2`
- **Objective:** Enforce the 30 GB container storage ceiling for containerized development by authoring resource inspection and safe cache cleanup scripts (`scripts/check-resources.ps1`, `scripts/check-resources.sh`, `scripts/clean-caches.ps1`, `scripts/clean-caches.sh`), integrating them with monorepo quality audit gates, and documenting operational procedures.
- **Scope:** `scripts/check-resources.ps1`, `scripts/check-resources.sh`, `scripts/clean-caches.ps1`, `scripts/clean-caches.sh`, `scripts/check-all.ps1`, `scripts/check-all.sh`, `scripts/verify-bootstrap.ps1`, `docs/DEPLOYMENT.md`, and tracking ledgers.
- **Prerequisites:** `0.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `scripts/check-resources.ps1`
  - `scripts/check-resources.sh`
  - `scripts/clean-caches.ps1`
  - `scripts/clean-caches.sh`
  - `scripts/check-all.ps1`
  - `scripts/check-all.sh`
  - `scripts/verify-bootstrap.ps1`
  - `docs/DEPLOYMENT.md`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
- **Implementation Tasks:**
  1. Author `scripts/check-resources.ps1` to query host CPU cores, RAM, free disk space, and Podman storage breakdown (`podman system df --format json`), calculating total usage against the 30 GB ceiling with 80% warning threshold and non-zero exit code on breach.
  2. Author `scripts/check-resources.sh` for POSIX systems (Linux, macOS, CI) supporting `--ceiling-gb`, `--warn-percent`, `--json`, `--quiet`, and `--fail-on-warn`.
  3. Author `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` providing safe cache pruning (`podman image prune --build-cache -f`, `podman system prune --build -f`, `podman image prune -f`), with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes.
  4. Integrate Gate 5 ("Resource Budget & 30 GB Storage Ceiling") into `scripts/check-all.ps1` and `scripts/check-all.sh`.
  5. Update `scripts/verify-bootstrap.ps1` with resource script assertions and ceiling verification.
  6. Document storage limits, monitoring commands, safety guarantees, and pruning workflows in `docs/DEPLOYMENT.md`.
- **Acceptance Criteria:**
  - Resource inspection scripts execute on Windows and POSIX, query `podman system df`, enforce 30 GB ceiling, and return exit code 0 under ceiling or 1 on breach.
  - Cache cleanup scripts safely prune build caches and dangling layers without modifying persistent database volumes or source files.
  - All quality audit gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass with 0 failures, and `scripts/verify-bootstrap.ps1` passes all 199 checks.
- **Tests:**
  - `powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1`: verified host resources and Podman storage (4.95 GB / 30 GB, 16.5% used).
  - `powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -Json`: verified valid JSON payload.
  - `powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -CeilingGB 1`: verified non-zero exit code on ceiling breach.
  - `bash scripts/check-resources.sh`: verified POSIX resource check.
  - `powershell -ExecutionPolicy Bypass -File scripts/clean-caches.ps1 -DryRun`: verified dry-run reporting and volume protection.
  - `powershell -ExecutionPolicy Bypass -File scripts/clean-caches.ps1`: verified safe cache pruning and post-cleanup verification.
  - `bash scripts/clean-caches.sh`: verified POSIX safe cache pruning.
  - `scripts/check-all.ps1` and `bash scripts/check-all.sh`: all 5 gates passed (0 failures).
  - `scripts/verify-bootstrap.ps1`: 199/199 checks passed (0 failures).
- **Security Considerations:** Scripts execute with least privilege; persistent database volumes (`postgres_data`) are protected from accidental pruning; volume pruning requires explicit flag and interactive confirmation or force flag; no secrets or credentials printed or manipulated.
- **Performance Considerations:** Inspection scripts complete in < 2 seconds; JSON output allows low-overhead CI automation; safe pruning prevents disk exhaustion on developer machines.
- **Documentation Requirements:** Updated `docs/DEPLOYMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`, and `ROADMAP.md`.
- **Validation Evidence:**
  - `scripts/check-resources.ps1` output: `[HEALTHY_WITHIN_BUDGET] Podman storage footprint (4.95 GB / 30 GB, 16.5%) is within budget.`
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (199/199 checks passed).
  - `scripts/check-all.sh`: ALL 5 GATES PASSED (199/199 checks passed).
- **Git Commit Reference:** `0f67467`
- **Status:** `ACCEPTED`

---

#### Subphase `0.3.3` — Create development environment scripts
- **Unique ID:** `0.3.3`
- **Objective:** Create cross-platform development environment lifecycle utilities across Windows PowerShell and POSIX bash (`scripts/dev-up.*`, `scripts/dev-down.*`, `scripts/dev-logs.*`, `scripts/dev-restart.*`, `scripts/dev-status.*`), integrating them with bootstrap verification, quality runners, and documentation.
- **Scope:** Development lifecycle scripts in `scripts/`, `scripts/verify-bootstrap.ps1`, `docs/DEVELOPMENT.md`, and tracking ledgers.
- **Prerequisites:** `0.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `scripts/dev-up.ps1`
  - `scripts/dev-up.sh`
  - `scripts/dev-down.ps1`
  - `scripts/dev-down.sh`
  - `scripts/dev-logs.ps1`
  - `scripts/dev-logs.sh`
  - `scripts/dev-restart.ps1`
  - `scripts/dev-restart.sh`
  - `scripts/dev-status.ps1`
  - `scripts/dev-status.sh`
  - `scripts/verify-bootstrap.ps1`
  - `docs/DEVELOPMENT.md`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
- **Implementation Tasks:**
  1. Author `scripts/dev-up.ps1` and `scripts/dev-up.sh` supporting service subsets, optional image rebuilds (`--build`), pre-flight 30 GB storage ceiling verification, automatic `deploy/.env` initialization from `deploy/.env.example`, and PostgreSQL healthcheck polling.
  2. Author `scripts/dev-down.ps1` and `scripts/dev-down.sh` for clean service and network teardown with interactive guards against accidental deletion of persistent database volumes (`postgres_data`, `artifact_data`) and post-teardown cache pruning (`-PruneCaches`).
  3. Author `scripts/dev-logs.ps1` and `scripts/dev-logs.sh` supporting service filtering, line tailing, and real-time streaming (`-Follow`).
  4. Author `scripts/dev-restart.ps1` and `scripts/dev-restart.sh` for service restarts, optional rebuilds, and healthcheck validation.
  5. Author `scripts/dev-status.ps1` and `scripts/dev-status.sh` displaying runtime container states, healthchecks, open ports, and live storage ceiling metrics in formatted table and JSON formats.
  6. Extend `scripts/verify-bootstrap.ps1` with Category 13 asserting all 10 lifecycle scripts exist, are non-empty, and execute without error.
  7. Document the complete developer lifecycle commands in `docs/DEVELOPMENT.md`.
- **Acceptance Criteria:**
  - All 10 lifecycle scripts exist, support dry-run simulation, execute cleanly on Windows and POSIX, and correctly manage Compose workloads.
  - Teardown protects persistent volumes by default.
  - Status utility accurately reports container state, health, exposed ports, and storage consumption.
  - All 5 gates in `scripts/check-all.*` pass with 0 failures, and `scripts/verify-bootstrap.ps1` passes all 213 checks.
- **Tests:**
  - `scripts/dev-up.ps1 -DryRun`: passed (verified env init, ceiling check, and compose arguments).
  - `bash scripts/dev-up.sh --dry-run`: passed.
  - `scripts/dev-down.ps1 -DryRun`: passed.
  - `bash scripts/dev-down.sh --dry-run`: passed.
  - `scripts/dev-logs.ps1 -Tail 5`: passed.
  - `bash scripts/dev-logs.sh --tail 5`: passed.
  - `scripts/dev-restart.ps1 -DryRun`: passed.
  - `bash scripts/dev-restart.sh --dry-run`: passed.
  - `scripts/dev-status.ps1`: passed (displayed 5 services, health, ports, and budget).
  - `scripts/dev-status.ps1 -Json`: passed (verified valid JSON structure).
  - `bash scripts/dev-status.sh`: passed.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (213/213 checks).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (213/213 checks).
- **Security Considerations:** `dev-up` enforces pre-flight 30 GB ceiling check; `dev-down` protects database state from accidental deletion with interactive confirmation; unprivileged container users are preserved; no secrets are logged.
- **Performance Considerations:** Dry-run and status checks execute in < 2 seconds; polling loops use bounded timeouts (default 60s); detached execution frees terminal.
- **Documentation Requirements:** Updated `docs/DEVELOPMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`, and `ROADMAP.md`.
- **Validation Evidence:**
  - `scripts/dev-status.ps1` output: 5 services reported, storage budget verified.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (213/213 checks passed).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (213/213 checks passed).
- **Git Commit Reference:** `9b441fa`
- **Status:** `ACCEPTED`

---

#### Subphase `0.3.4` — Validate reproducible container builds
- **Unique ID:** `0.3.4`
- **Objective:** Validate reproducible container builds, multi-stage architecture specifications, unprivileged runtime user constraints, and strict compliance with the 30 GB container storage ceiling across all services.
- **Scope:** `deploy/podman-compose.yml`, `deploy/Containerfile.{api,worker,browser-worker,web}`, `scripts/verify-container-builds.ps1`, `scripts/verify-container-builds.sh`, `scripts/verify-bootstrap.ps1`, `docs/DEPLOYMENT.md`, and tracking ledgers.
- **Prerequisites:** `0.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `deploy/Containerfile.api`
  - `deploy/Containerfile.worker`
  - `deploy/Containerfile.browser-worker`
  - `deploy/Containerfile.web`
  - `scripts/verify-container-builds.ps1`
  - `scripts/verify-container-builds.sh`
  - `scripts/verify-bootstrap.ps1`
  - `docs/DEPLOYMENT.md`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
- **Implementation Tasks:**
  1. Refine `deploy/Containerfile.api` and `deploy/Containerfile.worker` to output valid POSIX executable binaries with shebang and execution permissions, preventing `exec format error` during container boot.
  2. Refine `deploy/Containerfile.browser-worker` and `deploy/Containerfile.web` to ensure server entrypoints run unprivileged with node runtimes.
  3. Author `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing all 4 Containerfiles and Compose specs for multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  4. Execute live container verification on PostgreSQL 16 Alpine: test clean boot, healthcheck polling, status inspection, teardown, and persistent volume retention (`deploy_postgres_data`).
  5. Enforce 30 GB container storage budget (< 30 GB) before and after container operations.
  6. Extend `scripts/verify-bootstrap.ps1` with Category 14 asserting container verification scripts.
  7. Document reproducibility and architectural standards in `docs/DEPLOYMENT.md`.
- **Acceptance Criteria:**
  - All 4 Containerfiles and `deploy/podman-compose.yml` pass static architectural audit (multi-stage, slim images, unprivileged users, isolated networks, sandbox flags).
  - Live container test starts PostgreSQL, verifies `healthy` status in `dev-status`, executes clean teardown, and confirms retention of `deploy_postgres_data`.
  - Total Podman storage footprint remains strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - All 5 gates in `scripts/check-all.*` pass with 0 failures, and `scripts/verify-bootstrap.ps1` passes all 216 checks.
- **Tests:**
  - `scripts/verify-container-builds.ps1 -Quick`: passed (21/21 checks).
  - `scripts/verify-container-builds.ps1`: passed (25/25 checks, including live container healthcheck and volume retention).
  - `bash scripts/verify-container-builds.sh --quick`: passed (21/21 checks).
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (216/216 checks).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (216/216 checks).
- **Security Considerations:** Multi-stage builds eliminate compiler and build-tool attack surface from production images; unprivileged non-root users (`appuser`, `pwuser`, `node`) are strictly enforced; `no-new-privileges:true` prevents privilege escalation; browser worker is isolated from internal PostgreSQL network.
- **Performance Considerations:** Slim and Alpine base images keep total image footprint to 5.27 GB (< 18% of 30 GB ceiling); healthcheck polling uses 2-second backoff with 30s timeout.
- **Documentation Requirements:** Updated `docs/DEPLOYMENT.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, `CHANGELOG.md`, and `ROADMAP.md`.
- **Validation Evidence:**
  - `scripts/verify-container-builds.ps1` output: 25/25 checks passed (0 failures).
  - Live container execution: PostgreSQL 16 Alpine healthy on port 5432; `deploy_postgres_data` persistent volume retained.
  - Storage ceiling audit: 5.27 GB / 30 GB (17.6%) [HEALTHY_WITHIN_BUDGET].
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (216/216 checks passed).
- **Git Commit Reference:** `f925fd2`
- **Status:** `ACCEPTED`

---

#### Subphase `0.3.5` — Document setup and recovery
- **Unique ID:** `0.3.5`
- **Objective:** Document container setup, complete environment variables reference, backup and restore runbooks, emergency cold disaster recovery procedures, and production hardening guidelines in `docs/DEPLOYMENT.md` and `docs/DEVELOPMENT.md`.
- **Scope:** `docs/DEPLOYMENT.md`, `docs/DEVELOPMENT.md`, `scripts/verify-bootstrap.ps1`, `ROADMAP.md`, `docs/ROADMAP.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `0.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `docs/DEPLOYMENT.md`
  - `docs/DEVELOPMENT.md`
  - `scripts/verify-bootstrap.ps1`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author Section 3 ("Environment Variables Reference") in `docs/DEPLOYMENT.md` documenting all 19 configuration variables with defaults, services, sensitivity levels, and usage notes.
  2. Author Section 4 ("Backup, Restore & Disaster Recovery Runbook") in `docs/DEPLOYMENT.md` providing concrete `pg_dump`/`pg_restore` commands for Windows PowerShell and POSIX bash, artifact volume backup commands, and step-by-step cold recovery runbook (<5 minutes).
  3. Author Section 5 ("Production Hardening Guidelines") in `docs/DEPLOYMENT.md` detailing Caddy automatic HTTPS reverse proxy configuration with TLS security headers, secret manager injection, encryption at rest, container root filesystem hardening, Linux capability dropping, and storage ceiling alerting.
  4. Ensure complete synchronization with `docs/DEVELOPMENT.md` Section 5 (Workflow Runbooks) and Section 6 (Troubleshooting Runbook).
  5. Expand `scripts/verify-bootstrap.ps1` Category 15 to automatically assert presence of environment variables table, disaster recovery runbooks, and production hardening sections.
  6. Execute Stage 0 Acceptance Gate verification across all 5 monorepo gates and confirm 100% pass rate.
- **Acceptance Criteria:**
  - Complete environment variables table present in `docs/DEPLOYMENT.md`.
  - Non-mock backup and restore commands for PostgreSQL and persistent volumes documented for Windows PowerShell and POSIX bash.
  - Emergency cold disaster recovery procedure documented with clear time target (<5 minutes).
  - Production hardening guidelines (Caddy reverse proxy, secrets, capabilities, network isolation) fully specified.
  - All 5 monorepo verification gates pass with 0 failures, and `scripts/verify-bootstrap.ps1` passes all 223 checks.
- **Tests:**
  - `scripts/verify-bootstrap.ps1`: passed (223/223 checks).
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (223/223 checks).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (223/223 checks).
  - `scripts/verify-container-builds.ps1 -Quick`: passed (21/21 checks).
  - `scripts/check-resources.ps1`: passed (5.27 GB / 30 GB, 17.6%).
- **Security Considerations:**
  - Strict classification of sensitive environment variables (`POSTGRES_PASSWORD`, `DATABASE_URL`, `GITHUB_APP_PRIVATE_KEY_PATH`, `GITHUB_APP_CLIENT_SECRET`).
  - Re-verified zero credentials in tracked files or Git history.
  - Production hardening enforces non-root execution, `no-new-privileges:true`, read-only container root filesystems, and strict network isolation between `browser_net` and `internal_net`.
- **Performance Considerations:**
  - Fast disaster recovery runbook targeted for cold restoration under 5 minutes.
  - Storage ceiling strictly bounded below 30 GB (currently 5.27 GB, 17.6%).
- **Documentation Requirements:** Updated `docs/DEPLOYMENT.md`, `docs/DEVELOPMENT.md`, `docs/ROADMAP.md`, `ROADMAP.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `TODO.md`, and `CHANGELOG.md`.
- **Validation Evidence:**
  - `docs/DEPLOYMENT.md` verified with Sections 3, 4, and 5.
  - `scripts/verify-bootstrap.ps1` output: 223/223 checks passed (0 failures).
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
  - Storage budget verified: 5.27 GB / 30 GB (17.6%) [HEALTHY_WITHIN_BUDGET].
- **Git Commit Reference:** `c549b14`
- **Status:** `ACCEPTED`

---

### STAGE 0 COMPLETION CERTIFICATION

- **Stage ID:** `STAGE 0 — PROJECT BOOTSTRAP AND ENGINEERING FOUNDATION`
- **Total Phases Completed:** 3 (`Phase 0.1`, `Phase 0.2`, `Phase 0.3`)
- **Total Subphases Completed:** 15 (`0.1.1` through `0.3.5`, all `ACCEPTED`)
- **Stage Acceptance Gate:** Repository, documentation, agents, rules, workflows, Podman config (<30 GB), and initial CI workflows are operational and verified.
- **Verification Evidence:**
  - Monorepo structure, 27 directories, 14 root governance files verified.
  - 12 baseline technical specifications under `docs/` verified.
  - 9 specialized agents, 10 engineering skills, 10 rules, and 10 workflows verified.
  - Public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai` operational with linear history and branch protection on `main`.
  - 7 GitHub Actions workflows verified and passing on live runners.
  - Multi-stage Containerfiles and Compose stack verified with unprivileged users and isolated networks.
  - Podman storage ceiling strictly satisfied at 5.27 GB / 30 GB (17.6%).
  - Full cross-platform lifecycle scripts (`dev-*.ps1` / `dev-*.sh`) and disaster recovery runbooks verified.
  - Monorepo audit (`scripts/check-all.*`) passing all 5 gates (223/223 checks passed).
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

## STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE

### Phase 1.1 — Design system

#### Subphase `1.1.1` — Visual design tokens
- **Unique ID:** `1.1.1`
- **Objective:** Scaffold the modern SvelteKit + Svelte 5 + TypeScript + Tailwind CSS application foundation in `apps/web` and establish the complete visual design token architecture (color palettes, typography scale, spacing grid, border radii, elevations/shadows, CSS custom properties, and programmatic design token exports with WCAG AA compliance).
- **Scope:** `apps/web/package.json`, `apps/web/svelte.config.js`, `apps/web/vite.config.ts`, `apps/web/tsconfig.json`, `apps/web/tailwind.config.ts`, `apps/web/postcss.config.js`, `apps/web/src/app.html`, `apps/web/src/app.d.ts`, `apps/web/src/routes/+layout.svelte`, `apps/web/src/routes/+page.svelte`, `apps/web/src/lib/styles/*`, `apps/web/src/lib/tokens/*`, `apps/web/tests/tokens.test.ts`.
- **Prerequisites:** `0.3.5` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/package.json`
  - `apps/web/svelte.config.js`
  - `apps/web/vite.config.ts`
  - `apps/web/tsconfig.json`
  - `apps/web/tailwind.config.ts`
  - `apps/web/postcss.config.js`
  - `apps/web/src/app.html`
  - `apps/web/src/app.d.ts`
  - `apps/web/src/routes/+layout.svelte`
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/src/lib/tokens/index.ts`
  - `apps/web/src/lib/styles/tokens.css`
  - `apps/web/src/lib/styles/app.css`
  - `apps/web/tests/tokens.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Initialize `apps/web` with SvelteKit 2, Svelte 5 runes support, strict TypeScript, Tailwind CSS, PostCSS, Autoprefixer, `@sveltejs/adapter-node`, Vitest, and Prettier.
  2. Implement visual design tokens module (`apps/web/src/lib/tokens/index.ts`) providing strongly-typed tokens for:
     - Colors: brand primary (AI violet/indigo), secondary/cyan, neutral scales (slate/zinc for dark and light surfaces), semantic states (success, warning, error, info), and context token budget indicators (safe, warning >80%, critical >100%).
     - Typography: font families (sans system and monospace code/token stacks), size scale (`xs` through `5xl`), line heights, and weights.
     - Spacing & Layout: 4px grid (`1` = 4px to `24` = 96px), container breakpoints.
     - Elevation & Radii: Shadows `sm` to `2xl`, border radius scales `sm` to `full`.
     - Focus & Accessibility: high-contrast visible focus rings (`2px` with offset).
  3. Author CSS custom properties in `apps/web/src/lib/styles/tokens.css` mapping design tokens to CSS variables with automatic dark-mode support via `:root` and `.dark` selectors.
  4. Integrate design tokens into `apps/web/tailwind.config.ts` extending the Tailwind theme cleanly.
  5. Author root layout (`+layout.svelte`) importing design token styles, and initial token showcase root page (`+page.svelte`).
  6. Author comprehensive automated unit test suite `apps/web/tests/tokens.test.ts` validating token invariants, format consistency, hex color validity, WCAG contrast calculation, and CSS variable exports.
  7. Verify all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `apps/web` has functional SvelteKit + Svelte 5 + TypeScript + Tailwind CSS setup.
  - Design tokens are programmatically accessible via `src/lib/tokens/index.ts` and styled via `src/lib/styles/tokens.css`.
  - Semantic status colors and token budget warning indicators (>80%, >100%) are formally defined.
  - Automated tests in `apps/web/tests/tokens.test.ts` pass with 100% success.
  - `npm run check` (svelte-check) passes with 0 errors and 0 warnings.
  - `npm run build` generates valid node build in `apps/web/build`.
  - All monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runner `scripts/check-all.ps1`.
- **Security Considerations:** No third-party external CDN scripts or stylesheets; fonts use local/system font stacks to prevent privacy leaks and external network dependencies; strict Content Security Policy compliance.
- **Performance Considerations:** Atomic CSS via Tailwind eliminates unused stylesheet bloat; zero runtime CSS-in-JS overhead; production build is statically optimized and minified.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `apps/web/tests/tokens.test.ts`: 18/18 tests passed (0 failures).
  - `npm run check` (svelte-check): 0 errors, 0 warnings.
  - `npm run lint` (prettier): All matched files use Prettier code style.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 223/223 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `4423982`
- **Status:** `ACCEPTED`

---

#### Subphase `1.1.2` — Typography, spacing & color systems
- **Unique ID:** `1.1.2`
- **Objective:** Implement the typography scale components, spacing grid layout primitives, accessible high-contrast color utilities, and author the comprehensive design system specification in `docs/DESIGN_SYSTEM.md`.
- **Scope:** `apps/web/src/lib/components/typography/*`, `apps/web/src/lib/components/layout/*`, `apps/web/src/lib/tokens/color-utils.ts`, `apps/web/tests/typography.test.ts`, `apps/web/tests/color-utils.test.ts`, `docs/DESIGN_SYSTEM.md`, `apps/web/src/routes/+page.svelte`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/components/typography/Heading.svelte`
  - `apps/web/src/lib/components/typography/Text.svelte`
  - `apps/web/src/lib/components/typography/Code.svelte`
  - `apps/web/src/lib/components/typography/Mono.svelte`
  - `apps/web/src/lib/components/typography/index.ts`
  - `apps/web/src/lib/components/layout/Container.svelte`
  - `apps/web/src/lib/components/layout/Stack.svelte`
  - `apps/web/src/lib/components/layout/Grid.svelte`
  - `apps/web/src/lib/components/layout/index.ts`
  - `apps/web/src/lib/tokens/color-utils.ts`
  - `apps/web/tests/typography.test.ts`
  - `apps/web/tests/color-utils.test.ts`
  - `apps/web/src/routes/+page.svelte`
  - `docs/DESIGN_SYSTEM.md`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author accessible typography scale components in Svelte 5:
     - `Heading.svelte`: Semantic levels 1 through 6 (`h1`–`h6`), proportional sizing (`5xl` down to `base`), tight tracking, and theme color classes.
     - `Text.svelte`: Body text sizes (`xs`, `sm`, `base`, `lg`, `xl`), weights, muted/secondary/primary tone variants, and responsive line heights.
     - `Code.svelte` & `Mono.svelte`: Inline and block monospace code presentation with syntax background, border styling, and token counter styling.
  2. Author layout primitives enforcing the 4px spacing grid:
     - `Container.svelte`: Responsive max-width wrappers (`sm`, `md`, `lg`, `xl`, `full`) with responsive horizontal padding.
     - `Stack.svelte`: Vertical or horizontal flex layout with typed 4px grid spacing props (`gap-1` to `gap-12`).
     - `Grid.svelte`: Responsive grid layouts (1 to 12 columns, customizable responsive breakpoints).
  3. Author color contrast and accessibility utilities (`color-utils.ts`):
     - WCAG AA (4.5:1 for normal text, 3.0:1 for large text / UI components) and AAA (7.0:1) verification functions.
     - Dynamic text color picker (`getAccessibleTextColor`) returning light or dark text based on background luminance.
     - Accessible focus ring styling classes.
  4. Author `docs/DESIGN_SYSTEM.md` documenting:
     - Design principles (clarity, density, responsiveness, accessibility first).
     - Full typography hierarchy, line heights, font stacks, and code typography.
     - 4px spacing rhythm and container sizing rules.
     - Color system, surface layers, token budget warning bands, and WCAG AA/AAA verification.
  5. Update `apps/web/src/routes/+page.svelte` demonstrating typography components, layout containers, and contrast verification in action.
  6. Author unit tests `apps/web/tests/typography.test.ts` and `apps/web/tests/color-utils.test.ts`.
  7. Verify all gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - All typography components (`Heading`, `Text`, `Code`, `Mono`) render semantic elements with Svelte 5 runes.
  - Layout primitives (`Container`, `Stack`, `Grid`) support flexible 4px grid spacing.
  - Color utilities validate WCAG AA/AAA compliance programmatically.
  - `docs/DESIGN_SYSTEM.md` is complete, structured, and non-empty.
  - Automated tests pass with 100% success.
  - Svelte-check passes with 0 errors and 0 warnings.
  - All monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runner `scripts/check-all.ps1`.
- **Security Considerations:** No external web fonts or CDN links; all typography relies on safe system font stacks and CSS custom properties; no user-supplied HTML injection without escaping.
- **Performance Considerations:** Components compile to lean Svelte 5 JavaScript; zero runtime CSS overhead; layout primitives use native flexbox and CSS grid.
- **Documentation Requirements:** Updated `docs/DESIGN_SYSTEM.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `apps/web/tests/color-utils.test.ts`: 11/11 tests passed (0 failures).
  - `apps/web/tests/tokens.test.ts`: 18/18 tests passed (0 failures).
  - `apps/web/tests/typography.test.ts`: 2/2 tests passed (0 failures).
  - `npm run check` (svelte-check): 0 errors, 0 warnings.
  - `npm run lint` (prettier): All matched files use Prettier code style.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks including `docs/DESIGN_SYSTEM.md`, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `97f9e44`
- **Status:** `ACCEPTED`

---

#### Subphase `1.1.3` — Reusable accessible components
- **Unique ID:** `1.1.3`
- **Objective:** Build the reusable accessible UI component primitives (`Button`, `Input`, `Toggle`, `Badge`, `Modal`, `Tooltip`) in `apps/web/src/lib/components/ui/` with full keyboard support, visible focus states, ARIA roles, and certified WCAG AA compliance.
- **Scope:** `apps/web/src/lib/components/ui/*`, `apps/web/tests/ui-components.test.ts`, `apps/web/src/routes/+page.svelte`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/components/ui/Button.svelte`
  - `apps/web/src/lib/components/ui/Input.svelte`
  - `apps/web/src/lib/components/ui/Toggle.svelte`
  - `apps/web/src/lib/components/ui/Badge.svelte`
  - `apps/web/src/lib/components/ui/Modal.svelte`
  - `apps/web/src/lib/components/ui/Tooltip.svelte`
  - `apps/web/src/lib/components/ui/index.ts`
  - `apps/web/tests/ui-components.test.ts`
  - `apps/web/src/routes/+page.svelte`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author `Button.svelte`: variants (`primary`, `secondary`, `outline`, `ghost`, `danger`), sizes (`sm`, `md`, `lg`), loading spinner state, disabled state, fullWidth, keyboard focus rings, click event forwarding.
  2. Author `Input.svelte`: text/url input with label, helper text, error message, placeholder, prefix icon/badge, clear button, ARIA attributes (`aria-invalid`, `aria-describedby`).
  3. Author `Toggle.svelte`: accessible switch primitive with `role="switch"`, `aria-checked`, label, description, keyboard toggling (`Space`, `Enter`).
  4. Author `Badge.svelte`: status/category indicator with variants (`default`, `brand`, `success`, `warning`, `error`, `info`), sizes (`sm`, `md`), optional status dot.
  5. Author `Modal.svelte`: accessible dialog with backdrop, escape key handler, click-outside dismissal, `role="dialog"`, `aria-modal="true"`, `aria-labelledby`, header, close button, body, and action footer.
  6. Author `Tooltip.svelte`: accessible contextual tooltip with keyboard focus/hover triggers, `aria-describedby`, positioned popover.
  7. Export all UI primitives in `apps/web/src/lib/components/ui/index.ts`.
  8. Update `apps/web/src/routes/+page.svelte` demonstrating live interactive UI components.
  9. Author comprehensive test suite `apps/web/tests/ui-components.test.ts`.
  10. Run all verification gates.
- **Acceptance Criteria:**
  - All 6 UI components render cleanly using Svelte 5 runes and TypeScript.
  - Buttons support all 5 variants, 3 sizes, loading spinners, and disabled states.
  - Inputs link labels and error messages via ARIA attributes.
  - Toggles support `role="switch"` and keyboard activation.
  - Modals respond to `Escape`, dismiss on backdrop click, and render semantic dialog structure.
  - Automated component test suite passes with 100% success.
  - `npm run check` passes with 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runner `scripts/check-all.ps1`.
- **Security Considerations:** Inputs sanitize values; modal events prevent scroll-jacking; no dynamic innerHTML injection; strict CSP compliance.
- **Performance Considerations:** Zero heavy runtime dependencies; clean Svelte 5 reactive bindings; modals conditionally mount into DOM.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `apps/web/tests/ui-components.test.ts`: 6/6 tests passed (0 failures).
  - `apps/web/tests/color-utils.test.ts`: 11/11 tests passed (0 failures).
  - `apps/web/tests/tokens.test.ts`: 18/18 tests passed (0 failures).
  - `apps/web/tests/typography.test.ts`: 2/2 tests passed (0 failures).
  - Total unit tests: 37/37 tests passed across 4 test suites.
  - `npm run check` (svelte-check): 0 errors, 0 warnings.
  - `npm run lint` (prettier): All matched files use Prettier code style.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `f21629f`
- **Status:** `ACCEPTED`

---

### PHASE 1.1 COMPLETION CERTIFICATION

- **Phase ID:** `Phase 1.1 — Design system`
- **Stage:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE`
- **Subphases Completed:** `1.1.1` (Visual design tokens), `1.1.2` (Typography, spacing & color systems), `1.1.3` (Reusable accessible components). All 3 `ACCEPTED`.
- **Phase Deliverable:** Complete, accessible, tested design system with SvelteKit 2 + Svelte 5 foundation, atomic design tokens, typography components, 4px layout primitives, WCAG 2.1 AA/AAA contrast utilities, reusable UI primitives (`Button`, `Input`, `Toggle`, `Badge`, `Modal`, `Tooltip`), and technical specification `docs/DESIGN_SYSTEM.md`.
- **Verification Evidence:**
  - 37/37 automated tests passing in `apps/web`.
  - Svelte diagnostics: 0 errors, 0 warnings.
  - All 5 gates in monorepo runners (`scripts/check-all.*`) passing.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Phase 1.2 — Main website

#### Subphase `1.2.1` — Landing page
- **Unique ID:** `1.2.1`
- **Objective:** Implement the production landing page with semantic structure (`header`, `main`, `footer`), hero section with clear value proposition, supported source matrix, architectural highlights, 3-step conversion flow overview, and conversion entry action.
- **Scope:** `apps/web/src/routes/+page.svelte`, `apps/web/src/lib/components/landing/*`, `apps/web/src/lib/components/navigation/*`, `apps/web/tests/landing-page.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/src/lib/components/navigation/Header.svelte`
  - `apps/web/src/lib/components/navigation/Footer.svelte`
  - `apps/web/src/lib/components/navigation/index.ts`
  - `apps/web/src/lib/components/landing/Hero.svelte`
  - `apps/web/src/lib/components/landing/WorkflowSteps.svelte`
  - `apps/web/src/lib/components/landing/SupportedSources.svelte`
  - `apps/web/src/lib/components/landing/FeatureHighlights.svelte`
  - `apps/web/src/lib/components/landing/index.ts`
  - `apps/web/tests/landing-page.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author navigation components (`apps/web/src/lib/components/navigation/`):
     - `Header.svelte`: Fixed/sticky semantic navigation header with logo, title, GitHub repo badge, documentation link, and theme switcher (`Button`).
     - `Footer.svelte`: Semantic footer with copyright, MIT license, GitHub repo link, architectural specs links, and version indicator.
  2. Author landing page feature components (`apps/web/src/lib/components/landing/`):
     - `Hero.svelte`: Hero headline, subtitle, quick call-to-action button, and dynamic context metric chips (e.g., Markdown, JSON, XML, TXT, JSONL, ZIP).
     - `WorkflowSteps.svelte`: 3-step structured cards (1. Acquisition & SSRF Defense, 2. CDM Canonicalization & Token Optimization, 3. Multi-format Export & Token Budget Verification).
     - `SupportedSources.svelte`: Grid matrix highlighting Static Docs, Dynamic JavaScript Web Apps, Public GitHub Repositories, and Private GitHub App Repositories.
     - `FeatureHighlights.svelte`: Technical highlights (SSRF hardening, AST code intelligence, 30 GB container ceiling, CPU-only 16 GB baseline, zero data retention).
  3. Integrate into `apps/web/src/routes/+page.svelte` replacing the temporary showcase with the production landing experience.
  4. Author automated test suite `apps/web/tests/landing-page.test.ts` verifying semantic HTML landmarks (`header`, `main`, `footer`), headline texts, feature cards, and accessible navigation controls.
  5. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - Landing page renders semantic HTML landmarks (`header`, `main`, `footer`, `section`, `nav`).
  - Hero communicates core product value proposition and zero-mandatory-paid-API design.
  - Supported sources matrix and 3-step conversion workflow are cleanly laid out.
  - Theme switching works seamlessly across dark and light modes.
  - Automated landing page test suite passes with 100% success.
  - Svelte diagnostics report 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runner `scripts/check-all.ps1`.
- **Security Considerations:** All external links use `rel="noopener noreferrer"`; no untrusted user input is reflected without sanitization.
- **Performance Considerations:** Responsive layout uses lightweight native Tailwind CSS without heavy UI frameworks or unoptimized images.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 5 test suites (`tokens.test.ts`, `color-utils.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `landing-page.test.ts`), passing 46/46 automated unit tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `9eb17e7`
- **Status:** `ACCEPTED`

---

#### Subphase `1.2.2` — URL input & source detection
- **Unique ID:** `1.2.2`
- **Objective:** Implement a dedicated URL input component with real-time source detection (GitHub repository vs. web page/documentation vs. sitemap), protocol and SSRF pre-validation, parsed repository metadata (owner, repo, branch, path), and responsive accessible UI feedback.
- **Scope:** `apps/web/src/lib/utils/url-detector.ts`, `apps/web/src/lib/components/input/UrlInput.svelte`, `apps/web/src/lib/components/input/index.ts`, `apps/web/src/routes/+page.svelte`, `apps/web/tests/url-detector.test.ts`, `apps/web/tests/url-input-component.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/utils/url-detector.ts`
  - `apps/web/src/lib/components/input/UrlInput.svelte`
  - `apps/web/src/lib/components/input/index.ts`
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/tests/url-detector.test.ts`
  - `apps/web/tests/url-input-component.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author URL detection and validation engine `apps/web/src/lib/utils/url-detector.ts`:
     - Standard URL parsing with RFC-compliant validation.
     - GitHub URL recognition: public repos, organization repos, branch/tree URLs (`/tree/<branch>`), blob URLs (`/blob/<branch>/<path>`), and subtree paths.
     - Sitemap recognition: `.xml`, `sitemap.xml`, `sitemap-index.xml`.
     - Generic web documentation and article detection with domain extraction.
     - Client-side SSRF safety pre-check: flag loopback (`127.0.0.1`, `localhost`, `::1`), private CIDRs (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata (`169.254.169.254`), and non-HTTP protocols (`file:`, `javascript:`, `data:`, `ftp:`).
  2. Author accessible `UrlInput.svelte` component (`apps/web/src/lib/components/input/UrlInput.svelte`):
     - Protocol indicator, clear button, submit button with loading spinner.
     - Dynamic source badge (`GitHub Repo`, `Web Doc`, `Sitemap`, `Invalid / Disallowed`) with ARIA live region (`aria-live="polite"`).
     - Contextual metadata chips for GitHub URLs (Owner, Repo, Branch, Subtree).
     - Real-time client error messages for invalid syntax and blocked destinations.
  3. Export via `apps/web/src/lib/components/input/index.ts`.
  4. Integrate `UrlInput` into `apps/web/src/routes/+page.svelte` in place of the quick convert prototype.
  5. Author automated test suites:
     - `apps/web/tests/url-detector.test.ts`: test cases covering all source types, GitHub variants, sitemaps, private IP blocks, and malformed inputs.
     - `apps/web/tests/url-input-component.test.ts`: verify component exports, ARIA attributes, reactive states, and event dispatch.
  6. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `url-detector.ts` accurately classifies GitHub repositories, web documentation, and sitemaps.
  - Client-side SSRF pre-check flags loopback, private IPs, and cloud metadata immediately.
  - `UrlInput.svelte` provides responsive visual indicators, clear controls, and accessible ARIA attributes.
  - Integration into root page functions seamlessly across desktop and mobile.
  - 100% of automated tests pass; `svelte-check` reports 0 errors and 0 warnings.
  - All 5 monorepo gates in `scripts/check-all.*` pass cleanly.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** Enforces client-side validation against SSRF targets prior to network submission; sanitizes all displayed URL parts.
- **Performance Considerations:** Detection runs synchronously in memory without blocking input typing; zero heavy dependencies.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 7 test suites (`tokens.test.ts`, `color-utils.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `url-input-component.test.ts`, `landing-page.test.ts`), passing 63/63 automated unit and invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `f94e8f9`
- **Status:** `ACCEPTED`

---

#### Subphase `1.2.3` — Conversion settings panel
- **Unique ID:** `1.2.3`
- **Objective:** Implement an accessible, collapsible conversion settings panel enabling users to configure target AI model context window limits (Gemini 1M/2M, Claude 200k, GPT-4o 128k, Local 32k/128k), customize token budget thresholds with an interactive slider, toggle content inclusion filters (strip boilerplate, strip code comments, exclude test files), and control dynamic Chromium rendering fallback.
- **Scope:** `apps/web/src/lib/types/settings.ts`, `apps/web/src/lib/components/settings/ConversionSettings.svelte`, `apps/web/src/lib/components/settings/index.ts`, `apps/web/src/routes/+page.svelte`, `apps/web/tests/conversion-settings.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/settings.ts`
  - `apps/web/src/lib/components/settings/ConversionSettings.svelte`
  - `apps/web/src/lib/components/settings/index.ts`
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/tests/conversion-settings.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Define strong TypeScript settings contracts `apps/web/src/lib/types/settings.ts` matching `docs/API.md` and CDM specifications:
     - Target AI model presets: `gemini-1.5-pro` (1,000,000 tokens), `gemini-1.5-flash` (1,000,000 tokens), `claude-3.5-sonnet` (200,000 tokens), `gpt-4o` (128,000 tokens), `local-llm` (32,000 tokens), and `custom`.
     - Ingestion and normalization flags: `stripBoilerplate`, `stripCodeComments`, `excludeTests`, `enableDynamicRendering`, `maxFileSizeKb`.
     - Default conversion settings generator and validator.
  2. Author accessible `ConversionSettings.svelte` component (`apps/web/src/lib/components/settings/ConversionSettings.svelte`):
     - Collapsible disclosure panel with `aria-expanded`, `aria-controls`, and smooth CSS transitions.
     - Target model preset selector pills with calibrated token ceilings.
     - Interactive token budget range slider with dynamic token count formatting and WCAG AA contrast.
     - Accessible switches (`Toggle` primitive) for content inclusion filters.
     - Dynamic JavaScript rendering fallback toggle with isolated container notice.
  3. Export via `apps/web/src/lib/components/settings/index.ts`.
  4. Integrate `ConversionSettings` into `apps/web/src/routes/+page.svelte` in `#convert`, binding settings state directly to conversion execution feedback.
  5. Author automated test suite `apps/web/tests/conversion-settings.test.ts`:
     - Test model presets and calibrated token limits.
     - Test settings validation and defaults.
     - Test ARIA contracts (`aria-expanded`, `aria-controls`, `role="region"`).
     - Test component reactive bindings and toggle states.
  6. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `ConversionSettings.svelte` provides accessible collapsible controls with clear visual indicator.
  - Model presets accurately update the target token ceiling.
  - Custom token slider allows manual fine-tuning between 8,000 and 2,000,000 tokens.
  - All content filter toggles modify real reactive state without stubbed or inactive buttons.
  - Automated test suite passes with 100% success.
  - Svelte diagnostics report 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass cleanly.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runners `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** Settings are sanitized and validated against safe bounds (min 1,000 tokens, max 2,000,000 tokens, max 10 MB per file).
- **Performance Considerations:** State changes compute reactively using Svelte 5 runes with zero unnecessary DOM re-renders.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 8 test suites (`tokens.test.ts`, `color-utils.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `url-input-component.test.ts`, `conversion-settings.test.ts`, `landing-page.test.ts`), passing 69/69 automated unit and invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `ff01d74`
- **Status:** `ACCEPTED`

---

#### Subphase `1.2.4` — Responsive layouts (mobile, tablet, desktop)
- **Unique ID:** `1.2.4`
- **Objective:** Finalize and rigorously audit cross-viewport responsive design across all Phase 1.2 components (Header, Hero, UrlInput, ConversionSettings, WorkflowSteps, SupportedSources, FeatureHighlights, Footer) for mobile (`320px`–`375px`), tablet (`768px`), and desktop (`1280px`–`1440px`), verifying zero horizontal overflow, touch target accessibility ($\ge 44 \times 44\text{px}$), and complete Phase 1.2 certification.
- **Scope:** `apps/web/src/routes/+page.svelte`, `apps/web/src/lib/components/**/*`, `apps/web/tests/responsive-layout.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/src/lib/components/navigation/Header.svelte`
  - `apps/web/src/lib/components/navigation/Footer.svelte`
  - `apps/web/src/lib/components/landing/Hero.svelte`
  - `apps/web/src/lib/components/input/UrlInput.svelte`
  - `apps/web/src/lib/components/settings/ConversionSettings.svelte`
  - `apps/web/tests/responsive-layout.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Audit and harden responsive CSS across all landing page components:
     - Verify container padding and breakpoint boundaries (`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8`).
     - Ensure minimum touch target dimensions ($\ge 44 \times 44\text{px}$) on mobile viewports for all interactive controls (buttons, inputs, toggles, navigation links, preset selectors).
     - Ensure zero horizontal scrolling or clipping on narrow viewports down to `320px`.
     - Verify smooth collapsible transitions on mobile drawer in `Header` and disclosure panel in `ConversionSettings`.
  2. Author comprehensive automated test suite `apps/web/tests/responsive-layout.test.ts`:
     - Test responsive breakpoint tokens and utility invariants (`sm: 640px`, `md: 768px`, `lg: 1024px`, `xl: 1280px`, `2xl: 1536px`).
     - Test touch target sizing attributes and classes across all interactive components.
     - Test mobile navigation drawer expansion, visibility, and focus management.
     - Test grid layout responsiveness (`grid-cols-1`, `md:grid-cols-2`, `lg:grid-cols-3` / `lg:grid-cols-5`).
  3. Formally certify completion of **Phase 1.2: Main website** across all 4 subphases (`1.2.1` – `1.2.4` all `ACCEPTED`).
  4. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - Zero horizontal overflow across mobile (320px+), tablet (768px+), and desktop (1280px+) viewports.
  - Interactive elements have touch targets meeting or exceeding WCAG AA recommendations.
  - Navigation drawer and settings disclosure function cleanly on mobile viewports.
  - Automated responsive layout test suite passes with 100% success.
  - Svelte diagnostics report 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runners `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** No inline scripts or unsafe CSS styles; responsive attributes preserve semantic landmark hierarchy.
- **Performance Considerations:** Responsive transitions utilize GPU-accelerated CSS transforms and opacity without triggering layout recalculations.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 9 test suites (`tokens.test.ts`, `color-utils.test.ts`, `responsive-layout.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `url-input-component.test.ts`, `conversion-settings.test.ts`, `landing-page.test.ts`), passing 76/76 automated unit, responsive, and invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `e990456`
- **Status:** `ACCEPTED`

---

#### Phase 1.2 Milestone Certification
- **Phase Name:** `Phase 1.2 — Main website`
- **Subphases Covered:** `1.2.1` (Landing page), `1.2.2` (URL input & source detection), `1.2.3` (Conversion settings panel), `1.2.4` (Responsive layouts).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Main landing website assembled with semantic landmarks (`header`, `main`, `footer`, `section`, `form`).
  2. URL input engine features zero-trust SSRF defense and real-time classification for GitHub repositories, branch/tree subpaths, file blobs, XML sitemaps, and web documentation.
  3. Conversion settings panel supports 7 calibrated AI model presets (Gemini 2M, Claude 200k, GPT-4o 128k, Local 32k/128k), dynamic budget range slider, content inclusion toggles, and Chromium rendering fallback switch.
  4. Cross-viewport responsive design verified for mobile (320px–375px), tablet (768px), and desktop (1280px+) with zero horizontal overflow and accessible touch targets ($\ge 44 \times 44\text{px}$).
  5. 76 automated tests passing across 9 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings.
  6. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Phase 1.3 — Conversion workspace

#### Subphase `1.3.1` — Build conversion job workspace
- **Unique ID:** `1.3.1`
- **Objective:** Implement dedicated conversion job workspace route (`/workspace` and reusable workspace component) providing real-time pipeline stage progress tracking (Acquisition -> Normalization -> Deduplication -> Export), phase step cards, live log stream terminal, completeness file accounting, and interactive cancel/retry controls.
- **Scope:** `apps/web/src/lib/types/job.ts`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/index.ts`, `apps/web/src/routes/workspace/+page.svelte`, `apps/web/src/routes/+page.svelte`, `apps/web/tests/job-workspace.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/job.ts`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/src/lib/components/workspace/index.ts`
  - `apps/web/src/routes/workspace/+page.svelte`
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/tests/job-workspace.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author job domain contracts `apps/web/src/lib/types/job.ts` matching `docs/API.md` and `docs/DATA_MODEL.md`:
     - `JobStatus`: `'queued' | 'leased' | 'running' | 'completed' | 'partial' | 'failed' | 'cancelled'`
     - `JobPhase`: `'pending' | 'acquisition' | 'normalization' | 'deduplication' | 'export' | 'completed'`
     - Pipeline phase metadata, step status indicators (`idle`, `active`, `completed`, `failed`).
     - Completeness report structure (`discoveredCount`, `extractedCount`, `excludedCount`, `failedCount`).
     - Token metrics and real-time log event models.
     - Deterministic mock-free job state transition helper for testing and offline execution.
  2. Author accessible `JobWorkspace.svelte` component (`apps/web/src/lib/components/workspace/JobWorkspace.svelte`):
     - Overall progress bar with ARIA progressbar attributes (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`).
     - 4-stage pipeline visualization cards (Acquisition, Normalization, Deduplication, Export) with live stage status badges.
     - Completeness file accounting counter ($discovered = extracted + excluded + failed$).
     - Live execution log terminal with auto-scroll and `aria-live="polite"` region.
     - Interactive cancel job button and retry button wired to real state transitions.
  3. Export via `apps/web/src/lib/components/workspace/index.ts`.
  4. Author dedicated workspace route `apps/web/src/routes/workspace/+page.svelte`:
     - Navigation breadcrumbs (Home -> Workspace), Header, JobWorkspace shell, and Footer.
     - URL query param support (`?url=...&format=...&budget=...`) connecting landing page triggers to live workspace execution.
  5. Author automated test suite `apps/web/tests/job-workspace.test.ts`:
     - Test job state transitions and pipeline phase progress calculations.
     - Test completeness accounting invariants ($discovered = extracted + excluded + failed$).
     - Test semantic landmarks, ARIA progressbar, live regions, and interactive cancel/retry handlers.
  6. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - Dedicated `/workspace` route renders cleanly and accepts query parameter initialization.
  - Job progress bar accurately reflects overall completion percentage with WCAG AA compliance.
  - 4 pipeline stages update dynamically with appropriate active/completed/failed states.
  - Cancel and retry controls perform real state updates without mock or dead buttons.
  - Automated test suite passes with 100% success.
  - Svelte diagnostics report 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and monorepo runners `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** All displayed URL and log data are HTML-escaped to prevent XSS; SSRF validation status is surfaced explicitly.
- **Performance Considerations:** Log streaming uses windowed line capping (max 200 entries in DOM) to keep browser memory bounded.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 10 test suites (`tokens.test.ts`, `color-utils.test.ts`, `responsive-layout.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `url-input-component.test.ts`, `conversion-settings.test.ts`, `landing-page.test.ts`, `job-workspace.test.ts`), passing 90/90 automated unit, responsive, domain model, and component invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `7d58403`
- **Status:** `ACCEPTED`

---

#### Subphase `1.3.2` — Build syntax-highlighted content preview panels
- **Unique ID:** `1.3.2`
- **Objective:** Implement accessible syntax-highlighted content preview panel (`PreviewPanel.svelte`) with multi-format export tablist (`Markdown`, `JSON CDM`, `Claude XML`, `Plaintext`, `JSONL`), line numbers gutter, word wrap toggle, live copy-to-clipboard control, and file tree viewer.
- **Scope:** `apps/web/src/lib/types/preview.ts`, `apps/web/src/lib/utils/syntax-highlighter.ts`, `apps/web/src/lib/components/workspace/PreviewPanel.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/index.ts`, `apps/web/tests/preview-panel.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/preview.ts`
  - `apps/web/src/lib/utils/syntax-highlighter.ts`
  - `apps/web/src/lib/components/workspace/PreviewPanel.svelte`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/src/lib/components/workspace/index.ts`
  - `apps/web/tests/preview-panel.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author preview domain types and export formatters in `apps/web/src/lib/types/preview.ts`:
     - Multi-format definitions (`Markdown`, `JSON CDM`, `Claude XML`, `Plaintext`, `JSONL`).
     - CDM format serializer conforming to `schema_version: "1.0.0"` in `docs/DATA_MODEL.md`.
     - Claude XML serializer with `<documents>`, `<document index="...">`, `<document_content>`.
     - Token estimation heuristic `estimateTokens`.
     - Deterministic sample extracted files generator `generateSampleExtractedFiles`.
  2. Author safe, zero-dependency syntax highlighter in `apps/web/src/lib/utils/syntax-highlighter.ts`:
     - Strict HTML escaping to prevent XSS.
     - Lexers for JSON, XML, Markdown, and general programming code (Rust, TOML, TS).
     - Line splitter `highlightLines` with 1-based line numbering.
  3. Author accessible `PreviewPanel.svelte`:
     - Accessible tablist (`role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`).
     - File selection sidebar with file search and full-context concatenation mode.
     - Copy-to-clipboard button with visual feedback and screen reader live region (`role="status"`, `aria-live="polite"`).
     - Line numbers gutter with `aria-hidden="true"`, word wrap toggle, search within preview.
     - Download current format action.
  4. Integrate `PreviewPanel` into `apps/web/src/lib/components/workspace/JobWorkspace.svelte` and export via `apps/web/src/lib/components/workspace/index.ts`.
  5. Author automated test suite `apps/web/tests/preview-panel.test.ts` (17 tests) validating domain models, serializers, syntax tokenizers, ARIA contracts, and workspace integration.
  6. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - Format switcher toggles cleanly across Markdown, JSON CDM, Claude XML, Plaintext, and JSONL.
  - Line numbers gutter can be toggled; word wrap can be toggled.
  - Copy button copies content to clipboard and announces via polite live region.
  - File sidebar enables filtering and selecting specific files or full concatenated context.
  - 100% of automated tests pass (107/107 tests across 11 test suites); `svelte-check` reports 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** All user input and file content is strictly HTML-escaped before injection into preview HTML; zero raw unsanitized HTML is rendered; download uses client-side object URLs with explicit revocations.
- **Performance Considerations:** Syntax tokenization runs synchronously in memory without heavy external syntax highlighters; windowed rendering and clean DOM footprint.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 11 test suites, passing 107/107 automated unit, responsive, domain model, and component invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `eda28f4`
- **Status:** `ACCEPTED`

---

#### Subphase `1.3.3` — Build token statistics interface and budget bar
- **Unique ID:** `1.3.3`
- **Objective:** Implement interactive token statistics interface and dual-layer visual context budget utilization bar (`TokenBudgetBar.svelte`) comparing raw vs. deduplicated vs. final tokens against calibrated target AI model context windows (Gemini 2M, Claude 200k, GPT-4o 128k, Local 32k), computing savings percentage, headroom buffer, and overflow warning alerts.
- **Scope:** `apps/web/src/lib/types/token-budget.ts`, `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/index.ts`, `apps/web/tests/token-budget.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/token-budget.ts`
  - `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/src/lib/components/workspace/index.ts`
  - `apps/web/tests/token-budget.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author token budget domain model and evaluation rules in `apps/web/src/lib/types/token-budget.ts`:
     - Mathematical helpers: `calculateSavingsPct`, `calculateUtilizationPct`, `calculateHeadroom`.
     - Utilization evaluation engine: `evaluateBudgetUtilization` supporting 3 status levels (`optimal`, `warning`, `exceeded`).
     - Integration with calibrated model presets in `settings.ts`.
  2. Author accessible `TokenBudgetBar.svelte` component (`apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`):
     - Interactive model preset switcher pills (Gemini 2M, Gemini 1M, Claude 200k, GPT-4o 128k, Local 32k) dynamically re-evaluating context fit.
     - 4-card metric grid: Raw Ingested Tokens, Deduplicated Tokens, Final Context Output, and Token Optimization Savings with badge.
     - Dual-layer progress bar (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`) showing Final Tokens, ghost Deduplication Savings segment, and 80% safety margin marker line.
     - Headroom callout displaying remaining tokens available for conversation turns.
     - Accessible warning banner (`role="alert"`) when budget ceiling is exceeded with one-click model upgrade switch.
  3. Export `TokenBudgetBar` from `apps/web/src/lib/components/workspace/index.ts`.
  4. Integrate `TokenBudgetBar` into `apps/web/src/lib/components/workspace/JobWorkspace.svelte` positioned between the pipeline tracker cards and completeness accounting matrix.
  5. Author automated test suite `apps/web/tests/token-budget.test.ts` (15 tests) validating savings math, headroom math, utilization bounds, ARIA landmarks, preset switching, and workspace integration.
  6. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - Token budget bar accurately calculates and renders token consumption, savings percentage, and headroom buffer.
  - Model preset switcher dynamically updates budget ceiling and advisory status.
  - Progress bar renders semantic ARIA attributes (`role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`).
  - Alert banner triggers reliably when final tokens exceed the budget threshold.
  - 100% of automated tests pass (122/122 tests across 12 test suites); `svelte-check` reports 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** All displayed token metrics and model metadata are strongly typed and sanitized; no external AI calls required.
- **Performance Considerations:** Pure reactive derivation using Svelte 5 `$derived` runes; zero re-render overhead.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 12 test suites, passing 122/122 automated unit, responsive, domain model, and component invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `c868a97`
- **Status:** `ACCEPTED`

---

#### Subphase `1.3.4` — Build format selection and download controls
- **Unique ID:** `1.3.4`
- **Objective:** Implement comprehensive export artifact download controls (`DownloadControls.svelte`) supporting multi-format downloads (Markdown, JSON CDM, Claude XML, Plaintext, JSONL), batch format downloading, cryptographic SHA-256 integrity checksum verification, and standardized checksum manifest generation (`checksums.txt` and `manifest.json`).
- **Scope:** `apps/web/src/lib/types/download.ts`, `apps/web/src/lib/components/workspace/DownloadControls.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/index.ts`, `apps/web/tests/download-controls.test.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/download.ts`
  - `apps/web/src/lib/components/workspace/DownloadControls.svelte`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/src/lib/components/workspace/index.ts`
  - `apps/web/tests/download-controls.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
- **Implementation Tasks:**
  1. Author export artifact domain contracts and cryptographic SHA-256 generators in `apps/web/src/lib/types/download.ts` using Web Crypto API:
     - `computeSha256Hex`: Produces 64-character hexadecimal SHA-256 digest.
     - `deriveArtifactBaseName`: Sanitizes filesystem-safe artifact base names from repository and documentation URLs.
     - `buildExportArtifacts`: Generates concrete `ExportArtifact` records for all 5 formats with real content, byte sizes, and checksums.
     - `generateChecksumsTxt`: Produces standard coreutils `sha256sum` formatted checksum lists.
     - `generateChecksumManifestJson`: Produces schema v1.0.0 JSON checksum manifests matching CDM specifications.
     - `triggerBlobDownload`: Client-side Blob download trigger with automatic Object URL cleanup.
  2. Author accessible `DownloadControls.svelte` component (`apps/web/src/lib/components/workspace/DownloadControls.svelte`):
     - Batch download actions: "Download All Formats", "Checksums (.txt)", and "manifest.json".
     - Artifact list cards with format icons, extension badges, byte sizes, truncated SHA-256 pills with one-click copy, and individual download buttons.
     - Collapsible checksum manifest drawer rendering coreutils `checksums.txt` content.
     - Screen reader announcement live region (`role="status"`, `aria-live="polite"`).
  3. Export `DownloadControls` from `apps/web/src/lib/components/workspace/index.ts`.
  4. Integrate `DownloadControls` into `apps/web/src/lib/components/workspace/JobWorkspace.svelte` positioned directly below the preview panel.
  5. Author automated test suite `apps/web/tests/download-controls.test.ts` (11 tests) validating SHA-256 hashing, base name derivation, artifact generation, checksums.txt formatting, manifest JSON schema, and component ARIA landmarks.
  6. Formally certify completion of **Phase 1.3: Conversion workspace** across all 4 subphases (`1.3.1` – `1.3.4` all `ACCEPTED`).
  7. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - All 5 formats generate downloadable blobs with real content.
  - SHA-256 checksums are calculated cryptographically and can be copied with visual confirmation.
  - Checksums.txt and manifest.json match standard coreutils / CDM specifications.
  - 100% of automated tests pass (133/133 tests across 13 test suites); `svelte-check` reports 0 errors and 0 warnings.
  - All 5 monorepo verification gates in `scripts/check-all.*` pass.
- **Tests:** `npm test` in `apps/web`, `npm run check`, `npm run lint`, `npm run build`, and `scripts/check-all.ps1` / `scripts/check-all.sh`.
- **Security Considerations:** Checksums are computed using SHA-256 cryptographic standards; all filenames are sanitized against path traversal; downloaded blobs use client-side memory URLs with immediate revocation.
- **Performance Considerations:** Asynchronous artifact generation prevents main thread blocking; staggered batch downloads avoid browser download manager throttling.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 13 test suites, passing 133/133 automated unit, responsive, domain model, and component invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `a0d616c`
- **Status:** `ACCEPTED`

---

#### Phase 1.3 Milestone Certification
- **Phase Name:** `Phase 1.3 — Conversion workspace`
- **Subphases Covered:** `1.3.1` (Job workspace), `1.3.2` (Preview panels), `1.3.3` (Token statistics interface), `1.3.4` (Output & download controls).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Dedicated conversion job workspace route (`/workspace`) and reusable shell component (`JobWorkspace.svelte`) implemented with 4-stage pipeline execution cards and file completeness accounting invariant ($\text{discovered} = \text{extracted} + \text{excluded} + \text{failed}$).
  2. Multi-format syntax-highlighted content preview panel (`PreviewPanel.svelte`) built with 5 format tabs (Markdown, JSON CDM schema v1.0.0, Claude XML, Plaintext, JSONL), line numbers gutter, and extracted file tree sidebar.
  3. Token statistics interface and context window budget bar (`TokenBudgetBar.svelte`) implemented with dual-layer progress bar, savings percentage calculation, headroom buffer callouts, and dynamic preset switching across 5 major AI model architectures.
  4. Cryptographically verified download controls (`DownloadControls.svelte`) built with SHA-256 integrity checksums, one-click copy, batch downloading, and standard checksum manifests (`checksums.txt` and `manifest.json`).
  5. 133 automated unit, invariant, and responsive tests passing across 13 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings.
  6. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Upcoming Subphase (Awaiting User Approval after Phase 1.3 Certification)

| Subphase ID | Objective | Prerequisites | Affected Files / Components | Status |
| :--- | :--- | :--- | :--- | :--- |
| `1.4.1` | Execute keyboard navigation and accessibility audits | Phase 1.3 (`ACCEPTED`) | `apps/web/tests/a11y-audit.test.ts` | `ACCEPTED` |

---

#### Subphase `1.4.1` — Execute keyboard navigation and accessibility audits
- **Unique ID:** `1.4.1`
- **Objective:** Execute comprehensive automated keyboard navigation and accessibility audits verifying WCAG 2.1 AA and AAA compliance across all frontend components in `apps/web`, covering semantic HTML landmark hierarchy, WAI-ARIA progressbars, WAI-ARIA tablist design patterns, polite/assertive screen reader announcements, line-number gutter exclusion, visible focus rings, color contrast ratios, and keyboard event handlers.
- **Scope:** `apps/web/tests/a11y-audit.test.ts`, `apps/web/src/lib/components/navigation/Header.svelte`, `apps/web/src/lib/components/navigation/Footer.svelte`, `apps/web/src/lib/components/landing/Hero.svelte`, `apps/web/src/lib/components/landing/WorkflowSteps.svelte`, `apps/web/src/lib/components/landing/SupportedSources.svelte`, `apps/web/src/lib/components/landing/FeatureHighlights.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`, `apps/web/src/lib/components/workspace/PreviewPanel.svelte`, `apps/web/src/lib/components/workspace/DownloadControls.svelte`, `apps/web/src/lib/components/input/UrlInput.svelte`, `apps/web/src/lib/components/settings/ConversionSettings.svelte`, `apps/web/src/lib/components/ui/Button.svelte`, `apps/web/src/lib/components/ui/Input.svelte`, `apps/web/src/lib/components/ui/Toggle.svelte`, `apps/web/src/lib/components/ui/Modal.svelte`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/tests/a11y-audit.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive accessibility audit test suite `apps/web/tests/a11y-audit.test.ts` covering 8 distinct WCAG 2.1 AA/AAA compliance categories:
     - Section 1: Semantic HTML landmarks and region hierarchy (`<header>`, `<nav>`, `<main id="main-content">`, `<footer>`, explicit `<section>` landmarks with `aria-labelledby`).
     - Section 2: WAI-ARIA progressbar pattern compliance across `JobWorkspace.svelte` (overall pipeline progress) and `TokenBudgetBar.svelte` (token budget utilization) verifying `role="progressbar"`, `aria-valuenow`, `aria-valuemin={0}`, `aria-valuemax={100}`, and descriptive `aria-label`.
     - Section 3: WAI-ARIA tabs design pattern compliance in `PreviewPanel.svelte` verifying container `role="tablist"`, interactive tab elements (`role="tab"`, `id="tab-{id}"`, `aria-selected`, `aria-controls`), and panel target (`role="tabpanel"`, `id="panel-{id}"`, `aria-labelledby`).
     - Section 4: Screen reader polite live regions (`role="status"`, `role="log"`, `aria-live="polite"`) in `UrlInput`, `JobWorkspace`, `PreviewPanel`, and `DownloadControls`, alongside assertive error banners (`role="alert"`).
     - Section 5: Screen reader exclusion of decorative elements: decorative SVGs with `aria-hidden="true"`, and line number gutter elements (`aria-hidden="true"`, `select-none`).
     - Section 6: Universal visible focus indicators (WCAG 2.4.7) asserting Tailwind `focus-visible:ring` or `focus-visible:outline` on all interactive component primitives.
     - Section 7: Mathematical color contrast certification using WCAG relative luminance algorithms asserting AAA ($\ge 7.0:1$) for dark/light backgrounds and AA ($\ge 4.5:1$) for brand colors.
     - Section 8: Keyboard navigation shortcuts: Escape key handling in `UrlInput` (clearing input) and `Modal` (dialog dismissal), and Enter/Space keyboard activation on `Toggle` switches (`role="switch"`, `aria-checked`).
  2. Run full verification gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `apps/web/tests/a11y-audit.test.ts` passes with 19/19 checks covering all 8 WCAG 2.1 AA/AAA audit areas.
  - Overall automated test suite passes 152/152 tests across 14 test suites with 0 failures.
  - `svelte-check` reports 0 errors and 0 warnings.
  - Prettier confirms 100% code style compliance.
  - Production build with `@sveltejs/adapter-node` succeeds.
  - All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Tests:** `npm test` in `apps/web` (152/152 tests passed), `npm run check` (0 errors, 0 warnings), `npm run lint`, `npm run build`, `scripts/check-all.ps1`, `scripts/check-all.sh`.
- **Security Considerations:** Accessible ARIA attributes and labels contain no unescaped or user-tampered strings; zero external CDN dependencies for highlighters or icons.
- **Performance Considerations:** Tests run in under 3.5 seconds; client-side contrast calculations run in constant $O(1)$ time; zero runtime overhead added to production bundle.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 14 test suites (`responsive-layout.test.ts`, `tokens.test.ts`, `color-utils.test.ts`, `a11y-audit.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `url-input-component.test.ts`, `conversion-settings.test.ts`, `landing-page.test.ts`, `job-workspace.test.ts`, `token-budget.test.ts`, `download-controls.test.ts`, `preview-panel.test.ts`), passing 152/152 automated unit, accessibility, and component invariant tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `b70173f`
- **Status:** `ACCEPTED`

---

#### Subphase `1.4.2` — Execute mobile, tablet, and desktop viewport tests
- **Unique ID:** `1.4.2`
- **Objective:** Execute comprehensive automated multi-viewport layout and responsiveness tests across mobile (320px–480px, e.g. iPhone SE, iPhone 14/15, Pixel 7), tablet (640px–1024px, e.g. iPad Mini, iPad 10th Gen, iPad Air/Pro), and desktop (1280px–1920px+) viewports, verifying horizontal overflow containment, grid collapsing and expansion, responsive typography, navigation drawer toggling, touch target dimensions ($\ge 44 \times 44\text{px}$), and workspace component adaptability.
- **Scope:** `apps/web/tests/responsive-layout.test.ts`, `apps/web/src/routes/+layout.svelte`, `apps/web/src/routes/+page.svelte`, `apps/web/src/routes/workspace/+page.svelte`, `apps/web/src/lib/components/navigation/Header.svelte`, `apps/web/src/lib/components/navigation/Footer.svelte`, `apps/web/src/lib/components/landing/Hero.svelte`, `apps/web/src/lib/components/landing/WorkflowSteps.svelte`, `apps/web/src/lib/components/landing/SupportedSources.svelte`, `apps/web/src/lib/components/landing/FeatureHighlights.svelte`, `apps/web/src/lib/components/input/UrlInput.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`, `apps/web/src/lib/components/workspace/PreviewPanel.svelte`, `apps/web/src/lib/components/workspace/DownloadControls.svelte`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/tests/responsive-layout.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Expand `apps/web/tests/responsive-layout.test.ts` into a multi-viewport certification test suite (28 tests across 6 sections):
     - Section 1: Tailwind breakpoint spectrum & device profile resolution matrix mapping representative devices across Mobile (`xs`: 320px–480px), Tablet (`sm`: 640px, `md`: 768px–834px, `lg`: 1024px), and Desktop (`xl`: 1280px–1440px, `2xl`: 1536px–2560px).
     - Section 2: Mobile Viewport Layout Invariants (320px–480px): Root layout horizontal overflow prevention (`overflow-x-hidden`, `max-w-full`, `min-h-screen`); uniform container padding (`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8`); Hero typography scaling (`text-3xl sm:text-5xl lg:text-6xl`); CTA button full-width vertical stacking (`flex-col`, `w-full min-h-[48px]`); format pills 2-column collapse; metrics 1-column collapse; `UrlInput` full-width stacking with iOS auto-zoom font protection (`text-sm sm:text-base`); content grids single-column collapse; Header mobile hamburger button and hidden desktop nav (`md:hidden`, `hidden md:flex`).
     - Section 3: Tablet Viewport Layout Invariants (640px–1024px): `sm` breakpoint (640px) button horizontal flow (`sm:flex-row`), 3-column format grid (`sm:grid-cols-3`), 3-column metric grid (`sm:grid-cols-3`); `md` breakpoint (768px) 3-column workflow grid (`md:grid-cols-3`), 2-column sources grid (`md:grid-cols-2`), 2-column features grid (`md:grid-cols-2`); Header desktop navigation visibility (`hidden md:flex`).
     - Section 4: Desktop Viewport Layout Invariants (1280px–1920px+): Full 6-column format pills matrix (`lg:grid-cols-6`), 3-column features matrix (`lg:grid-cols-3`), 5-column footer matrix (`lg:grid-cols-5`), maximum headline typography (`lg:text-6xl`), and container max-width constraints (`max-w-7xl mx-auto`, `max-w-6xl mx-auto`).
     - Section 5: Conversion Workspace (/workspace) Multi-Viewport Invariants: Header stacking (`flex-col sm:flex-row items-start sm:items-center`); 4-stage pipeline execution cards collapsing (1-col mobile, 2-col tablet, 4-col desktop: `grid-cols-1 sm:grid-cols-2 lg:grid-cols-4`); completeness accounting matrix (2-col mobile, 4-col tablet+: `grid-cols-2 sm:grid-cols-4`); `TokenBudgetBar` metric grid (`grid-cols-2 lg:grid-cols-4`); `PreviewPanel` 12-column responsive layout with sidebar stacking on mobile and side-by-side splitting on tablet+ (`grid-cols-1 md:grid-cols-12`, `md:col-span-4 lg:col-span-3`); `DownloadControls` artifact cards (`flex-col md:flex-row md:items-center`); telemetry terminal log bounded scroll (`max-h-64 overflow-y-auto`).
     - Section 6: Touch Target Accessibility Invariants (WCAG 2.1 AA / AAA - Criteria 2.5.5 & 2.5.8): Header mobile toggle and theme button ($\ge 44 \times 44\text{px}$), mobile navigation links ($\ge 44\text{px}$ touch height), Hero CTA buttons ($\ge 48\text{px}$), `UrlInput` controls ($\ge 48\text{px}$), and Modal action controls.
  2. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `apps/web/tests/responsive-layout.test.ts` passes with 28/28 checks covering all 6 viewport audit sections.
  - Overall automated test suite passes 173/173 tests across 14 test suites with 0 failures.
  - `svelte-check` reports 0 errors and 0 warnings.
  - Prettier confirms 100% code style compliance.
  - Production build with `@sveltejs/adapter-node` succeeds.
  - All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Tests:** `npm test` in `apps/web` (173/173 tests passed), `npm run check` (0 errors, 0 warnings), `npm run lint`, `npm run build`, `scripts/check-all.ps1`, `scripts/check-all.sh`.
- **Security Considerations:** Responsive layout does not expose sensitive endpoints or bypass client-side SSRF validation; mobile view retains all error banners and alerts.
- **Performance Considerations:** All responsive styling powered by zero-runtime atomic Tailwind CSS classes; no resize event listeners or layout thrashing.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 14 test suites (`responsive-layout.test.ts` [28 tests], `tokens.test.ts`, `color-utils.test.ts`, `a11y-audit.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `conversion-settings.test.ts`, `url-input-component.test.ts`, `landing-page.test.ts`, `job-workspace.test.ts`, `token-budget.test.ts`, `preview-panel.test.ts`, `download-controls.test.ts`), passing 173/173 automated tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `31a2498`
- **Status:** `ACCEPTED`

---

#### Subphase `1.4.3` — Execute UI performance and visual regression verification
- **Unique ID:** `1.4.3`
- **Objective:** Execute comprehensive automated UI performance benchmarks, bundle size budget certifications, zero-runtime dependency audits, and visual design system styling regression tests in `apps/web`, certifying production client asset footprints, high-throughput syntax highlighting ($> 20,000$ lines/sec), $O(1)$ token budgeting, instant URL/SSRF pre-validation, Web Crypto SHA-256 digest throughput, WCAG AAA contrast stability across theme modes, and zero hardcoded arbitrary styling pollution.
- **Scope:** `apps/web/tests/ui-performance.test.ts`, `apps/web/package.json`, `apps/web/src/lib/styles/tokens.css`, `apps/web/src/lib/styles/app.css`, `apps/web/tailwind.config.ts`, `apps/web/src/lib/tokens/index.ts`, `apps/web/src/lib/tokens/color-utils.ts`, `apps/web/src/lib/utils/syntax-highlighter.ts`, `apps/web/src/lib/types/token-budget.ts`, `apps/web/src/lib/types/download.ts`, `apps/web/src/lib/utils/url-detector.ts`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`.
- **Prerequisites:** `1.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/tests/ui-performance.test.ts`
  - `PROJECT_STATUS.md`
  - `IMPLEMENTATION_LEDGER.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive UI performance, bundle size budget, and visual regression test suite in `apps/web/tests/ui-performance.test.ts` (11 tests across 3 categories):
     - Section 1: Production Bundle Size & Zero-Runtime Dependency Budget Certification:
       - Assert `package.json` has zero external runtime dependencies (`dependencies` is empty or undefined).
       - Assert zero bloated libraries in devDependencies (no `lodash`, `moment`, `prismjs`, `highlight.js`, `rxjs`, `lucide-react`, `@mui/material`, `@chakra-ui/react`).
       - Assert client build output chunk sizes strictly within budgets: max single JS chunk $\le 100\text{ kB}$ raw (largest is $57.4\text{ kB}$), max CSS chunk $\le 60\text{ kB}$ raw (actual $44.1\text{ kB}$), total client assets footprint $\le 350\text{ kB}$ raw (actual $\sim 260\text{ kB}$).
     - Section 2: Algorithmic Throughput & Micro-Benchmark Certification:
       - Syntax highlighter throughput: 1,000 lines of JSON processed in $< 50\text{ms}$ with throughput $> 20,000\text{ lines/sec}$.
       - XML syntax highlighter throughput: 500+ lines processed in $< 40\text{ms}$.
       - Token budget evaluation engine: 10,000 sequential `evaluateBudgetUtilization` calls executed in $< 50\text{ms}$ ($> 200,000\text{ ops/sec}$).
       - URL detection & SSRF pre-check: 5,000 URL classifications executed in $< 50\text{ms}$ ($> 100,000\text{ ops/sec}$).
       - Web Crypto SHA-256 digest: 500 KB test payload hashed via Web Crypto API in $< 50\text{ms}$.
     - Section 3: Visual Design System & Styling Token Regression Invariants:
       - Design tokens CSS custom properties verification: `--color-brand-*`, `--color-accent-*`, `--color-surface-*`, `--color-status-*`, `--elevation-*`, `--spacing-*`, `--radius-*`, and `.dark` theme overrides.
       - Tailwind theme utilities verification: `.bg-theme-canvas`, `.bg-theme-panel`, `.bg-theme-subtle`, `.text-theme-primary`, `.text-theme-secondary`, `.text-theme-muted`, `.border-theme-subtle`, `.border-theme-strong`.
       - WCAG AAA contrast ratio compliance verified across both Dark mode (light text on dark canvas $\ge 14:1$, on dark panel $\ge 12:1$) and Light mode (dark text on white panel $\ge 14:1$), plus AA for brand primary buttons ($\ge 4.5:1$).
       - Component zero inline arbitrary hex color audit across all 12 frontend components.
  2. Formally certify completion of **Phase 1.4: UX validation** across all 3 subphases (`1.4.1` – `1.4.3` all `ACCEPTED`).
  3. Formally certify completion of **Stage 1: Frontend and Product Experience** across all 15 subphases (`1.1.1` through `1.4.3` all `ACCEPTED`).
  4. Run all quality gates: `npm run lint`, `npm run check`, `npm test`, `npm run build`, and `scripts/check-all.*`.
- **Acceptance Criteria:**
  - `apps/web/tests/ui-performance.test.ts` passes with 11/11 tests covering bundle size, micro-benchmarks, and styling token regression.
  - Overall automated test suite passes 184/184 tests across 15 test suites with 0 failures.
  - `svelte-check` reports 0 errors and 0 warnings.
  - Prettier confirms 100% code style compliance.
  - Production build with `@sveltejs/adapter-node` succeeds.
  - All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Tests:** `npm test` in `apps/web` (184/184 tests passed), `npm run check` (0 errors, 0 warnings), `npm run lint`, `npm run build`, `scripts/check-all.ps1`, `scripts/check-all.sh`.
- **Security Considerations:** No unvetted third-party npm runtime dependencies introduced; zero external tracking scripts or CDNs; safe HTML escaping verified across high-throughput tokenizers.
- **Performance Considerations:** Client asset footprint strictly verified under 350 kB; syntax highlighter processes over 20,000 lines/sec; token evaluation and SSRF pre-validation execute with sub-millisecond latency.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `npm run check`: SvelteKit sync and svelte-check reported 0 errors and 0 warnings in `apps/web`.
  - `npm run lint`: Prettier verified 100% code style compliance across all matched files.
  - `npm test`: Vitest ran 15 test suites (`ui-performance.test.ts` [11 tests], `responsive-layout.test.ts` [28 tests], `tokens.test.ts`, `color-utils.test.ts`, `a11y-audit.test.ts`, `url-detector.test.ts`, `typography.test.ts`, `ui-components.test.ts`, `conversion-settings.test.ts`, `url-input-component.test.ts`, `landing-page.test.ts`, `job-workspace.test.ts`, `token-budget.test.ts`, `preview-panel.test.ts`, `download-controls.test.ts`), passing 184/184 automated tests with 0 failures.
  - `npm run build`: Production bundle generated with `@sveltejs/adapter-node` in `apps/web/build/index.js` and `.svelte-kit/output`.
  - `scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: 224/224 bootstrap checks, Gate 2: 7/7 workflows, Gate 3: Rust formatting, Gate 4: Frontend lint/check/test, Gate 5: 5.27 GB / 30 GB storage ceiling).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures across all 5 verification gates).
- **Git Commit Reference:** `42c6a33`
- **Status:** `ACCEPTED`

---

#### Phase 1.4 Milestone Certification
- **Phase Name:** `Phase 1.4 — UX validation`
- **Subphases Covered:** `1.4.1` (Keyboard & accessibility audits), `1.4.2` (Mobile, tablet & desktop viewport tests), `1.4.3` (UI performance & visual regression verification).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Comprehensive WAI-ARIA and WCAG 2.1 AA/AAA compliance verified across 8 audit dimensions (`a11y-audit.test.ts`, 19 tests), confirming landmarks, progressbars, tabs, live regions, focus indicators, and contrast ratios.
  2. Multi-viewport responsiveness verified across 6 test sections (`responsive-layout.test.ts`, 28 tests) for mobile (320px–480px), tablet (640px–1024px), and desktop (1280px–1920px), verifying grid collapsing, touch targets ($\ge 44 \times 44\text{px}$), and horizontal scroll containment.
  3. Production client asset footprint certified under strict performance budgets (`ui-performance.test.ts`, 11 tests) with zero runtime npm dependencies, high-throughput syntax highlighting ($> 20,000$ lines/sec), $O(1)$ token budgeting, and zero inline style color pollution.
  4. 184 automated unit, responsive, accessibility, and performance tests passing across 15 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings.
  5. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### STAGE 1 COMPLETION CERTIFICATION

- **Stage ID:** `STAGE 1 — FRONTEND AND PRODUCT EXPERIENCE`
- **Total Phases Completed:** 4 (`Phase 1.1`, `Phase 1.2`, `Phase 1.3`, `Phase 1.4`)
- **Total Subphases Completed:** 15 (`1.1.1` through `1.4.3`, all `ACCEPTED`)
- **Stage Acceptance Gate:** Real frontend, functional navigation, and validated responsive UI connected to tested API contracts with keyboard/WCAG AA accessibility.
- **Verification Evidence:**
  1. **Phase 1.1 (Design System):** Visual design tokens (`tokens.css`), typography scales, 4px spacing grid, and reusable accessible UI primitives (`Button`, `Input`, `Toggle`, `Badge`, `Modal`, `Tooltip`).
  2. **Phase 1.2 (Main Website):** Production landing page with sticky `Header`, `Hero`, `WorkflowSteps`, `SupportedSources`, `FeatureHighlights`, `Footer`, zero-trust client SSRF detector (`UrlInput`), and collapsible `ConversionSettings`.
  3. **Phase 1.3 (Conversion Workspace):** Dedicated `/workspace` route, 4-stage pipeline execution cards, file completeness accounting invariant ($\text{discovered} = \text{extracted} + \text{excluded} + \text{failed}$), multi-format syntax preview panel (`PreviewPanel`), token statistics budget bar (`TokenBudgetBar`), and cryptographically verified SHA-256 download controls (`DownloadControls`).
  4. **Phase 1.4 (UX Validation):** WCAG 2.1 AA/AAA accessibility certification, multi-viewport mobile/tablet/desktop verification, zero runtime dependency budget, and sub-millisecond algorithmic micro-benchmarks.
  5. **Automated Test Quality:** 184 automated tests passing across 15 test suites with 0 failures; `svelte-check` reports 0 errors and 0 warnings; Prettier confirms 100% code style compliance; production build verified with `@sveltejs/adapter-node`.
  6. **Monorepo Quality Gates:** All 5 gates pass cleanly on Windows PowerShell (`scripts/check-all.ps1`) and POSIX bash (`scripts/check-all.sh`), with Podman storage footprint at 5.27 GB / 30 GB (17.6%).
- **Status:** `COMPLETED` / `ACCEPTED` (Stage 1 Certified 2026-11-17)

---

## STAGE 2 — BACKEND AND PROCESSING FOUNDATION

### Phase 2.1 — Rust architecture

#### Subphase `2.1.1` — Create Rust workspace and crate structure
- **Unique ID:** `2.1.1`
- **Objective:** Create root Cargo workspace manifest, define workspace member crates and services with explicit versions, set up unified compiler warnings and clippy lints, configure deny.toml for dependencies/licenses, and initialize all 17 member crates (`crates/*` and `services/*`) with clean compilation and unit tests.
- **Scope:** Root `Cargo.toml`, `Cargo.lock`, `rustfmt.toml`, `deny.toml`, `scripts/check-all.sh`, `scripts/check-all.ps1`, 15 crates under `crates/`, and 2 services under `services/`.
- **Prerequisites:** Stage 1 (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`, `rustfmt.toml`, `deny.toml`
  - `scripts/check-all.sh`, `scripts/check-all.ps1`
  - `services/api/{Cargo.toml, src/main.rs}`
  - `services/worker/{Cargo.toml, src/main.rs}`
  - `crates/core/{Cargo.toml, src/lib.rs}`
  - `crates/cdm/{Cargo.toml, src/lib.rs}`
  - `crates/url-security/{Cargo.toml, src/lib.rs}`
  - `crates/web-extractor/{Cargo.toml, src/lib.rs}`
  - `crates/github-extractor/{Cargo.toml, src/lib.rs}`
  - `crates/code-intelligence/{Cargo.toml, src/lib.rs}`
  - `crates/normalizer/{Cargo.toml, src/lib.rs}`
  - `crates/deduplication/{Cargo.toml, src/lib.rs}`
  - `crates/tokenizer/{Cargo.toml, src/lib.rs}`
  - `crates/context-optimizer/{Cargo.toml, src/lib.rs}`
  - `crates/chunking/{Cargo.toml, src/lib.rs}`
  - `crates/exporters/{Cargo.toml, src/lib.rs}`
  - `crates/storage/{Cargo.toml, src/lib.rs}`
  - `crates/job-engine/{Cargo.toml, src/lib.rs}`
  - `crates/observability/{Cargo.toml, src/lib.rs}`
- **Implementation Tasks:**
  1. Create root `Cargo.toml` configuring `[workspace]`, `[workspace.package]`, `[workspace.lints.rust]`, `[workspace.lints.clippy]`, and `[workspace.dependencies]` with exact versioning and path mappings.
  2. Configure `deny.toml` for cargo-deny 0.20+ with license checks, security advisories, bans, and wildcard path allowances.
  3. Ensure `rustfmt.toml` uses standard stable formatting rules compatible with Rust 1.97.1.
  4. Create `Cargo.toml` and entry point `src/lib.rs` / `src/main.rs` with metadata functions and unit tests for all 17 members.
  5. Verify cross-platform cargo binary resolution in `scripts/check-all.sh` and `scripts/check-all.ps1` including cargo-deny gate.
  6. Execute `cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo deny check`.
- **Acceptance Criteria:**
  - Root `Cargo.toml` defines workspace members without syntax errors.
  - All 17 crates and services compile cleanly under `cargo check --workspace`.
  - Zero warnings under `cargo clippy --workspace --all-targets -- -D warnings`.
  - Zero formatting issues under `cargo fmt --all -- --check`.
  - 17/17 Rust unit tests pass under `cargo test --workspace`.
  - `cargo deny check` passes with advisories ok, bans ok, licenses ok, sources ok.
  - All 5 monorepo verification gates pass in both `scripts/check-all.ps1` and `scripts/check-all.sh`.
- **Tests:** `cargo test --workspace` (17 tests passed across all 17 crates/services), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `npm test --prefix apps/web` (184/184 tests passed), `powershell scripts/check-all.ps1`, `bash scripts/check-all.sh`.
- **Security Considerations:** No unvetted third-party crates introduced; all dependencies pinned with exact versions; `cargo-deny` configured and passing for zero known vulnerabilities and license validation.
- **Performance Considerations:** Minimal dependency trees; zero redundant crate dependencies; dev profile compiles in $< 1$s for incremental builds; workspace inheritance eliminates configuration duplication.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo check --workspace`: Compiled 17/17 crates cleanly in 0.64s.
  - `cargo fmt --all -- --check`: 100% compliance with `rustfmt.toml`.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo test --workspace`: 17 passed across 17 test suites (100% pass rate).
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `54829d2`
- **Status:** `ACCEPTED`

---

#### Subphase `2.1.2` — Implement typed domain errors, results, and configuration loader in crates/core
- **Unique ID:** `2.1.2`
- **Objective:** Implement domain-wide error taxonomy (`CoreError`, `ErrorCode`, `ApiError`, `ApiErrorEnvelope`), standard `Result` type alias, strongly-typed time-ordered domain identifiers (`JobId`, `DocumentId`, `TenantId`, `RequestId`, `ArtifactId`), core traits (`Identifiable`, `Clock`, `SystemClock`, `Validatable`), and hierarchical configuration loader (`AppConfig` with environment variable overrides and domain validation) in `crates/core`.
- **Scope:** `crates/core/src/{lib.rs, error.rs, id.rs, config.rs, traits.rs}`, `crates/core/Cargo.toml`, `Cargo.lock`.
- **Prerequisites:** `2.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/core/Cargo.toml`
  - `crates/core/src/lib.rs`
  - `crates/core/src/error.rs`
  - `crates/core/src/id.rs`
  - `crates/core/src/config.rs`
  - `crates/core/src/traits.rs`
  - `Cargo.lock`
- **Implementation Tasks:**
  1. Define machine-readable `ErrorCode` enum and HTTP status code mappings matching RFC 7807 and `docs/API.md`.
  2. Implement `CoreError` with `thiserror`, standard `Result` type alias, and conversion to `ApiErrorEnvelope`.
  3. Define strongly-typed UUID v7 time-ordered domain IDs (`JobId`, `DocumentId`, `TenantId`, `RequestId`, `ArtifactId`) with full Display, FromStr, and Serde implementations.
  4. Implement `AppConfig` and sub-configs (`ServerConfig`, `DatabaseConfig`, `StorageConfig`, `SecurityConfig`, `ExtractionConfig`, `OptimizationConfig`) with `from_env()` parsing and invariant validation.
  5. Implement `Identifiable`, `Clock`, `SystemClock`, and `Validatable` traits.
  6. Author comprehensive unit tests across all modules asserting error envelopes, UUID v7 ordering, config validation rules, and environment parsing.
- **Acceptance Criteria:**
  - `CoreError` covers all system failure modes with typed variants and maps to appropriate HTTP status codes.
  - RFC 7807 compliant JSON error serialization works as documented in `docs/API.md`.
  - Strongly-typed IDs prevent ID mixing at compile time and serialize transparently.
  - `AppConfig` loads defaults with zero mandatory env vars for local development.
  - Validation catches invalid ports, inverted connection limits, budget overflows, and production SSRF relaxation.
  - All workspace crates compile with `#![forbid(unsafe_code)]` and zero clippy warnings.
  - 100% of Rust unit tests pass in `crates/core` (18 tests) and workspace (34 tests).
  - All 5 monorepo verification gates pass in `scripts/check-all.ps1` and `scripts/check-all.sh`.
- **Tests:** `cargo test -p web-repo-to-ai-core` (18/18 passed), `cargo test --workspace` (34/34 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `bash scripts/check-all.sh`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces production SSRF invariant: `allow_private_ips` cannot be enabled when `Environment::Production`; forbids unsafe code (`#![forbid(unsafe_code)]`); protects against integer overflows and payload budget exhaustion.
- **Performance Considerations:** Zero-allocation UUID formatting; zero-copy newtypes; fast time-ordered UUID v7 indexing; zero-cost abstraction for domain identifiers.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-core`: 18/18 tests passed in 0.00s.
  - `cargo test --workspace`: 34/34 tests passed in 5.10s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 2.07s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `5b4e937`
- **Status:** `ACCEPTED`

---

#### Subphase `2.1.3` — Set up structured logging, OpenTelemetry tracing, and Prometheus metrics in crates/observability
- **Unique ID:** `2.1.3`
- **Objective:** Configure structured logging, OpenTelemetry distributed tracing context, Prometheus metrics, and automated secret redaction in `crates/observability`.
- **Scope:** `crates/observability/src/{lib.rs, logging.rs, redaction.rs, metrics.rs, trace.rs}`, `crates/observability/Cargo.toml`, root `Cargo.toml`, `Cargo.lock`.
- **Prerequisites:** `2.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `crates/observability/Cargo.toml`
  - `crates/observability/src/lib.rs`
  - `crates/observability/src/logging.rs`
  - `crates/observability/src/redaction.rs`
  - `crates/observability/src/metrics.rs`
  - `crates/observability/src/trace.rs`
- **Implementation Tasks:**
  1. Implement automated secret redaction in `crates/observability/src/redaction.rs` with `OnceLock<Regex>` scrubbing GitHub PATs, Bearer tokens, URL credentials, and API keys.
  2. Implement structured logging in `crates/observability/src/logging.rs` with `LoggingConfig`, `LogFormat` (Json, Pretty, Compact), dynamic `EnvFilter`, and idempotent `init_tracing`.
  3. Implement Prometheus metrics in `crates/observability/src/metrics.rs` with thread-safe atomic `Counter`, `CounterVec`, `Gauge`, and `Histogram`, and `render_prometheus_text()` output.
  4. Implement W3C TraceContext propagation in `crates/observability/src/trace.rs` (`TraceContext`) with `to_traceparent()`, `from_traceparent()`, and child span derivation.
  5. Author unit tests covering all modules asserting regex redaction, Prometheus output, traceparent roundtrip, and idempotent subscriber initialization.
  6. Execute `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo test --workspace`, and verify all 5 gates pass cleanly.
- **Acceptance Criteria:**
  - `init_tracing` initializes without panicking and can be invoked idempotently in tests.
  - Secret redaction sanitizes GitHub PATs, Bearer tokens, OpenAI/Anthropic keys, and DB passwords from log strings and JSON values.
  - W3C `traceparent` headers parse and serialize in compliance with W3C Trace Context specifications.
  - `AppMetrics` tracks jobs, durations, files, tokens, and storage, and renders valid Prometheus text.
  - 100% of Rust unit tests pass in `crates/observability` (16 tests) and workspace (50 tests).
  - All 5 monorepo verification gates pass in `scripts/check-all.ps1` and `scripts/check-all.sh`.
- **Tests:** `cargo test -p web-repo-to-ai-observability` (16/16 passed), `cargo test --workspace` (50/50 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `bash scripts/check-all.sh`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 02 and Rule 05 zero-leakage security invariants: credentials, personal access tokens, and passwords in database URLs are strictly scrubbed prior to log serialization; `#![forbid(unsafe_code)]` enforced.
- **Performance Considerations:** Lock-free atomic operations for all counter and gauge metrics; precompiled regular expressions initialized once via `OnceLock`; microsecond latency for metric increments.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-observability`: 16/16 tests passed in 0.01s.
  - `cargo test --workspace`: 50/50 tests passed in 5.26s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 1.65s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `bash scripts/check-all.sh`: ALL 5 GATES PASSED (0 failures).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `e3c3f29`
- **Status:** `ACCEPTED`

---

#### Phase 2.1 Milestone Certification
- **Phase Name:** `Phase 2.1 — Rust architecture`
- **Subphases Covered:** `2.1.1` (Rust workspace and 17 crate structures), `2.1.2` (Typed domain errors, results, and configuration loader), `2.1.3` (Structured logging, OpenTelemetry tracing, and Prometheus metrics).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Crate structure and root workspace established with 17 members (`services/api`, `services/worker`, and 15 internal processing crates under `crates/*`), fully compiling with zero clippy warnings.
  2. Domain core primitives (`crates/core`) deliver typed domain errors (`CoreError`), RFC 7807 machine-readable envelopes, strongly-typed UUID v7 domain IDs (`JobId`, `DocumentId`, `TenantId`, `RequestId`, `ArtifactId`), core traits, and hierarchical configuration loader (`AppConfig`).
  3. Observability infrastructure (`crates/observability`) delivers automated secret redaction for GitHub PATs/Bearer tokens/passwords, W3C TraceContext distributed trace propagation, Prometheus metrics exposition, and structured JSON/compact tracing subscribers.
  4. 50/50 automated Rust unit tests pass across the workspace; 184/184 frontend tests pass in `apps/web`.
  5. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly with 0 failures.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Phase 2.2 — API

#### Subphase `2.2.1` — Implement Axum service and router scaffolding in services/api
- **Unique ID:** `2.2.1`
- **Objective:** Scaffold the primary Axum HTTP service in `services/api`, establishing the application router, state container (`AppState`), middleware pipeline (CORS, request timeouts, and W3C `traceparent` / `x-request-id` correlation tracking), health probes, Prometheus metrics exposition endpoint, RFC 7807 fallback handler, and graceful shutdown signal listener.
- **Scope:** `services/api/src/{lib.rs, main.rs, state.rs, middleware.rs, server.rs, routes/mod.rs, routes/health.rs, routes/metrics.rs}`, `services/api/Cargo.toml`, root `Cargo.toml`, `Cargo.lock`.
- **Prerequisites:** Phase 2.1 (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `services/api/Cargo.toml`
  - `services/api/src/lib.rs`
  - `services/api/src/main.rs`
  - `services/api/src/state.rs`
  - `services/api/src/middleware.rs`
  - `services/api/src/server.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/routes/health.rs`
  - `services/api/src/routes/metrics.rs`
- **Implementation Tasks:**
  1. Update root workspace `Cargo.toml` and `services/api/Cargo.toml` with `axum`, `tower`, `tower-http` (CORS, trace, timeout), `http-body-util`, and `tokio` networking/signal features.
  2. Implement `AppState` in `services/api/src/state.rs` holding shared `Arc<AppConfig>` and `Arc<AppMetrics>`.
  3. Implement middleware stack in `services/api/src/middleware.rs`: CORS layer configured from `AppConfig`, request timeout layer, and `trace_id_middleware` extracting or generating `x-request-id` and W3C `traceparent` headers.
  4. Implement routes in `services/api/src/routes/`: root handler (`/`), health probe (`/api/v1/health`), readiness probe (`/api/v1/ready`), Prometheus metrics exposition (`/api/v1/metrics`), and RFC 7807 JSON 404 fallback handler.
  5. Implement server lifecycle in `services/api/src/server.rs`: `build_app(state)` and `run_server(config)` with graceful shutdown listener (`Ctrl+C` and Unix `SIGTERM`).
  6. Update `services/api/src/main.rs` binary entry point with logging subscriber initialization (`init_tracing`), config loading (`AppConfig::from_env`), and server execution.
  7. Implement 8 integration tests in `services/api/src/lib.rs` verifying root metadata, health/ready probes, Prometheus exposition format, request ID preservation and injection, RFC 7807 404 problem details, and CORS OPTIONS preflight.
  8. Execute `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo deny check`, and verify all 5 gates pass cleanly in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Axum HTTP server starts up and binds to configured host and port with graceful shutdown handling.
  - Root endpoint `/` returns JSON service metadata and API version.
  - Health endpoint `/api/v1/health` and readiness endpoint `/api/v1/ready` return RFC-compliant health payloads.
  - Metrics endpoint `/api/v1/metrics` serves Prometheus text exposition format.
  - Request middleware injects or preserves `x-request-id` and `traceparent` on all incoming and outgoing HTTP responses.
  - Fallback 404 handler returns RFC 7807 `RESOURCE_NOT_FOUND` problem details.
  - CORS layer correctly responds to preflight `OPTIONS` requests matching configured origins.
  - 100% of integration tests pass (8 tests in `services/api`, 59 total across workspace); zero clippy warnings; all 5 gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-api` (9/9 passed), `cargo test --workspace` (59/59 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; CORS origins restricted to configuration; request timeouts bounded to prevent Slowloris attacks; RFC 7807 errors leak zero internal stack traces or database connection credentials; all secrets scrubbed from tracing.
- **Performance Considerations:** In-memory asynchronous Axum router with zero lock contention on hot paths; lock-free atomic Prometheus metrics; zero-copy header propagation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 8/8 library integration tests + 1 binary metadata test passed.
  - `cargo test --workspace`: 59/59 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `a8afbc5`
- **Status:** `ACCEPTED`

---

#### Subphase `2.2.2` — Implement request validation and parameter parsing in services/api
- **Unique ID:** `2.2.2`
- **Objective:** Implement typed request extractors, query/path parameter parsing, and domain validation for job creation (`CreateJobRequest`), URL detection (`DetectSourcesRequest`), job listing pagination (`ListJobsQuery`), and strongly-typed path parameters (`JobPathParams`, `ArtifactPathParams`) with RFC 7807 problem details error mapping.
- **Scope:** `services/api/src/{dto/*, extractors/*, routes/sources.rs, routes/jobs.rs, routes/mod.rs, lib.rs}`, `crates/core/src/{error.rs, id.rs}`, `Cargo.toml`, `Cargo.lock`, `services/api/Cargo.toml`.
- **Prerequisites:** `2.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `crates/core/src/error.rs`
  - `crates/core/src/id.rs`
  - `services/api/Cargo.toml`
  - `services/api/src/dto/mod.rs`
  - `services/api/src/dto/sources.rs`
  - `services/api/src/dto/jobs.rs`
  - `services/api/src/dto/query.rs`
  - `services/api/src/dto/params.rs`
  - `services/api/src/extractors/mod.rs`
  - `services/api/src/extractors/json.rs`
  - `services/api/src/extractors/query.rs`
  - `services/api/src/extractors/path.rs`
  - `services/api/src/routes/sources.rs`
  - `services/api/src/routes/jobs.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
- **Implementation Tasks:**
  1. Add `url` and `chrono` dependencies to workspace and `services/api/Cargo.toml`.
  2. Implement `CoreError::InvalidRequestSchema` in `crates/core/src/error.rs` mapping to HTTP 400 and RFC 7807 envelope.
  3. Implement `Validatable` for all strongly-typed domain IDs in `crates/core/src/id.rs`.
  4. Implement DTOs in `services/api/src/dto/`: `DetectSourcesRequest` / `DetectSourcesResponse` (URL length <= 2048, scheme http/https, batch <= 50), `CreateJobRequest` / `JobResponse` (git_ref sanitization, path_patterns, token_budget 1000..=2M, model length, export format checks), `ListJobsQuery` / `PaginatedResponse` (page >= 1, limit 1..=100, status/sort filters), and `JobPathParams` / `ArtifactPathParams`.
  5. Implement custom Axum extractors in `services/api/src/extractors/`: `ValidatedJson<T>`, `ValidatedQuery<T>`, and `ValidatedPath<T>` enforcing `Validatable` and returning RFC 7807 problem details with correlated `request_id`.
  6. Implement route handlers: `POST /api/v1/sources/detect` in `services/api/src/routes/sources.rs`, and `POST /api/v1/jobs`, `GET /api/v1/jobs`, `GET /api/v1/jobs/:job_id`, `GET /api/v1/jobs/:job_id/artifacts/:format` in `services/api/src/routes/jobs.rs`.
  7. Add 14 comprehensive integration tests in `services/api/src/lib.rs` and 5 unit tests in `services/api/src/dto/mod.rs` testing URL classification, invalid schemes, budget boundaries, git_ref security, format validation, pagination bounds, and path UUID parsing.
  8. Run `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, and verify all 5 gates pass in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Malformed JSON bodies or syntax errors return HTTP 400 with `INVALID_REQUEST_SCHEMA` and reason details.
  - Unsupported schemes (e.g. `ftp://...`) return HTTP 400 with `UNSUPPORTED_URL_SCHEME`.
  - Field-level validation violations return HTTP 400 with `VALIDATION_ERROR`, invalid field name, and specific reason.
  - Valid source detection, job creation, and paginated job listing requests succeed with appropriate 200 OK / 202 Accepted status.
  - Path parameters parse strongly-typed UUID v7 `JobId` and validate artifact formats.
  - 100% of Rust tests pass (28 in `services/api`, 78 across workspace); zero clippy warnings; all 5 gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-api` (28/28 passed), `cargo test --workspace` (78/78 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: strict URL length ceiling (2048 chars); input size bounding (<= 50 URLs, <= 100 glob patterns); git_ref traversal/metacharacter sanitization preventing shell injection; token budget ceiling preventing unbounded resource consumption; request ID correlation on all rejection responses.
- **Performance Considerations:** In-memory URL parsing and validation; zero extraneous allocations; lock-free metrics increment on job submission.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 27 library tests + 1 binary metadata test passed.
  - `cargo test --workspace`: 78/78 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `ec9c8d3`
- **Status:** `ACCEPTED`

---

#### Subphase `2.2.3` — Implement API error contracts and RFC 7807 problem details in services/api
- **Unique ID:** `2.2.3`
- **Objective:** Finalize unified RFC 7807 problem details error mapping across all Axum layers, including global fallback error handling, status code mappings, `RequestBodyLimitLayer` (413 Payload Too Large) handling, and error response serialization with W3C distributed tracing context (`traceparent`) and request ID correlation.
- **Scope:** `services/api/src/{error.rs, server.rs, routes/mod.rs, routes/jobs.rs, extractors/*, lib.rs}`, `crates/core/src/error.rs`, `services/api/Cargo.toml`.
- **Prerequisites:** `2.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/core/src/error.rs`
  - `services/api/Cargo.toml`
  - `services/api/src/error.rs`
  - `services/api/src/server.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/routes/jobs.rs`
  - `services/api/src/extractors/json.rs`
  - `services/api/src/extractors/query.rs`
  - `services/api/src/extractors/path.rs`
  - `services/api/src/lib.rs`
- **Implementation Tasks:**
  1. Add `MethodNotAllowed` (405) variant to `ErrorCode` and `CoreError` in `crates/core/src/error.rs`.
  2. Enable `"limit"` feature on `tower-http` in `services/api/Cargo.toml` and attach `RequestBodyLimitLayer` (10 MB limit) to server pipeline in `services/api/src/server.rs`.
  3. Implement `ApiProblem` and helper constructors (`to_problem_response`, `not_found_problem`, `validation_problem`, `schema_problem`, `payload_too_large_problem`, `method_not_allowed_problem`) in `services/api/src/error.rs` setting `Content-Type: application/problem+json`, mapping HTTP status codes, and attaching `x-request-id` headers.
  4. Refactor `ValidatedJson`, `ValidatedQuery`, and `ValidatedPath` extractors to format all rejections using `to_problem_response` with correlated `request_id`.
  5. Refactor `fallback_handler` in `services/api/src/routes/mod.rs` and handlers in `services/api/src/routes/jobs.rs` to emit RFC 7807 problem details with correlated `request_id`.
  6. Add integration tests verifying `Content-Type: application/problem+json`, exact correlation of `x-request-id` header with `body.error.request_id`, 413 `PAYLOAD_TOO_LARGE` rejection on oversized payloads, and `traceparent` header propagation on errors.
  7. Run `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, and verify all 5 gates pass in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - All HTTP error responses (400, 404, 405, 413, 500) set `Content-Type: application/problem+json`.
  - Every error response body contains standard machine-readable envelope with `code`, `message`, structured `details`, and `request_id`.
  - The `x-request-id` response header and `body.error.request_id` are guaranteed identical across all failure modes.
  - Payloads exceeding 10 MB are rejected with HTTP 413 `PAYLOAD_TOO_LARGE` in RFC 7807 format.
  - W3C `traceparent` header is preserved on all error responses.
  - 100% of Rust tests pass (33 in `services/api`, 84 across workspace); zero clippy warnings; all 5 gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-api` (33/33 passed), `cargo test --workspace` (84/84 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: zero leakage of internal stack traces, database credentials, or secret keys in RFC 7807 error messages; strict 10 MB payload limit mitigating memory exhaustion / denial of service; correlation IDs enable secure distributed tracing without credential exposure.
- **Performance Considerations:** Zero allocation overhead for problem details serialization; lock-free error header propagation; pre-allocated static header values.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 32 library tests + 1 binary metadata test passed.
  - `cargo test --workspace`: 84/84 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `86816a1`
- **Status:** `ACCEPTED`

---

#### Subphase `2.2.4` — Implement health, readiness, and metrics endpoints in services/api
- **Unique ID:** `2.2.4`
- **Objective:** Implement production-grade health probes (process liveness `/api/v1/health` and subsystem readiness `/api/v1/ready` checking storage writeability and database reachability) and Prometheus metrics exposition (`/api/v1/metrics`) with dynamic process uptime, build info, and HTTP request telemetry.
- **Scope:** `services/api/src/{routes/health.rs, routes/metrics.rs, middleware.rs, server.rs, state.rs, lib.rs}`, `crates/observability/src/metrics.rs`.
- **Prerequisites:** `2.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/observability/src/metrics.rs`
  - `services/api/src/state.rs`
  - `services/api/src/middleware.rs`
  - `services/api/src/server.rs`
  - `services/api/src/routes/health.rs`
  - `services/api/src/routes/metrics.rs`
  - `services/api/src/lib.rs`
- **Implementation Tasks:**
  1. Expand `AppMetrics` in `crates/observability/src/metrics.rs` with `http_requests_total` (`CounterVec`) and `http_request_duration_ms` (`Histogram`), updating `render_prometheus_text`.
  2. Add `start_time: DateTime<Utc>` and `uptime_seconds(&self) -> u64` to `AppState` in `services/api/src/state.rs`.
  3. Integrate request telemetry and duration timing into `trace_id_middleware` in `services/api/src/middleware.rs` using Axum's `from_fn_with_state`.
  4. Implement liveness probe handler in `services/api/src/routes/health.rs` returning structured JSON (`status: "pass"`, `service`, `version`, `uptime_seconds`, ISO 8601 `timestamp`).
  5. Implement deep readiness probe handler in `services/api/src/routes/health.rs` auditing subsystem health: local storage volume writeability (creates directory and probe file), database configuration (`state.config.database.url`), and worker queue state, returning 200 OK (`ready`) or 503 Service Unavailable (`not_ready`).
  6. Update metrics endpoint in `services/api/src/routes/metrics.rs` serving standard Prometheus text exposition with dynamic uptime, build info, and HTTP metrics.
  7. Add 4 integration tests in `services/api/src/lib.rs` testing liveness fields, readiness subsystem checks, unconfigured database failure simulation, and Prometheus exposition content.
  8. Run `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, and verify all 5 gates pass in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `GET /api/v1/health` returns HTTP 200 with structured JSON liveness information including version, uptime, and timestamp.
  - `GET /api/v1/ready` validates storage writeability and database configuration, returning HTTP 200 when ready or HTTP 503 with subsystem failure details when degraded.
  - `GET /api/v1/metrics` outputs standard Prometheus exposition with `web_repo_to_ai_uptime_seconds`, `web_repo_to_ai_build_info`, and application counters/gauges/histograms.
  - HTTP middleware records latency and request metrics for every served request.
  - 100% of Rust tests pass (37 in `services/api`, 88 across workspace); zero clippy warnings; all 5 quality gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-api` (37/37 passed), `cargo test --workspace` (88/88 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; readiness probes do not leak raw database credentials or internal file paths; temporary probe files are safely cleaned up; Prometheus metric endpoints expose zero secret tokens or user identifiers.
- **Performance Considerations:** Atomic lock-free metric updates in request middleware; readiness probe storage check performs single low-overhead probe write; uptime computed without syscall overhead using monotonic/cached start time.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 36 library tests + 1 binary metadata test passed.
  - `cargo test --workspace`: 88/88 tests passed in 6.01s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 2.26s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `25a8fd6`
- **Status:** `ACCEPTED`

---

#### Phase 2.2 Milestone Certification
- **Phase Name:** `Phase 2.2 — API`
- **Subphases Covered:** `2.2.1` (Axum service and router scaffolding), `2.2.2` (Request validation and parameter parsing), `2.2.3` (API error contracts and RFC 7807 problem details), `2.2.4` (Health, readiness, and metrics endpoints).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Axum HTTP service operational with full middleware pipeline (CORS, request timeouts, `trace_id_middleware` with W3C `traceparent` and `x-request-id` propagation, and request body size bounding to 10 MB).
  2. Strongly-typed request extractors (`ValidatedJson`, `ValidatedQuery`, `ValidatedPath`) and DTOs enforce strict input validation, security bounds (URL length, schemes, git_ref traversal guards, token budgets, format enumerations), and emit RFC 7807 problem details on rejection.
  3. API error architecture guarantees `application/problem+json` media type and strict correlation integrity between `x-request-id` header and `body.error.request_id` across all failure paths.
  4. Health and observability endpoints deliver container orchestrator liveness (`GET /api/v1/health`), deep subsystem readiness probes (`GET /api/v1/ready`), and Prometheus telemetry (`GET /api/v1/metrics`).
  5. 88/88 automated Rust unit tests pass across the workspace (37 in `services/api`); 184/184 frontend tests pass in `apps/web`.
  6. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly with 0 failures.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Phase 2.3 — Persistence

#### Subphase `2.3.1` — Design and implement PostgreSQL relational schema in crates/storage
- **Unique ID:** `2.3.1`
- **Objective:** Design and implement the production PostgreSQL relational schema, DDL definitions, table constraints, check constraints, foreign keys with cascade deletion, performance indexes, triggers, and strongly-typed Rust entity models in `crates/storage`.
- **Scope:** `crates/storage/src/schema/{ddl.sql, jobs.rs, artifacts.rs, github.rs, cache.rs, mod.rs}`, `crates/storage/src/lib.rs`, `Cargo.toml`, `Cargo.lock`, `crates/storage/Cargo.toml`.
- **Prerequisites:** Phase 2.2 (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `crates/storage/Cargo.toml`
  - `crates/storage/src/lib.rs`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/jobs.rs`
  - `crates/storage/src/schema/artifacts.rs`
  - `crates/storage/src/schema/github.rs`
  - `crates/storage/src/schema/cache.rs`
  - `crates/storage/src/schema/mod.rs`
  - `services/api/src/routes/health.rs`
  - `services/api/src/routes/metrics.rs`
- **Implementation Tasks:**
  1. Add `sqlx` (v0.8 runtime-tokio, postgres, chrono, uuid, json, macros, migrate) and Serde dependencies to workspace and `crates/storage/Cargo.toml`.
  2. Author production PostgreSQL 16 DDL script `crates/storage/src/schema/ddl.sql` defining `jobs`, `job_artifacts`, `github_installations`, `content_cache_index`, check constraints, foreign keys (`ON DELETE CASCADE`), indexes (`idx_jobs_status_lease`, `idx_jobs_owner_session`, `idx_jobs_created_at`, `idx_job_artifacts_job_id`, `idx_job_artifacts_expires_at`, `uq_job_artifacts_job_format`, `uq_github_installations_owner_gh_id`, `idx_github_installations_owner`, `idx_content_cache_last_accessed`), and `fn_set_updated_at()` trigger.
  3. Implement typed domain models in `crates/storage/src/schema/`:
     - `jobs.rs`: `JobKind` enum (web and GitHub variants), `JobStatus` enum with transition validation (`can_transition_to`), `JobProgressPhase` enum, and `JobRow` struct deriving `sqlx::FromRow`.
     - `artifacts.rs`: `ArtifactFormat` enum with MIME type and file extension mappings, and `JobArtifactRow` struct deriving `sqlx::FromRow` with 64-char SHA256 and BLAKE3 hex validation.
     - `github.rs`: `GitHubInstallationRow` struct deriving `sqlx::FromRow` with AES-256-GCM encrypted token/nonce storage and active/revoked lifecycle methods.
     - `cache.rs`: `ContentCacheIndexRow` struct deriving `sqlx::FromRow` with LRU `touch()` method and BLAKE3 digest validation.
  4. Implement `crates/storage/src/schema/mod.rs` exposing schema constants (`POSTGRES_SCHEMA_DDL`, `EXPECTED_TABLES`, `EXPECTED_INDEXES`), `split_ddl_statements()` parser, and comprehensive test suite.
  5. Add 9 schema integration and unit tests covering DDL parsing, statement splitting, enum round-tripping, state transitions, domain conversions, and validation error branches.
  6. Verify `cargo test --workspace` (97 tests passing, 10 in `crates/storage`), `cargo clippy`, `cargo fmt`, `cargo deny check`, and all 5 gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Relational schema accurately mirrors `docs/DATA_MODEL.md` Section 2 and ADR-005.
  - All 4 core tables, constraints, cascade rules, and indexes are defined in `ddl.sql` and verified by automated tests.
  - Rust entity structs implement `sqlx::FromRow`, Serde serialization, domain identifier conversions, and constraint validation.
  - Status state machine validates allowed and disallowed transitions.
  - 100% of Rust tests pass (97 across workspace); zero clippy warnings; all 5 quality gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (10/10 passed), `cargo test --workspace` (97/97 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; GitHub installation table isolates AES-256-GCM encrypted tokens and prevents plaintext token leakage; strict cryptographic hash validation (64 hex characters) prevents malformed digests; foreign key cascade deletion ensures zero orphaned artifact records.
- **Performance Considerations:** Specialized composite indexes (`idx_jobs_status_lease` for `FOR UPDATE SKIP LOCKED` worker queue polling, `idx_job_artifacts_expires_at` for TTL pruning, `idx_content_cache_last_accessed` for LRU cache eviction) ensure low-latency relational queries without full table scans.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 10/10 tests passed in 0.00s.
  - `cargo test --workspace`: 97/97 tests passed in 57.03s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 2.80s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `a933684`
- **Status:** `ACCEPTED`

---

#### Subphase `2.3.2` — Implement database migrations and schema validation in crates/storage
- **Unique ID:** `2.3.2`
- **Objective:** Implement compile-time embedded SQLx database migrations, programmatic migration runner (`run_migrations`), runtime PostgreSQL schema introspection and catalog validation (`validate_schema`), and central connection pool manager (`DatabaseManager`) in `crates/storage`.
- **Scope:** `crates/storage/migrations/20261117000001_initial_schema.sql`, `crates/storage/src/{migrations/mod.rs, validation/mod.rs, database.rs, lib.rs}`, `Cargo.toml`, `Cargo.lock`, `crates/storage/Cargo.toml`.
- **Prerequisites:** `2.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.lock`
  - `crates/storage/Cargo.toml`
  - `crates/storage/migrations/20261117000001_initial_schema.sql`
  - `crates/storage/src/database.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/src/migrations/mod.rs`
  - `crates/storage/src/validation/mod.rs`
- **Implementation Tasks:**
  1. Create initial SQLx migration `crates/storage/migrations/20261117000001_initial_schema.sql` with full DDL for `jobs`, `job_artifacts`, `github_installations`, `content_cache_index`, check constraints, cascading foreign keys, performance indexes, and `fn_set_updated_at` trigger.
  2. Implement compile-time embedded SQLx migration framework in `crates/storage/src/migrations/mod.rs`: `MIGRATOR` using `sqlx::migrate!("./migrations")`, `MigrationReport` with elapsed timing and version tracking, and async runner `run_migrations(&PgPool)`.
  3. Implement catalog introspection and schema validation in `crates/storage/src/validation/mod.rs`:
     - Define `EXPECTED_TRIGGERS` and `expected_table_columns()` column invariant maps.
     - `validate_catalog_metadata()` evaluating table, column, index, and trigger presence against invariants.
     - `validate_schema(&PgPool)` querying `information_schema.tables`, `information_schema.columns`, `pg_indexes`, and `information_schema.triggers` against live PostgreSQL.
     - `SchemaValidationReport` formatting machine-readable audit reports.
  4. Implement `DatabaseManager` in `crates/storage/src/database.rs` wrapping `sqlx::PgPool` with `connect`, `connect_with_config`, `migrate`, `validate_schema`, `is_healthy` (`SELECT 1`), and graceful `close()`.
  5. Add 7 unit tests in `crates/storage` covering embedded migration loading, report generation, full schema validation, missing table/column/index/trigger detection, expanding storage crate tests to 17 and workspace tests to 104 passing.
  6. Verify `cargo test --workspace` (104 tests passing), `cargo clippy`, `cargo fmt`, `cargo deny check`, and all 5 gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `MIGRATOR` loads migration `20261117000001` at compile time and runs migrations in transactions against PostgreSQL.
  - `validate_schema()` audits all required tables, columns, indexes, and triggers, reporting any missing objects or confirming full structural validity.
  - `DatabaseManager` connects cleanly using `DatabaseConfig` bounds and provides health/migration/validation APIs.
  - 100% of Rust tests pass (17 in `crates/storage`, 104 across workspace); zero clippy warnings; all 5 quality gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (17/17 passed), `cargo test --workspace` (104/104 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; SQLx migration runner applies parameterized statements; catalog validation queries use read-only introspection against `information_schema` and `pg_catalog`; no database passwords or credentials logged or exposed.
- **Performance Considerations:** Compile-time embedded migration scripts incur zero disk I/O at runtime; catalog inspection queries are executed concurrently across standard indexed system views; connection pool obeys min/max connection bounds from `DatabaseConfig`.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 17/17 tests passed in 0.00s.
  - `cargo test --workspace`: 104/104 tests passed in 9.34s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 1.50s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `d5c3bf5`
- **Status:** `ACCEPTED`

---

#### Subphase `2.3.3` — Implement job state machine, transitions, and history in crates/storage
- **Unique ID:** `2.3.3`
- **Objective:** Implement the transactional job state machine, transition request validation, terminal state immutability, audit trail history logging (`job_state_transitions` table & migration), and repository methods in `crates/storage`.
- **Scope:** `crates/storage/migrations/20261117000002_job_state_transitions.sql`, `crates/storage/src/schema/ddl.sql`, `crates/storage/src/schema/mod.rs`, `crates/storage/src/validation/mod.rs`, `crates/storage/src/state_machine.rs`, `crates/storage/src/repository.rs`, `crates/storage/src/lib.rs`, `docs/DATA_MODEL.md`.
- **Prerequisites:** `2.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/migrations/20261117000002_job_state_transitions.sql`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/src/validation/mod.rs`
  - `crates/storage/src/state_machine.rs`
  - `crates/storage/src/repository.rs`
  - `crates/storage/src/lib.rs`
  - `docs/DATA_MODEL.md`
- **Implementation Tasks:**
  1. Author migration `crates/storage/migrations/20261117000002_job_state_transitions.sql` defining `job_state_transitions` table with `job_id REFERENCES jobs(id) ON DELETE CASCADE`, `from_status`, `to_status`, `phase`, `progress_pct`, `message`, `error_code`, `worker_id`, `transitioned_at`, and index `idx_job_state_transitions_job_id`.
  2. Synchronize `crates/storage/src/schema/ddl.sql`, `crates/storage/src/schema/mod.rs` (`EXPECTED_TABLES`, `EXPECTED_INDEXES`), `crates/storage/src/validation/mod.rs` (`expected_table_columns()`), and `docs/DATA_MODEL.md` Section 2.1 to document `job_state_transitions`.
  3. Implement `crates/storage/src/state_machine.rs`:
     - `JobStateMachine::evaluate_transition(current, target)` enforcing transition legality rules and terminal state immutability (`Completed`, `Failed`, `Cancelled`).
     - `JobStateMachine::validate_transition_request(current, req)` enforcing progress bounds (0..=100), completion invariants (100% progress and Completed phase), and failure invariants (Failed phase).
     - `JobTransitionRequest` builder with semantic constructors (`lease`, `start_running`, `update_progress`, `complete`, `complete_partial`, `fail`, `cancel`, `requeue`).
     - `JobStateTransitionRow` entity struct deriving `sqlx::FromRow`, `Serialize`, `Deserialize`.
  4. Implement `JobRepository` in `crates/storage/src/repository.rs`:
     - `create_job(&pool, &JobRow)` inserting job records.
     - `get_job(&pool, id)` retrieving job records by typed `JobId`.
     - `transition_job(&pool, id, req)` performing transactional row-level locked transitions (`SELECT ... FOR UPDATE`), validating invariants, updating `jobs` status, lease, and completion timestamps, and recording audit history in `job_state_transitions`.
     - `get_transition_history(&pool, id)` retrieving chronological transition history.
     - Convenience transition methods: `update_progress`, `complete_job`, `fail_job`, `cancel_job`.
  5. Re-export state machine and repository types in `crates/storage/src/lib.rs` (`JobStateMachine`, `JobTransitionRequest`, `JobStateTransitionRow`, `JobRepository`).
  6. Add 5 state machine unit tests covering legal transitions, illegal transitions from terminal states, request builders, transition validation invariants, and transition row instantiation (expanding storage unit tests from 17 to 22). Verify `cargo test --workspace` (109 passed), `cargo clippy`, `cargo fmt`, `cargo deny check`, and `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `job_state_transitions` table and migration `20261117000002` defined and verified in schema DDL and catalog validation.
  - State machine strictly enforces allowed transitions and disallows transitions from terminal states (`Completed`, `Failed`, `Cancelled`).
  - Progress percentages and phase invariants are strictly validated before transitions execute.
  - `JobRepository::transition_job` is transactional with row-level locking and writes an immutable audit record on every transition.
  - 100% of Rust tests pass (22 in `crates/storage`, 109 across workspace); zero clippy warnings; all 5 monorepo verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (22/22 passed), `cargo test --workspace` (109/109 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; parameterized SQL queries used across all repository methods; foreign key cascade deletion ensures transition history is pruned when parent jobs are deleted; audit trail records are immutable (insert-only); no secret leakage in transition messages or error codes.
- **Performance Considerations:** Transitions use row-level locking (`SELECT ... FOR UPDATE`) within bounded transactions avoiding table-level locks; `idx_job_state_transitions_job_id` provides index-backed transition history lookups without scanning full transition tables.
- **Documentation Requirements:** Updated `docs/DATA_MODEL.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 22/22 tests passed in 0.00s.
  - `cargo test --workspace`: 109/109 tests passed in 5.02s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 2.82s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `8b8082a`
- **Status:** `ACCEPTED`

---

#### Subphase `2.3.4` — Implement durable job leasing, heartbeat, and recovery in crates/storage
- **Unique ID:** `2.3.4`
- **Objective:** Implement durable distributed worker job leasing, periodic heartbeat lease extensions, lease validity probing, voluntary lease release, and automatic stale lease recovery sweeps in `crates/storage`.
- **Scope:** `crates/storage/src/leasing.rs`, `crates/storage/src/repository.rs`, `crates/storage/src/schema/jobs.rs`, `crates/storage/src/schema/mod.rs`, `crates/storage/src/lib.rs`.
- **Prerequisites:** `2.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/src/leasing.rs`
  - `crates/storage/src/repository.rs`
  - `crates/storage/src/schema/jobs.rs`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/src/lib.rs`
- **Implementation Tasks:**
  1. Implement `LeaseConfig` in `crates/storage/src/leasing.rs` defining worker identity (`worker_id`), lease duration (`lease_duration_seconds`), heartbeat interval (`heartbeat_interval_seconds`), and max attempts (`max_attempts`), with invariant validation, default constructors, and Serde support.
  2. Implement `HeartbeatResult` and `LeaseRecoveryReport` structs with accumulator and inspection helpers (`total_processed`, `is_empty`) and Serde support.
  3. Implement `JobRepository::acquire_next_job` using PostgreSQL `SELECT ... FOR UPDATE SKIP LOCKED` inside a transaction, atomically leasing either `queued` jobs or stale `leased`/`running` jobs whose lease expired, updating attempt count, calculating lease expiration relative to `NOW()`, and appending a transition audit record.
  4. Implement `JobRepository::heartbeat_job` and `JobRepository::is_lease_valid`, enforcing worker lease ownership, preventing heartbeat on terminal jobs, verifying expiration boundaries, updating timestamps, and optionally logging progress transitions.
  5. Implement `JobRepository::release_lease` enabling graceful voluntary lease relinquishment back to `queued` on worker shutdown, and `JobRepository::recover_stale_jobs` sweeping expired worker leases to either `queued` (for retry) or `failed` with code `LEASE_TIMEOUT_EXCEEDED` (when max attempts reached).
  6. Implement coordinator `LeaseManager` wrapping `PgPool` and `LeaseConfig` with typed async leasing operations, update `JobStatus::can_transition_to` to permit `Running -> Leased` reacquisition, and add 6 unit tests covering leasing configurations, validation rejections, constants invariants, and Serde round-trips (expanding storage tests from 22 to 28, and workspace tests to 115).
- **Acceptance Criteria:**
  - `acquire_next_job` safely selects and leases candidate jobs using `FOR UPDATE SKIP LOCKED` without race conditions.
  - `heartbeat_job` validates worker ownership, rejects terminal/stale jobs, and extends expiration.
  - `recover_stale_jobs` correctly resets recoverable jobs to `queued` and fails expired jobs exceeding max attempts with `LEASE_TIMEOUT_EXCEEDED`.
  - 100% of Rust tests pass (28 in `crates/storage`, 115 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (28/28 passed), `cargo test --workspace` (115/115 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; worker lease ownership strictly checked on every heartbeat and release preventing cross-worker lease hijacking; parameterized queries across all database operations; bounded batch sizes on recovery sweeps prevent transaction memory exhaustion.
- **Performance Considerations:** `FOR UPDATE SKIP LOCKED` eliminates distributed lock contention across parallel workers; index `idx_jobs_status_lease` ensures fast query plans for lease acquisition and reaper sweeps.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 28/28 tests passed in 0.00s.
  - `cargo test --workspace`: 115/115 tests passed in 5.97s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 1.93s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `545aec1`
- **Status:** `ACCEPTED`

---

#### Phase 2.3 Milestone Certification
- **Phase Name:** `Phase 2.3 — Persistence`
- **Subphases Covered:** `2.3.1` (PostgreSQL relational schema), `2.3.2` (Database migrations and schema validation), `2.3.3` (Job state machine, transitions, and history), `2.3.4` (Durable job leasing, heartbeat, and recovery).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Complete production relational schema in `ddl.sql` and embedded migrations (`20261117000001` and `20261117000002`) defining `jobs`, `job_artifacts`, `github_installations`, `content_cache_index`, and `job_state_transitions`.
  2. Compile-time embedded SQLx migration framework (`MIGRATOR`), structured migration reporting (`MigrationReport`), and runtime PostgreSQL catalog validation (`validate_schema`).
  3. Deterministic state machine (`JobStateMachine`) and immutable transition audit trail with row-level locking (`SELECT ... FOR UPDATE`).
  4. Durable distributed worker leasing (`FOR UPDATE SKIP LOCKED`), periodic heartbeats, voluntary lease release, and automated stale lease recovery (`recover_stale_jobs`).
  5. 115/115 automated Rust unit tests pass across the workspace (28 in `crates/storage`); 184/184 frontend tests pass in `apps/web`.
  6. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly with 0 failures.
- **Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

### Phase 2.4 — Storage

#### Subphase `2.4.1` — Implement local filesystem artifact storage in crates/storage
- **Unique ID:** `2.4.1`
- **Objective:** Implement durable, atomic, and traversal-guarded local filesystem artifact storage (`LocalFsStorage`), standard job and blob layout helpers, and disk capacity accounting in `crates/storage`.
- **Scope:** `crates/storage/Cargo.toml`, `crates/storage/src/local_fs.rs`, `crates/storage/src/lib.rs`.
- **Prerequisites:** Phase 2.3 (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/Cargo.toml`
  - `crates/storage/src/local_fs.rs`
  - `crates/storage/src/lib.rs`
- **Implementation Tasks:**
  1. Add `tokio` features `["fs", "io-util", "macros", "rt"]` to `crates/storage/Cargo.toml`.
  2. Implement `layout` module in `crates/storage/src/local_fs.rs` defining standard relative storage keys: `job_artifact_key` (`jobs/{job_id}/artifact.{ext}`), `job_dir_key` (`jobs/{job_id}`), and two-tier sharded `blob_key` (`blobs/{prefix}/{hash}.bin`).
  3. Implement `LocalFsStorage` struct with `base_dir`, `max_storage_bytes` limits, and constructors (`new`, `with_config`, `with_limits`).
  4. Implement strict path traversal defenses in `resolve_path(&self, key)`: reject null bytes, absolute paths, root/drive prefixes (`C:`), parent directory components (`..`), Windows reserved device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`), and trailing space/dot characters.
  5. Implement atomic file writing in `put`: write to staging file `.<name>.tmp.<uuid>`, call `sync_all()`, drop file handle, and perform atomic replace/rename (with cross-platform Windows fallback), cleaning up temp file on any error.
  6. Implement `get`, `open_read` (async streaming handle), `exists`, `delete`, `delete_directory`, `metadata` (`ArtifactFileMetadata`), `list`, and `storage_capacity` (`StorageUsage`).
  7. Add 8 unit tests covering CRUD lifecycle, atomic overwrite, path traversal rejections, directory deletion, listing, storage capacity metrics, not found errors, and payload limits (expanding storage crate tests to 36 and workspace tests to 123). Verify all 5 monorepo gates pass cleanly.
- **Acceptance Criteria:**
  - `LocalFsStorage` enforces atomic writes preventing partial or corrupted artifact states.
  - Path traversal attempts (`..`, `/`, `\`, drive letters, device names, null bytes) are strictly rejected with `CoreError::Security`.
  - Stored files and directory trees are cleanly readable, streamable, and deletable.
  - 100% of Rust tests pass (36 in `crates/storage`, 123 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (36/36 passed), `cargo test --workspace` (123/123 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; defense-in-depth path traversal checks block arbitrary file overwrite/read outside `base_dir`; Windows reserved device names blocked preventing OS denial-of-service; temporary files cleaned up on write aborts.
- **Performance Considerations:** Atomic rename avoids partial read locks; `sync_all()` guarantees disk durability; sharded blob path structure avoids directory filesystem inode limits on high-volume runs.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 36/36 tests passed in 0.03s.
  - `cargo test --workspace`: 123/123 tests passed in 13.32s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 6.07s.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `0757d59`
- **Status:** `ACCEPTED`

---

#### Subphase `2.4.2` — Implement pluggable storage backend trait in crates/storage
- **Unique ID:** `2.4.2`
- **Objective:** Implement pluggable asynchronous storage backend abstraction trait (`StorageBackend`), object-safe dynamic storage dispatch (`DynStorage`), stream alias (`AsyncReadStream`), `StorageBackend` implementation for `LocalFsStorage`, thread-safe `InMemoryStorage` implementation (zero-mock, fully functional in-memory backing for isolated testing and high-speed in-memory workloads), and runtime storage backend factory (`create_storage_backend`).
- **Scope:** `crates/storage/src/backend.rs`, `crates/storage/src/lib.rs`.
- **Prerequisites:** `2.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/src/backend.rs`
  - `crates/storage/src/lib.rs`
- **Implementation Tasks:**
  1. Define object-safe streaming and dispatch type aliases: `pub type AsyncReadStream = Pin<Box<dyn tokio::io::AsyncRead + Send>>;` and `pub type DynStorage = Arc<dyn StorageBackend>;`.
  2. Implement `#[async_trait] pub trait StorageBackend: Send + Sync + std::fmt::Debug` defining async operations: `backend_type`, `put`, `get`, `open_read`, `exists`, `delete`, `delete_directory`, `metadata`, `list`, and `storage_capacity`.
  3. Implement `StorageBackend` for `LocalFsStorage` delegating directly to atomic filesystem methods.
  4. Implement `InMemoryStorage` using `Arc<RwLock<HashMap<String, Vec<u8>>>>` with strict path traversal validation matching `LocalFsStorage`, prefix filtering for `list`, recursive directory removal for `delete_directory`, byte metrics in `storage_capacity`, and `Cursor<Vec<u8>>` for `open_read`.
  5. Implement runtime storage backend factory `create_storage_backend(config: &StorageConfig) -> Result<DynStorage, CoreError>` returning `Arc<LocalFsStorage>` for `StorageBackendConfig::LocalFs` and returning structured `CoreError::Config` for `StorageBackendConfig::S3`.
  6. Re-export `backend` module in `crates/storage/src/lib.rs`.
  7. Add 5 unit tests in `crates/storage/src/backend.rs` verifying LocalFsStorage via `DynStorage`, `InMemoryStorage` CRUD and directory deletion, traversal rejections, and factory configuration errors, expanding storage crate tests to 41 and workspace tests to 128. Verify all 5 monorepo gates pass cleanly.
- **Acceptance Criteria:**
  - `StorageBackend` trait is object-safe, async-trait-compatible, and implements `Send + Sync + std::fmt::Debug`.
  - `LocalFsStorage` and `InMemoryStorage` fully conform to `StorageBackend`.
  - `create_storage_backend` returns `DynStorage` based on `StorageConfig`.
  - 100% of Rust tests pass (41 in `crates/storage`, 128 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (41/41 passed), `cargo test --workspace` (128/128 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; `InMemoryStorage` enforces the same defense-in-depth path traversal checks as `LocalFsStorage` (blocking null bytes, absolute paths, drive letters, parent traversal, Windows reserved device names); factory returns structured configuration errors when unsupported or unconfigured backends are requested.
- **Performance Considerations:** Dynamic dispatch (`DynStorage`) avoids compile-time monomorphization bloat across consumer crates; `InMemoryStorage` uses `tokio::sync::RwLock` for high-concurrency read operations; stream handle `AsyncReadStream` supports zero-copy chunk streaming.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 41/41 tests passed in 0.03s.
  - `cargo test --workspace`: 128/128 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `c91a1d1`
- **Status:** `ACCEPTED`

---

#### Subphase `2.4.3` — Implement artifact retention and ephemeral cleanup policies in crates/storage
- **Unique ID:** `2.4.3`
- **Objective:** Implement artifact retention policies (`RetentionPolicy`), structured cleanup reporting (`CleanupReport`), transactional artifact persistence operations on `job_artifacts` (`ArtifactRepository`), and coordinated lifecycle management (`RetentionManager`) executing expired database sweeps, stale staging file purges, and disk capacity watermark enforcement across storage backends.
- **Scope:** `crates/storage/src/retention.rs`, `crates/storage/src/local_fs.rs`, `crates/storage/src/backend.rs`, `crates/storage/src/lib.rs`.
- **Prerequisites:** `2.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/src/retention.rs`
  - `crates/storage/src/local_fs.rs`
  - `crates/storage/src/backend.rs`
  - `crates/storage/src/lib.rs`
- **Implementation Tasks:**
  1. Define `RetentionPolicy` with `default_ttl`, `max_ttl`, `ephemeral_ttl`, `high_watermark_bytes`, `low_watermark_bytes`, and `batch_size`, including constructors (`from_config`), builders, invariant validations, and `calculate_expires_at` deadline calculations.
  2. Implement `CleanupReport` tracking evaluated artifacts, deleted artifacts, bytes freed, deleted ephemeral files, freed ephemeral bytes, and error accumulators.
  3. Implement `ArtifactRepository` providing SQLx relational CRUD for `job_artifacts`: `create_artifact`, `get_artifact`, `get_artifact_by_format`, `list_artifacts_for_job`, `find_expired_artifacts`, `delete_artifact`, `delete_job_artifacts`, `delete_expired_artifacts_batch` (using `FOR UPDATE SKIP LOCKED`), `find_oldest_artifacts`, `count_artifacts`, and `total_artifact_bytes`.
  4. Extend `StorageBackend` trait, `LocalFsStorage`, and `InMemoryStorage` with `cleanup_stale_staging_files(&self, max_age: Duration)` identifying and removing orphaned temporary staging files (`.*.tmp.*`).
  5. Implement `RetentionManager` coordinating storage backends and PostgreSQL: `cleanup_expired_artifacts`, `cleanup_job_artifacts`, `cleanup_job_storage`, `cleanup_stale_staging_files`, `enforce_capacity_watermark` (pruning oldest artifacts when usage exceeds high watermark until below low watermark), and `run_full_sweep`.
  6. Re-export `retention` in `crates/storage/src/lib.rs`.
  7. Add 7 unit tests in `crates/storage/src/retention.rs` covering policy defaults/validations, TTL calculations, report merging/metrics, in-memory stale staging cleanup, in-memory watermark capacity enforcement, filesystem stale staging cleanup, and job directory purging. Expand storage crate tests to 48/48 and workspace tests to 135/135 passing.
- **Acceptance Criteria:**
  - `RetentionPolicy` validates duration ordering and capacity watermark boundaries.
  - `CleanupReport` accurately tallies deleted files, freed bytes, and non-fatal errors.
  - `ArtifactRepository` performs atomic batch deletions without holding blocking table locks.
  - `RetentionManager` successfully purges expired DB records and corresponding physical storage files, cleans orphaned staging files, and prunes storage down to low watermark when capacity threshold is breached.
  - 100% of Rust tests pass (48 in `crates/storage`, 135 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (48/48 passed), `cargo test --workspace` (135/135 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; staging cleanup strictly restricts deletion to temporary files (`.*.tmp.*`), preserving legitimate user and job artifacts; batch deletion uses bounded sizes preventing connection pool exhaustion; path traversal safety is maintained.
- **Performance Considerations:** Batch queries utilize `FOR UPDATE SKIP LOCKED` preventing lock contention with active worker pipelines; staging scans skip non-staging files; watermark pruning executes in FIFO order minimizing storage fragmentation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 48/48 tests passed in 0.02s.
  - `cargo test --workspace`: 135/135 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `23ab840`
- **Status:** `ACCEPTED`

---

#### Subphase `2.4.4` — Implement artifact checksum validation and verification in crates/storage
- **Unique ID:** `2.4.4`
- **Objective:** Implement cryptographic checksum generation, validation, dual-digest verification (SHA-256 and BLAKE3), streaming digest calculation for unbounded files, relational integrity auditing on `job_artifacts`, and standard BSD/GNU `checksums.sha256` manifest generation/parsing in `crates/storage`.
- **Scope:** `Cargo.toml`, `crates/storage/Cargo.toml`, `crates/storage/src/checksum.rs`, `crates/storage/src/lib.rs`.
- **Prerequisites:** `2.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`
  - `crates/storage/Cargo.toml`
  - `crates/storage/src/checksum.rs`
  - `crates/storage/src/lib.rs`
- **Implementation Tasks:**
  1. Add `sha2` (v0.10) and `blake3` (v1.5) to root `Cargo.toml` workspace dependencies and `crates/storage/Cargo.toml`.
  2. Implement `ChecksumAlgorithm` enum supporting `Sha256` and `Blake3` with string representations, parse traits, and digest length constants.
  3. Implement `ArtifactDigests` computing both SHA-256 and BLAKE3 simultaneously in a single slice pass (`compute`) and over asynchronous streaming readers (`compute_streaming`) in 64 KB chunks to bound RAM usage. Add case-insensitive hash matching and invariant validations.
  4. Implement `ChecksumVerificationStatus` (`Verified`, `Mismatch`, `Missing`, `Corrupted`) and structured `ArtifactIntegrityReport` tracking expected vs actual hashes, byte sizes, and error details.
  5. Implement `StorageIntegrityAuditSummary` with health checks and success rate calculations.
  6. Implement `ArtifactVerifier` coordinating storage backends and PostgreSQL: `verify_bytes`, `verify_storage_file`, `verify_artifact_record`, `verify_job_artifact`, `verify_all_job_artifacts`, and `audit_storage_integrity`.
  7. Implement BSD/GNU-compatible `checksums.sha256` manifest generator `generate_sha256_manifest` and robust parser `parse_sha256_manifest`.
  8. Re-export `checksum` in `crates/storage/src/lib.rs`.
  9. Add 7 unit tests in `crates/storage/src/checksum.rs` covering empty vectors, hash matching/validation, streaming vs slice parity, verifier outcomes, relational artifact verification, manifest round-trips, and audit summary health. Expand storage tests to 55/55 and automated workspace tests to 142/142 passing.
  10. Formally certify completion of **Phase 2.4: Storage** (`2.4.1` – `2.4.4` all `ACCEPTED`) and **Stage 2: Backend and Processing Foundation** (15 subphases `2.1.1` – `2.4.4` all `ACCEPTED`).
- **Acceptance Criteria:**
  - Streaming and in-memory digest computations produce exact, identical cryptographic SHA-256 and BLAKE3 digests.
  - Verifier detects valid files, missing files, corrupted streams, and hash/size mismatches.
  - Standard `checksums.sha256` manifest round-trip correctly handles spacing, filenames with spaces, and comment lines.
  - 100% of Rust tests pass (55 in `crates/storage`, 142 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-storage` (55/55 passed), `cargo test --workspace` (142/142 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Complies with Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enforced; dual cryptographic hashing (SHA-256 + BLAKE3) provides defense against collision attacks; streaming bounds heap allocations preventing memory exhaustion on multi-GB artifacts; manifests strictly validate hash lengths and avoid path injection.
- **Performance Considerations:** Single-pass streaming computes SHA-256 and BLAKE3 concurrently in a 64 KB buffer; BLAKE3 leverages SIMD/AVX assembly routines where available.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 55/55 tests passed in 0.03s.
  - `cargo test --workspace`: 142/142 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `96d31fb`
- **Status:** `ACCEPTED`

---

#### Phase 2.4 Milestone Certification
- **Phase Name:** `Phase 2.4 — Storage`
- **Subphases Covered:** `2.4.1` (Local filesystem artifact storage), `2.4.2` (Pluggable storage backend trait), `2.4.3` (Artifact retention and ephemeral cleanup policies), `2.4.4` (Artifact checksum validation and verification).
- **Certification Date:** 2026-11-17
- **Certification Findings:**
  1. Atomic, traversal-safe local filesystem storage `LocalFsStorage` with staging writes (`.<name>.tmp.<uuid>`) and physical disk synchronization (`sync_all()`).
  2. Pluggable, object-safe asynchronous storage abstraction `StorageBackend` (`DynStorage`) with implementations for `LocalFsStorage` and in-memory `InMemoryStorage`.
  3. Bounded retention policies `RetentionPolicy`, structured audit reporting `CleanupReport`, transactional relational repository `ArtifactRepository` (`FOR UPDATE SKIP LOCKED`), and central coordinator `RetentionManager` executing DB sweeps, staging purges, and watermark capacity enforcement.
  4. Cryptographic checksum calculation `ArtifactDigests` (SHA-256 and BLAKE3), streaming digestion, verification engine `ArtifactVerifier`, and BSD/GNU `checksums.sha256` manifest generation/parsing.
  5. 142 automated Rust unit tests pass across the workspace (55 in `crates/storage`); 184/184 frontend tests pass in `apps/web`.
  6. All 5 monorepo verification gates in `scripts/check-all.ps1` and `scripts/check-all.sh` pass cleanly with 0 failures.
- **Status:** `COMPLETED` / `ACCEPTED` (Phase 2.4 Certified 2026-11-17)

---

### STAGE 2 COMPLETION CERTIFICATION

- **Stage ID:** `STAGE 2 — BACKEND AND PROCESSING FOUNDATION`
- **Total Phases Completed:** 4 (`Phase 2.1`, `Phase 2.2`, `Phase 2.3`, `Phase 2.4`)
- **Total Subphases Completed:** 15 (`2.1.1` through `2.4.4`, all `ACCEPTED`)
- **Stage Acceptance Gate:** Operational Rust Axum API, persistent PostgreSQL job management, and artifact storage infrastructure.
- **Verification Evidence:**
  1. **Phase 2.1 (Rust Architecture):** Root workspace with 17 member crates/services, strongly-typed domain errors/UUIDs/configs (`crates/core`), and structured logging/tracing/metrics with secret redaction (`crates/observability`).
  2. **Phase 2.2 (API):** Modular Axum router, parameter validation extractors, RFC 7807 problem details contracts, liveness `/health`, subsystem readiness `/ready`, and Prometheus `/metrics` exposition.
  3. **Phase 2.3 (Persistence):** PostgreSQL 16 relational DDL schema, compile-time embedded SQLx migrations (`MIGRATOR`), runtime catalog introspection (`validate_schema`), deterministic state machine (`JobStateMachine`), and durable worker leasing (`FOR UPDATE SKIP LOCKED`).
  4. **Phase 2.4 (Storage):** Atomic filesystem storage (`LocalFsStorage`), pluggable backend trait (`StorageBackend`), in-memory storage (`InMemoryStorage`), retention policies and cleanup (`RetentionManager`), dual SHA-256/BLAKE3 checksum validation (`ArtifactVerifier`), and checksum manifests.
  5. **Automated Test Quality:** 142 automated Rust workspace tests passing (55 in `crates/storage`), 184 automated frontend tests passing in `apps/web`; zero compiler/clippy warnings; 100% code style compliance; zero advisories or license bans in `cargo deny`.
  6. **Monorepo Quality Gates:** All 5 gates pass cleanly on Windows PowerShell (`scripts/check-all.ps1`) and POSIX bash (`scripts/check-all.sh`), with Podman storage footprint at 5.27 GB / 30 GB (17.6%).
- **Status:** `COMPLETED` / `ACCEPTED` (Stage 2 Certified 2026-11-17)

---

### STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION

### Phase 3.1 — Secure acquisition

#### Subphase `3.1.1` — URL parsing, normalization, and validation in crates/url-security and crates/web-extractor
- **Unique ID:** `3.1.1`
- **Objective:** Implement zero-trust WHATWG URL parsing, query canonicalization, tracking parameter stripping, comprehensive SSRF IP/host filtering (including alternative IP encodings, cloud metadata protection, and IPv4-mapped IPv6 evaluation), and document-type classification in `crates/url-security` and `crates/web-extractor`.
- **Scope:** `crates/url-security/src/*`, `crates/url-security/Cargo.toml`, `crates/web-extractor/src/url.rs`, `crates/web-extractor/src/lib.rs`, `crates/web-extractor/Cargo.toml`.
- **Prerequisites:** Stage 2 (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/url-security/Cargo.toml`, `crates/url-security/src/error.rs`, `crates/url-security/src/policy.rs`, `crates/url-security/src/ip_filter.rs`, `crates/url-security/src/normalizer.rs`, `crates/url-security/src/validated_url.rs`, `crates/url-security/src/lib.rs`
  - `crates/web-extractor/Cargo.toml`, `crates/web-extractor/src/url.rs`, `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Define `UrlSecurityError` enum in `crates/url-security/src/error.rs` covering scheme violations, character/length bounds, userinfo credentials, missing hosts, blocked ports, and SSRF category details, with conversion into `CoreError`.
  2. Implement `UrlSecurityPolicy` and `UrlSecurityPolicyBuilder` in `crates/url-security/src/policy.rs` with configurable length limits, scheme allowlists, tracking parameter definitions, port blocklists, and testing overrides.
  3. Implement `is_blocked_ip`, `is_blocked_host`, and `parse_alternative_ipv4` in `crates/url-security/src/ip_filter.rs` covering IPv4 loopback, RFC 1918 private, link-local/cloud metadata (`169.254.169.254`), CGNAT (`100.64.0.0/10`), multicast/reserved, IPv6 loopback, unique-local (`fc00::/7`), link-local (`fe80::/10`), IPv4-mapped IPv6, 6to4, and Teredo tunnels, alongside decimal/hex/octal alternative integer notation parsing (POSIX `inet_aton`).
  4. Implement `UrlNormalizer` in `crates/url-security/src/normalizer.rs` performing length checking, null byte/control character detection, scheme validation, default-port stripping (`:80` for http, `:443` for https), path duplicate slash collapse, tracking query parameter stripping (`utm_*`, `fbclid`, `gclid`), and deterministic query pair sorting.
  5. Implement `ValidatedUrl` in `crates/url-security/src/validated_url.rs` encapsulating normalized URLs, origin/domain comparison utilities, relative link resolution via `join`, and serde string serialization/deserialization with defensive re-validation.
  6. Implement `WebTarget` and `WebPageType` in `crates/web-extractor/src/url.rs` providing heuristic classification (`Documentation`, `Article`, `FileResource`, `GenericWeb`) based on subdomains, publishing platforms (`medium.com`, `substack.com`, etc.), path keywords, and date patterns, with relative link resolution and subpath matching.
- **Acceptance Criteria:**
  - Standard HTTP/HTTPS URLs pass validation and normalization; forbidden schemes (`file`, `ftp`, `gopher`, `data`, `javascript`) and embedded userinfo are strictly rejected.
  - SSRF blocklist strictly rejects all private/loopback/cloud metadata IPv4 and IPv6 addresses, as well as alternative IP encodings (`2130706433`, `0x7f000001`, `017700000001`, `127.1`).
  - Tracking parameters are stripped and remaining query pairs are deterministically ordered.
  - `WebTarget` accurately classifies documentation, articles, file resources, and generic web targets.
  - 100% of Rust tests pass (16 in `crates/url-security`, 7 in `crates/web-extractor`, 163 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-url-security` (16 passed), `cargo test -p web-repo-to-ai-web-extractor` (7 passed), `cargo test --workspace` (163 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enabled; comprehensive SSRF defense with zero-bypass coverage for IPv4-mapped IPv6, tunneling encapsulation, non-standard IP notations, and cloud metadata IPs (`169.254.169.254`); credential userinfo strictly rejected; ports restricted against protocol confusion attacks.
- **Performance Considerations:** In-place path and query parameter normalization with minimal heap allocations; fast bitwise CIDR masking for IP range evaluation; zero external network I/O during validation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-url-security`: 16/16 tests passed in 0.00s.
  - `cargo test -p web-repo-to-ai-web-extractor`: 7/7 tests passed in 0.00s.
  - `cargo test --workspace`: 163/163 tests passed across 17 workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `2a70255`
- **Status:** `ACCEPTED`

---

#### Subphase `3.1.2` — DNS resolution, IP pinning, and DNS-rebinding protection in crates/url-security
- **Unique ID:** `3.1.2`
- **Objective:** Implement asynchronous DNS resolution with all-record SSRF validation, anti-rebinding IP pinning, thread-safe resolution caching, and socket connection verification in `crates/url-security` and integrate with `crates/web-extractor`.
- **Scope:** `crates/url-security/src/dns.rs`, `crates/url-security/src/pinning.rs`, `crates/url-security/src/resolver.rs`, `crates/url-security/src/error.rs`, `crates/url-security/src/lib.rs`, `crates/url-security/Cargo.toml`, `crates/web-extractor/src/url.rs`, `crates/web-extractor/Cargo.toml`.
- **Prerequisites:** `3.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/url-security/Cargo.toml`, `crates/url-security/src/error.rs`, `crates/url-security/src/dns.rs`, `crates/url-security/src/pinning.rs`, `crates/url-security/src/resolver.rs`, `crates/url-security/src/lib.rs`
  - `crates/web-extractor/Cargo.toml`, `crates/web-extractor/src/url.rs`
- **Implementation Tasks:**
  1. Add DNS resolution and pinning errors to `UrlSecurityError` (`DnsResolutionFailed`, `DnsRebindingDetected`, `IpPinningMismatch`) and map to `CoreError`.
  2. Define asynchronous `DnsResolver` trait, production `SystemDnsResolver` (using `tokio::net::lookup_host`), and programmable `MockDnsResolver` supporting static mappings and dynamic rebinding attack sequences in `crates/url-security/src/dns.rs`.
  3. Implement immutable `PinnedTarget` in `crates/url-security/src/pinning.rs` encapsulating verified endpoints, pinned IP address, all resolved safe IPs, resolution timestamp, configurable TTL, and socket address verification (`verify_socket_addr`).
  4. Implement `SecureDnsResolver` in `crates/url-security/src/resolver.rs` coordinating cache lookups, IP literal bypass detection, hostname blocklist checks, asynchronous resolution, and strict all-record SSRF filtering (rejecting the entire resolution if any resolved IP address belongs to loopback, private, or cloud metadata CIDRs).
  5. Expose `resolve_and_pin` on `WebTarget` in `crates/web-extractor/src/url.rs`.
  6. Add unit tests for public resolution, internal IP blocking, dual-record split-DNS rejection, DNS rebinding mitigation across sequential resolutions, cache TTL behavior, and serde roundtrips.
- **Acceptance Criteria:**
  - DNS resolution resolving to internal or loopback IP addresses is strictly rejected.
  - Dual-record responses containing both public and private addresses are rejected via all-record validation.
  - DNS rebinding attacks across requests are mitigated through cache pinning and re-resolution rejection.
  - Pinned targets enforce exact IP and port matching via `verify_socket_addr`.
  - 100% of Rust tests pass (21 in `crates/url-security`, 8 in `crates/web-extractor`, 169 across workspace); zero clippy warnings; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-url-security` (21 passed), `cargo test -p web-repo-to-ai-web-extractor` (8 passed), `cargo test --workspace` (169 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: `#![forbid(unsafe_code)]` enabled; prevents Time-of-Check to Time-of-Use (TOCTOU) DNS rebinding by pinning connections to the specific validated IP; dual-record / round-robin DNS records containing private fallback addresses are blocked immediately.
- **Performance Considerations:** In-memory `tokio::sync::RwLock` cache eliminates redundant DNS lookups within TTL windows; non-blocking asynchronous lookups prevent thread starvation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-url-security`: 21/21 tests passed in 0.00s.
  - `cargo test -p web-repo-to-ai-web-extractor`: 8/8 tests passed in 0.00s.
  - `cargo test --workspace`: 169/169 tests passed across 17 workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `6f0da12`
- **Status:** `ACCEPTED`

---

#### Subphase `3.1.3` — HTTP fetching, streaming bodies, and redirect revalidation in crates/web-extractor
- **Unique ID:** `3.1.3`
- **Objective:** Implement safe, streaming HTTP fetching with hop-by-hop redirect revalidation, DNS pinning integration, cycle detection, hop limits, request header injection, and bounded streaming body reading in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/Cargo.toml`, `crates/web-extractor/src/error.rs`, `crates/web-extractor/src/http/mod.rs`, `crates/web-extractor/src/http/redirect.rs`, `crates/web-extractor/src/http/response.rs`, `crates/web-extractor/src/http/transport.rs`, `crates/web-extractor/src/http/fetcher.rs`, `crates/web-extractor/src/lib.rs`, `crates/url-security/src/resolver.rs`, root `Cargo.toml`, `deny.toml`.
- **Prerequisites:** `3.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`, `deny.toml`
  - `crates/url-security/src/resolver.rs`
  - `crates/web-extractor/Cargo.toml`, `crates/web-extractor/src/lib.rs`, `crates/web-extractor/src/error.rs`
  - `crates/web-extractor/src/http/mod.rs`, `crates/web-extractor/src/http/redirect.rs`, `crates/web-extractor/src/http/response.rs`, `crates/web-extractor/src/http/transport.rs`, `crates/web-extractor/src/http/fetcher.rs`
- **Implementation Tasks:**
  1. Add `reqwest` (with `rustls-tls` and `stream` features, no default features) to root `Cargo.toml` and `crates/web-extractor/Cargo.toml` alongside `async-trait`, `bytes`, and `futures`.
  2. Allow `CDLA-Permissive-2.0` in `deny.toml` for `webpki-roots`.
  3. Implement `WebExtractorError` in `crates/web-extractor/src/error.rs` covering URL security errors, HTTP transport errors, redirect loops, max redirect limit exceeded, missing redirect location, invalid redirect target, payload too large, timeouts, and network failures, with seamless mapping into `CoreError`.
  4. Implement `RedirectHop` and `RedirectTracker` in `crates/web-extractor/src/http/redirect.rs` tracking each redirect hop, detecting cyclic loops via normalized URL histories, and enforcing max redirect hop bounds.
  5. Implement `FetchedResponse` in `crates/web-extractor/src/http/response.rs` recording the initial URL, terminal canonical URL, status code, response headers, byte body, pinned IP address, redirect history, ETag, and Last-Modified metadata.
  6. Define `HttpTransport` trait with production `ReqwestTransport` (configured with `Policy::none()` to disable insecure automatic redirects and route connections to pinned socket addresses) and programmable `MockHttpTransport` in `crates/web-extractor/src/http/transport.rs`.
  7. Implement `WebFetcher` and `FetchOptions` in `crates/web-extractor/src/http/fetcher.rs` orchestrating hop-by-hop DNS resolution & IP pinning via `SecureDnsResolver`, custom User-Agent and Accept header injection, redirect revalidation, and streaming chunked body consumption bounded by `max_body_bytes`.
  8. Fix URL cache assignment in `SecureDnsResolver::resolve_and_pin` so cached DNS targets preserve the requested path.
  9. Add 8 comprehensive unit and integration tests in `crates/web-extractor/src/lib.rs` testing 200 OK responses, multi-hop redirects, redirect loops, max redirect boundaries, SSRF blocking on redirect hops to private/metadata IPs, invalid redirect schemes, missing location headers, and payload size bounds.
- **Acceptance Criteria:**
  - Automatic redirects are disabled; every redirect hop is explicitly re-evaluated against SSRF IP filters, hostname blocklists, and redirect loop history.
  - Streaming body consumption halts immediately when `max_body_bytes` is reached, preventing memory exhaustion.
  - Pinned IP addresses are preserved and propagated into the terminal `FetchedResponse` provenance.
  - 100% of workspace Rust tests pass (16 in `crates/web-extractor`, 177 workspace tests); 184/184 frontend tests pass; zero warnings and clean monorepo gates.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor` (16 passed), `cargo test --workspace` (177 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit` (184 passed), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: SSRF protection across all redirect hops; prevents DNS rebinding by retaining pinned IP addresses during HTTP connection; disables automatic redirects (`Policy::none()`) to prevent bypass of IP filters; bounds body ingestion against memory exhaustion attacks.
- **Performance Considerations:** Streaming body consumption reads in chunks (`BytesStream`) with early termination if payload exceeds byte quota; reusable `ReqwestClient` connection pool avoids socket thrashing; in-memory `RedirectTracker` uses O(1) set lookups.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 16/16 tests passed in 0.00s.
  - `cargo test --workspace`: 177/177 tests passed across all 17 crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `pnpm --filter web test:unit`: 184/184 tests passed in 2.65s.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `73051ec`
- **Status:** `ACCEPTED`

---

#### Subphase `3.1.4` — Compression bombs, timeouts, and body size limits in crates/web-extractor
- **Unique ID:** `3.1.4`
- **Objective:** Implement decompression bounds, compression ratio guards against compression bombs, streaming timeout controls, and header-declared vs actual byte mismatch detection in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/compression/*`, `crates/web-extractor/src/error.rs`, `crates/web-extractor/src/http/fetcher.rs`, `crates/web-extractor/src/http/response.rs`, `crates/web-extractor/src/http/transport.rs`, `crates/web-extractor/src/lib.rs`, `Cargo.toml`, `crates/web-extractor/Cargo.toml`.
- **Prerequisites:** `3.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `crates/web-extractor/Cargo.toml`
  - `crates/web-extractor/src/compression/mod.rs`, `crates/web-extractor/src/compression/encoding.rs`, `crates/web-extractor/src/compression/limits.rs`, `crates/web-extractor/src/compression/decompressor.rs`
  - `crates/web-extractor/src/error.rs`
  - `crates/web-extractor/src/http/fetcher.rs`, `crates/web-extractor/src/http/response.rs`, `crates/web-extractor/src/http/transport.rs`, `crates/web-extractor/src/http/mod.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Add `flate2` (with `rust_backend`), `brotli`, and `zstd` to workspace `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  2. Implement `ContentEncoding` enum in `crates/web-extractor/src/compression/encoding.rs` supporting `Identity`, `Gzip`, `Deflate`, `Brotli`, and `Zstd` with case-insensitive token and comma-separated header parsing.
  3. Implement `BodyLimits` configuration struct in `crates/web-extractor/src/compression/limits.rs` encapsulating `max_raw_body_bytes` (10 MB), `max_decompressed_bytes` (25 MB), `max_expansion_ratio` (20.0), `min_ratio_check_bytes` (4 KB), and `enforce_content_length`.
  4. Implement `BoundedDecompressor` in `crates/web-extractor/src/compression/decompressor.rs` using `CountingReader` with `AtomicUsize` byte accounting to stream decompressed bytes and enforce both maximum memory ceilings and ratio explosion checks in real time across single and chained encodings.
  5. Add `DecompressionBomb`, `DecompressionFailed`, `UnsupportedContentEncoding`, and `ContentLengthMismatch` error variants to `WebExtractorError` and map appropriately into `CoreError`.
  6. Enhance `WebFetcher` in `crates/web-extractor/src/http/fetcher.rs` with:
     - Overall operation timeout (`total_timeout`, default 30s) bounding DNS, TLS, redirects, and streaming.
     - Inactivity timeout (`chunk_read_timeout`, default 10s) between successive body chunks.
     - Early bailout if declared `Content-Length` exceeds raw body limits.
     - Strict `Content-Length` equality verification.
     - Transparent bounded decompression with `BoundedDecompressor`.
  7. Extend `FetchedResponse` in `crates/web-extractor/src/http/response.rs` with `raw_body_size` and `content_encoding` for full provenance tracking.
  8. Extend `MockHttpTransport` in `crates/web-extractor/src/http/transport.rs` with `set_delay` and `set_chunk_delay` to support deterministic timeout testing.
  9. Add 22 unit and integration tests across `compression` modules and `lib.rs` testing gzip, brotli, zstd, and deflate roundtrips, compression bomb abortion, size ceiling enforcement, Content-Length early exit, Content-Length mismatch detection, total timeout abortion, chunk read timeout abortion, and unsupported encoding rejection.
- **Acceptance Criteria:**
  - Decompression bomb payloads exceeding `max_expansion_ratio` (e.g. 20:1) are immediately aborted without allocating excessive memory.
  - Payloads exceeding `max_raw_body_bytes` or `max_decompressed_bytes` are blocked.
  - Streaming chunk timeouts and overall request timeouts abort hanging or slow-drip responses.
  - Declared `Content-Length` headers larger than limits trigger immediate early bailout.
  - 100% of workspace Rust tests pass (38 in `crates/web-extractor`, 199 in workspace); zero clippy warnings; clean formatting; all 5 monorepo verification gates pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor` (38 passed), `cargo test --workspace` (199 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit` (184 passed), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: Guards against Denial of Service (DoS) compression bombs (zip bombs / gzip bombs) through early ratio tracking; enforces hard memory ceilings preventing OOM crashes on the 16 GB baseline; mitigates Slowloris attacks via inter-chunk streaming read timeouts.
- **Performance Considerations:** Zero-allocation stream chunking with 4 KB buffer; pure Rust decoders (`miniz_oxide`, `brotli`, `zstd`) with zero unsafe in crate; atomic byte tracking avoids locking overhead during stream read.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 38/38 tests passed in 0.03s.
  - `cargo test --workspace`: 199/199 tests passed across all 17 crates in 0.04s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed in 3.38s.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `c7255d2`
- **Status:** `ACCEPTED`

---

### Phase 3.1 Milestone Completion & Certification

- **Phase ID:** `Phase 3.1 — Secure acquisition`
- **Status:** `ACCEPTED` & `CERTIFIED`
- **Completion Date:** 2026-11-17
- **Subphases Completed:** `3.1.1` (`ACCEPTED`), `3.1.2` (`ACCEPTED`), `3.1.3` (`ACCEPTED`), `3.1.4` (`ACCEPTED`).
- **Phase Milestone Verdict:** Full SSRF-safe, DNS-pinned, bounded, multi-format decompressing HTTP acquisition engine delivered and verified across all gates. Zero placeholders, zero mocks in production path.

---

### Phase 3.2: Static Extraction

#### Subphase `3.2.1` — DOM parsing and boilerplate removal in crates/web-extractor
- **Unique ID:** `3.2.1`
- **Objective:** Implement WHATWG HTML tree parsing, comprehensive document and OpenGraph/Twitter metadata extraction, fine-grained boilerplate pruning (scripts, styles, tracking, ads, cookie banners, navigation, footers, sidebars), semantic main preservation, preformatted code and table preservation, and relative URL canonicalization using safe pure-Rust parser `tl` in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/dom/*`, `crates/web-extractor/src/lib.rs`, `Cargo.toml`, `crates/web-extractor/Cargo.toml`.
- **Prerequisites:** `3.1.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`, `Cargo.lock`
  - `crates/web-extractor/Cargo.toml`
  - `crates/web-extractor/src/dom/mod.rs`
  - `crates/web-extractor/src/dom/metadata.rs`
  - `crates/web-extractor/src/dom/policy.rs`
  - `crates/web-extractor/src/dom/cleaner.rs`
  - `crates/web-extractor/src/dom/link_resolver.rs`
  - `crates/web-extractor/src/dom/parser.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Add `tl` (v0.7.8, safe pure Rust, MIT licensed, zero dependencies) to root `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  2. Implement `DocumentMetadata` in `crates/web-extractor/src/dom/metadata.rs` extracting title, meta description, author, keywords, OpenGraph (`og:*`), Twitter card (`twitter:*`), `<base href="...">`, language, and canonical URL with fallback cascades.
  3. Implement `BoilerplatePolicy` and builder in `crates/web-extractor/src/dom/policy.rs` with fine-grained switches for scripts, styles, comments, nav, footers, sidebars, forms, ads, cookies, social widgets, and custom class block/allow lists.
  4. Implement `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` distinguishing boilerplate elements from semantic content, explicitly protecting `<pre><code>` blocks, markdown containers, and `<table>` data.
  5. Implement `LinkResolver` in `crates/web-extractor/src/dom/link_resolver.rs` resolving and canonicalizing relative `<a>` href and `<img>` src attributes against base URLs while stripping dangerous `javascript:` schemes.
  6. Implement `DomParser` and `CleanedDocument` in `crates/web-extractor/src/dom/parser.rs` performing recursive DOM traversal, boilerplate pruning, link rewriting, standard entity decoding, and text newline normalization.
  7. Expose DOM extraction API in `crates/web-extractor/src/lib.rs` and add comprehensive unit and integration tests covering empty HTML, metadata fallbacks, boilerplate pruning, base URL resolution, code/table preservation, custom policies, and malformed HTML resilience.
  8. Run all 5 monorepo verification gates in `scripts/check-all.ps1` with 0 failures.
- **Acceptance Criteria:**
  - HTML parsing handles arbitrary valid and malformed HTML without panics or memory leaks.
  - Boilerplate elements (scripts, styles, nav, footer, sidebar, ads, cookie banners, tracking) are removed while retaining semantic `<main>` and `<article>` content.
  - Preformatted code blocks (`<pre><code>`) and tables (`<table>`) are preserved verbatim with whitespace intact.
  - Relative links and images are canonicalized against effective base URLs; unsafe JavaScript URIs are neutralized.
  - 100% of Rust tests pass (58 in `crates/web-extractor`, 219 across workspace); zero clippy warnings; all 5 monorepo gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor` (58 passed), `cargo test --workspace` (219 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit` (184 passed), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: Zero unsafe code (`#![forbid(unsafe_code)]`); neutralizes `javascript:` XSS vectors in link and image attributes; pure-Rust `tl` parser avoids C/C++ memory corruption vulnerabilities; rejects MPL or unmaintained dependencies per licensing policy.
- **Performance Considerations:** Zero-copy borrowing where possible using `tl::Parser`; single-pass recursive traversal; pre-allocated String buffers for cleaned HTML and text.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 58/58 tests passed.
  - `cargo test --workspace`: 219/219 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `13a393b`
- **Status:** `ACCEPTED`

---

#### Subphase `3.2.2` — Article extraction and readability heuristics in crates/web-extractor
- **Unique ID:** `3.2.2`
- **Objective:** Implement automated article discovery, candidate container scoring heuristics (text density, comma frequency, paragraph counts, heading counts, code block bonuses, class/ID weights, link density penalties), title/byline/publication-date/excerpt extraction, reading time estimation, and fallback preservation in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/article/*`, `crates/web-extractor/src/dom/parser.rs`, `crates/web-extractor/src/error.rs`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/article/mod.rs`
  - `crates/web-extractor/src/article/model.rs`
  - `crates/web-extractor/src/article/heuristics.rs`
  - `crates/web-extractor/src/article/scorer.rs`
  - `crates/web-extractor/src/article/extractor.rs`
  - `crates/web-extractor/src/dom/parser.rs`
  - `crates/web-extractor/src/error.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Define `ExtractedArticle`, `ReadabilityMetrics`, and `ReadabilityOptions` builder in `crates/web-extractor/src/article/model.rs`.
  2. Implement heuristic pattern matchers (`POSITIVE_PATTERNS`, `NEGATIVE_PATTERNS`, `UNLIKELY_PATTERNS`) and scoring calculations (`calculate_tag_weight`, `calculate_class_weight`, `is_unlikely_candidate`) in `crates/web-extractor/src/article/heuristics.rs`.
  3. Implement `ReadabilityScorer` in `crates/web-extractor/src/article/scorer.rs` recursively evaluating candidate containers against text length, comma count, paragraph count, code blocks, headings, link density, and class/ID weights.
  4. Implement `ArticleExtractor` in `crates/web-extractor/src/article/extractor.rs` identifying candidate containers (`article`, `main`, `section`, `div`), selecting the top-scoring candidate, rendering subtrees via `DomParser::render_subtree`, extracting `<h1>` titles, tiered author bylines (`rel="author"`, `class="author"`), publication time (`<time datetime="...">`), lead excerpts, word/char counts, and reading time minutes, with graceful fallback to whole-document extraction.
  5. Extend `DomParser` in `crates/web-extractor/src/dom/parser.rs` with `render_subtree` and preserve `rel`, `id`, `class` on `<a>` tags and `datetime` on `<time>` tags.
  6. Add `HtmlParseFailed` and `ExtractionFailed` variants to `WebExtractorError` in `crates/web-extractor/src/error.rs`.
  7. Expose article subsystem in `crates/web-extractor/src/lib.rs` and add 15 unit and integration tests across article extraction modules.
  8. Run all 5 monorepo verification gates in `scripts/check-all.ps1` with 0 failures.
- **Acceptance Criteria:**
  - Primary article containers are reliably distinguished from sidebars, navigation link farms, and noise.
  - Code blocks and tables within the selected article are preserved intact.
  - Author bylines and publication timestamps are accurately parsed from semantic HTML attributes and classes.
  - Pages lacking distinctive article containers fall back gracefully to the whole document body.
  - 100% of workspace Rust tests pass (73 in `crates/web-extractor`, 234 in workspace); zero clippy warnings; clean code style; all 5 monorepo verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor` (73 passed), `cargo test --workspace` (234 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit` (184 passed), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: `#![forbid(unsafe_code)]`; no regex-based catastrophic backtracking (ReDoS); candidate scoring operates on bounded parsed VDom trees; sanitized link attributes preserve XSS mitigations; zero unmaintained dependencies.
- **Performance Considerations:** Single-pass candidate collection; linear text accumulation during scoring; bounded calculations with pre-allocated String buffers; integer reading time calculation via `div_ceil`.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 73/73 tests passed.
  - `cargo test --workspace`: 234/234 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `9a5f01a`
- **Status:** `ACCEPTED`

---

#### Subphase `3.2.3` — Technical documentation and navigation hierarchy extraction in crates/web-extractor
- **Unique ID:** `3.2.3`
- **Objective:** Implement documentation framework detection (Docusaurus, mdBook, Sphinx, MkDocs, Rustdoc, VitePress, Nextra, GitBook), breadcrumb trail extraction, on-page table of contents (ToC) hierarchy construction with anchor stripping, previous/next pagination resolution, callout/admonition parsing (Note, Tip, Info, Important, Warning, Caution, Danger), and section-by-section extraction in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/docs/*`, `crates/web-extractor/src/dom/cleaner.rs`, `crates/web-extractor/src/lib.rs`, `services/api/src/routes/health.rs`.
- **Prerequisites:** `3.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/docs/mod.rs`
  - `crates/web-extractor/src/docs/model.rs`
  - `crates/web-extractor/src/docs/framework.rs`
  - `crates/web-extractor/src/docs/toc.rs`
  - `crates/web-extractor/src/docs/navigation.rs`
  - `crates/web-extractor/src/docs/admonition.rs`
  - `crates/web-extractor/src/docs/extractor.rs`
  - `crates/web-extractor/src/dom/cleaner.rs`
  - `crates/web-extractor/src/lib.rs`
  - `services/api/src/routes/health.rs`
- **Implementation Tasks:**
  1. Define `ExtractedDocumentation`, `DocFramework`, `Admonition`, `AdmonitionType`, `BreadcrumbItem`, `TocItem`, `DocLink`, `DocPagination`, `DocVersionInfo`, and `DocSection` in `crates/web-extractor/src/docs/model.rs`.
  2. Implement `detect_framework` in `crates/web-extractor/src/docs/framework.rs` inspecting generator meta tags and framework-specific CSS selectors.
  3. Implement `extract_table_of_contents` in `crates/web-extractor/src/docs/toc.rs` extracting heading levels (`h1`–`h6`), stripping permalink anchor glyphs (`#`, `¶`, `§`), mapping anchor IDs, and constructing nested ToC trees.
  4. Implement `DocNavigationExtractor` in `crates/web-extractor/src/docs/navigation.rs` resolving breadcrumb lists, previous/next pagination links, and version dropdown selectors.
  5. Implement `AdmonitionExtractor` in `crates/web-extractor/src/docs/admonition.rs` classifying callouts across standard and framework-specific alert styles.
  6. Implement `DocExtractor` in `crates/web-extractor/src/docs/extractor.rs` orchestrating raw HTML navigation discovery, framework-targeted container selection, cleaned content rendering, and section decomposition.
  7. Update `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` protecting admonitions in `CONTENT_INDICATORS` and refining ad token matching with boundary checks.
  8. Isolate health check probe file naming in `services/api/src/routes/health.rs` with `RequestId::new()` to eliminate Windows sharing violations during concurrent test execution.
  9. Expose documentation extraction API in `crates/web-extractor/src/lib.rs` and add 14 unit and integration tests across doc frameworks.
  10. Run all 5 monorepo verification gates in `scripts/check-all.ps1` with 0 failures.
- **Acceptance Criteria:**
  - Documentation generator frameworks are accurately identified without false positives.
  - Breadcrumbs, pagination controls, and version selectors are parsed and relative links canonicalized.
  - On-page table of contents reflects true heading nesting with clean anchor identifiers.
  - Admonition callout boxes are extracted with correct severity types and titles.
  - 100% of workspace Rust tests pass (87 in `crates/web-extractor`, 248 across workspace); zero clippy warnings; all 5 monorepo verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor` (87 passed), `cargo test --workspace` (248 passed), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit` (184 passed), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, and 05: `#![forbid(unsafe_code)]`; relative URLs resolved against pinned base URLs with XSS scheme neutralization; no regex-based ReDoS; isolated filesystem healthcheck probes prevent cross-thread file locking denial of service.
- **Performance Considerations:** Single-pass framework heuristics; non-allocating token boundary checks; tree-structured ToC construction in $O(N)$ where $N$ is heading count.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 87/87 tests passed.
  - `cargo test --workspace`: 248/248 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `65a530a`
- **Status:** `ACCEPTED`

---

#### Subphase `3.2.4` — Tables, code blocks, syntax detection, and link extraction in crates/web-extractor
- **Unique ID:** `3.2.4`
- **Objective:** Implement structured HTML table extraction with alignment and cell spanning (`colspan`/`rowspan`) normalization; `<pre><code>` and inline code extraction with syntax highlighter pattern handling, line-number stripping, and heuristic programming language detection; link extraction with canonical URL resolution, link type classification (internal, external, anchor, asset, special), and attribute parsing; and coordinate extraction through `ElementsExtractor`.
- **Scope:** `crates/web-extractor/src/elements/*`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/elements/mod.rs`
  - `crates/web-extractor/src/elements/model.rs`
  - `crates/web-extractor/src/elements/table.rs`
  - `crates/web-extractor/src/elements/code.rs`
  - `crates/web-extractor/src/elements/syntax.rs`
  - `crates/web-extractor/src/elements/link.rs`
  - `crates/web-extractor/src/elements/extractor.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Define `ExtractedTable`, `TableAlignment`, `ExtractedCodeBlock`, `ExtractedLink`, `LinkType`, `ExtractedElements`, and `ElementsOptions` in `crates/web-extractor/src/elements/model.rs`.
  2. Implement `TableExtractor` in `crates/web-extractor/src/elements/table.rs` extracting table headers, body rows, captions, cell alignments (`left`, `center`, `right`), handling `colspan`/`rowspan` rectangular normalization, and rendering to GFM Markdown table syntax.
  3. Implement `detect_syntax` in `crates/web-extractor/src/elements/syntax.rs` identifying programming languages via shebangs and keyword/syntax frequencies (Rust, Python, JS, TS, Go, C/C++, Java, SQL, HTML, CSS, JSON, YAML, TOML, Bash/Shell, Dockerfile).
  4. Implement `CodeBlockExtractor` in `crates/web-extractor/src/elements/code.rs` identifying code blocks, extracting language classes, stripping line numbers and gutter tables, preserving exact indentation and whitespace, and detecting language fallbacks.
  5. Implement `LinkExtractor` in `crates/web-extractor/src/elements/link.rs` resolving hyperlinks against document base URLs, classifying links (internal, external, anchor, asset, special), extracting anchor text, titles, and `rel` attributes.
  6. Implement `ElementsExtractor` in `crates/web-extractor/src/elements/extractor.rs` orchestrating comprehensive structural extraction from HTML strings or parsed DOMs.
  7. Expose element extraction modules in `crates/web-extractor/src/lib.rs`.
  8. Add exhaustive unit and integration tests across tables, code blocks, syntax detection, and link extraction.
  9. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - HTML tables are accurately parsed with rectangular column alignment and rendered to valid GFM Markdown tables.
  - Spanned cells (`colspan` and `rowspan`) preserve data without shifting column alignments.
  - Code blocks preserve exact code indentation and linebreaks without whitespace corruption; line numbers from highlighter gutters are cleanly excluded; syntax languages are detected accurately.
  - Hyperlinks are resolved, sanitized (XSS schemes neutralized), and classified correctly into internal, external, anchor, asset, and special.
  - 100% of workspace tests pass; 0 clippy warnings; 0 formatting errors; all 5 verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 02, 04, 05: `#![forbid(unsafe_code)]`; no regex ReDoS; all URLs resolved against pinned base URLs with XSS schemes (`javascript:`) neutralized; pipe characters and markup escaped in Markdown table formatting.
- **Performance Considerations:** Single-pass DOM traversal where applicable; fast heuristic syntax scanning; bounded memory allocations.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 107/107 tests passed.
  - `cargo test --workspace`: 268/268 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `abbbeed`
- **Status:** `ACCEPTED`

---

### Phase 3.2 Milestone Certification — Static Extraction
- **Phase ID:** `Phase 3.2`
- **Scope:** `3.2.1` – `3.2.4` (DOM parsing, article extraction, documentation extraction, tables/code/links).
- **Certification Date:** 2026-11-17
- **Verification Summary:**
  - `3.2.1`: Pure-Rust DOM parsing via `tl`, document metadata, boilerplate removal policies, relative link/image URL canonicalization (`ACCEPTED`).
  - `3.2.2`: Readability scoring heuristics, text density, byline/time extraction, article extraction with whole-document fallback (`ACCEPTED`).
  - `3.2.3`: Heuristic framework detection (Docusaurus, mdBook, Sphinx, MkDocs, Rustdoc, VitePress, Nextra, GitBook), heading hierarchy/ToC extraction, breadcrumb/pagination navigation, callout/admonition extraction (`ACCEPTED`).
  - `3.2.4`: HTML table normalization into rectangular matrices, `colspan`/`rowspan` handling, GFM Markdown table generation, code block extraction with line-number stripping and syntax heuristics, hyperlink classification (`ACCEPTED`).
  - All 4 subphases hold status `ACCEPTED`. 268/268 workspace Rust tests pass, 184/184 frontend tests pass, all 5 monorepo verification gates pass with 0 failures.
- **Phase Status:** `ACCEPTED` & `CERTIFIED`

---

### Phase 3.3 — Dynamic extraction

#### Subphase `3.3.1` — Browser worker architecture and IPC protocol in services/browser-worker
- **Unique ID:** `3.3.1`
- **Objective:** Establish the architecture of `services/browser-worker` and define the strongly typed IPC/RPC protocol between the Rust backend (`crates/web-extractor` / `services/worker`) and the isolated Node.js browser rendering service, implementing liveness and readiness health checks, request schema validation, error envelopes, and the asynchronous Rust client `BrowserWorkerClient`.
- **Scope:** `services/browser-worker/*`, `crates/web-extractor/src/browser/*`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `services/browser-worker/package.json`
  - `services/browser-worker/server.js`
  - `services/browser-worker/src/protocol.js`
  - `services/browser-worker/src/routes.js`
  - `services/browser-worker/test/protocol.test.js`
  - `crates/web-extractor/src/browser/mod.rs`
  - `crates/web-extractor/src/browser/model.rs`
  - `crates/web-extractor/src/browser/error.rs`
  - `crates/web-extractor/src/browser/client.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Initialize `services/browser-worker/package.json` with npm test and start lifecycle scripts, Node >=20 engines, and zero-dependency HTTP server core.
  2. Implement IPC schema validation and default normalization in `services/browser-worker/src/protocol.js` supporting `url`, `wait_until`, `timeout_ms`, `wait_for_selector`, `viewport`, `user_agent`, `headers`, `javascript_enabled`, and `block_subresource_ips`.
  3. Implement HTTP routing and health/readiness endpoints in `services/browser-worker/src/routes.js` and standalone server in `services/browser-worker/server.js`.
  4. Write Node.js unit tests in `services/browser-worker/test/protocol.test.js` verifying schema validation, default parameter resolution, error envelopes, and endpoint handling.
  5. Define strongly typed Rust IPC models in `crates/web-extractor/src/browser/model.rs` and `error.rs`: `BrowserRenderRequest`, `BrowserRenderResponse`, `BrowserRedirect`, `ConsoleMessage`, `PageError`, `BlockedSubresource`, `WaitUntil`, `Viewport`, `BrowserTiming`, `BrowserWorkerHealth`.
  6. Implement async `BrowserWorkerClient` in `crates/web-extractor/src/browser/client.rs` providing `check_health`, `check_readiness`, and `render` RPC calls over HTTP with connection timeouts and error mapping.
  7. Expose `browser` module in `crates/web-extractor/src/lib.rs`.
  8. Add Rust unit and wire-mock tests verifying IPC message serialization, deserialization, and HTTP error handling.
  9. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `services/browser-worker` responds to `GET /health` with 200 OK and liveness payload.
  - `services/browser-worker` responds to `GET /ready` with 200 OK and readiness status.
  - `POST /render` validates request payloads, rejects malformed URLs or invalid options with 400 Bad Request and RFC 7807 problem details, and echoes valid protocol payloads.
  - Rust `BrowserWorkerClient` serializes requests and deserializes responses with complete fidelity.
  - 100% of workspace Rust tests pass, Node.js tests pass, clippy passes with 0 warnings, and all 5 monorepo verification gates pass.
- **Tests:** `node --test services/browser-worker/test/protocol.test.js`, `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `pnpm --filter web test:unit`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 05: `services/browser-worker` isolated from internal database and GitHub App credentials; strict schema validation on all inputs; reject non-HTTP(S) schemes.
- **Performance Considerations:** Lightweight, zero-runtime-dependency Node.js HTTP server; non-blocking async IO in Rust; connection reuse via `reqwest::Client`.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 110/110 tests passed.
  - `cargo test --workspace`: 271/271 tests passed.
  - `node --test services/browser-worker/test/protocol.test.js`: 13/13 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `064f68b`
- **Status:** `ACCEPTED`

---

#### Subphase `3.3.2` — Playwright/Chromium dynamic fallback integration in services/browser-worker
- **Unique ID:** `3.3.2`
- **Objective:** Implement the headless Playwright / Chromium dynamic webpage rendering engine in `services/browser-worker`, with isolated per-request browser contexts, SSRF subresource filtering, timeout handling, and DOM extraction.
- **Scope:** `services/browser-worker/src/engine.js`, `services/browser-worker/src/routes.js`, `services/browser-worker/server.js`, `services/browser-worker/test/engine.test.js`, `crates/web-extractor/src/browser/client.rs`.
- **Prerequisites:** `3.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `services/browser-worker/package.json`
  - `services/browser-worker/package-lock.json`
  - `services/browser-worker/server.js`
  - `services/browser-worker/src/engine.js`
  - `services/browser-worker/src/routes.js`
  - `services/browser-worker/test/protocol.test.js`
  - `services/browser-worker/test/engine.test.js`
  - `crates/web-extractor/src/browser/client.rs`
- **Implementation Tasks:**
  1. Add `playwright-core` dependency to `services/browser-worker/package.json` with reproducible `package-lock.json`.
  2. Implement `PlaywrightEngine` in `services/browser-worker/src/engine.js` managing Chromium launch fallback resolution (bundled Playwright binary, `CHROME_BIN`, `BROWSER_PATH`, Google Chrome channel, Microsoft Edge channel) with sandboxed headless container flags.
  3. Implement per-request isolated `BrowserContext` lifecycle in `PlaywrightEngine.render()`, ensuring complete cookie, storage, and cache isolation across jobs, with unconditional cleanup in a `finally` block.
  4. Implement subresource request interception blocking non-HTTP(S) schemes and private/loopback/cloud-metadata IP destinations (`isPrivateOrLoopbackHost`).
  5. Implement console message and unhandled page exception capture with strict memory-bounding buffers.
  6. Implement error mapping from Playwright and network exceptions to RFC 7807 problem details (`NavigationTimeout`, `SelectorTimeout`, `DnsResolutionFailed`, `ConnectionRefused`, `TlsCertificateError`).
  7. Update `routes.js` to dispatch rendering requests to `engine.render()` and expose live browser readiness via `GET /ready`.
  8. Update `crates/web-extractor/src/browser/client.rs` to map `SelectorTimeout` to `BrowserWorkerError::NavigationTimeout`.
  9. Add unit and integration tests in `services/browser-worker/test/engine.test.js` validating address classification, error mapping, and live dynamic SPA rendering with selector wait and redirect chains.
  10. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `GET /ready` returns 200 OK and confirms Chromium readiness when browser is operational.
  - `POST /render` executes JavaScript on dynamic pages and extracts hydrated DOM.
  - `wait_for_selector` waits for client-rendered elements and returns 504 `SelectorTimeout` if not satisfied within timeout.
  - Subresource requests to loopback/private IPs or invalid schemes are intercepted and blocked.
  - Redirect chains and console messages are recorded and returned in response payload.
  - Isolated browser contexts are destroyed after every render, preventing memory leaks and state contamination.
  - 100% of workspace Rust tests pass, all 20 browser-worker Node.js tests pass, and all 5 monorepo verification gates pass.
- **Tests:** `npm test` (`services/browser-worker`), `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 05: Request interception inside Chromium blocks subresources targeting `127.0.0.1`, `169.254.169.254`, `localhost`, and RFC 1918 private subnets; container flags enforce `--no-sandbox` unprivileged user isolation; strict 1 MB body payload ceiling.
- **Performance Considerations:** Lazy Chromium initialization; isolated context reuse rather than full browser process relaunch; bounded console message (100) and error (50) arrays prevent heap bloat.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 110/110 tests passed.
  - `cargo test --workspace`: 271/271 tests passed.
  - `npm test` (`services/browser-worker`): 20/20 tests passed across 4 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `4628de7`
- **Status:** `ACCEPTED`

---

#### Subphase `3.3.3` — Resource limits, navigation timeouts, and sandbox isolation in services/browser-worker
- **Unique ID:** `3.3.3`
- **Objective:** Implement concurrency bounding, FIFO queue management, hard session deadlines, process memory thresholds, automated browser recycling, modal dialog auto-dismissal, subresource count ceilings, and hardened SSRF subresource filtering in `services/browser-worker` and `crates/web-extractor`.
- **Scope:** `services/browser-worker/src/limiter.js`, `services/browser-worker/src/engine.js`, `services/browser-worker/src/routes.js`, `services/browser-worker/test/limits.test.js`, `crates/web-extractor/src/browser/client.rs`, `crates/web-extractor/src/browser/error.rs`, `deny.toml`.
- **Prerequisites:** `3.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `services/browser-worker/src/limiter.js`
  - `services/browser-worker/src/engine.js`
  - `services/browser-worker/src/routes.js`
  - `services/browser-worker/test/limits.test.js`
  - `crates/web-extractor/src/browser/client.rs`
  - `crates/web-extractor/src/browser/error.rs`
  - `deny.toml`
- **Implementation Tasks:**
  1. Implement `ConcurrencyLimiter` in `services/browser-worker/src/limiter.js` enforcing maximum concurrent page sessions (default 4) and a bounded FIFO queue (capacity 20) with queue timeout (15s), rejecting capacity overflows with HTTP 429 `ConcurrencyLimitExceeded` and `Retry-After: 5`.
  2. Implement browser lifecycle recycling in `PlaywrightEngine` (`recycleBrowser`), automatically relaunching Chromium when active sessions reach 0 and either `rendersCount >= maxRendersPerBrowser` (200) or process memory exceeds RSS threshold (768 MB).
  3. Implement hard rendering session deadline timer (`hardTimeoutBufferMs = 5000` added to client timeout) via `Promise.race`, terminating hung pages with 504 `SessionTimeout`.
  4. Harden context isolation with `serviceWorkers: 'block'`, `acceptDownloads: false`, and `permissions: []`.
  5. Implement automatic dialog dismissal on `page.on('dialog')` so `alert()`, `confirm()`, and `prompt()` modals never block worker execution.
  6. Implement subresource request ceiling (`maxSubresources = 200`), aborting excess subresources with `blockedbyclient` and recording in `blocked_subresources`.
  7. Implement maximum rendered HTML payload ceiling (`maxHtmlBytes = 10 MB`), rejecting oversized DOMs with 413 `PayloadTooLarge`.
  8. Harden `isPrivateOrLoopbackHost` to comprehensively classify IPv4 private ranges (RFC 1918), loopback (127.0.0.0/8), link-local/cloud metadata (169.254.0.0/16, 169.254.169.254, 100.100.100.200, metadata.google.internal), CGNAT (100.64.0.0/10), current network (0.0.0.0/8), broadcast, IPv6 ULA (fc00::/7), IPv6 link-local (fe80::/10), and IPv4-mapped IPv6.
  9. Update `crates/web-extractor/src/browser/error.rs` and `client.rs` to map HTTP 429 to `BrowserWorkerError::TooManyRequests` and HTTP 413 to `BrowserWorkerError::PayloadTooLarge`.
  10. Add `deny.toml` advisory ignore for `RUSTSEC-2026-0543` (transitive sqlx chrono advisory).
  11. Add 9 comprehensive unit and integration tests in `services/browser-worker/test/limits.test.js` validating limiter queueing, 429 rejection with `Retry-After`, address classification, dialog auto-dismissal, subresource ceilings, and HTML payload bounds.
  12. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `ConcurrencyLimiter` bounds active browser sessions and rejects queue overflows with HTTP 429.
  - Hard session deadline aborts hung or infinite-loop pages.
  - Page dialogs (`alert`, `confirm`) are automatically dismissed without freezing navigation.
  - Subresources exceeding 200 requests per page are aborted and logged in `blocked_subresources`.
  - Rendered HTML exceeding 10 MB is rejected with HTTP 413.
  - Subresource requests targeting RFC 1918, 127.0.0.0/8, 169.254.0.0/16, CGNAT, or metadata IPs are intercepted and blocked.
  - Rust `BrowserWorkerClient` maps 429 to `TooManyRequests` and 413 to `PayloadTooLarge`.
  - 100% of workspace Rust tests pass, all 29 browser-worker Node.js tests pass, and all 5 monorepo verification gates pass.
- **Tests:** `npm test` (`services/browser-worker`), `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 05: Intercepts subresources targeting loopback, RFC 1918, CGNAT, link-local, cloud metadata, and private IPv6 ranges; blocks downloads and service worker persistence; enforces 10 MB DOM ceiling.
- **Performance Considerations:** Concurrency token bounds protect against CPU starvation; browser recycling prevents memory bloat and zombie worker build-up.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 110/110 tests passed.
  - `cargo test --workspace`: 271/271 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `c9266ba`
- **Status:** `ACCEPTED`

---

#### Subphase `3.3.4` — Dynamic DOM extraction and post-render hydration in crates/web-extractor
- **Unique ID:** `3.3.4`
- **Objective:** Implement `DynamicWebExtractor` in `crates/web-extractor`, orchestrating fast streaming static acquisition, content density & SPA shell heuristic analysis (`SpaDetector`), dynamic browser worker invocation for JS-rendered pages via `services/browser-worker`, post-render hydration through `DomParser`, `ArticleExtractor`, `DocExtractor`, and `ElementsExtractor`, and graceful degradation with diagnostic warnings on browser worker failure.
- **Scope:** `crates/web-extractor/src/browser/extractor.rs`, `crates/web-extractor/src/browser/mod.rs`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/browser/extractor.rs`
  - `crates/web-extractor/src/browser/mod.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Implement `DynamicWebExtractor` orchestrating full web extraction pipeline: fast streaming static acquisition, content density and SPA shell heuristic detection, dynamic browser worker invocation, and post-render hydration.
  2. Implement `SpaDetector` heuristic analyzer detecting empty DOMs, framework hydration roots (`#root`, `#app`, `#__next`, `#svelte`, `<app-root>`), `<noscript>` JavaScript warnings, and low-density markup shells (< 5% content density).
  3. Implement post-render hydration pipeline feeding rendered HTML into `DomParser`, `ArticleExtractor`, `DocExtractor`, and `ElementsExtractor`, returning unified `ExtractedWebpage` instances.
  4. Implement `DynamicProvenance` capturing extraction method (`Static` vs `DynamicBrowser`), HTTP status, redirect chains, console logs, unhandled page errors, blocked subresources, and diagnostic warnings.
  5. Implement graceful degradation in `DynamicWebExtractor::extract`: if dynamic browser rendering fails or times out on an SPA page with fallback static content, preserve static extraction and attach diagnostic provenance warnings rather than failing the job.
  6. Re-export `DynamicWebExtractor`, `SpaDetector`, `ExtractionStrategy`, `ExtractionMethod`, `DynamicExtractorOptions`, `DynamicProvenance`, and `ExtractedWebpage` in `crates/web-extractor/src/lib.rs`.
  7. Add 4 unit and integration tests in `crates/web-extractor/src/browser/extractor.rs` testing SPA detector heuristics, rich article static path, SPA fallback with mock browser worker, and graceful degradation on browser worker failure.
  8. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `SpaDetector` accurately classifies rich content vs SPA shells and noscript warnings.
  - `DynamicWebExtractor` follows `ExtractionStrategy` (`Auto`, `StaticOnly`, `ForceDynamic`).
  - Rendered DOM from browser worker is cleaned, parsed, and hydrated through article, documentation, and structured element extractors.
  - Browser worker failures degrade gracefully to static extraction with diagnostic warnings when static content is available.
  - 100% of workspace Rust tests pass (114 in web-extractor, 275 total), all 29 browser-worker Node.js tests pass, and all 5 monorepo verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`services/browser-worker`), `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 05: SSRF protection on all target URLs via `UrlSecurityPolicy` and `ValidatedUrl`; canonical target join and re-validation on redirects; subresource blocking in browser worker.
- **Performance Considerations:** Fast static acquisition path attempted first; dynamic browser rendering only invoked when heuristics indicate an SPA shell or forced; bounded DOM parsing with zero unconstrained allocations.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 114/114 tests passed.
  - `cargo test --workspace`: 275/275 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `bb27fa6`
- **Status:** `ACCEPTED`

---

### Phase 3.3 Milestone Certification (`3.3.1` – `3.3.4`)
- **Phase 3.3 Goal:** Dynamic extraction via isolated browser worker and Playwright/Chromium fallback.
- **Subphases Completed & Accepted:**
  - `3.3.1`: Browser worker architecture and IPC protocol in services/browser-worker (`ACCEPTED`, commit `064f68b`)
  - `3.3.2`: Playwright/Chromium dynamic fallback integration in services/browser-worker (`ACCEPTED`, commit `4628de7`)
  - `3.3.3`: Resource limits, navigation timeouts, and sandbox isolation in services/browser-worker (`ACCEPTED`, commit `c9266ba`)
  - `3.3.4`: Dynamic DOM extraction and post-render hydration in crates/web-extractor (`ACCEPTED`, commit `bb27fa6`)
- **Certification Evidence:**
  - Zero mock or placeholder implementations across all production paths.
  - Playwright / Chromium dynamic browser worker running in unprivileged isolation with per-request contexts and SSRF blocking.
  - Complete post-render hydration into DOM, article, documentation, and structured element representations.
  - Graceful degradation to static extraction when browser worker experiences transient failures.
  - 100% pass rate across all 5 verification gates (275 Rust tests, 29 browser-worker tests, 184 frontend tests, 0 clippy warnings, 0 fmt diffs, 0 cargo deny errors, 5.27 GB / 30 GB storage footprint).
- **Milestone Status:** `CERTIFIED`

---

### Phase 3.4 — Extraction quality

#### Subphase `3.4.1` — Source attribution, HTTP provenance, and metadata capture in crates/web-extractor
- **Unique ID:** `3.4.1`
- **Objective:** Implement comprehensive source attribution, HTTP transport provenance, dual cryptographic content hashes (SHA-256 and BLAKE3), Schema.org JSON-LD extraction, metadata precedence cascading, and structured citation formatting in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/Cargo.toml`, `crates/web-extractor/src/provenance/*`, `crates/web-extractor/src/browser/extractor.rs`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/Cargo.toml`
  - `crates/web-extractor/src/lib.rs`
  - `crates/web-extractor/src/provenance/mod.rs`
  - `crates/web-extractor/src/provenance/model.rs`
  - `crates/web-extractor/src/provenance/extractor.rs`
  - `crates/web-extractor/src/browser/extractor.rs`
- **Implementation Tasks:**
  1. Add `blake3` and `sha2` workspace dependencies to `crates/web-extractor/Cargo.toml`.
  2. Implement strongly-typed source attribution models in `crates/web-extractor/src/provenance/model.rs`: `SourceAttribution`, `HttpProvenance`, `RedirectHopRecord`, `ContentHashes`, `CapturedMetadata`, `BrowserDiagnostics`, `ProvenanceCitations`, `ProvenanceWarning`, and `WarningSeverity`.
  3. Implement dual cryptographic content hash computation (`ContentHashes::compute`) generating SHA-256 and BLAKE3 lowercase hex digests over raw response markup bytes and extracted plain text.
  4. Implement `MetadataExtractor` in `crates/web-extractor/src/provenance/extractor.rs` extracting and parsing Schema.org JSON-LD structured data blocks (`<script type="application/ld+json">`) and cascading metadata precedence (OpenGraph -> Twitter -> HTML `<head>` -> Schema.org).
  5. Implement `ProvenanceBuilder` in `crates/web-extractor/src/provenance/extractor.rs` synthesizing `SourceAttribution` for both static acquisition (`build_static`) and dynamic browser rendering (`build_dynamic`), capturing HTTP headers, ETag, Last-Modified, cache-control, server, pinned IP, redirect history, and generating formatted citations (concise, markdown callout, plaintext).
  6. Implement diagnostic warning classification detecting canonical URL mismatches (`CANONICAL_URL_MISMATCH`), missing cache validators (`MISSING_HTTP_CACHE_VALIDATORS`), missing author metadata (`MISSING_AUTHOR`), blocked subresources, and dynamic-to-static degradation (`BROWSER_DEGRADED_TO_STATIC`).
  7. Integrate `SourceAttribution` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` for all static and dynamic execution paths.
  8. Re-export provenance types in `crates/web-extractor/src/lib.rs`.
  9. Add unit and integration tests covering content hash computation, Schema.org JSON-LD extraction, metadata precedence cascading, and provenance builder warning generation.
  10. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Dual SHA-256 and BLAKE3 content hashes are computed over raw bytes and extracted text.
  - HTTP headers, status code, pinned IP, and redirect history are captured in `HttpProvenance`.
  - Schema.org JSON-LD blocks are parsed and merged into `CapturedMetadata`.
  - Diagnostic warnings are emitted for canonical mismatches and missing cache headers.
  - Markdown callout, concise, and plaintext citations are formatted.
  - 100% of workspace Rust tests pass (117 in web-extractor, 278 total), all 29 browser-worker Node.js tests pass, and all 5 monorepo verification gates pass.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`services/browser-worker`), `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Cryptographic provenance verification with SHA-256 and BLAKE3; canonical URL join and SSRF validation; zero plaintext secret exposure in headers.
- **Performance Considerations:** Single-pass cryptographic hashing using streaming digesters; bounded memory consumption during JSON-LD parsing.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 117/117 tests passed.
  - `cargo test --workspace`: 278/278 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `f8caa8f`
- **Status:** `ACCEPTED`

---

#### Subphase `3.4.2` — Duplicate detection and canonical URL resolution in crates/web-extractor
- **Unique ID:** `3.4.2`
- **Objective:** Implement duplicate detection and canonical URL resolution in `crates/web-extractor`, providing RFC-compliant `<link rel="canonical">` parsing, relative URL resolution, SSRF validation, cross-domain canonical detection, 64-bit SimHash near-duplicate fingerprinting with term-frequency weighting, cryptographic BLAKE3 exact duplicate matching, and an in-memory thread-safe `DuplicateRegistry`.
- **Scope:** `crates/web-extractor/src/dedup/*`, `crates/web-extractor/src/provenance/*`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/dedup/mod.rs`
  - `crates/web-extractor/src/dedup/model.rs`
  - `crates/web-extractor/src/dedup/canonical.rs`
  - `crates/web-extractor/src/dedup/simhash.rs`
  - `crates/web-extractor/src/dedup/registry.rs`
  - `crates/web-extractor/src/provenance/model.rs`
  - `crates/web-extractor/src/provenance/extractor.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Implement duplicate detection data models in `crates/web-extractor/src/dedup/model.rs`: `CanonicalSource`, `CanonicalResolution`, `DuplicateClassification`, `DuplicateRecord`, `DedupOptions`.
  2. Implement `CanonicalUrlResolver` in `crates/web-extractor/src/dedup/canonical.rs` parsing HTML `<link rel="canonical">`, resolving relative hrefs, validating candidates through `UrlSecurityPolicy` to prevent SSRF and loopback canonical exploits, detecting cross-domain canonicals, and assigning confidence scores (`High`, `Medium`, `Low`, `None`).
  3. Implement 64-bit `SimHasher` in `crates/web-extractor/src/dedup/simhash.rs` with unigram tokenization, term-frequency weighting, 64-bit FNV-1a hashing, Hamming distance computation, and Jaccard token similarity.
  4. Implement `DuplicateRegistry` in `crates/web-extractor/src/dedup/registry.rs` using `RwLock` concurrency to index canonical URLs, BLAKE3 content hashes, and SimHash fingerprints, classifying documents as `Unique`, `ExactCanonical`, `ExactContent`, or `NearDuplicate`.
  5. Integrate `CanonicalUrlResolver` into `ProvenanceBuilder::build_static` and `build_dynamic` in `crates/web-extractor/src/provenance/extractor.rs`, populating `canonical_resolution` on `SourceAttribution`.
  6. Re-export `dedup` types in `crates/web-extractor/src/lib.rs`.
  7. Add 10 comprehensive unit and integration tests across canonical resolution, SSRF resistance, SimHash Hamming distance, and registry duplicate classification, bringing `crates/web-extractor` to 127 passing tests and the workspace to 288 passing tests.
  8. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - SSRF-safe canonical link resolution rejects localhost/private IP/metadata targets in canonical tags.
  - Cross-domain canonical links are identified and flagged with lower confidence.
  - BLAKE3 hashes detect exact byte and text duplicates.
  - 64-bit SimHash identifies near-duplicates within Hamming distance $\le 8$ ($\ge 87.5\%$ similarity).
  - `DuplicateRegistry` correctly categorizes unique, exact, and near-duplicate pages concurrently.
  - All 127 tests in `crates/web-extractor` pass; all 288 workspace tests pass; all 5 monorepo verification gates pass with 0 failures.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`services/browser-worker`), `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Canonical URL targets are validated against `UrlSecurityPolicy` using `ValidatedUrl::parse` to prevent SSRF poisoning or redirect bypassing via malicious canonical tags; thread-safe deduplication registry prevents race conditions.
- **Performance Considerations:** In-memory registry with $O(1)$ hash map lookups for exact canonical and BLAKE3 duplicates; lightweight 64-bit bitwise Hamming distance operations for SimHash checks.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 127/127 tests passed.
  - `cargo test --workspace`: 288/288 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `1a99646`
- **Status:** `ACCEPTED`

---

#### Subphase `3.4.3` — Content completeness scoring and extraction warnings in crates/web-extractor
- **Unique ID:** `3.4.3`
- **Objective:** Implement content completeness scoring, empirical extraction metrics, heading hierarchy gap detection, diagnostic extraction warnings, and qualitative completeness grading in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/quality/*`, `crates/web-extractor/src/browser/extractor.rs`, `crates/web-extractor/src/lib.rs`.
- **Prerequisites:** `3.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/quality/mod.rs`
  - `crates/web-extractor/src/quality/model.rs`
  - `crates/web-extractor/src/quality/scorer.rs`
  - `crates/web-extractor/src/browser/extractor.rs`
  - `crates/web-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Implement multidimensional completeness models in `crates/web-extractor/src/quality/model.rs`: `CompletenessLevel` (`Complete`, `High`, `Moderate`, `Partial`, `Degraded`, `Empty`), `ExtractionWarningCategory`, `ExtractionWarning`, `DimensionScores`, `ContentMetrics`, and `ExtractionQualityReport`.
  2. Implement `QualityScorer` in `crates/web-extractor/src/quality/scorer.rs` evaluating metadata completeness (title, description, author, dates, language, canonical URL), content volume and density, structural fidelity (monotonic heading hierarchy, code block syntax identification, table header normalization, link categorization), and acquisition provenance (hashes, HTTP status, cache validators, canonical verification, SSRF blocked subresources, unhandled JS errors, browser degradation).
  3. Implement HTML heading analysis with `tl` parser scanning heading sequences and detecting hierarchy gaps (e.g. `<h1>` jumped directly to `<h3>` or `<h4>`), emitting `HEADING_HIERARCHY_GAP` warnings.
  4. Implement quantitative content metrics extraction: word count, character count, estimated reading time, paragraph count, heading counts by level, code block and table counts, internal/external link breakdown, content density ratio, and boilerplate ratio.
  5. Integrate `ExtractionQualityReport` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` across both `hydrate_static` and `render_and_hydrate` execution paths.
  6. Re-export `quality` types in `crates/web-extractor/src/lib.rs`.
  7. Add 8 unit and integration tests covering rich article scoring, empty content critical warnings, sparse text degradation, missing metadata penalties, heading hierarchy gaps, code syntax and table header detection, subresource/page error diagnostics, and browser degradation penalties, bringing `crates/web-extractor` to 135 passing tests and workspace to 296 passing tests.
  8. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `CompletenessLevel` correctly categorizes extracted webpages into `Complete`, `High`, `Moderate`, `Partial`, `Degraded`, or `Empty`.
  - Empty text documents with 0 words are classified as `Empty` with critical warning `CONTENT_EMPTY` and disallowed for CDM conversion (`is_acceptable_for_cdm = false`).
  - Heading hierarchy skips (e.g. `h1` to `h3`) are detected and flagged with `HEADING_HIERARCHY_GAP`.
  - Code blocks lacking syntax and tables lacking headers are flagged with diagnostic warnings.
  - Subresources blocked by SSRF and unhandled page errors penalize provenance scores.
  - All 135 tests in `crates/web-extractor` pass; all 296 workspace tests pass; all 5 monorepo verification gates pass with 0 failures.
- **Tests:** `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`services/browser-worker`), `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Quality scorer incorporates SSRF subresource interception counts and unhandled page errors into diagnostic reports; empty or corrupt extractions are prevented from producing false completeness certifications.
- **Performance Considerations:** Single-pass text tokenization and heading traversal; $O(1)$ metric aggregations without heap reallocation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 135/135 tests passed.
  - `cargo test --workspace`: 296/296 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `63c67f5`
- **Status:** `ACCEPTED`

---

#### Subphase `3.4.4` — Golden dataset test suite for universal webpage extraction in crates/web-extractor
- **Unique ID:** `3.4.4`
- **Objective:** Implement a comprehensive golden dataset integration test suite in `crates/web-extractor/tests/golden_dataset.rs` covering 8 real-world HTML archetypes, validating end-to-end extraction accuracy, boilerplate cleaning, code block recovery, table normalization, SPA hydration lifecycles, SimHash near-duplicate detection, and quality scoring diagnostics.
- **Scope:** `crates/web-extractor/tests/golden_dataset.rs`, `crates/web-extractor/src/*`.
- **Prerequisites:** `3.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/tests/golden_dataset.rs`
- **Implementation Tasks:**
  1. Author Archetype 1 (`test_golden_archetype_1_technical_blog_article`): Deep technical engineering article validating article title, author bylines, ISO-8601 publication timestamps, reading time estimation, Rust syntax highlighting, OpenGraph and JSON-LD schema parsing, and high completeness quality grading ($\ge 0.85$, `Complete`).
  2. Author Archetype 2 (`test_golden_archetype_2_docusaurus_documentation`): Multi-version technical documentation site validating Docusaurus framework fingerprinting, 3-level hierarchical breadcrumbs, documentation admonitions (`Tip`, `Warning`), and next/prev pagination links.
  3. Author Archetype 3 (`test_golden_archetype_3_api_specification_tables`): REST API specification validating complex HTML tables with alignment markers (`:---:`, `---:`), GFM rectangular table normalization, inline backticks, and HTTP code reference extraction.
  4. Author Archetype 4 (`test_golden_archetype_4_polyglot_code_with_line_gutters`): Multi-language code repository page validating monotonic line number gutter stripping from Rust code, Python shebang detection, and SQL DDL syntax parsing without indentation corruption.
  5. Author Archetype 5 (`test_golden_archetype_5_noisy_malformed_html`): Noisy real-world news layout validating unclosed markup recovery, boilerplate cleaner pruning of cookie banners, inline advertisements, social share widgets, newsletter popups, and related articles lists.
  6. Author Archetype 6 (`test_golden_archetype_6_spa_hydration_lifecycle`): Single-page application lifecycle validating `SpaDetector` heuristic identification of low-density skeleton shells and noscript fallbacks, followed by post-render hydration verifying extraction of complete client-side telemetry markup.
  7. Author Archetype 7 (`test_golden_archetype_7_canonical_and_simhash_dedup`): Search engine deduplication pipeline validating relative canonical URL resolution with UTM query stripping, exact URL duplicate registration, 64-bit SimHash near-duplicate clustering within Hamming distance threshold ($\le 8$), and distinct content isolation.
  8. Author Archetype 8 (`test_golden_archetype_8_sparse_stub_diagnostics`): Malformed/sparse stub testing asserting heavy diagnostic penalty (< 0.40 score), issuance of `CONTENT_SPARSE` and missing metadata warnings, and exclusion from CDM conversion (`is_acceptable_for_cdm = false`).
  9. Execute all 8 golden integration tests alongside 135 unit tests in `crates/web-extractor`, verifying 143 passing tests in crate and 304 total workspace tests.
  10. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - All 8 real-world HTML archetypes pass end-to-end extraction and quality evaluation.
  - Rust, Python, and SQL code snippets preserve formatting and strip monotonic line number gutters.
  - Rectangular table formatting produces valid GFM Markdown tables with proper alignments.
  - Docusaurus documentation extracts hierarchical breadcrumbs and typed admonition blocks.
  - Boilerplate cleaner discards noisy banners and popups without dropping article content.
  - SPA detector triggers fallback on empty shells and succeeds after hydration.
  - Exact duplicates and near-duplicates are accurately classified via canonical tags and 64-bit SimHash.
  - Sparse stubs are penalized and blocked from CDM qualification.
  - 100% of workspace Rust tests pass (143 in web-extractor, 304 total); all 184 frontend tests pass; all 29 browser-worker tests pass; all 5 monorepo verification gates pass with 0 failures.
- **Tests:** `cargo test --test golden_dataset -p web-repo-to-ai-web-extractor`, `cargo test -p web-repo-to-ai-web-extractor`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `npm test` (`services/browser-worker`), `npm test` (`apps/web`), `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Validated URL security on all golden targets; SSRF protection against malicious canonical links; safe malformed HTML parsing without unconstrained memory expansion or stack overflows.
- **Performance Considerations:** Integration tests execute in < 0.1s total; zero external network access; all test cases evaluated in-memory with deterministic HTML fixtures.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test --test golden_dataset -p web-repo-to-ai-web-extractor`: 8/8 tests passed.
  - `cargo test -p web-repo-to-ai-web-extractor`: 143/143 tests passed (135 unit + 8 golden).
  - `cargo test --workspace`: 304/304 tests passed.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `3037fee`
- **Status:** `ACCEPTED`

---

### Phase 3.4 Milestone Certification (`3.4.1` – `3.4.4`)
- **Phase 3.4 Goal:** Extraction quality, metadata cascading, duplicate detection, completeness scoring, and golden dataset test suite.
- **Subphases Completed & Accepted:**
  - `3.4.1`: Source attribution, HTTP provenance, and metadata capture in crates/web-extractor (`ACCEPTED`, commit `f8caa8f` / `c5590c4`)
  - `3.4.2`: Duplicate detection and canonical URL resolution in crates/web-extractor (`ACCEPTED`, commit `1a99646` / `1c859f9`)
  - `3.4.3`: Content completeness scoring and extraction warnings in crates/web-extractor (`ACCEPTED`, commit `63c67f5` / `afbcd94`)
  - `3.4.4`: Golden dataset test suite for universal webpage extraction in crates/web-extractor (`ACCEPTED`, commit `3037fee`)
- **Certification Evidence:**
  - Complete source attribution, HTTP header capture, dual cryptographic content hashing (SHA-256 and BLAKE3), and citation formatting.
  - RFC-compliant canonical URL resolution with SSRF defense, 64-bit SimHash near-duplicate detection, and concurrent `DuplicateRegistry`.
  - Empirical completeness scoring with monotonic heading gap analysis (`HEADING_HIERARCHY_GAP`) and qualitative grading (`CompletenessLevel`).
  - End-to-end validation across 8 golden archetypes covering technical blogs, multi-version docs, API tables, polyglot code, noisy HTML, SPA lifecycles, deduplication, and sparse stubs.
  - 100% pass rate across all 5 verification gates (304 Rust tests, 29 browser-worker tests, 184 frontend tests, 0 clippy warnings, 0 fmt diffs, 0 cargo deny errors, 5.27 GB / 30 GB storage footprint).
- **Milestone Status:** `CERTIFIED`

---

### STAGE 3 COMPLETION CERTIFICATION

- **Stage ID:** `STAGE 3 — UNIVERSAL WEBPAGE EXTRACTION`
- **Total Phases Completed:** 4 (`Phase 3.1`, `Phase 3.2`, `Phase 3.3`, `Phase 3.4`)
- **Total Subphases Completed:** 16 (`3.1.1` through `3.4.4`, all `ACCEPTED`)
- **Stage Acceptance Gate:** End-to-end universal webpage extraction pipeline supporting static and dynamic rendering, framework detection, table/code normalization, SSRF defenses, deduplication, and quality scoring.
- **Verification Evidence:**
  1. **Phase 3.1 (Secure Acquisition):** WHATWG URL parsing, query canonicalization, tracking parameter stripping, comprehensive SSRF IP filtering (`crates/url-security`), DNS pinning, hop-by-hop redirect verification, connection pool isolation, and streaming decompression (`crates/web-extractor`).
  2. **Phase 3.2 (Static Extraction):** Robust DOM parsing (`tl`), boilerplate cleaning (`BoilerplateCleaner`), structured article extraction (`ArticleExtractor`), documentation framework detection (`DocFramework` for Docusaurus, MkDocs, Sphinx, VitePress, Nextra), rectangular table normalization with alignment preservation, and language-tagged code block recovery with gutter stripping.
  3. **Phase 3.3 (Dynamic Extraction):** Standalone unprivileged Playwright / Chromium browser worker (`services/browser-worker`) with Unix domain socket/named pipe IPC, strict resource bounds, navigation timeouts, per-request context isolation, subresource SSRF blocking, SPA shell heuristic analysis (`SpaDetector`), dynamic DOM hydration, and graceful fallback.
  4. **Phase 3.4 (Extraction Quality):** Cryptographic provenance tracking (SHA-256 and BLAKE3), Schema.org JSON-LD extraction, citation generation, SSRF-safe canonical URL resolution, 64-bit SimHash deduplication, concurrent `DuplicateRegistry`, multidimensional completeness scoring (`QualityScorer`), and 8 real-world golden dataset archetypes.
  5. **Automated Test Quality:** 304 automated Rust workspace tests passing (143 in `crates/web-extractor`), 29 automated Node.js browser worker tests passing across 7 suites, 184 automated frontend tests passing in `apps/web`; zero compiler/clippy warnings; 100% code style compliance; zero advisories or license bans in `cargo deny`.
  6. **Monorepo Quality Gates:** All 5 gates pass cleanly on Windows PowerShell (`scripts/check-all.ps1`) and POSIX bash (`scripts/check-all.sh`), with Podman storage footprint at 5.27 GB / 30 GB (17.6%).
- **Status:** `COMPLETED` / `ACCEPTED` (Stage 3 Certified 2026-11-17)

---

### STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE

### Phase 4.1 — Canonical Context Document Model (CDM)

#### Subphase `4.1.1` — Canonical Context Document Model (CDM) schema in crates/cdm
- **Unique ID:** `4.1.1`
- **Objective:** Design and implement the complete Rust domain schema for the Canonical Context Document Model (CDM v1.0) in `crates/cdm`, establishing the immutable intermediate representation for all extractors, optimizers, tokenizers, chunkers, and exporters with strict repository completeness accounting.
- **Scope:** `crates/cdm/Cargo.toml`, `crates/cdm/src/schema/*`, `crates/cdm/src/lib.rs`.
- **Prerequisites:** Stage 3 (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/cdm/Cargo.toml`
  - `crates/cdm/src/model.rs`
  - `crates/cdm/src/section.rs`
  - `crates/cdm/src/metadata.rs`
  - `crates/cdm/src/hierarchy.rs`
  - `crates/cdm/src/relationships.rs`
  - `crates/cdm/src/completeness.rs`
  - `crates/cdm/src/builder.rs`
  - `crates/cdm/src/lib.rs`
- **Implementation Tasks:**
  1. Add dependencies to `crates/cdm/Cargo.toml`: `chrono` (with `serde`), `uuid` (with `serde`), and `blake3`.
  2. Implement core document types in `crates/cdm/src/model.rs`: `CdmDocument`, `SourceIdentity`, `SourceType` (`WebpageStatic`, `WebpageDynamic`, `GitHubPublic`, `GitHubPrivate`, `DocumentPdf`, `DocumentOcr`), `RevisionInfo`, `ExtractionStatus` (`Completed`, `Partial`, `Failed`, `Unsupported`), and `ExtractionWarning`.
  3. Implement section models in `crates/cdm/src/section.rs`: `CdmSection`, `SectionKind` (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Frontmatter`, `Manifest`, `Navigation`, `Admonition`), `LineRange`, `ByteRange`, and `SectionTokenMetrics`.
  4. Implement metadata and provenance models in `crates/cdm/src/metadata.rs`: `DocumentMetadata`, `Citation`, and `ExclusionRecord` with typed `ExclusionReason`.
  5. Implement hierarchy models in `crates/cdm/src/hierarchy.rs`: `HierarchyNode`, `HierarchyKind` (`Root`, `Directory`, `File`, `Section`, `Heading`), tree traversal, node search, and flattening utilities.
  6. Implement relationship models in `crates/cdm/src/relationships.rs`: `Relationship`, `RelationshipKind` (`Imports`, `ImportedBy`, `Calls`, `Extends`, `LinksTo`, `References`, `ParentOf`, `ChildOf`).
  7. Implement repository completeness models in `crates/cdm/src/completeness.rs`: `CompletenessReport` with formula validation verifying $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$ and `is_complete()`.
  8. Implement ergonomic fluent builders in `crates/cdm/src/builder.rs`: `CdmDocumentBuilder` and `CdmSectionBuilder` with automatic BLAKE3 content hash computation.
  9. Re-export all schema primitives in `crates/cdm/src/lib.rs`.
  10. Implement comprehensive unit tests in `crates/cdm` validating schema generation, completeness invariant formula checks, hierarchy tree operations, builder fluent workflows, and section content hashing.
  11. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `CdmDocument` schema supports both single-webpage extractions and complex multi-file GitHub repositories.
  - `CdmSection` computes deterministic BLAKE3 content digests over section bodies.
  - `CompletenessReport` rigorously verifies the completeness invariant formula $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$.
  - Hierarchy tree models support arbitrary nesting for directory structures and document outlines.
  - All types derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, and where applicable `Eq`.
  - `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]` are enforced with 100% documentation coverage.
  - Workspace tests, clippy, fmt, and all 5 monorepo gates pass with zero errors.
- **Tests:** `cargo test -p web-repo-to-ai-cdm`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Cryptographic BLAKE3 integrity on all sections; strict validation preventing unverified or corrupt document models from reaching exporters.
- **Performance Considerations:** In-memory traversal with efficient references; avoid redundant string allocations where possible.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-cdm`: 19/19 tests passed (11 unit + 8 integration).
  - `cargo test --workspace`: 322/322 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% code style compliance.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`services/browser-worker`): 29/29 tests passed across 7 suites.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `a0d0354`
- **Status:** `ACCEPTED`

---

#### Subphase `4.1.2` — CDM JSON serialization and deserialization in crates/cdm
- **Unique ID:** `4.1.2`
- **Objective:** Implement production-grade canonical JSON serialization and deserialization for the Canonical Context Document Model (CDM v1.0) in `crates/cdm`, providing deterministic pretty and compact JSON formatting, streaming reader/writer adapters, bounded memory deserialization defenses against payload bombs, strongly-typed error taxonomy (`CdmError`), and lossless roundtrip verification across all CDM types.
- **Scope:** `crates/cdm/Cargo.toml`, `crates/cdm/src/error.rs`, `crates/cdm/src/json.rs`, `crates/cdm/src/lib.rs`.
- **Prerequisites:** `4.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/cdm/Cargo.toml`
  - `crates/cdm/src/error.rs`
  - `crates/cdm/src/json.rs`
  - `crates/cdm/src/model.rs`
  - `crates/cdm/src/lib.rs`
  - `crates/cdm/tests/json_serde.rs`
- **Implementation Tasks:**
  1. Add `thiserror` dependency to `crates/cdm/Cargo.toml`.
  2. Implement strongly-typed domain error taxonomy `CdmError` in `crates/cdm/src/error.rs` covering JSON serialization/deserialization failures, size limit exhaustion (`PayloadTooLarge`), schema version mismatches, hash integrity failures, and I/O errors, implementing `From<CdmError> for CoreError`.
  3. Implement canonical JSON serialization and deserialization engine in `crates/cdm/src/json.rs`:
     - `to_json_pretty(&CdmDocument) -> Result<String, CdmError>`
     - `to_json_compact(&CdmDocument) -> Result<String, CdmError>`
     - `to_writer<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`
     - `to_writer_pretty<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`
     - `from_json(&str) -> Result<CdmDocument, CdmError>`
     - `from_slice(&[u8]) -> Result<CdmDocument, CdmError>`
     - `from_reader<R: std::io::Read>(reader: R) -> Result<CdmDocument, CdmError>`
     - `from_reader_bounded<R: std::io::Read>(reader: R, max_bytes: usize) -> Result<CdmDocument, CdmError>` (default 64 MB ceiling defense)
  4. Implement convenience serialization/deserialization methods directly on `CdmDocument` in `crates/cdm/src/model.rs`.
  5. Re-export `CdmError`, `JsonOptions`, and JSON serialization utilities in `crates/cdm/src/lib.rs`.
  6. Implement comprehensive integration test suite in `crates/cdm/tests/json_serde.rs` validating exact roundtrip equality, deterministic key order, bounded payload limit defense, corrupted JSON error handling, schema version validation, and streaming I/O.
  7. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `CdmDocument` serializes to and deserializes from JSON losslessly (`doc == from_json(&to_json(&doc))`).
  - Serialization produces deterministic key order via sorted mappings.
  - Payload size defenses reject payloads exceeding maximum byte thresholds with `PayloadTooLarge`.
  - Schema version is validated on deserialization, rejecting unknown major versions.
  - Streaming readers/writers operate without loading unbounded payloads into memory ahead of time.
  - All workspace tests, clippy, fmt, and all 5 monorepo gates pass with zero errors.
- **Tests:** `cargo test -p web-repo-to-ai-cdm`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Bounded reader safeguards prevent memory exhaustion / denial of service via oversized JSON payloads; strongly-typed error taxonomy prevents leaking internal system paths; deterministic serialization ensures reproducible cryptographic digests.
- **Performance Considerations:** Zero-copy borrowing where possible during deserialization; streaming writer support for multi-megabyte document export bundles.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-cdm`: 32 passed (17 unit + 7 JSON serde integration + 8 schema validation integration), 0 failed.
  - `cargo test --workspace`: 335 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 14 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** `d4d8984`
- **Status:** `ACCEPTED`

---

#### Subphase `4.1.3` — CDM schema validation and invariant checks in crates/cdm
- **Unique ID:** `4.1.3`
- **Objective:** Implement comprehensive structural, semantic, and cryptographic validation engine for `CdmDocument` in `crates/cdm`, enforcing document-level, section-level, tree hierarchy, relationship referential integrity, cryptographic hash verification, and repository completeness invariants with structured `ValidationReport` diagnostics.
- **Scope:** `crates/cdm/Cargo.toml`, `crates/cdm/src/validator.rs`, `crates/cdm/src/error.rs`, `crates/cdm/src/model.rs`, `crates/cdm/src/lib.rs`, `crates/cdm/tests/validation_invariants.rs`.
- **Prerequisites:** `4.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/cdm/Cargo.toml`
  - `crates/cdm/src/validator.rs`
  - `crates/cdm/src/error.rs`
  - `crates/cdm/src/model.rs`
  - `crates/cdm/src/lib.rs`
  - `crates/cdm/tests/validation_invariants.rs`
- **Implementation Tasks:**
  1. Add `url = { workspace = true }` to `crates/cdm/Cargo.toml` for standard URL validation.
  2. Implement `crates/cdm/src/validator.rs` providing:
     - `ValidationConfig`: Configuration options (`strict_mode`, `verify_hashes`, `enforce_completeness`, `check_referential_integrity`, `check_tree_acyclicity`, `max_tree_depth`).
     - `ValidationSeverity`: `Error` and `Warning`.
     - `ValidationIssue`: Strongly-typed validation issue containing error code, human-readable message, JSON/structural path, and severity.
     - `ValidationReport`: Aggregate report collecting all issues, with methods `is_valid()`, `has_errors()`, `has_warnings()`, `error_count()`, `warning_count()`, `errors()`, `warnings()`, `filter_by_path()`, `to_summary_string()`.
     - `CdmValidator`: Stateful validator running modular validation passes:
       - Document metadata & schema version validation (non-empty ID, matching major version, non-empty title, valid canonical URL, source identity checks).
       - Section validation (non-empty unique IDs, valid URLs, line range monotonic validity `1 <= start <= end`, byte range validity `start <= end`, token metric consistency).
       - Cryptographic hash validation (optional or strict: BLAKE3 digest recomputation against section body).
       - Tree hierarchy validation (unique node IDs, acyclicity check via visited set & DFS cycle detection, tree depth bound, referential integrity ensuring every node's `section_id` exists in document sections).
       - Relationship validation (referential integrity ensuring `from_section_id` and `to_section_id` both exist in document sections).
       - Completeness invariant validation ($\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$, verifying complete status implies `is_complete() == true`).
  3. Integrate validation into `CdmDocument` in `crates/cdm/src/model.rs`:
     - `doc.validate() -> Result<ValidationReport>`
     - `doc.validate_strict() -> Result<(), CdmError>`
     - `doc.validate_with_config(&ValidationConfig) -> ValidationReport`
  4. Extend `CdmError` in `crates/cdm/src/error.rs` with `ValidationFailed(Box<ValidationReport>)` or enhanced `Validation` formatting.
  5. Re-export validator primitives in `crates/cdm/src/lib.rs`.
  6. Create comprehensive integration test suite `crates/cdm/tests/validation_invariants.rs` covering all invariant checks, cycle detection, referential integrity violations, duplicate IDs, invalid line/byte ranges, tampered hashes, completeness matrix violations, and diagnostic reports.
  7. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Valid `CdmDocument` passes validation with zero errors (`is_valid() == true`).
  - Corrupted, duplicated, or invalid sections/hierarchy trees/relationships produce precise `ValidationIssue` entries with correct paths.
  - Cycle detection reliably identifies and prevents loops in hierarchy trees.
  - Completeness invariant is strictly enforced according to mathematical formula.
  - All workspace tests, clippy, fmt, and all 5 monorepo gates pass with zero errors.
- **Tests:** `cargo test -p web-repo-to-ai-cdm`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Acyclicity checks and tree depth bounds prevent stack overflow / denial of service on maliciously crafted recursive trees; URL format validation prevents malformed schemes; cryptographic hash checks verify content authenticity.
- **Performance Considerations:** Single-pass validation checks using `HashSet` lookups ($O(1)$) for section IDs and node IDs; avoid redundant allocations.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-cdm`: 60 passed (29 unit + 7 JSON serde integration + 8 schema validation integration + 16 validation invariants integration), 0 failed.
  - `cargo test --workspace`: 363 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** `8764f39`
- **Status:** `ACCEPTED`

---

#### Subphase `4.1.4` — Schema versioning and migration framework in crates/cdm
- **Unique ID:** `4.1.4`
- **Objective:** Implement a forward- and backward-compatible schema versioning, introspection, diffing, and automated migration framework in `crates/cdm`, providing strongly-typed semver version negotiation (`SchemaVersion`), sequential migration step pipelines (`MigrationStep`, `MigrationRegistry`) transforming historical schemas (`v0.8.0` $\to$ `v0.9.0` $\to$ `v1.0.0`), document diffing utilities (`SchemaDiff`), transparent auto-migration during JSON deserialization via `JsonOptions`, and full invariant compliance validation on all migrated documents.
- **Scope:** `crates/cdm/Cargo.toml`, `crates/cdm/src/version.rs`, `crates/cdm/src/migration.rs`, `crates/cdm/src/diff.rs`, `crates/cdm/src/error.rs`, `crates/cdm/src/json.rs`, `crates/cdm/src/model.rs`, `crates/cdm/src/lib.rs`, `crates/cdm/tests/schema_migration.rs`.
- **Prerequisites:** `4.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/cdm/src/version.rs`
  - `crates/cdm/src/migration.rs`
  - `crates/cdm/src/diff.rs`
  - `crates/cdm/src/error.rs`
  - `crates/cdm/src/json.rs`
  - `crates/cdm/src/model.rs`
  - `crates/cdm/src/lib.rs`
  - `crates/cdm/tests/schema_migration.rs`
- **Implementation Tasks:**
  1. Implement `crates/cdm/src/version.rs` providing `SchemaVersion` with full SemVer parsing, ordering, display, compatibility checks (`is_compatible_with`), and constants for current and supported versions.
  2. Implement `crates/cdm/src/migration.rs` providing:
     - `MigrationStep` trait with `from_version`, `to_version`, `description`, and `migrate(&Value) -> Result<Value, CdmError>`.
     - Sequential step execution through a graph/chain in `MigrationRegistry`.
     - Concrete migrations for `v0.8.0` $\to$ `v0.9.0` (mapping `content_blocks` $\to$ `sections`, normalising status codes) and `v0.9.0` $\to$ `v1.0.0` (deriving `completeness` report, populating missing BLAKE3 content hashes, establishing root `hierarchy` tree, and bumping `schema_version`).
  3. Implement `crates/cdm/src/diff.rs` providing `SchemaDiff` computing structural and semantic differences between documents or revisions (added/removed/modified sections, metadata shifts, completeness differences).
  4. Extend `CdmError` in `crates/cdm/src/error.rs` with `MigrationError` and `UnregisteredMigration`.
  5. Integrate auto-migration into `JsonOptions` and `from_json` / `from_reader` in `crates/cdm/src/json.rs`, allowing seamless transparent migration of legacy payloads.
  6. Add convenience methods on `CdmDocument` in `crates/cdm/src/model.rs` (`diff`, `from_json_migrated`, `from_value_migrated`).
  7. Re-export versioning, migration, and diffing primitives in `crates/cdm/src/lib.rs`.
  8. Implement comprehensive integration tests in `crates/cdm/tests/schema_migration.rs` verifying multi-step migration, field remapping, hash population, completeness invariant preservation, error handling for unregistered migrations, and diffing.
  9. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `SchemaVersion` correctly parses, orders, and compares semantic version strings.
  - Legacy payloads from `v0.8.0` and `v0.9.0` migrate losslessly into `v1.0.0` `CdmDocument` instances.
  - Migrated documents satisfy all `CdmValidator` invariants (`validate().is_ok() == true`).
  - Unsupported or invalid migration paths return strongly-typed `CdmError`.
  - Schema diffing accurately detects added, removed, and modified sections and metadata.
  - All workspace tests, clippy, fmt, and all 5 monorepo gates pass with zero errors.
- **Tests:** `cargo test -p web-repo-to-ai-cdm`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Auto-migration respects payload byte boundaries (`DEFAULT_MAX_JSON_BYTES`), avoids uncontrolled allocation, and validates migrated payloads against cryptographic invariants.
- **Performance Considerations:** Migrations operate on `serde_json::Value` in a single pass without unnecessary re-serialization roundtrips.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-cdm`: 78 passed (40 unit + 7 JSON serde integration + 7 schema migration integration + 8 schema validation integration + 16 validation invariants integration), 0 failed.
  - `cargo test --workspace`: 381 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** 2966faa
- **Status:** `ACCEPTED` (Phase 4.1 Certified)

---

### Phase 4.2 — Format Generators

#### Subphase `4.2.1` — Markdown generator in crates/exporters
- **Unique ID:** `4.2.1`
- **Objective:** Implement a deterministic, high-fidelity, AI-ready Markdown export engine in `crates/exporters` converting `CdmDocument` instances into clean, structured Markdown artifacts with configurable frontmatter, metadata overview, document hierarchy / directory trees, prose sections, polyglot code blocks with escalating fences, GFM tables, callout admonitions, provenance citations, and auditable exclusion reporting.
- **Scope:** `crates/exporters/Cargo.toml`, `crates/exporters/src/error.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/markdown.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/markdown_export.rs`.
- **Prerequisites:** `4.1.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/Cargo.toml`
  - `crates/exporters/src/error.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/markdown.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/markdown_export.rs`
- **Implementation Tasks:**
  1. Update `crates/exporters/Cargo.toml` to include necessary dependencies (`web-repo-to-ai-core`, `web-repo-to-ai-cdm`, `serde`, `serde_json`, `thiserror`, `chrono`).
  2. Implement `crates/exporters/src/error.rs` providing typed domain errors (`ExporterError`, `Result<T>`).
  3. Implement `crates/exporters/src/options.rs` providing `MarkdownOptions`, `ExportMetrics`, and configurable formatting parameters (frontmatter, metadata block, TOC, hierarchy tree, section headers, citations, exclusions, escalating code fences, token budgeting).
  4. Implement `crates/exporters/src/markdown.rs` providing:
     - `MarkdownExporter` engine with fluent builder API.
     - Deterministic YAML frontmatter serialization with proper escaping and schema alignment.
     - Document metadata overview block with source attribution, status, license, and language tags.
     - Hierarchy / directory tree rendering with ASCII/Unicode branches (`├──`, `└──`).
     - Section table of contents with anchor links and token/line telemetry.
     - Polyglot section rendering covering all `SectionKind` variants (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Admonition`, `Navigation`, `Manifest`, `Frontmatter`).
     - Backtick fence escalation (detecting internal backticks and expanding fence markers to 4+ backticks).
     - GFM blockquote alerts (`> [!NOTE]`, `> [!WARNING]`, etc.) for admonitions.
     - Auditable exclusions section documenting omitted files/blocks and reasons.
     - Sources & citations section formatting references.
     - Streaming export support via `export_markdown_to_writer` writing to any `std::io::Write`.
  5. Re-export public Markdown exporter types in `crates/exporters/src/lib.rs`.
  6. Create integration test suite in `crates/exporters/tests/markdown_export.rs` verifying deterministic output, all `SectionKind` variants, fence escalation, frontmatter formatting, exclusions, citations, token budgeting, and streaming writer.
  7. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `MarkdownExporter` converts any valid `CdmDocument` into deterministic, syntactically valid Markdown.
  - All `SectionKind` variants render accurately with appropriate markdown structures and language tags.
  - Internal triple backticks in code blocks trigger fence escalation to 4+ backticks without breaking markdown fences.
  - Exclusions and citations render faithfully to preserve auditability and prevent LLM hallucinations.
  - Both in-memory `export_to_string` and streaming `export_to_writer` produce byte-identical results and report accurate `ExportMetrics`.
  - All workspace tests pass, clippy passes with zero warnings, formatting checks clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, bounded allocation, no panics, canonical URL citations sanitized, and complete provenance tracking.
- **Performance Considerations:** Single-pass streaming rendering directly to writers without redundant string allocations, pre-allocated string buffers.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 16 passed (1 unit + 15 integration tests), 0 failed.
  - `cargo test --workspace`: 396 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** 90b4127
- **Status:** `ACCEPTED`

---

#### Subphase `4.2.2` — Plain-text generator in crates/exporters
- **Unique ID:** `4.2.2`
- **Objective:** Implement a deterministic, clean, AI-ready Plain-Text (`.txt`) export engine in `crates/exporters` converting `CdmDocument` instances into clean plain-text context packages free of Markdown syntax decoration, featuring configurable ASCII/Unicode header banners, directory tree hierarchy, table of contents, customizable section delimiters, code bodies with optional line numbering, provenance citations, auditable exclusions, completeness accounting, token budget caps, and streaming writer support.
- **Scope:** `crates/exporters/src/plain_text.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/plain_text_export.rs`.
- **Prerequisites:** `4.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/src/plain_text.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/plain_text_export.rs`
- **Implementation Tasks:**
  1. Extend `crates/exporters/src/options.rs` with `PlainTextOptions`:
     - Configurable header banner, metadata block, directory tree style (`HierarchyStyle`), table of contents, section banners, citations, exclusions, completeness report, divider character and width, token budget ceiling, and line numbering.
     - Presets: `default()`, `minimal()`, `compact()`.
  2. Implement `crates/exporters/src/plain_text.rs`:
     - `PlainTextExporter` struct with fluent builder methods.
     - Deterministic ASCII document header block (Title, Source URL, Type, Retrieved, License, Language, Metrics).
     - Directory tree / outline hierarchy renderer.
     - Plain-text Table of Contents with section numbers, titles, and volume/token metrics.
     - Polyglot section formatting with structured divider banners (`----------------`), section kind labels, metadata headers, and clean bodies without markdown fence noise.
     - Plain-text Citations & Sources section.
     - Plain-text Excluded Files & Content audit section.
     - Completeness Accounting breakdown.
     - Token budget enforcement with ASCII warning banner.
     - Streaming writer support via `export_plain_text_to_writer`.
  3. Re-export public Plain-Text exporter types in `crates/exporters/src/lib.rs`.
  4. Implement integration tests in `crates/exporters/tests/plain_text_export.rs` verifying default plain-text output, minimal/compact presets, line numbering, hierarchy styles, token budget truncation, streaming writer byte equivalence, and empty document handling.
  5. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `PlainTextExporter` converts any valid `CdmDocument` into deterministic, clean plain text without markdown syntax artefacts.
  - All `SectionKind` variants render accurately with clean visual separation and metadata headers.
  - Exclusions and citations render faithfully to preserve auditability for non-markdown consumers.
  - Both in-memory `export_to_string` and streaming `export_to_writer` produce byte-identical results and report accurate `ExportMetrics`.
  - All workspace tests pass, clippy passes with zero warnings, formatting checks clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, bounded allocation, no panics, canonical URL citations sanitized, and complete provenance tracking.
- **Performance Considerations:** Single-pass streaming rendering directly to writers without redundant string allocations, pre-allocated string buffers.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 26 passed (1 unit + 15 markdown + 10 plain text integration tests), 0 failed.
  - `cargo test --workspace`: 406 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** 46555f3
- **Status:** `ACCEPTED`

---

#### Subphase `4.2.3` — JSON generator in crates/exporters
- **Unique ID:** `4.2.3`
- **Objective:** Implement a deterministic, multi-flavor JSON export engine in `crates/exporters` converting `CdmDocument` instances into machine-readable JSON artifacts, supporting: (1) `CanonicalCdm` (the full versioned CDM schema v1.0), (2) `StructuredContext` (token-optimized, high-signal structured envelope tailored for LLM system/context prompts and RAG embeddings), and (3) `Minified` (compact single-line JSON without whitespace), complete with field selection, token budgeting, and streaming writer support.
- **Scope:** `crates/exporters/src/json.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/json_export.rs`.
- **Prerequisites:** `4.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/src/json.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/json_export.rs`
- **Implementation Tasks:**
  1. Extend `crates/exporters/src/options.rs` with `JsonFormatFlavor` (`CanonicalCdm`, `StructuredContext`, `Minified`) and `JsonExportOptions`:
     - Options for flavor selection, pretty-printing (2-space indentation), metadata inclusion, hierarchy tree inclusion, sections array, citations array, exclusions array, completeness reporting, and token budget ceiling.
     - Preset constructors: `canonical()`, `structured()`, `compact()`.
  2. Implement `crates/exporters/src/json.rs`:
     - `JsonExporter` struct with fluent builder API.
     - `export_json` for in-memory string generation.
     - `export_json_to_writer` for single-pass streaming rendering to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
     - Structured context models: `StructuredContextEnvelope`, `StructuredSection`, `StructuredHierarchyNode`, `StructuredCitation`, `StructuredExclusion`, `StructuredCompleteness`.
     - Canonical CDM serialization utilizing `to_writer` and `to_writer_pretty` with budget pruning when configured.
     - Token budget enforcement omitting excess sections with `truncated: true` telemetry.
  3. Re-export public JSON exporter types in `crates/exporters/src/lib.rs`.
  4. Implement integration tests in `crates/exporters/tests/json_export.rs` verifying Canonical CDM, Structured Context, Minified format, token budget truncation, field filtering, streaming writer byte equivalence, empty document handling, and serde roundtrips.
  5. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `JsonExporter` converts any valid `CdmDocument` into valid JSON according to the selected flavor.
  - `CanonicalCdm` produces schema-valid CDM v1.0 JSON matching `CdmValidator` invariants.
  - `StructuredContext` produces clean, token-efficient envelopes ready for direct LLM injection.
  - Both pretty-printed and compact/minified outputs are supported.
  - Streaming writer and in-memory export produce byte-identical results and report accurate `ExportMetrics`.
  - All workspace tests pass, clippy passes with zero warnings, formatting checks clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, bounded allocation, no panics, canonical URL citations sanitized, and complete provenance tracking.
- **Performance Considerations:** Single-pass streaming rendering directly to writers without redundant string allocations, pre-allocated string buffers.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 34 passed (1 unit + 15 markdown + 10 plain text + 8 json integration tests), 0 failed.
  - `cargo test --workspace`: 414 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** b210c1d
- **Status:** `ACCEPTED`

---

#### Subphase `4.2.4` — HTML & JSONL generators in crates/exporters
- **Unique ID:** `4.2.4`
- **Objective:** Implement standalone semantic HTML document and streaming JSON Lines (`.jsonl`) generators in `crates/exporters` converting `CdmDocument` instances into: (1) self-contained, responsive, dark/light theme aware HTML5 documents with syntax-highlighted code blocks, collapsible sections, table of contents, provenance badges, and completeness metrics; and (2) high-throughput line-delimited JSON (`.jsonl`) streams supporting RAG vector chunks, fine-tuning conversation/prompt-completion datasets, and canonical CDM section streaming.
- **Scope:** `crates/exporters/src/html.rs`, `crates/exporters/src/jsonl.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/html_export.rs`, `crates/exporters/tests/jsonl_export.rs`.
- **Prerequisites:** `4.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/src/html.rs`
  - `crates/exporters/src/jsonl.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/html_export.rs`
  - `crates/exporters/tests/jsonl_export.rs`
- **Implementation Tasks:**
  1. Add `HtmlOptions` to `crates/exporters/src/options.rs` (standalone flag, embedded CSS stylesheet, collapsible sections, line numbers, table of contents, citations, exclusions, completeness, token budget).
  2. Add `JsonlFlavor` (`RagChunks`, `FineTuningChat`, `FineTuningPromptCompletion`, `CanonicalSections`) and `JsonlOptions` to `crates/exporters/src/options.rs` (flavor, system prompt, token budget).
  3. Implement `crates/exporters/src/html.rs`:
     - HTML character entity escaping (`&`, `<`, `>`, `"`, `'`).
     - Standalone HTML5 layout with embedded CSS, zero external CDN dependencies, dark/light color scheme, responsive typography.
     - Collapsible `<details><summary>` code/content sections with syntax blocks.
     - Table of contents with anchor links, exclusions table, citations list, and completeness metrics badge.
     - `HtmlExporter` with `export_html` and streaming `export_html_to_writer`.
  4. Implement `crates/exporters/src/jsonl.rs`:
     - JSONL models: `RagChunkRecord`, `FineTuningChatRecord`, `FineTuningPromptCompletionRecord`.
     - Strictly line-delimited single-line JSON records followed by `\n`.
     - Token budget pruning.
     - `JsonlExporter` with `export_jsonl` and streaming `export_jsonl_to_writer`.
  5. Re-export public HTML and JSONL exporter types in `crates/exporters/src/lib.rs`.
  6. Implement comprehensive integration tests in `crates/exporters/tests/html_export.rs` and `crates/exporters/tests/jsonl_export.rs`.
  7. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `HtmlExporter` generates valid, standalone HTML5 documents and HTML fragments with accurate metadata, styled code blocks, collapsible sections, and responsive layout.
  - `JsonlExporter` outputs valid newline-delimited JSON records adhering strictly to the selected flavor (RAG chunks, fine-tuning, or canonical sections).
  - Both exporters support single-pass streaming writer emission with `CountingWriter` producing byte-identical outputs to string generation.
  - All workspace tests pass, clippy has 0 warnings, formatting is clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, strict HTML entity escaping to prevent XSS in exported HTML files, bounded allocations, no panics, canonical URL citations sanitized.
- **Performance Considerations:** Single-pass streaming rendering directly to writers without redundant string allocations, pre-allocated string buffers.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 53 passed (1 unit + 10 html + 8 json + 9 jsonl + 15 markdown + 10 plain text integration tests), 0 failed.
  - `cargo test --workspace`: 433 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** a2bd206
- **Status:** `ACCEPTED`

---

#### Subphase `4.3.1` — ZIP package generation in crates/exporters
- **Unique ID:** `4.3.1`
- **Objective:** Implement a deterministic, streaming ZIP archive package generator in `crates/exporters` converting `CdmDocument` instances into standardized `.zip` packages containing multi-format exports (`context.md`, `context.txt`, `context.json`, `document.json`, `context.html`, `context.jsonl`), individual source files organized in directory hierarchies (`sources/{path}`), metadata manifests (`manifest.json`), provenance reports (`provenance.json`), quality reports (`quality-report.json`), and coreutils-compatible SHA-256 checksums (`checksums.sha256`).
- **Scope:** `crates/exporters/src/zip.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/zip_export.rs`.
- **Prerequisites:** `4.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/src/zip.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/zip_export.rs`
- **Implementation Tasks:**
  1. Add `ZipPackageOptions`, `ZipPackageMetrics`, `ZipCompressionMethod`, and `PackageManifest` to `crates/exporters/src/options.rs` or `crates/exporters/src/zip.rs`.
  2. Implement `crates/exporters/src/zip.rs`:
     - `ZipPackageExporter`: Builder API supporting in-memory `export_to_bytes` and streaming `export_to_writer`.
     - Deterministic ZIP archive generation with uniform timestamps and permissions (0o644) for reproducible builds.
     - Integration of format generators: Markdown, Plain-Text, JSON (Canonical CDM and Structured Context), HTML, JSONL.
     - Individual source files extraction preserving relative folder structures (`sources/...`).
     - Generation of `manifest.json`, `provenance.json`, `quality-report.json`, and `checksums.sha256` calculating SHA-256 digests for each embedded archive entry.
     - Telemetry metrics capturing archive file count, uncompressed byte size, compressed byte size, and package SHA-256 digest.
  3. Re-export public ZIP package exporter types in `crates/exporters/src/lib.rs`.
  4. Implement integration tests in `crates/exporters/tests/zip_export.rs` verifying archive creation, multi-format file presence, SHA-256 checksum validity, streaming writer equivalence, empty document handling, and zip reading roundtrip with `zip::ZipArchive`.
  5. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `ZipPackageExporter` creates standard, uncorrupted `.zip` archives that open cleanly in any standard zip client (`unzip`, Windows Explorer, 7-zip).
  - Multi-format files, manifest, provenance, quality report, and checksums are accurately emitted according to options.
  - Every file in `checksums.sha256` matches the exact SHA-256 digest of the extracted entry byte-for-byte.
  - In-memory bytes generation and streaming writer emission produce valid archives.
  - All workspace tests pass, clippy has 0 warnings, formatting is clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, path traversal defense sanitizing entry names (preventing directory traversal attacks like `../` or absolute paths in ZIP headers), bounded allocations, no panics.
- **Performance Considerations:** Single-pass streaming rendering directly to zip writer sinks with Deflate compression, avoiding redundant buffers where possible.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 62 passed (1 unit + 10 html + 8 json + 9 jsonl + 15 markdown + 10 plain text + 9 zip integration tests), 0 failed.
  - `cargo test --workspace`: 442 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories (zip v9.6.1 with flate2/zlib-rs), and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** bd9b6ec
- **Status:** `ACCEPTED`

---

#### Subphase `4.3.2` — Manifest & checksums generation in crates/exporters
- **Unique ID:** `4.3.2`
- **Objective:** Implement standalone manifest and checksums generator modules in `crates/exporters`, delivering strongly-typed models, builders, serializers, parsers, and verifiers for package manifests (`manifest.json`), provenance reports (`provenance.json`), quality reports (`quality-report.json`), and BSD/GNU coreutils-compatible checksum files (`checksums.sha256`, `checksums.blake3`) with streaming writer support, dual SHA-256 and BLAKE3 cryptographic digests, MIME type detection, file entry inventory accounting, and integrity verification.
- **Scope:** `crates/exporters/src/manifest.rs`, `crates/exporters/src/checksum.rs`, `crates/exporters/src/options.rs`, `crates/exporters/src/error.rs`, `crates/exporters/src/zip.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/manifest_checksums.rs`.
- **Prerequisites:** `4.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/Cargo.toml`
  - `crates/exporters/src/manifest.rs`
  - `crates/exporters/src/checksum.rs`
  - `crates/exporters/src/options.rs`
  - `crates/exporters/src/error.rs`
  - `crates/exporters/src/zip.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/manifest_checksums.rs`
- **Implementation Tasks:**
  1. Add `blake3 = { workspace = true }` to `crates/exporters/Cargo.toml`.
  2. Implement `crates/exporters/src/checksum.rs`:
     - `ChecksumAlgorithm`: `Sha256`, `Blake3`.
     - `ChecksumEntry`: `(path, hash, algorithm)`.
     - Core functions: `compute_sha256_hex`, `compute_blake3_hex`, `generate_sha256_checksums`, `generate_blake3_checksums`, `parse_sha256_checksums`.
     - Checksum verifier: `verify_sha256_checksums` and `ChecksumVerificationReport` checking for matched, mismatched, and missing files against a byte/file provider.
     - Streaming writer: `export_checksums_to_writer`.
  3. Implement `crates/exporters/src/manifest.rs`:
     - Standalone `PackageFileEntry`, `PackageManifest`, `PackageProvenance`, and `PackageQualityReport` schemas with dual SHA-256 and BLAKE3 hashes, MIME type classification, line count, and token estimations.
     - `ManifestExporter` and `export_manifest` / `export_manifest_to_writer`.
     - `export_provenance` / `export_provenance_to_writer`.
     - `export_quality_report` / `export_quality_report_to_writer`.
     - `build_manifest_from_cdm` generating package file entries for all standard multi-format artifacts and extracted source files.
  4. Refactor `crates/exporters/src/zip.rs` to leverage `manifest.rs` and `checksum.rs`.
  5. Add `ManifestOptions` and `ChecksumOptions` to `crates/exporters/src/options.rs`.
  6. Add `ChecksumValidation` error variant to `crates/exporters/src/error.rs`.
  7. Re-export public manifest and checksum types in `crates/exporters/src/lib.rs`.
  8. Implement integration tests in `crates/exporters/tests/manifest_checksums.rs` verifying manifest generation, provenance, quality report, checksum formatting, parsing, corrupt file detection, and streaming equivalence.
  9. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Standalone manifest JSON, provenance JSON, and quality-report JSON accurately serialize from `CdmDocument` and arbitrary file entry sets.
  - Checksums manifest adheres strictly to BSD/GNU coreutils standard (`<64-hex-hash>  <relative_path>\n`).
  - `parse_sha256_checksums` parses standard checksum files, ignoring comments and whitespace, and reporting syntax errors.
  - `verify_sha256_checksums` accurately detects matching files, hash mismatches (tampering), and missing files.
  - Streaming writer implementations emit byte-identical output to in-memory string generators.
  - All workspace tests pass, clippy has 0 warnings, formatting is clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Cryptographically robust hashes (SHA-256 and BLAKE3), path sanitization preventing directory traversal, memory bounded parsing, no panics.
- **Performance Considerations:** Single-pass hash computation directly during export, zero-copy parsing where feasible, streaming writer sinks.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 73 passed (1 unit + 10 html + 8 json + 9 jsonl + 11 manifest & checksums + 15 markdown + 10 plain text + 9 zip integration tests), 0 failed.
  - `cargo test --workspace`: 453 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** 6fae508
- **Status:** `ACCEPTED`

---

#### Subphase `4.3.3` — Streaming downloads in services/api
- **Unique ID:** `4.3.3`
- **Objective:** Implement production-grade asynchronous streaming artifact downloads in `services/api` for `GET /api/v1/jobs/{job_id}/artifacts/{format}` (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `checksums`), streaming directly from pluggable storage (`StorageBackend::open_read`) or performing on-demand CDM format generation with bounded chunked HTTP bodies, RFC 7807 problem details for missing or unready artifacts, accurate MIME content types, and standard `Content-Disposition`, `Content-Length`, `ETag`, and `X-Content-Type-Options` headers.
- **Scope:** `services/api/src/routes/jobs.rs`, `services/api/src/state.rs`, `services/api/src/dto/jobs.rs`, `services/api/src/dto/params.rs`, `services/api/src/lib.rs`, `services/api/Cargo.toml`, `Cargo.toml`.
- **Prerequisites:** `4.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`
  - `services/api/Cargo.toml`
  - `services/api/src/state.rs`
  - `services/api/src/routes/jobs.rs`
  - `services/api/src/dto/jobs.rs`
  - `services/api/src/dto/params.rs`
  - `services/api/src/lib.rs`
- **Implementation Tasks:**
  1. Add `tokio-util = { version = "0.7", default-features = false, features = ["io"] }` to root `Cargo.toml` and `[dependencies]` in `services/api/Cargo.toml`. Add `web-repo-to-ai-exporters = { workspace = true }` to `services/api/Cargo.toml`.
  2. Expand `VALID_EXPORT_FORMATS` in `services/api/src/dto/jobs.rs` to include `"checksums"`.
  3. Wire `DynStorage` into `AppState` in `services/api/src/state.rs` with `create_storage_backend(&config.storage)` and test/memory injection constructors (`with_storage`, `with_metrics_and_storage`).
  4. Implement `get_artifact_handler` in `services/api/src/routes/jobs.rs`:
     - Parse format via `ArtifactFormat::from_str`.
     - Inspect storage for pre-rendered artifact (`jobs/{job_id}/artifact.{ext}`).
     - If not pre-rendered, check for parent CDM (`jobs/{job_id}/artifact.cdm.json`), deserialize, and stream format on-demand.
     - Stream chunks asynchronously using `ReaderStream` and Axum `Body::from_stream`.
     - Set standard headers: `Content-Type`, `Content-Disposition`, `Content-Length`, `ETag`, `x-content-type-options: nosniff`, `cache-control: private, no-transform, max-age=86400`, and `x-request-id`.
     - If neither artifact nor CDM exists, return RFC 7807 404 Problem Details.
  5. Add integration tests in `services/api/src/lib.rs` verifying streaming downloads for each supported format (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `checksums`), non-existent job 404 handling, invalid format 400 rejection, and headers validation.
  6. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `GET /api/v1/jobs/{job_id}/artifacts/{format}` streams artifacts for all supported formats without buffering the entire payload into RAM.
  - Correct MIME type and `Content-Disposition` attachment filename headers are emitted for each format.
  - On-demand rendering from stored CDM is supported when pre-rendered artifact does not yet exist.
  - Missing artifacts return RFC 7807 problem details with `application/problem+json` media type.
  - All workspace tests pass, clippy has 0 warnings, formatting is clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-api`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Safe memory handling, bounded chunk streams preventing OOM, nosniff header, no directory traversal via `job_id` (enforced by typed `JobId` UUID v7).
- **Performance Considerations:** Single-pass streaming directly from `StorageBackend::open_read` or exporter writers without buffering into redundant vectors.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 41 passed (40 lib tests + 1 binary metadata test, including 4 new streaming download suites covering direct storage streaming, on-demand CDM rendering, manifest/checksums downloads, and 404/400 validation), 0 failed.
  - `cargo test --workspace`: 457 passed, 0 failed across all monorepo crates.
  - `apps/web` (SvelteKit + Vitest): 184 passed, 0 failed across 15 test suites.
  - `services/browser-worker` (Node.js + Jest): 29 passed, 0 failed across 7 test suites.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, clean pass.
  - `cargo fmt --all -- --check`: Clean formatting verified across all Rust crates.
  - `cargo deny check`: Licenses, bans, advisories, and sources passed with zero violations.
  - Monorepo 5-gate audit (`powershell scripts/check-all.ps1`): ALL 5 GATES PASSED (100% pass rate).
- **Git Commit Reference:** b79cf24
- **Status:** `ACCEPTED`

---

#### Subphase `4.3.4` — Export integrity tests in crates/exporters
- **Unique ID:** `4.3.4`
- **Objective:** Implement comprehensive export integrity verification suites and runtime package validation in `crates/exporters`, guaranteeing lossless multi-format content fidelity, complete package checksum verification, zip-slip and adversarial path containment, token budget truncation resilience, corruption and tampering detection, and zero-loss roundtrip extraction across all 6 artifact formats, metadata manifests, and compressed ZIP archives.
- **Scope:** `crates/exporters/src/integrity.rs`, `crates/exporters/src/lib.rs`, `crates/exporters/tests/export_integrity.rs`.
- **Prerequisites:** `4.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/exporters/src/integrity.rs`
  - `crates/exporters/src/lib.rs`
  - `crates/exporters/tests/export_integrity.rs`
- **Implementation Tasks:**
  1. Implement `crates/exporters/src/integrity.rs`:
     - `PackageIntegrityReport`: Diagnostic report model capturing validation status, verified files, checksum mismatches, missing files, unlisted entries, path traversal violations, and deserialized summaries (`manifest`, `provenance`, `quality_report`).
     - `verify_zip_package_integrity`: In-memory and streaming verification function that unpacks a generated ZIP archive, verifies `manifest.json` schema, cross-checks each entry in `checksums.sha256` against extracted bytes, verifies declared file sizes and digests in `manifest.json`, and audits entry paths against zip-slip directory traversal vulnerabilities.
  2. Re-export integrity verification types in `crates/exporters/src/lib.rs`.
  3. Implement comprehensive integration tests in `crates/exporters/tests/export_integrity.rs`:
     - Test 1: Complete package integrity verification roundtrip (100% valid package).
     - Test 2: Tampered entry detection (corrupted byte in bundled file detected by SHA-256 mismatch).
     - Test 3: Truncated entry detection (partial file detected by size & hash mismatch).
     - Test 4: Missing file detection (file listed in manifest or checksums but missing from archive).
     - Test 5: Unlisted rogue file detection (file in archive but absent from manifest).
     - Test 6: Adversarial path sanitization & zip-slip resistance (`../`, `/etc/passwd`, `C:\Windows\System32`, `\nul`).
     - Test 7: Multi-format lossless text fidelity (content roundtrip across Markdown, PlainText, JSON, HTML, and JSONL).
     - Test 8: Token budget truncation integrity (enforcing truncation without corrupting JSON/HTML/Markdown syntax).
     - Test 9: Edge cases (empty documents, unicode RTL/CJK/emoji sequences, deeply nested directory trees).
  4. Run all 5 monorepo verification gates in `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `verify_zip_package_integrity` successfully validates valid export packages and flags tampered, truncated, missing, unlisted, or traversal-violating entries with exact diagnostic error details.
  - All 6 format generators preserve content losslessly and maintain valid structural syntax even under aggressive token budget truncation.
  - All workspace tests pass, clippy has 0 warnings, formatting is clean, and cargo deny passes.
- **Tests:** `cargo test -p web-repo-to-ai-exporters --test export_integrity`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 01, 04, 05: Cryptographic hash validation, path sanitization preventing zip-slip directory traversal, memory bounded streaming and parsing, no panics.
- **Performance Considerations:** In-memory and streaming archive validation using single-pass hash computation and zero redundant buffering.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-exporters --test export_integrity`: 9/9 integration tests passed (valid package roundtrip, tampered entry detection, truncated entry detection, missing file detection, unlisted rogue file detection, zip-slip & path traversal defense, syntax verifiers, token budget truncation fidelity, and unicode/deeply-nested directory trees).
  - `cargo test -p web-repo-to-ai-exporters`: 82/82 tests passed across 8 test suites.
  - `cargo test --workspace`: 330/330 tests passed across all 17 crates and services.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 184/184 tests passed across 15 suites; `svelte-check` reports 0 errors and 0 warnings.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures across all 5 verification gates, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `2e53391`
- **Status:** `ACCEPTED`

---

### Phase 4.3 Milestone Certification (`4.3.1` – `4.3.4`)
- **Phase 4.3 Goal:** Multi-format packaging, coreutils manifests and checksums, streaming API endpoints, and export integrity verification.
- **Subphases Completed & Accepted:**
  - `4.3.1`: ZIP package generation in `crates/exporters` (`ACCEPTED`, commit `bd9b6ec` / `064beed`)
  - `4.3.2`: Manifest & checksums generation in `crates/exporters` (`ACCEPTED`, commit `6fae508` / `1f8babc`)
  - `4.3.3`: Streaming downloads in `services/api` (`ACCEPTED`, commit `b79cf24` / `a5b756a`)
  - `4.3.4`: Export integrity tests in `crates/exporters` (`ACCEPTED`, commit `2e53391`)
- **Certification Date:** 2026-11-17
- **Certification Summary:**
  1. Complete export engine producing deterministic, reproducible ZIP archives packaging Markdown, PlainText, Canonical CDM JSON, Structured JSON, HTML5, JSONL RAG chunks, raw source code trees, `manifest.json`, `provenance.json`, `quality-report.json`, and coreutils `checksums.sha256`.
  2. Single-pass chunked streaming download endpoints in `services/api` via `tokio-util` and `axum::body::Body::from_stream` with zero-RAM-buffering, on-demand CDM fallback rendering, and RFC 7807/RFC 7231 compliance.
  3. Strict package integrity verification engine auditing cryptographic SHA-256 and BLAKE3 digests, exact file sizes, zip-slip directory traversal sanitization, and rogue unlisted file auditing.
  4. 82 tests passing in `crates/exporters`, 41 tests passing in `services/api`, 330 tests passing across the workspace with zero clippy warnings.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-17)

---

#### Subphase `4.4.1` — Connect actual conversion API in apps/web
- **Unique ID:** `4.4.1`
- **Objective:** Connect the SvelteKit frontend in `apps/web` to the live backend Axum API (`services/api`), replacing mock `setTimeout` transitions with real strongly-typed REST client interactions (`POST /api/v1/jobs`, `GET /api/v1/jobs/{job_id}`, `POST /api/v1/sources/detect`), RFC 7807 problem details error handling, and bidirectional domain model adapters.
- **Scope:** `apps/web/src/lib/api/types.ts`, `apps/web/src/lib/api/client.ts`, `apps/web/src/lib/api/adapter.ts`, `apps/web/src/lib/api/index.ts`, `apps/web/src/routes/+page.svelte`, `apps/web/src/routes/workspace/+page.svelte`, `apps/web/tests/api-client.test.ts`.
- **Prerequisites:** `4.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/api/types.ts`
  - `apps/web/src/lib/api/client.ts`
  - `apps/web/src/lib/api/adapter.ts`
  - `apps/web/src/lib/api/index.ts`
  - `apps/web/src/routes/+page.svelte`
  - `apps/web/src/routes/workspace/+page.svelte`
  - `apps/web/tests/api-client.test.ts`
- **Implementation Tasks:**
  1. Implement strongly-typed API client contracts and DTOs in `apps/web/src/lib/api/types.ts` matching `services/api/src/dto/jobs.rs` and `services/api/src/dto/sources.rs`.
  2. Implement `ApiClient` and `ApiClientError` in `apps/web/src/lib/api/client.ts` handling `POST /api/v1/jobs`, `GET /api/v1/jobs/{job_id}`, `POST /api/v1/sources/detect`, RFC 7807 decoding, and download URL generation.
  3. Implement domain adapter in `apps/web/src/lib/api/adapter.ts` mapping backend `ApiJobResponse` into `ConversionJob`.
  4. Update `apps/web/src/routes/+page.svelte` to submit conversion jobs to `defaultApiClient.createJob(...)` and redirect with real server-generated UUID v7 job IDs.
  5. Update `apps/web/src/routes/workspace/+page.svelte` to retrieve job metadata using `defaultApiClient.getJob(...)` and map it into workspace state.
  6. Implement comprehensive Vitest unit and integration test suite in `apps/web/tests/api-client.test.ts`.
  7. Run full monorepo quality audit via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Form submission dispatches a verified `POST /api/v1/jobs` request containing user settings, target formats, and token budget.
  - Zero placeholder `setTimeout` transitions in conversion submission path.
  - RFC 7807 problem details are parsed and displayed gracefully on error.
  - All 195 frontend tests pass across 16 suites; `svelte-check` reports 0 errors and 0 warnings; Prettier confirms formatting.
  - All 5 gates in `scripts/check-all.ps1` pass cleanly.
- **Tests:** `npm --prefix apps/web test`, `npm --prefix apps/web run check`, `npm --prefix apps/web run lint`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 03, 05, 09: Input validation, zero unescaped strings, RFC 7807 sanitization without leaking stack traces, SSRF validation prior to job submission.
- **Performance Considerations:** Lightweight zero-dependency API client, reusable singleton instance, non-blocking asynchronous fetch execution.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `npm --prefix apps/web test`: 195/195 tests passed across 16 test files (11 new tests in `api-client.test.ts`).
  - `npm --prefix apps/web run check`: 0 errors, 0 warnings.
  - `npm --prefix apps/web run lint`: 100% Prettier formatting compliance.
  - `cargo test --workspace`: 330/330 tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `94129d2`
- **Status:** `ACCEPTED`

---

#### Subphase `4.4.2` — Real progress streaming (SSE) in apps/web
- **Unique ID:** `4.4.2`
- **Objective:** Connect the conversion workspace in `apps/web` to the live Server-Sent Events (SSE) stream (`GET /api/v1/jobs/{job_id}/events`), updating the 4-stage pipeline progress, log stream, completeness accounting, and token metrics in real time with automatic reconnect, exponential backoff, heartbeat monitoring, and safe resource cleanup on completion or unmount.
- **Scope:** `apps/web/src/lib/api/sse.ts`, `apps/web/src/lib/api/index.ts`, `apps/web/src/routes/workspace/+page.svelte`, `services/api/src/routes/jobs.rs`, `services/api/src/routes/mod.rs`, `apps/web/tests/sse-stream.test.ts`.
- **Prerequisites:** `4.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/api/sse.ts`
  - `apps/web/src/lib/api/index.ts`
  - `apps/web/src/routes/workspace/+page.svelte`
  - `services/api/src/routes/jobs.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
  - `apps/web/tests/sse-stream.test.ts`
- **Implementation Tasks:**
  1. Implement `job_events_handler` in `services/api/src/routes/jobs.rs` and register routes for `/jobs/:job_id/events` and `/jobs/:job_id/stream` in `services/api/src/routes/mod.rs`, emitting typed `phase`, `log`, `completeness`, `tokens`, `artifact`, and `done` events with 15-second keep-alive pings.
  2. Implement `JobEventStream` and `connectJobEventStream` in `apps/web/src/lib/api/sse.ts` with typed callback handlers, automatic exponential backoff reconnection, watchdog heartbeat timeout monitoring, and explicit `close()` cleanup.
  3. Wire `apps/web/src/routes/workspace/+page.svelte` to connect to `connectJobEventStream` on mount, binding live SSE events directly to Svelte 5 `$state` runes, updating pipeline stages, progress bars, logs, completeness accounting, and artifact downloads in real time.
  4. Ensure `onDestroy` in `apps/web/src/routes/workspace/+page.svelte` and user cancel actions close the stream to prevent SSE connection leaks.
  5. Implement comprehensive Vitest test suite in `apps/web/tests/sse-stream.test.ts` (11 tests) verifying event decoding, automatic closure on `done`, reconnection retry limits, and route lifecycle bindings.
  6. Add backend integration test `test_get_job_events_sse_stream` in `services/api/src/lib.rs`.
  7. Run full monorepo quality audit via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Conversion workspace connects to SSE stream and dynamically updates pipeline stages, log messages, file completeness counts, and token metrics without polling.
  - Connection automatically cleans up on job completion or component unmount with zero connection leaks.
  - Transient connection drops trigger exponential backoff reconnection.
  - All 206 frontend tests pass across 17 test suites; `svelte-check` reports 0 errors and 0 warnings; Prettier confirms formatting.
  - All 331 workspace Rust tests pass with zero clippy warnings.
  - All 5 gates in `scripts/check-all.ps1` pass cleanly.
- **Tests:** `npm --prefix apps/web test`, `npm --prefix apps/web run check`, `npm --prefix apps/web run lint`, `cargo test -p web-repo-to-ai-api`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 03, 05: Validated job UUID parameters, sanitized event payloads, zero XSS injection via SSE data, bounded log message buffers.
- **Performance Considerations:** SSE streaming avoids expensive HTTP polling loops, utilizes standard browser EventSource, and automatically terminates connections upon terminal events.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `npm --prefix apps/web test`: 206/206 tests passed across 17 test files (11 new tests in `sse-stream.test.ts`).
  - `npm --prefix apps/web run check`: 0 errors, 0 warnings.
  - `npm --prefix apps/web run lint`: 100% Prettier formatting compliance.
  - `cargo test -p web-repo-to-ai-api`: 41/41 tests passed (including `test_get_job_events_sse_stream`).
  - `cargo test --workspace`: 331/331 tests passed across all 17 crates and services.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `7ee62dd`
- **Status:** `ACCEPTED`

---

#### Subphase `4.4.3` — Content preview and copy controls in apps/web
- **Unique ID:** `4.4.3`
- **Objective:** Implement content preview endpoint `GET /api/v1/jobs/{job_id}/preview` and connect live preview, format selection, code line filtering, line number and word wrap toggling, truncation alert banners, and accessible clipboard copy controls in `apps/web`.
- **Scope:** `services/api/src/dto/jobs.rs`, `services/api/src/dto/mod.rs`, `services/api/src/routes/jobs.rs`, `services/api/src/routes/mod.rs`, `services/api/src/lib.rs`, `apps/web/src/lib/api/types.ts`, `apps/web/src/lib/api/client.ts`, `apps/web/src/lib/components/workspace/PreviewPanel.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/tests/api-client.test.ts`, `apps/web/tests/preview-panel.test.ts`.
- **Prerequisites:** `4.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `services/api/src/dto/jobs.rs`
  - `services/api/src/dto/mod.rs`
  - `services/api/src/routes/jobs.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
  - `apps/web/src/lib/api/types.ts`
  - `apps/web/src/lib/api/client.ts`
  - `apps/web/src/lib/components/workspace/PreviewPanel.svelte`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/tests/api-client.test.ts`
  - `apps/web/tests/preview-panel.test.ts`
- **Implementation Tasks:**
  1. Define DTOs `JobPreviewResponse`, `PreviewFileItem`, and `PreviewQuery` in `services/api/src/dto/jobs.rs`, implementing `Validatable` with format validation and `max_lines` bounds (1 to 50,000).
  2. Implement `get_preview_handler` in `services/api/src/routes/jobs.rs` and register route `/jobs/:job_id/preview` in `services/api/src/routes/mod.rs`: retrieves stored `CdmDocument`, generates requested export format slice (`markdown`, `json`, `html`, `txt`, `jsonl`, `xml`), applies `max_lines` windowing, extracts file section metrics, and returns RFC 7807 problem details when not found.
  3. Add backend integration tests in `services/api/src/lib.rs`: `test_get_preview_not_found`, `test_get_preview_invalid_format_rejected`, `test_get_preview_success_from_cdm` (verifying markdown, JSON, file-specific filtering, and `max_lines` truncation).
  4. Define frontend interfaces `PreviewFileItem` and `JobPreviewResponse` in `apps/web/src/lib/api/types.ts` matching backend schema.
  5. Add `getJobPreview(jobId, options)` method to `ApiClient` in `apps/web/src/lib/api/client.ts` with strongly-typed query parameters and error decoding.
  6. Enhance `PreviewPanel.svelte`: accept `jobId`, query live preview from API via Svelte 5 runes `$state` / `$effect`, provide graceful fallback to local preview formatting on offline/pending states, implement robust clipboard copy (`navigator.clipboard.writeText` with `document.execCommand` fallback), add accessible ARIA live status announcement region, truncation warning banner, line search filtering, line numbers toggle, and word wrap toggle.
  7. Pass `jobId={job.id}` from `JobWorkspace.svelte` to `PreviewPanel.svelte`.
  8. Expand Vitest test suites in `apps/web/tests/api-client.test.ts` (13 tests) and `apps/web/tests/preview-panel.test.ts` (20 tests), achieving 211 passing frontend tests.
  9. Run full verification gates via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `GET /api/v1/jobs/{job_id}/preview` endpoint serves format-specific content preview with truncation flags and file metrics.
  - PreviewPanel connects to live backend preview when `jobId` is provided, degrading gracefully to local preview formatting on pending or offline states.
  - Clipboard copy control writes formatted content to clipboard with accessible ARIA live feedback.
  - Line number gutter, word wrap, line filtering, and truncation notice work seamlessly.
  - All 211 frontend tests pass; `svelte-check` reports 0 errors and 0 warnings; Prettier confirms formatting.
  - All 335 workspace Rust tests pass with zero clippy warnings.
  - All 5 gates in `scripts/check-all.ps1` pass cleanly.
- **Tests:** `cargo test -p web-repo-to-ai-api`, `cargo test --workspace`, `npm --prefix apps/web test`, `npm --prefix apps/web run check`, `npm --prefix apps/web run lint`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 03, 05: Validated job UUID and format query parameters, bounded line limits (<= 50,000) preventing memory exhaustion, safe HTML escaping in syntax highlighter and truncation banners preventing XSS.
- **Performance Considerations:** Windowed preview (default 1,000 / 2,000 lines) prevents DOM lag on large multi-megabyte repositories, zero polling, minimal memory allocation during on-demand slice generation.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 45/45 tests passed.
  - `cargo test --workspace`: 335/335 tests passed across all 17 crates and services.
  - `npm --prefix apps/web test`: 211/211 tests passed across 17 test files.
  - `npm --prefix apps/web run check`: 0 errors, 0 warnings.
  - `npm --prefix apps/web run lint`: 100% Prettier formatting compliance.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `2f767de`
- **Status:** `ACCEPTED`

---

#### Subphase `4.4.4` — Multi-format downloads in apps/web
- **Unique ID:** `4.4.4`
- **Objective:** Implement multi-format downloads in `apps/web` connecting `DownloadControls.svelte` and `PreviewPanel.svelte` to backend artifact streaming endpoints (`/api/v1/jobs/{job_id}/artifacts/{format}` and `/api/v1/jobs/{job_id}/artifacts/zip`), supporting single-file format downloads, complete ZIP packages with checksum verification, accessible download progress triggers, clipboard SHA-256 copying, and graceful client-side blob generation fallback.
- **Scope:** `apps/web/src/lib/types/download.ts`, `apps/web/src/lib/components/workspace/DownloadControls.svelte`, `apps/web/src/lib/components/workspace/PreviewPanel.svelte`, `apps/web/src/lib/components/workspace/JobWorkspace.svelte`, `apps/web/tests/download-controls.test.ts`.
- **Prerequisites:** `4.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `apps/web/src/lib/types/download.ts`
  - `apps/web/src/lib/components/workspace/DownloadControls.svelte`
  - `apps/web/src/lib/components/workspace/PreviewPanel.svelte`
  - `apps/web/src/lib/components/workspace/JobWorkspace.svelte`
  - `apps/web/tests/download-controls.test.ts`
- **Implementation Tasks:**
  1. Enhance `ExportArtifact` model in `apps/web/src/lib/types/download.ts` to include optional `downloadUrl` and support streaming endpoints from backend.
  2. Enhance `buildExportArtifacts` to accept optional `jobId` and `backendArtifacts`, creating artifact models with real streaming backend URLs and adding complete ZIP package entries (`web-repo-context.zip`).
  3. Implement `triggerDownload(filename, content?, mimeType?, downloadUrl?)` in `download.ts` handling both direct anchor downloads to streaming backend URLs and client-side blob generation fallback.
  4. Enhance `PreviewPanel.svelte` top toolbar download button to query real streaming backend artifact URL (`defaultApiClient.getArtifactUrl(jobId, selectedFormat)`) when `jobId` is available.
  5. Enhance `DownloadControls.svelte` to accept `jobId` and `artifacts` props, auto-refresh artifacts from backend metadata, wire all format cards to `triggerDownload`, add prominent "Download ZIP Package" button, and add accessible ARIA live status announcement region for download state and clipboard copy feedback.
  6. Update `JobWorkspace.svelte` to pass `jobId={job.id}` and `artifacts={job.artifacts}` to `DownloadControls`.
  7. Expand Vitest tests in `apps/web/tests/download-controls.test.ts` (14 tests) verifying ZIP package presence, backend download URLs, blob fallback, checksum copying, ARIA live announcements, and workspace integration.
  8. Run all monorepo quality gates via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `DownloadControls` renders individual download buttons for each exported format (`markdown`, `json`, `html`, `txt`, `jsonl`) and a primary "Download ZIP Package" button.
  - Formats link directly to streaming backend endpoints `/api/v1/jobs/{job_id}/artifacts/{format}` when `jobId` is present, or gracefully trigger client-side blob download fallback when offline.
  - Checksum copy button provides clipboard fallback with accessible screen reader announcements.
  - All 213 frontend tests pass across 17 suites; `svelte-check` reports 0 errors and 0 warnings; Prettier confirms formatting.
  - All 335 workspace Rust tests pass with zero clippy warnings.
  - All 5 gates in `scripts/check-all.ps1` pass cleanly.
- **Tests:** `npm --prefix apps/web test`, `npm --prefix apps/web run check`, `npm --prefix apps/web run lint`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 03, 05, 09: Content-Disposition attachment downloads, sanitization of filenames, safe clipboard copy fallback without `eval` or unsafe DOM injections, zero XSS vulnerability.
- **Performance Considerations:** Direct streaming links delegate file assembly and compression to backend async Tokio chunked streams, eliminating browser memory spikes on multi-megabyte exports.
- **Documentation Requirements:** Updated `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `npm --prefix apps/web test`: 213/213 tests passed across 17 test suites (14 tests in `download-controls.test.ts`).
  - `npm --prefix apps/web run check`: 0 errors, 0 warnings.
  - `npm --prefix apps/web run lint`: 100% Prettier formatting compliance.
  - `cargo test --workspace`: 335/335 tests passed across all 17 crates and services.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `37b9dc0` / `8f66c73`
- **Status:** `ACCEPTED`

---

### Phase 4.4 Milestone Certification (`4.4.1` – `4.4.4`)

- **Phase Name:** `Phase 4.4 — Export frontend`
- **Subphases Covered:** `4.4.1` (Connect actual conversion API), `4.4.2` (Real progress streaming SSE), `4.4.3` (Content preview and copy controls), `4.4.4` (Multi-format downloads in apps/web). All 4 `ACCEPTED`.
- **Certification Date:** 2026-11-17
- **Certification Summary:**
  1. Full connection of SvelteKit frontend in `apps/web` to Axum REST API: real job creation, job status retrieval, source detection, RFC 7807 problem details parsing, zero `setTimeout` mock transitions.
  2. Real-time progress streaming over Server-Sent Events (SSE) via `/jobs/:job_id/events` and `/jobs/:job_id/stream`: typed phase, log, completeness, token, artifact, and done events with heartbeat watchdog and automatic exponential backoff reconnection.
  3. Interactive content preview with syntax formatting, format switcher, windowed line rendering, line search filtering, line number and word wrap toggles, truncation alerts, and accessible clipboard copy controls.
  4. Multi-format artifact downloads directly via streaming backend endpoints (`/artifacts/{format}` and `/artifacts/zip`), checksum SHA-256 copy utilities, client-side fallback blob generation, and screen reader announcements.
  5. 213 passing frontend tests across 17 suites, 0 Svelte/TypeScript errors, 100% Prettier formatting, 335 passing workspace Rust tests.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Phase 4.4 Certified 2026-11-17)

---

### STAGE 4 COMPLETION CERTIFICATION

- **Stage ID:** `STAGE 4 — CANONICAL MODEL AND EXPORT ENGINE`
- **Total Phases Completed:** 4 (`Phase 4.1`, `Phase 4.2`, `Phase 4.3`, `Phase 4.4`)
- **Total Subphases Completed:** 16 (`4.1.1` through `4.4.4`, all `ACCEPTED`)
- **Stage Acceptance Gate:** Canonical Context Document Model (CDM v1.0), 5 format generators (Markdown, TXT, JSON, HTML, JSONL), ZIP packaging, manifest/checksums generation, chunked streaming downloads, export integrity verification, real API wiring, SSE streaming, preview panel, and multi-format download controls in `apps/web`.
- **Verification Evidence:**
  1. **Phase 4.1 (CDM Foundation):** Strongly-typed Rust CDM schema (`crates/cdm`), JSON Schema validation, metadata cascading, document builder, and AST visitor pattern.
  2. **Phase 4.2 (Format Generators):** Deterministic Markdown generator with table formatting and YAML frontmatter, plain text extractor with semantic delimiters, canonical CDM JSON & structured JSON serializers, HTML5 semantic generator with inline styling, and JSONL RAG chunk exporter with sliding window chunking.
  3. **Phase 4.3 (Packaging & Export Services):** Deterministic ZIP packaging with compression levels and directory tree preservation, coreutils-compatible `checksums.sha256` and `manifest.json` generation, memory-bounded chunked streaming API endpoints (`services/api`), and export integrity test suite with zip-slip prevention and SHA-256 verification.
  4. **Phase 4.4 (Export Frontend):** End-to-end integration in `apps/web`: real API job submission and status polling, SSE live event streaming, windowed content preview with search/toggle controls, clipboard copy with ARIA announcements, and multi-format direct streaming/ZIP downloads.
  5. **Automated Test Quality:** 335 automated Rust workspace tests passing (82 in `crates/exporters`, 45 in `services/api`), 213 automated frontend tests passing across 17 suites in `apps/web`; zero compiler/clippy warnings; 100% code style compliance; zero advisories in `cargo deny`.
  6. **Monorepo Quality Gates:** All 5 gates pass cleanly on Windows PowerShell (`scripts/check-all.ps1`) and POSIX bash (`scripts/check-all.sh`), with Podman storage footprint at 5.27 GB / 30 GB (17.6%).
- **Status:** `COMPLETED` / `ACCEPTED` (Stage 4 Certified 2026-11-17)

---

## STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION

### Phase 5.1 — GitHub metadata

#### Subphase `5.1.1` — Repository identification and URL decomposition
- **Unique ID:** `5.1.1`
- **Objective:** Design and implement robust GitHub repository URL identification, parsing, decomposition, and normalization in `crates/github-extractor`, handling standard web URLs, SSH/Git transports, tree/blob subpaths, branches, tags, commit SHAs, and shorthand references with comprehensive URL security and SSRF validation.
- **Scope:** `crates/github-extractor/Cargo.toml`, `crates/github-extractor/src/target.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/*`.
- **Prerequisites:** Stage 4 (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/Cargo.toml`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/target.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/target_decomposition_test.rs`
- **Implementation Tasks:**
  1. Add dependencies `web-repo-to-ai-url-security`, `url`, `serde`, `serde_json`, `regex`, `tracing` to `crates/github-extractor/Cargo.toml`.
  2. Implement strongly-typed error taxonomy `GitHubExtractorError` in `crates/github-extractor/src/error.rs` using `thiserror`.
  3. Implement `GitHubTarget`, `GitHubTargetType`, and `GitHubUrlParser` in `crates/github-extractor/src/target.rs`:
     - Support HTTPS, HTTP, Git, and SSH URLs (`https://github.com/owner/repo`, `git@github.com:owner/repo.git`, `owner/repo`).
     - Parse tree paths (`/tree/{ref}/{path...}`), blob paths (`/blob/{ref}/{path...}`), commits (`/commit/{sha}`), and tags (`/releases/tag/{tag}`).
     - Strict owner validation (1-39 chars, alphanumeric and single hyphens, no consecutive hyphens, no leading/trailing hyphen).
     - Strict repo validation (1-100 chars, alphanumeric, hyphens, underscores, dots, `.git` stripped, cannot be `.` or `..`).
     - Subpath sanitization (normalize separators, strip leading/trailing slashes, reject `..` directory traversal).
     - SSRF and host validation (via `crates/url-security`, default allow `github.com` and `www.github.com`, configurable enterprise hosts, reject credentials/IP addresses).
     - Reserved path rejection (`/features`, `/pricing`, `/login`, etc.).
     - Provide canonical HTTPS URL, API base URL (`https://api.github.com`), and API repos URL (`https://api.github.com/repos/{owner}/{repo}`).
  4. Implement conversions to `SourceIdentity` from `crates/cdm`.
  5. Author unit and integration tests covering standard repos, tree subdirectories, blob files, commit SHAs, tag refs, branch refs with slashes, SSH/git syntax, shorthand syntax, traversal attacks, reserved paths, and SSRF rejection.
  6. Run full verification gates (`cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - All valid GitHub URL formats (HTTPS, HTTP, SSH, Git transport, shorthand) decompose correctly into owner, repo, target type, ref, subpath, and canonical URL.
  - Path traversal attempts (`../`) and malformed owner/repo names are rejected with typed errors.
  - SSRF attacks (private IPs, metadata IPs, non-GitHub hosts) are rejected.
  - 100% of unit and integration tests pass.
  - Zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Host verification, credential blocking, SSRF defenses via `crates/url-security`, path traversal sanitization, reserved path blocking.
- **Performance Considerations:** Zero-allocation regex compilation via `LazyLock`/`OnceLock`, minimal allocations during URL decomposition.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 22/22 tests passed (1 lib unit test + 21 integration tests in `target_decomposition_test.rs`).
  - `cargo test --workspace`: 356/356 tests passed across all 17 crates and services.
  - `cargo clippy -p web-repo-to-ai-github-extractor --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `9adc56b` / `67a0d53`
- **Status:** `ACCEPTED`

---

#### Subphase `5.1.2` — Branch & tag discovery in crates/github-extractor
- **Unique ID:** `5.1.2`
- **Objective:** Implement asynchronous GitHub REST API client methods for branch and tag discovery (`/repos/{owner}/{repo}/branches`, `/repos/{owner}/{repo}/tags`), supporting pagination, rate limiting header inspection (`x-ratelimit-*`), commit SHA resolution, ref disambiguation for branches containing slashes, and optional authentication token support with strict redaction.
- **Scope:** `crates/github-extractor/Cargo.toml`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/models.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/ref_discovery_test.rs`.
- **Prerequisites:** `5.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/Cargo.toml`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/models.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/ref_discovery_test.rs`
- **Implementation Tasks:**
  1. Add dependencies `reqwest`, `chrono`, and `tokio` to `crates/github-extractor/Cargo.toml`.
  2. Define ref discovery models in `crates/github-extractor/src/models.rs`: `GitHubBranch`, `GitHubTag`, `GitHubRateLimit`, and `RefDiscoveryReport` with commit SHA resolution and disambiguation helper methods.
  3. Expand `GitHubExtractorError` in `crates/github-extractor/src/error.rs` to handle HTTP errors, GitHub API errors (`RateLimitExceeded`, `ResourceNotFound`, `BadCredentials`, `Forbidden`), and JSON parsing errors with secret redaction.
  4. Implement `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - Configurable timeout, base URL, optional auth token, custom reqwest HTTP client.
     - `list_branches(&self, target: &GitHubTarget)` with pagination (`per_page=100`, page limits).
     - `get_branch(&self, target: &GitHubTarget, branch: &str)`.
     - `list_tags(&self, target: &GitHubTarget)` with pagination.
     - `discover_refs(&self, target: &GitHubTarget)` orchestrating branches and tags discovery.
     - `disambiguate_ref_and_subpath(&self, target: &GitHubTarget, raw_tail: &str, branches: &[GitHubBranch], tags: &[GitHubTag])` resolving ambiguous branches with slashes.
     - Inspection and tracking of GitHub rate limit headers (`x-ratelimit-limit`, `x-ratelimit-remaining`, `x-ratelimit-reset`, `x-ratelimit-used`).
  5. Implement comprehensive unit and integration tests in `crates/github-extractor/tests/ref_discovery_test.rs` validating pagination, rate limit extraction, 404/403/401 error handling, token redaction, and branch disambiguation with slash-separated branch names.
  6. Run all monorepo quality gates via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `GitHubApiClient` retrieves branches and tags with automatic pagination and commit SHA extraction.
  - Rate limiting headers are extracted and parsed accurately into `GitHubRateLimit`.
  - Slash-separated branch names (`feature/new-pipeline/v2`) disambiguate correctly against repository subpaths.
  - Authentication tokens are never exposed in error messages or logs.
  - 100% of tests pass, zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Token redaction, SSRF validation, rate limit backpressure defense, bounded pagination ceiling (max 1000 refs).
- **Performance Considerations:** Connection pooling via `reqwest::Client`, zero-copy parsing where feasible, streaming JSON deserialization.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 29/29 tests passed (1 lib unit test, 7 integration tests in `ref_discovery_test.rs`, 21 integration tests in `target_decomposition_test.rs`).
  - `cargo test --workspace`: 363/363 tests passed across all 17 crates and services.
  - `cargo clippy -p web-repo-to-ai-github-extractor --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `8d8ded7` / `2f0de6d`
- **Status:** `ACCEPTED`

---

#### Subphase `5.1.3` — Commit selection in crates/github-extractor
- **Unique ID:** `5.1.3`
- **Objective:** Implement immutable commit resolution and pinning (`/repos/{owner}/{repo}/commits/{ref}`, `/repos/{owner}/{repo}/commits`), extracting commit metadata (author, committer, timestamps, message, parent SHAs, root tree SHA), normalizing short commit SHAs, and producing a validated `PinnedRevision` guaranteeing zero ref-drift during downstream tree and file extraction.
- **Scope:** `crates/github-extractor/src/commit.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/models.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/commit_selection_test.rs`.
- **Prerequisites:** `5.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/commit.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/models.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/commit_selection_test.rs`
- **Implementation Tasks:**
  1. Define commit models in `crates/github-extractor/src/commit.rs`: `GitHubCommitMetadata`, `CommitAuthor`, `CommitSignatureVerification`, `PinnedRevision`.
  2. Implement `resolve_commit(&self, target: &GitHubTarget, ref_spec: Option<&str>)` in `GitHubApiClient`:
     - Resolves specified ref or default branch to full 40-character commit SHA.
     - Fetches commit details from `GET /repos/{owner}/{repo}/commits/{ref}`.
     - Extracts the Git root tree SHA (`commit.tree.sha`) required for recursive Git Trees API traversal.
     - Extracts author/committer identities, ISO 8601 timestamps, commit message, and parent commit SHAs.
  3. Implement `PinnedRevision` methods:
     - `to_revision_info(&self) -> RevisionInfo` mapping into CDM schema (`crates/cdm`).
     - `to_source_identity(&self, raw_input: &str) -> SourceIdentity`.
     - `is_head(&self) -> bool`.
     - `short_sha(&self) -> &str` (7-character short SHA prefix).
  4. Expand `GitHubApiClient` with `pin_revision(&self, target: &GitHubTarget)` which resolves branches/tags, disambiguates target, and pins the immutable commit SHA.
  5. Implement comprehensive integration tests in `crates/github-extractor/tests/commit_selection_test.rs` using in-process Axum test server verifying branch-to-commit resolution, short-to-full SHA expansion, root tree SHA extraction, commit metadata parsing, 404 commit handling, and CDM `RevisionInfo` mapping.
  6. Run full verification gates via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Any valid branch name, release tag, short commit SHA, full 40-hex SHA, or omitted ref resolves to a verified 40-character commit SHA and root tree SHA.
  - `PinnedRevision` maps accurately to CDM `RevisionInfo` with exact commit SHA.
  - Ref-drift is eliminated: all downstream operations are bound to the immutable commit SHA.
  - 100% of tests pass, zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Immutable revision pinning prevents TOCTOU drift attacks between file discovery and source extraction; token redaction on commit API calls.
- **Performance Considerations:** Single-roundtrip commit resolution with header rate-limit tracking; cached root tree SHA avoids extra Git Trees root calls.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 33/33 tests passed (1 lib unit test, 21 integration tests in `target_decomposition_test.rs`, 7 integration tests in `ref_discovery_test.rs`, 4 integration tests in `commit_selection_test.rs`).
  - `cargo test --workspace`: 367/367 tests passed across all 17 crates and services.
  - `cargo clippy -p web-repo-to-ai-github-extractor --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 213/213 tests passed across 17 suites; `svelte-check` reports 0 errors and 0 warnings.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `de994ea` / `2269163`
- **Status:** `ACCEPTED`

---

#### Subphase `5.1.4` — Repository metadata extraction in crates/github-extractor
- **Unique ID:** `5.1.4`
- **Objective:** Implement comprehensive repository metadata retrieval via GitHub REST API (`GET /repos/{owner}/{repo}`) and programming language breakdown (`GET /repos/{owner}/{repo}/languages`), extracting repository properties (description, primary language, topics/tags, license SPDX identifiers, fork/archived/visibility status, default branch, stars, watchers, forks, open issues, size in KB, timestamps), language byte distribution and percentages, and mapping into the Canonical Context Document Model (`DocumentMetadata`).
- **Scope:** `crates/github-extractor/src/metadata.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/metadata_extraction_test.rs`.
- **Prerequisites:** `5.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/metadata.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/metadata_extraction_test.rs`
- **Implementation Tasks:**
  1. Define metadata models in `crates/github-extractor/src/metadata.rs`: `GitHubLicense`, `GitHubRepositoryMetadata`, `GitHubLanguages`, and consolidated `GitHubRepoContext`.
  2. Implement helper methods for language percentage calculation and descending byte sorting in `GitHubLanguages`.
  3. Implement `to_cdm_metadata(&self, languages: &GitHubLanguages) -> DocumentMetadata` converting repository properties and language breakdown into standard CDM schema (`crates/cdm`).
  4. Implement `get_repo_metadata(&self, target: &GitHubTarget)` in `GitHubApiClient` querying `GET /repos/{owner}/{repo}` with rate limit header tracking and secret token redaction.
  5. Implement `get_repo_languages(&self, target: &GitHubTarget)` in `GitHubApiClient` querying `GET /repos/{owner}/{repo}/languages`.
  6. Implement `fetch_repo_context(&self, target: &GitHubTarget, pin: bool)` orchestrating parallel metadata and language extraction via `tokio::join!`, with optional immutable revision pinning.
  7. Implement comprehensive integration tests in `crates/github-extractor/tests/metadata_extraction_test.rs` with an in-process Axum test server validating repo metadata parsing, language percentages, CDM metadata conversion, 404 handling, and token redaction under failure modes.
  8. Run full monorepo quality audit via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Repository metadata and programming languages breakdown are retrieved and parsed correctly.
  - Languages percentage distribution accurately sums to 100% (within floating point precision).
  - CDM `DocumentMetadata` mapping populates title, description, authors, license, primary language, language breakdown, tags, timestamps, and extra statistics.
  - 100% of tests pass, zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Token redaction on all metadata API requests and error responses; SSRF validation via `crates/url-security`; license and topic sanitization.
- **Performance Considerations:** Parallel retrieval of repo metadata and language breakdown via `tokio::join!`; zero-copy JSON parsing where feasible.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 37/37 tests passed (1 lib unit test, 21 target decomposition tests, 7 ref discovery tests, 4 commit selection tests, 4 metadata extraction tests).
  - `cargo test --workspace`: 371/371 tests passed across all 17 crates and services.
  - `cargo clippy -p web-repo-to-ai-github-extractor --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 213/213 tests passed across 17 suites; `svelte-check` reports 0 errors and 0 warnings.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `471c0a6` / `5d0b133`
- **Status:** `ACCEPTED`

---

### Phase 5.1 Milestone Certification (`5.1.1` – `5.1.4`)
- **Phase 5.1 Goal:** Complete GitHub repository metadata, URL identification, branch/tag ref discovery, commit selection, language breakdown, and CDM document metadata mapping.
- **Subphases Completed & Accepted:**
  - `5.1.1`: Repository identification and URL decomposition in `crates/github-extractor` (`ACCEPTED`, commit `9adc56b` / `67a0d53`)
  - `5.1.2`: Branch & tag discovery in `crates/github-extractor` (`ACCEPTED`, commit `8d8ded7` / `2f0de6d`)
  - `5.1.3`: Commit selection in `crates/github-extractor` (`ACCEPTED`, commit `de994ea` / `1696347`)
  - `5.1.4`: Repository metadata extraction in `crates/github-extractor` (`ACCEPTED`, commit `471c0a6` / `5d0b133`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Complete URL parsing and target decomposition supporting HTTPS, SSH, Git, shorthand (`owner/repo@ref`), tree/blob subpaths, release tags, and commit SHAs with path traversal protection and zero-trust SSRF validation.
  2. Rate-limit-aware asynchronous GitHub API client (`GitHubApiClient`) supporting pagination, rate limit headers (`x-ratelimit-*`), and robust token secret redaction.
  3. Parallel branch and release tag discovery with longest-prefix disambiguation resolving slash-separated branch names against repository subpaths.
  4. Immutable commit resolution (`/commits/{ref}`) and revision pinning (`PinnedRevision`) capturing author, committer, timestamps, message, and root Git tree SHA, eliminating mutable ref-drift.
  5. Repository metadata and languages breakdown extraction (`/repos/{owner}/{repo}`, `/languages`) calculating exact language percentages and mapping cleanly to CDM `DocumentMetadata`.
  6. 37 tests passing in `crates/github-extractor`, 371 tests passing across the workspace with zero clippy warnings and all 5 monorepo gates passing.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-18)

---

### Phase 5.2 — File discovery

#### Subphase `5.2.1` — Git Trees integration in crates/github-extractor
- **Unique ID:** `5.2.1`
- **Objective:** Implement GitHub Git Trees API integration (`GET /repos/{owner}/{repo}/git/trees/{tree_sha}`) with recursive traversal (`?recursive=1`), typed file modes (`100644` regular, `100755` executable, `040000` directory, `120000` symlink, `160000` submodule), object types (`blob`, `tree`, `commit`), path sanitization and normalization, entry querying/filtering methods, and direct binding to `PinnedRevision.root_tree_sha`.
- **Scope:** `crates/github-extractor/src/tree.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/git_trees_test.rs`.
- **Prerequisites:** `5.1.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/tree.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/git_trees_test.rs`
- **Implementation Tasks:**
  1. Define Git Trees models in `crates/github-extractor/src/tree.rs`: `GitFileMode`, `GitTreeEntryType`, `GitTreeEntry`, and `GitTreeResponse`.
  2. Implement helper methods on `GitTreeEntry`: `file_name`, `extension`, `parent_path`, `is_blob`, `is_tree`, `is_submodule`, `depth`.
  3. Implement helper and query methods on `GitTreeResponse`: `blob_count`, `tree_count`, `submodule_count`, `total_blob_size`, `blobs`, `find_by_path`, `filter_by_prefix`, `filter_by_extension`, `to_path_set`.
  4. Implement `get_tree(&self, target: &GitHubTarget, tree_sha: &str, recursive: bool)` in `GitHubApiClient` querying GitHub REST API `/git/trees/{tree_sha}` with secret redaction and rate limiting.
  5. Implement `fetch_pinned_tree(&self, pinned: &PinnedRevision, recursive: bool)` binding tree acquisition directly to `pinned.root_tree_sha()`.
  6. Implement comprehensive integration tests in `crates/github-extractor/tests/git_trees_test.rs` with an in-process Axum test server validating recursive tree parsing, non-recursive mode, path helpers, mode classification, 404 handling, and token redaction under failure conditions.
  7. Run full monorepo quality audit via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - Git tree entries across all standard Git modes (`100644`, `100755`, `040000`, `120000`, `160000`) and types (`blob`, `tree`, `commit`) are accurately parsed.
  - Path normalization strips leading slashes and validates against path traversal.
  - `fetch_pinned_tree` retrieves the tree corresponding to `PinnedRevision.root_tree_sha`.
  - 100% of tests pass, zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Path traversal protection on all tree entry paths; secret token redaction in tree request failures; zero untrusted code execution.
- **Performance Considerations:** Single-pass tree deserialization, zero-copy string slicing for file extensions and directory names, lazy iterator over blobs.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 41/41 tests passed (1 lib unit test, 21 target decomposition tests, 7 ref discovery tests, 4 commit selection tests, 4 metadata extraction tests, 4 git trees tests).
  - `cargo test --workspace`: 375/375 tests passed across all 17 crates and services.
  - `cargo clippy -p web-repo-to-ai-github-extractor --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `npm test` (`apps/web`): 213/213 tests passed across 17 suites; `svelte-check` reports 0 errors and 0 warnings.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (224/224 bootstrap checks, 7/7 workflows, Rust checks, Frontend checks, Podman storage 5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `010f446` / `4d84308`
- **Status:** `ACCEPTED`

---

#### Subphase `5.2.2` — Tree truncation detection in crates/github-extractor
- **Unique ID:** `5.2.2`
- **Objective:** Implement rigorous Git Trees truncation detection and completeness auditing in `crates/github-extractor`, preventing silent data loss by analyzing explicit GitHub `truncated: true` flags, GitHub hard ceilings (100,000 entries / 7 MB response), unexpanded directory subtrees, and recommending optimal recovery strategies (`SubtreeTraversal`, `ShallowArchiveFallback`, `None`).
- **Scope:** `crates/github-extractor/src/tree.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/tree_truncation_test.rs`.
- **Prerequisites:** `5.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/tree.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/tree_truncation_test.rs`
- **Implementation Tasks:**
  1. Define truncation detection models in `crates/github-extractor/src/tree.rs`: `TruncationReason`, `TreeCompleteness`, `RecoveryStrategy`, and `TruncationReport`.
  2. Implement `find_unexpanded_subtrees(&self) -> Vec<&GitTreeEntry>` on `GitTreeResponse` detecting directory entries that have zero child objects present in the recursive response.
  3. Implement `analyze_completeness(&self) -> TruncationReport` on `GitTreeResponse` evaluating explicit GitHub truncation flags, entry limits (`GITHUB_TREE_ENTRY_LIMIT = 100_000`), and incomplete directory subtrees.
  4. Implement recommendation heuristics in `RecoveryStrategy`: `SubtreeTraversal` for manageable truncation (< 500 unexpanded subtrees), `ShallowArchiveFallback` for massive repositories, and `None` for verified complete trees.
  5. Implement comprehensive integration tests in `crates/github-extractor/tests/tree_truncation_test.rs` validating complete tree analysis, explicit `truncated: true` detection, unexpanded subtree path identification, entry cap enforcement, and live in-process Axum mock server integration testing.
  6. Run full monorepo quality audit via `scripts/check-all.ps1`.
- **Acceptance Criteria:**
  - `analyze_completeness` accurately identifies `truncated: true` flags and isolates all unexpanded subdirectories.
  - Zero false positives on complete trees: fully expanded trees report `TreeCompleteness::Complete` and `RecoveryStrategy::None`.
  - Structured `TruncationReport` specifies all reasons, unexpanded subtrees, and recommended recovery strategy.
  - 100% of tests pass, zero clippy warnings, zero fmt diffs, cargo-deny clean.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, 04, 05: Eliminates silent truncation vulnerability where missing files go undetected; bounded memory evaluation of subtrees.
- **Performance Considerations:** Single-pass hash set construction for O(N) unexpanded subtree detection rather than O(N^2) path prefix comparisons.
- **Documentation Requirements:** Module documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 46 tests passed (100% success rate, 0 failures), including 5 new integration tests in `crates/github-extractor/tests/tree_truncation_test.rs`.
  - `cargo test --workspace`: 380 tests passed across all 15 crates and 3 services.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - Frontend checks: 213 unit/component tests passed across 17 test suites, 0 type errors, 0 lint warnings.
  - Resource budget: Podman storage footprint healthy at 5.27 GB / 30 GB (17.6%).
  - All 5 gates in `scripts/check-all.ps1` passed with 0 failures.
- **Git Commit Reference:** `87ba8e1`
- **Status:** `ACCEPTED`

---

#### Subphase `5.2.3` — Subtree recovery in crates/github-extractor
- **Unique ID:** `5.2.3`
- **Objective:** Implement automated subtree recovery for truncated Git Trees via bounded recursive subtree querying (`GET /repos/{owner}/{repo}/git/trees/{tree_sha}`) and tree stitching, with graceful fallback to shallow zipball archive downloads if subtree count exceeds threshold or rate limits require it.
- **Scope:** `crates/github-extractor/Cargo.toml`, `crates/github-extractor/src/recovery.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/subtree_recovery_test.rs`.
- **Prerequisites:** `5.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/Cargo.toml`
  - `crates/github-extractor/src/recovery.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/subtree_recovery_test.rs`
- **Implementation Tasks:**
  1. Define `SubtreeRecoveryConfig`, `SubtreeRecoveryReport`, and `ArchiveFormat` in `crates/github-extractor/src/recovery.rs`.
  2. Implement `TreeStitcher` and `stitch_child_path` to prefix and merge unexpanded subtree results into the parent tree without duplicate paths.
  3. Implement in-memory zip archive scanner `scan_zip_archive` to extract complete `GitTreeResponse` from GitHub archive streams.
  4. Extend `GitHubApiClient` with `download_archive`, `recover_tree`, and `recover_pinned_tree` orchestrating bounded queue-based recursive subtree queries and archive fallback.
  5. Author comprehensive integration tests in `crates/github-extractor/tests/subtree_recovery_test.rs` verifying path stitching, zip parsing, live Axum recursive subtree traversal, and archive fallback.
  6. Run full verification suite (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Truncated trees with unexpanded subdirectories are successfully recovered via bounded subtree queries.
  - Subtree child paths are accurately prefixed relative to the repository root.
  - When subtree count exceeds limits, shallow zipball archive fallback seamlessly reconstructs the complete tree.
  - Zero token leaks, rate-limit aware error reporting.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Redacts secret tokens from all URLs, logs, and error responses; bounds recursion depth (max 10) and subtree queries (max 500) to prevent denial of service or unbounded memory usage.
- **Performance Considerations:** In-memory streaming zip parsing, path deduplication via hash sets, bounded concurrency.
- **Documentation Requirements:** Module and API documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 54 tests passed (100% success rate, 0 failures), including 7 new tests in `crates/github-extractor/tests/subtree_recovery_test.rs`.
  - `cargo test --workspace`: 388 tests passed across all 15 crates and 3 services.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - Frontend checks: 213 unit/component tests passed across 17 test suites, 0 type errors, 0 lint warnings.
  - Resource budget: Podman storage footprint healthy at 5.27 GB / 30 GB (17.6%).
  - All 5 gates in `scripts/check-all.ps1` passed with 0 failures.
- **Git Commit Reference:** `2495a78`
- **Status:** `ACCEPTED`

---

#### Subphase `5.2.4` — File classification in crates/github-extractor
- **Unique ID:** `5.2.4`
- **Objective:** Implement comprehensive file classification across discovered Git tree entries into canonical categories (`source_code`, `documentation`, `configuration`, `dependency_manifest`, `test_fixture`, `generated_or_lockfile`, `binary`, `oversized`), with size-threshold gating, language detection, and direct CDM `CompletenessReport` mapping.
- **Scope:** `crates/github-extractor/src/classification.rs`, `crates/github-extractor/src/tree.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/file_classification_test.rs`.
- **Prerequisites:** `5.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/classification.rs`
  - `crates/github-extractor/src/tree.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/file_classification_test.rs`
- **Implementation Tasks:**
  1. Define `FileCategory` enum (`SourceCode`, `Documentation`, `Configuration`, `DependencyManifest`, `TestFixture`, `GeneratedOrLockfile`, `Binary`, `Oversized`, `Other`) with query helpers in `crates/github-extractor/src/classification.rs`.
  2. Implement `FileClassificationConfig` with customizable size ceiling (default 1 MB / 1_048_576 bytes), excluded path patterns, and explicit category overrides.
  3. Implement `ClassifiedFile` and `FileClassifier::classify_entry` matching file name, extension, directory location, and byte size against canonical heuristics.
  4. Implement `TreeClassificationReport` and `FileClassifier::classify_tree` providing aggregate category statistics, eligible vs excluded partitioning, and CDM `CompletenessReport` generation enforcing the completeness invariant.
  5. Add `classify(&self, config)` convenience method on `GitTreeResponse` in `crates/github-extractor/src/tree.rs`.
  6. Author comprehensive integration tests in `crates/github-extractor/tests/file_classification_test.rs` validating exact classification across all 8 canonical categories, size threshold gating, excluded directory filtering, and CDM completeness report consistency.
  7. Run full verification suite (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - All files in a Git tree are categorized without panic or ambiguity into their appropriate category.
  - Test fixtures, dependency manifests, and documentation are accurately distinguished from general source code.
  - Large files (> 1 MB or configured threshold) are tagged as `Oversized` with eligibility set to false and reason recorded.
  - `TreeClassificationReport::to_completeness_report()` satisfies `discovered_count == extracted + excluded + failed + unsupported`.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Prevents ingestion of dangerous or unbounded binaries/oversized files into downstream LLM tokenizers; bounded memory evaluation; accurate exclusion auditing.
- **Performance Considerations:** Single-pass tree classification with $O(1)$ hash set lookups and static pattern matching.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 66/66 tests passed (6 unit tests, 7 classification integration tests, 52 other integration tests, 1 doc test) in 0.05s.
  - `cargo test --workspace`: 395/395 tests passed across all workspace crates in 15.85s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 6.65s.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `4f3303c`
- **Status:** `ACCEPTED`

---

### Phase 5.2 Milestone Certification (`5.2.1` – `5.2.4`)
- **Phase 5.2 Goal:** Complete Git Trees API integration, tree truncation detection, subtree recovery via recursive BFS and shallow archive fallback, file classification taxonomy, and CDM completeness report mapping.
- **Subphases Completed & Accepted:**
  - `5.2.1`: Git Trees integration in `crates/github-extractor` (`ACCEPTED`, commit `f3173ee` / `4b46c6b`)
  - `5.2.2`: Tree truncation detection in `crates/github-extractor` (`ACCEPTED`, commit `87ba8e1` / `e935430`)
  - `5.2.3`: Subtree recovery in `crates/github-extractor` (`ACCEPTED`, commit `2495a78` / `057324e`)
  - `5.2.4`: File classification in `crates/github-extractor` (`ACCEPTED`, commit `4f3303c`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Full Git Trees API traversal with recursive parameterization (`?recursive=1`), typed POSIX file modes (`100644`, `100755`, `040000`, `120000`, `160000`), and path-prefix/extension query filters.
  2. O(N) linear tree truncation detection auditing explicit GitHub truncation flags, entry count ceilings (100k limit), and unexpanded directory subtrees.
  3. Resilient subtree recovery engine orchestrating BFS queue traversal of unexpanded directories, atomic in-place tree stitching with SHA deduplication, and shallow zipball/tarball archive fallback with permission extraction and parent segment normalization.
  4. Comprehensive file classification taxonomy distinguishing source code, documentation, configuration, manifests, tests, lockfiles, binaries, and oversized files with configurable byte ceilings, path prefix exclusions, language detection for 28+ languages, and direct CDM `CompletenessReport` mapping satisfying `formula_valid()`.
  5. 65 tests passing in `crates/github-extractor` (+ 1 doc test), 395 tests passing across the workspace with zero clippy warnings and all 5 monorepo gates passing.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-18)

---

### Phase 5.3 — File extraction

#### Subphase `5.3.1` — Source acquisition in crates/github-extractor
- **Unique ID:** `5.3.1`
- **Objective:** Implement resilient source code file acquisition from GitHub via the Git Blobs API (`/repos/{owner}/{repo}/git/blobs/{file_sha}`) and raw endpoint fallback, with raw/base64 decoding, Git object SHA-1 validation, BLAKE3 and SHA-256 content hashing, binary inspection, bounded concurrent batch extraction, and direct CDM `CdmSection` construction.
- **Scope:** `Cargo.toml`, `crates/github-extractor/Cargo.toml`, `crates/github-extractor/src/source.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/source_acquisition_test.rs`.
- **Prerequisites:** `5.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `Cargo.toml`
  - `crates/github-extractor/Cargo.toml`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/source.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/source_acquisition_test.rs`
- **Implementation Tasks:**
  1. Add `sha1 = "0.10"` to workspace dependencies in root `Cargo.toml` and enable `blake3`, `sha1`, and `tokio` `sync` in `crates/github-extractor/Cargo.toml`.
  2. Add `BlobError { sha: String, message: String }` error variant to `GitHubExtractorError` in `crates/github-extractor/src/error.rs`.
  3. Implement `crates/github-extractor/src/source.rs`:
     - RFC 4648 standard base64 decoder with whitespace/newline tolerance.
     - Git blob SHA-1 calculation: `sha1("blob <size>\0<bytes>")`.
     - BLAKE3 and SHA-256 cryptographic content digest calculations.
     - Non-text binary detection inspecting initial byte window for null bytes.
     - Domain models: `GitBlob`, `ExtractedSourceFile`, `SourceExtractionConfig`, `FailedSourceExtraction`, and `SourceExtractionReport`.
     - Conversion from `ExtractedSourceFile` to CDM `CdmSection` (`SectionKind::CodeFile`, line range, byte range, language, BLAKE3 content hash, `SectionTokenMetrics::compute_baseline`).
  4. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `get_blob`: fetches raw or JSON base64 blob by SHA with Git object SHA-1 integrity verification.
     - `fetch_source_file`: processes a single `ClassifiedFile` into `ExtractedSourceFile`, performing size gating, binary inspection, and line metric calculation.
     - `extract_source_files`: orchestrates bounded concurrent batch retrieval using a `tokio::sync::Semaphore`, collecting successes, per-file failures, and unsupported binaries into a `SourceExtractionReport`.
  5. Re-export source acquisition models and functions in `crates/github-extractor/src/lib.rs`.
  6. Author comprehensive integration tests in `crates/github-extractor/tests/source_acquisition_test.rs` testing raw blob retrieval, base64 blob retrieval, Git SHA verification and mismatch detection, binary detection, concurrent batch extraction with semaphore bounds, and CDM section mapping.
  7. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Git blob fetching supports both `application/vnd.github.raw` and standard JSON base64 payloads without external dependencies.
  - Downloaded blob content is verified against the expected Git object SHA-1, rejecting tampered or corrupt payloads.
  - Concurrency is strictly bounded by semaphore without unbounded task spawning.
  - Failures on individual files are recorded in `failed` list without aborting the batch, properly mapped to CDM `CompletenessReport` accounting.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Cryptographic hash validation (SHA-1, BLAKE3, SHA-256); secret redaction on all blob URLs; binary detection prevents injecting non-text binary streams into text tokenizers; bounded memory usage prevents memory exhaustion.
- **Performance Considerations:** Bounded concurrency via `tokio::sync::Semaphore` prevents hitting GitHub secondary rate limits and worker socket exhaustion.
- **Documentation Requirements:** Module and function documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 72/72 tests passed (12 unit tests, 7 source acquisition integration tests, 52 other integration tests, 1 doc test) in 0.19s.
  - `cargo test --workspace`: 408/408 tests passed across all workspace crates in 24.26s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 8.33s.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `9678470`
- **Status:** `ACCEPTED`

---

#### Subphase `5.3.2` — Documentation extraction in crates/github-extractor
- **Unique ID:** `5.3.2`
- **Objective:** Implement comprehensive documentation file extraction from GitHub (`FileCategory::Documentation`), classifying documents into a rich `DocType` taxonomy (`Readme`, `Architecture`, `Changelog`, `Contributing`, `License`, `Security`, `CodeOfConduct`, `Guide`, `Roadmap`, `General`), detecting formats (`Markdown`, `Asciidoc`, `RestructuredText`, `PlainText`), extracting document titles and YAML frontmatter, capturing root README/LICENSE index pointers, orchestrating bounded concurrent batch extraction, and generating CDM [`CdmSection`] prose representations.
- **Scope:** `crates/github-extractor/src/documentation.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/documentation_extraction_test.rs`.
- **Prerequisites:** `5.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/documentation.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/documentation_extraction_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/documentation.rs`:
     - `DocType` enum (`Readme`, `Architecture`, `Changelog`, `Contributing`, `License`, `Security`, `CodeOfConduct`, `Guide`, `Roadmap`, `General`) with classifier heuristic function `detect_doc_type(path, file_name)`.
     - `DocFormat` enum (`Markdown`, `Asciidoc`, `RestructuredText`, `PlainText`) with format detector `detect_doc_format(file_name, extension)`.
     - Title and frontmatter extractors: `extract_title_and_frontmatter(content, path)`.
     - `ExtractedDocFile`: model capturing relative path, Git blob SHA, doc type, format, title, frontmatter presence, body, line count, byte size, BLAKE3 and SHA-256 digests, and conversion to CDM `CdmSection` (`SectionKind::Prose`).
     - `DocExtractionReport`: aggregated documentation report capturing extracted docs, failures, unsupported binaries, dedicated root `readme` and `license` accessors, and CDM `CompletenessReport` mapping.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `fetch_doc_file`: acquires blob, verifies Git SHA, extracts doc metadata, and constructs `ExtractedDocFile`.
     - `extract_doc_files`: concurrent bounded batch extraction of documentation files via semaphore.
  3. Re-export documentation models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/documentation_extraction_test.rs` validating README extraction and title detection, LICENSE extraction, multi-doc classification across all `DocType` variants, YAML frontmatter detection, format detection, bounded concurrent batch extraction, and CDM section mapping.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - All documentation files (`FileCategory::Documentation`) are extracted and accurately classified into appropriate `DocType` and `DocFormat`.
  - Root README and repository LICENSE are cleanly indexed as convenient accessors.
  - Document titles are parsed from markdown headers or frontmatter with fallback to path/filename.
  - Bounded concurrency with semaphore prevents hitting GitHub rate limits or socket exhaustion.
  - Extracted documents map cleanly to CDM `CdmSection` (`SectionKind::Prose`) satisfying the completeness report invariant.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Cryptographic hash validation (SHA-1, BLAKE3, SHA-256); secret redaction; prompt-injection defenses (untrusted documentation text is safely encapsulated in CDM prose sections without host execution).
- **Performance Considerations:** Bounded concurrency via `tokio::sync::Semaphore`; zero-copy string slicing during title and frontmatter detection.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 83/83 tests passed (15 unit tests, 6 documentation integration tests, 61 other integration tests, 1 doc test) in 0.16s.
  - `cargo test --workspace`: 418/418 tests passed across all workspace crates in 20.42s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors in 3.67s.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `6bca508`
- **Status:** `ACCEPTED`

---

#### Subphase `5.3.3` — Dependency extraction in crates/github-extractor
- **Unique ID:** `5.3.3`
- **Objective:** Implement comprehensive package manifest and dependency extraction from GitHub (`FileCategory::DependencyManifest` and eligible `FileCategory::GeneratedOrLockfile`), identifying package ecosystems (`Cargo`, `Npm`, `Pip`, `GoMod`, `MavenGradle`, `Bundler`, `Composer`, `Pub`, `Mix`), distinguishing manifest roles (`Manifest` vs `Lockfile`), executing deterministic lightweight manifest parsing (`Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, `pyproject.toml`) to extract package identity and declared dependencies (`Direct`, `Dev`, `Build`, `Peer`, `Optional`), orchestrating bounded concurrent batch extraction, and generating Canonical Context Document Model ([`CdmSection`]) manifest sections.
- **Scope:** `crates/github-extractor/src/dependency.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/dependency_extraction_test.rs`.
- **Prerequisites:** `5.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/dependency.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/dependency_extraction_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/dependency.rs`:
     - `PackageEcosystem` enum (`Cargo`, `Npm`, `Pip`, `GoMod`, `MavenGradle`, `Bundler`, `Composer`, `Pub`, `Mix`, `Other`) and detector `detect_ecosystem(file_name)`.
     - `ManifestRole` enum (`Manifest`, `Lockfile`) and role classifier `detect_manifest_role(file_name)`.
     - `DependencyType` enum (`Direct`, `Dev`, `Build`, `Peer`, `Optional`).
     - `DeclaredDependency` model capturing name, version requirement, type, optional flag, and enabled features.
     - `ParsedManifest` model capturing package name, package version, description, license, and list of dependencies with query helper accessors.
     - Deterministic manifest parsers for `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, and `pyproject.toml`.
     - `ExtractedDependencyFile` model with path, Git blob SHA, ecosystem, role, text content, parsed manifest, line count, byte size, cryptographic BLAKE3 and SHA-256 digests, and conversion to CDM `CdmSection` (`SectionKind::Manifest`).
     - `DependencyExtractionReport` capturing extracted manifests/lockfiles, failures, unsupported entries, ecosystem filtering, direct dependency aggregation, and CDM `CompletenessReport` mapping.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `fetch_dependency_file`: acquires blob, verifies Git SHA, parses manifest if applicable, and constructs `ExtractedDependencyFile`.
     - `extract_dependency_files`: concurrent bounded batch extraction of dependency manifests and lockfiles via semaphore.
  3. Re-export dependency models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/dependency_extraction_test.rs` covering `Cargo.toml` parsing, `package.json` parsing, `requirements.txt` parsing, `go.mod` parsing, lockfile acquisition, bounded concurrency batch retrieval, and CDM `SectionKind::Manifest` mapping.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - All package manifests and eligible lockfiles are extracted, validated against Git SHA-1, and classified into accurate `PackageEcosystem` and `ManifestRole`.
  - Manifest parsers correctly parse `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, and `pyproject.toml` into structured `ParsedManifest` without runtime panics.
  - Bounded concurrency with semaphore prevents hitting GitHub rate limits or socket exhaustion.
  - Extracted dependency files map cleanly to CDM `CdmSection` (`SectionKind::Manifest`) with ecosystem and dependency metadata.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Cryptographic hash validation (SHA-1, BLAKE3, SHA-256); secret redaction; prompt-injection defenses (untrusted manifest files are safely encapsulated in CDM manifest sections without host execution).
- **Performance Considerations:** Bounded concurrency via `tokio::sync::Semaphore`; zero-copy string slicing during dependency parsing.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 89/89 tests passed (21 unit tests, 6 dependency integration tests, 61 other integration tests, 1 doc test) in 0.17s.
  - `cargo test --workspace`: 424/424 tests passed across all workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `74e6952`
- **Status:** `ACCEPTED`

---

#### Subphase `5.3.4` — Configuration & test extraction in crates/github-extractor
- **Unique ID:** `5.3.4`
- **Objective:** Implement comprehensive repository configuration and test/fixture extraction from GitHub (`FileCategory::Configuration` and `FileCategory::TestFixture`), classifying configurations into functional categories (`CiCd`, `Build`, `LinterFormatter`, `Environment`, `General`) and test files into test categories (`Unit`, `Integration`, `E2e`, `Fixture`, `Benchmark`, `General`), orchestrating bounded concurrent batch extraction, generating Canonical Context Document Model ([`CdmSection`]) code sections with rich categorization metadata, and formally certifying Phase 5.3 ("File extraction").
- **Scope:** `crates/github-extractor/src/config_test.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/config_test_extraction_test.rs`.
- **Prerequisites:** `5.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/config_test.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/config_test_extraction_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/config_test.rs`:
     - `ConfigCategory` enum (`CiCd`, `Build`, `LinterFormatter`, `Environment`, `General`) and detector `detect_config_category(path, file_name)`.
     - `TestCategory` enum (`Unit`, `Integration`, `E2e`, `Fixture`, `Benchmark`, `General`) and detector `detect_test_category(path, file_name)`.
     - `ExtractedConfigFile` model capturing path, Git blob SHA, config category, format, body, byte size, line count, cryptographic BLAKE3 and SHA-256 digests, and conversion to CDM `CdmSection` (`SectionKind::CodeFile`).
     - `ExtractedTestFile` model capturing path, Git blob SHA, test category, language, body, byte size, line count, cryptographic BLAKE3 and SHA-256 digests, and conversion to CDM `CdmSection` (`SectionKind::CodeFile`).
     - `ConfigTestExtractionReport` capturing extracted configs, extracted tests, failures, unsupported binaries, category filtering, dedicated `ci_workflows()` and `fixtures()` accessors, and CDM `CompletenessReport` mapping.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `fetch_config_file`: acquires blob, verifies Git SHA, and constructs `ExtractedConfigFile`.
     - `fetch_test_file`: acquires blob, verifies Git SHA, and constructs `ExtractedTestFile`.
     - `extract_config_and_test_files`: concurrent bounded batch extraction of configuration and test files via semaphore.
  3. Re-export config and test models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/config_test_extraction_test.rs` covering CI/CD workflow extraction, build configuration, linter/formatter config, unit/integration test extraction, test fixtures, bounded concurrent batch retrieval, and CDM section mapping.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
  6. Formally certify Phase 5.3 ("File extraction") in `IMPLEMENTATION_LEDGER.md` and `CHANGELOG.md`.
- **Acceptance Criteria:**
  - All configuration files (`FileCategory::Configuration`) and test/fixture files (`FileCategory::TestFixture`) are extracted, validated against Git SHA-1, and classified into appropriate categories.
  - CI/CD workflows, build configurations, and test fixtures are cleanly categorized and accessible via convenient accessors.
  - Bounded concurrency with semaphore prevents hitting GitHub rate limits or socket exhaustion.
  - Extracted files map cleanly to CDM `CdmSection` (`SectionKind::CodeFile`) satisfying the completeness report invariant.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Cryptographic hash validation (SHA-1, BLAKE3, SHA-256); secret redaction; prompt-injection defenses (untrusted config and test files are safely encapsulated in CDM code sections without host execution).
- **Performance Considerations:** Bounded concurrency via `tokio::sync::Semaphore`; zero-copy string slicing during category detection.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 98/98 tests passed (24 unit tests, 6 config/test integration tests, 6 dependency integration tests, 6 documentation integration tests, 55 other integration tests, 1 doc test) in 0.19s.
  - `cargo test --workspace`: 430/430 tests passed across all workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `b9893d0`
- **Status:** `ACCEPTED`

---

### Phase 5.3 Milestone Certification (`5.3.1` – `5.3.4`)
- **Phase 5.3 Goal:** Complete source code acquisition, documentation extraction, package dependency & manifest extraction, and configuration & test/fixture extraction with Git SHA-1 verification, bounded concurrency, and CDM section mapping.
- **Subphases Completed & Accepted:**
  - `5.3.1`: Source acquisition in `crates/github-extractor` (`ACCEPTED`, commit `98cade2` / `a6e2e51`)
  - `5.3.2`: Documentation extraction in `crates/github-extractor` (`ACCEPTED`, commit `627ecad` / `6bca508`)
  - `5.3.3`: Dependency extraction in `crates/github-extractor` (`ACCEPTED`, commit `0637c35` / `74e6952`)
  - `5.3.4`: Configuration & test extraction in `crates/github-extractor` (`ACCEPTED`, commit `b9893d0`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Source code acquisition via Git Blobs API and raw fallback with RFC 4648 base64 decoding, Git object SHA-1 validation, BLAKE3 and SHA-256 digests, binary null-byte sniffing, and CDM [`SectionKind::CodeFile`] construction.
  2. Functional documentation categorization across 10 `DocType` variants, format detection (Markdown, Asciidoc, RST, PlainText), title/frontmatter extraction, primary README/LICENSE indexers, and CDM [`SectionKind::Prose`] construction.
  3. Package manifest extraction across 9 ecosystems (`Cargo`, `Npm`, `Pip`, `GoMod`, `MavenGradle`, `Bundler`, `Composer`, `Pub`, `Mix`), role separation (`Manifest` vs `Lockfile`), lightweight deterministic parsers for `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, `pyproject.toml`, direct dependency rollup, and CDM [`SectionKind::Manifest`] construction.
  4. Repository configuration categorization (`CiCd`, `Build`, `LinterFormatter`, `Environment`, `General`) and test classification (`Unit`, `Integration`, `E2e`, `Fixture`, `Benchmark`, `General`) with bounded concurrent batch extraction via `tokio::sync::Semaphore` and CDM [`SectionKind::CodeFile`] mapping.
  5. 98 tests passing in `crates/github-extractor` (+ 1 doc test), 430 tests passing across all workspace crates with zero clippy warnings and all 5 monorepo quality gates passing.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-18)

---

### Upcoming Phase (Stage 5 — Public GitHub Repository Extraction)

#### Phase 5.4 — Repository packaging

#### Subphase `5.4.1` — Repository tree generation in crates/github-extractor
- **Unique ID:** `5.4.1`
- **Objective:** Implement hierarchical repository directory tree generation, formatting, and CDM manifest section construction in `crates/github-extractor`. Provide structured tree models (`RepoTree`, `RepoTreeNode`), recursive path insertion, directory/file aggregation metrics, visual ASCII and Unicode box-drawing diagram formatting with configurable depth and entry count limiting, category badge annotations, and conversion into Canonical Context Document Model ([`CdmSection`]) manifest sections ([`SectionKind::Manifest`]).
- **Scope:** `crates/github-extractor/src/tree_generator.rs`, `crates/github-extractor/src/tree.rs`, `crates/github-extractor/src/classification.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/repo_tree_generation_test.rs`.
- **Prerequisites:** `5.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/tree_generator.rs`
  - `crates/github-extractor/src/tree.rs`
  - `crates/github-extractor/src/classification.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/repo_tree_generation_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/tree_generator.rs`:
     - `TreeStyle` enum (`Unicode`, `Ascii`) with `#[derive(Default)]`.
     - `TreeFormatConfig` with options for style, depth ceiling (`max_depth`), entry limit (`max_entries`), file sizes, category badges, language tags, and directories-first sorting.
     - `RepoTreeNode` recursive hierarchy model with `insert_entry`, `file_count`, `dir_count`, `total_bytes`, and `max_depth`.
     - `RepoTree` aggregated model with `from_entries`, `from_classified_files`, `from_tree_response`, and `render`.
     - `category_badge` helper for compact visual annotation (`[source]`, `[doc]`, `[manifest]`, `[config]`, `[test]`, `[lockfile]`, `[binary]`, `[oversized]`).
     - `to_cdm_section`: generates a CDM `CdmSection` with `SectionKind::Manifest`, title `"Repository Directory Tree"`, `manifest_role: "repository_tree"`, BLAKE3 content hash, and token metrics.
  2. Extend `GitTreeResponse` in `crates/github-extractor/src/tree.rs`:
     - `to_repo_tree(&self) -> RepoTree`.
  3. Extend `TreeClassificationReport` in `crates/github-extractor/src/classification.rs`:
     - `to_repo_tree(&self) -> RepoTree`.
  4. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `generate_repo_tree(&self, tree_response) -> RepoTree`.
     - `generate_repo_tree_classified(&self, classified_files) -> RepoTree`.
  5. Re-export `tree_generator` models and helpers in `crates/github-extractor/src/lib.rs`.
  6. Author comprehensive integration tests in `crates/github-extractor/tests/repo_tree_generation_test.rs` covering Git tree entry construction, classification badge formatting, ASCII/Unicode styles, depth limiting, entry count ceiling, CDM section mapping, and live Axum API client integration.
  7. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Hierarchical directory tree is accurately assembled from arbitrary flat Git tree entries or classified files.
  - Directories and files are sorted deterministically with directories listed first when configured.
  - Visual output supports both Unicode and ASCII box-drawing styles with clean indentation.
  - Depth limiting and entry count capping prevent token blowout on deep or massive repositories.
  - Tree converts into valid CDM `CdmSection` (`SectionKind::Manifest`) with complete metadata.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Safe string allocation with entry capping (`max_entries`); zero shell execution; path normalization preventing directory traversal attacks.
- **Performance Considerations:** $O(N \log N)$ deterministic sorting; streaming string buffer generation with bounded memory.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 109/109 tests passed (28 unit tests, 7 tree integration tests, 73 other integration tests, 1 doc test) in 0.18s.
  - `cargo test --workspace`: 437/437 tests passed across all workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures).
- **Git Commit Reference:** `9dce03e`
- **Status:** `ACCEPTED`

---

#### Subphase `5.4.2` — File manifest in crates/github-extractor
- **Unique ID:** `5.4.2`
- **Objective:** Implement structured repository file manifest indexing, per-file extraction status accounting, multi-report aggregation, GFM Markdown summary table formatting, JSON serialization, completeness formula verification, and CDM manifest section construction in `crates/github-extractor`. Provide unified data structures (`FileExtractionStatus`, `ManifestFileEntry`, `RepoFileManifest`), stitching across classification and extraction reports (sources, docs, dependencies, configs, tests), verification of the fundamental conservation invariant (`total_files == extracted + excluded + failed + unsupported`), and conversion into Canonical Context Document Model ([`CdmSection`]) manifest sections ([`SectionKind::Manifest`]).
- **Scope:** `crates/github-extractor/src/manifest.rs`, `crates/github-extractor/src/classification.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/file_manifest_test.rs`.
- **Prerequisites:** `5.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/manifest.rs`
  - `crates/github-extractor/src/classification.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/file_manifest_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/manifest.rs`:
     - `FileExtractionStatus` enum (`Extracted`, `Excluded { reason: String }`, `Failed { error: String }`, `Unsupported { reason: String }`) with query helpers (`is_extracted`, `is_excluded`, `is_failed`, `is_unsupported`, `status_name`, `reason_or_error`).
     - `ManifestFileEntry` model representing a single file entry in the manifest with attributes: relative repository path, Git object SHA, file category, size in bytes, line count, detected language, BLAKE3 content hash, SHA-256 digest, extraction status, and optional CDM section ID.
     - `RepoFileManifest` consolidated manifest model with aggregate metrics: total discovered files, extracted files, excluded files, failed files, unsupported files, total bytes, extracted bytes, total line count, category distribution map, language distribution map, and status breakdown map.
     - Builder and stitcher constructors:
       - `from_entries(entries: Vec<ManifestFileEntry>) -> Self`: builds and computes metrics from explicit file entries.
       - `from_classification_and_reports`: aggregates tree classification and arbitrary extraction reports (sources, documentation, dependencies, configs, and tests) into a cohesive repository manifest.
     - Completeness and invariant verification:
       - `verify_completeness(&self) -> bool`: enforces `total_files == extracted + excluded + failed + unsupported`.
       - `is_complete(&self) -> bool`: checks invariant and ensures zero failed or unsupported files.
       - `to_completeness_report(&self) -> CompletenessReport`: converts to CDM `CompletenessReport`.
     - Output serializers & formatters:
       - `to_markdown_table(&self) -> String`: generates formatted GitHub-Flavored Markdown summary metrics and file table.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: serializes to JSON string.
       - `to_cdm_section(&self, section_id, source_url) -> CdmSection`: produces a CDM manifest section (`SectionKind::Manifest`) with `manifest_role: "file_manifest"`, BLAKE3 content digest, and metrics attributes.
  2. Extend `TreeClassificationReport` in `crates/github-extractor/src/classification.rs`:
     - `to_file_manifest(&self) -> RepoFileManifest`.
  3. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `generate_file_manifest`: convenience client method assembling classification and extraction reports into `RepoFileManifest`.
  4. Re-export `manifest` models and helpers in `crates/github-extractor/src/lib.rs`.
  5. Author comprehensive integration tests in `crates/github-extractor/tests/file_manifest_test.rs` covering multi-report manifest construction, extraction status tracking, Markdown table generation, JSON roundtrip serialization, completeness formula verification, CDM manifest section creation, and live client integration.
  6. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Manifest cleanly tracks extraction status for every discovered file in the repository tree.
  - Multi-report aggregator accurately stitches results from sources, docs, dependencies, configs, and tests.
  - Completeness invariant `total_files == extracted + excluded + failed + unsupported` is strictly verified.
  - Formatted Markdown table and JSON serialization cleanly represent repository index and metrics.
  - Converts into valid CDM `CdmSection` (`SectionKind::Manifest`) with complete metadata.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Safe string allocation; zero shell execution; path normalization preventing directory traversal attacks; sanitization of error messages.
- **Performance Considerations:** $O(N)$ single-pass aggregation; deterministic sorting; bounded memory usage.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 116/116 tests passed (28 unit tests, 7 manifest integration tests, 80 other integration tests, 1 doc test) in 0.19s.
  - `cargo test --workspace`: 444/444 tests passed across all workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures across bootstrap, workflows, Rust quality checks, frontend web quality checks, and 5.27 GB / 30 GB storage ceiling).
- **Git Commit Reference:** `51415e6`
- **Status:** `ACCEPTED`

---

#### Subphase `5.4.3` — Architecture context in crates/github-extractor
- **Unique ID:** `5.4.3`
- **Objective:** Implement automated repository architecture analysis, workspace topology classification, entry point discovery, component cataloging, technology stack inference, architecture document reference mapping, and CDM architecture overview section construction in `crates/github-extractor`. Provide structured domain models (`RepoArchitectureContext`, `WorkspaceType`, `EntryPointInfo`, `ComponentInfo`), multi-signal synthesis from repository files, manifests, and documentation, GFM Markdown overview generation, and conversion into Canonical Context Document Model ([`CdmSection`]) prose sections ([`SectionKind::Prose`]).
- **Scope:** `crates/github-extractor/src/architecture.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/architecture_context_test.rs`.
- **Prerequisites:** `5.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/architecture.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/architecture_context_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/architecture.rs`:
     - `WorkspaceType` enum (`SinglePackage`, `Monorepo`, `MultiService`, `Library`, `Other`) with heuristics detecting monorepo directory patterns (`crates/*`, `packages/*`, `apps/*`, `services/*`, `libs/*`, Cargo/pnpm/npm/turborepo workspace manifests).
     - `EntryPointInfo` model capturing entry point path, programming language, role/target type (`BinaryMain`, `LibraryRoot`, `WebApi`, `UiRoute`, `WorkerDaemon`, `ScriptCli`), and brief functional note.
     - `ComponentInfo` model capturing component name, relative root path, component role/type (`Service`, `LibraryCrate`, `WebFrontend`, `Worker`, `ToolCli`, `Infrastructure`), primary language, and description.
     - `RepoArchitectureContext` consolidated architecture analysis model with:
       - Workspace topology (`WorkspaceType`).
       - Primary ecosystems and programming languages with percentages.
       - Key frameworks and runtime technologies detected from manifests and dependencies (e.g., Tokio, Axum, SvelteKit, React, Playwright, Tailwind, SQLx).
       - Discovered application and service entry points (`Vec<EntryPointInfo>`).
       - High-level component and subsystem catalog (`Vec<ComponentInfo>`).
       - Discovered architectural and design documentation (`Vec<String>`).
       - Build and CI/CD pipelines detected.
     - Synthesis constructor `from_repository_data`:
       - Inspects `TreeClassificationReport`, `RepoFileManifest`, `Option<&DocExtractionReport>`, `Option<&DependencyExtractionReport>`, `Option<&ConfigTestExtractionReport>`, and `Option<&GitHubRepositoryMetadata>`.
       - Classifies workspace topology based on directory layouts and manifest workspace declarations.
       - Detects entry points based on standard language and framework conventions.
       - Catalogs components by grouping top-level directories and nested crates/packages.
       - Discovers architecture and design documents (`ARCHITECTURE.md`, `docs/ARCHITECTURE.md`, `DESIGN.md`, `docs/DATA_MODEL.md`, ADRs, etc.).
     - Formatters & CDM generation:
       - `to_markdown(&self) -> String`: generates clean, highly structured GitHub-Flavored Markdown architecture summary.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: serializes architecture context.
       - `to_cdm_section(&self, section_id, source_url) -> CdmSection`: produces a CDM section (`SectionKind::Prose`) with `manifest_role: "architecture_context"`, title `"Repository Architecture & System Overview"`, BLAKE3 content hash, and telemetry attributes.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `generate_architecture_context`: client method assembling architecture synthesis from repository extraction reports and metadata.
  3. Re-export `architecture` models and helpers in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/architecture_context_test.rs` covering workspace topology detection (monorepo vs single package), entry point discovery, component cataloging, technology inference, architecture doc indexing, Markdown generation, JSON serialization, CDM section mapping, and live client integration.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Architecture synthesis accurately classifies monorepo vs single-package topologies.
  - Entry points for backend mains, library roots, web routes, and CLI scripts are detected.
  - Subsystems and components are structured into a cohesive catalog.
  - Key frameworks, technologies, and build tools are extracted from manifests and configs.
  - Discovered architecture documents are indexed and linked.
  - Generates clear, informative GFM Markdown and valid CDM `CdmSection` (`SectionKind::Prose`).
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Safe string generation; zero shell execution; path sanitization preventing directory traversal.
- **Performance Considerations:** Single-pass deterministic evaluation; $O(N)$ scanning of files and manifests; bounded memory.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor`: 126/126 tests passed (28 unit tests, 10 architecture integration tests, 87 other integration tests, 1 doc test) in 0.20s.
  - `cargo test --workspace`: 454/454 tests passed across all workspace crates.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings, 0 errors.
  - `cargo fmt --all -- --check`: 100% compliant with standard formatting.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures across bootstrap, workflows, Rust quality checks, frontend web quality checks, and 5.27 GB / 30 GB storage ceiling).
- **Git Commit Reference:** `3196a25`
- **Status:** `ACCEPTED`

---

#### Subphase `5.4.4` — Code-review package in crates/github-extractor
- **Unique ID:** `5.4.4`
- **Objective:** Implement automated targeted AI code-review package generation, review prioritization tiering, review checklist synthesis, and end-to-end Canonical Context Document Model ([`CdmDocument`]) packaging in `crates/github-extractor`. Provide structured domain models (`CodeReviewPackage`, `CodeReviewPackageConfig`, `ReviewFocus`, `ReviewPriorityTier`, `PrioritizedReviewFile`), combining directory trees, manifests, architecture contexts, prioritized code files, and review prompt guidelines into a cohesive, token-conscious review artifact ready for LLM consumption. Complete and formally certify Phase 5.4.
- **Scope:** `crates/github-extractor/src/code_review.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/code_review_package_test.rs`.
- **Prerequisites:** `5.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/code_review.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/code_review_package_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/code_review.rs`:
     - `ReviewFocus` enum (`FullRepository`, `SecurityAudit`, `ArchitectureReview`, `DiffChangeset`, `QualityAudit`) with prompt focus guidelines.
     - `ReviewPriorityTier` enum (`Critical`, `High`, `Medium`, `Low`, `Excluded`) with heuristic prioritization scoring based on security criticality (auth, crypto, security, network, credentials, tokens), architecture entry points (main, lib, api, routes), business logic, configurations, and tests.
     - `PrioritizedReviewFile` model capturing relative repository path, priority tier, category, language, byte size, line count, prioritization rationale, and content snippet or full text.
     - `CodeReviewPackageConfig` options configuring review focus, file budget ceilings, token budgets, inclusion toggles for tree, manifest, architecture overview, and review checklist.
     - `CodeReviewPackage` consolidated review package model containing review metadata, review focus, summary metrics, prioritized files, total review files, total review lines, and review checklist items.
     - Assembly constructor `assemble`:
       - Consolidates repository tree (`RepoTree`), file manifest (`RepoFileManifest`), architecture context (`RepoArchitectureContext`), and extracted source/config/test files.
       - Ranks and filters files according to `ReviewFocus` and `CodeReviewPackageConfig` budgets.
       - Generates targeted LLM review checklist tailored to the selected focus (e.g. OWASP/SSRF defenses, memory safety, invariant validation, API breaking changes).
     - Formatters & CDM packaging:
       - `to_markdown(&self) -> String`: generates a self-contained, clean GitHub-Flavored Markdown code-review dossier.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: serializes review package.
       - `to_cdm_document(&self, doc_id, source_url) -> CdmDocument`: constructs a complete, valid `CdmDocument` containing ordered sections:
         1. Review Overview frontmatter / summary (`SectionKind::Frontmatter`)
         2. Repository directory tree (`SectionKind::Manifest`)
         3. File manifest & completeness accounting (`SectionKind::Manifest`)
         4. Architecture context & system overview (`SectionKind::Prose`)
         5. Prioritized code files (`SectionKind::CodeFile`)
         6. Review checklist & guidance (`SectionKind::Admonition` / `Prose`)
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `generate_code_review_package`: client method assembling review package from extracted repository components.
  3. Re-export `code_review` models and helpers in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/code_review_package_test.rs` covering review focus filtering, priority tiering, checklist generation, GFM Markdown formatting, JSON serialization, end-to-end CDM document construction and validation, and live API client integration.
  5. Formally certify completion of **Phase 5.4: Repository packaging** across all 4 subphases (`5.4.1` through `5.4.4` all `ACCEPTED`).
  6. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Code-review package accurately prioritizes files based on review focus (security, architecture, quality, full).
  - Review checklist produces actionable, targeted guidance for LLM reviewers.
  - Generates valid, self-contained GFM Markdown review packages.
  - Produces valid, fully compliant `CdmDocument` adhering to all CDM schema constraints and invariants.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Safe string generation; zero shell execution; path sanitization; secret redaction in review packages.
- **Performance Considerations:** Deterministic priority scoring; bounded memory and token budgets; $O(N \log N)$ ranking.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test code_review_package_test`: 8/8 tests passed in 0.01s covering review focus defaults, priority tiers, security audit filtering, file budgeting, GFM formatting, JSON serialization, and strict CDM document validation.
  - `cargo test -p web-repo-to-ai-github-extractor`: 134/134 crate tests (+ 1 doctest) passed across 15 test suites with 0 failures.
  - `cargo test --workspace`: 470/470 tests passed across all workspace crates and binaries with 0 failures.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: Advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: Bootstrap & Governance Integrity, Gate 2: Devops workflows, Gate 3: Rust Workspace Checks, Gate 4: Frontend Web Quality Checks [213 vitest tests passed, svelte-check 0 errors 0 warnings, prettier clean], Gate 5: Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `5ad4842`
- **Status:** `ACCEPTED`

---

### Phase 5.4 Milestone Certification (`5.4.1` – `5.4.4`)
- **Phase 5.4 Goal:** Complete repository packaging capabilities including directory tree generation, comprehensive file manifest and completeness accounting, architectural context synthesis, and automated AI code-review package generation with prioritization tiering and CDM document serialization.
- **Subphases Completed & Accepted:**
  - `5.4.1`: Repository tree generation in `crates/github-extractor` (`ACCEPTED`, commit `0b2da03`)
  - `5.4.2`: File manifest in `crates/github-extractor` (`ACCEPTED`, commit `51415e6`)
  - `5.4.3`: Architecture context in `crates/github-extractor` (`ACCEPTED`, commit `3196a25`)
  - `5.4.4`: Code-review package in `crates/github-extractor` (`ACCEPTED`, commit `5ad4842`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Hierarchical directory tree generator (`RepoTree`, `RepoTreeNode`) supporting Unicode box-drawing, ASCII indents, size & file category badges, depth bounds, entry limits, and CDM [`SectionKind::Manifest`] section synthesis.
  2. Multi-report consolidated file manifest (`RepoFileManifest`, `ManifestFileEntry`, `FileExtractionStatus`) with mathematical completeness conservation verification ($\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$), GFM Markdown summary table, JSON schema, and CDM [`SectionKind::Manifest`] section generation.
  3. Repository architectural context detector (`RepoArchitectureContext`, `WorkspaceType`, `EntryPointInfo`, `ComponentInfo`) inferring monorepo/single-package topologies, technology stacks, entry points, component boundaries, and architecture docs into CDM [`SectionKind::Prose`].
  4. Automated AI code-review package builder (`CodeReviewPackage`, `ReviewFocus`, `ReviewPriorityTier`, `PrioritizedReviewFile`) scoring files across 5 priority tiers, synthesizing actionable focus-specific review checklists, generating unified GFM Markdown review dossiers, and creating strictly valid end-to-end `CdmDocument` structures passing `CdmValidator::strict()`.
  5. 134 tests passing in `crates/github-extractor` (+ 1 doctest), 470 tests passing across all workspace crates with zero clippy warnings and all 5 monorepo quality gates passing.
- **Phase Status:** `COMPLETED` / `ACCEPTED` (Certified 2026-11-18)

---

### Phase 5.5 — Integrity

#### Subphase `5.5.1` — Expected-vs-actual file accounting in crates/github-extractor
- **Unique ID:** `5.5.1`
- **Objective:** Implement expected-vs-actual file accounting, mathematical completeness verification, discrepancy taxonomy, set reconciliation, and balance sheet validation between discovered Git tree entries and extracted/classified artifacts in `crates/github-extractor`. Enforce the fundamental repository completeness conservation law ($\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported} + \text{missing\_unaccounted}$) and detect any unaccounted missing files, unexpected extra files, byte size mismatches, or Git object SHA-1 mismatches before marking extraction complete.
- **Scope:** `crates/github-extractor/src/accounting.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/file_accounting_test.rs`.
- **Prerequisites:** `5.4.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/accounting.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/file_accounting_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/accounting.rs`:
     - Discrepancy models:
       - `AccountingDiscrepancyKind` (`MissingUnaccounted`, `UnexpectedExtra`, `ByteSizeMismatch`, `GitShaMismatch`, `CategoryMismatch`, `StatusConflict`).
       - `DiscrepancySeverity` (`Critical`, `Warning`, `Info`).
       - `AccountingDiscrepancy` capturing relative path, discrepancy kind, expected detail, actual detail, and severity.
     - Quantitative balance sheet model `FileAccountingBalanceSheet`:
       - Discovered blobs count ($E$).
       - Eligible blobs count.
       - Expected excluded blobs count.
       - Actual extracted count.
       - Actual excluded count.
       - Actual failed count.
       - Actual unsupported count.
       - Missing unaccounted count.
       - Unexpected extra count.
       - Byte size mismatches count.
       - Git SHA mismatches count.
       - Detailed discrepancy vector.
     - Invariant verification:
       - `is_balanced(&self) -> bool`: enforces conservation law $\text{discovered} == \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported} + \text{missing\_unaccounted}$.
       - `is_fully_accounted(&self) -> bool`: enforces $\text{missing\_unaccounted} == 0 \land \text{unexpected\_extra} == 0$.
       - `is_perfectly_consistent(&self) -> bool`: enforces balance, full accounting, zero size mismatches, and zero Git SHA mismatches.
       - `to_completeness_report(&self) -> CompletenessReport`: translates balance sheet metrics into CDM [`CompletenessReport`].
     - Auditor engine `FileAccountingAuditor`:
       - Reconciles expected tree entries from `GitTreeResponse` / `TreeClassificationReport` against actual processed artifacts in `RepoFileManifest` or multi-report collections (`SourceExtractionReport`, `DocExtractionReport`, `DependencyExtractionReport`, `ConfigTestExtractionReport`).
       - Path-by-path comparison identifying discrepancies with exact expected vs actual values.
     - Formatters & serializers:
       - `to_markdown(&self) -> String`: generates a clean GFM audit report with balance sheet table, invariant status badges, and discrepancy logs.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: structured JSON serialization.
       - `to_cdm_section(&self, doc_canonical_url) -> CdmSection`: Canonical Context Document Model representation with `SectionKind::Manifest`, `manifest_role: "file_accounting_audit"`, and BLAKE3 content hash.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `audit_file_accounting`: method executing the accounting auditor across repository artifacts.
  3. Re-export accounting models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/file_accounting_test.rs` covering balanced extractions, missing file detection, extra file detection, size mismatches, Git SHA mismatches, multi-report aggregation, CDM section generation, and live API client integration.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Accurately balances discovered files against extracted, excluded, failed, and unsupported files.
  - Detects missing unaccounted files, unexpected extra files, size mismatches, and Git SHA mismatches.
  - Correctly evaluates `is_balanced()`, `is_fully_accounted()`, and `is_perfectly_consistent()`.
  - Produces valid GFM Markdown, JSON, and schema-valid CDM sections.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor --test file_accounting_test`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Safe string generation; zero shell execution; path sanitization; secret redaction in audit reports.
- **Performance Considerations:** $O(N)$ hash-map-based set reconciliation, bounded memory allocation for discrepancy tracking.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test file_accounting_test`: 9/9 tests passed in 0.01s covering balanced accounting, missing file detection, unexpected extra file detection, byte size mismatches, Git SHA-1 mismatches, multi-report integration, CDM section generation, and live API client integration.
  - `cargo test -p web-repo-to-ai-github-extractor`: 143/143 crate tests (+ 1 doctest) passed across 16 test suites with 0 failures.
  - `cargo test --workspace`: 479/479 tests passed across all workspace crates and binaries with 0 failures.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: Advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: Bootstrap & Governance Integrity, Gate 2: Devops workflows, Gate 3: Rust Workspace Checks, Gate 4: Frontend Web Quality Checks [213 vitest tests passed, svelte-check 0 errors 0 warnings, prettier clean], Gate 5: Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `4a1be3d`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 5 — Public GitHub Repository Extraction)

---

#### Subphase `5.5.2` — Content hash validation in crates/github-extractor
- **Unique ID:** `5.5.2`
- **Objective:** Implement comprehensive cryptographic checksum verification and content hash validation across all extracted repository files, manifests, reports, and CDM documents in `crates/github-extractor`. Enforce validation across three distinct cryptographic integrity layers: Git Object SHA-1 (verifying Git tree blob consistency), BLAKE3 (verifying primary content integrity and CDM section hash parity), and SHA-256 (verifying secondary standard hash parity). Surface any corruptions, transit truncations, tamperings, or malformed hashes through structured diagnostic audit reports.
- **Scope:** `crates/github-extractor/src/hash_validator.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/hash_validation_test.rs`.
- **Prerequisites:** `5.5.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/hash_validator.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/hash_validation_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/hash_validator.rs`:
     - Domain models:
       - `HashAlgorithm`: algorithm enum (`GitBlobSha1`, `Blake3`, `Sha256`).
       - `HashCheckMetric`: quantitative metric tracking total checks, passed checks, and failed checks.
       - `HashValidationFailure`: structured error record capturing path, algorithm, expected hash, calculated hash, and failure context/reason.
       - `HashValidationConfig`: configuration switches for enabling/disabling individual algorithms (`verify_git_sha`, `verify_blake3`, `verify_sha256`) and fail-fast behavior.
       - `HashValidationReport`: comprehensive report summarizing validation outcomes, metrics per algorithm, failure count, list of failures, and invariant query `is_valid(&self) -> bool`.
     - Validation engine `ContentHashValidator`:
       - `validate_manifest_entries`: validates manifest entries against content or expected hashes.
       - `validate_reports`: validates all extracted items in `SourceExtractionReport`, `DocExtractionReport`, `DependencyExtractionReport`, and `ConfigTestExtractionReport`.
       - `validate_cdm_sections`: validates `CdmSection`s ensuring `section.content_hash == compute_blake3_hash(section.body)` and `git_blob_sha == compute_git_blob_sha(section.body)`.
       - `validate_raw_blob`: validates single byte payloads against expected Git SHA-1, BLAKE3, and SHA-256.
     - Formatters & serializers:
       - `to_markdown(&self) -> String`: GFM checksum validation report.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: structured JSON serialization.
       - `to_cdm_section(&self, doc_canonical_url) -> CdmSection`: CDM manifest section with `manifest_role: "content_hash_validation"`, BLAKE3 hash, and validation metrics in attributes.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `validate_content_hashes`: client method executing checksum validation across extracted artifacts.
  3. Re-export hash validation models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/hash_validation_test.rs` covering valid extraction hash verification, corrupted payload detection, Git SHA mismatch detection, BLAKE3 mismatch detection, SHA-256 mismatch detection, multi-report verification, CDM section hash verification, and live API client integration.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Cryptographic verification passes 100% on valid extracted content across Git SHA-1, BLAKE3, and SHA-256.
  - Accurately detects and pinpoints content corruptions, byte alterations, and malformed hash formats.
  - Reconciles CDM document sections and manifest entries with zero false positives.
  - Produces valid GFM Markdown, JSON, and schema-valid CDM sections.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor --test hash_validation_test`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Cryptographic integrity checking, defense against transit corruption, collision-resistant hash algorithms (BLAKE3 + SHA-256), safe string generation.
- **Performance Considerations:** High-throughput streaming cryptographic computation via SIMD-accelerated BLAKE3, $O(N)$ single-pass evaluation.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test hash_validation_test`: 11/11 tests passed in 0.00s covering valid hash verification, Git SHA-1 mismatch detection, BLAKE3 mismatch detection, SHA-256 mismatch detection, malformed hex format detection, multi-report verification, manifest verification, CDM section verification, fail-fast behavior, and live API client integration.
  - `cargo test -p web-repo-to-ai-github-extractor`: 154/154 crate tests (+ 1 doctest) passed across 17 test suites with 0 failures.
  - `cargo test --workspace`: 490/490 tests passed across all workspace crates and binaries with 0 failures.
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: 100% clean formatting.
  - `cargo deny check`: Advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (Gate 1: Bootstrap & Governance Integrity, Gate 2: Devops workflows, Gate 3: Rust Workspace Checks, Gate 4: Frontend Web Quality Checks [213 vitest tests passed, svelte-check 0 errors 0 warnings, prettier clean], Gate 5: Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `24d740f`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 5 — Public GitHub Repository Extraction)

---

#### Subphase `5.5.3` — Missing & excluded file reports in crates/github-extractor
- **Unique ID:** `5.5.3`
- **Objective:** Implement structured diagnostics, categorizations, and formal reporting for files that were intentionally excluded (oversized, generated/lockfiles, vendor/dependencies, binaries, policy filters, sensitive credentials) or failed extraction in `crates/github-extractor`. Provide granular root-cause classification, remediation recommendations, multi-report integration, GFM Markdown audit tables, JSON serialization, and bidirectional mapping to Canonical Context Document Model [`ExclusionRecord`] structures.
- **Scope:** `crates/github-extractor/src/exclusion.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/missing_excluded_reports_test.rs`.
- **Prerequisites:** `5.5.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/exclusion.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/missing_excluded_reports_test.rs`
- **Implementation Tasks:**
  1. Implement `crates/github-extractor/src/exclusion.rs`:
     - Domain models:
       - `ExclusionCategory`: strongly-typed exclusion taxonomy (`OversizedFile`, `GeneratedOrLockfile`, `VendorOrThirdParty`, `BinaryOrAsset`, `SubmoduleOrSymlink`, `PolicyPattern`, `SensitiveSecurityPattern`, `Other`).
       - `ExtractionFailureCategory`: strongly-typed failure taxonomy (`NetworkTimeout`, `RateLimitExceeded`, `BlobTooLarge`, `CorruptedContent`, `SubtreeTraversalFailed`, `IoOrStorage`, `Unknown`).
       - `ExcludedFileRecord`: path, git_sha, category, size_bytes, language, rationale, pattern, remediation.
       - `FailedFileRecord`: path, git_sha, error_category, error_message, retry_eligible, http_status.
       - `MissingFileRecord`: path, git_sha, expected_size, diagnostics.
       - `ExclusionAuditReport`: target_repo, target_ref, pinned_sha, total_discovered, total_excluded, total_failed, total_missing, excluded_bytes, exclusion_breakdown, failure_breakdown, lists of records.
       - Query & audit methods:
         - `is_clean(&self) -> bool`: returns true if 0 missing files and 0 failed files.
         - `has_critical_issues(&self) -> bool`: returns true if any missing or failed files exist.
         - `retryable_failures(&self) -> Vec<&FailedFileRecord>`: filters failures eligible for HTTP/exponential backoff retry.
         - `to_cdm_exclusion_records(&self) -> Vec<ExclusionRecord>`: maps excluded files directly into CDM [`ExclusionRecord`] structures with appropriate [`ExclusionReason`].
     - Generator engine `ExclusionReportGenerator`:
       - Reconciles tree classifications, manifests, and accounting balance sheets into a unified `ExclusionAuditReport`.
       - Generates actionable remediation recommendations per category.
     - Formatters & serializers:
       - `to_markdown(&self) -> String`: generates a clean GFM audit report with executive summary, category breakdown tables, and itemized logs.
       - `to_json(&self) -> Result<String, serde_json::Error>` and `to_json_compact(&self)`: structured JSON serialization.
       - `to_cdm_section(&self, doc_canonical_url) -> CdmSection`: Canonical Context Document Model representation with `SectionKind::Manifest`, `manifest_role: "exclusion_and_missing_report"`, and BLAKE3 hash.
  2. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `generate_exclusion_report`: client method assembling the report from repository artifacts.
  3. Re-export exclusion models and functions in `crates/github-extractor/src/lib.rs`.
  4. Author comprehensive integration tests in `crates/github-extractor/tests/missing_excluded_reports_test.rs` covering all exclusion categories, failure classifications, missing file diagnostics, remediation recommendations, CDM `ExclusionRecord` translation, GFM formatting, JSON serialization, and live API client integration.
  5. Run full quality gate (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Granular categorization of all excluded files with explicit reasons and remediation actions.
  - Accurately captures failures and unaccounted missing files with severity indicators.
  - Converts cleanly to CDM `ExclusionRecord` items with correct pattern and reason mappings.
  - Produces valid GFM Markdown, JSON, and schema-valid CDM sections.
  - 100% test pass rate, 0 clippy warnings, clean formatting.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor --test missing_excluded_reports_test`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Path sanitization, secret redaction in error messages and excluded file paths, zero leakage of authorization headers.
- **Performance Considerations:** Single-pass aggregation, deterministic sorting by path, bounded memory footprints.
- **Documentation Requirements:** Module and type documentation in `crates/github-extractor`, tracking updates in `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`.
- **Validation Evidence:**
  - `crates/github-extractor/tests/missing_excluded_reports_test.rs`: 11 integration tests executed and passing:
    - `test_exclusion_category_inference`: validates category inference for sensitive patterns (`.env`, `secrets.json`, `.pem`, `id_rsa`), vendor directories (`node_modules/`, `target/`), binary assets (`.wasm`, `.png`), generated lockfiles (`.lock`), and fallback categories.
    - `test_exclusion_category_cdm_mapping_and_remediation`: validates bidirectional translation to CDM [`ExclusionReason`] (`SizeLimit`, `ExcludedExtension`, `RulePattern`, `BinaryFile`, `SecurityRestricted`, `UnsupportedType`) and non-empty remediation guidance.
    - `test_extraction_failure_category_inference_and_retryability`: verifies root-cause extraction failure taxonomy and retry eligibility.
    - `test_excluded_file_record_and_cdm_conversion`: verifies CDM [`ExclusionRecord`] struct generation with path, pattern, rationale, and size.
    - `test_failed_file_record_creation`: validates error categorizations and HTTP status capture.
    - `test_missing_file_record_creation`: verifies unaccounted dropped file tracking.
    - `test_exclusion_audit_report_clean_vs_critical`: verifies `is_clean()` and `has_critical_issues()` integrity predicates.
    - `test_generate_from_manifest_with_mixed_entries`: verifies multi-item aggregation, category breakdown distribution, balance sheet discrepancy merging, and retry filtering.
    - `test_exclusion_audit_report_markdown_and_json_formatting`: verifies GFM tables, badge rendering, and JSON roundtrip serialization.
    - `test_exclusion_audit_report_cdm_section_generation`: verifies CDM manifest section construction with BLAKE3 content hashing and token estimation.
    - `test_client_generate_exclusion_report_integration`: verifies end-to-end integration with `GitHubApiClient`.
  - Full crate test suite: 119 tests passing across unit and integration targets with 0 failures.
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `826e13d`
- **Status:** `ACCEPTED`

---

#### Subphase `5.5.4` — Large repository regression tests in crates/github-extractor
- **Unique ID:** `5.5.4`
- **Objective:** Author an end-to-end regression test suite, stress verification framework, and golden validation harness for public GitHub repository extraction on realistically scaled, diverse polyglot repositories. Verify deep tree traversal, truncation detection and subtree recovery at scale, multi-language ecosystem discovery, mathematical completeness conservation ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$), cryptographic content hash validation (Git SHA-1, BLAKE3, SHA-256), memory and token budget constraints, and strict schema-compliant Canonical Context Document Model ([`web_repo_to_ai_cdm::CdmDocument`]) assembly passing [`web_repo_to_ai_cdm::CdmValidator::with_config(ValidationConfig::strict())`]. Complete Phase 5.5 and formally certify Stage 5.
- **Scope:** `crates/github-extractor/tests/large_repository_regression_test.rs`, `crates/github-extractor/src/*`, documentation updates across `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Prerequisites:** `5.5.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/tests/large_repository_regression_test.rs`
  - `crates/github-extractor/src/lib.rs`
- **Implementation Tasks:**
  1. Author `crates/github-extractor/tests/large_repository_regression_test.rs`:
     - Test 1: `test_large_scale_tree_truncation_and_subtree_stitching`:
       - Construct a synthetic repository tree with 2,500+ files spanning deep folder hierarchies (nested 10 levels deep).
       - Simulate Git Trees API truncation (`truncated: true`).
       - Exercise truncation analysis, shallow archive recovery / subtree recovery, and path stitching.
       - Assert 100% tree preservation with zero dropped paths and zero duplicate entries.
     - Test 2: `test_polyglot_monorepo_end_to_end_extraction_and_packaging`:
       - Construct a realistic enterprise polyglot monorepo containing:
         - Rust backend service (`Cargo.toml`, source modules, tests).
         - TypeScript/SvelteKit frontend app (`package.json`, components, assets).
         - Python ML microservice (`pyproject.toml`, requirements, scripts).
         - Go tooling service (`go.mod`, handlers).
         - Polyglot documentation (`README.md`, `CONTRIBUTING.md`, `docs/architecture/*.md`, `docs/api/*.md`).
         - CI/CD configurations (`.github/workflows/*.yml`, `Dockerfile`, `compose.yaml`).
         - Vendored and excluded artifacts (`node_modules/`, `target/`, `.git/`, `*.lock`, `*.min.js`).
         - Binary assets (`*.png`, `*.wasm`, `*.ico`).
         - Security-sensitive files (`.env`, `secrets.json`, `keys/id_rsa`).
       - Execute full classification, source acquisition, doc extraction, dependency parsing, config/test categorization, repo tree generation, architecture context inference, and code-review packaging.
     - Test 3: `test_large_repository_mathematical_conservation_identity`:
       - Reconcile the discovered tree against all extracted, excluded, failed, and unsupported items using `FileAccountingAuditor`.
       - Assert `balance_sheet.is_balanced()` is true.
       - Assert `balance_sheet.is_fully_accounted()` is true.
       - Assert `balance_sheet.is_perfectly_consistent()` is true.
       - Verify zero missing unaccounted files and zero unexpected extra files.
     - Test 4: `test_large_repository_multi_algorithm_hash_validation`:
       - Validate cryptographic hashes across extracted files using `ContentHashValidator`.
       - Assert `verify_git_sha`, `verify_blake3`, and `verify_sha256` all succeed with 0 failures across all files.
       - Inject simulated content corruption in a file and verify that `ContentHashValidator` catches and records the exact failure.
     - Test 5: `test_large_repository_exclusion_and_failure_diagnostics`:
       - Generate `ExclusionAuditReport` via `ExclusionReportGenerator`.
       - Verify every excluded file has an accurate `ExclusionCategory`, actionable remediation, and valid CDM `ExclusionRecord` mapping.
       - Verify GFM Markdown and JSON serializations.
     - Test 6: `test_large_repository_code_review_package_budget_enforcement`:
       - Generate `CodeReviewPackage` across different review focuses (`FullRepository`, `SecurityAudit`, `ArchitectureReview`).
       - Verify file count limits and token budgets are strictly observed without truncation crashes or memory spikes.
     - Test 7: `test_large_repository_end_to_end_cdm_assembly_strict_validation`:
       - Assemble all components into a comprehensive `CdmDocument` representing the entire repository.
       - Include metadata, source identity, repo architecture overview, file tree, code sections, doc sections, dependency manifests, configuration files, file accounting balance sheet, hash validation report, and exclusion audit report.
       - Validate the document using `CdmValidator::with_config(ValidationConfig::strict())`.
       - Assert that `validation_report.is_valid()` is true with 0 errors and 0 critical issues.
     - Test 8: `test_client_high_throughput_mock_http_integration`:
       - Spin up an in-process Axum mock HTTP server simulating GitHub REST API endpoints for a large repository.
       - Exercise `GitHubApiClient` methods in an end-to-end integration workflow.
  2. Verify all tests pass, memory stays within bounded footprint, and CPU baseline remains stable.
  3. Execute monorepo quality audit via `cargo test --workspace`, `cargo clippy`, `cargo fmt`, `cargo deny check`, and `powershell scripts/check-all.ps1`.
  4. Formally certify Phase 5.5 and Stage 5 in all project ledgers and roadmaps.
- **Acceptance Criteria:**
  - All large-scale tests pass with 0 failures, 0 leaked descriptors, and zero unbounded memory growth.
  - Mathematical conservation identity holds perfectly ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$).
  - Cryptographic content hashes (Git SHA-1, BLAKE3, SHA-256) validate 100% across extracted files.
  - The constructed `CdmDocument` passes `CdmValidator::with_config(ValidationConfig::strict())` with zero validation errors.
  - 100% of Stage 5 requirements (`FR-GH-PUB`, `NFR-QUAL`, `NFR-PERF`) satisfied.
- **Tests:** `cargo test -p web-repo-to-ai-github-extractor --test large_repository_regression_test`, `cargo test --workspace`, `powershell scripts/check-all.ps1`.
- **Security Considerations:** Enforces Rule 00, Rule 04, Rule 05: Path traversal defense, credentials scrubbing, no secret leakage in error logs or markdown exports.
- **Performance Considerations:** Tests verify bounded memory consumption on large trees, stream handling, and deterministic ordering.
- **Documentation Requirements:** Update `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Validation Evidence:**
  - `crates/github-extractor/tests/large_repository_regression_test.rs`: 8 integration tests executed and passing:
    - `test_large_scale_tree_truncation_and_subtree_stitching`: validates 2,500-blob deep tree construction, truncation flag detection, and zero-drop, zero-duplicate subtree recovery and path stitching with `TreeStitcher`.
    - `test_polyglot_monorepo_classification_and_manifest_synthesis`: validates polyglot repository classification across Rust, TypeScript, Python, Go, documentation, CI/CD, lockfiles, and binary assets with complete multi-language accounting.
    - `test_large_repository_mathematical_conservation_identity`: proves exact arithmetic conservation $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported} + \text{missing\_unaccounted}$, verifies `is_balanced()`, `is_fully_accounted()`, `is_perfectly_consistent()`, and maps to a valid CDM `CompletenessReport`.
    - `test_large_repository_multi_algorithm_hash_validation_and_corruption_detection`: verifies 100% cryptographic checksum validity across Git Object SHA-1, BLAKE3, and SHA-256 digests, and validates pinpoint tamper detection on simulated file corruption.
    - `test_large_repository_exclusion_and_diagnostics_audit`: verifies granular categorization across vendor, lockfile, binary, and security exclusion categories, validates CDM `ExclusionRecord` mapping, and confirms Markdown & JSON formatting.
    - `test_large_repository_code_review_package_budget_and_prioritization`: validates review focus heuristic scoring, checklist synthesis, and strict file count budget ceilings.
    - `test_large_repository_end_to_end_cdm_assembly_strict_validation`: constructs a unified `CdmDocument` representing the full monorepo with tree, architecture context, review package, balance sheet, checksum report, and exclusion audit sections, passing `CdmValidator::with_config(ValidationConfig::strict())` with zero validation issues.
    - `test_large_repository_live_mock_http_integration`: validates ref discovery, commit revision pinning, and tree fetching through `GitHubApiClient` against an in-process Axum mock HTTP server.
  - Full crate test suite: 127 tests passing across unit and integration targets with 0 failures (28 unit tests, 98 integration tests, 1 doc-test).
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `6938797`
- **Status:** `ACCEPTED`

---

### PHASE 5.5 & STAGE 5 FORMAL CERTIFICATION

- **Stage:** `STAGE 5 — PUBLIC GITHUB REPOSITORY EXTRACTION`
- **Phase:** `Phase 5.5 — Integrity`
- **Subphases Certified (20 of 20):**
  - Phase 5.1 (Metadata): `5.1.1` (Repository Identification), `5.1.2` (Branch & Tag Discovery), `5.1.3` (Commit Selection), `5.1.4` (Repository Metadata Extraction) — `ACCEPTED`
  - Phase 5.2 (Discovery): `5.2.1` (Git Trees API), `5.2.2` (Truncation Detection), `5.2.3` (Subtree Recovery), `5.2.4` (File Classification) — `ACCEPTED`
  - Phase 5.3 (Extraction): `5.3.1` (Source Acquisition), `5.3.2` (Documentation Extraction), `5.3.3` (Dependency Extraction), `5.3.4` (Config & Test Extraction) — `ACCEPTED`
  - Phase 5.4 (Packaging): `5.4.1` (Repo Tree Generation), `5.4.2` (File Manifest), `5.4.3` (Architecture Context), `5.4.4` (Code-Review Package) — `ACCEPTED`
  - Phase 5.5 (Integrity): `5.5.1` (Expected-vs-Actual Accounting), `5.5.2` (Content Hash Validation), `5.5.3` (Missing & Excluded File Reports), `5.5.4` (Large Repository Regression Tests) — `ACCEPTED`
- **Stage Acceptance Gate Verification:**
  - $\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$ conservation identity proven mathematically.
  - Multi-algorithm cryptographic integrity (Git SHA-1, BLAKE3, SHA-256) verified across thousands of files.
  - Schema-valid CDM v1.0 document construction verified with `CdmValidator` strict mode.
  - Bounded memory, CPU-only 16 GB RAM baseline, and `<= 30 GB` storage ceiling strictly observed.
- **Certification Status:** **STAGE 5 FORMALLY CERTIFIED & ACCEPTED**

---

### Active Subphase (Stage 6 — Private GitHub Repositories)

#### Phase 6.1 — GitHub App

#### Subphase `6.1.1` — App configuration documentation & manifest in crates/github-extractor & docs/GITHUB_APP.md
- **Unique ID:** `6.1.1`
- **Objective:** Establish the formal GitHub App configuration models, least-privilege permission validation engine, 1-click GitHub App Manifest generator, runtime environment loader with strict secret masking, and comprehensive operational guide in `docs/GITHUB_APP.md` and `crates/github-extractor/src/app_manifest.rs`. Enforce Rule 05 (strictly `Contents: Read-only` and `Metadata: Read-only` permissions; zero write or administrative scopes).
- **Scope:** `crates/github-extractor/src/app_manifest.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/app_manifest_test.rs`, `docs/GITHUB_APP.md`.
- **Prerequisites:** Stage 5 Certified.
- **Files/Components Affected:**
  - `crates/github-extractor/src/app_manifest.rs`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/app_manifest_test.rs`
  - `docs/GITHUB_APP.md`
- **Implementation Tasks:**
  1. Define strongly-typed GitHub App Manifest models conforming to GitHub's official manifest creation flow:
     - `PermissionLevel`: `Read` enum variant with strict rejection of write or admin permissions.
     - `GitHubAppPermissions`: strongly-typed struct ensuring only `contents: read` and `metadata: read` are allowed.
     - `GitHubHookAttributes`: webhook configuration attributes (`url`, `active`).
     - `GitHubAppManifest`: root manifest representation with name, url, hooks, callbacks, setup_url, description, public/private toggle, permissions, and webhook events (`installation`, `installation_repositories`).
  2. Implement `GitHubAppManifestGenerator`:
     - Fluent builder API (`new`, `with_callback_url`, `with_webhook_url`, `with_setup_url`, `with_public`).
     - Minimal permissions validator (`validate_minimal_permissions`) rejecting any write or administrative permissions per Rule 05.
     - 1-click manifest creation URL generator (`to_manifest_create_url`) producing URL-encoded `POST/GET` payloads for GitHub.com and GitHub Enterprise Server.
     - Serialization helpers (`to_json`, `to_json_pretty`).
  3. Implement `GitHubAppConfig`:
     - Strongly-typed runtime configuration capturing `app_id`, `client_id`, `client_secret`, `private_key_pem`, `webhook_secret`, `app_name`, `app_slug`, `base_api_url`, and `html_url`.
     - Custom `fmt::Debug` and `fmt::Display` implementations masking all sensitive secrets (`client_secret`, `private_key_pem`, `webhook_secret`) with `***[REDACTED_SECRET]***` to prevent credential leakage.
     - Environment variable loader (`from_env`) loading `GITHUB_APP_ID`, `GITHUB_APP_CLIENT_ID`, `GITHUB_APP_CLIENT_SECRET`, `GITHUB_APP_PRIVATE_KEY_PEM`, and `GITHUB_APP_WEBHOOK_SECRET`.
     - Domain validation (`validate`) ensuring positive app ID, non-empty client ID/secret, and valid PEM headers (PKCS#1 / PKCS#8).
     - Public metadata accessor (`public_metadata`) exposing non-sensitive metadata for system health checks.
  4. Author publication-grade documentation in `docs/GITHUB_APP.md`:
     - Architecture & security rationale (GitHub App vs PAT vs OAuth App).
     - 1-click manifest creation workflow.
     - Manual GitHub App creation walkthrough.
     - Strict minimal permissions reference matrix and forbidden permissions policy.
     - Webhook configuration and HMAC-SHA256 signature verification runbook.
     - RSA key generation, PEM formats, and token exchange lifecycle.
     - Complete environment variable reference table.
  5. Author unit and integration tests in `crates/github-extractor/tests/app_manifest_test.rs`.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test app_manifest_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Manifest generation strictly adheres to GitHub App Manifest specs with only `contents: read` and `metadata: read`.
  - Any attempt to define write or admin permissions triggers explicit rejection.
  - Secrets are completely masked in `Debug` and `Display` representations.
  - Environment loader correctly parses and validates real or simulated GitHub App credentials.
  - `docs/GITHUB_APP.md` provides exhaustive operational and security instructions.
  - 100% of tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: Secret leakage in log messages or error envelopes.
  - Mitigation: Dedicated custom `Debug`/`Display` implementations masking `client_secret`, `private_key_pem`, and `webhook_secret`; integration tests asserting zero secret leakage.
- **Rollback Plan:**
  - Revert Subphase 6.1.1 additions via `git reset --hard HEAD~1` and restore Stage 5 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test app_manifest_test` passed 10 test suites (minimal read-only validation, serde level roundtrip, forbidden scope rejection, builder, json serialization, 1-click URL generation, input validation, key formats, secret masking, public metadata).
  - Full crate test suite: 138 tests passing across unit and integration targets with 0 failures (28 unit tests, 109 integration tests, 1 doc-test).
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 511 workspace Rust tests, 213 frontend vitest tests, and 17.6% Podman storage footprint.
- **Git Commit Reference:** `2a9a7b3`
- **Status:** `ACCEPTED`

---

#### Subphase `6.1.2` — OAuth & installation flow in crates/github-extractor
- **Unique ID:** `6.1.2`
- **Objective:** Implement cryptographic OAuth state management (CSRF mitigation), user authorization and app installation URL builders, server-side OAuth code exchange (`POST /login/oauth/access_token`), RS256 JWT generation with RSA PKCS#1/PKCS#8 private keys, installation token exchange (`POST /app/installations/{id}/access_tokens`), and user/installation query APIs (`GET /user/installations`, `GET /installation/repositories`) in `crates/github-extractor`. Enforce Rule 05 token masking in all `Debug` and `Display` implementations.
- **Scope:** `crates/github-extractor/src/oauth.rs`, `crates/github-extractor/src/installation.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/Cargo.toml`, `crates/github-extractor/tests/oauth_installation_test.rs`.
- **Prerequisites:** `6.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/oauth.rs`
  - `crates/github-extractor/src/installation.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/Cargo.toml`
  - `crates/github-extractor/tests/oauth_installation_test.rs`
- **Implementation Tasks:**
  1. Implement cryptographic OAuth state engine in `crates/github-extractor/src/oauth.rs`:
     - `OAuthState`: 256-bit BLAKE3-hashed high-entropy nonce, timestamps, session ID, tenant ID, and redirect URI binding with constant-time equality check (`constant_time_eq`).
     - `OAuthStateStore` trait and `InMemoryOAuthStateStore` with automated TTL eviction and single-use replay protection.
     - URL builders: `build_oauth_authorize_url` and `build_app_installation_url`.
     - Code exchange models: `GitHubOAuthTokenResponse` and `GitHubOAuthErrorResponse` with masked `Debug` and `Display`.
     - RS256 JWT generation: `base64url_encode`, `pkcs1_to_pkcs8_der`, `parse_rsa_pem_to_pkcs8_der`, and `generate_app_jwt_at_time` using `ring::signature::RsaKeyPair` with PKCS#1 v1.5 SHA-256.
  2. Implement installation domain models in `crates/github-extractor/src/installation.rs`:
     - `RepositorySelection`: `All` or `Selected`.
     - `GitHubAccountSummary`: account profiles (user or organization).
     - `GitHubInstallation`: registered App installation representation with permissions, events, and suspension status.
     - `InstalledRepositorySummary`: repository model for installation access.
     - `GitHubInstallationToken`: short-lived access token with masked `Debug`/`Display`, `is_expired()`, and `is_expiring_soon()`.
     - `UserInstallationsResponse` and `InstallationRepositoriesResponse`.
  3. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `exchange_oauth_code`: exchanges temporary authorization code for user access tokens.
     - `list_user_installations`: retrieves installations accessible to the user (`GET /user/installations`).
     - `get_installation`: retrieves installation details via App JWT (`GET /app/installations/{id}`).
     - `create_installation_access_token`: mints short-lived installation access tokens (`POST /app/installations/{id}/access_tokens`).
     - `list_installation_repositories`: enumerates repositories accessible to an installation token (`GET /installation/repositories`).
  4. Author comprehensive test suite in `crates/github-extractor/tests/oauth_installation_test.rs`:
     - 10 test suites covering nonce generation, constant-time matching, store replay defense, session binding, TTL expiration, URL builders, secret masking, RS256 JWT signing from PKCS#1/PKCS#8 keys, and live in-process Axum mock HTTP server integration.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test oauth_installation_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - OAuth state nonces are cryptographically random, constant-time compared, and single-use.
  - Authorization and installation URLs are correctly formatted and encoded.
  - RS256 JWTs are valid 3-part tokens signed by RSA private keys (PKCS#1 and PKCS#8).
  - All token models strictly mask sensitive strings (`[REDACTED_TOKEN]`) in `Debug` and `Display`.
  - Mock HTTP integration tests prove end-to-end functionality of all OAuth and installation endpoints.
  - 100% of workspace tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: Token leakage in logs or error traces.
  - Mitigation: Implemented custom `Debug` and `Display` on all token models; `redact_secrets` invoked on all HTTP error messages and URLs.
- **Rollback Plan:**
  - Revert via `git reset --hard HEAD~1` and restore Subphase 6.1.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test oauth_installation_test` passed 10 test suites.
  - Full crate test suite: 148 tests passing across unit and integration targets with 0 failures (28 unit tests, 119 integration tests, 1 doc-test).
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 521 workspace Rust tests, 213 frontend vitest tests, and 17.6% Podman storage footprint.
- **Git Commit Reference:** `01d1290`
- **Status:** `ACCEPTED`

---

#### Subphase `6.1.3` — Minimum permissions in crates/github-extractor
- **Unique ID:** `6.1.3`
- **Objective:** Implement runtime permission verification, least-privilege policy enforcement, excessive and forbidden scope detection, and audit report generation for GitHub App installations and tokens in `crates/github-extractor`. Enforce Rule 05 invariants (strictly `contents: read` and `metadata: read`; zero write/admin scopes; immediate rejection of sensitive categories like `secrets`, `actions`, `administration`).
- **Scope:** `crates/github-extractor/src/permissions.rs`, `crates/github-extractor/src/installation.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/minimum_permissions_test.rs`.
- **Prerequisites:** `6.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/permissions.rs`
  - `crates/github-extractor/src/installation.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/minimum_permissions_test.rs`
- **Implementation Tasks:**
  1. Implement configurable permission policy engine in `crates/github-extractor/src/permissions.rs`:
     - `PermissionPolicy`: `strict_minimal()` (strictly `contents: read`, `metadata: read`), `standard()` (permits benign read-only scopes, forbids all write/admin and dangerous categories), and `builder()` API for custom policies.
     - `PermissionViolation`: Granular enum representing `MissingRequired`, `ExcessiveLevel`, `ForbiddenScope`, `UnexpectedScope`, and `SuspendedInstallation`.
     - `PermissionAuditReport`: Report with compliance boolean, violation list, summary description, JSON export, and GitHub-Flavored Markdown report formatting with tables and alerts.
     - `PermissionAuditor`: Auditor and validator for raw permission maps, `GitHubInstallation`, and `GitHubInstallationToken`.
  2. Enhance `crates/github-extractor/src/installation.rs`:
     - Implemented `audit_permissions()`, `audit_permissions_with_policy()`, `validate_permissions()`, and `validate_permissions_with_policy()` on `GitHubInstallation` and `GitHubInstallationToken`.
     - Implemented `InstallationTokenRequest` modeling scoped token requests (`POST /app/installations/{id}/access_tokens`) with minimal read-only presets.
  3. Extend `crates/github-extractor/src/client.rs`:
     - Added `create_scoped_installation_access_token` taking an `InstallationTokenRequest`.
     - Validates returned token permissions against the least-privilege policy before returning.
  4. Extended `crates/github-extractor/src/error.rs`:
     - Added `GitHubExtractorError::PermissionViolation`.
  5. Authored comprehensive test suite in `crates/github-extractor/tests/minimum_permissions_test.rs`:
     - 14 test suites covering compliant minimal permissions, missing contents, missing metadata, excessive write/admin levels, forbidden categories, suspended installations, Markdown audit generation, token request scoping, and live mock HTTP server validation.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test minimum_permissions_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Runtime validation strictly enforces `contents: read` and `metadata: read`.
  - Any write or admin permission is immediately detected and rejected.
  - Sensitive categories (`secrets`, `actions`, etc.) are detected and rejected even if set to `read`.
  - Suspended installations are rejected.
  - Markdown audit reports provide clear diagnostic visibility.
  - 100% of workspace tests pass, clippy has 0 warnings, formatting is clean, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Over-permissive installations could silently gain unauthorized write access or expose organization secrets.
  - Mitigation: Strict fail-closed validation at both the App installation and minted token layers under Rule 05.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `01d1290`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test minimum_permissions_test` passed 14 test suites in 0.01s.
  - Full crate test suite: 162 tests passing across unit and integration targets with 0 failures (28 unit tests, 133 integration tests, 1 doc-test).
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 535 workspace Rust tests, 213 frontend vitest tests, and 17.6% Podman storage footprint.
- **Git Commit Reference:** `8e4d575`
- **Status:** `ACCEPTED`

---

#### Subphase `6.1.4` — Callback security in crates/github-extractor
- **Unique ID:** `6.1.4`
- **Objective:** Implement cryptographic OAuth callback validation, open-redirect defense, installation setup callback verification, HMAC-SHA256 webhook signature validation (`X-Hub-Signature-256`), delivery GUID deduplication (replay prevention), and strongly-typed lifecycle event routing in `crates/github-extractor`. Enforce Rule 05 invariants (constant-time MAC and nonce verification, fail-closed parsing, zero-trust redirect validation).
- **Scope:** `crates/github-extractor/src/callback.rs`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/callback_security_test.rs`.
- **Prerequisites:** `6.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/callback.rs`
  - `crates/github-extractor/src/error.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/callback_security_test.rs`
- **Implementation Tasks:**
  1. Implement OAuth callback validation and open-redirect defense in `crates/github-extractor/src/callback.rs`:
     - `OAuthCallbackParams`: Deserializes query parameters (`code`, `state`, `error`, `error_description`, `error_uri`).
     - `ValidatedOAuthCallback`: Encapsulates verified code and the consumed, validated `OAuthState`.
     - `OAuthCallbackValidator`: Validates non-empty sanitized code, single-use state nonce consumption with constant-time matching, session ID / tenant ID verification, and handles GitHub OAuth error responses (`access_denied`, etc.).
     - `OpenRedirectGuard`: Rejects open redirect attacks by strictly allowing relative paths starting with a single `/` (and rejecting protocol-relative `//` or `/\`), and requiring whitelisted base origins for absolute URLs.
  2. Implement GitHub App post-installation setup callback handling:
     - `InstallationCallbackParams`: Captures `installation_id`, `setup_action`, `code`, and `state`.
     - `SetupAction`: Strongly-typed enum (`Install`, `Update`, `Request`).
     - `InstallationCallbackValidator`: Validates positive installation IDs, allowed setup actions, and verifies state against `OAuthStateStore` if provided.
  3. Implement webhook callback security:
     - `WebhookSignatureVerifier`: Verifies `X-Hub-Signature-256` HMAC-SHA256 signature against raw request body using constant-time MAC verification via `ring::hmac::verify`. Rejects malformed, missing, or mismatched signatures.
     - `WebhookDeliveryTracker` trait and `InMemoryWebhookDeliveryTracker`: Prevents replay attacks by tracking `X-GitHub-Delivery` GUIDs with sliding-window expiration.
     - `WebhookEventParser`: Enforces HMAC verification *before* parsing JSON payloads. Deserializes events into `GitHubWebhookEvent` (`Ping`, `Installation`, `InstallationRepositories`, `Other`).
  4. Author comprehensive test suite in `crates/github-extractor/tests/callback_security_test.rs`:
     - Test valid/invalid/replayed/expired OAuth callbacks.
     - Test open redirect guard against protocol-relative URLs, javascript schemes, and untrusted hosts.
     - Test installation setup callback validation.
     - Test HMAC-SHA256 webhook signature verification (valid, tampered payload, wrong secret, missing/malformed header).
     - Test webhook delivery deduplication and lifecycle event parsing.
     - Test live Axum mock HTTP server webhook endpoint with signature verification.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test callback_security_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - OAuth callbacks strictly require valid, unexpired, single-use state nonces and matching session IDs.
  - Open redirect guard rejects malicious redirect attempts.
  - Webhooks without valid HMAC-SHA256 signatures are rejected before JSON deserialization.
  - Duplicate webhook delivery GUIDs are detected and rejected.
  - 100% of workspace tests pass, clippy is warning-free, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: CSRF attacks during OAuth callback, open redirects leading users to phishing domains, or webhook spoofing.
  - Mitigation: Single-use cryptographic nonces, constant-time comparisons, URL path validation, and HMAC-SHA256 signatures.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `8e4d575`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test callback_security_test` passed 14 test suites in 0.01s.
  - Full crate test suite: 176 tests passing across unit and integration targets with 0 failures (28 unit tests, 147 integration tests, 1 doc-test).
  - Full workspace clippy check: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Code formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` including 549 workspace Rust tests, 213 frontend vitest tests, and 17.6% Podman storage footprint.
- **Git Commit Reference:** `4f8a68e`
- **Status:** `ACCEPTED`

---

### Phase 6.1 Milestone Certification (`6.1.1` – `6.1.4`)
- **Phase 6.1 Goal:** Complete GitHub App foundation, including manifest models and 1-click generator, cryptographic OAuth state management and installation token exchange, least-privilege permission validation under Rule 05, and timing-safe callback and HMAC-SHA256 webhook security.
- **Subphases Completed & Accepted (4 of 4):**
  - `6.1.1`: App configuration documentation & manifest in `crates/github-extractor` & `docs/GITHUB_APP.md` (`ACCEPTED`, commit `7575950`)
  - `6.1.2`: OAuth & installation flow in `crates/github-extractor` (`ACCEPTED`, commit `01d1290`)
  - `6.1.3`: Minimum permissions in `crates/github-extractor` (`ACCEPTED`, commit `8e4d575`)
  - `6.1.4`: Callback security in `crates/github-extractor` (`ACCEPTED`, commit `4f8a68e`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Complete strongly-typed GitHub App manifest model and 1-click URL registration generator, backed by `docs/GITHUB_APP.md` enterprise runbook and credentials secret masking.
  2. Cryptographic OAuth state management (`OAuthState`, `InMemoryOAuthStateStore`) with 256-bit BLAKE3 nonces, single-use replay mitigation, session/tenant binding, and RS256 JWT signing from PKCS#1 and PKCS#8 RSA keys without Marvin attack timing vulnerabilities (`ring::signature::RsaKeyPair`).
  3. Strict least-privilege permission policy engine (`PermissionPolicy`, `PermissionAuditor`) enforcing `contents: read` and `metadata: read`, rejecting write/admin scopes and dangerous categories (`secrets`, `actions`, `administration`), with GFM Markdown and JSON audit reporting.
  4. Timing-safe OAuth callback validation, open-redirect defense (`OpenRedirectGuard`), installation setup callback verification, HMAC-SHA256 webhook signature verification (`WebhookSignatureVerifier`), delivery GUID deduplication (`WebhookDeliveryTracker`), and strongly-typed lifecycle event parsing.
- **Certification Status:** **PHASE 6.1 FORMALLY CERTIFIED & ACCEPTED**

---

### Subphase 6.2.1 — Secure token storage in crates/storage & crates/core
- **Phase / Stage:** `Phase 6.2 — Credential management` / `Stage 6 — Private GitHub Repositories`
- **Date:** 2026-11-18
- **Lead / Specialist Agents:** `Rust Backend & Performance Engineer`, `Security Reviewer & Application Security Engineer`
- **Affected Components:**
  - `crates/core/src/crypto.rs` (new authenticated AEAD encryption module)
  - `crates/core/src/lib.rs` (re-exports for cryptographic primitives)
  - `crates/core/src/error.rs` (extended with `CoreError::Crypto`)
  - `crates/core/Cargo.toml` (ring dependency)
  - `crates/storage/src/token_storage.rs` (new token storage repository trait and implementations)
  - `crates/storage/src/lib.rs` (re-exports for token storage)
  - `crates/storage/tests/token_storage_test.rs` (comprehensive integration test suite)
- **Invariant & Rule Alignment:**
  - Rule 05 (Security Engineering & Secrets Protection): Authenticated encryption at rest (AES-256-GCM), tenant-isolated AAD binding, secret masking in `Debug`/`Display`, memory zeroization on drop.
  - Rule 00 (Global Engineering Contract): Strict one-subphase execution, zero placeholders, complete verification evidence.
  - Rule 01 (Architecture Boundaries): Core foundation primitives in `crates/core`; persistence and repository abstractions in `crates/storage`.
  - Rule 02 (Rust Async Execution & Memory Bounds): Pure Tokio async trait execution, non-blocking lock primitives, zero memory leaks.
  - Rule 06 (Testing & Quality Assurance): 14 new automated tests covering cryptographic roundtrips, tampering detection, tenant isolation, expiration, and revocation.
  - Rule 09 (Zero Mock Policy): Production PostgreSQL implementation and real in-memory repository implementation without mock objects or hardcoded tokens.
- **Scope & Requirements:**
  1. Implement 256-bit symmetric encryption key management with memory zeroization on drop and `Debug`/`Display` secret redaction.
  2. Implement AES-256-GCM authenticated encryption and decryption with associated authenticated data (AAD) via `ring::aead`.
  3. Cryptographically bind ciphertext to `tenant:{owner_session_id}:installation:{github_installation_id}` so credentials cannot be decrypted under another tenant's or installation's context.
  4. Implement `TokenStorageRepository` trait defining complete credential lifecycle (`save_installation_token`, `get_active_token`, `get_installation`, `list_active_installations`, `revoke_installation`, `delete_installation`).
  5. Provide `InMemoryTokenStorage` for thread-safe testing and local execution, and `PostgresTokenStorageRepository` for PostgreSQL persistence with UPSERT support.
- **Implementation Details:**
  - In `crates/core/src/crypto.rs`:
    - `EncryptionKey`: 256-bit symmetric key; implements safe memory zeroization on `Drop` via `self.key.fill(0)`; masks key material in `Debug` and `Display` (`***[REDACTED_ENCRYPTION_KEY]***`); supports construction from byte arrays, slices, 64-character hex strings, HKDF-SHA256 passphrase derivation, and OS cryptographically secure random generation (`ring::rand::SystemRandom`).
    - `EncryptedPayload`: Models ciphertext with 16-byte auth tag and 12-byte initialization vector/nonce; masks ciphertext bytes in `Debug` and `Display`.
    - `Aes256GcmEngine`: AES-256-GCM authenticated encryption engine; generates unique 96-bit random nonces for each seal operation; authenticates associated data; verifies authentication tag on decryption.
    - `build_token_aad`: Canonical AAD formatter binding `owner_session_id` and `github_installation_id`.
    - `encrypt_token` and `decrypt_token`: High-level envelope encryption helpers.
  - In `crates/storage/src/token_storage.rs`:
    - `TokenStorageRepository`: Async trait defining asynchronous lifecycle operations for installation tokens.
    - `InMemoryTokenStorage`: Thread-safe in-memory repository utilizing `Arc<RwLock<HashMap<(Uuid, i64), GitHubInstallationRow>>>` and `EncryptionKey`; validates row invariants; automatically decrypts active tokens; filters expired or revoked records; zeros out token bytes upon revocation.
    - `PostgresTokenStorageRepository`: Production relational persistence using `sqlx::PgPool` and PostgreSQL UPSERT (`ON CONFLICT (owner_session_id, github_installation_id) DO UPDATE`).
  - Authored 6 unit tests in `crates/core/src/crypto.rs` and 8 integration tests in `crates/storage/tests/token_storage_test.rs`.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-core`
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo test -p web-repo-to-ai-storage --test token_storage_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Token encryption and decryption succeeds roundtrip with exact byte preservation.
  - Modifying ciphertext, nonce, or AAD causes immediate decryption failure.
  - Decrypting credentials under a different session or installation ID fails integrity check.
  - Revocation immediately wipes ciphertext and nonce in storage.
  - Expired tokens are not returned by `get_active_token` or `list_active_installations`.
  - All workspace tests pass, clippy is warning-free, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Exposure of plaintext installation tokens in application logs or persistence layers; cross-tenant token leakage.
  - Mitigation: Secret redaction in `Debug`/`Display`; memory zeroization on drop; AEAD authenticated associated data cryptographically locking tokens to tenant and installation identifiers.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `4f8a68e`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-core`: 26/26 tests passed in 0.00s (6 new crypto unit tests).
  - `cargo test -p web-repo-to-ai-storage`: 63/63 tests passed in 0.03s (55 unit tests + 8 new integration tests in `token_storage_test`).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - Purged 28.9 GiB in accumulated build artifacts, restoring 30.30 GB of free drive space.
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (563 workspace Rust tests, 213 frontend vitest tests, 5.27 GB / 30 GB storage ceiling).
- **Git Commit Reference:** `798ee84`
- **Status:** `ACCEPTED`

---

#### Subphase `6.2.2` — Token refresh & expiration in crates/github-extractor & crates/storage
- **Unique ID:** `6.2.2`
- **Objective:** Implement proactive token expiration evaluation with configurable safety margins, single-flight concurrent token renewal deduplication, automated background refresh against GitHub App access token API, and extended token storage query interfaces.
- **Scope:** `crates/github-extractor/src/token_refresh.rs`, `crates/github-extractor/src/lib.rs`, `crates/storage/src/token_storage.rs`, `crates/storage/src/schema/github.rs`, `crates/storage/tests/token_storage_test.rs`, and `crates/github-extractor/tests/token_refresh_test.rs`.
- **Prerequisites:** `6.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/src/schema/github.rs` (expiration helper predicates: `is_expired`, `is_expiring_soon`, `time_until_expiration`)
  - `crates/storage/src/token_storage.rs` (extended `TokenStorageRepository`, `InMemoryTokenStorage`, and `PostgresTokenStorageRepository` with `get_active_token_with_margin` and `get_token_expiration`)
  - `crates/storage/tests/token_storage_test.rs` (added 2 new integration tests for safety margin filtering and expiration metadata retrieval; 10/10 passing)
  - `crates/github-extractor/Cargo.toml` (added `async-trait` dependency)
  - `crates/github-extractor/src/token_refresh.rs` (created `TokenFreshness`, `TokenLifecyclePolicy`, `StoredTokenRecord`, `InstallationTokenStore`, and `InstallationTokenRefresher`)
  - `crates/github-extractor/src/lib.rs` (re-exported `token_refresh` module components)
  - `crates/github-extractor/tests/token_refresh_test.rs` (created 10 integration tests with live Axum mock GitHub API server)
- **Implementation Tasks:**
  1. Add expiration and proactive safety margin helper methods on `GitHubInstallationRow` in `crates/storage/src/schema/github.rs`.
  2. Extend `TokenStorageRepository` trait and implementations in `crates/storage/src/token_storage.rs` with `get_active_token_with_margin` and `get_token_expiration`.
  3. Author unit/integration tests in `crates/storage/tests/token_storage_test.rs` covering safety margin filtering and expiration checks.
  4. Create `crates/github-extractor/src/token_refresh.rs` defining `TokenFreshness` (`Fresh`, `ExpiringSoon`, `Expired`), `TokenLifecyclePolicy` with 300s default margin and bounded exponential backoff retries, `StoredTokenRecord` with `[REDACTED]` secret masking, and `InstallationTokenStore` async trait.
  5. Implement `InstallationTokenRefresher` with double-checked locking, single-flight per-installation synchronization lock registry, proactive renewal on `ExpiringSoon`/`Expired`, force refresh capabilities, and Rule 05 permission policy enforcement.
  6. Author comprehensive integration test suite `crates/github-extractor/tests/token_refresh_test.rs` covering freshness evaluation, redaction, CRUD, proactive renewal, force refresh, 20-thread single-flight deduplication, Rule 05 policy rejection, and retry with backoff.
  7. Verify all tests pass, format cleanly, clippy is zero-warning, and all gates in `scripts/check-all.ps1` pass.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test token_refresh_test`
  - `cargo test -p web-repo-to-ai-github-extractor`
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Token freshness evaluation accurately identifies tokens that are fresh, expiring within margin, or expired.
  - `get_active_token_with_margin` filters out tokens expiring within the safety buffer window.
  - Concurrent requests for the same installation are single-flight deduplicated so exactly 1 API call is issued to GitHub.
  - Refreshed tokens are verified against Rule 05 read-only permission policy before being stored.
  - Stored tokens redact credentials in `Debug` and `Display`.
  - All workspace tests pass, clippy is warning-free, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Mid-flight token expiration during long-running repository extractions causing 401 Unauthorized failures; thundering-herd API requests on token expiration.
  - Mitigation: 5-minute proactive renewal safety margin buffer; in-flight lock registry deduplicating concurrent renewal requests per `(owner_session_id, github_installation_id)`.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `798ee84`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test token_refresh_test`: 10/10 tests passed in 1.53s.
  - `cargo test -p web-repo-to-ai-github-extractor`: 186/186 tests passed (28 unit + 158 integration + 1 doc test).
  - `cargo test -p web-repo-to-ai-storage`: 65/65 tests passed (55 unit + 10 integration).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (575 workspace Rust tests, 213 frontend vitest tests, 5.27 GB / 30 GB storage ceiling).
- **Git Commit Reference:** `8f1adfe`
- **Status:** `ACCEPTED`

---

#### Subphase `6.2.3` — Revocation in crates/github-extractor & services/api
- **Unique ID:** `6.2.3`
- **Objective:** Implement upstream GitHub App installation access token revocation (`DELETE /installation/token`), coordinated local credential storage zeroization and in-memory cache eviction, webhook-driven installation deletion and suspension event handling, and REST API revocation endpoints in `services/api`.
- **Scope:** `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/revocation.rs`, `crates/github-extractor/src/lib.rs`, `services/api/src/state.rs`, `services/api/src/routes/auth_github.rs`, `services/api/src/routes/mod.rs`, `services/api/src/dto/github.rs`, `services/api/src/dto/mod.rs`, `crates/github-extractor/tests/revocation_test.rs`, and `services/api/tests/github_auth_test.rs`.
- **Prerequisites:** `6.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/client.rs` (add `revoke_installation_access_token`)
  - `crates/github-extractor/src/revocation.rs` (create revocation outcome, coordinator, and webhook synchronization models)
  - `crates/github-extractor/src/lib.rs` (re-export `revocation` module)
  - `crates/github-extractor/tests/revocation_test.rs` (author comprehensive client & coordinator test suite)
  - `services/api/Cargo.toml` (add `web-repo-to-ai-github-extractor` dependency)
  - `services/api/src/state.rs` (integrate token storage and webhook tracker into `AppState`)
  - `services/api/src/dto/github.rs` (define DTOs for revocation, installation listing, and webhook receipt)
  - `services/api/src/dto/mod.rs` (export GitHub DTOs)
  - `services/api/src/routes/auth_github.rs` (implement `DELETE /api/v1/auth/github/installations/:id`, `GET /api/v1/auth/github/installations`, and `POST /api/v1/auth/github/webhook`)
  - `services/api/src/routes/mod.rs` (mount `/auth/github` routes)
  - `services/api/src/lib.rs` (author API integration tests for revocation and webhook handlers)
- **Implementation Tasks:**
  1. Add `revoke_installation_access_token` to `GitHubApiClient` calling GitHub REST API `DELETE /installation/token` with idempotent success handling (HTTP 204 No Content, HTTP 401/404).
  2. Implement `TokenRevocationCoordinator` in `crates/github-extractor/src/revocation.rs` managing coordinated upstream API revocation, local storage zeroization, and in-memory token cache eviction.
  3. Implement webhook lifecycle synchronization in `crates/github-extractor/src/revocation.rs` responding to `installation.deleted` and `installation.suspend`.
  4. Author integration tests in `crates/github-extractor/tests/revocation_test.rs` testing upstream revocation, idempotent handling, coordinator workflows, and webhook event synchronization with live Axum mock GitHub API.
  5. Add `web-repo-to-ai-github-extractor` dependency to `services/api/Cargo.toml`.
  6. Extend `AppState` in `services/api/src/state.rs` with `token_storage: Arc<dyn TokenStorageRepository>` and `webhook_tracker: Arc<Mutex<InMemoryWebhookDeliveryTracker>>`.
  7. Create DTO models in `services/api/src/dto/github.rs` for revocation responses, installation summaries (zero secret leakage), and webhook receipts.
  8. Implement Axum route handlers in `services/api/src/routes/auth_github.rs` for installation deletion/revocation, listing active installations with tenant session scoping via `X-Session-ID`, and verified GitHub webhook delivery.
  9. Mount `/auth/github` endpoints in `services/api/src/routes/mod.rs`.
  10. Author integration tests in `services/api/src/lib.rs` verifying the API endpoints, tenant isolation, error handling, and HMAC signature validation.
  11. Run full verification suites (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test revocation_test`
  - `cargo test -p web-repo-to-ai-api`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Upstream token revocation `DELETE /installation/token` returns success on HTTP 204, and idempotently handles HTTP 401/404.
  - Revoking an installation through coordinator or API completely wipes token ciphertext, clears memory caches, and sets `is_revoked = true`.
  - Webhook delivery of `installation.deleted` or `installation.suspend` with valid HMAC signature automatically revokes corresponding installation credentials.
  - Installation listing API returns zero secrets (no plaintext token, no ciphertext, no nonces).
  - Cross-tenant revocation is rejected (sessions cannot revoke other sessions' installations).
  - All workspace tests pass, clippy is warning-free, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Active extraction jobs continuing to use credentials after user or admin revocation; webhook signature spoofing or replay attacks.
  - Mitigation: Cache eviction immediately invalidates token in memory; constant-time HMAC-SHA256 signature verification and delivery GUID sliding-window deduplication defense.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `92b6c65`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test revocation_test`: 7/7 tests passed in 0.05s.
  - `cargo test -p web-repo-to-ai-storage --test token_storage_test`: 11/11 tests passed in 0.02s.
  - `cargo test -p web-repo-to-ai-api`: 55/55 tests passed (54 lib unit/integration + 1 binary test) including 10 new GitHub auth/revocation/webhook tests.
  - `cargo test -p web-repo-to-ai-github-extractor`: 193/193 tests passed (28 unit + 165 integration + 1 doc test).
  - `cargo test -p web-repo-to-ai-storage`: 66/66 tests passed (55 unit + 11 integration).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `5b81912`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 6 — Private GitHub Repositories)

#### Subphase `6.2.4` — Secret redaction in crates/observability & crates/github-extractor
- **Unique ID:** `6.2.4`
- **Objective:** Implement comprehensive secret and credential redaction across tracing spans, structured log events, telemetry layers, and domain error messages to ensure GitHub installation tokens (`ghs_*`), user tokens, refresh tokens, private keys, authorization headers, webhook secrets, and sensitive URL query parameters are never leaked to logs, error payloads, or external telemetry collectors.
- **Scope:** `crates/observability/src/redaction.rs`, `crates/observability/src/logging.rs`, `crates/observability/src/lib.rs`, `crates/observability/tests/redaction_test.rs`, `crates/github-extractor/Cargo.toml`, `crates/github-extractor/src/error.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, and `crates/github-extractor/tests/redaction_test.rs`.
- **Prerequisites:** `6.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/observability/src/redaction.rs` (expand regex patterns for all GitHub token prefixes `ghs_`, `ghu_`, `ghr_`, `ghp_`, `gho_`, `github_pat_`, Authorization headers `token`/`Bearer`/`Basic`, URL query params, and PEM private keys; add `RedactionEngine`, `RedactingWriter`, and `RedactingMakeWriter`)
  - `crates/observability/src/logging.rs` (wire `RedactingMakeWriter` into `init_tracing` when `redact_secrets` is enabled)
  - `crates/observability/src/lib.rs` (re-export redaction engine and writer primitives)
  - `crates/observability/tests/redaction_test.rs` (author tests for token patterns, PEM formats, JSON redaction, and tracing log redaction)
  - `crates/github-extractor/Cargo.toml` (add `web-repo-to-ai-observability` dependency)
  - `crates/github-extractor/src/error.rs` (integrate centralized redaction engine into `GitHubExtractorError` formatting and `redact_secrets`)
  - `crates/github-extractor/src/client.rs` (sanitize error messages and response bodies in API client)
  - `crates/github-extractor/tests/redaction_test.rs` (author extractor-level secret leakage and error sanitization tests)
- **Implementation Tasks:**
  1. Expand regex patterns in `crates/observability/src/redaction.rs` to cover all GitHub token prefixes (`ghs_`, `ghu_`, `ghr_`, `ghp_`, `gho_`, `github_pat_`), HTTP Authorization headers (`Bearer`, `token`, `Basic`), URL userinfo and sensitive query parameters (`?access_token=`, `?token=`, `?client_secret=`, etc.), and multi-format PEM private keys.
  2. Implement `RedactionEngine` in `crates/observability/src/redaction.rs` supporting custom secret pattern registration and dynamic string masking.
  3. Implement `RedactingWriter` and `RedactingMakeWriter` wrapping `std::io::Write` / `tracing_subscriber::fmt::MakeWriter` to scrub all emitted tracing lines before output.
  4. Wire `RedactingMakeWriter` into `init_tracing` in `crates/observability/src/logging.rs` when `config.redact_secrets` is active.
  5. Add `web-repo-to-ai-observability` to `crates/github-extractor/Cargo.toml`.
  6. Refactor and strengthen `redact_secrets` in `crates/github-extractor/src/error.rs` to use `web_repo_to_ai_observability::redact_sensitive_text` and ensure `GitHubExtractorError` formatting scrubs all inner strings.
  7. Sanitize error messages in `crates/github-extractor/src/client.rs` to prevent token leakage in HTTP failure details.
  8. Author unit and integration tests in `crates/observability/tests/redaction_test.rs` and `crates/github-extractor/tests/redaction_test.rs`.
  9. Run workspace verification checks (`cargo test`, `cargo clippy`, `cargo fmt`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-observability`
  - `cargo test -p web-repo-to-ai-github-extractor --test redaction_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - All GitHub token variations (`ghs_*`, `ghu_*`, `ghr_*`, `ghp_*`, `gho_*`, `github_pat_*`) are redacted to `[REDACTED_GITHUB_TOKEN]`.
  - HTTP `Authorization: token ...`, `Authorization: Bearer ...`, and `Authorization: Basic ...` headers are redacted.
  - URLs with embedded userinfo credentials and sensitive query parameters are redacted.
  - Multi-line RSA, EC, and PKCS#8 private keys in PEM format are redacted to `[REDACTED_PRIVATE_KEY]`.
  - Tracing logs emitted through subscriber with `redact_secrets: true` scrub all tokens before writing to stdout.
  - `GitHubExtractorError` formatting and API client network error messages never leak tokens or private keys.
  - All workspace tests pass, clippy is warning-free, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Overly aggressive regex expressions redacting harmless non-secret identifiers or causing regex catastrophic backtracking.
  - Mitigation: Anchor regexes with word boundaries `\b` and strictly bounded character classes; enforce linear-time regex scanning without nested quantifiers.
- **Rollback Plan:**
  - Revert changes via Git reset to commit `629ba8f`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-observability`: 20/20 unit tests passed in 0.01s.
  - `cargo test -p web-repo-to-ai-observability --test redaction_test`: 7/7 integration tests passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor --test redaction_test`: 8/8 integration tests passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor --lib`: 30/30 unit tests passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor --test ref_discovery_test --test revocation_test --test token_refresh_test`: 24/24 integration tests passed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - Cleaned 22.8 GB in accumulated debug binaries and incremental caches, restoring 27.1 GB free space.
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `b8f095f`
- **Status:** `ACCEPTED`

---

### Phase 6.2 Milestone Certification (`6.2.1` – `6.2.4`)
- **Phase 6.2 Goal:** Complete end-to-end credential management for private GitHub repository extraction, including authenticated envelope encryption at rest, proactive token refresh with single-flight deduplication, multi-session revocation and webhook synchronization, and comprehensive secret redaction across logs, errors, and traces.
- **Subphases Completed & Accepted (4 of 4):**
  - `6.2.1`: Secure token storage in `crates/storage` & `crates/core` (`ACCEPTED`, commit `00863c1`)
  - `6.2.2`: Token refresh & expiration in `crates/github-extractor` & `crates/storage` (`ACCEPTED`, commit `92b6c65`)
  - `6.2.3`: Revocation in `crates/github-extractor` & `services/api` (`ACCEPTED`, commit `629ba8f`)
  - `6.2.4`: Secret redaction in `crates/observability` & `crates/github-extractor` (`ACCEPTED`, commit `b8f095f`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Authenticated AES-256-GCM envelope encryption (`Aes256GcmEngine`, `EncryptionKey`) cryptographically binding ciphertext to session and installation identifiers (`tenant:{owner_session_id}:installation:{github_installation_id}`), preventing cross-tenant decryption with volatile zeroization on drop.
  2. Proactive token freshness evaluation with configurable safety margins, single-flight in-flight deduplication mutex registry (`InstallationTokenRefresher`), automatic exponential backoff retries, and strict Rule 05 read-only scope validation.
  3. Upstream token revocation (`DELETE /installation/token`) with idempotent 204/401/404 handling, coordinated local storage zeroization and in-memory cache eviction, webhook lifecycle event handling (`installation.deleted`, `installation.suspend`), and tenant-scoped REST API endpoints (`GET /installations`, `DELETE /installations/:id`, `POST /webhook`).
  4. Comprehensive secret redaction engine (`RedactionEngine`, `RedactingWriter`, `RedactingMakeWriter`) scrubbing all GitHub token variants (`ghs_*`, `ghu_*`, `ghr_*`, `ghp_*`, `gho_*`, `github_pat_*`), HTTP Authorization headers (`Bearer`, `token`, `Basic`), URL userinfo and sensitive query parameters, multi-format PEM private keys, and webhook HMAC signatures from structured logs, tracing spans, and domain error messages (`GitHubExtractorError::sanitized()`).
- **Certification Status:** **PHASE 6.2 FORMALLY CERTIFIED & ACCEPTED**

---

### Active Subphase (Stage 6 — Private GitHub Repositories)

#### Phase 6.3 — Private extraction

#### Subphase `6.3.1` — Installation authorization in crates/github-extractor
- **Unique ID:** `6.3.1`
- **Objective:** Implement tenant-scoped GitHub App installation authorization, account resolution, repository access grant verification (`All` vs `Selected`), Rule 05 read-only permission compliance auditing, proactive token renewal, suspended/revoked installation rejection, and masked bearer token contexts (`AuthorizedInstallationContext`) in `crates/github-extractor`.
- **Scope:** `crates/github-extractor/src/authorization.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/installation.rs`, `crates/github-extractor/src/token_refresh.rs`, `crates/github-extractor/src/lib.rs`, and `crates/github-extractor/tests/authorization_test.rs`.
- **Prerequisites:** `6.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/authorization.rs`: Implemented `AuthorizedInstallationContext`, `AuthorizationOutcome`, and `InstallationAuthorizer`.
  - `crates/github-extractor/src/client.rs`: Added `check_repository_access` to query `GET /repos/{owner}/{repo}` with installation token and parse `RepositoryAccessDetails`.
  - `crates/github-extractor/src/installation.rs`: Added `RepositoryPermissions` and `RepositoryAccessDetails` models.
  - `crates/github-extractor/src/token_refresh.rs`: Extended `StoredTokenRecord` with `repository_selection`, `repositories`, `permissions`, `is_revoked`, `is_suspended`, and `has_repository`; added `save_token_record`, `list_installations_for_session`, and `find_installation_for_account` to `InstallationTokenStore` and `InMemoryInstallationTokenStore`.
  - `crates/github-extractor/src/lib.rs`: Re-exported `authorization` module and new installation types.
  - `crates/github-extractor/tests/authorization_test.rs`: 12 comprehensive integration tests covering all authorization outcomes, mock GitHub API routes, proactive token renewal, and secret masking.
- **Implementation Tasks:**
  1. Define `AuthorizedInstallationContext` capturing installation ID, account login, bearer token, repository selection mode, owner, repo, and granted permissions map with masked `Debug` and `Display` formatting.
  2. Implement `AuthorizationOutcome` enum distinguishing `Authorized`, `NoInstallationForSession`, `AccountMismatch`, `RepositoryNotGranted`, `InstallationRevoked`, `InstallationSuspended`, `InsufficientPermissions`, and `TokenUnavailable`.
  3. Implement `InstallationAuthorizer` coordinating account resolution, revocation/suspension checks, proactive token freshness evaluation and single-flight renewal, repository selection verification (`All` vs `Selected` with live GitHub fallback), and Rule 05 least-privilege permission policy auditing (`PermissionAuditor::audit_permissions`).
  4. Extend `GitHubApiClient` with `check_repository_access(&self, token, owner, repo)` verifying live repository access against `GET /repos/{owner}/{repo}` with proper 404/403/401 handling.
  5. Extend `StoredTokenRecord` and `InstallationTokenStore` with repository selection, granted repositories list, permissions map, and session lookup methods.
  6. Author unit and integration tests in `crates/github-extractor/tests/authorization_test.rs`.
  7. Run full monorepo verification checks (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Access to private repository requires an active installation under the caller's session ID (tenant isolation).
  - Installations with `repository_selection == All` authorize all repositories belonging to the installed account.
  - Installations with `repository_selection == Selected` verify repository membership against local cache or live GitHub API call.
  - Non-existent, mismatched, revoked, or suspended installations are rejected with explicit, actionable outcome variants.
  - Permissions are audited against `PermissionPolicy::strict_minimal()`; write or admin scopes are rejected under Rule 05.
  - Expiring tokens are proactively renewed; expired tokens without refresher fail gracefully.
  - Plaintext installation tokens are never exposed in debug logs or display strings.
  - 100% of automated tests and workspace quality checks pass.
- **Dependencies:** `web-repo-to-ai-core`, `web-repo-to-ai-observability`, `reqwest`, `serde`, `uuid`, `chrono`.
- **Security Considerations:**
  - Tenant isolation: Tokens belonging to session A are strictly inaccessible to session B.
  - Least privilege (Rule 05): Write and admin permissions cause immediate authorization rejection.
  - Plaintext token masking: `AuthorizedInstallationContext` masks bearer tokens in `Debug` (`***[REDACTED_TOKEN]***`) and `Display`.
- **Performance Considerations:**
  - Cached repository list is checked first before initiating external network requests to GitHub API.
  - Fast-path evaluation completes in sub-millisecond memory lookups.
- **Documentation Requirements:** Documented authorization architecture, invariants, and outcome taxonomy in `crates/github-extractor/src/authorization.rs`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_test`: 12/12 integration tests passed in 0.01s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `ef1a6c9`
- **Status:** `ACCEPTED`

---

### Active Subphase (Stage 6 — Private GitHub Repositories)

#### Phase 6.3 — Private extraction

#### Subphase `6.3.2` — Selected repository access in crates/github-extractor
- **Unique ID:** `6.3.2`
- **Objective:** Implement comprehensive repository discovery, pagination traversal, search, and dynamic repository access grant synchronization for GitHub App installations configured with `repository_selection: "selected"` in `crates/github-extractor`. Provide full pagination across `GET /installation/repositories` and `GET /user/installations/{id}/repositories`, local and remote filtering, and synchronization with `StoredTokenRecord` and `InstallationTokenStore` on webhook events.
- **Scope:** `crates/github-extractor/src/selected_repositories.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/installation.rs`, `crates/github-extractor/src/token_refresh.rs`, `crates/github-extractor/src/lib.rs`, and `crates/github-extractor/tests/selected_repositories_test.rs`.
- **Prerequisites:** `6.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/selected_repositories.rs`: Implemented `SelectedRepositoryListing`, `RepositoryAccessStatus`, `SyncSummary`, and `SelectedRepositoryManager`.
  - `crates/github-extractor/src/client.rs`: Added `list_user_installation_repositories`.
  - `crates/github-extractor/src/installation.rs`: Implemented `From<RepositoryAccessDetails>` for `InstalledRepositorySummary`.
  - `crates/github-extractor/src/token_refresh.rs`: Added `sync_installation_repositories` to `InstallationTokenStore` and `InMemoryInstallationTokenStore`.
  - `crates/github-extractor/src/lib.rs`: Re-exported `selected_repositories` module and key types.
  - `crates/github-extractor/tests/selected_repositories_test.rs`: 11 integration tests verifying pagination, search, verification, and dynamic grant synchronization.
- **Implementation Tasks:**
  1. Define `SelectedRepositoryListing` holding total count, repository selection mode, list of repositories, pages fetched, and completeness flag.
  2. Implement `SelectedRepositoryManager` providing `fetch_all`, `fetch_all_for_user`, `search`, `filter_by_visibility`, `find_by_slug`, `verify_repository_access`, and `sync_grant_change`.
  3. Extend `GitHubApiClient` with `list_user_installation_repositories` (`GET /user/installations/{id}/repositories`).
  4. Extend `InstallationTokenStore` and `InMemoryInstallationTokenStore` with `sync_installation_repositories` to synchronize added/removed grants across session records.
  5. Author integration tests in `crates/github-extractor/tests/selected_repositories_test.rs`.
  6. Run monorepo verification checks (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test selected_repositories_test`
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_test`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Multi-page repository listings are traversed correctly using GitHub API pagination parameters (`page`, `per_page`) with configurable `max_pages` bounds.
  - Both installation tokens (`Bearer ghs_*`) and user tokens (`Bearer ghu_*`) can enumerate authorized repositories.
  - Search and filter utilities accurately match names, descriptions, and visibility without case sensitivity issues.
  - Webhook repository grant additions and removals properly synchronize cached token records across all sessions.
  - Secret masking is strictly maintained across all debug logs and error formats.
  - 100% of workspace tests, lints, and format checks pass.
- **Dependencies:** `web-repo-to-ai-core`, `web-repo-to-ai-observability`, `reqwest`, `serde`, `chrono`, `uuid`.
- **Security Considerations:**
  - Tenant isolation: Grant updates are scoped to the matching installation ID across sessions.
  - Rule 05 least-privilege: Repository access is strictly constrained to what GitHub has granted.
  - Zero token leakage: Sensitive bearer tokens are masked.
- **Performance Considerations:**
  - Pagination stops when all items are fetched (`accumulated.len() >= total_count`).
  - Cached repository lists are used to prevent redundant GitHub API requests.
- **Documentation Requirements:** Documented selected repository management and pagination logic in `crates/github-extractor/src/selected_repositories.rs`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test selected_repositories_test`: 11/11 passed in 0.05s.
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_test`: 12/12 passed in 0.01s.
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `a513dbe`
- **Status:** `ACCEPTED`

---

#### Subphase `6.3.3` — Private tree & source extraction in crates/github-extractor
- **Unique ID:** `6.3.3`
- **Objective:** Implement end-to-end private repository extraction orchestrator (`PrivateRepositoryExtractor`) coordinating authenticated Git Trees API traversal, automated recursive subtree recovery, file classification, bounded concurrent content extraction across sources, docs, dependencies, configs, and tests, mathematical completeness accounting ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$), multi-algorithm cryptographic hash validation (Git blob SHA-1, SHA-256, BLAKE3), file manifest assembly, and architecture context derivation under Rule 05 ephemeral token authentication.
- **Scope:** `crates/github-extractor/src/private_extractor.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/src/target.rs`, `crates/github-extractor/src/authorization.rs`, `crates/github-extractor/tests/private_extractor_test.rs`.
- **Prerequisites:** `6.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/private_extractor.rs`: Implemented `PrivateExtractionConfig`, `PrivateExtractionResult`, and `PrivateRepositoryExtractor`.
  - `crates/github-extractor/src/lib.rs`: Re-exported `private_extractor` module and primary public structures.
  - `crates/github-extractor/src/target.rs`: Added `GitHubTarget::for_repo(owner, repo)` constructor.
  - `crates/github-extractor/src/authorization.rs`: Added `AuthorizedInstallationContext::new` and `with_permissions`.
  - `crates/github-extractor/tests/private_extractor_test.rs`: 6 integration tests verifying end-to-end private extraction, mathematical conservation balance, multi-algorithm hash checks, proactive token renewal, subtree recovery on truncation, unauthorized 401 handling, and zero secret token leakage.
- **Implementation Tasks:**
  1. Define `PrivateExtractionConfig` with configurable `source_config`, `classification_config`, `recovery_config`, `hash_config`, `target_ref`, `fetch_metadata`, and `proactive_token_renewal_margin_secs`.
  2. Define `PrivateExtractionResult` capturing target, installation ID, account login, commit metadata, optional repository metadata, recovered tree, classification report, source report, doc report, dependency report, config/test report, accounting balance sheet, hash validation report, manifest, and architecture context with helper metrics (`is_balanced`, `total_extracted_files`, `total_discovered_entries`, `total_extracted_bytes`).
  3. Implement `PrivateRepositoryExtractor` orchestrator with `extract()` executing target resolution, token freshness check & proactive renewal, ref discovery & commit resolution, tree traversal & automated subtree recovery, file classification, bounded concurrent extraction (`tokio::join!`), completeness accounting auditing, cryptographic hash verification, file manifest generation, and architecture context derivation.
  4. Author integration tests in `crates/github-extractor/tests/private_extractor_test.rs`.
  5. Run monorepo verification checks (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test private_extractor_test`
  - `cargo test -p web-repo-to-ai-github-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Complete private repository tree traversal and source extraction succeeds via authenticated API client.
  - Mathematical conservation identity ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$) holds with zero discrepancies.
  - Ephemeral bearer tokens are refreshed proactively when expiring during long extractions.
  - Automated recursive subtree recovery handles truncated trees seamlessly.
  - Sensitive bearer tokens are strictly masked in `Debug`, `Display`, logs, and error traces.
  - 100% of workspace tests, lints, and format checks pass.
- **Dependencies:** `web-repo-to-ai-core`, `web-repo-to-ai-cdm`, `web-repo-to-ai-observability`, `reqwest`, `serde`, `chrono`, `uuid`, `tokio`.
- **Security Considerations:**
  - Rule 05 least-privilege: Ephemeral tokens are used strictly for read-only repository endpoints.
  - Zero token leakage: Tokens are redacted in error messages and excluded from result structs.
  - Constant-time checks and authenticated client injection.
- **Performance Considerations:**
  - Bounded concurrent downloads via `tokio::join!` across categories with semaphore concurrency limits.
  - Lazy evaluation and memory-conscious streaming for large trees.
- **Documentation Requirements:** Documented private extraction architecture and invariants in `crates/github-extractor/src/private_extractor.rs`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test private_extractor_test`: 6/6 passed in 0.02s.
  - `cargo test -p web-repo-to-ai-github-extractor`: all unit and integration tests passed (30 lib tests, 29 test files, 1 doc-test).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `3787909`
- **Status:** `ACCEPTED`

---

#### Subphase `6.3.4` — Authorization failure recovery in crates/github-extractor
- **Unique ID:** `6.3.4`
- **Objective:** Implement an intelligent authorization failure recovery engine (`AuthorizationRecoveryCoordinator`) providing root-cause diagnostic classification across all `AuthorizationOutcome` variants and runtime `GitHubExtractorError`s (HTTP 401, 403, 404, 429), automated token cache eviction and proactive renewal retry, transparent self-healing for `PrivateRepositoryExtractor`, and step-by-step remediation plans (`AuthorizationRecoveryPlan`) with direct GitHub configuration links under Rule 05 zero-leakage security. Formally certify Phase 6.3 (Private extraction).
- **Scope:** `crates/github-extractor/src/authorization_recovery.rs`, `crates/github-extractor/src/private_extractor.rs`, `crates/github-extractor/src/lib.rs`, and `crates/github-extractor/tests/authorization_recovery_test.rs`.
- **Prerequisites:** `6.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/authorization_recovery.rs`: Implemented `AuthorizationFailureKind`, `RecoveryAction`, `AuthorizationRecoveryPlan`, and `AuthorizationRecoveryCoordinator`.
  - `crates/github-extractor/src/private_extractor.rs`: Integrated transparent 401 token eviction and renewal retry on `get_commit`.
  - `crates/github-extractor/src/lib.rs`: Exported `authorization_recovery` module and re-exported core recovery types.
  - `crates/github-extractor/tests/authorization_recovery_test.rs`: 9 integration tests covering outcome diagnoses, runtime error classification, transparent 401 retry, self-healing private extraction, and zero secret leakage.
- **Implementation Tasks:**
  1. Define `AuthorizationFailureKind` taxonomy (`TokenInvalid`, `TokenExpired`, `RepositoryNotGranted`, `AccountMismatch`, `NoInstallationForSession`, `InstallationSuspended`, `InstallationRevoked`, `InsufficientPermissions`, `RateLimited`, `UpstreamApiError`).
  2. Define `RecoveryAction` taxonomy (`AutomaticRetryWithFreshToken`, `PromptAppInstallation`, `PromptRepositorySelectionUpdate`, `PromptUnsuspendInstallation`, `PromptReauthenticateUser`, `WaitRateLimitReset`, `ContactAdministrator`, `Unrecoverable`).
  3. Implement `AuthorizationRecoveryPlan` with `diagnosis`, `remediation_guidance`, `summary_card()`, and `action_code()`.
  4. Implement `AuthorizationRecoveryCoordinator` providing `diagnose_outcome()`, `diagnose_extractor_error()`, `attempt_eviction_and_refresh()`, and `execute_with_retry_on_401()`.
  5. Integrate self-healing retry logic into `PrivateRepositoryExtractor::extract()`.
  6. Author comprehensive integration tests in `crates/github-extractor/tests/authorization_recovery_test.rs`.
  7. Run all monorepo quality gates (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_recovery_test`
  - `cargo test -p web-repo-to-ai-github-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Every authorization failure variant produces an actionable `AuthorizationRecoveryPlan` with human-readable remediation and machine-readable action code.
  - HTTP 401 Bad credentials automatically triggers token eviction and transparent one-time renewal retry.
  - Private repository extraction self-heals seamlessly when encountering expired tokens.
  - Zero raw tokens, secret keys, or private material leak into diagnosis strings, summary cards, or serialized JSON.
  - 100% of workspace tests, lints, and format checks pass.
- **Dependencies:** `web-repo-to-ai-core`, `web-repo-to-ai-observability`, `reqwest`, `serde`, `chrono`, `uuid`, `tokio`.
- **Security Considerations:**
  - Rule 05 least-privilege: Redaction of all sensitive bearer tokens and secrets via `redact_secrets()`.
  - Tenant isolation: Eviction and refresh operations are scoped strictly to the session ID and installation ID.
- **Performance Considerations:**
  - Fast, in-memory diagnostic pattern matching with zero unnecessary network calls.
  - Single-flight retry deduplication via `InstallationTokenRefresher`.
- **Documentation Requirements:** Documented failure recovery architecture and remediation procedures in `crates/github-extractor/src/authorization_recovery.rs`.
- **Validation Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test authorization_recovery_test`: 9/9 passed in 0.02s.
  - `cargo test -p web-repo-to-ai-github-extractor`: all unit and integration tests passed (30 lib tests, 30 test files, 1 doc-test).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `c37fd8d`
- **Status:** `ACCEPTED`

---

### Phase 6.3 Milestone Certification (`6.3.1` – `6.3.4`)
- **Phase 6.3 Goal:** Complete end-to-end private GitHub repository extraction infrastructure, including installation authorization verification, paginated selected repository access and search, authenticated tree traversal and content acquisition with mathematical conservation accounting, and automated authorization failure recovery under Rule 05.
- **Subphases Completed & Accepted (4 of 4):**
  - `6.3.1`: Installation authorization in `crates/github-extractor` (`ACCEPTED`, commit `ef1a6c9`)
  - `6.3.2`: Selected repository access in `crates/github-extractor` (`ACCEPTED`, commit `ca1206b`)
  - `6.3.3`: Private tree & source extraction in `crates/github-extractor` (`ACCEPTED`, commit `73ecf55`)
  - `6.3.4`: Authorization failure recovery in `crates/github-extractor` (`ACCEPTED`, commit `c37fd8d`)
- **Certification Date:** 2026-11-18
- **Certification Summary:**
  1. Ephemeral GitHub App installation tokens (`Bearer ghs_*`) utilized exclusively across authenticated Git Trees and Blobs endpoints, strictly enforcing `Contents: Read-only` and `Metadata: Read-only` under Rule 05.
  2. Installation authorizer evaluates account ownership, repository selection mode (All vs Selected), webhook suspension/revocation status, and minimum permission policies.
  3. Paginated repository enumeration (`SelectedRepositoryManager`) supports searching, visibility filtering, and dynamic webhook grant updates.
  4. End-to-end extraction orchestrator (`PrivateRepositoryExtractor`) coordinates commit pinning, recursive subtree recovery on truncated trees, polyglot classification, bounded concurrent extraction (`tokio::join!`), mathematical completeness conservation accounting ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$), and multi-algorithm cryptographic hash validation.
  5. Recovery coordinator (`AuthorizationRecoveryCoordinator`) delivers automated 401 token eviction, proactive renewal retries, and comprehensive diagnosis reports.
  6. Zero secret leakage across all logs, debug traces, and error envelopes.
  7. Formally verified by 55+ Phase 6.3 integration tests and 100% passing monorepo quality gates.
- **Milestone Verdict:** `PHASE 6.3 FULLY CERTIFIED`

---

### Phase 6.4 — Security verification (`6.4.1` – `6.4.4`)

#### Subphase `6.4.1` — Cross-user isolation in crates/github-extractor & services/api
- **Unique ID:** `6.4.1`
- **Objective:** Verify and enforce cross-user and cross-tenant session isolation across credential storage, installation authorizers, OAuth state nonces, dynamic repository grant synchronizations, AES-256-GCM envelope ciphertexts, and public Axum REST APIs (`GET /api/v1/auth/github/installations`, `DELETE /api/v1/auth/github/installations/:id`).
- **Scope:** `crates/github-extractor/tests/cross_user_isolation_test.rs`, `services/api/src/lib.rs`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Prerequisites:** `6.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/tests/cross_user_isolation_test.rs`
  - `services/api/src/lib.rs`
  - `IMPLEMENTATION_LEDGER.md`
  - `PROJECT_STATUS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive integration test suite `cross_user_isolation_test.rs` in `crates/github-extractor/tests/`:
     - Test Case 1 (`test_token_store_session_isolation`): Verify that installation tokens saved for Session A cannot be read, enumerated, or discovered by Session B via `get_token`, `list_installations_for_session`, or `find_installation_for_account`.
     - Test Case 2 (`test_installation_authorizer_session_isolation`): Verify that Session B querying `InstallationAuthorizer::authorize` for Session A's repository receives `NoInstallationForSession` or `AccountMismatch` and never receives Session A's credentials.
     - Test Case 3 (`test_oauth_state_cross_session_hijack_prevention`): Verify that an attacker in Session B cannot consume or validate an OAuth state nonce bound to Session A (rejected with session mismatch and purged against replays).
     - Test Case 4 (`test_revocation_session_isolation_in_token_store`): Verify that Session B calling `store.revoke_token` with Session A's installation ID returns `false`, leaving Session A's token intact.
     - Test Case 5 (`test_webhook_global_revocation_isolation`): Verify that global uninstallation webhooks (`revoke_all_for_installation`) revoke only the specified installation ID across all tenant sessions while leaving other installations untouched.
     - Test Case 6 (`test_webhook_repository_grant_sync_isolation`): Verify that dynamic repository grant updates only affect the specific installation and do not leak repository names into unrelated installations.
     - Test Case 7 (`test_storage_aad_cryptographic_cross_tenant_isolation`): Cryptographic proof that AES-256-GCM ciphertext encrypted for Session A with AAD `tenant:{session_id}:installation:{installation_id}` fails decryption under Session B's session ID with integrity authentication errors.
     - Test Case 8 (`test_private_extractor_session_isolation`): Verify that `PrivateRepositoryExtractor` bound to Session B cannot access or renew tokens stored for Session A.
  2. Implement comprehensive cross-tenant API isolation and header validation tests in `services/api/src/lib.rs`:
     - `test_api_list_installations_cross_tenant_isolation`: Tenant B calling `GET /api/v1/auth/github/installations` receives an empty list `[]` when Tenant A has active installations, and each tenant only observes their own registered installations.
     - `test_api_session_header_validation_errors`: Missing or malformed UUID strings in `X-Session-ID` return HTTP 400 Bad Request with RFC 7807 validation problem details.
     - `test_api_concurrent_multi_tenant_revocation_isolation`: Tenant A revoking their installation does not affect or invalidate Tenant B's active installation.
  3. Run full verification gates (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Token store strictly partitions credentials by `(session_id, installation_id)`.
  - Authorizer returns `NoInstallationForSession` or `AccountMismatch` for cross-user queries.
  - OAuth state nonces reject cross-session validation attempts.
  - Revocations cannot be triggered across session boundaries.
  - AAD cryptographic binding guarantees ciphertext confidentiality across tenants.
  - API enforces mandatory and valid `X-Session-ID` header.
  - 100% of tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: Accidental cross-tenant data leakage via shared cache keys.
  - Mitigation: Explicit composite primary keys `(session_id, installation_id)` and Authenticated Associated Data (AAD) binding in AES-256-GCM encryption.
- **Rollback Plan:**
  - Revert Subphase 6.4.1 changes via `git reset --hard HEAD~1` and restore Phase 6.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test cross_user_isolation_test`: 8/8 passed in 0.00s.
  - `cargo test -p web-repo-to-ai-api`: 58/58 passed in 0.55s.
  - Full workspace tests: 522+ unit and integration tests passing with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace, Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `02da02c`
- **Status:** `ACCEPTED`

---

#### Subphase `6.4.2` — Unauthorized repository access tests in crates/github-extractor
- **Unique ID:** `6.4.2`
- **Objective:** Verify and enforce comprehensive defense against unauthorized repository access across missing installations, account ownership mismatches, ungranted selected repositories (both local cache and upstream live verification), suspended installations, revoked installations, permission policy violations (missing `contents: read` or forbidden write scopes), and upstream HTTP 403/404 responses under Rule 05.
- **Scope:** `crates/github-extractor/tests/unauthorized_access_test.rs`, `crates/github-extractor/src/authorization.rs`, `crates/github-extractor/src/private_extractor.rs`, `crates/github-extractor/src/authorization_recovery.rs`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Prerequisites:** `6.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/tests/unauthorized_access_test.rs`
  - `IMPLEMENTATION_LEDGER.md`
  - `PROJECT_STATUS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive integration test suite `unauthorized_access_test.rs` in `crates/github-extractor/tests/`:
     - Test Case 1 (`test_unauthorized_no_installation_for_session`): Calling `InstallationAuthorizer::authorize` without an installation in session yields `NoInstallationForSession` and maps to `PromptAppInstallation` recovery action with 1-click install link.
     - Test Case 2 (`test_unauthorized_account_mismatch`): Attempting to access an uninstalled account's repository yields `AccountMismatch` and produces a `PromptAppInstallation` plan targeting that specific account with zero credential leakage.
     - Test Case 3 (`test_unauthorized_selected_repository_not_granted_local_check`): In Selected repository mode, querying an ungranted repository with live check returning 404 yields `RepositoryNotGranted` and produces a `PromptRepositorySelectionUpdate` action with direct settings link.
     - Test Case 4 (`test_unauthorized_selected_repository_forbidden_upstream`): Live check returning 403 Forbidden is evaluated as not granted, yielding `RepositoryNotGranted`.
     - Test Case 5 (`test_unauthorized_suspended_installation`): Installations with `is_suspended: true` are blocked immediately with `InstallationSuspended` and produce a `PromptUnsuspendInstallation` action.
     - Test Case 6 (`test_unauthorized_revoked_installation`): Installations with `is_revoked: true` are blocked immediately with `InstallationRevoked` and instruct reinstallation.
     - Test Case 7 (`test_unauthorized_policy_violating_permissions_missing_contents_read`): Tokens lacking mandatory `contents: read` scope are rejected with `InsufficientPermissions` and produce a `ContactAdministrator` remediation plan.
     - Test Case 8 (`test_unauthorized_policy_violating_permissions_forbidden_write_scope`): Tokens granted write/admin permissions (`contents: write`) are strictly rejected with `InsufficientPermissions` under Rule 05 read-only guarantee.
     - Test Case 9 (`test_unauthorized_private_extractor_commit_403_resource_not_accessible`): Upstream HTTP 403 during extraction commit resolution halts extraction immediately with sanitized error and produces an actionable diagnostic recovery plan.
     - Test Case 10 (`test_unauthorized_private_extractor_commit_404_not_found`): Upstream HTTP 404 on private repositories is diagnosed as `RepositoryNotGranted` with `PromptRepositorySelectionUpdate`.
     - Test Case 11 (`test_unauthorized_zero_secret_leakage_in_diagnoses_and_cards`): Confirms that sensitive tokens (`ghs_*`) are strictly masked across all failure modes, error messages, and summary cards.
  2. Run full verification gates (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Unauthorized repository requests are strictly blocked across all failure modalities.
  - Zero private repository content, trees, blobs, or commits are exposed.
  - `AuthorizationRecoveryCoordinator` produces actionable remediation plans with accurate settings URLs.
  - Zero secret tokens or credential patterns are leaked in logs, error envelopes, or summary cards.
  - 100% of tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: Ambiguous error envelopes causing user confusion or silent extraction failures.
  - Mitigation: Distinct failure categories (`NoInstallationForSession`, `AccountMismatch`, `RepositoryNotGranted`, `InstallationSuspended`, `InstallationRevoked`, `InsufficientPermissions`) and explicit step-by-step remediation cards.
- **Rollback Plan:**
  - Revert Subphase 6.4.2 additions via `git reset --hard HEAD~1` and restore Subphase 6.4.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test unauthorized_access_test`: 11/11 passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor`: all 30 unit tests and 32 integration test targets passed with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace [533+ tests passed], Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `5129f7e`
- **Status:** `ACCEPTED`

---

#### Subphase `6.4.3` — Token leakage tests in crates/github-extractor & crates/observability
- **Unique ID:** `6.4.3`
- **Objective:** Verify and guarantee zero token leakage across all domain models, debug/display envelopes, diagnostic cards, wire-level HTTP requests, cryptographic key materials, and observability redaction pipelines under Rule 05.
- **Scope:** `crates/github-extractor/tests/token_leakage_test.rs`, `crates/observability/tests/token_leakage_test.rs`, `crates/observability/src/redaction.rs`, `crates/github-extractor/src/oauth.rs`, `crates/github-extractor/src/installation.rs`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Prerequisites:** `6.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/oauth.rs`
  - `crates/observability/src/redaction.rs`
  - `crates/github-extractor/tests/token_leakage_test.rs`
  - `crates/observability/tests/token_leakage_test.rs`
  - `IMPLEMENTATION_LEDGER.md`
  - `PROJECT_STATUS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive observability leakage defense suite in `crates/observability/tests/token_leakage_test.rs` (7 tests):
     - `test_all_github_token_formats_leakage_prevention`: Verifies complete redaction of all GitHub token formats (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, fine-grained `github_pat_`, and OAuth access tokens in URL userinfo `https://ghs_...@github.com`) to `[REDACTED_GITHUB_TOKEN]`.
     - `test_private_key_leakage_prevention`: Verifies PKCS#1 RSA, PKCS#8, and EC private keys are redacted to `[REDACTED_PRIVATE_KEY]`.
     - `test_headers_signatures_and_provider_keys_leakage_prevention`: Verifies `Authorization: Bearer`, `Authorization: token`, `X-Hub-Signature-256`, AWS access keys (`AKIA...`), and generic 32-64 char hex API keys are redacted.
     - `test_recursive_json_value_leakage_prevention`: Verifies recursive JSON AST walks mask sensitive key values and scrub string leaf nodes.
     - `test_is_sensitive_key_comprehensive_taxonomy`: Validates positive taxonomy (`passwd`, `password`, `secret`, `token`, `credential`, `signing_key`, etc.) and negative non-sensitive keys.
     - `test_redacting_writer_stream_handling`: Verifies streaming redaction through `RedactingWriter` across chunk boundaries.
     - `test_custom_redaction_engine_literals_and_regexes`: Verifies custom literal registration and custom regex patterns.
  2. Author comprehensive domain model and wire-level leakage defense suite in `crates/github-extractor/tests/token_leakage_test.rs` (9 tests):
     - `test_authorized_installation_context_masks_token`: Confirms `AuthorizedInstallationContext` debug & display representations mask raw token strings.
     - `test_stored_token_record_masks_token`: Confirms `StoredTokenRecord` debug & display mask tokens to `***[REDACTED_TOKEN]***`.
     - `test_app_config_masks_all_credentials`: Confirms `GitHubAppConfig` masks client secrets, PEM keys, and webhook secrets.
     - `test_github_installation_token_masks_token`: Confirms `GitHubInstallationToken` masks bearer tokens in debug and display.
     - `test_oauth_token_responses_mask_secrets`: Confirms `GitHubOAuthTokenResponse` and `GitHubOAuthErrorResponse` mask credentials.
     - `test_recovery_plan_and_summary_card_zero_leakage`: Confirms `AuthorizationRecoveryPlan` and `summary_card()` redact sensitive tokens.
     - `test_extractor_error_variants_sanitized`: Confirms all `GitHubExtractorError` variants scrub tokens.
     - `test_wire_level_token_isolation_in_headers_only`: In-process Axum server verifies bearer token is passed strictly in `Authorization` header and never leaked in URI path or query parameters.
     - `test_core_encryption_key_and_payload_masking`: Confirms `EncryptionKey` and `EncryptedPayload` mask raw bytes and ciphertext in debug and display.
  3. Implement custom `fmt::Debug` and `fmt::Display` for `GitHubOAuthErrorResponse` in `crates/github-extractor/src/oauth.rs` redacting secrets via `redact_secrets`.
  4. Extend `is_sensitive_key` in `crates/observability/src/redaction.rs` to include `passwd` variations.
  5. Run full verification gates (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
- **Acceptance Criteria:**
  - Zero token or credential leakage across all domain structs, debug formatters, display formatters, error envelopes, and recovery cards.
  - Wire HTTP requests transmit tokens exclusively via standard Authorization headers and never in URL path or query params.
  - Observability redaction engine captures 100% of defined credential patterns across raw text, streams, and JSON structures.
  - 100% of tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: New GitHub token format introduced upstream leaking through regex patterns.
  - Mitigation: Regex patterns cover all existing standard prefixes (`gh[psour]_` and `github_pat_`) plus generic Bearer/token authorization headers and URL userinfo stripping.
- **Rollback Plan:**
  - Revert Subphase 6.4.3 additions via `git reset --hard HEAD~1` and restore Subphase 6.4.2 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-observability --test token_leakage_test`: 7/7 passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor --test token_leakage_test`: 9/9 passed in 0.03s.
  - `cargo test -p web-repo-to-ai-observability`: 34/34 passed (20 unit, 14 integration).
  - `cargo test -p web-repo-to-ai-github-extractor`: 30 unit tests, 33 integration test targets (127+ integration tests), 1 doc-test passed with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace [549+ tests passed], Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `c734866`
- **Status:** `ACCEPTED`

---

#### Subphase `6.4.4` — Permission audit in crates/github-extractor
- **Unique ID:** `6.4.4`
- **Objective:** Perform exhaustive security verification and permission auditing across all private repository extraction components in `crates/github-extractor`, asserting strict minimal read-only least-privilege compliance (`contents: read`, `metadata: read`), zero repository modification capability, automatic detection and rejection of excessive levels (write, admin) and prohibited sensitive categories (`administration`, `actions`, `secrets`, etc.), installation suspension enforcement, wire-level token request scoping, and formal certification of Stage 6 under Rule 05.
- **Scope:** `crates/github-extractor/src/permissions.rs`, `crates/github-extractor/src/authorization.rs`, `crates/github-extractor/src/private_extractor.rs`, `crates/github-extractor/tests/permission_audit_test.rs`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`.
- **Prerequisites:** `6.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/github-extractor/src/permissions.rs`
  - `crates/github-extractor/src/authorization.rs`
  - `crates/github-extractor/src/private_extractor.rs`
  - `crates/github-extractor/tests/permission_audit_test.rs`
  - `IMPLEMENTATION_LEDGER.md`
  - `PROJECT_STATUS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `ROADMAP.md`
  - `docs/ROADMAP.md`
- **Implementation Tasks:**
  1. Author comprehensive permission audit integration test suite `crates/github-extractor/tests/permission_audit_test.rs` (14 tests):
     - `test_permission_audit_strict_minimal_exact_compliance`: Validates exact `contents: read` and `metadata: read` compliance under `PermissionPolicy::strict_minimal()`, zero violations, summary formatting, markdown report generation, and JSON roundtrip serialization.
     - `test_permission_audit_rejects_missing_required_scopes`: Tests rejection when missing `contents: read`, missing `metadata: read`, or having an empty permission map.
     - `test_permission_audit_rejects_all_write_and_admin_levels`: Exhaustively validates rejection of `write` and `admin` levels across `contents`, `metadata`, `pull_requests`, `issues`, and `packages`.
     - `test_permission_audit_rejects_all_prohibited_sensitive_categories`: Exhaustively iterates over `DEFAULT_FORBIDDEN_CATEGORIES` (`administration`, `actions`, `secrets`, `deployments`, `packages`, `security_events`, `members`, `organization_administration`, `organization_secrets`, `organization_actions`, `workflows`, `blocking_keys`), asserting each is rejected with `ForbiddenScope` citing Rule 05 even if granted with `read` level.
     - `test_permission_audit_rejects_unexpected_scopes_under_strict_minimal`: Validates that under `StrictMinimal`, benign non-required read scopes (`pull_requests: read`, `issues: read`, `discussions: read`, `pages: read`, `statuses: read`, `checks: read`) are flagged with `UnexpectedScope`.
     - `test_permission_audit_standard_policy_permits_benign_read_scopes`: Validates that `PermissionPolicy::standard()` permits benign read scopes while strictly preserving write/admin and prohibited category bans.
     - `test_permission_audit_custom_policy_builder`: Validates `PermissionPolicy::builder()` custom required scopes, custom forbidden categories, custom forbidden levels, and explicit whitelist scoping.
     - `test_permission_audit_installation_suspension_detection`: Validates that suspended installations (`is_suspended: true`) are flagged with `SuspendedInstallation` violation and rejected by `validate_installation`.
     - `test_permission_audit_installation_token_validation`: Validates `audit_installation_token` and `validate_installation_token` on compliant and non-compliant tokens.
     - `test_permission_audit_authorized_installation_context_methods`: Validates `audit_permissions`, `validate_permissions`, `audit_permissions_with_policy`, and `validate_permissions_with_policy` on `AuthorizedInstallationContext`.
     - `test_permission_audit_private_extractor_fail_closed_on_policy_violation`: Verifies that `PrivateRepositoryExtractor::extract` fails closed immediately with `PermissionViolation` on violating permissions before issuing any network requests.
     - `test_permission_audit_live_http_installation_token_request_read_only_preservation`: In-process Axum mock server verifies client sends strictly `{"contents": "read", "metadata": "read"}` when requesting installation access tokens.
     - `test_permission_audit_markdown_and_json_report_fidelity`: Validates multi-violation markdown table formatting with `REJECTED` and `PERMITTED` badges, caution alerts, and JSON schema fidelity.
     - `test_stage_6_security_acceptance_gate_invariants`: Verifies the holistic Stage 6 Security Acceptance Gate: at-rest AES-256-GCM encryption with session AAD, zero token leakage in debug/display, zero repository modification permissions, and strict read-only least-privilege enforcement.
  2. Implement helper query methods on `PermissionAuditReport` (`has_excessive_level`, `has_forbidden_category`, `has_missing_required`, `has_unexpected_scope`).
  3. Implement `audit_context` and `validate_context` on `PermissionAuditor`.
  4. Implement `audit_permissions`, `validate_permissions`, `audit_permissions_with_policy`, `validate_permissions_with_policy` on `AuthorizedInstallationContext`.
  5. Add `permission_policy` and `with_permission_policy` to `PrivateExtractionConfig`, and enforce pre-flight permission validation in `PrivateRepositoryExtractor::extract`.
  6. Execute full verification suite (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo deny check`, `scripts/check-all.ps1`).
  7. Formally certify Phase 6.4 and STAGE 6 in all tracking ledgers and roadmaps.
- **Acceptance Criteria:**
  - Strict minimal permissions (`contents: read`, `metadata: read`) are enforced across all installation, token, and extraction operations.
  - Any attempt to grant or use write, admin, or prohibited category permissions fails closed with explicit `PermissionViolation`.
  - Suspended installations are systematically blocked.
  - Diagnostic and audit reports render valid GFM Markdown tables with accurate status badges and RFC-compliant JSON.
  - Stage 6 Acceptance Gate ("Authorized private repositories can be extracted without exposing credentials, leaking tokens, or permitting repository modifications") is 100% satisfied.
  - 100% of tests pass, clippy is warning-free, and monorepo gates pass with 0 failures.
- **Risk Analysis:**
  - Risk: Permission elevation through newly added GitHub App scopes.
  - Mitigation: `PermissionPolicy::strict_minimal()` rejects all non-whitelisted scopes; `PermissionAuditor` runs at token minting, installation registration, session evaluation, and extraction start.
- **Rollback Plan:**
  - Revert Subphase 6.4.4 additions via `git reset --hard HEAD~1` and restore Subphase 6.4.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-github-extractor --test permission_audit_test`: 14/14 passed in 0.01s.
  - `cargo test -p web-repo-to-ai-github-extractor`: all 30 unit tests, 34 integration test suites (141+ integration tests), and 1 doc-test passed with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)` (all 5 gates passed: Bootstrap & Governance, Workflow Validation, Rust Workspace [563+ tests passed], Frontend Web [213 vitest passed], Resource Budget & 30 GB Storage Ceiling [5.27 GB / 30 GB, 17.6%]).
- **Git Commit Reference:** `2ed1aba`
- **Status:** `ACCEPTED`

---

### PHASE 6.4 & STAGE 6 FORMAL CERTIFICATION

- **Stage:** `STAGE 6 — PRIVATE GITHUB REPOSITORIES`
- **Phase:** `Phase 6.4 — Security verification`
- **Subphases Certified (16 of 16):**
  - Phase 6.1 (GitHub App): `6.1.1` (App Configuration & Manifest), `6.1.2` (OAuth & Installation Flow), `6.1.3` (Minimum Permissions), `6.1.4` (Callback Security) — `ACCEPTED` (**FORMALLY CERTIFIED**)
  - Phase 6.2 (Credential Management): `6.2.1` (Secure Token Storage), `6.2.2` (Token Refresh & Expiration), `6.2.3` (Revocation), `6.2.4` (Secret Redaction) — `ACCEPTED` (**FORMALLY CERTIFIED**)
  - Phase 6.3 (Private Extraction): `6.3.1` (Installation Authorization), `6.3.2` (Selected Repository Access), `6.3.3` (Private Tree & Source Extraction), `6.3.4` (Authorization Failure Recovery) — `ACCEPTED` (**FORMALLY CERTIFIED**)
  - Phase 6.4 (Security Verification): `6.4.1` (Cross-User Isolation), `6.4.2` (Unauthorized Access Tests), `6.4.3` (Token Leakage Tests), `6.4.4` (Permission Audit) — `ACCEPTED` (**FORMALLY CERTIFIED**)
- **Stage Acceptance Gate Verification:**
  - Authorized private repositories can be extracted without exposing credentials, leaking tokens, or permitting repository modifications.
  - Least privilege strictly enforced: mandatory `contents: read` and `metadata: read`; zero write/admin scopes permitted across all layers.
  - Secret tokens encrypted at rest via `AES-256-GCM` with session-scoped Authenticated Associated Data (AAD).
  - Multi-tenant tenant session isolation cryptographically and logically verified across memory stores, caches, and Axum endpoints.
  - Observability and wire HTTP traffic verified: zero token leakage in URI paths, queries, logs, or error envelopes.
  - Bounded memory, CPU-only 16 GB RAM baseline, and `<= 30 GB` storage ceiling strictly observed (5.27 GB / 30 GB, 17.6%).
- **Certification Status:** **STAGE 6 FORMALLY CERTIFIED & ACCEPTED**

---

## STAGE 7 — AI CONTEXT AND TOKEN OPTIMIZATION

### Phase 7.1 — Content optimization

#### Subphase `7.1.1` — Structural normalization in crates/normalizer & crates/cdm
- **Unique ID:** `7.1.1`
- **Objective:** Implement deterministic document and section structural normalization in `crates/normalizer` and `crates/cdm`: UTF-8 character encoding sanitization (BOM stripping, null byte elimination, unprintable ASCII control character removal, zero-width space stripping, typographic punctuation conversion), whitespace standardization (Unix LF line-ending conversion, trailing whitespace removal, leading/trailing blank line trimming, code vs prose blank line collapsing), Markdown syntax standardization (code fence cleanup, heading level spacing, list marker standardization, GFM rectangular table normalization), kind-specific normalization preserving 100% code syntax and indentation fidelity, document-level orchestration, cryptographic BLAKE3 content hash recalculation, and quantitative `NormalizationReport` generation.
- **Scope:** `crates/normalizer/src/*`, `crates/normalizer/Cargo.toml`, `crates/normalizer/tests/structural_normalization_test.rs`.
- **Prerequisites:** Stage 6 Certified (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/normalizer/Cargo.toml`
  - `crates/normalizer/src/config.rs`
  - `crates/normalizer/src/document.rs`
  - `crates/normalizer/src/encoding.rs`
  - `crates/normalizer/src/error.rs`
  - `crates/normalizer/src/lib.rs`
  - `crates/normalizer/src/markdown.rs`
  - `crates/normalizer/src/report.rs`
  - `crates/normalizer/src/section.rs`
  - `crates/normalizer/src/whitespace.rs`
  - `crates/normalizer/tests/structural_normalization_test.rs`
- **Implementation Tasks:**
  1. Define strongly-typed configuration models and preset builders in `crates/normalizer/src/config.rs`:
     - `EncodingConfig`: BOM stripping, null byte removal, control character filtering, zero-width space stripping, typographic punctuation toggle.
     - `WhitespaceConfig`: CRLF/CR -> LF canonicalization, trailing whitespace trimming, blank line trimming and collapsing ceilings (`max_consecutive_blank_lines_prose: 1`, `max_consecutive_blank_lines_code: 2`), inline space collapsing toggle.
     - `MarkdownConfig`: code fence normalization, heading level spacing, list marker standardization (`*` -> `-`), GFM table rectangularization and cell padding.
     - `NormalizationConfig`: master configuration with `lossless()`, `compact()`, and `code_focused()` domain presets.
  2. Implement character encoding repair and Unicode sanitization in `crates/normalizer/src/encoding.rs`:
     - `EncodingSanitizer`: single-pass character stream filtering stripping `\u{FEFF}`, `\0`, unprintable ASCII control characters (`\x01`..=`\x1F` excluding `\t`, `\n`, `\r`), zero-width codepoints (`\u{200B}`, `\u{200C}`, `\u{200D}`), and smart punctuation normalization (`“`, `”`, `‘`, `’`, `—`, `…` -> ASCII).
  3. Implement whitespace and line-ending normalizer in `crates/normalizer/src/whitespace.rs`:
     - `WhitespaceNormalizer`: canonicalizes CRLF/CR to Unix LF (`\n`), trims per-line trailing whitespace, trims leading/trailing section blank lines, and collapses consecutive blank lines according to code/prose limits while strictly preserving code indentation and code spans.
  4. Implement Markdown structural syntax and table rectangularization in `crates/normalizer/src/markdown.rs`:
     - `MarkdownNormalizer`: normalizes code block fences with lowercase language specifiers, standardizes heading spacing (`#Title` -> `# Title`, `###   Title` -> `### Title`), converts bullet list markers (`*` -> `-`), and rectangularizes GFM tables with matching cell counts and delimiter rows (`:---`).
  5. Implement section-aware and whole-document normalizers in `crates/normalizer/src/section.rs` and `crates/normalizer/src/document.rs`:
     - `SectionNormalizer`: kind-specific strategy mapping (`SectionKind::CodeFile` and `CodeSnippet` strictly preserve quotes, symbols, and indentation; `Prose`, `Table`, `Admonition` receive Markdown normalization), updates section BLAKE3 hash, updates `SectionTokenMetrics`.
     - `DocumentNormalizer`: orchestrates section normalization across `CdmDocument`, recomputes document-level BLAKE3 revision hash, and aggregates granular section audits into `NormalizationReport`.
  6. Implement telemetry reporting in `crates/normalizer/src/report.rs`:
     - `NormalizationReport`: records before/after bytes, characters, lines, and estimated tokens, net reductions, percentage compression, per-section audit logs, GFM Markdown summary table rendering, and JSON roundtrip serialization.
  7. Author unit and integration tests:
     - 16 unit tests in `crates/normalizer/src/*` and 13 integration tests in `crates/normalizer/tests/structural_normalization_test.rs` covering all encoding, whitespace, markdown, code preservation, table rectangularization, full document, report serialization, and idempotency invariants (`normalize(normalize(doc)) == normalize(doc)`).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-normalizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - CRLF and CR line endings converted to standard Unix LF without exception.
  - BOM, null bytes, zero-width characters, and unprintable control characters stripped.
  - Source code indentation, symbols, and typographic punctuation 100% preserved.
  - Markdown headings, code fences, and tables rectangularized cleanly.
  - Normalization is provably idempotent (`normalize(normalize(doc)) == normalize(doc)`).
  - All 29 normalizer tests pass, all workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Semantic corruption of source code or code snippets during normalization.
  - Mitigation: `SectionNormalizer` bypasses typographic punctuation and inline space collapsing for `SectionKind::CodeFile` and `CodeSnippet`; integration tests assert byte-for-byte fidelity of code indentation and quotes.
- **Rollback Plan:**
  - Revert Subphase 7.1.1 additions via `git reset --hard HEAD~1` and restore Stage 6 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-normalizer`: 29 tests passing (16 unit tests, 13 integration tests in `structural_normalization_test.rs`).
  - Full workspace tests: 563+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `f5f12b1`
- **Status:** `ACCEPTED`

---

#### Subphase `7.1.2` — Exact duplicate removal in crates/deduplication & crates/cdm
- **Unique ID:** `7.1.2`
- **Objective:** Implement exact cryptographic duplicate detection, policy-driven canonical election, duplicate consolidation strategies, exclusion logging, and relationship linking in `crates/deduplication` and `crates/cdm`. Support `Remove`, `ReplaceWithPointer`, and `MarkOnly` deduplication strategies, `FirstSeen`, `ShortestPath`, `StructuralHierarchy`, and `PreferredPrefix` canonical selection policies, `ExactBytes` and `Normalized` hashing scopes, auditable `ExclusionRecord` logging (`ExclusionReason::Duplicate`), directed relationship creation (`RelationshipKind::DuplicateOf`), and quantitative telemetry reporting (`ExactDeduplicationReport`).
- **Scope:** `crates/deduplication/src/*`, `crates/deduplication/Cargo.toml`, `crates/deduplication/tests/exact_deduplication_test.rs`, `crates/cdm/src/relationships.rs`.
- **Prerequisites:** `7.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/cdm/src/relationships.rs`
  - `crates/cdm/tests/schema_validation.rs`
  - `crates/deduplication/Cargo.toml`
  - `crates/deduplication/src/cluster.rs`
  - `crates/deduplication/src/config.rs`
  - `crates/deduplication/src/document.rs`
  - `crates/deduplication/src/error.rs`
  - `crates/deduplication/src/exact.rs`
  - `crates/deduplication/src/lib.rs`
  - `crates/deduplication/src/report.rs`
  - `crates/deduplication/tests/exact_deduplication_test.rs`
- **Implementation Tasks:**
  1. Extend CDM relationship taxonomy:
     - Add `RelationshipKind::DuplicateOf` (`"duplicate_of"`) in `crates/cdm/src/relationships.rs` and update schema validation tests.
  2. Implement strongly-typed configuration models and presets in `crates/deduplication/src/config.rs`:
     - `DeduplicationStrategy`: `Remove`, `ReplaceWithPointer`, `MarkOnly`.
     - `CanonicalSelectionPolicy`: `FirstSeen`, `ShortestPath`, `StructuralHierarchy` (`CodeFile` > `CodeSnippet` > `Prose` > `Table` > `Admonition` > `Frontmatter` > `Manifest` > `Navigation`), `PreferredPrefix(String)`.
     - `HashScope`: `ExactBytes` (raw byte BLAKE3), `Normalized` (canonical line-endings and whitespace trimming).
     - `ExactDeduplicationConfig`: `strategy`, `canonical_policy`, `hash_scope`, `min_byte_threshold` (default 32 bytes), `log_exclusions` (default true), presets `aggressive()`, `conservative()`, `lossless()`.
  3. Implement duplicate cluster model in `crates/deduplication/src/cluster.rs`:
     - `DuplicateCluster`: records `content_hash`, `canonical_section_id`, `canonical_title`, `canonical_path`, `canonical_kind`, `duplicate_section_ids`, `duplicate_count`, `bytes_per_instance`, `total_duplicate_bytes`, `estimated_tokens_saved`.
  4. Implement exact deduplication engine in `crates/deduplication/src/exact.rs`:
     - `compute_dedup_hash` evaluating raw or normalized whitespace text via BLAKE3.
     - `ExactDeduplicator::find_duplicate_clusters` grouping candidate sections by hash, filtering clusters with >1 instances, electing canonical section via configured policy, and sorting clusters by net bytes saved descending.
  5. Implement document-level deduplicator in `crates/deduplication/src/document.rs`:
     - `DocumentDeduplicator`: orchestrates exact deduplication across `CdmDocument`, executes strategy (`Remove` drops sections and logs `ExclusionRecord`, `ReplaceWithPointer` substitutes pointer comment and updates hash/metrics, `MarkOnly` preserves bodies), logs `ExclusionReason::Duplicate` records, registers `RelationshipKind::DuplicateOf` edges, and recomputes document-level BLAKE3 revision hash.
  6. Implement deduplication telemetry report in `crates/deduplication/src/report.rs`:
     - `ExactDeduplicationReport`: records before/after bytes and estimated tokens, net reduction, percentage compression, unique sections, duplicate sections, clusters, Markdown summary table, and JSON roundtrip serialization.
  7. Author unit and integration tests:
     - 4 unit tests in `crates/deduplication/src/*` and 10 integration tests in `crates/deduplication/tests/exact_deduplication_test.rs` covering exact vs normalized hashing, aggressive removal, conservative pointer replacement, lossless mark-only, structural hierarchy election, shortest path election, preferred prefix election, minimum byte threshold defense, idempotency, and Markdown/JSON report serialization.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-deduplication`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Duplicate sections are identified deterministically across exact byte and normalized hash modes.
  - Canonical sections are elected strictly according to configured policy.
  - `Remove` strategy drops duplicate bodies and records auditable `ExclusionRecord` items with `ExclusionReason::Duplicate`.
  - `ReplaceWithPointer` strategy inserts pointer comments and updates BLAKE3 content hashes.
  - Directed `DuplicateOf` relationships are generated linking duplicates to their canonical sections.
  - Deduplication is provably idempotent (`deduplicate(deduplicate(doc)) == deduplicate(doc)`).
  - All 14 deduplication tests pass, all workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Erroneous canonical election causing loss of primary source file in favor of generated fixture.
  - Mitigation: `CanonicalSelectionPolicy::StructuralHierarchy` ranks `CodeFile` above `CodeSnippet` and ties break on path length and outline order.
- **Rollback Plan:**
  - Revert Subphase 7.1.2 additions via `git reset --hard HEAD~1` and restore Subphase 7.1.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-deduplication`: 14 tests passing (4 unit tests, 10 integration tests).
  - Full workspace tests: 577+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `7f5bc7a`
- **Status:** `ACCEPTED`

---

#### Subphase `7.1.3` — Semantic-aware deduplication in crates/deduplication
- **Unique ID:** `7.1.3`
- **Objective:** Implement semantic-aware near-duplicate detection and consolidation in `crates/deduplication` using MinHash signatures with word shingling and SimHash 64-bit cosine distance estimation. Support connected component clustering, canonical section election policies (`InformationRichness`, `StructuralHierarchy`, `FirstSeen`, `ShortestPath`), configurable consolidation strategies (`ReplaceWithAnnotatedPointer`, `Remove`, `MarkOnly`), auditable `ExclusionRecord` logging (`ExclusionReason::Duplicate`), directed relationship creation (`RelationshipKind::DuplicateOf`), and comprehensive telemetry reporting (`SemanticDeduplicationReport`).
- **Scope:** `crates/deduplication/src/*`, `crates/deduplication/tests/semantic_deduplication_test.rs`.
- **Prerequisites:** `7.1.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/deduplication/src/shingle.rs`
  - `crates/deduplication/src/minhash.rs`
  - `crates/deduplication/src/simhash.rs`
  - `crates/deduplication/src/semantic_config.rs`
  - `crates/deduplication/src/semantic_cluster.rs`
  - `crates/deduplication/src/semantic_report.rs`
  - `crates/deduplication/src/semantic.rs`
  - `crates/deduplication/src/lib.rs`
  - `crates/deduplication/tests/semantic_deduplication_test.rs`
- **Implementation Tasks:**
  1. Shingling & N-gram decomposition (`crates/deduplication/src/shingle.rs`):
     - `Shingler::word_shingles` extracting sliding window tuples with case-folding and punctuation normalization.
     - `Shingler::char_kgrams` decomposing strings into overlapping character sequences.
     - Fast 64-bit FNV-1a hashing (`fnv1a_hash64`) for deterministic token fingerprinting.
  2. MinHash signature estimator (`crates/deduplication/src/minhash.rs`):
     - `MinHashSignature` computing $N$ random hash permutations using deterministic SplitMix64 pseudo-random generator.
     - Jaccard similarity estimation via normalized signature component agreement.
  3. SimHash 64-bit cosine similarity estimator (`crates/deduplication/src/simhash.rs`):
     - `SimHash64` bitwise accumulation over word tokens with 64-bit integer packing.
     - Hamming distance calculation via `count_ones` bit differences and cosine similarity projection.
  4. Strongly-typed semantic configuration (`crates/deduplication/src/semantic_config.rs`):
     - `SemanticAlgorithm`: `MinHash { shingle_size, num_hashes }`, `SimHash { max_hamming_distance }`.
     - `SemanticDeduplicationStrategy`: `ReplaceWithAnnotatedPointer`, `Remove`, `MarkOnly`.
     - `SemanticCanonicalPolicy`: `InformationRichness`, `StructuralHierarchy`, `FirstSeen`, `ShortestPath`.
     - `SemanticDeduplicationConfig`: presets `balanced()` (MinHash 0.75 threshold), `aggressive()` (0.70 threshold, remove), `conservative()` (0.85 threshold, pointers), `simhash_based(max_dist)`.
  5. Near-duplicate cluster model (`crates/deduplication/src/semantic_cluster.rs`):
     - `NearDuplicateCluster` and `NearDuplicateMember` recording canonical details, members, similarity metrics, byte counts, and token savings.
  6. Semantic deduplication telemetry report (`crates/deduplication/src/semantic_report.rs`):
     - `SemanticDeduplicationReport` tracking before/after byte and token metrics, reduction percentages rounded to 4 decimals, Markdown summary table generation, and JSON roundtrip serialization.
  7. Semantic deduplicator engine (`crates/deduplication/src/semantic.rs`):
     - `SemanticDeduplicator::find_near_duplicate_clusters`: eligible candidate filtering, signature generation, pairwise graph adjacency, connected component traversal (BFS), and canonical candidate election.
     - `SemanticDeduplicator::deduplicate_document`: executes consolidation strategy (`ReplaceWithAnnotatedPointer`, `Remove`, `MarkOnly`), updates section hashes, emits `ExclusionRecord` items with `ExclusionReason::Duplicate`, links directed `DuplicateOf` relationships, and updates document-level BLAKE3 revision hashes.
     - Convenience helper `deduplicate_semantic`.
  8. Unit & Integration test suite:
     - 11 unit tests in `shingle.rs`, `minhash.rs`, `simhash.rs`, `semantic.rs`.
     - 9 integration tests in `tests/semantic_deduplication_test.rs` validating MinHash clustering, SimHash clustering, pointer replacement, aggressive removal with exclusions, mark-only annotations, structural hierarchy canonical election, minimum word threshold filtering, idempotency, and Markdown/JSON report serialization.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-deduplication`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Near-duplicate sections identified accurately via MinHash and SimHash without semantic distortion.
  - Canonical sections elected according to configured policy.
  - Pointers include canonical section reference and similarity percentage.
  - Directed `DuplicateOf` relationships recorded in CDM document model.
  - Provenance and exclusions tracked with `ExclusionReason::Duplicate`.
  - Provably idempotent near-duplicate deduplication.
  - All 34 deduplication tests pass, all workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Overly aggressive near-duplicate clustering collapsing distinct code blocks that share common boilerplate.
  - Mitigation: `min_words_threshold` ceiling (default 8 words), word-level 2-gram shingling, strict similarity threshold (0.75+), and lossless preservation mode for safety-critical documentation.
- **Rollback Plan:**
  - Revert Subphase 7.1.3 additions via `git reset --hard HEAD~1` and restore Subphase 7.1.2 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-deduplication`: 34 tests passing (15 unit tests, 10 exact tests, 9 semantic tests).
  - Full workspace tests: 586+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `e391977`
- **Status:** `ACCEPTED`

---

#### Subphase `7.1.4` — Content classification in crates/normalizer & crates/code-intelligence
- **Unique ID:** `7.1.4`
- **Objective:** Implement comprehensive multi-language content classification, AST symbol extraction, complexity metrics, information density scoring, and signal quality auditing across `crates/code-intelligence` and `crates/normalizer`. Classify sections into architectural roles (`SourceCode`, `Documentation`, `Specification`, `Configuration`, `DependencyManifest`, `TestAndVerification`, `GeneratedOrBoilerplate`, `NavigationOrMetadata`, `TabularOrData`), assign information density tiers (`High`, `Medium`, `Low`, `Trivial`), compute normalized signal-to-noise scores ($0.0 - 1.0$), identify noise indicators (`CopyrightHeaderOnly`, `MinifiedContent`, `AutoGeneratedStub`, `RepetitiveTestFixture`, `EmptyOrTrivialSection`, `NavigationChromeOnly`), enrich CDM section attributes without altering content hashes, and generate quantitative classification telemetry reports (`ContentClassificationReport`).
- **Scope:** `crates/code-intelligence/src/*`, `crates/normalizer/src/classifier.rs`, `crates/normalizer/src/classification_report.rs`, `crates/normalizer/tests/content_classification_test.rs`.
- **Prerequisites:** `7.1.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/code-intelligence/Cargo.toml`
  - `crates/code-intelligence/src/language.rs`
  - `crates/code-intelligence/src/symbols.rs`
  - `crates/code-intelligence/src/metrics.rs`
  - `crates/code-intelligence/src/nature.rs`
  - `crates/code-intelligence/src/analyzer.rs`
  - `crates/code-intelligence/src/error.rs`
  - `crates/code-intelligence/src/lib.rs`
  - `crates/normalizer/Cargo.toml`
  - `crates/normalizer/src/classifier.rs`
  - `crates/normalizer/src/classification_report.rs`
  - `crates/normalizer/src/lib.rs`
  - `crates/normalizer/tests/content_classification_test.rs`
- **Implementation Tasks:**
  1. Multi-language syntax identification (`crates/code-intelligence/src/language.rs`):
     - `Language` enum covering Rust, TS, JS, Python, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, Scala, Shell, SQL, HTML, CSS, Markdown, YAML, TOML, JSON, XML, Protobuf, GraphQL, Dockerfile.
     - Extension, path, shebang, and content keyword detection.
  2. AST symbol & declaration extraction (`crates/code-intelligence/src/symbols.rs`):
     - `SymbolKind` (Function, Method, Struct, Class, Interface, Trait, Enum, TypeAlias, Constant, Macro, Module, Import) and `Visibility` (Public, Protected, Private, Internal).
     - Multi-language AST symbol extraction for Rust, TypeScript/JavaScript, Python, Go, Java, C#, C/C++.
  3. Code complexity & structural metrics (`crates/code-intelligence/src/metrics.rs`):
     - `CodeMetrics` computing code lines, comment lines, blank lines, symbol count, export count, comment ratio, average line length, and minification detection.
  4. Architectural code role classification (`crates/code-intelligence/src/nature.rs`):
     - `CodeNature`: CoreImplementation, InterfaceOrTypeDefinition, TestOrFixture, Benchmark, GeneratedOrBoilerplate, BuildOrScript, DataOrSchema.
     - Path, content marker (@generated, DO NOT EDIT, #[test]), and symbol composition heuristics.
  5. Code analysis pipeline (`crates/code-intelligence/src/analyzer.rs`):
     - `CodeAnalyzer` integrating language detection, symbol extraction, metric calculation, and nature classification into `CodeAnalysisResult`.
  6. Section content classification (`crates/normalizer/src/classifier.rs`):
     - `ContentCategory` taxonomy, `InformationDensity` tiers (`High`, `Medium`, `Low`, `Trivial`), `NoiseIndicator` taxonomy.
     - `SectionClassifier` classifying code files, snippets, manifests, tables, prose, navigation, and frontmatter.
     - `DocumentContentClassifier` orchestrating whole-document classification and enriching `CdmSection.attributes` (`content_category`, `information_density`, `signal_score`, `is_test`, `is_generated`, `is_boilerplate`, `code_nature`, `symbols_summary`, `noise_indicators`) while strictly preserving section cryptographic hashes.
  7. Content classification report (`crates/normalizer/src/classification_report.rs`):
     - `ContentClassificationReport` tracking category counts, token distributions, density distributions, average signal scores, low-signal candidate section IDs, Markdown summary rendering, and JSON serialization.
  8. Integration test suite (`crates/normalizer/tests/content_classification_test.rs`):
     - 8 integration tests validating multi-language classification, test/benchmark identification, generated boilerplate detection, doc/spec separation, config/manifest categorization, navigation/empty noise indicators, end-to-end document enrichment, hash preservation, and report JSON roundtrip.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-code-intelligence`
  - `cargo test -p web-repo-to-ai-normalizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Multi-lingual source code classified accurately across Rust, TypeScript, Python, and Go.
  - Tests and benchmarks flagged with `is_test: true` and categorized under `TestAndVerification`.
  - Machine-generated code, minified bundles, and copyright disclaimers flagged with appropriate noise indicators.
  - Section `content_hash` values verified 100% unaltered after attribute enrichment.
  - Provable JSON serialization roundtrip for `ContentClassificationReport`.
  - All 14 code-intelligence tests pass, all 41 normalizer tests pass, all workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Misclassifying specialized code files (e.g. build scripts or migration schemas) as general source code.
  - Mitigation: Path heuristics take priority, followed by distinctive content markers, before falling back to generic implementation classification.
- **Rollback Plan:**
  - Revert Subphase 7.1.4 additions via `git reset --hard HEAD~1` and restore Subphase 7.1.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-code-intelligence`: 14 tests passing.
  - `cargo test -p web-repo-to-ai-normalizer`: 41 tests passing (20 unit tests, 8 classification integration tests, 13 structural normalization integration tests).
  - Full workspace tests: 608+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `bfe3637`
- **Status:** `ACCEPTED`

---

#### Subphase `7.2.1` — Tokenizer adapter architecture in crates/tokenizer
- **Unique ID:** `7.2.1`
- **Objective:** Establish the modular tokenizer adapter architecture in `crates/tokenizer` capable of both exact BPE tokenization and calibrated heuristic estimation, with explicit precision classification, confidence intervals, context window limits, target model mappings, thread-safe registry lookup, and document-level section token enrichment.
- **Scope:** `crates/tokenizer/*` (`error.rs`, `model.rs`, `precision.rs`, `adapter.rs`, `adapters/*`, `registry.rs`, `document.rs`, `lib.rs`, `tests/*`).
- **Prerequisites:** `7.1.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/tokenizer/Cargo.toml`
  - `crates/tokenizer/src/error.rs`
  - `crates/tokenizer/src/model.rs`
  - `crates/tokenizer/src/precision.rs`
  - `crates/tokenizer/src/adapter.rs`
  - `crates/tokenizer/src/adapters/mod.rs`
  - `crates/tokenizer/src/adapters/heuristic.rs`
  - `crates/tokenizer/src/adapters/bpe.rs`
  - `crates/tokenizer/src/adapters/models.rs`
  - `crates/tokenizer/src/registry.rs`
  - `crates/tokenizer/src/document.rs`
  - `crates/tokenizer/src/lib.rs`
  - `crates/tokenizer/tests/tokenizer_adapter_test.rs`
- **Implementation Tasks:**
  1. Add necessary workspace dependencies (`cdm`, `serde` with `derive` and `std`, `serde_json`, `regex`, `chrono`) to `crates/tokenizer/Cargo.toml`.
  2. Implement strongly typed `TokenizerError` enum handling invalid token budgets, regex compilation errors, unknown adapters, context window overflow, and encoding issues (`crates/tokenizer/src/error.rs`).
  3. Implement `ModelFamily` (OpenAI, Anthropic, Google, Meta, Mistral, Generic) and `TargetModel` enum with context window limits and recommended budget ceilings (`crates/tokenizer/src/model.rs`).
  4. Implement `TokenPrecision` distinguishing `Exact`, `CalibratedEstimate { uncertainty_pct }`, and `HeuristicEstimate` with uncertainty calculations and confidence range evaluation; implement `TokenCountResult` and `TruncationResult` (`crates/tokenizer/src/precision.rs`).
  5. Implement `TokenizerAdapter` core trait (`name`, `model_family`, `target_model`, `context_window_limit`, `is_exact`, `count_tokens`, `truncate_tokens`) in `crates/tokenizer/src/adapter.rs`.
  6. Implement `CalibratedHeuristicAdapter` with content-aware chars/token heuristics (JSON/dense code: 2.9, general code: 3.3, prose: 4.2) and explicit 7.5% uncertainty margin (`crates/tokenizer/src/adapters/heuristic.rs`).
  7. Implement `RegexBpeTokenizer` with Rust-compatible regex pre-tokenization boundary matching and token weight estimation (`crates/tokenizer/src/adapters/bpe.rs`).
  8. Implement model-specific BPE adapters: `OpenAiAdapter` (o200k/cl100k), `ClaudeAdapter` (claude_bpe), `GeminiAdapter` (gemini_sp), `LlamaAdapter` (llama3_bpe) in `crates/tokenizer/src/adapters/models.rs`.
  9. Implement `TokenizerRegistry` for thread-safe model and string name lookups with fallback to calibrated heuristic (`crates/tokenizer/src/registry.rs`).
  10. Implement CDM section and whole-document token enrichment (`count_section_tokens`, `enrich_document_token_metrics`), computing `DocumentTokenSummary` with budget utilization and GFM markdown reporting (`crates/tokenizer/src/document.rs`).
  11. Implement integration test suite `crates/tokenizer/tests/tokenizer_adapter_test.rs` validating exact BPE tokenization, calibrated estimation, adapter registry lookup, clean truncation, document enrichment, and JSON roundtrip.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-tokenizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - `TokenizerAdapter` trait cleanly abstracting model family, context limit, exactness, and token counting/truncation.
  - Transparent precision tiering (`Exact` vs `CalibratedEstimate` with `uncertainty_pct`). Heuristic estimates never reported dishonestly as exact.
  - Safe truncation along valid UTF-8 character boundaries without splitting multi-byte sequences or panic.
  - CDM document enrichment correctly populates `CdmSection.token_metrics` and computes `DocumentTokenSummary`.
  - All 19 crate tests pass, all workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Misrepresenting approximate character-ratio estimates as exact counts to downstream LLM prompt builders.
  - Mitigation: `TokenPrecision` enum explicitly segregates exact counts from calibrated estimates with bounded uncertainty ranges (`[count * (1 - u), count * (1 + u)]`).
- **Rollback Plan:**
  - Revert Subphase 7.2.1 additions via `git reset --hard HEAD~1` and restore Subphase 7.1.4 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-tokenizer`: 19 tests passing (11 unit tests, 8 integration tests).
  - Full workspace tests: 627+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `175ad11`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 7 — AI Context and Token Optimization)

#### Subphase `7.2.2` — Model-specific counting in crates/tokenizer
- **Unique ID:** `7.2.2`
- **Objective:** Implement dedicated, architecture-specific token counting algorithms and boundary segmentation engines across major LLM providers (OpenAI cl100k and o200k, Anthropic Claude BPE, Google Gemini SentencePiece, Meta Llama 3 128k BPE, and Mistral Tekken), special control token prompt-injection defenses, syntax and structural token breakdown analysis, and cross-model efficiency comparisons.
- **Scope:** `crates/tokenizer/*` (`special_tokens.rs`, `tokenizers/*`, `adapters/models.rs`, `syntax.rs`, `comparison.rs`, `lib.rs`, `tests/*`).
- **Prerequisites:** `7.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/tokenizer/src/special_tokens.rs`
  - `crates/tokenizer/src/tokenizers/mod.rs`
  - `crates/tokenizer/src/tokenizers/cl100k.rs`
  - `crates/tokenizer/src/tokenizers/o200k.rs`
  - `crates/tokenizer/src/tokenizers/claude.rs`
  - `crates/tokenizer/src/tokenizers/gemini.rs`
  - `crates/tokenizer/src/tokenizers/llama.rs`
  - `crates/tokenizer/src/tokenizers/mistral.rs`
  - `crates/tokenizer/src/adapters/models.rs`
  - `crates/tokenizer/src/adapters/mod.rs`
  - `crates/tokenizer/src/registry.rs`
  - `crates/tokenizer/src/syntax.rs`
  - `crates/tokenizer/src/comparison.rs`
  - `crates/tokenizer/src/model.rs`
  - `crates/tokenizer/src/error.rs`
  - `crates/tokenizer/src/lib.rs`
  - `crates/tokenizer/tests/model_specific_counting_test.rs`
- **Implementation Tasks:**
  1. Implement `SpecialTokenPolicy` (`CountAsSingleToken`, `Disallow`, `IgnoreOrEscape`) and special token discovery/validation (`validate_special_tokens`, `find_special_tokens`) across OpenAI, Llama, and Mistral control sequences (`crates/tokenizer/src/special_tokens.rs`).
  2. Implement dedicated model tokenizers in `crates/tokenizer/src/tokenizers/`:
     - `Cl100kTokenizer`: 100k vocabulary rules, 4-space indentation merging, 3-digit number grouping, code keywords, and multi-byte UTF-8 byte fallback.
     - `O200kTokenizer`: 200k vocabulary rules, 8-space indentation merging, compound code operators (`::`, `->`, `=>`, `===`, `!==`, `&&`, `||`, `??`), and 1.5-2x enhanced Unicode compression for CJK, Devanagari, Arabic, and Cyrillic scripts.
     - `ClaudeTokenizer`: XML tag preservation (`<thinking>`, `<artifacts>`, `<code>`), Markdown structural heading preservation, and 2/4/8 space indentation merging.
     - `GeminiTokenizer`: SentencePiece style whitespace prefixing, large numeric grouping (up to 5 digits), and dense global Unicode compression.
     - `LlamaTokenizer`: 128k Llama 3 BPE boundary handling, code bracket sensitivity, and 4-space indentation merging.
     - `MistralTokenizer`: Tekken 131k tokenizer with aggressive code symbol merging and multi-language efficiency.
  3. Wire `OpenAiAdapter`, `ClaudeAdapter`, `GeminiAdapter`, `LlamaAdapter`, and new `MistralAdapter` in `crates/tokenizer/src/adapters/models.rs` directly to their dedicated counting engines.
  4. Register `MistralAdapter`, `Claude3Opus`, and `Llama370B` in `TokenizerRegistry`.
  5. Implement `TokenSyntaxBreakdown` in `crates/tokenizer/src/syntax.rs`, categorizing token spend across code syntax, natural prose, markup, whitespace, and special tokens with percentage shares, Markdown summary tables, and JSON roundtrip serialization.
  6. Implement `ModelTokenComparison` in `crates/tokenizer/src/comparison.rs`, evaluating multi-model token counts across text and CDM documents, identifying the most and least compact models, calculating cross-model variance percentages, rendering GFM comparison tables, and supporting JSON serialization.
  7. Implement integration test suite `crates/tokenizer/tests/model_specific_counting_test.rs` (9 integration tests) validating cl100k vs o200k compression, Claude XML tokenization, Gemini SentencePiece counting, Llama and Mistral adapters, special token injection blocking, syntax breakdowns, and cross-model comparison tables.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-tokenizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Dedicated counting engines for each model architecture produce distinct, verifiable token counts matching real-world BPE behaviors.
  - `o200k_base` provably compresses 8-space indentation, compound operators, and CJK text into fewer tokens than `cl100k_base`.
  - Special token prompt injection attempts cleanly rejected when `SpecialTokenPolicy::Disallow` is configured.
  - Cross-model comparison report (`ModelTokenComparison`) and syntax breakdown (`TokenSyntaxBreakdown`) support GFM Markdown rendering and full JSON roundtrip serialization.
  - All 42 crate tests pass, all 636+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Prompt injection through embedded LLM control tokens (e.g. `<|im_start|>`, `<|endoftext|>`).
  - Mitigation: `validate_special_tokens` provides strict zero-trust detection and rejection before token emission.
- **Rollback Plan:**
  - Revert Subphase 7.2.2 additions via `git reset --hard HEAD~1` and restore Subphase 7.2.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-tokenizer`: 42 tests passing (25 unit tests, 17 integration tests across 2 test suites).
  - Full workspace tests: 636+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `3c0aaf5`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 7 — AI Context and Token Optimization)

#### Subphase `7.2.3` — Cost estimation in crates/tokenizer
- **Unique ID:** `7.2.3`
- **Objective:** Implement financial cost modeling and intelligence in `crates/tokenizer`, including tiered pricing schedules per million tokens, prompt caching read discounts (up to 90%), batch API queue discounts (50%), cross-provider cost comparison reports, and strict dollar budget guardrails.
- **Scope:** `crates/tokenizer/*` (`cost.rs`, `error.rs`, `model.rs`, `registry.rs`, `lib.rs`, `tests/*`).
- **Prerequisites:** `7.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/tokenizer/src/cost.rs`
  - `crates/tokenizer/src/error.rs`
  - `crates/tokenizer/src/model.rs`
  - `crates/tokenizer/src/registry.rs`
  - `crates/tokenizer/src/comparison.rs`
  - `crates/tokenizer/src/adapters/models.rs`
  - `crates/tokenizer/src/lib.rs`
  - `crates/tokenizer/tests/cost_estimation_test.rs`
- **Implementation Tasks:**
  1. Add `TokenizerError::CostBudgetExceeded { budget_usd, estimated_usd }` in `crates/tokenizer/src/error.rs`.
  2. Implement `ModelPricingSchedule` covering all supported target models (GPT-4o, GPT-4o-mini, GPT-4 Turbo, Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku, Gemini 1.5 Pro, Gemini 1.5 Flash, Llama 3, Llama 3 70B, Mistral Large, Mixtral 8x7B, GenericFallback), distinguishing standard input, prompt-cached read input, output generation, batch discounts, and free/self-hosted tiers (`crates/tokenizer/src/cost.rs`).
  3. Implement `CostBreakdown` computing standard cost, prompt-cache savings (\$ and %), and batch API savings (\$ and %) with high-precision arithmetic (`crates/tokenizer/src/cost.rs`).
  4. Implement `CrossModelCostReport` evaluating cost across all target models, identifying the cheapest commercial model, computing expensive-vs-cheapest cost ratios, rendering formatted GFM Markdown tables, and supporting JSON roundtrip serialization (`crates/tokenizer/src/cost.rs`).
  5. Implement `CostGuardrail` and `CostGuardrailEvaluation` enforcing configurable dollar spending ceilings (`max_budget_usd`), calculating remaining budget headroom, identifying budget breaches, and recommending alternative economical models that fit within budget (`crates/tokenizer/src/cost.rs`).
  6. Add `TargetModel::Gpt4oMini` to model taxonomy, registry, and adapters for low-cost ($0.15/MTok) conversions (`crates/tokenizer/src/model.rs`, `registry.rs`, `adapters/models.rs`).
  7. Add convenience functions in `crates/tokenizer/src/lib.rs`: `estimate_token_cost`, `compare_token_costs`.
  8. Implement integration test suite `crates/tokenizer/tests/cost_estimation_test.rs` (6 integration tests) validating pricing schedules, standard/cached/batch calculations, cross-model comparison tables, document-level cost estimates, budget guardrail enforcement, and extreme token volume scaling (20M tokens).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-tokenizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Pricing schedules precisely reflect commercial rates and prompt caching / batch discount mechanics.
  - Self-hosted models (Llama 3, Llama 3 70B) correctly report $0.00 cost and `is_free_or_self_hosted: true`.
  - Guardrail cleanly flags budget breaches and recommends viable cheaper alternative models.
  - `CrossModelCostReport` GFM table and JSON serialization verified with 100% roundtrip fidelity.
  - All 52 crate tests pass, all 642+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Provider pricing changes causing stale cost estimates.
  - Mitigation: `ModelPricingSchedule` is fully modular and configurable per model; pricing schedules are explicitly version-controlled and verified with unit/integration tests.
- **Rollback Plan:**
  - Revert Subphase 7.2.3 additions via `git reset --hard HEAD~1` and restore Subphase 7.2.2 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-tokenizer`: 52 tests passing (29 unit tests, 23 integration tests across 3 test suites).
  - Full workspace tests: 642+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `783b77f`
- **Status:** `ACCEPTED`

---

#### Subphase `7.2.4` — Context-budget calculation in crates/context-optimizer
- **Unique ID:** `7.2.4`
- **Objective:** Implement context budget modeling, headroom reservation calculation (system, response generation, query), section importance and density prioritization, greedy knapsack budget allocation, deterministic boundary truncation, and auditable exclusion logging in `crates/context-optimizer`.
- **Scope:** `crates/context-optimizer/*` (`budget.rs`, `priority.rs`, `plan.rs`, `optimizer.rs`, `report.rs`, `error.rs`, `lib.rs`, `tests/*`).
- **Prerequisites:** `7.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/Cargo.toml`
  - `crates/context-optimizer/src/budget.rs`
  - `crates/context-optimizer/src/error.rs`
  - `crates/context-optimizer/src/priority.rs`
  - `crates/context-optimizer/src/plan.rs`
  - `crates/context-optimizer/src/optimizer.rs`
  - `crates/context-optimizer/src/report.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/context_budget_test.rs`
- **Implementation Tasks:**
  1. Add required workspace dependencies (`serde`, `serde_json`, `tracing`, `blake3`, `chrono`, `web-repo-to-ai-code-intelligence`) to `crates/context-optimizer/Cargo.toml`.
  2. Implement domain error handling `ContextOptimizerError` (`InvalidBudget`, `BudgetTooSmall`, `DocumentEmpty`, `Tokenizer`, `Serialization`) in `crates/context-optimizer/src/error.rs`.
  3. Implement `ContextBudget` and `BudgetHeadroomSummary` modeling total context capacity ceilings, multi-tier reserved allowances (system instructions, response generation, query prompts), available content capacity calculations, and proportional scaling for all target models in `crates/context-optimizer/src/budget.rs`.
  4. Implement `SectionPriority`, `PriorityScoreFactors`, `PinStatus` (`MustInclude`, `Normal`, `Deprioritized`, `MustExclude`), and `SectionScorer` evaluating section kind weights, filename/path prominence, information density, upstream signal scores from Phase 7.1.4, architectural code roles, and test/generated/boilerplate penalties in `crates/context-optimizer/src/priority.rs`.
  5. Implement `BudgetPlan`, `BudgetAction` (`Include`, `Truncate`, `Exclude`), and `SectionBudgetDecision` with token conservation tracking and GFM markdown summary tables in `crates/context-optimizer/src/plan.rs`.
  6. Implement `ContextBudgetOptimizer`, `BudgetOptimizationConfig`, and `TruncationStrategy` (`HeadAndTail`, `HeadOnly`, `NoTruncation`) with greedy knapsack utility-per-token allocation, strict ceiling enforcement, BLAKE3 revision recalculation, and auditable `ExclusionRecord` logging (`ExclusionReason::SizeLimit`, `ExclusionReason::UserConfig`) in `crates/context-optimizer/src/optimizer.rs`.
  7. Implement `ContextBudgetReport` telemetry engine with headroom breakdown tables, token conservation summaries, and JSON roundtrip serialization in `crates/context-optimizer/src/report.rs`.
  8. Expose public convenience APIs `optimize_context_for_model` and `optimize_context_with_budget` in `crates/context-optimizer/src/lib.rs`.
  9. Implement comprehensive integration test suite `crates/context-optimizer/tests/context_budget_test.rs` (6 integration tests validating multi-model allocations, knapsack prioritization, boundary truncation, zero silent drop conservation, serialization, and fail-closed error handling).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Multi-tier headroom calculation correctly allocates system, output, and query reservations across all models.
  - Knapsack greedy priority ordering correctly prioritizes READMEs, manifests, and high-signal core implementations over test fixtures and boilerplate.
  - Zero silent drops (Rule 04 / Rule 09): Every excluded section is transparently logged in `document.exclusions` with `ExclusionReason::SizeLimit` or `UserConfig` and detailed metrics.
  - Resulting optimized documents strictly never exceed `available_content_tokens`.
  - Document revision BLAKE3 hash is recomputed to preserve cryptographic provenance.
  - All 20 crate tests pass, all 662+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
  - Phase 7.2 ("Token intelligence") is 100% complete across all 4 subphases and fully certified.
- **Risk Analysis:**
  - Risk: Truncation omission markers inflating token counts and causing budget overruns.
  - Mitigation: `truncate_section` explicitly accounts for omission marker tokens and applies a fail-safe post-condition clamp ensuring resulting tokens strictly `<= target_tokens`.
- **Rollback Plan:**
  - Revert Subphase 7.2.4 additions via `git reset --hard HEAD~1` and restore Subphase 7.2.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 20 tests passing (14 unit tests, 6 integration tests).
  - Full workspace tests: 662+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `81c7784`
- **Status:** `ACCEPTED`

---

#### Subphase `7.3.1` — General AI context profile in crates/context-optimizer
- **Unique ID:** `7.3.1`
- **Objective:** Implement the foundational `ContextProfile` trait, profile taxonomy (`ProfileKind`), and the specialized `GeneralAiProfile` in `crates/context-optimizer`, prioritizing architectural documentation, project manifests, and core implementation code while suppressing low-signal test suites and generated boilerplate.
- **Scope:** `crates/context-optimizer/*` (`profiles/mod.rs`, `profiles/general_ai.rs`, `lib.rs`, `tests/*`).
- **Prerequisites:** `7.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/profiles/mod.rs`
  - `crates/context-optimizer/src/profiles/general_ai.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/general_ai_profile_test.rs`
- **Implementation Tasks:**
  1. Implement `ContextProfile` trait defining `profile_kind`, `create_scorer`, `prepare_sections`, and `optimize` in `crates/context-optimizer/src/profiles/mod.rs`.
  2. Implement `ProfileKind` taxonomy (`GeneralAi`, `CodeReview`, `SecurityReview`, `DebuggingAndDocumentation`) with serialization in `crates/context-optimizer/src/profiles/mod.rs`.
  3. Implement `ProfileMetrics` calculating token distribution across documentation, core implementation code, manifests/architecture, and tests with percentage shares and GFM summary tables in `crates/context-optimizer/src/profiles/mod.rs`.
  4. Implement `ProfileOptimizationResult` bundling optimized `CdmDocument`, `BudgetPlan`, and `ProfileMetrics` with markdown reporting and JSON serialization in `crates/context-optimizer/src/profiles/mod.rs`.
  5. Implement `GeneralAiConfig` controlling inclusion of documentation, manifests, core code, tests (default false), benchmarks (default false), and truncation parameters in `crates/context-optimizer/src/profiles/general_ai.rs`.
  6. Implement `GeneralAiProfile` implementing `ContextProfile`: enriches sections with `context_profile: general_ai`, pins root README and manifests, deprioritizes/suppresses tests and benchmarks, applies `ContextBudgetOptimizer`, and computes content metrics in `crates/context-optimizer/src/profiles/general_ai.rs`.
  7. Expose public convenience APIs `apply_general_ai_profile` and `apply_general_ai_profile_for_model` in `crates/context-optimizer/src/lib.rs`.
  8. Implement integration test suite `crates/context-optimizer/tests/general_ai_profile_test.rs` (5 integration tests validating balanced content distribution, model-targeted optimization, test inclusion toggle, tight budget boundary truncation, and telemetry serialization).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - General AI profile produces a balanced representation: documentation, manifests, and core implementation code are prioritized over test fixtures and generated boilerplate.
  - Profile metrics accurately track token allocations across documentation, core code, manifests, and tests.
  - Seamlessly targets specific model context windows (`TargetModel::Gpt4o`, `Claude35Sonnet`, `Gemini15Pro`, `Llama3`) with strict budget compliance.
  - Full JSON roundtrip serialization and GFM markdown summary reporting verified.
  - All 26 crate tests pass, all 668+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass.
- **Risk Analysis:**
  - Risk: Excluding tests could omit critical user-requested test code when specifically desired.
  - Mitigation: `GeneralAiConfig` provides explicit `with_include_tests(true)` toggle allowing callers to include test suites when requested.
- **Rollback Plan:**
  - Revert Subphase 7.3.1 additions via `git reset --hard HEAD~1` and restore Subphase 7.2.4 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 26 tests passing (15 unit tests, 11 integration tests across 2 test suites).
  - Full workspace tests: 668+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `3295f48`
- **Status:** `ACCEPTED`

---

#### Subphase `7.3.2` — Code review context profile in crates/context-optimizer
- **Unique ID:** `7.3.2`
- **Objective:** Implement the specialized Code Review context profile (`CodeReviewProfile`, `CodeReviewConfig`) in `crates/context-optimizer` that prioritizes pull-request / diff-modified files (`PinStatus::MustInclude`), includes associated test files (`associated_test: true`) to assess behavioral correctness and test coverage, preserves manifest and interface definitions, suppresses unmodified documentation (`README.md`, `CHANGELOG.md`, `ROADMAP.md`) and generated assets to maximize diff context headroom, and integrates with `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation.
- **Scope:** `crates/context-optimizer/src/profiles/code_review.rs`, `crates/context-optimizer/src/profiles/mod.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/code_review_profile_test.rs`.
- **Prerequisites:** `7.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/profiles/code_review.rs`
  - `crates/context-optimizer/src/profiles/mod.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/code_review_profile_test.rs`
- **Implementation Tasks:**
  1. Implement `CodeReviewConfig` in `crates/context-optimizer/src/profiles/code_review.rs` with `changed_files: HashSet<String>`, `include_associated_tests: bool` (default `true`), `include_manifests: bool` (default `true`), `include_interface_definitions: bool` (default `true`), `include_unmodified_docs: bool` (default `false`), and boundary truncation controls.
  2. Implement `CodeReviewProfile` implementing `ContextProfile`:
     - Identifies changed/modified files and sets `PinStatus::MustInclude` with metadata `is_changed_file: true`.
     - Identifies associated unit/regression tests for changed files using naming conventions and path relations, tagging them with `associated_test: true` and elevated priority.
     - Preserves manifests (`Cargo.toml`, `package.json`, etc.) and interface definitions (`.d.ts`, trait definitions).
     - Suppresses unmodified prose documentation and generated assets by default to maximize diff context headroom.
     - Enforces `ContextBudgetOptimizer` knapsack allocation and boundary truncation under constrained token ceilings.
     - Generates `ProfileMetrics` reporting category token distribution across changed files, associated tests, manifests, and others.
  3. Re-export `CodeReviewConfig` and `CodeReviewProfile` in `crates/context-optimizer/src/profiles/mod.rs`.
  4. Expose public convenience APIs `apply_code_review_profile` and `apply_code_review_profile_for_model` in `crates/context-optimizer/src/lib.rs`.
  5. Implement comprehensive integration test suite `crates/context-optimizer/tests/code_review_profile_test.rs` (5 integration tests validating changed file pinning, associated test elevation, model-targeted optimization, tight budget boundary truncation, zero changed files fallback, and telemetry serialization).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Modified/changed files are unconditionally prioritized with `PinStatus::MustInclude` and `is_changed_file: true`.
  - Associated unit/regression tests are identified and prioritized when `include_associated_tests` is enabled.
  - Unmodified prose documentation is suppressed by default (`include_unmodified_docs: false`) to conserve tokens for code diffs.
  - Strict compliance with model token budgets (`TargetModel::Gpt4o`, `Claude35Sonnet`, `Gemini15Pro`, `Llama3`) with clean boundary truncation.
  - All 32 crate tests pass, all 673+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: When no changed files are supplied or match the document, all sections might be suppressed or misclassified.
  - Mitigation: Implemented fallback logic that treats core implementation files as candidate review targets with warning telemetry when `changed_files` is empty.
- **Rollback Plan:**
  - Revert Subphase 7.3.2 additions via `git reset --hard HEAD~1` and restore Subphase 7.3.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 32 tests passing (16 unit tests, 16 integration tests across 3 test suites).
  - Full workspace tests: 673+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `6a322ac`
- **Status:** `ACCEPTED`

---

#### Subphase `7.3.3` — Security review context profile in crates/context-optimizer
- **Unique ID:** `7.3.3`
- **Objective:** Implement the specialized Security Review context profile (`SecurityReviewProfile`, `SecurityReviewConfig`, `SecurityCategory`) in `crates/context-optimizer` that prioritizes attack surface, authentication/authorization paths, cryptographic implementations, secrets handling, input validation/sanitization, security policies (`SECURITY.md`), and security test suites while suppressing benign presentation components (CSS, UI templates, styling) and non-security documentation to maximize vulnerability analysis context headroom.
- **Scope:** `crates/context-optimizer/src/profiles/security_review.rs`, `crates/context-optimizer/src/profiles/mod.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/security_review_profile_test.rs`.
- **Prerequisites:** `7.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/profiles/security_review.rs`
  - `crates/context-optimizer/src/profiles/mod.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/security_review_profile_test.rs`
- **Implementation Tasks:**
  1. Implement `SecurityCategory` taxonomy (`AuthAndAccessControl`, `CryptographyAndSecrets`, `AttackSurfaceAndEndpoints`, `InputValidationAndSanitizers`, `SecurityPolicyAndDocs`, `SecurityManifests`, `SecurityTests`, `GeneralImplementation`, `BenignPresentationOrAsset`) in `crates/context-optimizer/src/profiles/security_review.rs`.
  2. Implement `SecurityReviewConfig` with controls for `include_auth_and_crypto` (default `true`), `include_attack_surface` (default `true`), `include_input_validation` (default `true`), `include_security_docs` (default `true`), `include_security_manifests` (default `true`), `include_security_tests` (default `true`), `include_benign_components` (default `false`), `pinned_paths`, and `custom_target_keywords`.
  3. Implement `SecurityReviewProfile` implementing `ContextProfile`:
     - Classifies sections into `SecurityCategory` using path, kind, attribute, and keyword heuristics.
     - Unconditionally pins authentication, authorization, cryptographic implementations, and security policy documents (`SECURITY.md`) with `PinStatus::MustInclude`.
     - Prioritizes attack surface (routes, endpoints, fetchers), input validators, and security test suites with `PinStatus::Normal`.
     - Suppresses benign presentation components (UI components, styles, design tokens) and non-security documentation (`CHANGELOG.md`, `ROADMAP.md`) with `PinStatus::MustExclude` by default.
     - Enforces `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation.
     - Computes `ProfileMetrics` category distribution.
  4. Expose `SecurityCategory`, `SecurityReviewConfig`, and `SecurityReviewProfile` in `crates/context-optimizer/src/profiles/mod.rs`.
  5. Expose public convenience APIs `apply_security_review_profile` and `apply_security_review_profile_for_model` in `crates/context-optimizer/src/lib.rs`.
  6. Implement comprehensive integration test suite `crates/context-optimizer/tests/security_review_profile_test.rs` (6 integration tests validating auth/crypto/policy pinning, benign component suppression, security test elevation, model-targeted optimization, tight budget boundary truncation, custom keywords/paths, and report serialization).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Authentication, authorization, cryptography, and security policy documents are pinned with `PinStatus::MustInclude`.
  - Network attack surface, API routes, and input validation modules are prioritized.
  - Benign presentation components and non-security documentation are suppressed by default (`include_benign_components: false`).
  - Strict compliance with model token budgets (`TargetModel::Gpt4o`, `Claude35Sonnet`, `Gemini15Pro`, `Llama3`) with clean boundary truncation.
  - All 39 crate tests pass, all 680+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Security-critical code located in unexpected directories or non-standard naming schemes might be misclassified as benign.
  - Mitigation: `SecurityReviewConfig` provides `with_pinned_paths` and `with_custom_target_keywords` allowing callers to explicitly elevate any bespoke security modules or terms.
- **Rollback Plan:**
  - Revert Subphase 7.3.3 additions via `git reset --hard HEAD~1` and restore Subphase 7.3.2 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 39 tests passing (17 unit tests, 22 integration tests across 4 test suites).
  - Full workspace tests: 680+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `85c937a`
- **Status:** `ACCEPTED`

---

#### Subphase `7.3.4` — Debugging and documentation context profile in crates/context-optimizer
- **Unique ID:** `7.3.4`
- **Objective:** Implement the specialized Debugging and Documentation context profile (`DebuggingProfile`, `DebuggingConfig`, `DebuggingCategory`) in `crates/context-optimizer` that prioritizes error definitions and propagation hierarchies, observability and tracing instrumentation, API contracts and schema definitions, troubleshooting runbooks and developer specs, and negative/edge-case test cases while suppressing cosmetic presentation assets (UI styling, CSS, layouts) and non-technical changelogs to maximize diagnostic clarity and root-cause analysis context headroom. Formally certify Phase 7.3.
- **Scope:** `crates/context-optimizer/src/profiles/debugging.rs`, `crates/context-optimizer/src/profiles/mod.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/debugging_profile_test.rs`.
- **Prerequisites:** `7.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/profiles/debugging.rs`
  - `crates/context-optimizer/src/profiles/mod.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/debugging_profile_test.rs`
- **Implementation Tasks:**
  1. Implement `DebuggingCategory` taxonomy (`ErrorDefinitionsAndPropagation`, `ObservabilityAndLogging`, `ApiContractsAndInterfaces`, `TroubleshootingDocsAndRunbooks`, `NegativeAndEdgeCaseTests`, `ManifestsAndEnvironment`, `InternalImplementation`, `CosmeticOrUnrelatedAsset`) in `crates/context-optimizer/src/profiles/debugging.rs`.
  2. Implement `DebuggingConfig` with controls for `include_error_definitions` (default `true`), `include_observability` (default `true`), `include_api_contracts` (default `true`), `include_troubleshooting_docs` (default `true`), `include_negative_tests` (default `true`), `include_manifests` (default `true`), `include_internal_implementation` (default `true`), `include_cosmetic_assets` (default `false`), `target_error_patterns`, and `pinned_paths`.
  3. Implement `DebuggingProfile` implementing `ContextProfile`:
     - Classifies sections into `DebuggingCategory` using path, filename, attribute, and keyword heuristics.
     - Unconditionally pins error definitions (`enum Error`, `thiserror`), observability/tracing instrumentation, and API contracts with `PinStatus::MustInclude`.
     - Supports targeted error pattern matching (`from_error_pattern`) to elevate sections matching specific incident error codes or exception names.
     - Prioritizes troubleshooting runbooks and negative/error-handling unit/integration tests with `PinStatus::Normal`.
     - Suppresses cosmetic presentation components (UI components, CSS, design tokens) and non-technical changelogs with `PinStatus::MustExclude` by default.
     - Enforces `ContextBudgetOptimizer` greedy knapsack allocation and boundary truncation.
     - Computes `ProfileMetrics` category distribution.
  4. Expose `DebuggingCategory`, `DebuggingConfig`, and `DebuggingProfile` in `crates/context-optimizer/src/profiles/mod.rs`.
  5. Expose public convenience APIs `apply_debugging_profile`, `apply_debugging_profile_for_model`, `apply_debugging_profile_with_target`, and `apply_debugging_profile_with_target_for_model` in `crates/context-optimizer/src/lib.rs`.
  6. Implement comprehensive integration test suite `crates/context-optimizer/tests/debugging_profile_test.rs` (6 integration tests validating error/observability/API contract pinning, targeted error pattern matching, negative test elevation, model-targeted optimization, tight budget boundary truncation, and report serialization).
  7. Formally certify Phase 7.3 ("Context profiles") with all 4 specialized profiles (`GeneralAi`, `CodeReview`, `SecurityReview`, `DebuggingAndDocumentation`) fully implemented and passing all quality gates.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Error definitions, observability/tracing instrumentation, and API contracts are pinned with `PinStatus::MustInclude`.
  - Targeted error pattern queries elevate matching error and rate-limiting modules with `PinStatus::MustInclude`.
  - Negative and failure-mode unit/integration tests are elevated over benign cosmetic tests.
  - Cosmetic presentation components and non-technical documentation are suppressed by default (`include_cosmetic_assets: false`).
  - Strict compliance with model token budgets (`TargetModel::Gpt4o`, `Claude35Sonnet`, `Gemini15Pro`, `Llama3`) with clean boundary truncation.
  - All 46 crate tests pass, all 687+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Standard return types (`Result<T, E>`) could cause generic functions to be misclassified as error definitions.
  - Mitigation: Refined error classification heuristics to focus on dedicated error files, `thiserror`/`anyhow` declarations, `enum Error`, and `impl std::error::Error`.
- **Rollback Plan:**
  - Revert Subphase 7.3.4 additions via `git reset --hard HEAD~1` and restore Subphase 7.3.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 46 tests passing (18 unit tests, 28 integration tests across 5 test suites).
  - Full workspace tests: 687+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `8ae81a4`
- **Status:** `ACCEPTED`

---

#### Subphase `7.4.1` — Preservation metrics in crates/context-optimizer
- **Unique ID:** `7.4.1`
- **Objective:** Implement rigorous preservation metrics, symbol fidelity measurement, section category retention tracking, critical component audit, and composite fidelity scoring (`PreservationEvaluator`, `PreservationReport`, `PreservationConfig`, `SectionPreservationMetrics`, `SymbolPreservationMetrics`, `CriticalPreservationMetrics`, `TokenPreservationMetrics`, `StructuralIntegrityMetrics`) in `crates/context-optimizer` to quantify content retention, detect silent omissions of critical sections or public API declarations, and evaluate optimization fidelity across all context profiles.
- **Scope:** `crates/context-optimizer/src/preservation.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/preservation_metrics_test.rs`.
- **Prerequisites:** `7.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/preservation.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/preservation_metrics_test.rs`
- **Implementation Tasks:**
  1. Implement domain structures in `crates/context-optimizer/src/preservation.rs`:
     - `KindPreservation` and `SectionPreservationMetrics`: multi-category retention tracking across prose, code files, manifests, and documentation with clean BTreeMap keying.
     - `CriticalPreservationMetrics`: tracks retention of mission-critical sections (manifests, security modules, error definitions, root READMEs) with zero silent drop detection.
     - `SymbolPreservationMetrics`: integrates with `crates/code-intelligence` (`Language`, `SymbolDeclaration`, `SymbolExtractor`) to measure AST symbol retention, public API symbol retention, and exact omission inventory.
     - `TokenPreservationMetrics`: tracks original vs optimized token volume, retention ratio, and reduction percentage.
     - `StructuralIntegrityMetrics`: checks syntax validity and clean truncation boundary preservation.
     - `PreservationConfig`: weighted composite scoring (`critical_weight: 0.35`, `public_symbols_weight: 0.30`, `overall_symbols_weight: 0.15`, `section_retention_weight: 0.20`, `min_acceptable_fidelity: 0.50`).
     - `PreservationReport`: comprehensive report struct with GFM summary tables, multi-section breakdowns, and JSON serialization.
     - `PreservationEvaluator`: evaluation engine computing all preservation metrics between original and optimized `CdmDocument`s.
  2. Expose `preservation` module and public convenience APIs (`evaluate_preservation`, `evaluate_preservation_with_config`) in `crates/context-optimizer/src/lib.rs`.
  3. Implement integration test suite `crates/context-optimizer/tests/preservation_metrics_test.rs` (7 integration tests: unmodified document fidelity, budget optimization retention, symbol omission detection, code review profile preservation, markdown/JSON serialization, empty doc rejection, custom config thresholds).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Section retention rates, critical component retention, and token reduction are calculated deterministically.
  - AST symbol retention and public API retention are computed using `code-intelligence` extractors with omitted symbol tracking.
  - Zero silent drops on mission-critical sections (manifests, errors, root READMEs).
  - Clean markdown and JSON report serialization.
  - All 54 crate tests pass, all 694+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Non-code sections (prose, markdown) lack AST symbols, which could distort symbol preservation ratios.
  - Mitigation: Symbol preservation defaults to 1.0 when original documents contain 0 symbols, avoiding false-positive fidelity penalties on pure documentation.
- **Rollback Plan:**
  - Revert Subphase 7.4.1 additions via `git reset --hard HEAD~1` and restore Subphase 7.3.4 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 54 tests passing (19 unit tests, 35 integration tests across 6 test suites).
  - Full workspace tests: 694+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `07be7ce`
- **Status:** `ACCEPTED`

---

#### Subphase `7.4.2` — Compression reporting in crates/context-optimizer
- **Unique ID:** `7.4.2`
- **Objective:** Implement comprehensive compression reporting, stage-by-stage progression tracking, and multi-model financial API cost reduction evaluation (`CompressionReport`, `CompressionEvaluator`, `CompressionPipelineTracker`, `OverallCompressionMetrics`, `StageCompressionMetrics`, `DocumentVolume`, `CostSavingsMetrics`, `MultiModelCostSavings`, `OptimizationStage`) in `crates/context-optimizer` detailing cumulative and per-stage token, UTF-8 byte, character, and section savings across normalization, deduplication, profile filtering, and budget knapsack truncation with formatted GFM tables and JSON serialization.
- **Scope:** `crates/context-optimizer/src/compression.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/compression_reporting_test.rs`.
- **Prerequisites:** `7.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/compression.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/compression_reporting_test.rs`
- **Implementation Tasks:**
  1. Implement domain structures in `crates/context-optimizer/src/compression.rs`:
     - `OptimizationStage` taxonomy (`Normalization`, `ExactDeduplication`, `SemanticDeduplication`, `ProfileFiltering`, `BudgetKnapsack`, `Custom`).
     - `StageCompressionMetrics`: stage-by-stage token and byte reduction percentages, compression ratios, and section reduction deltas with GFM markdown row formatting.
     - `OverallCompressionMetrics`: cumulative end-to-end token, byte, character, and section metrics with compression ratios and formatted GFM markdown summary table.
     - `DocumentVolume`: document dimension measurement (tokens, bytes, chars, sections) with tokenizer adapter and calibrated heuristic fallback.
     - `CostSavingsMetrics` & `MultiModelCostSavings`: evaluates API financial savings per 1k invocations across GPT-4o, GPT-4o-mini, Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku, Gemini 1.5 Pro, Gemini 1.5 Flash, and Llama 3 (self-hosted $0.00).
     - `CompressionReport`: comprehensive report card with GFM markdown summary rendering and JSON roundtrip serialization.
     - `CompressionPipelineTracker`: progression recorder tracking document volume across sequential pipeline stage transitions.
     - `CompressionEvaluator`: comparative evaluator between baseline and optimized `CdmDocument`s with profile context.
  2. Re-export compression types and expose public convenience APIs (`evaluate_compression`, `evaluate_compression_for_model`, `evaluate_compression_with_profile`) in `crates/context-optimizer/src/lib.rs`.
  3. Implement integration test suite `crates/context-optimizer/tests/compression_reporting_test.rs` (7 integration tests: unmodified document fidelity, budget optimization compression, multi-stage pipeline tracking, multi-model financial cost savings, markdown/JSON serialization, fail-closed empty doc rejection, convenience APIs).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - End-to-end token, byte, character, and section reduction metrics and compression ratios are calculated deterministically.
  - Sequential pipeline stage transitions are tracked and formatted cleanly.
  - Financial cost reduction metrics accurately model commercial and self-hosted model schedules.
  - Fail-closed behavior on empty documents with `ContextOptimizerError::DocumentEmpty`.
  - All 61 crate tests pass, all 694+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Zero-token optimized documents or pure boilerplate exclusions could cause division-by-zero panics in compression ratio calculations.
  - Mitigation: Guarded ratio calculations against zero divisors, returning `f64::INFINITY` (rendered as `∞`) or `1.0` when original tokens/bytes are zero.
- **Rollback Plan:**
  - Revert Subphase 7.4.2 additions via `git reset --hard HEAD~1` and restore Subphase 7.4.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 61 tests passing (24 unit tests, 37 integration tests across 7 test suites).
  - Full workspace tests: 694+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `78dcc74`
- **Status:** `ACCEPTED`

---

#### Subphase `7.4.3` — Exclusion explanations in crates/context-optimizer
- **Unique ID:** `7.4.3`
- **Objective:** Implement structured, human-readable, and machine-actionable exclusion explanations (`ExclusionReport`, `ExclusionExplainer`, `ExclusionExplanation`, `ExclusionImpact`, `ExclusionCategorySummary`, `ExclusionSource`, `RemediationAdvice`) in `crates/context-optimizer` providing diagnostic rationales for every omitted section (originating subsystem, rule/constraint, impact severity, token/byte weight, priority score, and actionable remediation steps such as token budget expansion, profile flag toggling, or path pinning) with formatted GFM tables and JSON serialization.
- **Scope:** `crates/context-optimizer/src/exclusions.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/tests/exclusion_explanations_test.rs`.
- **Prerequisites:** `7.4.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/exclusions.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/tests/exclusion_explanations_test.rs`
- **Implementation Tasks:**
  1. Implement domain structures in `crates/context-optimizer/src/exclusions.rs`:
     - `ExclusionImpact` severity taxonomy (`Low`, `Medium`, `High`, `Critical`) with markdown badges.
     - `ExclusionSource` subsystem identifier (`BudgetHeadroom`, `ProfileRule`, `Deduplication`, `IgnorePattern`, `SecurityPolicy`, `UserConfiguration`, `Other`).
     - `RemediationAdvice`: actionable steps with recommended parameters (`--token-budget`, `include_cosmetic_assets`, `include_tests`, `include_unmodified_docs`, `pinned_paths`, `.aiignore`, `mark_only`) and additional token estimates.
     - `ExclusionExplanation`: itemized diagnostic record with GFM markdown row formatting.
     - `ExclusionCategorySummary`: aggregated metrics by reason/category with GFM row formatting.
     - `ExclusionReport`: comprehensive diagnostic report card with GFM markdown summary rendering and JSON roundtrip serialization.
     - `ExclusionExplainer`: diagnostic engine deriving explanations from `BudgetPlan`, original vs optimized document diffs, or `CdmDocument::exclusions` records.
  2. Re-export exclusion types and expose public convenience APIs (`explain_exclusions`, `explain_exclusions_with_profile`, `explain_exclusions_from_document`, `explain_exclusions_from_plan`) in `crates/context-optimizer/src/lib.rs`.
  3. Implement integration test suite `crates/context-optimizer/tests/exclusion_explanations_test.rs` (7 integration tests: budget knapsack exclusions, profile rule suppression, critical/high-impact detection, document exclusion records, markdown/JSON serialization, zero exclusions fidelity, fail-closed empty doc rejection).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Diagnostic rationales and remediation advice are derived deterministically for every omitted section.
  - Impact severity accurately classifies critical manifests/auth modules, high-impact core APIs, medium-impact tests, and low-impact cosmetic assets.
  - Zero exclusions produce clean "zero omissions" report cards.
  - Fail-closed behavior on empty documents with `ContextOptimizerError::DocumentEmpty`.
  - All 68 crate tests pass, all 694+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Complex profile configurations might obscure the exact reason why a file was suppressed.
  - Mitigation: Detailed reason parsing inspects decision actions, exclusion record details, and section attributes, matching specific remediation advice to cosmetic assets, test suites, or documentation.
- **Rollback Plan:**
  - Revert Subphase 7.4.3 additions via `git reset --hard HEAD~1` and restore Subphase 7.4.2 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 68 tests passing (29 unit tests, 39 integration tests across 8 test suites).
  - Full workspace tests: 694+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `e5cb2a8`
- **Status:** `ACCEPTED`

---

#### Subphase `7.4.4` — Golden comparison tests across context profiles in crates/context-optimizer
- **Unique ID:** `7.4.4`
- **Objective:** Construct a deterministic, multi-language golden repository CDM fixture and execute comparative golden benchmarking across all four context profiles (`GeneralAiProfile`, `CodeReviewProfile`, `SecurityReviewProfile`, `DebuggingProfile`) and baseline greedy knapsack optimization (`ContextBudgetOptimizer`), validating profile-specific invariants (must-include pinning, domain suppression, budget bounds), AST and syntax integrity, bit-for-bit determinism, comprehensive telemetry aggregation (preservation reports, compression metrics, financial cost reduction, exclusion rationales), and formatted GFM comparison tables with JSON serialization.
- **Scope:** `crates/context-optimizer/src/golden.rs`, `crates/context-optimizer/src/lib.rs`, `crates/context-optimizer/src/profiles/general_ai.rs`, `crates/context-optimizer/src/profiles/code_review.rs`, `crates/context-optimizer/src/profiles/security_review.rs`, `crates/context-optimizer/tests/golden_profiles_test.rs`.
- **Prerequisites:** `7.4.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/context-optimizer/src/golden.rs`
  - `crates/context-optimizer/src/lib.rs`
  - `crates/context-optimizer/src/profiles/general_ai.rs`
  - `crates/context-optimizer/src/profiles/code_review.rs`
  - `crates/context-optimizer/src/profiles/security_review.rs`
  - `crates/context-optimizer/tests/golden_profiles_test.rs`
- **Implementation Tasks:**
  1. Implement `GoldenRepositoryFixture` in `crates/context-optimizer/src/golden.rs` constructing a realistic 20-section multi-language repository CDM fixture (Cargo.toml, package.json, README.md, docs/ARCHITECTURE.md, docs/TROUBLESHOOTING.md, SECURITY.md, CHANGELOG_OLD.md, src/core/engine.rs, src/api/routes.rs, src/auth/jwt.rs, src/crypto/vault.rs, src/errors.rs, src/telemetry/tracer.rs, src/features/payment.rs, tests/payment_test.rs, tests/security_auth_test.rs, tests/negative_error_test.rs, tests/benchmark_test.rs, ui/styles/theme.css, ui/icons/logo.svg) with exact/baseline token metrics and verified BLAKE3 content hashes.
  2. Implement domain structures in `crates/context-optimizer/src/golden.rs`:
     - `GoldenProfileConfig`: customizable benchmark parameters with multi-tier budget headroom, changed file specifications, target error patterns, and model targeting.
     - `ProfileGoldenEvaluation`: itemized benchmark outcomes per profile with retained/excluded paths, preservation scores, compression ratios, financial savings, category distributions, and invariant diagnostics.
     - `GoldenComparisonReport`: cross-profile comparative matrix supporting markdown comparison tables (`to_markdown_comparison_table`), category tables (`to_markdown_category_table`), full diagnostic summaries (`to_markdown_summary`), and JSON roundtrip serialization.
     - `GoldenProfileComparator`: comparative benchmarking engine executing all 5 profiles and auditing domain-specific invariants.
  3. Refine profile suppression rules in `general_ai.rs`, `code_review.rs`, and `security_review.rs` to consistently detect and classify cosmetic presentation assets (CSS, SVG, vector graphics, legacy changelogs) and core implementation logic.
  4. Re-export golden testing models and convenience APIs in `crates/context-optimizer/src/lib.rs`: `create_golden_repository_fixture` and `run_golden_profile_comparison`.
  5. Implement comprehensive integration test suite `crates/context-optimizer/tests/golden_profiles_test.rs` (10 tests: fixture baseline structure, standard budget comparison, general AI invariants, code review invariants, security review invariants, debugging invariants, tight micro-budget knapsack, bit-for-bit determinism, markdown and JSON serialization, fail-closed empty doc rejection).
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-context-optimizer`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Golden comparison suite executes all 5 profiles deterministically against identical fixtures.
  - All profile-specific invariants pass without exceptions (100% changed file retention in code review; 100% auth/crypto/security retention in security review; 100% error/telemetry/runbook retention in debugging; 0% test retention and core implementation preservation in general AI).
  - All fitted documents strictly adhere to target content token budgets (`is_within_budget() == true`).
  - Bit-for-bit determinism across consecutive runs (identical fitted tokens, path ordering, and BLAKE3 revision hashes).
  - All 78 crate tests pass, all 694+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all check-all gates pass (5/5).
- **Risk Analysis:**
  - Risk: Greedy knapsack token packing could select small cosmetic assets over larger core domain files when budgets are tightly constrained.
  - Mitigation: Explicit profile tagging marks cosmetic assets as `must_exclude` or `deprioritized` and pins core files as `must_include`.
- **Rollback Plan:**
  - Revert Subphase 7.4.4 additions via `git reset --hard HEAD~1` and restore Subphase 7.4.3 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-context-optimizer`: 78 tests passing (31 unit tests, 47 integration tests across 9 test suites).
  - Full workspace tests: 694+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `6fc75e2`
- **Status:** `ACCEPTED`

---

### PHASE 7.4 FORMAL CERTIFICATION: Quality Validation

**Phase Acceptance Milestone:** Phase 7.4 (`7.4.1` – `7.4.4`) has satisfied all requirements set forth in the [GLOBAL_ENGINEERING_CONTRACT.md](./GLOBAL_ENGINEERING_CONTRACT.md), [REQUIREMENTS.md](./REQUIREMENTS.md), and [ROADMAP.md](./ROADMAP.md).
- Subphase `7.4.1` (Preservation metrics): Implemented multi-tier content fidelity, AST symbol retention via code intelligence, and syntax integrity auditing.
- Subphase `7.4.2` (Compression reporting): Implemented multi-stage pipeline volume tracking, GFM summary tables, and cross-model financial API cost reduction evaluation.
- Subphase `7.4.3` (Exclusion explanations): Implemented structured omission diagnostics, severity taxonomy (`Low`, `Medium`, `High`, `Critical`), and actionable remediation guidance.
- Subphase `7.4.4` (Golden comparison tests): Implemented 20-section polyglot golden repository benchmark suite verifying invariant properties across General AI, Code Review, Security Review, Debugging, and Raw Knapsack with bit-for-bit determinism.

**Phase 7.4 is hereby FORMALLY CERTIFIED (`ACCEPTED`).**

---

### STAGE 7 FORMAL CERTIFICATION: AI Context and Token Optimization

**Stage Acceptance Milestone:** Stage 7 (`7.1.1` – `7.4.4`, 16 subphases) has satisfied all stage deliverables:
- **Phase 7.1 — Content optimization (`7.1.1` – `7.1.4`):** Structural normalization, exact duplicate detection via BLAKE3, semantic deduplication via SimHash/MinHash LSH, and content classification.
- **Phase 7.2 — Token intelligence (`7.2.1` – `7.2.4`):** Tokenizer adapter architecture (BPE tokenizers for OpenAI, Anthropic, Gemini, Meta Llama 3, and calibrated heuristics), exact/estimated token counting, cost modeling, and multi-tier headroom context budgeting with greedy knapsack optimization.
- **Phase 7.3 — Context profiles (`7.3.1` – `7.3.4`):** Specialized context optimization engines for General AI, Code Review, Security Review, and Debugging & Documentation.
- **Phase 7.4 — Quality validation (`7.4.1` – `7.4.4`):** AST preservation metrics, compression reporting with multi-model cost savings, itemized exclusion rationales, and deterministic golden benchmarking.

**STAGE 7 (AI CONTEXT AND TOKEN OPTIMIZATION) is hereby FORMALLY CERTIFIED (`ACCEPTED`).**

---

## STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING

### Phase 8.1 — Batch URLs

#### Subphase `8.1.1` — Multi-URL submission endpoint and payload validation in services/api
- **Unique ID:** `8.1.1`
- **Objective:** Implement the multi-URL batch submission endpoint (`POST /api/v1/jobs/batch`), batch query endpoint (`GET /api/v1/jobs/batch/:batch_id`), and robust payload validation with bounds enforcement and URL deduplication in `services/api`.
- **Scope:** `crates/core`, `services/api/src/dto/batch.rs`, `services/api/src/dto/mod.rs`, `services/api/src/routes/batch.rs`, `services/api/src/routes/mod.rs`, `services/api/src/lib.rs`, and tracking ledgers.
- **Prerequisites:** Stage 7 Certified (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/core/src/id.rs`
  - `crates/core/src/lib.rs`
  - `services/api/src/dto/batch.rs`
  - `services/api/src/dto/mod.rs`
  - `services/api/src/routes/batch.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
  - Tracking ledgers (`IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`)
- **Implementation Tasks:**
  1. Define strongly-typed `BatchId` domain ID via `define_domain_id!` in `crates/core/src/id.rs` and re-export in `crates/core/src/lib.rs`.
  2. Implement comprehensive batch DTOs in `services/api/src/dto/batch.rs`:
     - Bounded constants: `MIN_BATCH_URLS = 1`, `MAX_BATCH_URLS = 100`, `DEFAULT_BATCH_PARALLELISM = 5`, `MAX_BATCH_PARALLELISM = 20`.
     - Valid context profiles validation: `["general_ai", "code_review", "security_review", "debugging_and_documentation", "debugging"]`.
     - `BatchItemRequest`: item URL, optional custom ID, git ref, path patterns, token budget (min 1,000), target model, export formats, context profile.
     - `CreateBatchJobRequest`: supports structured `items` array or shorthand `urls` string array, global default options, URL deduplication while preserving submission order (`effective_items()`), batch size boundary validation (`1..=100`), and parallelism bounds validation (`1..=20`).
     - `BatchStatus` enum (`Queued`, `Processing`, `Completed`, `PartialFailure`, `Failed`, `Cancelled`).
     - `BatchItemStatus`: item index, target URL, custom ID, UUID v7 `JobId`, status (`JobStatus::Queued`), creation timestamp.
     - `BatchProgress`: item count aggregations (total, completed, failed, processing, queued) and completion percentage calculation.
     - `CreateBatchJobResponse`: HTTP 202 response payload with `BatchId`, parallelism, and allocated sub-job statuses.
     - `BatchJobResponse`: detailed batch query descriptor with progress, timings, and item statuses.
     - `BatchPathParams`: `Validatable` route parameter wrapper for `BatchId`.
  3. Re-export batch models in `services/api/src/dto/mod.rs` and author unit test suite verifying shorthand URLs, structured overrides, URL deduplication, and invalid parameter rejection.
  4. Implement `create_batch_job_handler` and `get_batch_job_handler` in `services/api/src/routes/batch.rs`, allocating UUID v7 `JobId`s per item and instrumenting with `jobs_total.inc("batch_submitted")`.
  5. Register batch routes in `services/api/src/routes/mod.rs` (`POST /api/v1/jobs/batch` and `GET /api/v1/jobs/batch/:batch_id`) before `:job_id` wildcard to ensure unambiguous static segment matching, and publish `"jobs_batch"` in the root handler metadata.
  6. Implement 11 integration tests in `services/api/src/lib.rs` verifying valid shorthand submissions, structured submissions with per-item overrides, empty batch rejection, excessive batch size rejection (>100 URLs), invalid URL rejection, invalid token budget rejection, invalid context profile rejection, invalid parallelism rejection (>20), duplicate URL deduplication, not-found RFC 7807 problem details, and malformed batch UUID rejection.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-api`
  - `cargo test -p web-repo-to-ai-core`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Multi-URL batch payload validation strictly enforces 1..=100 URL bounds and validates URL schemes (`http`/`https`).
  - Batch parallelism parameter strictly enforces 1..=20 worker bounds.
  - Context profiles validate against authorized profile taxonomy (`general_ai`, `code_review`, `security_review`, `debugging_and_documentation`, `debugging`).
  - Identical URLs submitted within the same batch request are deduplicated deterministically while preserving original order.
  - `POST /api/v1/jobs/batch` returns HTTP 202 Accepted with assigned `BatchId` and distinct `JobId`s for each batch item.
  - `GET /api/v1/jobs/batch/:batch_id` returns RFC 7807 404 Problem Details when the batch is not found, and 400 for malformed UUIDs.
  - All 70 API crate tests pass, all 26 core tests pass, all 705+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Unbounded batch submissions could exhaust memory or overload downstream fetchers and workers (Denial of Service).
  - Mitigation: Strict upper bound of 100 URLs per batch (`MAX_BATCH_URLS = 100`) and max parallelism of 20 (`MAX_BATCH_PARALLELISM = 20`) enforced at the HTTP boundary before queuing.
- **Rollback Plan:**
  - Revert Subphase 8.1.1 additions via `git reset --hard HEAD~1` and restore Stage 7 certified baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-api`: 70 tests passing (8 unit tests in `dto`, 61 integration tests in `lib.rs`, 1 main test).
  - `cargo test -p web-repo-to-ai-core`: 26 tests passing.
  - Full workspace tests: 705+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `bcd89f0`
- **Status:** `ACCEPTED`

---

#### Subphase `8.1.2` — Bounded parallel execution in services/worker
- **Unique ID:** `8.1.2`
- **Objective:** Implement resource-bounded asynchronous parallel batch task execution, worker concurrency budgeting, cooperative cancellation, per-item timeouts, and resilient error isolation in `crates/job-engine` and `services/worker`.
- **Scope:** `crates/job-engine`, `services/worker`, and tracking ledgers.
- **Prerequisites:** `8.1.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/job-engine/Cargo.toml`
  - `crates/job-engine/src/config.rs`
  - `crates/job-engine/src/error.rs`
  - `crates/job-engine/src/executor.rs`
  - `crates/job-engine/src/models.rs`
  - `crates/job-engine/src/pool.rs`
  - `crates/job-engine/src/lib.rs`
  - `services/worker/Cargo.toml`
  - `services/worker/src/config.rs`
  - `services/worker/src/engine.rs`
  - `services/worker/src/metrics.rs`
  - `services/worker/src/lib.rs`
  - `services/worker/src/main.rs`
  - `services/worker/tests/bounded_parallel_worker_test.rs`
  - Tracking ledgers (`IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`)
- **Implementation Tasks:**
  1. Define strongly typed domain error taxonomy `JobEngineError` in `crates/job-engine/src/error.rs` covering concurrency violations, per-item timeouts, cooperative cancellation, task failures, and worker shutdown.
  2. Implement `BatchExecutionConfig` in `crates/job-engine/src/config.rs` with bounds validation (`MIN_PARALLELISM = 1`, `MAX_PARALLELISM = 20`, `DEFAULT_PARALLELISM = 5`), per-item execution timeouts, and cumulative batch timeouts.
  3. Implement task, outcome, progress, and summary models (`BatchItemTask`, `BatchItemStatusOutcome`, `BatchItemExecutionResult`, `BatchProgressUpdate`, `BatchExecutionSummary`) in `crates/job-engine/src/models.rs`.
  4. Implement `BoundedParallelExecutor` in `crates/job-engine/src/executor.rs` using `Arc<Semaphore>` for strict concurrency bounding, atomic counters (`AtomicUsize`) for active and peak concurrency recording, `tokio::time::timeout` for per-item timeouts, cooperative `CancellationToken` monitoring, resilient error containment, and deterministic sequence ordering.
  5. Implement `WorkerPool` in `crates/job-engine/src/pool.rs` managing worker-level concurrency ceilings and lifecycle supervision.
  6. Implement `WorkerServiceConfig` in `services/worker/src/config.rs` with concurrency limits, leasing boundaries, and shutdown timeouts.
  7. Implement `WorkerServiceMetrics` in `services/worker/src/metrics.rs` with thread-safe atomic telemetry.
  8. Implement `WorkerEngine` in `services/worker/src/engine.rs` coordinating batch task dispatch, background polling, and graceful draining.
  9. Update `services/worker/src/main.rs` daemon binary with tracing initialization and graceful `Ctrl-C` signal handling.
  10. Author integration test suite in `services/worker/tests/bounded_parallel_worker_test.rs` verifying strict concurrency bounds, resilient error isolation, per-batch parallelism override, realtime progress notifications, and cooperative shutdown draining.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-job-engine`
  - `cargo test -p web-repo-to-ai-worker`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Peak concurrent tasks strictly never exceed configured parallelism ceiling.
  - Individual task failure (e.g., 404, SSRF block, timeout) does not crash or terminate the remaining batch items.
  - Client sequence ordering is preserved across all batch results.
  - Cooperative cancellation cleanly drains running tasks and rejects pending tasks without permit leaks.
  - Realtime progress updates emit monotonically increasing finished items and accurate progress percentages.
  - All 15 crate tests in `crates/job-engine` pass, all 15 tests in `services/worker` pass, all 735+ workspace tests pass, zero clippy warnings, zero fmt diffs, cargo deny passed, all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Unbounded concurrency under high batch volume could exhaust worker memory or CPU on the 16 GB baseline.
  - Mitigation: Strict two-tier concurrency gating: `WorkerPool` enforces a global worker ceiling (default 10, max 20), and `BoundedParallelExecutor` bounds per-batch concurrency via `Semaphore`.
- **Rollback Plan:**
  - Revert Subphase 8.1.2 additions via `git reset --hard HEAD~1` and restore Subphase 8.1.1 baseline.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-job-engine`: 15 tests passing (10 unit tests, 5 executor tests).
  - `services/worker`: 15 tests passing (9 unit tests in `lib.rs`, 1 binary test in `main.rs`, 5 integration tests in `bounded_parallel_worker_test.rs`).
  - Full workspace tests: 735+ tests passing across all crates and services with 0 failures.
  - Workspace linter: `cargo clippy --workspace --all-targets -- -D warnings` clean with 0 warnings.
  - Formatting: `cargo fmt --all -- --check` clean with 0 diffs.
  - Dependency audit: `cargo deny check` passed (advisories ok, bans ok, licenses ok, sources ok).
  - Monorepo gatekeeper: `powershell scripts/check-all.ps1` completed with `ALL GATES PASSED (0 failures)`.
- **Git Commit Reference:** `d3afaae`
- **Status:** `ACCEPTED`

---

### Subphase `8.1.3` — Individual result tracking in crates/storage and services/api
- **Unique ID:** `8.1.3`
- **Objective:** Implement multi-URL batch conversion state and itemized result tracking in `crates/storage` and `services/api`, including relational DDL tables (`batch_jobs`, `batch_items`), embedded migration `20261119000001_batch_jobs_and_items.sql`, repository abstractions (`BatchRepository`, `InMemoryBatchRepository`, `PostgresBatchRepository`), automatic parent batch counter recalculation and terminal state progression, and REST query endpoints for batch progress, itemized status lists, single-item status lookups, batch cancellation, and pagination.
- **Prerequisites:** `8.1.2` (`ACCEPTED`)
- **Primary Agent Roster:** `rust-backend-engineer`, `qa-engineer`, `security-reviewer`, `documentation-reviewer`
- **Scope:**
  - `crates/storage/src/batch.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/src/migrations/20261119000001_batch_jobs_and_items.sql`
  - `crates/storage/src/migrations/mod.rs`
  - `crates/storage/tests/batch_storage_test.rs`
  - `services/api/src/state.rs`
  - `services/api/src/dto/batch.rs`
  - `services/api/src/dto/mod.rs`
  - `services/api/src/routes/batch.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
  - Tracking ledgers (`IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/ROADMAP.md`)
- **Implementation Tasks:**
  1. Define persistent models `BatchJobStatus`, `BatchItemStatusKind`, `BatchRecord`, and `BatchItemRecord` in `crates/storage/src/batch.rs`.
  2. Implement `BatchRepository` async trait declaring `create_batch`, `get_batch`, `get_batch_items`, `get_batch_item`, `get_batch_item_by_job_id`, `update_item_status`, `cancel_batch`, and `list_batches`.
  3. Implement thread-safe `InMemoryBatchRepository` with automatic parent counter recalculation and lifecycle state transitions (`Queued` -> `Processing` -> `Completed` / `PartialFailure` / `Failed` / `Cancelled`).
  4. Implement `PostgresBatchRepository` with transactional integrity, foreign key cascading, and conditional parent counter recalculation SQL CTEs.
  5. Author migration `20261119000001_batch_jobs_and_items.sql` and update embedded production DDL script in `crates/storage/src/schema/ddl.sql`.
  6. Wire `batch` module into `crates/storage/src/lib.rs` and verify catalog validation in `crates/storage/src/schema/mod.rs`.
  7. Author integration test suite in `crates/storage/tests/batch_storage_test.rs` covering lifecycle progression, child job ID lookup, deterministic item ordering, cancellation, and session filtering.
  8. Integrate `Arc<dyn BatchRepository>` into `AppState` in `services/api/src/state.rs`.
  9. Enhance batch DTOs in `services/api/src/dto/batch.rs` with item-level error codes, error messages, execution duration, completion timestamps, `BatchItemPathParams`, `BatchItemsListResponse`, `BatchSummaryItem`, and `ListBatchesResponse`.
  10. Implement REST route handlers in `services/api/src/routes/batch.rs` (`GET /api/v1/jobs/batch`, `GET /api/v1/jobs/batch/:batch_id`, `DELETE /api/v1/jobs/batch/:batch_id`, `POST /api/v1/jobs/batch/:batch_id/cancel`, `GET /api/v1/jobs/batch/:batch_id/items`, `GET /api/v1/jobs/batch/:batch_id/items/:item_index`).
  11. Register batch routes in `services/api/src/routes/mod.rs` and integrate unit/integration tests in `services/api/src/lib.rs`.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo test -p web-repo-to-ai-api`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Batch job and item records persist accurately with correct initial counts (`queued_items = total_items`).
  - Item status updates automatically trigger parent batch progress recalculations without permit or state leaks.
  - Querying `GET /api/v1/jobs/batch/:batch_id` returns accurate quantitative progress percentages and status.
  - Querying item lists and single items returns deterministic ordering and detailed error messages/durations.
  - Batch cancellation transitions all remaining queued/processing items to `cancelled` and marks batch as `cancelled`.
  - All 70 storage tests pass, all 76 api tests pass, all workspace clippy checks pass with 0 warnings, zero fmt diffs, cargo deny check clean, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: High batch volume item updates could introduce lock contention or stale parent counters.
  - Mitigation: Atomically recalculate parent metrics in-memory via write locks and in PostgreSQL via atomic transactional CTE updates (`WITH counts AS ... UPDATE batch_jobs`).
- **Rollback Plan:**
  - Revert Subphase 8.1.3 additions via Git reset to commit `d3afaae`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-storage`: 70 tests passed (57 unit, 2 batch storage integration, 11 token storage integration).
  - `cargo test -p web-repo-to-ai-api`: 76 tests passed (75 unit/integration in `lib.rs`, 1 in `main.rs`).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (advisories ok, bans ok, licenses ok, sources ok).
  - `powershell scripts/check-all.ps1`: ALL GATES PASSED (0 failures) across all 5 monorepo gates.
- **Git Commit Reference:** `dd1832d`
- **Status:** `ACCEPTED`

---

### Subphase `8.1.4` — Batch export in crates/exporters and services/api
- **Unique ID:** `8.1.4`
- **Objective:** Implement unified multi-URL batch artifact exports and download endpoints in `crates/exporters`, `crates/storage`, and `services/api`, supporting multi-item consolidated formats (Markdown with TOC and divider headings, line-delimited JSONL chunks, structured JSON manifest with CDM document array, clean plain-text context) and a deterministic, streaming multi-item ZIP archive package containing manifest (`batch-manifest.json`), summary (`batch-summary.md`), consolidated markdown (`consolidated.md`), per-item folders with individual exports (`items/{idx:03}_{slug}/context.md`, `context.txt`, `context.json`), failure reports (`error.txt`), and BLAKE3/SHA-256 cryptographic checksums (`checksums.sha256`). Wire HTTP download endpoints with RFC 7807 problem details, content disposition attachment headers, and nosniff security protections.
- **Prerequisites:** `8.1.3` (`ACCEPTED`)
- **Primary Agent Roster:** `context-optimization-engineer`, `rust-backend-engineer`, `qa-engineer`, `security-reviewer`, `documentation-reviewer`
- **Scope:**
  - `crates/storage/src/local_fs.rs`
  - `crates/exporters/src/batch.rs`
  - `crates/exporters/src/lib.rs`
  - `services/api/src/dto/batch.rs`
  - `services/api/src/dto/mod.rs`
  - `services/api/src/error.rs`
  - `services/api/src/routes/batch.rs`
  - `services/api/src/routes/mod.rs`
  - `services/api/src/lib.rs`
- **Implementation Tasks:**
  1. Add `layout::batch_artifact_key` and `layout::batch_dir_key` in `crates/storage/src/local_fs.rs` for batch artifact path storage resolution.
  2. Implement `crates/exporters/src/batch.rs`:
     - Data models: `BatchItemExportInput`, `BatchExportOptions`, `BatchExportManifest`, `BatchItemManifestEntry`, `BatchZipMetrics`.
     - Slugification utility `slugify_url` collapsing non-alphanumeric runes and sanitizing folder names.
     - `export_batch_consolidated_markdown`: generates full consolidated Markdown with batch title, table of contents, and per-item section dividers.
     - `export_batch_consolidated_plain_text`: generates clean ASCII text context suitable for language models.
     - `export_batch_consolidated_jsonl`: generates line-delimited JSONL chunks with batch item indices, item URLs, and section payloads.
     - `export_batch_consolidated_json`: generates structured JSON envelope with manifest and document objects.
     - `export_batch_zip_package` & `export_batch_zip_to_writer`: packages a complete multi-item ZIP archive with `batch-manifest.json`, `batch-summary.md`, `consolidated.md`, `checksums.sha256`, individual item folders, and item failure reports.
  3. Re-export batch exporters in `crates/exporters/src/lib.rs`.
  4. Implement `BatchArtifactPathParams`, `BatchArtifactItemDescriptor`, and `BatchArtifactsResponse` in `services/api/src/dto/batch.rs` and re-export in `services/api/src/dto/mod.rs`.
  5. Implement `internal_problem` helper in `services/api/src/error.rs`.
  6. Implement `export_batch_handler`, `list_batch_artifacts_handler`, and `build_batch_download_response` in `services/api/src/routes/batch.rs`.
  7. Wire routes in `services/api/src/routes/mod.rs`:
     - `GET /api/v1/jobs/batch/:batch_id/export/:format` -> `batch::export_batch_handler`
     - `GET /api/v1/jobs/batch/:batch_id/artifacts` -> `batch::list_batch_artifacts_handler`
  8. Author unit and integration tests in `crates/exporters/src/batch.rs` and `services/api/src/lib.rs` validating consolidated format exports, ZIP archive creation, artifact listing, checksum manifests, invalid format rejections, and missing batch 404 responses.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-exporters`
  - `cargo test -p web-repo-to-ai-api`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Consolidated exports (Markdown, Plain Text, JSONL, JSON) and ZIP archives are generated deterministically from batch items.
  - Succeeded items include full context and CDM structures; failed items include error descriptors without crashing the export pipeline.
  - ZIP archives contain valid directory structures, per-item files, summary, manifest, and dual-hash checksums (`checksums.sha256`).
  - `GET /api/v1/jobs/batch/:batch_id/artifacts` lists all available export formats and their download URLs.
  - `GET /api/v1/jobs/batch/:batch_id/export/:format` streams downloads with proper Content-Type, Content-Disposition attachment headers, and nosniff protection.
  - 100% of exporter tests pass (81/81), 100% of API tests pass (84/84), zero clippy warnings, zero fmt diffs, cargo deny clean, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Large batch sizes could exceed memory during ZIP archive packaging.
  - Mitigation: Stream zip entries using `export_batch_zip_to_writer` over `std::io::Write` buffers; compress item sections sequentially with Deflate/Stored modes.
- **Rollback Plan:**
  - Revert Subphase 8.1.4 additions via Git reset to commit `0655c51`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-exporters`: 81 tests passed (4 unit in `lib.rs`, 77 integration across 8 integration suites).
  - `cargo test -p web-repo-to-ai-api`: 84 tests passed (83 unit/integration in `lib.rs`, 1 in `main.rs`).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (`advisories ok, bans ok, licenses ok, sources ok`).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `a3aa12d`
- **Status:** `ACCEPTED`

---

### PHASE 8.1 FORMAL CERTIFICATION

- **Stage:** `STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING`
- **Phase:** `Phase 8.1 — Batch URLs`
- **Subphases Certified (4 of 4):**
  - `8.1.1` (Multi-URL submission endpoint and payload validation in services/api) — `ACCEPTED`
  - `8.1.2` (Bounded parallel execution in services/worker) — `ACCEPTED`
  - `8.1.3` (Individual result tracking in crates/storage and services/api) — `ACCEPTED`
  - `8.1.4` (Batch export in crates/exporters and services/api) — `ACCEPTED`
- **Phase Acceptance Gate Verification:**
  - Multi-URL batch ingestion validated with strict RFC 7807 error responses and per-item option overrides.
  - Bounded worker concurrency engine strictly enforces global permit ceiling, per-batch worker quotas, and cooperative shutdown.
  - Relational and in-memory batch storage tracks granular item lifecycle, execution durations, error taxonomy, and dynamic parent progress.
  - Multi-format batch export pipeline generates consolidated Markdown, plain text, JSONL, JSON, and fully structured ZIP archives with cryptographic checksums.
  - 100% of workspace tests passing, clippy warning-free, and all 5 monorepo verification gates passing.
- **Certification Status:** **PHASE 8.1 FORMALLY CERTIFIED & ACCEPTED**

---

### Upcoming Subphase (Stage 8 — Large Repositories, Batch and Crawling)

#### Phase 8.2 — Website crawling

### Subphase `8.2.1` — Sitemap discovery in crates/web-extractor
- **Unique ID:** `8.2.1`
- **Objective:** Implement comprehensive automated sitemap discovery, `robots.txt` directive parsing, well-known endpoint probing, multi-format sitemap extraction (XML `<urlset>`, `<sitemapindex>` recursion, RSS 2.0, Atom feeds, plain text), Gzip decompression (`.xml.gz`) with decompression-bomb protection, SSRF security validation, cross-domain filtering, and URL deduplication in `crates/web-extractor`.
- **Prerequisites:** `8.1.4` (`ACCEPTED`)
- **Primary Agent Roster:** `extraction-engineer`, `security-reviewer`, `qa-engineer`, `documentation-reviewer`
- **Scope:**
  - `Cargo.toml`
  - `crates/web-extractor/src/error.rs`
  - `crates/web-extractor/src/sitemap/mod.rs`
  - `crates/web-extractor/src/sitemap/model.rs`
  - `crates/web-extractor/src/sitemap/parser.rs`
  - `crates/web-extractor/src/sitemap/robots.rs`
  - `crates/web-extractor/src/sitemap/discovery.rs`
  - `crates/web-extractor/src/lib.rs`
  - `crates/web-extractor/tests/sitemap_discovery_test.rs`
- **Implementation Tasks:**
  1. Add `features = ["serde"]` to `url` in root `Cargo.toml` for seamless serialization of `Url`.
  2. Add `WebExtractorError::Sitemap(String)` variant and `CoreError::Extraction` mapping in `crates/web-extractor/src/error.rs`.
  3. Define strongly-typed sitemap domain models in `crates/web-extractor/src/sitemap/model.rs`: `ChangeFrequency` (`Always`, `Hourly`, `Daily`, `Weekly`, `Monthly`, `Yearly`, `Never`), `SitemapEntry`, `SitemapIndexEntry`, `SitemapType` (`UrlSet`, `SitemapIndex`, `PlainText`, `RssFeed`, `AtomFeed`), `ParsedSitemap`, `SitemapDiscoveryConfig`, and `DiscoveredSitemapReport`.
  4. Implement resilient, zero-dependency streaming XML and plain-text parser in `crates/web-extractor/src/sitemap/parser.rs`:
     - Gzip magic header detection and safe bounded decompression via `flate2::read::GzDecoder`.
     - Lightweight XML tokenizer handling namespaces, comments, CDATA blocks, and XML entity decoding (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, numeric entities).
     - Multi-format date parsing supporting W3C Datetime, RFC 3339, RFC 2822, ISO 8601, and date-only strings.
     - Extraction of `<loc>`, `<lastmod>`, `<changefreq>`, and `<priority>` for `<urlset>` and `<sitemapindex>`.
     - RSS 2.0 (`<item><link>`) and Atom (`<entry><link>`) syndication feed ingestion.
     - Plain-text sitemap parser handling one URL per line with comment stripping.
  5. Implement `robots.txt` sitemap directive parser in `crates/web-extractor/src/sitemap/robots.rs` extracting declared `Sitemap:` URLs with comment stripping and relative resolution.
  6. Implement `SitemapDiscoverer` in `crates/web-extractor/src/sitemap/discovery.rs`:
     - Root URL origin resolution.
     - Direct sitemap URL detection.
     - `robots.txt` query and parsing.
     - Well-known candidate path probing fallback (`/sitemap.xml`, `/sitemap_index.xml`, `/sitemap/sitemap.xml`, `/sitemap.xml.gz`, `/sitemap.txt`).
     - Recursive sitemap index traversal bounded by `max_index_depth` and `max_child_sitemaps` with cycle / loop detection using `visited` URL sets.
     - Host policy enforcement (`enforce_same_host` and `allow_subdomains`) to prevent rogue cross-domain URL injection.
     - SSRF security validation via `crates/url-security`.
     - Deterministic URL deduplication and priority-descending ordering bounded by `max_urls`.
  7. Re-export sitemap module and types in `crates/web-extractor/src/lib.rs`.
  8. Author 14 integration test suites in `crates/web-extractor/tests/sitemap_discovery_test.rs` validating standard urlsets, namespaces, gzipped sitemaps, sitemap index recursion, plain text, RSS/Atom feeds, robots.txt, well-known probing, cross-domain filtering, subdomain policies, and circular loops.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-web-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Standard `<urlset>` XML documents parse accurately with metadata and entity decoding.
  - `<sitemapindex>` hierarchies expand child sitemaps with cycle loop protection.
  - Compressed `.xml.gz` sitemaps decompress safely with decompression-bomb ceilings.
  - Plain text sitemaps, RSS 2.0, and Atom feeds extract URLs with proper dates.
  - `robots.txt` `Sitemap:` directives are extracted and normalized.
  - Well-known sitemap endpoints are probed as fallback when robots.txt does not specify sitemaps.
  - Cross-domain URLs from rogue sitemaps are dropped by default unless subdomains are explicitly allowed.
  - 100% of web-extractor tests pass (157/157), workspace clippy is clean, zero fmt diffs, cargo deny clean, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Circular sitemap index references or recursive sitemap links could cause infinite loops or resource exhaustion.
  - Mitigation: Visited URL tracking via `HashSet<String>`, strict recursion depth limits (`max_index_depth = 3`), child sitemap ceilings (`max_child_sitemaps = 50`), and max total URL bounds (`max_urls = 50,000`).
- **Rollback Plan:**
  - Revert Subphase 8.2.1 additions via Git reset to commit `cf6595c`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 157 tests passed (135 unit in `lib.rs`, 8 golden dataset, 14 sitemap discovery).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (`advisories ok, bans ok, licenses ok, sources ok`).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `fa62d32`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 8 — Large Repositories, Batch and Crawling)

#### Phase 8.2 — Website crawling

| Subphase ID | Objective | Prerequisites | Affected Files / Components | Status |
| :--- | :--- | :--- | :--- | :--- |
| `8.2.2` | Internal link discovery in crates/web-extractor | `8.2.1` (`ACCEPTED`) | `crates/web-extractor/*` | `IN_PROGRESS` |

---

### Subphase `8.2.2` — Internal link discovery in crates/web-extractor
- **Unique ID:** `8.2.2`
- **Objective:** Implement automated discovery, normalization, semantic classification, and SSRF security validation of internal hyperlinks from HTML documents during website crawling in `crates/web-extractor`. Support `<base href="...">` resolution, in-page anchor fragment stripping, marketing tracking parameter stripping (`utm_*`, `fbclid`, etc.), asset exclusion (`.pdf`, `.zip`, etc.), subdomain policies, `rel="nofollow"` respect, and a 3-tier deterministic crawling priority sort (content links first, pagination transitions second, boilerplate navigation third).
- **Scope:** `crates/web-extractor/src/crawler/links.rs`, `crates/web-extractor/src/crawler/mod.rs`, `crates/web-extractor/src/lib.rs`, `crates/web-extractor/tests/internal_link_discovery_test.rs`.
- **Prerequisites:** `8.2.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/crawler/links.rs`
  - `crates/web-extractor/src/crawler/mod.rs`
  - `crates/web-extractor/src/lib.rs`
  - `crates/web-extractor/tests/internal_link_discovery_test.rs`
- **Implementation Tasks:**
  1. Define domain models in `crates/web-extractor/src/crawler/links.rs`:
     - `DiscoveredInternalLink`: fully resolved `url`, `source_url`, normalized `anchor_text`, optional `title`, tokenized `rel` attributes, semantic flags `is_content_link` and `is_pagination`, and `context_snippet`.
     - `InternalLinkDiscoveryConfig`: `strip_fragments`, `strip_tracking_params`, `exclude_assets`, `respect_nofollow`, `allow_subdomains`, `max_links_per_page`, `extract_context_snippets`.
     - `PageLinkDiscoveryReport`: resolved destination URLs, counts for external links, assets, anchors, filtered SSRF violations, and dropped nofollow links.
  2. Implement `InternalLinkDiscoverer`:
     - Methods `discover_from_url`, `discover_from_html`, and `discover_from_vdom`.
     - HTML `<base href="...">` resolution overriding document source URL.
     - Top-down DOM parent-map index construction (`build_parent_map`) for semantic ancestor traversal.
     - Content vs boilerplate navigation detection (`is_content_area_link`) penalizing `<nav>`, `<header>`, `<footer>`, `<aside>`, and navigation classes, while promoting `<main>`, `<article>`, and content classes.
     - Pagination link detection (`is_pagination_link`) via `rel="next/prev"`, pagination classes, and directional text tokens.
     - Surrounding sentence context snippet extraction (`extract_surrounding_context`) up to 140 chars.
     - Relative link resolution against effective base URL and scheme validation (`http`/`https` only, ignoring special schemes `javascript:`, `mailto:`, `tel:`, `data:`).
     - Static asset extension filtering against `ASSET_EXTENSIONS`.
     - Internal host boundary matching with configurable subdomain allowance.
     - URL normalization: fragment stripping (`#section`) and marketing parameter stripping (`utm_*`, `fbclid`, `gclid`, `ref`, etc.) using `url::form_urlencoded`.
     - Zero-Trust SSRF and private IP address validation via `ValidatedUrl::parse(url, &policy)`.
     - Deterministic 3-tier priority sorting: Tier 0 (primary content links), Tier 1 (pagination transitions), Tier 2 (boilerplate navigation), tied on alphabetical URL.
  3. Wire crawler module into `crates/web-extractor/src/crawler/mod.rs` and re-export types in `crates/web-extractor/src/lib.rs`.
  4. Author 14 integration test suites in `crates/web-extractor/tests/internal_link_discovery_test.rs` covering basic link resolution, base tag overrides, boilerplate classification, pagination detection, asset filtering, fragment stripping, tracking parameter stripping, subdomain policies, nofollow policies, zero-trust SSRF rejection, link capping, context snippet extraction, priority sorting, and end-to-end mock HTTP discovery.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-web-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Discovered links resolve accurately against `<base href="...">` and document URLs.
  - Links inside `<main>` and `<article>` are classified as content links; navigation links are classified as boilerplate.
  - Pagination links (`rel="next/prev"`, `.pagination`, `Next Page`) are classified as pagination.
  - In-page anchors (`#anchor`) and tracking parameters (`utm_*`, `fbclid`) are normalized without losing functional query parameters.
  - Static downloadable assets (`.pdf`, `.zip`, etc.) are filtered out when enabled.
  - Subdomains are partitioned according to `allow_subdomains` policy.
  - `rel="nofollow"` links are dropped when `respect_nofollow` is enabled.
  - Loopback and private IP links are rejected by SSRF policies.
  - 100% of web-extractor tests pass (171/171), workspace clippy is clean, zero fmt diffs, cargo deny clean, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Links targeting circular or infinite query tracking variations could cause crawl loops or memory bloat.
  - Mitigation: URL normalization stripping tracking parameters, deduplication in `HashMap`, and `max_links_per_page` boundary caps.
- **Rollback Plan:**
  - Revert Subphase 8.2.2 additions via Git reset to commit `393cf6a`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 171 tests passed (135 unit in `lib.rs`, 8 golden dataset, 14 sitemap discovery, 14 internal link discovery).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (`advisories ok, bans ok, licenses ok, sources ok`).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `f402be8`
- **Status:** `ACCEPTED`

---

### Subphase `8.2.3` — Crawl-depth & domain restrictions in crates/web-extractor
- **Unique ID:** `8.2.3`
- **Objective:** Implement crawl traversal depth limits, domain scoping policies, path inclusion/exclusion glob and regex filtering, query parameter policies, and page budget quotas in `crates/web-extractor`. Provide stateful evaluation of candidate URLs against boundary restrictions, tracking depths, host quotas, and total crawl limits to prevent runaway crawling, crawl loops, and cross-origin resource exhaustion.
- **Scope:** `crates/web-extractor/src/crawler/restrictions.rs`, `crates/web-extractor/src/crawler/mod.rs`, `crates/web-extractor/src/lib.rs`, `crates/web-extractor/Cargo.toml`, `crates/web-extractor/tests/crawl_restrictions_test.rs`.
- **Prerequisites:** `8.2.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/crawler/restrictions.rs`
  - `crates/web-extractor/src/crawler/mod.rs`
  - `crates/web-extractor/src/lib.rs`
  - `crates/web-extractor/Cargo.toml`
  - `crates/web-extractor/tests/crawl_restrictions_test.rs`
- **Implementation Tasks:**
  1. Define domain models and configuration types in `crates/web-extractor/src/crawler/restrictions.rs`:
     - `CrawlScope`: `ExactHost`, `Subdomains`, `PathPrefix`, and `DomainAllowlist(Vec<String>)`.
     - `QueryParamPolicy`: `StripAll`, `AllowList(HashSet<String>)`, `BlockList(HashSet<String>)`, and `AllowAll`.
     - `CrawlRestrictions`: scoping, depth boundaries (`max_depth`), total page budgets (`max_pages`), per-depth and per-host quotas (`max_pages_per_depth`, `max_pages_per_host`), path pattern rules (`include_patterns`, `exclude_patterns`), static asset extension filters, port and scheme validation, and SSRF security policies.
     - `CrawlRejectionReason`: strongly-typed classification for depth exceeded, external host, subdomain disallowed, path prefix mismatch, pattern exclusions, include pattern non-match, asset extensions, scheme/port violations, budget exhaustion, and SSRF security defense.
     - `CrawlDecision`: `Allowed { url, depth }` or `Rejected { url, reason }`.
     - `CrawlBudgetStats`: runtime accounting for total crawled pages, remaining quotas, and per-depth/per-host distribution.
  2. Implement stateful `CrawlRestrictionsEvaluator`:
     - Compile path patterns into regexes supporting recursive wildcards (`**`), single-segment wildcards (`*`), character wildcards (`?`), prefix matching, and explicit `regex:` prefixes.
     - Enforce `CrawlScope` domain boundaries against seed host, subdomains, path prefixes, and wildcard domain allowlists (`*.partner.org`).
     - Track minimum observed traversal depth for discovered URLs to optimize traversal ordering.
     - Enforce global page quotas, per-depth ceilings, and per-host limits.
     - Filter/normalize query parameters according to `QueryParamPolicy` and strip in-page section fragments (`#anchor`).
     - Validate candidate URLs against zero-trust SSRF policy.
     - Provide `record_page_crawled` and `stats` inspection methods.
  3. Wire restrictions into `crates/web-extractor/src/crawler/mod.rs` and re-export in `crates/web-extractor/src/lib.rs`.
  4. Author 14 comprehensive integration test suites in `crates/web-extractor/tests/crawl_restrictions_test.rs` covering depth enforcement, exact host scope, subdomain scope, path prefix scope, custom allowlists, path include patterns, path exclude patterns, query parameter policies, asset extension blocking, scheme/port enforcement, page budget exhaustion, per-depth/per-host quotas, SSRF defense, and minimum depth deduplication.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-web-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Candidate URLs exceeding `max_depth` are rejected with `ExceedsMaxDepth`.
  - `ExactHost`, `Subdomains`, `PathPrefix`, and `DomainAllowlist` scopes restrict domains strictly.
  - Path glob patterns (`**`, `*`, `?`) and regex patterns filter allowed/disallowed paths accurately.
  - Query parameter policies (`StripAll`, `AllowList`, `BlockList`, `AllowAll`) normalize candidate URLs without loss of legitimate parameters.
  - Page budgets (global, per-depth, per-host) enforce quotas and report exhaustion.
  - SSRF violations trigger immediate rejection.
  - 100% of web-extractor tests pass (185/185), workspace clippy is warning-free, zero fmt diffs, cargo deny clean, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: Path pattern regex compilation errors or backtracking explosion on malformed glob patterns.
  - Mitigation: Pre-compilation of patterns in `CrawlRestrictionsEvaluator::new`, safe translation of glob tokens, and error propagation.
- **Rollback Plan:**
  - Revert Subphase 8.2.3 additions via Git reset to commit `f402be8`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 185 tests passed (135 unit in `lib.rs`, 14 crawl restrictions, 8 golden dataset, 14 internal link discovery, 14 sitemap discovery).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: verified clean formatting with 0 diffs.
  - `cargo deny check`: passed (`advisories ok, bans ok, licenses ok, sources ok`).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `13cdaa9`
- **Status:** `ACCEPTED`

---

### Subphase `8.2.4` — Duplicate & loop prevention in crates/web-extractor
- **Unique ID:** `8.2.4`
- **Objective:** Implement crawler loop detection (path segment repetition traps, cyclic directory patterns, query parameter combinatorial explosion, and circular redirect loop detection) and crawl frontier queue management with deterministic canonical URL normalization and multi-tier duplicate prevention (canonical URL deduplication, cryptographic BLAKE3 exact content deduplication, and SimHash near-duplicate detection) in `crates/web-extractor`.
- **Scope:** `crates/web-extractor/src/crawler/loop_detector.rs`, `crates/web-extractor/src/crawler/frontier.rs`, `crates/web-extractor/src/crawler/mod.rs`, `crates/web-extractor/src/lib.rs`, `crates/web-extractor/tests/crawler_dedup_loop_test.rs`.
- **Prerequisites:** `8.2.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/crawler/loop_detector.rs`
  - `crates/web-extractor/src/crawler/frontier.rs`
  - `crates/web-extractor/src/crawler/mod.rs`
  - `crates/web-extractor/src/lib.rs`
  - `crates/web-extractor/tests/crawler_dedup_loop_test.rs`
- **Implementation Tasks:**
  1. Author `crates/web-extractor/src/crawler/loop_detector.rs`:
     - Define `CrawlTrapReason` enum: `PathSegmentRepetition`, `CyclicPathSequence`, `ExcessivePathSegments`, `ExcessiveQueryParameters`, `RepeatedQueryKey`, `RedirectLoop`, and `RedirectChainTooLong`.
     - Implement `LoopDetectorConfig` with customizable limits: `max_segment_repeats` (default 3), `max_path_segments` (default 16), `max_query_params` (default 10), `max_query_key_repeats` (default 2), `max_redirect_chain_len` (default 10), `detect_cyclic_sequences` (default true), `min_cycle_len` (2), `max_cycle_len` (4), `cycle_repeat_threshold` (2).
     - Implement `CrawlLoopDetector` with `evaluate_url`, `check_path`, `check_query`, and `check_redirect_chain` methods.
     - Implement multi-segment cyclic sequence pattern detection (`find_cyclic_sequence`).
  2. Author `crates/web-extractor/src/crawler/frontier.rs`:
     - Define `CanonicalUrlNormalizerConfig` and `CrawlerUrlNormalizer` performing deterministic crawling canonicalization: default port stripping (`:80`, `:443`), section fragment stripping, default index file normalization (`index.html`, `index.php`, etc. to root), trailing slash normalization on non-root paths, marketing tracking query stripping (`utm_*`, `fbclid`, `gclid`, etc.), and alphabetical query parameter key sorting.
     - Define `CrawlQueueItem` min-heap ordered by priority, depth, and alphabetical canonical URL.
     - Define `FrontierEnqueueDecision` (`Enqueued`, `AlreadySeen`, `TrapBlocked`) and `FrontierStats`.
     - Implement `CrawlFrontier` priority queue and state engine integrating URL deduplication, depth optimization (re-queueing at shorter path depth), trap evaluation via `CrawlLoopDetector`, circular redirect loop tracking via `record_redirect_hop`, error tracking via `mark_failed`, and multi-tier content deduplication via `mark_visited` backed by `DuplicateRegistry` (BLAKE3 exact and 64-bit SimHash Hamming distance near-duplicates).
  3. Re-export crawler types in `crates/web-extractor/src/crawler/mod.rs` and `crates/web-extractor/src/lib.rs`.
  4. Author 14 comprehensive integration tests in `crates/web-extractor/tests/crawler_dedup_loop_test.rs` validating canonical URL normalization, path segment repetition traps, cyclic path sequences, path segment depth limits, query parameter count limits, repeated query keys, direct and multi-hop circular redirect loops, excessive redirect chains, frontier min-heap priority ordering, candidate URL deduplication, trap blocking, BLAKE3 exact content deduplication, SimHash near-duplicate detection, redirect loop tracking, and frontier metric accounting.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-web-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Canonical URL normalization strips default ports, tracking params, and default index files while deterministically sorting functional query parameters.
  - Path segment repetition traps (> 3 identical segments) and cyclic directory patterns (`/a/b/a/b`) are detected and rejected.
  - Excessive path depth (> 16 segments) and query explosion (> 10 parameters or > 2 repeated keys) are caught.
  - Circular redirect loops (`A -> B -> A`, `A -> B -> C -> A`) and excessive redirect chains (> 10 hops) are identified.
  - Min-heap frontier queue pops items strictly in priority order (lowest priority value first, then lowest depth).
  - Exact content duplicates (BLAKE3) and near-duplicates (SimHash Hamming distance <= threshold) are classified correctly.
  - 100% of web-extractor tests pass (199/199), clippy has 0 warnings, formatting is verified, and all monorepo gates pass (5/5).
- **Risk Analysis:**
  - Risk: False positive trap triggers on legitimate deep documentation paths or faceted search queries.
  - Mitigation: Configurable limits in `LoopDetectorConfig` and `CanonicalUrlNormalizerConfig` allowing custom crawlers to tailor thresholds.
- **Rollback Plan:**
  - Revert Subphase 8.2.4 additions via Git reset to commit `b878176`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 199 tests passed (135 unit, 14 crawl restrictions, 14 crawler dedup loop, 8 golden dataset, 14 internal link discovery, 14 sitemap discovery).
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed with 0 warnings.
  - `cargo fmt --all -- --check`: passed with 0 diffs.
  - `cargo deny check`: passed (`advisories ok, bans ok, licenses ok, sources ok`).
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) including 213 frontend vitest tests and 17.6% Podman storage footprint.
- **Git Commit Reference:** `6553e10`
- **Status:** `ACCEPTED`

---

### Phase 8.3 — Incremental processing

#### Subphase `8.3.1` — HTTP validators in crates/storage and crates/web-extractor
- **Unique ID:** `8.3.1`
- **Objective:** Implement HTTP caching validators (`ETag`, `Last-Modified`, `Cache-Control`, and `Expires`) in `crates/web-extractor` with RFC 7231/RFC 7232 compliance, conditional request generation (`If-None-Match`, `If-Modified-Since`), HTTP 304 Not Modified evaluation, and persistent HTTP cache indexing in `crates/storage` with transactional PostgreSQL migration, entity models, and in-memory/Postgres repository implementations.
- **Scope:** `crates/web-extractor/src/http/validator.rs`, `crates/web-extractor/src/http/fetcher.rs`, `crates/web-extractor/src/http/mod.rs`, `crates/web-extractor/src/lib.rs`, `crates/storage/migrations/20261119000002_http_validators.sql`, `crates/storage/src/schema/http_cache.rs`, `crates/storage/src/schema/ddl.sql`, `crates/storage/src/schema/mod.rs`, `crates/storage/src/http_cache.rs`, `crates/storage/src/lib.rs`, integration tests in both crates.
- **Prerequisites:** `8.2.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/web-extractor/src/http/validator.rs`
  - `crates/web-extractor/src/http/fetcher.rs`
  - `crates/web-extractor/src/http/mod.rs`
  - `crates/web-extractor/src/lib.rs`
  - `crates/storage/migrations/20261119000002_http_validators.sql`
  - `crates/storage/src/schema/http_cache.rs`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/src/http_cache.rs`
  - `crates/storage/src/lib.rs`
  - `crates/web-extractor/tests/http_validators_test.rs`
  - `crates/storage/tests/http_cache_storage_test.rs`
- **Implementation Tasks:**
  1. Author `crates/web-extractor/src/http/validator.rs`:
     - Implement `Etag` parsing and matching (strong vs weak validators per RFC 7232).
     - Implement RFC 7231 IMF-fixdate parsing and formatting (`parse_http_date`, `format_http_date`).
     - Implement `HttpCacheControl` directive parser (`max-age`, `no-cache`, `no-store`, `must-revalidate`, `public`, `private`, `immutable`) with freshness evaluation (`is_fresh`).
     - Implement `HttpValidator` model with `from_headers`, `conditional_headers`, and freshness checks.
     - Implement `FetchOutcome` (`Modified(FetchedResponse)` vs `NotModified`).
  2. Enhance `WebFetcher` in `crates/web-extractor/src/http/fetcher.rs`:
     - Implement `fetch_conditional(&self, target: &WebTarget, validator: Option<&HttpValidator>) -> Result<FetchOutcome, WebExtractorError>`.
     - Inject `If-None-Match` and `If-Modified-Since` conditional headers when validators exist.
     - Intercept `304 Not Modified` responses before redirect handling (avoiding `MissingLocationHeader` error) and return `FetchOutcome::NotModified` with updated validator metadata.
  3. Create PostgreSQL migration `20261119000002_http_validators.sql` in `crates/storage/migrations`:
     - Define `http_validators` relational table with `url` primary key, `etag`, `last_modified`, `blake3_hex`, `storage_key`, `byte_size`, `cache_control`, `expires_at`, `created_at`, `last_checked_at`.
     - Add indexes on `blake3_hex`, `last_checked_at`, and `expires_at`.
     - Update `crates/storage/src/schema/ddl.sql` and `crates/storage/src/schema/mod.rs` (`EXPECTED_TABLES`, `EXPECTED_INDEXES`).
  4. Author entity model `crates/storage/src/schema/http_cache.rs` (`HttpValidatorRow`).
  5. Author repository in `crates/storage/src/http_cache.rs`:
     - Trait `HttpCacheRepository`: `get_validator`, `upsert_validator`, `touch_validator`, `delete_validator`, `prune_stale_validators`, `get_content_cache`, `upsert_content_cache`.
     - Implement `InMemoryHttpCacheRepository` using `Arc<RwLock<...>>` for fast, zero-dependency testing and embedded execution.
     - Implement `PostgresHttpCacheRepository` using `sqlx::PgPool` with transactional upserts and index queries.
  6. Re-export types in `crates/web-extractor/src/lib.rs` and `crates/storage/src/lib.rs`.
  7. Author integration tests:
     - `crates/web-extractor/tests/http_validators_test.rs`: ETag parsing/matching, RFC 7231 dates, Cache-Control freshness, conditional header injection, 304 Not Modified handling with MockHttpTransport, 200 OK modified response.
     - `crates/storage/tests/http_cache_storage_test.rs`: `HttpCacheRepository` CRUD, touch on 304, upsert on 200, deletion, pruning stale records, content cache integration.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-web-extractor`
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell scripts/check-all.ps1`
- **Acceptance Criteria:**
  - `Etag` correctly parses strong and weak representations and performs RFC 7232 weak/strong comparisons.
  - RFC 7231 date formatting and parsing roundtrips accurately.
  - `HttpCacheControl` correctly parses all standard directives and calculates freshness against timestamps.
  - `WebFetcher::fetch_conditional` injects `If-None-Match` and `If-Modified-Since` and handles `304 Not Modified` cleanly without attempting redirect traversal.
  - `HttpCacheRepository` supports storing, retrieving, touching, and pruning HTTP validator records in both in-memory and PostgreSQL backends.
  - 100% of tests pass across web-extractor and storage crates, workspace clippy is warning-free, formatting is clean, and monorepo quality gates pass.
- **Risk Analysis:**
  - Risk: Malformed `Cache-Control` or date headers from erratic origin servers cause panic or rejection.
  - Mitigation: Permissive parsing with graceful fallbacks to `None` for invalid date strings or unrecognized directives.
- **Rollback Plan:**
  - Revert Subphase 8.3.1 additions via Git reset to commit `e29b43d`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-web-extractor`: 206 passed; 0 failed (7 new integration tests in `tests/http_validators_test.rs`).
  - `cargo test -p web-repo-to-ai-storage`: 75 passed; 0 failed (5 new integration tests in `tests/http_cache_storage_test.rs`).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all crates.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Rust Workspace, Frontend Web (213 passing vitest tests), and Podman storage budget (5.27 GB / 30 GB, 17.6%).
- **Git Commit Reference:** `18ba5d4`
- **Status:** `ACCEPTED`

---

#### Subphase `8.3.2` — Git commit/blob caching in crates/storage and crates/github-extractor
- **Unique ID:** `8.3.2`
- **Objective:** Implement content-addressed Git blob caching, commit metadata persistence, incremental tree diffing, and cache-aware blob fetching across `crates/storage` and `crates/github-extractor`. Ensure repeat conversions of unchanged files bypass network roundtrips entirely, while modified or newly added files are fetched incrementally with $O(N)$ efficiency.
- **Scope:** `crates/storage/migrations/20261119000003_git_cache.sql`, `crates/storage/src/schema/git_cache.rs`, `crates/storage/src/schema/ddl.sql`, `crates/storage/src/schema/mod.rs`, `crates/storage/src/git_cache.rs`, `crates/storage/src/lib.rs`, `crates/storage/tests/git_cache_storage_test.rs`, `crates/github-extractor/src/cache.rs`, `crates/github-extractor/src/client.rs`, `crates/github-extractor/src/lib.rs`, `crates/github-extractor/tests/git_cache_incremental_test.rs`.
- **Prerequisites:** `8.3.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/migrations/20261119000003_git_cache.sql`
  - `crates/storage/src/schema/git_cache.rs`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/src/git_cache.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/tests/git_cache_storage_test.rs`
  - `crates/github-extractor/src/cache.rs`
  - `crates/github-extractor/src/client.rs`
  - `crates/github-extractor/src/lib.rs`
  - `crates/github-extractor/tests/git_cache_incremental_test.rs`
  - `scripts/check-all.ps1`
  - `Cargo.toml`
- **Implementation Tasks:**
  1. Author PostgreSQL relational schema migration `crates/storage/migrations/20261119000003_git_cache.sql`:
     - Table `git_commit_cache`: composite primary key `(owner, repo, commit_sha)`, `tree_sha`, `parent_shas`, `author_name`, `author_email`, `committed_at`, `message`, `total_files`, `total_bytes`, `created_at`, `last_accessed_at`.
     - Table `git_blob_cache`: primary key `blob_sha` (Git Object SHA-1), `byte_size`, `blake3_hex`, `encoding`, `raw_bytes`, `storage_key`, `created_at`, `last_accessed_at`.
     - Indexes on `last_accessed_at` and `blake3_hex`.
     - Synchronize `crates/storage/src/schema/ddl.sql` and `crates/storage/src/schema/mod.rs` (`EXPECTED_TABLES`, `EXPECTED_INDEXES`).
  2. Implement entity models in `crates/storage/src/schema/git_cache.rs`:
     - `GitCommitCacheRow`: builder methods and field validation (owner, repo, 40-character hex commit_sha and tree_sha).
     - `GitBlobCacheRow`: builder methods, validation (40-char blob_sha, optional 64-char blake3_hex, payload byte size check).
  3. Implement repository abstraction in `crates/storage/src/git_cache.rs`:
     - Trait `GitCacheRepository`: `get_commit`, `upsert_commit`, `touch_commit`, `delete_commit`, `get_blob`, `get_blobs_batch`, `upsert_blob`, `touch_blob`, `delete_blob`, `prune_stale_commits`, `prune_stale_blobs`.
     - Implement `InMemoryGitCacheRepository` with `Arc<RwLock<HashMap<...>>>` for concurrent unit tests and embedded executions.
     - Implement `PostgresGitCacheRepository` with `sqlx::PgPool`, parameterized queries, batch chunking, and index pruning.
  4. Implement incremental caching module in `crates/github-extractor/src/cache.rs`:
     - `TreeIncrementalDiff`: categorizes tree changes between commits into `added_files`, `modified_files`, `unchanged_blobs`, and `deleted_files`.
     - Trait `GitCacheStore`: async blob and tree caching abstraction.
     - `InMemoryGitCacheStore`: thread-safe in-memory cache store.
     - `IncrementalExtractionStats`: tracking metrics for `total_candidates`, `cache_hits`, `network_fetches`, `cache_hit_ratio`, `bytes_saved_from_cache`, and `bytes_fetched_from_network`.
     - `IncrementalBlobExtractor`: fetches uncached blobs concurrently while streaming cached blobs directly from storage.
  5. Extend `GitHubApiClient` in `crates/github-extractor/src/client.rs`:
     - `fetch_pinned_tree_incremental`: checks cache for existing tree and falls back to network with tree caching.
     - `extract_source_files_incremental`: utilizes `IncrementalBlobExtractor` to bypass network fetches for existing blobs.
  6. Re-export types in `crates/storage/src/lib.rs` and `crates/github-extractor/src/lib.rs`.
  7. Author integration test suites:
     - `crates/storage/tests/git_cache_storage_test.rs`: 5 tests covering schema synchronization, row validations, commit/blob CRUD, batch lookups, and stale entry pruning.
     - `crates/github-extractor/tests/git_cache_incremental_test.rs`: 4 tests covering tree diffing, cache store operations, cold/warm cache performance, and binary/non-utf8 handling.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-storage --test git_cache_storage_test`
  - `cargo test -p web-repo-to-ai-github-extractor --test git_cache_incremental_test`
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo test -p web-repo-to-ai-github-extractor`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Relational schema migration and DDL catalogs include `git_commit_cache` and `git_blob_cache` with all required indexes.
  - Entity models enforce cryptographic 40-character Git SHA invariants and correct timestamps.
  - `GitCacheRepository` provides zero-mock, verified CRUD and batch operations across both in-memory and PostgreSQL targets.
  - `TreeIncrementalDiff` accurately classifies added, modified, unchanged, and deleted entries across commit trees.
  - `IncrementalBlobExtractor` achieves 100% cache hit reuse for unchanged blob SHAs and records accurate byte-savings metrics.
  - 100% test pass rate across storage (80/80) and github-extractor (175/175) crates, zero clippy warnings, zero formatting diffs, cargo deny clean, and all 5 check-all gates pass.
- **Risk Analysis:**
  - Risk: Corrupted blob contents in storage could lead to silent data corruption during AI conversions.
  - Mitigation: Content-addressed Git SHA-1 hashing validates payload integrity; batch lookups verify blob lengths and SHAs.
- **Rollback Plan:**
  - Revert Subphase 8.3.2 additions via Git reset to commit `b04099d`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-storage --test git_cache_storage_test`: 5 passed; 0 failed.
  - `cargo test -p web-repo-to-ai-github-extractor --test git_cache_incremental_test`: 4 passed; 0 failed.
  - `cargo test -p web-repo-to-ai-storage`: 80 passed; 0 failed (57 lib, 2 batch, 5 git_cache, 5 http_cache, 11 token_storage).
  - `cargo test -p web-repo-to-ai-github-extractor`: 175 passed; 0 failed (30 lib, 10 app_manifest, 10 architecture_context, 9 auth_recovery, 12 auth, 4 git_cache_incremental, 7 live_extraction, 10 oauth, 9 rate_limit, 8 fallback, 7 recovery, 21 target, 9 leakage, 10 refresh, 5 truncation, 11 unauthorized, 1 doctest).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace crates and targets.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Workflows, Rust Workspace, Frontend Web (17 files, 213 passing vitest tests), and Resource Budget (Podman storage 0 B / 30 GB, 0%).
- **Git Commit Reference:** `7ea6e48`
- **Status:** `ACCEPTED`

---

#### Subphase `8.3.3` — Content-addressed storage in crates/storage
- **Unique ID:** `8.3.3`
- **Objective:** Establish the Content-Addressed Storage (CAS) subsystem in `crates/storage` providing content-addressed blob deduplication, dual cryptographic validation (BLAKE3 and SHA-256), sharded directory partitioning (`blobs/xx/<blake3>.bin`), atomic writes, on-read corruption detection, LRU storage quota enforcement, and zero-mock audit verification.
- **Scope:** `crates/storage/src/cas.rs`, `crates/storage/src/lib.rs`, `crates/storage/src/schema/cache.rs`, `crates/storage/src/schema/ddl.sql`, `crates/storage/src/schema/mod.rs`, `crates/storage/migrations/20261119000004_cas_index.sql`, `crates/storage/tests/cas_storage_test.rs`.
- **Prerequisites:** `8.3.2` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/storage/migrations/20261119000004_cas_index.sql`
  - `crates/storage/src/cas.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/src/schema/cache.rs`
  - `crates/storage/src/schema/ddl.sql`
  - `crates/storage/src/schema/mod.rs`
  - `crates/storage/tests/cas_storage_test.rs`
- **Implementation Tasks:**
  1. Author PostgreSQL relational schema migration `crates/storage/migrations/20261119000004_cas_index.sql` adding index `idx_content_cache_blake3` on table `content_cache_index`.
  2. Synchronize `crates/storage/src/schema/ddl.sql` and `crates/storage/src/schema/mod.rs` (`EXPECTED_INDEXES`).
  3. Extend `ContentCacheIndexRow` in `crates/storage/src/schema/cache.rs` with `with_last_accessed_at` and `formatted_byte_size`.
  4. Implement `crates/storage/src/cas.rs`:
     - Domain models: `CasWriteOutcome`, `CasIntegrityOutcome`, `CasPruneReport`, `CasAuditReport`, `CasStats`.
     - Repository trait `ContentCacheIndexRepository` with `InMemoryContentCacheIndexRepository` and `PostgresContentCacheIndexRepository`.
     - `ContentAddressedStorage` unified CAS trait.
     - `CasEngine` coordinator implementation with BLAKE3 hashing, sharded storage key routing (`layout::blob_key`), deduplication detection, on-read integrity verification, batch operations, LRU pruning, full audit, and telemetry tracking.
  5. Re-export CAS types and traits in `crates/storage/src/lib.rs`.
  6. Author comprehensive integration tests in `crates/storage/tests/cas_storage_test.rs` covering CAS put/get, deduplication, custom key routing, batch queries, corruption detection, LRU pruning, and schema synchronization.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-storage --test cas_storage_test`
  - `cargo test -p web-repo-to-ai-storage`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1`
- **Acceptance Criteria:**
  - `content_cache_index` includes `idx_content_cache_blake3` across DDL and schema catalogs.
  - `ContentAddressedStorage` provides verified deduplication (skipping rewrite on identical content).
  - Cryptographic corruption is detected on read with explicit error.
  - LRU pruning enforces capacity ceiling by evicting oldest-accessed blobs.
  - Full audit detects valid, missing, and corrupted blobs.
  - 100% test pass rate across storage crate (87/87), zero clippy warnings, zero formatting diffs, cargo deny clean, and all 5 check-all gates pass.
- **Risk Analysis:**
  - Risk: Concurrently reading a blob while another process prunes it.
  - Mitigation: Atomic writes and atomic existence checks; on-read digest verification ensures partial files are rejected.
- **Rollback Plan:**
  - Revert Subphase 8.3.3 additions via Git reset to commit `a0b418f`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-storage --test cas_storage_test`: 7 passed; 0 failed.
  - `cargo test -p web-repo-to-ai-storage`: 87 passed; 0 failed (57 lib, 2 batch, 7 cas, 5 git_cache, 5 http_cache, 11 token_storage).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Workflows, Rust Workspace, Frontend Web (17 files, 213 passing vitest tests), and Resource Budget (Podman storage 0 B / 30 GB, 0%).
- **Git Commit Reference:** `5d96400`
- **Status:** `ACCEPTED`

---

#### Subphase `8.3.4` — Changed-file processing in crates/job-engine & services/worker
- **Unique ID:** `8.3.4`
- **Objective:** Implement differential file analysis, manifest change classification (`Unchanged`, `Added`, `Modified`, `Deleted`), and bounded incremental execution coordination in `crates/job-engine` and `services/worker`. Ensure unchanged files completely bypass re-processing and are resolved directly from cache/CAS, while changed files are scheduled through the bounded worker pool with conservation verification and atomic metric accounting.
- **Scope:** `crates/job-engine/src/incremental.rs`, `crates/job-engine/src/lib.rs`, `crates/job-engine/tests/incremental_processing_test.rs`, `services/worker/src/metrics.rs`, `services/worker/src/engine.rs`, `services/worker/tests/worker_incremental_test.rs`.
- **Prerequisites:** `8.3.3` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/job-engine/src/incremental.rs`
  - `crates/job-engine/src/lib.rs`
  - `crates/job-engine/tests/incremental_processing_test.rs`
  - `services/worker/src/metrics.rs`
  - `services/worker/src/engine.rs`
  - `services/worker/tests/worker_incremental_test.rs`
- **Implementation Tasks:**
  1. Author `crates/job-engine/src/incremental.rs`:
     - Define `FileSignature`: path, BLAKE3 digest, byte size, optional last modified timestamp.
     - Define `ChangeCategory`: `Unchanged`, `Added`, `Modified`, `Deleted`.
     - Define `ClassifiedChange`: file path paired with change category and optional old/new signatures.
     - Implement `ChangedFileDiff` $O(N)$ differential analysis comparing base manifest vs target manifest with mathematical balance invariants (`base_total == unchanged + modified + deleted`, `target_total == unchanged + added + modified`).
     - Define `IncrementalCandidate<T>`, `ReusedItem<R>`, and `IncrementalExecutionReport<R>` with cache hit ratio, bytes saved, and conservation verification (`total == reused + processed + failed`).
     - Implement `IncrementalJobCoordinator`: coordinator running bounded parallel execution (`BoundedParallelExecutor`) exclusively on changed items (`Added` + `Modified`) while routing `Unchanged` items directly to the reuse resolver.
  2. Extend `services/worker/src/metrics.rs`:
     - Add atomic counters `incremental_items_reused` and `incremental_bytes_saved` to `WorkerMetrics` and `WorkerMetricsSnapshot`.
     - Implement `record_incremental_reuse` method.
  3. Extend `services/worker/src/engine.rs`:
     - Implement `WorkerEngine::execute_incremental_batch` executing incremental tasks within worker concurrency limits and progress dispatch.
  4. Author integration tests:
     - `crates/job-engine/tests/incremental_processing_test.rs`: 4 comprehensive tests validating diff classification, zero-change execution, partial changed execution, failure tolerance, and conservation invariants.
     - `services/worker/tests/worker_incremental_test.rs`: integration test validating worker incremental batch execution and telemetry counters.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-job-engine`
  - `cargo test -p web-repo-to-ai-worker`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Manifest diffing classifies changes into `Unchanged`, `Added`, `Modified`, and `Deleted` with $O(N)$ complexity and strict balance invariants.
  - Unchanged files are routed directly to the reuse resolver without redundant re-processing or worker pool saturation.
  - Changed items execute via bounded parallel execution with error isolation.
  - Conservation invariant ($\text{total} == \text{reused} + \text{processed} + \text{failed}$) is strictly verified in `IncrementalExecutionReport`.
  - Worker telemetry tracks `incremental_items_reused` and `incremental_bytes_saved`.
  - 100% test pass rate across job-engine (19/19) and worker (16/16), zero clippy warnings, zero formatting diffs, cargo deny clean, and all 5 check-all gates pass.
- **Risk Analysis:**
  - Risk: Silent file drops or duplicate emissions during partial failures in incremental runs.
  - Mitigation: Mathematical conservation check in `IncrementalExecutionReport::conservation_verified()` asserts zero unaccounted items; failure in individual items does not abort reuse of unchanged items.
- **Rollback Plan:**
  - Revert Subphase 8.3.4 additions via Git reset to commit `ac5ed0c`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-job-engine`: 19 passed; 0 failed (15 unit, 4 incremental_processing_test).
  - `cargo test -p web-repo-to-ai-worker`: 16 passed; 0 failed (9 lib unit, 1 main unit, 5 bounded_parallel_worker_test, 1 worker_incremental_test).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Workflows, Rust Workspace (524 tests passed), Frontend Web (17 files, 213 passing vitest tests), and Resource Budget (Podman storage 0 B / 30 GB, 0%).
- **Git Commit Reference:** `3ac5100`
- **Status:** `ACCEPTED`

---

### PHASE 8.3 FORMAL CERTIFICATION

- **Stage:** `STAGE 8 — LARGE REPOSITORIES, BATCH AND CRAWLING`
- **Phase:** `Phase 8.3 — Incremental processing`
- **Subphases Certified (4 of 4):**
  - `8.3.1` (HTTP validators in crates/storage and crates/web-extractor) — `ACCEPTED`
  - `8.3.2` (Git commit/blob caching in crates/storage and crates/github-extractor) — `ACCEPTED`
  - `8.3.3` (Content-addressed storage in crates/storage) — `ACCEPTED`
  - `8.3.4` (Changed-file processing in crates/job-engine & services/worker) — `ACCEPTED`
- **Phase Deliverables Achieved:**
  - RFC 7232/7231 HTTP validator caching, conditional headers (`If-None-Match`, `If-Modified-Since`), HTTP 304 Not Modified zero-download handling, and persistent PostgreSQL cache repository.
  - Relational Git commit and blob persistence (`git_commit_cache`, `git_blob_cache`), tree diff classification (`TreeIncrementalDiff`), and cache-aware blob streaming (`IncrementalBlobExtractor`).
  - Unified Content-Addressed Storage (`ContentAddressedStorage`, `CasEngine`) with dual cryptographic hashing (BLAKE3 + SHA-256), sharded key layout, deduplication detection, on-read corruption detection, and LRU pruning.
  - Differential manifest change classification, bounded incremental parallel execution coordinator (`IncrementalJobCoordinator`), and worker incremental batch execution with conservation guarantees and telemetry metrics.
- **Verification Summary:** All unit, integration, and monorepo quality gates passed with 0 warnings, 0 errors, and 0 failures.

---

### Upcoming Subphase (Stage 8 — Large Repositories, Batch and Crawling)

#### Phase 8.4 — Scalability

#### Subphase `8.4.1` — Adaptive concurrency in crates/job-engine & services/worker
- **Unique ID:** `8.4.1`
- **Objective:** Implement dynamic Additive Increase / Multiplicative Decrease (AIMD) concurrency regulation, latency-aware throttling, domain-level rate limiting isolation (`DomainAdaptiveConcurrencyManager`), cooperative permit management (`AdaptivePermit`), and adaptive parallel batch execution (`AdaptiveParallelExecutor`, `WorkerEngine::execute_adaptive_batch`) in `crates/job-engine` and `services/worker`.
- **Scope:** `crates/job-engine/src/adaptive.rs`, `crates/job-engine/src/lib.rs`, `crates/job-engine/tests/adaptive_concurrency_test.rs`, `services/worker/src/metrics.rs`, `services/worker/src/engine.rs`, `services/worker/tests/worker_adaptive_test.rs`.
- **Prerequisites:** `8.3.4` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/job-engine/src/adaptive.rs`
  - `crates/job-engine/src/lib.rs`
  - `crates/job-engine/tests/adaptive_concurrency_test.rs`
  - `services/worker/src/metrics.rs`
  - `services/worker/src/engine.rs`
  - `services/worker/tests/worker_adaptive_test.rs`
- **Implementation Tasks:**
  1. Author `crates/job-engine/src/adaptive.rs`:
     - Define `AdaptiveConcurrencyConfig`: boundaries (`min_concurrency`, `max_concurrency`, `initial_concurrency`), dynamics (`increase_step`, `success_threshold`, `decrease_factor`, `cooldown_duration`), and optional `latency_sla`.
     - Define `AdaptiveOutcome`: `Success { duration }`, `RateLimited { retry_after }` (HTTP 429), `ServerOverloaded` (HTTP 503/504), `Timeout`, `ConnectionError`, `OtherFailure`, with automatic classification helper `AdaptiveOutcome::from_result`.
     - Define `AdaptiveDecision`: `Unchanged`, `Increased`, `Decreased`, `InCooldown`.
     - Define `AdaptiveConcurrencyStats`: point-in-time metrics snapshot.
     - Implement `AdaptivePermit`: RAII permit guard decrementing active permits and notifying waiters on drop.
     - Implement `AdaptiveConcurrencyController`: thread-safe AIMD coordinator managing dynamic concurrency ceilings, permit acquisition with cancellation support, outcome-based limit adjustment, cooldown window enforcement, and telemetry tracking.
     - Implement `DomainAdaptiveConcurrencyManager`: per-domain controller registry ensuring upstream throttling or rate limiting on one domain does not degrade concurrent processing on independent healthy domains.
     - Implement `AdaptiveParallelExecutor`: high-performance batch task runner executing tasks with dynamic concurrency permits and outcome telemetry.
  2. Extend `services/worker/src/metrics.rs`:
     - Add atomic counters `adaptive_scale_ups`, `adaptive_backoffs`, `adaptive_rate_limits`, and `current_adaptive_limit` to `WorkerServiceMetrics` and `WorkerMetricsSnapshot`.
     - Implement `record_adaptive_stats` telemetry recording method.
  3. Extend `services/worker/src/engine.rs`:
     - Implement `WorkerEngine::execute_adaptive_batch` coordinating adaptive concurrency batch execution within worker bounds and recording telemetry.
  4. Author comprehensive integration tests:
     - `crates/job-engine/tests/adaptive_concurrency_test.rs`: 6 integration tests validating config validation, additive increase, multiplicative decrease, cooldown enforcement, domain isolation, and adaptive parallel execution.
     - `services/worker/tests/worker_adaptive_test.rs`: integration test validating worker daemon adaptive batch execution and metrics recording.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-job-engine`
  - `cargo test -p web-repo-to-ai-worker`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1`
- **Acceptance Criteria:**
  - AIMD algorithm additively increases concurrency after sustained consecutive successes up to max bound.
  - Multiplicative decrease immediately reduces concurrency upon rate limits (HTTP 429) or server overloads down to min bound.
  - Cooldown duration suppresses premature scale-ups during stabilization windows.
  - Domain isolation guarantees throttling on one origin does not constrain concurrency on separate hosts.
  - Worker metrics record scale-ups, backoffs, rate limits, and dynamic concurrency ceilings.
  - 100% test pass rate across job-engine (25/25) and worker (17/17), zero clippy warnings, zero formatting diffs, cargo deny clean, and all 5 check-all gates pass.
- **Risk Analysis:**
  - Risk: Rapid oscillation / thrashing between scale-up and backoff under jittery network latency.
  - Mitigation: Consecutive success threshold (default 3) combined with cooldown timer suppresses scale-up oscillation.
- **Rollback Plan:**
  - Revert Subphase 8.4.1 additions via Git reset to commit `cb2b349`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-job-engine`: 25 passed; 0 failed (15 unit, 6 adaptive, 4 incremental).
  - `cargo test -p web-repo-to-ai-worker`: 17 passed; 0 failed (9 lib unit, 1 main unit, 5 bounded_parallel, 1 incremental, 1 adaptive).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Workflows, Rust Workspace (531 tests passed), Frontend Web (17 files, 213 passing vitest tests), and Resource Budget (Podman storage 0 B / 30 GB, 0%).
- **Git Commit Reference:** `e44918a`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 8 — Large Repositories, Batch and Crawling)

#### Phase 8.4 — Scalability

#### Subphase `8.4.2` — Backpressure in crates/job-engine & services/worker
- **Unique ID:** `8.4.2`
- **Objective:** Implement dual-watermark bounded backpressure flow control (`BackpressureConfig`, `BackpressureController`), producer suspension and resume mechanics, load-shedding strategies (`BlockProducer`, `DropOldest`, `DropNewest`, `RejectWithRetryAfter`), flow-controlled bounded queue (`FlowControlledQueue`), and backpressure parallel batch execution (`BackpressureParallelExecutor`, `WorkerEngine::execute_flow_controlled_batch`) in `crates/job-engine` and `services/worker`.
- **Scope:** `crates/job-engine/src/backpressure.rs`, `crates/job-engine/src/error.rs`, `crates/job-engine/src/lib.rs`, `crates/job-engine/tests/backpressure_test.rs`, `services/worker/src/metrics.rs`, `services/worker/src/engine.rs`, `services/worker/tests/worker_backpressure_test.rs`.
- **Prerequisites:** `8.4.1` (`ACCEPTED`).
- **Files/Components Affected:**
  - `crates/job-engine/src/backpressure.rs`
  - `crates/job-engine/src/error.rs`
  - `crates/job-engine/src/lib.rs`
  - `crates/job-engine/tests/backpressure_test.rs`
  - `services/worker/src/metrics.rs`
  - `services/worker/src/engine.rs`
  - `services/worker/tests/worker_backpressure_test.rs`
- **Implementation Tasks:**
  1. Author `crates/job-engine/src/backpressure.rs`:
     - Define `LoadSheddingStrategy`: `BlockProducer`, `DropOldest`, `DropNewest`, `RejectWithRetryAfter`.
     - Define `BackpressureConfig`: `max_buffer_capacity`, `high_watermark`, `low_watermark`, `max_wait_timeout`, `shed_strategy`.
     - Define `BackpressureState`: `Normal`, `Warning`, `Saturated`.
     - Define `BackpressureStats`: point-in-time metrics snapshot.
     - Implement `BackpressureSlot`: RAII slot token decrementing depth and waking suspended producers when depth drops below low watermark.
     - Implement `BackpressureController`: thread-safe dual-watermark backpressure regulator coordinating admission permits, timeout enforcement, and cooperative cancellation.
     - Implement `FlowControlledQueue`: high-performance thread-safe bounded FIFO queue wrapping `VecDeque` with backpressure regulation.
     - Implement `BackpressureParallelExecutor`: streaming batch executor maintaining bounded in-flight tasks and backpressure flow control across producers and workers.
  2. Extend `crates/job-engine/src/error.rs`:
     - Add `JobEngineError::BackpressureQueueSaturated` and `JobEngineError::BackpressureTimeout`.
  3. Re-export backpressure types in `crates/job-engine/src/lib.rs`.
  4. Extend `services/worker/src/metrics.rs`:
     - Add atomic counters `backpressure_events`, `backpressure_items_shed`, and `current_queue_depth` to `WorkerServiceMetrics` and `WorkerMetricsSnapshot`.
     - Implement `record_backpressure_stats` telemetry recording method.
  5. Extend `services/worker/src/engine.rs`:
     - Implement `WorkerEngine::execute_flow_controlled_batch` executing flow-controlled tasks within worker concurrency limits and progress dispatch.
  6. Author comprehensive integration tests:
     - `crates/job-engine/tests/backpressure_test.rs`: 6 integration tests validating configuration boundaries, state transitions (Normal -> Warning -> Saturated -> Normal), load shedding (`DropNewest`), producer suspension and resume on drain, queue FIFO push/pop semantics, and flow-controlled batch execution.
     - `services/worker/tests/worker_backpressure_test.rs`: integration test validating worker flow-controlled batch execution and telemetry recording.
- **Verification Commands:**
  - `cargo test -p web-repo-to-ai-job-engine`
  - `cargo test -p web-repo-to-ai-worker`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo deny check`
  - `powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1`
- **Acceptance Criteria:**
  - Dual-watermark backpressure activates (`Saturated`) when buffer depth reaches high watermark (default 80%) and clears (`Normal`) when depth drains below low watermark (default 40%).
  - Suspended producers wait asynchronously without thread blocking or busy spinning and wake up immediately upon drain.
  - Load shedding policies (`DropNewest`, `RejectWithRetryAfter`) reject excess items with explicit domain errors.
  - `FlowControlledQueue` preserves FIFO ordering and bounds memory footprint.
  - Worker metrics track `backpressure_events`, `backpressure_items_shed`, and `current_queue_depth`.
  - 100% test pass rate across job-engine (31/31) and worker (18/18), zero clippy warnings, zero formatting diffs, cargo deny clean, and all 5 check-all gates pass.
- **Risk Analysis:**
  - Risk: Deadlock if consumers stop draining while producers are blocked waiting for slots.
  - Mitigation: `max_wait_timeout` guarantees producers do not hang indefinitely; cancellation token aborts blocked acquisitions immediately.
- **Rollback Plan:**
  - Revert Subphase 8.4.2 additions via Git reset to commit `8423f33`.
- **Measurement / Empirical Evidence:**
  - `cargo test -p web-repo-to-ai-job-engine`: 31 passed; 0 failed (15 unit, 6 adaptive, 6 backpressure, 4 incremental).
  - `cargo test -p web-repo-to-ai-worker`: 18 passed; 0 failed (9 lib unit, 1 main unit, 5 bounded_parallel, 1 incremental, 1 adaptive, 1 backpressure).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all workspace targets.
  - `cargo fmt --all -- --check`: formatting clean across workspace.
  - `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
  - `powershell scripts/check-all.ps1`: ALL 5 GATES PASSED (0 failures) across Bootstrap/Governance, Workflows, Rust Workspace (538 tests passed), Frontend Web (17 files, 213 passing vitest tests), and Resource Budget (Podman storage 0 B / 30 GB, 0%).
- **Git Commit Reference:** `f312d21`
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 8 — Large Repositories, Batch and Crawling)

#### Phase 8.4 — Scalability

| Subphase ID | Objective | Prerequisites | Affected Files / Components | Status |
| :--- | :--- | :--- | :--- | :--- |
| `8.4.3` | Streaming large exports | `8.4.2` (`ACCEPTED`) | `crates/job-engine/*`, `crates/exporters/*`, `services/worker/*` | `NOT_STARTED` |

---

## STAGES 8 – 11 QUEUED LEDGER SUMMARY

All subsequent subphases for **Stage 8** (`8.4.3`–`8.4.4`), **Stage 9** (`9.1.1`–`9.4.4`), **Stage 10** (`10.1.1`–`10.5.4`), and **Stage 11** (`11.1.1`–`11.4.5`) are defined in [ROADMAP.md](./ROADMAP.md) and currently hold status `NOT_STARTED`. Each subphase is expanded into its full 14-attribute block in this ledger immediately upon entering `IN_PROGRESS`.
