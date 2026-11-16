# GLOBAL ENGINEERING CONTRACT

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Enforcement Level:** NON-NEGOTIABLE / MANDATORY ACROSS ALL STAGES, PHASES, SUBPHASES, AND AGENTS  
**Status:** ACTIVE

---

## 1. Product Integrity Contract

1. **Zero Mock / Zero Placeholder Policy**:
   - No mock data, fake content, simulated functionality, `todo!()`, `unimplemented!()`, or placeholder UI controls are permitted in any completed subphase.
   - Every rendered UI control must perform its advertised action against real state or a verified API contract.
   - Never hardcode sample repositories, canned conversion outputs, fake token counts, or simulated progress bars.
2. **Real External Integrations**:
   - All webpage extraction, GitHub repository acquisition, DOM rendering, and storage operations must use real protocols, parsers, and source content.
3. **Explicit Operational States**:
   - The system must strictly distinguish between `UNSUPPORTED`, `FAILED`, `PARTIAL`, and `COMPLETED` operations in domain types, database records, API responses, and UI views.
4. **Evidence-Backed Completion**:
   - No subphase, phase, or stage may be marked `VERIFIED` or `ACCEPTED` without implementation, integration, automated test execution, and verifiable evidence recorded in `IMPLEMENTATION_LEDGER.md`.

---

## 2. Architecture Contract

1. **Rust-First Processing Engine**:
   - All core acquisition, HTML/code extraction, Canonical Context Document Model (CDM) construction, normalization, deduplication, token counting, chunking, and archive generation must be implemented in safe, idiomatic Rust.
2. **Strict Layer & Crate Boundaries**:
   - Frontend (`apps/web`), HTTP API (`services/api`), background worker (`services/worker`), isolated browser worker (`services/browser-worker`), and shared domain crates (`crates/*`) must maintain acyclic, explicitly defined dependency boundaries.
   - HTTP handlers in `services/api` must never execute synchronous heavy extraction or direct filesystem manipulation outside the `storage` and `job-engine` abstractions.
3. **Canonical Context Document Model (CDM)**:
   - Every extraction source (static web, dynamic web, public GitHub, private GitHub, PDF/OCR) must normalize into the versioned `cdm` schema before any optimization or export generator runs.
   - All exporters (Markdown, TXT, JSON, HTML, JSONL, ZIP) must consume the CDM as their single source of truth.
4. **Provider-Independent Abstractions**:
   - Storage (`LocalFsStorage`, `S3CompatibleStorage`) and job orchestration (`PostgresJobEngine`) must implement clean traits allowing self-hosted single-node deployment or distributed cloud deployment without altering extraction logic.
5. **Bounded Asynchronous Execution**:
   - All network I/O, child processes, and worker tasks must use bounded queues, explicit timeouts, cancellation tokens, and resumable state transitions.

---

## 3. Performance & Resource Contract

1. **Tiered Acquisition Strategy**:
   - Always attempt fast, streaming HTTP extraction (`reqwest` + `html5ever`/`scraper`) before invoking isolated browser rendering (`Playwright` + `Chromium`).
   - Browser rendering is invoked only when static extraction detects an empty/JS-shell document or when explicitly requested by user configuration.
2. **Bounded Concurrency & Memory Safety**:
   - Worker pools, concurrent HTTP fetches, and tree-sitter parsers must enforce configurable concurrency semaphores and per-job memory/size budgets.
   - Large downloads and ZIP/JSONL exports must stream directly to/from storage rather than buffering entire repositories in RAM.
3. **Content-Addressed Caching**:
   - Source blobs, normalized sections, and tokenized segments must be indexed by cryptographic content hash (`BLAKE3` / `SHA-256`) to prevent redundant work.
4. **Zero Mandatory External AI / Paid Dependencies**:
   - Extraction, normalization, deduplication, classification, and token optimization must execute locally on CPU without paid LLM APIs or paid scraping services.
5. **Hardware & Container Budget**:
   - Must operate reliably on a CPU-only machine with 16 GB RAM.
   - Total Podman storage footprint must never exceed **30 GB**.

---

## 4. Accuracy & Provenance Contract

