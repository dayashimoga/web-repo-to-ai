# Rule 05: Network, GitHub & Application Security

1. **Mandatory SSRF Protection:** All outbound HTTP/HTTPS fetches and redirects must pass through `crates/url-security` with DNS pinning and strict blocking of loopback, RFC1918, link-local/metadata (`169.254.169.254`), CGNAT, and private IPv6 ranges.
2. **Redirect Revalidation:** Disable automatic HTTP redirects; validate every redirect target against the full URL and IP policy before following.
3. **Browser Isolation:** Isolate `services/browser-worker` from internal networks, credentials, and host filesystems; block private/local subresource requests.
4. **Minimal GitHub Permissions:** Private repository access must use a GitHub App with strictly `Contents: Read-only` and `Metadata: Read-only` permissions—never write or push scopes.
5. **Credential Protection & Redaction:** Encrypt tokens at rest (`AES-256-GCM`), validate OAuth CSRF state, and scrub token patterns from all logs, errors, and exports.
6. **Untrusted Content Framing:** Never execute extracted code; frame untrusted content with explicit provenance delimiters to mitigate prompt injection.
