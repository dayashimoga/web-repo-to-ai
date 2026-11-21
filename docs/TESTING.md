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

---

## 3. Reliability & Interrupted-Job Recovery Verification (Phase 11.3)

### 3.1 Reliability SLAs & Failure Invariants

The conversion pipeline enforces strict mathematical and operational invariants across unexpected host failures, worker process crashes, network partitions, and lease expirations:

| Reliability Invariant | SLA Target | Verification Mechanism | Status |
| :--- | :--- | :--- | :--- |
| **Durable Checkpoint Integrity** | 100% cryptographic validation | Canonical BLAKE3 digest verification over state metadata, item results, and completed indices; bit rot / tamper detection via `JobEngineError::CheckpointCorrupted`. | `ENFORCED` |
| **Zero-Duplication on Resume** | 0 re-executions of completed items | Partitioning via `ResumableExecutionPlan`; items in `cp.completed_indices` bypass worker processing and are resolved directly from checkpoint state. | `ENFORCED` |
| **Mathematical Completeness Conservation** | $\text{Items}_{\text{total}} = \text{Items}_{\text{reused}} + \text{Items}_{\text{executed}}$ | Strict conservation across single or cascading crashes; final output vector contains all $N$ elements in sequence. | `ENFORCED` |
| **Distributed Lease Failover** | Automatic stale reclamation | Row-level locking (`FOR UPDATE SKIP LOCKED`) with expiration deadlines (`lease_expires_at`); expired leases safely requeued or failed after `max_attempts`. | `ENFORCED` |
| **Corrupted State Quarantine** | Zero deadlocks / zero unhandled panics | Graceful error propagation on checksum mismatch, allowing quarantine deletion and fresh recovery without daemon disruption. | `ENFORCED` |
| **Cooperative Cancellation Drainage** | Clean shutdown & partial state preservation | `CancellationToken` drainage captures active and completed permits into persistent store prior to exit, allowing seamless resumption. | `ENFORCED` |
| **Streaming Export Package Parity** | 100% artifact & checksum fidelity | Synthesized multi-document archives from resumed runs produce identical ZIP packages, coreutils SHA-256 manifests, and package manifests. | `ENFORCED` |

### 3.2 Automated Recovery Test Suite (`services/worker/tests/interrupted_job_recovery_test.rs`)

The test suite exercises real pipeline components without mocks or stubs:

1. `test_interrupted_batch_worker_crash_and_resumption`:
   - Simulates mid-batch worker crash after 5 items checkpointed out of 12.
   - Replacement worker boots up, reuses 5 items, executes only remaining 7 items.
   - Verifies 100% completion (12 items), correct sequence ordering, and worker metrics snapshot (`resumed_jobs_count == 1`, `items_reused_from_checkpoint == 5`).
2. `test_multi_stage_cascading_worker_crashes_and_resumption`:
   - Simulates cascading multi-stage crashes across 3 worker generations on a 20-item batch.
   - Worker 1 processes 6 items and crashes; Worker 2 resumes (6 reused, 7 processed, total 13) and crashes; Worker 3 resumes (13 reused, 7 processed) to 100% completion.
   - Proves state monotonicity and cumulative checkpoint progression across generations.
3. `test_interrupted_job_tampered_or_corrupted_checkpoint_recovery`:
   - Simulates bit rot or partial disk write mid-crash corrupting checkpoint data.
   - Verifies `JobEngineError::CheckpointCorrupted` is returned with `verify_checksum_on_load: true`.
   - Exercises quarantine deletion workflow and verifies successful fresh re-execution.
4. `test_distributed_lease_expiration_and_worker_failover`:
   - Evaluates distributed lease lifecycle (`Queued` $\to$ `Leased` $\to$ `Running` $\to$ `Queued` (requeue) $\to$ `Leased` $\to$ `Running` $\to$ `Completed`).
   - Verifies worker lease validity probing, stale lease recovery sweep, attempt count bounding, and permanent failure threshold (`attempt_count >= max_attempts`).
5. `test_interrupted_batch_cancellation_and_safe_drain`:
   - Evaluates cooperative token cancellation mid-batch.
   - Verifies in-flight cancellation drainage, partial checkpoint persistence, and subsequent resumption of cancelled items to 100% success.
6. `test_interrupted_job_cdm_synthesis_and_export_integrity`:
   - Evaluates end-to-end CDM document assembly, section cryptographic hashing, and streaming ZIP package creation across interrupted runs.
   - Verifies archive contains 6 document directories with matching BLAKE3/SHA-256 manifests.

### 3.3 Retry Dynamics & Mathematical Idempotency SLAs (Subphase 11.3.2)

The system guarantees deterministic fault-tolerance, transient failure recovery, and mathematical idempotency $f(f(x)) \equiv f(x)$ across all pipeline layers:

