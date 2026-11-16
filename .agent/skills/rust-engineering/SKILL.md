---
name: rust-engineering
description: >-
  Use this skill when implementing or reviewing Rust code across services/api,
  services/worker, and crates/*, covering async Tokio execution, Axum APIs, SQLx
  persistence, error handling, memory bounds, and performance.
---

# Rust Backend & Performance Engineering Skill

## 1. Purpose
Deliver safe, idiomatic, high-performance, resource-bounded Rust services and shared crates that compile with zero warnings and operate reliably within a CPU-only 16 GB RAM budget.

## 2. When to Use
- Implementing or refactoring any crate in `crates/*` or service in `services/api` and `services/worker`.
- Designing async Tokio concurrency, streaming I/O, or PostgreSQL `sqlx` queries and migrations.
- Auditing Rust code for `.unwrap()`, memory allocations, blocking calls on async executors, or compiler warnings.

## 3. Required Procedures
1. Define typed domain errors using `thiserror` in each crate and map them cleanly to API error envelopes in `services/api`.
2. Wrap all outbound network calls and heavy tasks in `tokio::time::timeout` and bounded `Semaphore` permits.
3. Stream large files and ZIP/JSONL exports using `AsyncRead` / `AsyncWrite` / `Stream` rather than buffering full repositories in memory.
4. Use `sqlx` parameterized queries and transactions (`FOR UPDATE SKIP LOCKED` for job leasing).
5. Run formatting, linting, and unit/integration tests before completing any subphase.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [01-architecture.md](../../rules/01-architecture.md)
- Enforces [02-rust.md](../../rules/02-rust.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- `cargo fmt --all -- --check` exits with `0`.
- `cargo clippy --workspace --all-targets -- -D warnings` exits with `0`.
- `cargo test --workspace` passes 100% with zero disabled tests.

## 6. Expected Outputs
- Production-grade Rust modules with comprehensive rustdoc comments on public traits/structs and unit test coverage >= 95% on critical core crates.

## 7. Common Failure Modes
- **Executor Starvation:** Running CPU-intensive `tree-sitter` or HTML parsing directly on core Tokio I/O threads without `spawn_blocking` or bounded worker pools.
- **Unbounded Buffering:** Calling `.bytes().await` on an arbitrary HTTP response without enforcing a streaming byte-limit check first.
- **Panic in Production:** Using `.unwrap()` on user input, header parsing, or regex captures.
