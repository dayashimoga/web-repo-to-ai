---
description: Execute Stage 11 production release certification, verifying all security, performance, reliability, SBOM, license, and build gates.
---

# Workflow: Release Certification (`release-certification.md`)

## Purpose
Certify a production release of `web-repo-to-ai` with repeatable, objective evidence across all critical acceptance gates.

## Certification Gates
1. **Ledger Completeness:** All required stages, phases, and subphases for the release are `ACCEPTED` in `IMPLEMENTATION_LEDGER.md`.
2. **Quality Gate:** 100% pass rate on all unit, integration, contract, golden, and E2E tests; >= 95% coverage on critical core crates; zero compiler or linter warnings.
3. **Security Gate:** `.agent/workflows/security-audit.md` passes with zero unresolved critical/high vulnerabilities.
4. **Performance & Reliability Gate:** `.agent/workflows/performance-audit.md`, interrupted-job recovery, retry idempotency, and storage cleanup tests pass within the 16 GB RAM / 30 GB Podman envelope.
5. **Artifacts, SBOM & Licensing Gate:**
   - Production frontend build, backend container images, CycloneDX/SPDX SBOMs, third-party license inventory (`docs/LICENSING.md`), and `checksums.sha256` are generated and verified.
6. **Honest Certification Report:** Document verified capabilities and any remaining known limitations; never certify unverified features.
