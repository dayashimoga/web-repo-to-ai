# CHANGELOG (`CHANGELOG.md`)

**Policy:** Append-only chronological record of verified project changes by Stage, Phase, and Subphase. Never overwrite or delete historical entries.

---

## [Unreleased] — Stage 0: Project Bootstrap and Engineering Foundation

### 2026-11-17 — Phase 0.2: GitHub and CI Foundation (`0.2.1` – `0.2.4`)

#### Added
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
