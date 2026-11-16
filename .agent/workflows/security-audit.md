---
description: Execute a comprehensive security audit covering SSRF/DNS rebinding, redirect revalidation, GitHub credentials, browser isolation, secrets, and dependencies.
---

# Workflow: Security Audit (`security-audit.md`)

## Purpose
Verify compliance with Section 5 of `GLOBAL_ENGINEERING_CONTRACT.md`, `.agent/rules/05-github-security.md`, and `docs/SECURITY.md`.

## Audit Checklist
1. **SSRF & Network Destination Audit:**
   - Verify all outbound HTTP clients route through `crates/url-security`.
   - Run SSRF unit/integration tests covering IPv4 loopback, RFC1918, link-local/metadata (`169.254.169.254`), CGNAT, IPv6 private/loopback, IPv4-mapped IPv6, DNS rebinding, and multi-hop `3xx` redirects.
2. **Browser Worker Isolation Audit:**
   - Verify `services/browser-worker` has no database or GitHub credentials and blocks subresource requests to private/local IPs.
3. **GitHub App & Credential Audit:**
   - Verify permissions are strictly `contents: read` and `metadata: read`.
   - Verify OAuth state CSRF validation, `AES-256-GCM` encryption at rest, cross-user access isolation, and log/export token redaction.
4. **Untrusted Content & Code Execution Audit:**
   - Confirm zero execution of extracted repository code or scripts.
   - Confirm structural framing and prompt-injection warning telemetry.
5. **Secrets & Dependency Vulnerability Scan:**
   - Scan repository history/working tree for leaked keys or tokens.
   - Run `cargo audit` / `cargo deny` and `npm audit`.
