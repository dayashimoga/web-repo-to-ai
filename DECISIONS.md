# ARCHITECTURE DECISION RECORDS (`DECISIONS.md`)

**Project:** `web-repo-to-ai`  
**Format:** Sequential Architecture Decision Records (ADR)

---

## ADR-001: Monorepo Architecture with Rust Processing Engine and SvelteKit Frontend

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** The platform requires high-throughput, memory-safe HTML/code parsing, deterministic token counting, and streaming archive generation on a CPU-only 16 GB RAM budget, paired with a modern, accessible, responsive web UI.
- **Decision:** Organize the repository as a single monorepo containing:
  - `apps/web`: SvelteKit + Svelte 5 + TypeScript + Tailwind CSS frontend.
  - `services/api` and `services/worker`: Rust services built on Axum, Tokio, Reqwest, and SQLx.
  - `services/browser-worker`: Isolated Playwright + Chromium dynamic rendering service.
  - `crates/*`: 15 focused, independently testable Rust library crates.
- **Consequences:** Shared domain models (`crates/core`, `crates/cdm`) are enforced at compile time across the API, worker, CLI, and MCP server without microservice serialization overhead.

---

## ADR-002: Canonical Context Document Model (CDM) as Single Intermediate Representation

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** Extracting from diverse sources (static webpages, JS-rendered SPAs, public/private GitHub repos, PDFs) and exporting to multiple formats (Markdown, TXT, JSON, HTML, JSONL, ZIP) risks $M \times N$ ad-hoc conversion paths and inconsistent provenance tracking.
- **Decision:** Define a versioned Canonical Context Document Model (`crates/cdm`, starting at `v1.0`) as the mandatory hub. Every extractor outputs a `CdmDocument`; every optimizer mutates/annotates a `CdmDocument` with full audit trails; every exporter serializes from a `CdmDocument`.
- **Consequences:** Provenance, content hashes, exclusion manifests, and completeness checks are guaranteed uniformly across all input types and output formats.

---

## ADR-003: Tiered Web Acquisition (Static HTTP First, Isolated Browser Worker Fallback)

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** Headless browser rendering consumes 10x–50x more CPU and RAM than streaming HTTP + HTML parsing and expands the security attack surface.
- **Decision:** Always perform SSRF-validated HTTP fetching (`reqwest`) and static DOM extraction (`html5ever`/`scraper`) first. Invoke `services/browser-worker` (Playwright + Chromium) only when static extraction detects a client-rendered shell (e.g., empty root container, noscript hydration requirement) or when requested by the user.
- **Consequences:** Most documentation sites, blogs, articles, and SSR web apps convert in milliseconds with minimal memory, preserving the 16 GB RAM / CPU-only budget while still supporting JS-heavy apps.

---

## ADR-004: Defense-in-Depth SSRF & DNS Rebinding Protection in `crates/url-security`

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** Any service that fetches arbitrary user-supplied URLs is a prime target for Server-Side Request Forgery (SSRF), cloud metadata theft (`169.254.169.254`), internal port scanning, and DNS rebinding attacks.
- **Decision:** Implement a dedicated `crates/url-security` crate that:
  1. Validates scheme (`http` or `https` only), port allowlist, and host syntax.
  2. Performs custom DNS resolution and validates all returned IPv4/IPv6 addresses against a strict blocklist (loopback, RFC1918, link-local, CGNAT, multicast, broadcast, unspecified, IPv4-mapped IPv6 private ranges).
  3. Pins the validated IP address in the `reqwest` custom DNS resolver / socket connector so TOCTOU DNS rebinding is impossible.
  4. Disables automatic HTTP redirects in `reqwest` and manually re-runs the full URL + DNS + IP validation on every `3xx` redirect hop (up to a bounded max hop count).
- **Consequences:** Eliminates both direct SSRF and time-of-check-to-time-of-use (TOCTOU) DNS rebinding across static fetching and browser worker navigation.

---

## ADR-005: PostgreSQL-Backed Job Leasing and External Artifact Storage

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** Storing multi-megabyte repository archives inside PostgreSQL rows bloats WAL logs and degrades query performance, while introducing Redis/RabbitMQ increases operational complexity and container footprint.
- **Decision:** Use PostgreSQL (`FOR UPDATE SKIP LOCKED` with lease expiration timestamps and worker heartbeats) for durable job queuing and metadata, and store all extracted CDM documents, cached blobs, and export bundles outside PostgreSQL via `crates/storage` (`LocalFsStorage` for local/single-node deployments and `S3CompatibleStorage` for S3/MinIO/Cloudflare R2).
- **Consequences:** Zero extra message-broker containers required initially, crash-resilient job recovery out of the box, and fast streaming artifact I/O.

---

## ADR-006: Read-Only GitHub App Authentication with Envelope Token Encryption

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** Users converting private repositories need strong assurance that `web-repo-to-ai` cannot modify their code, access unauthorized repositories, or leak tokens.
- **Decision:** Use a GitHub App (never broad personal OAuth scopes like `repo`) requesting strictly `Contents: Read-only` and `Metadata: Read-only` on user-selected repositories. Exchange installation IDs for short-lived installation tokens server-side, encrypt stored credentials with `AES-256-GCM`, and enforce automated token pattern scrubbing in `crates/observability` and `crates/exporters`.
- **Consequences:** Adheres strictly to least privilege and prevents credential exposure even in the event of log inspection or export sharing.

---

## ADR-007: Permissive Open-Source Dependency Policy & 30 GB Podman Storage Ceiling

- **Date:** 2026-11-16
- **Status:** Accepted
- **Context:** The project must remain open-source, self-hostable without commercial lock-in, and operable within a 30 GB container disk ceiling on Windows/Linux development hosts.
- **Decision:** Enforce automated license verification (`cargo deny` / license inventory scripts) allowing permissive licenses (`MIT`, `Apache-2.0`, `BSD`, `ISC`, `PostgreSQL`, `Unicode-3.0`) and flagging copyleft/proprietary dependencies. Use slim multi-stage container builds and provide scripts in `scripts/` to monitor and prune Podman build caches safely below 30 GB.
- **Consequences:** Predictable legal posture for open-source distribution and reliable local container operations within fixed disk limits.
