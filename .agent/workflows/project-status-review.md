---
description: Inspect current project status, active/completed subphases, blockers, and next eligible subphase without modifying application code.
---

# Workflow: Project Status Review (`project-status-review.md`)

## Purpose
Provide an immediate, accurate audit of project progress, subphase completion status, active blockers, and the next eligible subphase.

## Procedure
1. Read `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, `ROADMAP.md`, and `TODO.md`.
2. Run `git status` and `git log -n 10 --oneline` to verify working tree cleanliness and recent commit alignment.
3. Summarize:
   - Current Stage, Phase, and last completed Subphase.
   - Counts of `ACCEPTED`, `VERIFIED`, `IMPLEMENTED`, `IN_PROGRESS`, `BLOCKED`, and `NOT_STARTED` subphases.
   - Any active blockers or required external user actions.
   - The exact single next subphase awaiting user approval.
