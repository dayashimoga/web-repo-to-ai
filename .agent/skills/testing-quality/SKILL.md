---
name: testing-quality
description: >-
  Use this skill when designing, implementing, or executing unit, integration,
  contract, golden-dataset, adversarial security, accessibility, E2E, or
  performance benchmark tests across the monorepo.
---

# QA, Reliability & Performance Testing Skill

## 1. Purpose
Prove that every implemented subphase satisfies its functional, accuracy, security, and performance acceptance criteria through repeatable automated tests and objective evidence.

## 2. When to Use
- Writing test suites for any Rust crate, backend service, or frontend component.
- Building golden datasets for webpage and GitHub repository conversion accuracy.
- Executing `.agent/workflows/verify-subphase.md` or `.agent/workflows/regression-audit.md`.
- Measuring test coverage, memory usage, and throughput benchmarks.

## 3. Required Procedures
1. Write unit and property tests covering normal paths, boundary conditions, malformed inputs, and error recovery.
2. Write integration and contract tests verifying real PostgreSQL state transitions, job lease recovery, and API error schemas.
3. Verify source-hash (`BLAKE3` / `SHA-256`) preservation and golden snapshot parity on extraction outputs.
4. Record exact test execution commands, pass/fail counts, and coverage metrics in `IMPLEMENTATION_LEDGER.md` and `docs/TESTING.md`.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [04-extraction-accuracy.md](../../rules/04-extraction-accuracy.md)
- Enforces [06-testing.md](../../rules/06-testing.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- >= 95% meaningful test coverage on critical core modules.
- 100% pass rate on mandatory unit, integration, security, and UI test suites.
- Zero `#[ignore]` or skipped tests used to mask failures.

## 6. Expected Outputs
- Automated test suites, golden fixtures, benchmark harnesses, and verifiable test evidence recorded in `IMPLEMENTATION_LEDGER.md`.

## 7. Common Failure Modes
- **Testing Mocks Instead of Reality:** Mocking the HTML parser or CDM serializer in unit tests so real parser regressions go undetected.
- **Coverage Vanity:** Writing assertion-free tests solely to inflate line coverage numbers.
- **Flaky Timing Dependencies:** Using arbitrary `sleep()` calls in async tests instead of deterministic condition synchronization.
