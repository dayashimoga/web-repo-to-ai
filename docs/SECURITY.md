# Security Architecture & Threat Model (`docs/SECURITY.md`)

**Project:** `web-repo-to-ai`  
**Status:** Baseline Security Specification v1.0

---

## 1. Threat Model Overview

Because `web-repo-to-ai` fetches arbitrary user-submitted URLs, parses untrusted HTML and source code, and integrates with private GitHub repositories, it operates across four high-risk trust boundaries:

1. **Untrusted Network Boundary:** Arbitrary external HTTP/HTTPS endpoints and DNS servers controlled by potential attackers.
2. **Untrusted Content Boundary:** Maliciously crafted HTML, JavaScript, deeply nested archives, oversized files, and prompt-injection payloads embedded in web pages or repositories.
3. **Credential & Tenant Boundary:** GitHub App installation tokens, OAuth sessions, and private repository artifacts belonging to distinct users.
4. **Host & Container Boundary:** CPU, memory, disk, and local network resources on the self-hosted server or developer workstation.

---

## 2. Mandatory Security Controls

### 2.1 SSRF & DNS Rebinding Prevention (`crates/url-security`)
- **Scheme Restriction:** Only `http` and `https` schemes are permitted (`file://`, `ftp://`, `gopher://`, `data:`, `javascript:`, `ws://` are rejected immediately).
- **Blocked IP Ranges:**
  - IPv4 Loopback: `127.0.0.0/8`
  - IPv4 Private (RFC1918): `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
  - IPv4 Link-Local & Cloud Metadata: `169.254.0.0/16` (explicitly blocking `169.254.169.254`)
  - IPv4 CGNAT: `100.64.0.0/10`
  - IPv4 Broadcast, Multicast, Unspecified: `0.0.0.0/8`, `224.0.0.0/4`, `240.0.0.0/4`
  - IPv6 Loopback (`::1`), Unspecified (`::`), Unique Local (`fc00::/7`), Link-Local (`fe80::/10`), Multicast (`ff00::/8`), and IPv4-mapped IPv6 addresses (`::ffff:0:0/96`) mapping to any blocked IPv4 range.
- **DNS Pinning:** DNS resolution occurs inside `url-security` prior to socket connect; the verified socket IP is pinned for that HTTP connection to defeat time-of-check-to-time-of-use (TOCTOU) DNS rebinding.
- **Redirect Revalidation:** Automatic redirects are disabled. Every `301`/`302`/`303`/`307`/`308` response `Location` header is parsed, resolved, and validated against the full SSRF policy before following (maximum 5 redirects).

### 2.2 Browser Rendering Sandboxing (`services/browser-worker`)
- Runs in a dedicated, unprivileged container without access to PostgreSQL, GitHub App keys, or internal host services.
- Intercepts all Playwright page and subresource requests (XHR, fetch, images, frames, stylesheets) and blocks any request resolving to a private/loopback IP or non-HTTP(S) scheme.
- Enforces hard per-page navigation timeouts (default 30s) and memory ceilings.

### 2.3 Private GitHub App & Secret Protection
- **Least Privilege:** GitHub App requests only `contents: read` and `metadata: read`. No write, push, webhook administration, or organization administration scopes are ever requested.
- **OAuth State Validation:** OAuth login and installation callbacks verify a cryptographically signed, short-lived `state` token bound to the caller's session cookie (`HttpOnly`, `SameSite=Lax`, `Secure` in production).
- **Encryption at Rest:** Installation tokens and refresh tokens are encrypted using `AES-256-GCM` with per-record random nonces before persistence.
- **Secret Redaction:** Structured logs, error payloads, and exported documents pass through a redaction filter that strips GitHub tokens (`ghp_*`, `ghs_*`, `ghu_*`, `github_pat_*`), Authorization headers, and configured secret patterns.

### 2.4 Untrusted Content & Prompt Injection Defense
- **Zero Code Execution:** Target repository code, Makefiles, `build.rs`, `package.json` postinstall hooks, or template scripts are never executed. Parsing is strictly static (`tree-sitter`, `html5ever`).
- **Structural Delimiting:** Exporters wrap extracted file and webpage bodies inside unambiguous provenance headers and fenced/delimited boundaries so downstream LLMs can distinguish system instructions from untrusted source content.
- **Injection Telemetry:** Static heuristics scan extracted content for hidden text (zero-width characters, invisible CSS text, instruction-override phrases) and surface findings in `quality-report.json` warnings without silently altering lossless source code.

### 2.5 Resource Exhaustion & Denial-of-Service Controls
- **Decompression Bomb Protection:** Streaming HTTP and archive readers enforce both compressed and uncompressed byte limits.
- **Rate Limiting:** Token-bucket rate limiting per client IP and per authenticated user on job creation and OAuth endpoints.