1. **Exact Source Preservation**:
   - Source code files, code blocks, indentation, and symbols must be preserved byte-for-byte in lossless mode.
   - Webpage headings, hierarchical sections, tables, lists, and code blocks must retain structural semantics.
2. **End-to-End Provenance Tracking**:
   - Every CDM section and exported chunk must record its canonical source URL, relative file path, Git commit SHA/revision, byte/line ranges, and content hash.
3. **No Silent Truncation or Omission**:
   - Never silently truncate a webpage or skip an eligible repository file.
   - Every excluded, skipped, binary, oversized, or failed file/section must be recorded in the extraction manifest with an explicit machine-readable reason code and human-readable explanation.
4. **Completeness Verification Gate**:
   - A repository extraction job cannot transition to `COMPLETED` unless `discovered_files == extracted_files + excluded_files + failed_files + unsupported_files` and all eligible files are accounted for.
5. **Lossless vs. Lossy Separation**:
   - Lossless extraction and structural cleanup must be strictly separated from lossy context compression or budget truncation.

---

## 5. Security Contract

1. **Mandatory SSRF & DNS Rebinding Defense**:
   - Every outbound URL request must pass through `crates/url-security`.
   - Resolve DNS prior to connection, pin the validated IP address for the socket connection, and block loopback (`127.0.0.0/8`, `::1`), RFC1918 private ranges (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), link-local/cloud metadata (`169.254.0.0/16`, `fe80::/10`), carrier-grade NAT (`100.64.0.0/10`), multicast, and broadcast addresses.
   - Re-validate every HTTP redirect hop against the exact same URL and IP security policy with a strict maximum redirect limit.
2. **Browser Rendering Isolation**:
   - `services/browser-worker` runs in an isolated container/process boundary with no access to database credentials, GitHub tokens, or internal service networks, and enforces network interception to block private/local IP destinations from subresources.
3. **Zero Execution of Extracted Code**:
   - Extracted repository files, build scripts, macros, or webpage scripts from target repositories must never be executed on the host or worker.
4. **GitHub Credential & OAuth Security**:
   - Private repository access uses a GitHub App with strictly read-only permissions (`Contents: Read-only`, `Metadata: Read-only`).
   - OAuth flows require cryptographic state/CSRF validation, short-lived installation tokens, envelope encryption at rest (`AES-256-GCM` / `ChaCha20-Poly1305`), and automatic secret redaction in logs and exports.
5. **Untrusted Content & Prompt Injection Mitigation**:
   - All extracted web and repository text is treated as untrusted input.
   - Exporters wrap untrusted source content in unambiguous structural delimiters and flag known prompt-injection patterns in the quality/security report without corrupting lossless source code.
6. **Rate, Size, and Retention Limits**:
   - Enforce per-IP/user rate limits, maximum download bytes, maximum decompression ratios (zip-bomb defense), job execution timeouts, and automated artifact TTL cleanup.

---

## 6. Code Quality & Dependency Contract

1. **Zero Unjustified Warnings or Debt**:
   - `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `npm run check`, and `npm run lint` must pass cleanly.
   - `#[allow(...)]` or linter suppressions require an inline justification comment.
2. **Strict Error Handling**:
   - No `.unwrap()` or `.expect()` in production library/service code paths outside statically provable invariants or tests.
   - No swallowed errors (`let _ = ...`) or silent fallback to empty defaults when an operation fails.
3. **Dependency & License Hygiene**:
   - Every dependency must be evaluated for active maintenance, known CVEs (`cargo audit` / `cargo deny` / `npm audit`), and license compatibility.
   - Prefer `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unicode-3.0`, and `PostgreSQL` licenses.
   - Copyleft (`GPL`, `AGPL`, `LGPL`), source-available, non-commercial, or unknown licenses are blocked unless explicitly reviewed and isolated.

---

## 7. Execution & Documentation Discipline

1. **One Subphase at a Time**:
   - Follow `Stage → Phase → Subphase → Tasks → Verification`.
   - Never combine subphases or start the next subphase without explicit user approval.
2. **Append-Only Historical Records**:
   - `CHANGELOG.md`, `TODO.md`, and `IMPLEMENTATION_LEDGER.md` preserve complete historical audit trails; never erase prior entries or completion evidence.
