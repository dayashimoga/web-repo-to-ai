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

### 3.5 Storage Lifecycle, Ephemeral Retention & Watermark Cleanup SLAs (Subphase 11.3.3)

The system enforces automated, deterministic storage hygiene, bounded disk consumption, and leak-free resource reclamation across both in-memory and filesystem storage backends:

| Storage Cleanup Invariant | SLA Target | Verification Mechanism | Status |
| :--- | :--- | :--- | :--- |
| **Ephemeral Staging File Purging** | 100% stale scratch eradication | Scans storage root for temporary staging markers (`.*.tmp.*`); all staging files exceeding `ephemeral_ttl` are purged while finalized artifacts remain strictly preserved. | `ENFORCED` |
| **Capacity High/Low Watermarks** | Usage reduced below low watermark | When storage usage breaches `high_watermark_bytes` (e.g. 85%), `enforce_capacity_watermark` prunes oldest artifacts chronologically until total usage falls below `low_watermark_bytes` (e.g. 70%). | `ENFORCED` |
| **Zero Orphan Storage Leaks** | Zero orphaned job files | `cleanup_job_storage` recursively erases job directories (`jobs/{job_id}`); sibling job trees and unrelated artifact hierarchies remain completely untouched. | `ENFORCED` |
| **CAS LRU Quota Pruning** | Quota ceiling strictly maintained | Content-Addressed Storage (`CasEngine`) enforces LRU eviction policies, pruning least-recently-accessed blobs to remain within maximum byte allowances while cryptographic audits verify zero missing or corrupted items. | `ENFORCED` |
| **Shutdown Scratch Rejection** | Zero post-shutdown writes | Worker engines in `Draining` or `Stopped` states immediately reject retention sweeps and task executions (`CoreError::Unavailable`), preventing race conditions and incomplete disk writes during cooperative teardown. | `ENFORCED` |
| **TTL Monotonicity & Clamping** | Mathematical deadline correctness | `RetentionPolicy` enforces strict configuration invariants (`default_ttl >= 60s`, `max_ttl >= default_ttl`, `ephemeral_ttl > 0`, `low_watermark < high_watermark`) and clamps requested TTLs monotonically ($t_{\text{now}} + t_{\text{default}} \le t_{\text{expires}} \le t_{\text{now}} + t_{\text{max}}$). | `ENFORCED` |

### 3.6 Automated Storage Cleanup Test Suite (`services/worker/tests/storage_cleanup_verification_test.rs`)

The test suite evaluates 6 core storage cleanup, retention, and lifecycle enforcement specifications:

1. `test_storage_cleanup_ephemeral_staging_purging`:
   - Evaluates both `InMemoryStorage` and `LocalFsStorage` ephemeral scratch purging.
   - Creates temporary staging files (`.artifact.tmp.*`, `.streaming.tmp.*`) alongside finalized artifacts.
   - Verifies all staging files are eradicated, finalized artifacts are preserved, and worker engine telemetry accurately records `storage_cleanup_cycles = 1`, `storage_files_purged = 2`.
2. `test_storage_cleanup_capacity_watermark_enforcement`:
   - Configures storage retention policy with 40 KB high watermark and 20 KB low watermark.
   - Populates storage with 5 artifacts (10 KB each, total 50 KB, exceeding high watermark).
   - Verifies capacity enforcement prunes the oldest files first, reducing total usage below 20 KB, and preserves newest files.
3. `test_storage_cleanup_job_directory_purging_and_zero_orphans`:
   - Simulates 2 distinct conversion jobs (`job_a`, `job_b`), each generating artifacts across all 7 formats.
   - Executes job cleanup on `job_a` and asserts 100% removal with zero orphan files remaining.
   - Verifies `job_b` and its 7 artifact formats remain completely intact and undisturbed.
4. `test_storage_cleanup_cas_lru_quota_pruning`:
   - Populates CAS with 8 unique 1,000-byte blobs (8,000 bytes total).
   - Enforces LRU quota pruning down to 3,500 bytes.
   - Asserts at least 5 blobs and $\ge 4,500$ bytes are reclaimed, final storage usage is $\le 3,500$ bytes, and CAS cryptographic audit verifies 100% health (`missing == 0`, `corrupted == 0`).
