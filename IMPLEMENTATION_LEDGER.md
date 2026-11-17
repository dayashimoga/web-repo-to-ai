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
- **Git Commit Reference:** `8a13ca6`
- **Status:** `ACCEPTED`

---

### Phase 0.3 — Upcoming Subphases (Awaiting User Approval for `0.3.3`)

| Subphase ID | Objective | Prerequisites | Affected Files / Components | Status |
| :--- | :--- | :--- | :--- | :--- |
| `0.3.3` | Create development environment scripts | `0.3.2` (`ACCEPTED`) | `scripts/dev-*.ps1` | `NOT_STARTED` |
| `0.3.4` | Validate reproducible container builds | `0.3.3` | `deploy/*`, `IMPLEMENTATION_LEDGER.md` | `NOT_STARTED` |
| `0.3.5` | Document setup and recovery | `0.3.4` | `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md` | `NOT_STARTED` |

---

## STAGES 1 – 11 QUEUED LEDGER SUMMARY

All subphases for **Stage 1** (`1.1.1`–`1.4.3`), **Stage 2** (`2.1.1`–`2.4.4`), **Stage 3** (`3.1.1`–`3.4.4`), **Stage 4** (`4.1.1`–`4.4.4`), **Stage 5** (`5.1.1`–`5.5.4`), **Stage 6** (`6.1.1`–`6.4.4`), **Stage 7** (`7.1.1`–`7.4.4`), **Stage 8** (`8.1.1`–`8.4.4`), **Stage 9** (`9.1.1`–`9.4.4`), **Stage 10** (`10.1.1`–`10.5.4`), and **Stage 11** (`11.1.1`–`11.4.5`) are defined in [ROADMAP.md](./ROADMAP.md) and currently hold status `NOT_STARTED`. Each subphase is expanded into its full 14-attribute block in this ledger immediately upon entering `IN_PROGRESS`.
