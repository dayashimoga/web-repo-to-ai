---
name: architecture
description: >-
  Use this skill when designing or modifying system architecture, crate dependency
  boundaries, the Canonical Context Document Model (CDM), storage/job traits, or
  recording Architecture Decision Records (ADRs).
---

# Architecture Engineering Skill

## 1. Purpose
Maintain a clean, modular, acyclic architecture across the Rust workspace (`crates/*`, `services/*`) and web frontend (`apps/web`), centered on the versioned Canonical Context Document Model (`crates/cdm`).

## 2. When to Use
- Creating or modifying crate boundaries in `crates/*` or services in `services/*`.
- Defining or evolving the `CdmDocument` schema (`crates/cdm`).
- Designing provider-independent traits (`StorageBackend`, `JobEngine`, `TokenizerAdapter`).
- Evaluating new third-party dependencies or recording decisions in `DECISIONS.md`.

## 3. Required Procedures
1. Inspect `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, and `DECISIONS.md` before proposing structural changes.
2. Verify that the dependency graph remains strictly acyclic (`crates/*` never depend on `services/*`).
3. Ensure every extraction source maps into `CdmDocument` and every exporter reads exclusively from `CdmDocument`.
4. Verify third-party crate/package licenses against `docs/LICENSING.md` before adding dependencies.
5. Record significant architectural trade-offs as a new ADR entry in `DECISIONS.md`.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [01-architecture.md](../../rules/01-architecture.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- `cargo metadata --no-deps` and `cargo check --workspace` confirm valid, acyclic workspace resolution.
- Schema version and backward-compatibility tests in `crates/cdm` pass 100%.

## 6. Expected Outputs
- Well-bounded Rust trait and struct definitions in `crates/core` and `crates/cdm`.
- Updated architecture diagrams and data flow documentation in `docs/ARCHITECTURE.md` and `docs/DATA_MODEL.md`.
- Append-only ADR entries in `DECISIONS.md`.

## 7. Common Failure Modes
- **Layer Leakage:** Allowing `services/api` handlers to perform direct HTML parsing or raw filesystem I/O instead of delegating through `job-engine` and `storage`.
- **CDM Bypass:** Writing direct extractor-to-Markdown shortcuts that bypass `CdmDocument` validation and provenance tracking.
- **Database Bloat:** Storing multi-megabyte source payloads directly in PostgreSQL columns instead of `crates/storage`.
