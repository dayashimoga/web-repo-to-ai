---
description: Execute Stage 0 project bootstrap one subphase at a time, establishing repository structure, governance contracts, skills, rules, workflows, and ledgers.
---

# Workflow: Bootstrap Project (`bootstrap-project.md`)

## Purpose
Initialize and verify the `web-repo-to-ai` repository foundation, engineering governance, agent customizations, CI pipelines, and containerized development environment across Stage 0 (`0.1` -> `0.2` -> `0.3`), strictly one subphase at a time.

## Procedure
1. **Inspect Environment (`0.1.1`):**
   - Check OS, CPU/RAM, free disk space, Git status, GitHub credentials, Rust (`rustc`/`cargo`), Node (`node`/`npm`), and Podman (`podman`/`podman-compose`).
2. **Initialize Monorepo (`0.1.2`):**
   - Initialize Git (`main`), configure `.gitignore`, `.gitattributes`, `.editorconfig`, `LICENSE`, `README.md`, and create `apps/`, `services/`, `crates/`, `docs/`, `deploy/`, `scripts/`, and `.agent/` directories.
3. **Establish Engineering Governance (`0.1.3`):**
   - Author `GLOBAL_ENGINEERING_CONTRACT.md`, `REQUIREMENTS.md`, `DECISIONS.md`, and baseline `docs/*.md` specifications.
4. **Create Agent Ecosystem (`0.1.4`):**
   - Author `AGENTS.md`, `.agent/skills/*/SKILL.md` (10 skills), `.agent/rules/*.md` (10 rules), `.agent/workflows/*.md` (10 workflows), and `.agent/agents/*.md` (9 roles).
5. **Initialize Tracking Ledgers (`0.1.5`):**
   - Author `ROADMAP.md`, `docs/ROADMAP.md`, `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, and `CHANGELOG.md`.
6. **Commit & Stop Policy:**
   - Commit verified artifacts and **STOP** before the next unapproved subphase (`0.2.1`), presenting complete evidence and awaiting explicit user approval.
