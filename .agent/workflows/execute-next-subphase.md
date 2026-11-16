---
description: Execute the single next eligible subphase according to the strict one-subphase-at-a-time engineering contract.
---

# Workflow: Execute Next Subphase (`execute-next-subphase.md`)

## Strict Rule
**Execute exactly ONE subphase per invocation.** Never automatically advance to the subsequent subphase without explicit user confirmation.

## Step-by-Step Procedure
1. **Read Current State:**
   - Read `PROJECT_STATUS.md` and `IMPLEMENTATION_LEDGER.md`.
   - Identify the single next subphase with status `NOT_STARTED` (or `IN_PROGRESS` / `BLOCKED` being resumed) whose prerequisites are all `ACCEPTED` or `VERIFIED`.
2. **Present Subphase Plan:**
   - State the Subphase ID, Objective, Scope, Prerequisites, Affected Files/Crates, Implementation Tasks, and Measurable Acceptance Criteria.
   - Mark the subphase `IN_PROGRESS` in `PROJECT_STATUS.md` and `IMPLEMENTATION_LEDGER.md`.
3. **Activate Relevant Skills & Rules:**
   - Read the applicable `.agent/skills/*/SKILL.md` and `.agent/rules/*.md` files for the domain.
4. **Implement Bounded Scope:**
   - Implement ONLY the tasks belonging to this subphase.
   - Do not introduce mocks, placeholders, or out-of-scope features. If new work is discovered, record it in `TODO.md` and `ROADMAP.md`.
5. **Verify Subphase:**
   - Run `.agent/workflows/verify-subphase.md` (linting, formatting, unit/integration/security tests).
6. **Update Ledgers & Documentation:**
   - Append verification evidence to `IMPLEMENTATION_LEDGER.md`, update `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and relevant `docs/*.md`.
7. **Commit Verified Work:**
   - Stage and commit with message format: `<type>(<scope>): <summary> [<subphase-id>]` and record the commit SHA in `IMPLEMENTATION_LEDGER.md`.
8. **Report & Stop:**
   - Present created/modified files, test results, commit SHA, any remaining limitations, and the exact next subphase ID awaiting approval.
   - **STOP and WAIT for explicit user confirmation.**