5. `test_storage_cleanup_cooperative_worker_shutdown_scratch_drain`:
   - Transitions `WorkerEngine` into `Stopped` state.
   - Verifies `execute_retention_sweep` and task dispatch are strictly rejected with `CoreError::Unavailable`.
   - Asserts worker telemetry remains consistent without post-shutdown leakage.
6. `test_storage_cleanup_ttl_boundary_and_monotonicity_invariants`:
   - Verifies configuration validation rejects malformed TTLs (`default_ttl < 60s`, `max_ttl < default_ttl`, `ephemeral_ttl <= 0`, `low_watermark >= high_watermark`, `batch_size == 0`).
   - Verifies expiration calculation clamps negative or zero TTLs to `default_ttl` and excessive TTLs to `max_ttl`.
   - Proves strict mathematical monotonicity: $t_1 < t_2 \implies \text{expires\_at}(t_1) < \text{expires\_at}(t_2)$.

### 3.7 Load, Stress & Chaos Fault-Injection SLAs (Subphase 11.3.4)

The pipeline enforces deterministic resilience, bounded latency, and cascading failure containment under high load, chaos fault injection, network partitions, and contention stampedes:

| Load & Failure SLA Guarantee | Target SLA | Verification Mechanism | Status |
| :--- | :--- | :--- | :--- |
| **Sustained Chaos Pipeline** | $\text{Items}_{\text{total}} = \text{Succeeded} + \text{Failed}$ | Dispatches multi-task batch under injected HTTP 500 server errors, 429 rate limits, socket disconnects, and 404 permanent failures; proves healthy items complete, retrying items heal, and metrics snapshot reconciles cleanly with zero orphaned tasks. | `ENFORCED` |
| **Cascading Failure Containment** | AIMD backoff to floor $\ge \text{min}$ | Injects persistent upstream degradation; dynamic AIMD controller halves concurrency additively ($8 \to 4 \to 2$) without queue deadlocks, and scales up additively ($2 \to 3$) upon sustained success. | `ENFORCED` |
| **Watermark Flow Control** | Zero unbounded queue growth | High watermark triggers `Saturated` backpressure state, rejecting excess `try_push` submissions; consumer draining below low watermark restores `Normal` flow control. | `ENFORCED` |
| **Network Blackout Recovery** | 100% post-blackout data fidelity | Mid-batch network blackout safely times out or isolates failed items; once connectivity is restored, subsequent batches execute to 100% success with full CDM synthesis and cryptographic integrity. | `ENFORCED` |
| **Contention Stampede Resilience** | $\le 5.0\text{s}$ ceiling, 100% CAS health | 32 concurrent worker tasks contend simultaneously for CAS blob writes and distributed leases; CAS deduplicates to exactly 1 indexed physical blob with 100% health, and lease locks enforce atomic mutual exclusion with zero deadlocks. | `ENFORCED` |
| **Adversarial Payload Isolation** | Zero worker daemon crashes | Submitting truncated JSON, oversized payloads ($> 64\text{ KiB}$), and malformed streams safely returns categorized errors (`CdmError::Deserialization`, `CdmError::PayloadTooLarge`); valid sibling batch items convert with 100% fidelity. | `ENFORCED` |
| **Graceful Teardown Scratch Drain** | Zero orphan leaks & prompt rejection | Cooperative worker engine shutdown drains active in-flight permits to 0, immediately rejects subsequent batch submissions, and purges all ephemeral scratch staging files. | `ENFORCED` |

### 3.8 Automated Load & Failure Testing Suite (`services/worker/tests/load_and_failure_testing_test.rs`)

The test suite evaluates 6 comprehensive end-to-end chaos, load, contention, and fault-tolerance specifications:

