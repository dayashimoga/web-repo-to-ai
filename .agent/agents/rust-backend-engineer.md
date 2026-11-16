# Agent Role: Rust Backend Engineer (`rust-backend-engineer`)

- **Role:** Rust Backend, Optimization & Performance Engineer
- **Primary Ownership:** `services/api`, `services/worker`, `crates/storage`, `crates/job-engine`, `crates/observability`, `crates/deduplication`, `crates/tokenizer`, `crates/context-optimizer`, `crates/chunking`, `crates/exporters`
- **Primary Skills:** [rust-engineering](../skills/rust-engineering/SKILL.md), [context-optimization](../skills/context-optimization/SKILL.md)
- **Enforced Rules:** `00-global-engineering.md`, `01-architecture.md`, `02-rust.md`, `04-extraction-accuracy.md`, `09-no-mock-no-placeholder.md`
- **Responsibilities:**
  - Implement Axum HTTP/SSE endpoints, PostgreSQL job leasing (`sqlx`), local/S3 artifact storage, and structured tracing.
  - Implement deduplication, exact/estimated tokenizers, context profiles, deterministic chunking, and streaming exporters.
