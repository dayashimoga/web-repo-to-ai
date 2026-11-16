---
description: Verify the active subphase against all functional, accuracy, security, performance, and documentation acceptance criteria before marking it VERIFIED or ACCEPTED.
---

# Workflow: Verify Subphase (`verify-subphase.md`)

## Purpose
Provide objective, repeatable proof that the current subphase meets every acceptance criterion and complies with `GLOBAL_ENGINEERING_CONTRACT.md`.

## Verification Checklist
1. **No-Mock / No-Placeholder Audit:**
   - Scan modified files for `todo!()`, `unimplemented!()`, hardcoded sample responses, or unwired UI controls (`.agent/rules/09-no-mock-no-placeholder.md`).
2. **Rust Workspace Checks (when Rust files are present):**
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
3. **Frontend Checks (when `apps/web` is initialized):**
   - `npm --prefix apps/web run lint`
   - `npm --prefix apps/web run check`
   - `npm --prefix apps/web test`
4. **Security & Diff Review:**
   - Inspect `git status` and `git diff` for accidental secret inclusion, SSRF bypasses, unvalidated redirects, or swallowed errors.
5. **Acceptance Criteria Traceability:**
   - Confirm each acceptance criterion defined for the subphase in `IMPLEMENTATION_LEDGER.md` has corresponding passing test output or objective command verification.
6. **Status Transition:**
   - Transition subphase to `VERIFIED` once objective test evidence is captured, and to `ACCEPTED` only when 100% of acceptance criteria and documentation updates are complete.