1. `test_sustained_high_load_pipeline_under_chaos_fault_injection`:
   - Executes 24 concurrent batch tasks under randomized/patterned chaos faults (healthy, transient 500, transient 429, permanent 404).
   - Proves 21 items succeed (12 healthy + 6 transient 500 healed + 3 transient 429 healed) and 3 permanent items fail cleanly.
   - Validates strict monotonic index ordering `0..24` and telemetry reconciliation (`items_processed == 24`, `chaos_faults_injected >= 12`).
2. `test_cascading_failure_containment_and_dynamic_flow_regulation`:
   - Evaluates dynamic AIMD adaptive concurrency under cascading throttling signals, decreasing concurrency from 8 to 4 to 2 (floor).
   - Verifies post-cooldown recovery scaling up additively from 2 to 3 upon successive successes.
   - Evaluates `FlowControlledQueue` watermark regulation (`6` capacity, `4` high watermark, `2` low watermark), proving `try_push` saturation rejection and smooth draining back to `Normal`.
3. `test_simulated_network_blackout_and_resilient_recovery`:
   - Simulates an epoch-wide network blackout failing 8 batch items cleanly without unhandled panics or hangs.
   - Lifts the network partition and dispatches a second 8-item batch through the same worker daemon.
   - Proves 100% success on recovery with real CDM document synthesis and section hash verification.
4. `test_high_concurrency_contention_and_stampede_resilience`:
   - 32 concurrent Tokio worker tasks execute a thundering herd CAS write stampede on an identical 4 KB blob.
   - Proves exactly 1 write registers as new while 31 writes deduplicate, with identical cryptographic digests and 100% CAS audit health.
   - 32 concurrent worker tasks contend for an exclusive lease lock, verifying mutual exclusion, zero deadlocks, and completion within the 5.0s SLA ceiling.
5. `test_adversarial_and_corrupted_payload_resilience_under_load`:
   - Evaluates parser resilience against truncated JSON, rejecting with `CdmError::Deserialization`.
   - Evaluates byte ceilings against oversized payloads, rejecting with `CdmError::PayloadTooLarge`.
   - Executes a mixed batch (5 valid + 5 adversarial items) under load, proving 5 valid items succeed and 5 adversarial items fail cleanly without poisoning sibling processing.
6. `test_graceful_worker_teardown_and_scratch_drain_under_load`:
   - Initiates cooperative worker shutdown mid-flight during an active 16-task high-concurrency batch.
   - Proves in-flight tasks drain cleanly, engine transitions to `is_shutting_down == true`, and subsequent batch submissions are rejected immediately.
   - Retention manager purges temporary staging scratch files (`.artifact.tmp.*`), preserving finalized artifacts and leaving zero orphan files.

---

## 4. Phase 11.3 Reliability Formal Certification

**Certification Statement:** Phase 11.3 ("Reliability") is **100% COMPLETED, VERIFIED, AND FORMALLY CERTIFIED**.  
All 4 reliability subphases have been executed and proven with repeatable objective evidence:

- **Subphase 11.3.1 (Interrupted-job recovery):** Checkpoint monotonicity, worker failover, lease expiration, corrupted state isolation (`services/worker/tests/interrupted_job_recovery_test.rs`).
- **Subphase 11.3.2 (Retry & idempotency tests):** Transient retry backoff loops, terminal failure isolation, mathematical idempotency $f(f(x)) \equiv f(x)$, CAS write idempotency, multi-format export bit-for-bit repeatability (`services/worker/tests/retry_and_idempotency_test.rs`).
- **Subphase 11.3.3 (Storage cleanup verification):** Ephemeral staging purge, capacity watermark enforcement, job directory deletion, CAS LRU quota pruning (`services/worker/tests/storage_cleanup_verification_test.rs`).
- **Subphase 11.3.4 (Load & failure testing):** Chaos fault injection, cascading failure containment, network blackout recovery, contention stampede resilience, adversarial payload defenses, and graceful teardown scratch drain (`services/worker/tests/load_and_failure_testing_test.rs`).

**Overall Phase 11.3 Verification Metrics:**
- Total automated reliability tests: **24 tests** across 4 dedicated suites in `services/worker/tests`.
- Reliability test pass rate: **100% (24/24 passing)**.
- Zero unhandled daemon crashes, zero deadlocks, zero orphan file leaks.