| Idempotency & Retry Guarantee | Target SLA | Verification Mechanism | Status |
| :--- | :--- | :--- | :--- |
| **Transient Error Auto-Recovery** | Automatic backoff recovery | Bounded parallel executor automatically retries failing batch items up to `max_retries` with exponential backoff (`retry_backoff * attempt`); items healing mid-retry achieve 100% success. | `ENFORCED` |
| **Backoff Exhaustion Isolation** | Zero whole-batch cascade | Failing items exceeding `max_retries` deterministically transition to `BatchItemStatusOutcome::Failed` with original upstream error preserved while healthy batch peers complete normally. | `ENFORCED` |
| **Text Normalization Idempotence** | $f(f(x)) \equiv f(x)$ exact identity | `normalize_text(normalize_text(s)) == normalize_text(s)` across CRLF/LF, BOM stripping, duplicate spacing, and control char sanitation. | `ENFORCED` |
| **Document Normalization Idempotence** | Zero secondary modifications | `normalize_document(&doc_norm_1)` yields `sections_modified == 0`, preserving document tree structure and cryptographic section hashes. | `ENFORCED` |
| **Document Deduplication Idempotence** | Zero secondary cluster discoveries | `deduplicate_document(&doc_dedup_1)` yields `duplicate_sections_found == 0` and bit-for-bit identical section vectors on repeated passes. | `ENFORCED` |
| **CAS Multi-Write Idempotency** | $O(1)$ physical storage growth | Sequential and concurrent multi-threaded writes of identical blobs produce identical BLAKE3/SHA-256 addresses with `is_new == false` and zero disk bloat. | `ENFORCED` |
| **Multi-Format Export Determinism** | Bit-for-bit identical byte streams | Repeated exports of CDM documents across Markdown, Plain Text, JSON v1.0, HTML5, and streaming ZIP packages produce invariant cryptographic digests and identical CRC32 checksums. | `ENFORCED` |
| **State Machine Immutability** | Terminal state lockdown | `evaluate_transition` rejects any mutation from `Completed`, `Failed`, or `Cancelled` states, guaranteeing state machine immutability. | `ENFORCED` |

### 3.4 Automated Retry & Idempotency Test Suite (`services/worker/tests/retry_and_idempotency_test.rs`)

The test suite evaluates 6 core reliability and idempotency specifications:

1. `test_transient_failure_automatic_retry_and_recovery`:
   - Configures worker pool with `max_retries = 3` and fast backoff.
   - Tasks 1, 3, and 5 simulate transient connection drops on attempts 0 and 1, healing on attempt 2.
   - Verifies all 6 items complete with `BatchItemStatusOutcome::Succeeded`, preserves sequence order, and records telemetry.
2. `test_retry_backoff_exhaustion_deterministic_terminal_failure`:
   - Configures worker pool with `max_retries = 2`.
   - Task 2 permanently fails with HTTP 404, while tasks 0, 1, 3, 4 succeed immediately.
   - Verifies task 2 executes exactly 3 times (1 initial + 2 retries) and records terminal failure without failing or aborting healthy sibling items.
3. `test_document_normalization_and_deduplication_mathematical_idempotency`:
   - Verifies $f(f(x)) \equiv f(x)$ for `normalize_text` (reaches immediate fixed point).
   - Verifies $f(f(x)) \equiv f(x)$ for `normalize_document` (pass 2 performs 0 section modifications).
   - Verifies $f(f(x)) \equiv f(x)$ for `deduplicate_document` (pass 2 discovers 0 new duplicates).
4. `test_content_addressed_storage_and_cache_write_idempotency`:
   - Verifies `cas.put(payload)` returns `is_new: true` on initial write and `is_new: false` on duplicate write.
   - Verifies 10 concurrent Tokio tasks writing identical payloads produce matching digests and zero corruption.
   - Verifies CAS audit confirms healthy storage with exactly 1 indexed blob.
5. `test_multi_format_export_bit_for_bit_deterministic_idempotency`:
   - Evaluates 5 consecutive export iterations across Markdown, Plain Text, JSON, and HTML.
   - Verifies cryptographic BLAKE3 hashes remain 100% invariant across all iterations.
   - Verifies streaming ZIP archive exports from identical documents produce matching file entry names, sizes, and CRC32 checksums.
6. `test_state_machine_transition_and_lease_renewal_idempotency`:
   - Evaluates distributed leasing lifecycle transitions: `Queued` $\to$ `Leased` $\to$ `Running` $\to$ `Completed`.
   - Verifies terminal immutability: all transitions originating from `Completed`, `Failed`, or `Cancelled` are strictly rejected.
   - Verifies `LeaseConfig` boundary invariants (worker identity, duration ceilings, heartbeat windows, attempt thresholds).

