---
name: documentation
description: >-
  Use this skill when creating, updating, or auditing project tracking ledgers
  (PROJECT_STATUS.md, IMPLEMENTATION_LEDGER.md, ROADMAP.md, TODO.md,
  CHANGELOG.md, DECISIONS.md) and technical documentation under docs/*.
---

# Technical Documentation & Ledger Governance Skill

## 1. Purpose
Maintain accurate, synchronized, append-only project governance ledgers and comprehensive technical documentation across every stage, phase, and subphase.

## 2. When to Use
- At the start and completion of every subphase (`execute-next-subphase.md`, `verify-subphase.md`).
- When updating architecture, API contracts, security controls, user onboarding, or third-party license inventories.
- Auditing documentation completeness before phase, stage, or release certification.

## 3. Required Procedures
1. Read `PROJECT_STATUS.md` and `IMPLEMENTATION_LEDGER.md` before starting any subphase work.
2. Record every subphase's unique ID, objective, scope, prerequisites, affected files, tasks, acceptance criteria, tests, security/performance notes, validation evidence, git commit SHA, and status (`NOT_STARTED`, `IN_PROGRESS`, `BLOCKED`, `IMPLEMENTED`, `VERIFIED`, `ACCEPTED`).
3. Append new entries to `CHANGELOG.md` and `TODO.md` without erasing or overwriting historical records.
4. Keep `docs/*.md` synchronized with the actual implemented code—never document unimplemented features as already completed.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [07-documentation.md](../../rules/07-documentation.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- All Markdown links resolve to existing files in the repository.
- Every subphase marked `VERIFIED` or `ACCEPTED` in `IMPLEMENTATION_LEDGER.md` includes concrete command outputs and commit references.

## 6. Expected Outputs
- Up-to-date root tracking files (`PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `ROADMAP.md`, `TODO.md`, `CHANGELOG.md`, `DECISIONS.md`) and `docs/*.md` reference guides.

## 7. Common Failure Modes
- **History Erasure:** Overwriting prior subphase verification records in `IMPLEMENTATION_LEDGER.md` or truncating `CHANGELOG.md`.
- **Premature Status Inflation:** Marking a subphase `VERIFIED` or `ACCEPTED` before tests have actually been executed.
