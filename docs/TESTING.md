# Testing, Validation & Quality Evidence (`docs/TESTING.md`)

**Project:** `web-repo-to-ai`

---

## 1. Testing Pyramid & Coverage Requirements

- **Critical Core Coverage Target:** >= 95% meaningful line/branch test coverage across core domain crates (`cdm`, `url-security`, `web-extractor`, `github-extractor`, `normalizer`, `deduplication`, `tokenizer`, `context-optimizer`, `chunking`, `exporters`).
- **Mandatory CI Pass Rate:** 100%. Zero disabled or skipped tests to force green builds.

---

## 2. Required Test Suites by Category

1. **Unit & Property Tests:**
   - CDM schema serialization, validation, and version migration invariants.
   - Tokenizer exact vs. approximate counting accuracy and chunk boundary determinism.
   - Completeness accounting equation verification ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$).
2. **Security & Adversarial Tests:**
   - SSRF blocklist tests covering IPv4 loopback, RFC1918, link-local/metadata (`169.254.169.254`), CGNAT, IPv6 loopback/unique-local/link-local, IPv4-mapped IPv6 (`::ffff:127.0.0.1`), DNS rebinding simulation, and multi-hop redirect bypass attempts.
   - GitHub token redaction and cross-tenant authorization isolation tests.
   - Prompt-injection payload detection and structural framing verification.
   - Zip-bomb / decompression limit enforcement tests.
3. **Golden Dataset Regression Tests:**
   - Real HTML article, documentation, table, and code-block fixtures compared against deterministic golden CDM and Markdown outputs.
   - Real public and controlled private repository trees verified for 100% file accounting and exact `BLAKE3` / `SHA-256` source hash preservation.
4. **End-to-End & Accessibility UI Tests:**
   - Keyboard navigation, ARIA semantics, mobile/tablet/desktop viewports, and live SSE job workspace transitions.
