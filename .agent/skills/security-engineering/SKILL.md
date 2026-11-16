---
name: security-engineering
description: >-
  Use this skill when implementing or auditing URL/SSRF defenses (crates/url-security),
  OAuth and GitHub App credentials, encryption at rest, secret redaction, browser
  sandbox isolation, rate limiting, or prompt-injection defenses.
---

# Application Security Engineering Skill

## 1. Purpose
Enforce defense-in-depth security controls across all network acquisition, credential handling, browser rendering, storage, and export surfaces.

## 2. When to Use
- Implementing or modifying `crates/url-security` (URL validation, DNS resolution, IP pinning, redirect revalidation).
- Implementing GitHub App OAuth, token storage encryption, session authorization, or secret redaction.
- Auditing `services/browser-worker` container and network isolation.
- Performing subphase security reviews or executing `.agent/workflows/security-audit.md`.

## 3. Required Procedures
1. Verify that every outbound HTTP client in `web-extractor`, `github-extractor`, and `browser-worker` enforces `url-security` IP blocklists and DNS pinning on initial requests and every redirect hop.
2. Verify that private IPv4 (`127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16`, `100.64.0.0/10`) and private/loopback IPv6 (`::1`, `fc00::/7`, `fe80::/10`, `::ffff:0:0/96`) are blocked with zero bypasses.
3. Verify that GitHub App credentials use `AES-256-GCM` encryption at rest, CSRF-protected OAuth state, and strictly read-only permissions.
4. Ensure all untrusted extracted content is framed with explicit structural boundaries and never executed.
5. Run dependency vulnerability scans (`cargo audit` / `cargo deny` / `npm audit`).

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [05-github-security.md](../../rules/05-github-security.md)
- Enforces [06-testing.md](../../rules/06-testing.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- 100% pass rate on SSRF, DNS rebinding, redirect-to-loopback, IPv6-mapped IPv4, cross-user access, and token-redaction test suites.
- Zero unresolved critical or high-severity vulnerabilities in dependency audits.

## 6. Expected Outputs
- Hardened security primitives in `crates/url-security` and `services/api`, adversarial security test suites, and documented threat-model updates in `docs/SECURITY.md`.

## 7. Common Failure Modes
- **TOCTOU DNS Rebinding:** Validating a hostname's DNS record in a pre-check, then letting `reqwest` re-resolve the hostname independently when opening the TCP socket.
- **Unvalidated Redirects:** Validating the initial URL (`https://example.com/redirect`) while allowing the HTTP client to automatically follow a `302` redirect to `http://169.254.169.254/latest/meta-data/`.
- **Error Log Secret Leakage:** Logging full `reqwest::Error` or HTTP headers containing `Authorization: Bearer ghs_...`.
