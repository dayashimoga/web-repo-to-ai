---
description: Run full-workspace regression, golden dataset, completeness accounting, and contract test suites.
---

# Workflow: Regression Audit (`regression-audit.md`)

## Purpose
Ensure recent changes have not degraded extraction fidelity, source-code preservation, repository completeness accounting, API contracts, or UI responsiveness.

## Procedure
1. **Workspace Unit & Integration Suite:**
   - Run `cargo test --workspace` and `npm --prefix apps/web test`.
2. **Golden Dataset Comparison:**
   - Run golden webpage and repository extraction tests verifying exact `BLAKE3` content hashes, section hierarchies, tables, and code blocks.
3. **Completeness Invariant Verification:**
   - Confirm repository test fixtures satisfy $\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$ with zero unaccounted files.
4. **Coverage Verification:**
   - Verify critical core crates maintain >= 95% meaningful test coverage.
