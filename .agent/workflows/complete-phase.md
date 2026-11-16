---
description: Audit and certify completion of an entire Phase once all of its constituent subphases have reached ACCEPTED status.
---

# Workflow: Complete Phase (`complete-phase.md`)

## Purpose
Verify that all subphases (`X.Y.1` .. `X.Y.N`) within Phase `X.Y` are `ACCEPTED`, integrated cleanly, and free of cross-subphase regressions.

## Procedure
1. **Ledger Audit:**
   - Inspect `IMPLEMENTATION_LEDGER.md` and confirm every subphase in Phase `X.Y` has status `ACCEPTED`, recorded test evidence, and a valid Git commit reference.
2. **Cross-Subphase Integration Test:**
   - Run full workspace test suites covering all modules touched during Phase `X.Y`.
3. **Documentation Sync Check:**
   - Confirm `docs/*.md`, `CHANGELOG.md`, `TODO.md`, and `PROJECT_STATUS.md` accurately reflect the completed phase capabilities.
4. **Stop Gate:**
   - Report phase completion summary to the user and identify the first subphase of the next phase (`X.(Y+1).1`). Do not begin that subphase until explicitly approved.
