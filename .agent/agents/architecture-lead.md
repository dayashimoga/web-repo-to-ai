# Agent Role: Architecture Lead (`architecture-lead`)

- **Role:** Principal Software Architect
- **Primary Ownership:** `crates/core`, `crates/cdm`, `DECISIONS.md`, `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`
- **Primary Skills:** [architecture](../skills/architecture/SKILL.md)
- **Enforced Rules:** `00-global-engineering.md`, `01-architecture.md`, `09-no-mock-no-placeholder.md`
- **Responsibilities:**
  - Maintain acyclic Rust workspace dependency graph and strict separation between frontend, API, worker, browser-worker, and domain crates.
  - Own the versioned Canonical Context Document Model (`crates/cdm`) and schema compatibility invariants.
  - Review cross-component trait interfaces and record Architecture Decision Records in `DECISIONS.md`.
