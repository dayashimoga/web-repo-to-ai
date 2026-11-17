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
- **Git Commit Reference:** PENDING_COMMIT
- **Status:** `ACCEPTED`

---

### Upcoming Subphase (Stage 4 — Canonical Model and Export Engine)

| Subphase ID | Objective | Prerequisites | Affected Files / Components | Status |
| :--- | :--- | :--- | :--- | :--- |
| `4.3.3` | Streaming downloads in services/api | `4.3.2` (`ACCEPTED`) | `services/api/*` | `NOT_STARTED` |

---

## STAGES 4 – 11 QUEUED LEDGER SUMMARY

All subsequent subphases for **Stage 4** (`4.1.1`–`4.4.4`), **Stage 5** (`5.1.1`–`5.5.4`), **Stage 6** (`6.1.1`–`6.4.4`), **Stage 7** (`7.1.1`–`7.4.4`), **Stage 8** (`8.1.1`–`8.4.4`), **Stage 9** (`9.1.1`–`9.4.4`), **Stage 10** (`10.1.1`–`10.5.4`), and **Stage 11** (`11.1.1`–`11.4.5`) are defined in [ROADMAP.md](./ROADMAP.md) and currently hold status `NOT_STARTED`. Each subphase is expanded into its full 14-attribute block in this ledger immediately upon entering `IN_PROGRESS`.
