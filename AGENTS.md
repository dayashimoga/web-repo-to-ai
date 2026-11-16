# AGENTS ORGANIZATION & OWNERSHIP (`AGENTS.md`)

**Project:** `web-repo-to-ai`  
**Role:** Cross-Agent Source of Truth & Workspace Directory Rule  
**Enforcement:** All agents operating in this repository MUST comply with [GLOBAL_ENGINEERING_CONTRACT.md](./GLOBAL_ENGINEERING_CONTRACT.md) and the strict one-subphase-at-a-time rule.

---

## 1. Accountable Coordination Model

Work is integrated through **one accountable Technical Lead / Coordinator**. Specialized agents may perform parallel research, design analysis, and independent code/security/QA review, but **concurrent agents must never modify the same files simultaneously without explicit file-ownership locking**.

---

## 2. Specialized Agent Roster & Ownership Boundaries

| Agent ID | Role Title | Primary Directory / Crate Ownership | Associated Skills & Rules |
| :--- | :--- | :--- | :--- |
| `architecture-lead` | Architecture Lead | `crates/core`, `crates/cdm`, `DECISIONS.md`, `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md` | `architecture` / `00`, `01`, `09` |
| `rust-backend-engineer` | Rust Backend & Performance Engineer | `services/api`, `services/worker`, `crates/storage`, `crates/job-engine`, `crates/observability` | `rust-engineering` / `00`, `01`, `02`, `09` |
| `frontend-engineer` | Web Frontend & UX Engineer | `apps/web` | `frontend-engineering` / `00`, `03`, `06`, `09` |
| `extraction-engineer` | Web Extraction & Crawling Engineer | `crates/web-extractor`, `crates/code-intelligence`, `crates/normalizer`, `services/browser-worker` | `web-extraction` / `00`, `02`, `04`, `09` |
| `github-integration-engineer` | GitHub Integration Engineer | `crates/github-extractor` | `github-integration` / `00`, `04`, `05`, `09` |
| `context-optimization-engineer` *(via Rust/Extraction)* | AI Context & Token Optimization Engineer | `crates/deduplication`, `crates/tokenizer`, `crates/context-optimizer`, `crates/chunking`, `crates/exporters` | `context-optimization` / `00`, `02`, `04`, `09` |
| `security-reviewer` | Security Reviewer & Application Security Engineer | `crates/url-security`, `docs/SECURITY.md`, cross-repo security review | `security-engineering` / `00`, `05`, `09` |
| `qa-engineer` | QA, Reliability & Performance Engineer | `docs/TESTING.md`, `docs/PERFORMANCE.md`, integration/golden/E2E test suites | `testing-quality` / `00`, `06`, `09` |
| `devops-engineer` | DevOps & CI/CD Engineer | `.github/workflows`, `deploy/`, `scripts/`, `docs/DEPLOYMENT.md` | `devops-cicd` / `00`, `08`, `09` |
| `documentation-reviewer` | Documentation Reviewer & Technical Writer | `README.md`, `CHANGELOG.md`, `TODO.md`, `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `docs/*` | `documentation` / `00`, `07`, `09` |

---

## 3. Non-Negotiable Agent Operating Rules

1. **One Subphase at a Time:**
   - Read `PROJECT_STATUS.md` and `IMPLEMENTATION_LEDGER.md` first.
   - Execute ONLY the currently active subphase.
   - Run all verification commands, record objective evidence, commit, and **STOP** for explicit user confirmation.
2. **Zero Mock / Zero Placeholder Policy (`.agent/rules/09-no-mock-no-placeholder.md`):**
   - Never introduce fake responses, hardcoded repository outputs, `todo!()` macros in delivered paths, or inactive UI buttons.
3. **File Conflict Prevention:**
   - Before invoking a write-enabled subagent, assign an explicit list of allowed file paths in its prompt.
   - Review all subagent changes via `git diff` and `security-reviewer` / `qa-engineer` checks before committing.

---

## 4. Skills, Rules & Workflows Index

- **Skills (`.agent/skills/*/SKILL.md`):**
  - [architecture](./.agent/skills/architecture/SKILL.md)
  - [rust-engineering](./.agent/skills/rust-engineering/SKILL.md)
  - [frontend-engineering](./.agent/skills/frontend-engineering/SKILL.md)
  - [web-extraction](./.agent/skills/web-extraction/SKILL.md)
  - [github-integration](./.agent/skills/github-integration/SKILL.md)
  - [context-optimization](./.agent/skills/context-optimization/SKILL.md)
  - [security-engineering](./.agent/skills/security-engineering/SKILL.md)
  - [testing-quality](./.agent/skills/testing-quality/SKILL.md)
  - [devops-cicd](./.agent/skills/devops-cicd/SKILL.md)
  - [documentation](./.agent/skills/documentation/SKILL.md)
- **Rules (`.agent/rules/*.md`):** `00-global-engineering.md` through `09-no-mock-no-placeholder.md`
- **Workflows (`.agent/workflows/*.md`):** `bootstrap-project.md`, `execute-next-subphase.md`, `verify-subphase.md`, `complete-phase.md`, `complete-stage.md`, `security-audit.md`, `performance-audit.md`, `regression-audit.md`, `release-certification.md`, `project-status-review.md`
