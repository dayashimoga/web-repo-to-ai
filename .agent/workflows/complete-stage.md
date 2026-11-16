---
description: Certify completion of a full Stage deliverable after all phases and subphases within that stage are ACCEPTED.
---

# Workflow: Complete Stage (`complete-stage.md`)

## Purpose
Verify that Stage `X` delivers a complete, independently verifiable increment of the `web-repo-to-ai` product satisfying the stage acceptance gate.

## Procedure
1. **Verify All Constituent Phases:**
   - Confirm all phases (`X.1` .. `X.M`) and subphases within Stage `X` are marked `ACCEPTED` in `IMPLEMENTATION_LEDGER.md`.
2. **Stage Acceptance Gate Validation:**
   - Execute end-to-end verification of the Stage `X` deliverable as defined in `ROADMAP.md`.
3. **Security, Performance & Resource Check:**
   - Run `.agent/workflows/security-audit.md`, `.agent/workflows/performance-audit.md`, and verify Podman storage remains <= 30 GB.
4. **Update Stage Status:**
   - Record Stage `X` completion in `PROJECT_STATUS.md`, `IMPLEMENTATION_LEDGER.md`, and `CHANGELOG.md`.
5. **Stop Gate:**
   - Present the Stage Certification Summary and await explicit user authorization before starting Stage `X+1`, Phase `(X+1).1`, Subphase `(X+1).1.1`.
