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

