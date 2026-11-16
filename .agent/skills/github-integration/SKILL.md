---
name: github-integration
description: >-
  Use this skill when implementing public or private GitHub repository
  extraction, Git Trees API traversal, subtree recovery, GitHub App OAuth and
  installation token management, or repository completeness accounting in
  crates/github-extractor.
---

# GitHub Integration Engineering Skill

## 1. Purpose
Acquire and convert public and authorized private GitHub repositories across any branch, tag, commit, or path filter with 100% file completeness accounting and strict read-only credential security.

## 2. When to Use
- Implementing GitHub URL parsing, metadata retrieval, branch/tag/commit resolution, and Git Trees discovery in `crates/github-extractor`.
- Handling `truncated: true` Git Trees via subtree recovery or shallow Git archive fallback.
- Implementing GitHub App OAuth flows, installation token exchange, encryption at rest, and secret redaction.
- Generating repository file trees, architecture context, and completeness manifests.

## 3. Required Procedures
1. Resolve target owner, repo, and immutable commit SHA before fetching tree entries so all extracted files correspond to a single consistent revision.
2. Check the `truncated` flag on GitHub Git Trees API responses; if `true`, traverse remaining subtrees or use archive/git fallback so zero files are silently missed.
3. Classify every discovered file (`source_code`, `documentation`, `configuration`, `dependency_manifest`, `test_fixture`, `generated_or_lockfile`, `binary`, `oversized`).
4. Verify the completeness invariant before marking extraction complete:
   $$\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$$
5. For private repositories, use short-lived GitHub App installation tokens with `contents: read` and `metadata: read` only, encrypted at rest and redacted from all logs/outputs.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [04-extraction-accuracy.md](../../rules/04-extraction-accuracy.md)
- Enforces [05-github-security.md](../../rules/05-github-security.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- Completeness verification tests proving that missing/unaccounted files block job completion.
- `BLAKE3` / `SHA-256` content hash verification against raw source blobs.
- Cross-user authorization isolation and token-leakage redaction tests.

## 6. Expected Outputs
- Complete `CdmDocument` representing the repository hierarchy, source files, manifest, architecture relationships, and `CompletenessReport`.

## 7. Common Failure Modes
- **Silent Tree Truncation:** Ignoring `truncated: true` on repositories with >100,000 entries and falsely reporting a complete extraction.
- **Mutable Ref Drift:** Fetching file blobs against a moving branch name (`main`) instead of pinning the resolved commit SHA.
- **Token Exposure:** Including an authenticated tarball download URL or `Authorization` header inside error messages or provenance exports.
