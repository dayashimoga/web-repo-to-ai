# PROJECT TODO TRACKER (`TODO.md`)

**Policy:** Append-only historical task tracker. Never delete or overwrite completed items; mark completed tasks with `[x]` and their completion date/subphase.

---

## Stage 0 — Project Bootstrap and Engineering Foundation

### Phase 0.1 — Repository and governance initialization
- [x] **[0.1.1]** Inspect environment, repository status, hardware budget, Podman machine limits, and available CLI tools *(Completed 2026-11-16)*
- [x] **[0.1.2]** Initialize Git repository (`main`), `.gitignore`, `.gitattributes`, `.editorconfig`, `LICENSE`, `README.md`, and 27 monorepo directories *(Completed 2026-11-16, commit `58e0f80`)*
- [x] **[0.1.3]** Create `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md`, and baseline `docs/*.md` specifications *(Completed 2026-11-16, commit `6372d8a`)*
- [x] **[0.1.4]** Create `AGENTS.md`, 9 specialized agent definitions, 10 `.agent/skills/*/SKILL.md`, 10 `.agent/rules/*.md`, and 10 `.agent/workflows/*.md` *(Completed 2026-11-16, commit `f2aa8e9`)*
- [x] **[0.1.5]** Create `ROADMAP.md`, `docs/ROADMAP.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1` *(Completed 2026-11-16)*

### Phase 0.2 — GitHub and CI foundation
- [x] **[0.2.1]** Create or connect GitHub remote repository (`web-repo-to-ai`) and push `main` branch *(Completed 2026-11-17, https://github.com/dayashimoga/web-repo-to-ai)*
- [x] **[0.2.2]** Configure branch protection and repository settings where authorized *(Completed 2026-11-17, linear history, Dependabot alerts/fixes, main protection)*
- [x] **[0.2.3]** Create executable GitHub Actions CI workflows (`ci.yml`, `build.yml`, `release.yml`, `container.yml`) *(Completed 2026-11-17, workflows created & validated on GitHub runners)*
- [x] **[0.2.4]** Configure linting, formatting, and test checks for Rust and Frontend *(Completed 2026-11-17, rustfmt.toml, .prettierrc.json, check-all.ps1/sh)*
- [x] **[0.2.5]** Configure security and dependency scanning workflows (`security.yml`, `dependency-review.yml`, `codeql.yml`, `deny.toml`) *(Completed 2026-11-17, deny.toml & security workflows active)*

### Phase 0.3 — Containerized development foundation
- [x] **[0.3.1]** Create Podman development configuration (`deploy/podman-compose.yml` and multi-stage Containerfiles) *(Completed 2026-11-17, podman-compose.yml & 4 Containerfiles)*
- [x] **[0.3.2]** Enforce the 30 GB storage ceiling (`scripts/check-resources.*`, `scripts/clean-caches.*`) *(Completed 2026-11-17, storage monitoring & safe cache pruning)*
- [x] **[0.3.3]** Create development environment startup, test, and recovery scripts (`scripts/dev-*.*`) *(Completed 2026-11-17, cross-platform dev lifecycle scripts)*
- [ ] **[0.3.4]** Validate reproducible container builds
- [ ] **[0.3.5]** Document container setup, resource management, and recovery in `docs/DEVELOPMENT.md` and `docs/DEPLOYMENT.md`

---

## Subsequent Stages (Queued)
- [ ] **[Stage 1]** Frontend and Product Experience (`1.1.1` – `1.4.3`)
- [ ] **[Stage 2]** Backend and Processing Foundation (`2.1.1` – `2.4.4`)
- [ ] **[Stage 3]** Universal Webpage Extraction (`3.1.1` – `3.4.4`)
- [ ] **[Stage 4]** Canonical Model and Export Engine (`4.1.1` – `4.4.4`)
- [ ] **[Stage 5]** Public GitHub Repository Extraction (`5.1.1` – `5.5.4`)
- [ ] **[Stage 6]** Private GitHub Repositories (`6.1.1` – `6.4.4`)
- [ ] **[Stage 7]** AI Context and Token Optimization (`7.1.1` – `7.4.4`)
- [ ] **[Stage 8]** Large Repositories, Batch and Crawling (`8.1.1` – `8.4.4`)
- [ ] **[Stage 9]** Developer and AI Agent Integrations (`9.1.1` – `9.4.4`)
- [ ] **[Stage 10]** Advanced Documents and Content (`10.1.1` – `10.5.4`)
- [ ] **[Stage 11]** Production Certification (`11.1.1` – `11.4.5`)
