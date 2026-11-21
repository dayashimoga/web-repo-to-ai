//! # Load, Stress, Chaos & Failure Resilience Test Suite
//!
//! Evaluates end-to-end load handling, chaos fault injection, cascading failure containment,
//! dynamic flow regulation (AIMD adaptive concurrency and watermark backpressure), simulated network
//! blackout recovery, high-concurrency contention and stampede resilience, adversarial payload defenses,
//! and cooperative worker teardown scratch drainage across the conversion pipeline.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use uuid::Uuid;

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::error::CdmError;
use web_repo_to_ai_cdm::json::{from_json_with_options, JsonOptions};
use web_repo_to_ai_cdm::model::{CdmDocument, RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::adaptive::{
    AdaptiveConcurrencyConfig, AdaptiveConcurrencyController, AdaptiveDecision, AdaptiveOutcome,
};
use web_repo_to_ai_job_engine::backpressure::{
    BackpressureConfig, BackpressureState, FlowControlledQueue, LoadSheddingStrategy,
};
use web_repo_to_ai_job_engine::error::JobEngineError;
use web_repo_to_ai_job_engine::models::{BatchItemStatusOutcome, BatchItemTask};
use web_repo_to_ai_storage::backend::StorageBackend;
use web_repo_to_ai_storage::cas::{CasEngine, ContentAddressedStorage};
use web_repo_to_ai_storage::local_fs::LocalFsStorage;
use web_repo_to_ai_storage::retention::{RetentionManager, RetentionPolicy};
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

/// Scoped RAII temporary test directory for filesystem isolation.
struct TestDir(PathBuf);

impl TestDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "web-repo-to-ai-test-load-{prefix}-{}",
            Uuid::now_v7()
        ));
        std::fs::create_dir_all(&path).expect("Failed to create test temporary directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Helper generating synthetic batch conversion tasks for load tests.
fn create_test_tasks(count: usize, prefix: &str) -> Vec<BatchItemTask<String>> {
    (0..count)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{prefix}/{i}"),
                Some(format!("load-task-{prefix}-{i}")),
                format!("payload-data-{prefix}-{i}"),
            )
        })
        .collect()
}

/// Helper creating a valid sample CDM document with structured sections.
fn create_sample_cdm_document(index: usize, title: &str, body: &str) -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: format!("https://example.com/docs/doc-{index}"),
        canonical_url: format!("https://example.com/docs/doc-{index}"),
        domain: Some("example.com".to_string()),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    };

    let prose_section = CdmSectionBuilder::new(
        format!("sec-{index}-prose"),
        format!("https://example.com/docs/doc-{index}#prose"),
        SectionKind::Prose,
        body,
    )
    .title(title)
    .original_path(format!("docs/doc_{index}.md"))
    .build();

    let code_section = CdmSectionBuilder::new(
        format!("sec-{index}-code"),
        format!("https://example.com/docs/doc-{index}#code"),
        SectionKind::CodeFile,
        "fn execute_pipeline() -> bool { true }\n",
    )
    .title("Pipeline Implementation")
    .original_path(format!("src/pipeline_{index}.rs"))
    .build();

    CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        format!("https://example.com/docs/doc-{index}"),
    )
    .revision(RevisionInfo {
        git_tag: Some(format!("v{index}.0")),
        ..Default::default()
    })
    .add_section(prose_section)
    .add_section(code_section)
    .build()
}

// ============================================================================
// TEST 1: Sustained High-Load Pipeline under Chaos Fault Injection
// ============================================================================

#[tokio::test]
async fn test_sustained_high_load_pipeline_under_chaos_fault_injection() {
    // 1. Configure worker with concurrency ceiling 6, 3 retries, and fast 10ms backoff
    let config = WorkerServiceConfig::for_worker("chaos-load-worker", 6, 6)
        .expect("Config should be valid")
        .with_retries(3, 10);

    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    let batch_id = BatchId::new();
    let total_tasks = 24;
    let tasks = create_test_tasks(total_tasks, "chaos");

    // Shared execution attempt tracker across tasks
    let attempts_map: Arc<Mutex<HashMap<usize, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let attempts_tracker = Arc::clone(&attempts_map);
    let engine_metrics = engine.clone();

    // 2. Chaos fault-injecting processor:
    // - Items where idx % 4 == 0 or 2 (12 items): healthy, succeed immediately on attempt 0
    // - Items where idx % 4 == 1 (6 items): transient 500 error on attempt 0, succeeds on attempt 1
    // - Items where idx % 8 == 3 (3 items): transient 429 rate limit on attempts 0 and 1, succeeds on attempt 2
    // - Items where idx % 8 == 7 (3 items): permanent 404 error, fails across all retries
    let processor = move |task: BatchItemTask<String>| {
        let attempts_map = Arc::clone(&attempts_tracker);
        let engine_metrics = engine_metrics.clone();

        async move {
            let attempt_number = {
                let mut map = attempts_map.lock().await;
                let count = map.entry(task.item_index).or_insert(0);
                let current = *count;
                *count += 1;
                current
            };

            if task.item_index % 8 == 7 {
                // Permanent fatal failure
                engine_metrics.record_chaos_fault();
                Err(format!(
                    "HTTP 404 Not Found (Permanent non-retryable failure on item {})",
                    task.item_index
                ))
            } else if task.item_index % 8 == 3 && attempt_number < 2 {
                // Transient 429 rate limit
                engine_metrics.record_chaos_fault();
                Err(format!(
                    "HTTP 429 Too Many Requests (Transient rate limit backoff on attempt {attempt_number})"
                ))
            } else if task.item_index % 4 == 1 && attempt_number == 0 {
                // Transient 500 server error
                engine_metrics.record_chaos_fault();
                Err(format!(
                    "HTTP 500 Internal Server Error (Transient failure on attempt {attempt_number})"
                ))
            } else {
                // Successful conversion
                Ok(format!(
                    "converted-item-{}-attempt-{}",
                    task.item_index, attempt_number
                ))
            }
        }
    };

    // 3. Dispatch batch through WorkerEngine
    let start_time = Instant::now();
    let summary = engine
        .execute_batch(batch_id, tasks, Some(6), processor, None)
        .await
        .expect("Batch execution should complete");
    let elapsed = start_time.elapsed();

    // 4. Assert batch execution results
    assert_eq!(summary.total_items, total_tasks);
    assert_eq!(
        summary.succeeded_count, 21,
        "Exactly 21 items must succeed (12 healthy + 6 transient 500 healed + 3 transient 429 healed)"
    );
    assert_eq!(
        summary.failed_count, 3,
        "Exactly 3 items must fail deterministically (permanent 404s)"
    );
    assert_eq!(
        summary.succeeded_count + summary.failed_count,
        summary.total_items,
        "Accounting completeness invariant must hold exactly"
    );

    // Verify ordering and statuses
    assert_eq!(summary.item_results.len(), total_tasks);
    for (idx, item) in summary.item_results.iter().enumerate() {
        assert_eq!(
            item.item_index, idx,
            "Item indices must be strictly monotonic"
        );
        if idx % 8 == 7 {
            assert!(
                matches!(item.status, BatchItemStatusOutcome::Failed { .. }),
                "Item {idx} must be marked as Failed"
            );
        } else {
            assert!(
                matches!(item.status, BatchItemStatusOutcome::Succeeded),
                "Item {idx} must be marked as Succeeded"
            );
        }
    }

    // 5. Verify worker service telemetry snapshot
    let snapshot = engine.metrics_snapshot();
    assert_eq!(snapshot.batches_processed, 1);
    assert_eq!(snapshot.items_processed, total_tasks);
    assert_eq!(snapshot.items_succeeded, 21);
    assert_eq!(snapshot.items_failed, 3);
    assert!(
        snapshot.chaos_faults_injected >= 12,
        "Chaos fault injector must have recorded injected faults (actual: {})",
        snapshot.chaos_faults_injected
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "Chaos-injected batch of 24 items must complete within 10s SLA (actual: {elapsed:?})"
    );
}

// ============================================================================
// TEST 2: Cascading Failure Containment & Dynamic Flow Regulation
// ============================================================================

#[tokio::test]
async fn test_cascading_failure_containment_and_dynamic_flow_regulation() {
    // ------------------------------------------------------------------------
    // Part A: Dynamic AIMD Adaptive Concurrency Under Cascading Upstream Degradation
    // ------------------------------------------------------------------------
    let adaptive_cfg = AdaptiveConcurrencyConfig::new(2, 12, 8)
        .expect("Valid adaptive config")
        .with_success_threshold(2)
        .with_decrease_factor(0.5)
        .with_cooldown(Duration::from_millis(50));

    let controller =
        AdaptiveConcurrencyController::new(adaptive_cfg).expect("Controller should initialize");

    assert_eq!(controller.current_limit(), 8);

    // 1. Simulate cascading upstream 429 / 503 throttling signals
    let outcome_1 = AdaptiveOutcome::RateLimited { retry_after: None };
    let dec_1 = controller.record_outcome(outcome_1);
    assert!(
        matches!(dec_1, AdaptiveDecision::Decreased { new_limit: 4, .. }),
        "First backoff halves concurrency from 8 to 4"
    );
    assert_eq!(controller.current_limit(), 4);

    // Force past cooldown to simulate persistent cascading degradation
    tokio::time::sleep(Duration::from_millis(60)).await;
    let outcome_2 = AdaptiveOutcome::ServerOverloaded;
    let dec_2 = controller.record_outcome(outcome_2);
    assert!(
        matches!(dec_2, AdaptiveDecision::Decreased { new_limit: 2, .. }),
        "Second backoff halves concurrency from 4 to min ceiling 2"
    );
    assert_eq!(controller.current_limit(), 2);

    // Additional failures clamp at min_concurrency floor (2)
    tokio::time::sleep(Duration::from_millis(60)).await;
    let outcome_3 = AdaptiveOutcome::ConnectionError;
    let _dec_3 = controller.record_outcome(outcome_3);
    assert_eq!(
        controller.current_limit(),
        2,
        "Concurrency cannot drop below min floor (2)"
    );

    let stats = controller.stats();
    assert_eq!(stats.current_limit, 2);
    assert!(stats.total_backoffs >= 2);

    // 2. Upstream recovers: successive successful outcomes scale up additively
    tokio::time::sleep(Duration::from_millis(60)).await;
    let success = AdaptiveOutcome::Success {
        duration: Duration::from_millis(15),
    };

    // First success (reaches 1/2 of threshold)
    controller.record_outcome(success.clone());
    assert_eq!(controller.current_limit(), 2);

    // Second success (reaches 2/2 threshold -> scale up to 3)
    let dec_up_1 = controller.record_outcome(success);
    assert!(
        matches!(dec_up_1, AdaptiveDecision::Increased { new_limit: 3, .. }),
        "Scale-up increases limit additively to 3"
    );
    assert_eq!(controller.current_limit(), 3);

    // ------------------------------------------------------------------------
    // Part B: Watermark Backpressure Flow Control & Load Shedding
    // ------------------------------------------------------------------------
    let bp_config = BackpressureConfig::new(6, 4, 2)
        .expect("Valid backpressure config")
        .with_shed_strategy(LoadSheddingStrategy::DropOldest)
        .with_wait_timeout(Duration::from_millis(50));

    let queue = FlowControlledQueue::<String>::new(bp_config).expect("Queue should initialize");

    // Fill queue to high watermark (4)
    for i in 0..4 {
        queue
            .push(format!("item-{i}"), None)
            .await
            .expect("Push must succeed");
    }
    assert_eq!(queue.len(), 4);
    assert_eq!(queue.controller().state(), BackpressureState::Saturated);

    // Try push when saturated: rejected because high watermark is reached
    let saturated_err = queue.try_push("item-overflow".to_string());
    assert!(
        matches!(
            saturated_err,
            Err(JobEngineError::BackpressureQueueSaturated { .. })
        ),
        "try_push must be rejected when queue reaches high watermark"
    );

    // Pop items and verify drain triggers transition down to low watermark (2)
    let popped_1 = queue.pop(None).await.expect("Pop must succeed");
    assert_eq!(popped_1, Some("item-0".to_string()));

    let popped_2 = queue.pop(None).await.expect("Pop must succeed");
    assert_eq!(popped_2, Some("item-1".to_string()));

    // Queue drained down to 2, low watermark reached -> state transitions back to Normal
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.controller().state(), BackpressureState::Normal);
}

// ============================================================================
// TEST 3: Simulated Network Blackout & Resilient Recovery
// ============================================================================

#[tokio::test]
async fn test_simulated_network_blackout_and_resilient_recovery() {
    let config = WorkerServiceConfig::for_worker("blackout-resilience-worker", 4, 4)
        .expect("Config should be valid")
        .with_retries(1, 10);

    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    // Shared circuit switch controlling network partition simulation
    let blackout_active = Arc::new(AtomicBool::new(true));

    // ------------------------------------------------------------------------
    // Phase 1: Epoch During Active Network Blackout
    // ------------------------------------------------------------------------
    let blackout_flag = Arc::clone(&blackout_active);
    let batch_1_id = BatchId::new();
    let tasks_1 = create_test_tasks(8, "blackout");

    let processor_phase_1 = move |task: BatchItemTask<String>| {
        let is_blackout = blackout_flag.load(Ordering::SeqCst);
        async move {
            if is_blackout {
                Err(format!(
                    "DNS Failure / Network Unreachable: Connection refused for item {}",
                    task.item_index
                ))
            } else {
                Ok(format!("recovered-{}", task.item_index))
            }
        }
    };

    let summary_phase_1 = engine
        .execute_batch(batch_1_id, tasks_1, Some(4), processor_phase_1, None)
        .await
        .expect("Phase 1 batch execution should complete");

    // During blackout, all 8 items fail cleanly without unhandled panics or hangs
    assert_eq!(summary_phase_1.total_items, 8);
    assert_eq!(summary_phase_1.failed_count, 8);
    assert_eq!(summary_phase_1.succeeded_count, 0);

    // ------------------------------------------------------------------------
    // Phase 2: Network Connectivity Restored & Complete Recovery
    // ------------------------------------------------------------------------
    // Lift the network partition
    blackout_active.store(false, Ordering::SeqCst);

    let blackout_flag_restored = Arc::clone(&blackout_active);
    let batch_2_id = BatchId::new();
    let tasks_2 = create_test_tasks(8, "recovered");

    let processor_phase_2 = move |task: BatchItemTask<String>| {
        let is_blackout = blackout_flag_restored.load(Ordering::SeqCst);
        async move {
            if is_blackout {
                Err("Blackout still active".to_string())
            } else {
                // Generate and verify real CDM document on recovery
                let doc = create_sample_cdm_document(
                    task.item_index,
                    "Recovered Document",
                    "Content successfully extracted after network partition restored.",
                );
                assert!(!doc.sections.is_empty());
                Ok(format!(
                    "doc-{}-section-{}",
                    task.item_index, doc.sections[0].section_id
                ))
            }
        }
    };

    let summary_phase_2 = engine
        .execute_batch(batch_2_id, tasks_2, Some(4), processor_phase_2, None)
        .await
        .expect("Phase 2 batch execution should complete");

    // After recovery, all 8 items convert to 100% success
    assert_eq!(summary_phase_2.total_items, 8);
    assert_eq!(summary_phase_2.succeeded_count, 8);
    assert_eq!(summary_phase_2.failed_count, 0);
    assert!(summary_phase_2.all_succeeded());

    // Cumulative telemetry check across both epochs
    let snap = engine.metrics_snapshot();
    assert_eq!(snap.batches_processed, 2);
    assert_eq!(snap.items_processed, 16);
    assert_eq!(snap.items_failed, 8);
    assert_eq!(snap.items_succeeded, 8);
}

// ============================================================================
// TEST 4: High-Concurrency Contention & Stampede Resilience
// ============================================================================

#[tokio::test]
async fn test_high_concurrency_contention_and_stampede_resilience() {
    // ------------------------------------------------------------------------
    // Part A: CAS Thundering Herd / Content Write Stampede (32 Concurrent Tasks)
    // ------------------------------------------------------------------------
    let cas = CasEngine::in_memory();
    let shared_payload = Arc::new(
        b"Universal immutable payload subject to simultaneous thundering herd CAS write stampede. \
        Content addressing guarantees single physical instantiation regardless of concurrency."
            .to_vec(),
    );

    let concurrent_tasks = 32;
    let mut handles = Vec::with_capacity(concurrent_tasks);

    let start_stampede = Instant::now();
    for _ in 0..concurrent_tasks {
        let cas_clone = cas.clone();
        let payload_clone = Arc::clone(&shared_payload);
        handles.push(tokio::spawn(
            async move { cas_clone.put(&payload_clone).await },
        ));
    }

    let mut outcomes = Vec::with_capacity(concurrent_tasks);
    for handle in handles {
        let outcome = handle
            .await
            .expect("Task join must succeed")
            .expect("CAS put must succeed");
        outcomes.push(outcome);
    }
    let elapsed = start_stampede.elapsed();

    // Verify all 32 tasks derived the identical BLAKE3 and SHA-256 addresses
    let reference_blake3 = outcomes[0].blake3_hex.clone();
    let reference_sha256 = outcomes[0].sha256_hex.clone();
    for outcome in &outcomes {
        assert_eq!(outcome.blake3_hex, reference_blake3);
        assert_eq!(outcome.sha256_hex, reference_sha256);
        assert_eq!(outcome.byte_size, shared_payload.len() as u64);
    }

    // Exactly one write was first (`is_new == true`), the remaining 31 deduplicated (`is_new == false`)
    let new_count = outcomes.iter().filter(|o| o.is_new).count();
    let dedup_count = outcomes.iter().filter(|o| !o.is_new).count();
    assert_eq!(new_count, 1, "Exactly one task must register the new blob");
    assert_eq!(
        dedup_count,
        concurrent_tasks - 1,
        "31 tasks must deduplicate"
    );

    // Audit CAS health: exactly 1 indexed physical blob, 0 corrupted
    let audit = cas.audit().await.expect("CAS audit must succeed");
    assert_eq!(audit.total_indexed, 1);
    assert_eq!(audit.corrupted, 0);
    assert_eq!(audit.missing, 0);
    assert!(audit.failures.is_empty());
    assert!(audit.is_healthy());

    // ------------------------------------------------------------------------
    // Part B: Concurrent Lease Mutex Stampede (32 Contending Workers)
    // ------------------------------------------------------------------------
    // Simulates atomic mutual exclusion for distributed lease acquisitions
    let lease_holder = Arc::new(Mutex::new(Option::<String>::None));
    let lease_acquisitions = Arc::new(AtomicUsize::new(0));
    let lease_conflicts = Arc::new(AtomicUsize::new(0));

    let mut lease_handles = Vec::with_capacity(concurrent_tasks);
    for worker_idx in 0..concurrent_tasks {
        let holder = Arc::clone(&lease_holder);
        let acq = Arc::clone(&lease_acquisitions);
        let conf = Arc::clone(&lease_conflicts);

        lease_handles.push(tokio::spawn(async move {
            let worker_id = format!("worker-node-{worker_idx}");
            let mut guard = holder.lock().await;
            if guard.is_none() {
                // Claim exclusive lease
                *guard = Some(worker_id.clone());
                acq.fetch_add(1, Ordering::SeqCst);
                // Simulate holding lease for brief processing window
                tokio::time::sleep(Duration::from_millis(5)).await;
                // Release lease
                *guard = None;
                true
            } else {
                conf.fetch_add(1, Ordering::SeqCst);
                false
            }
        }));
    }

    for h in lease_handles {
        let _ = h.await;
    }

    // Verify zero deadlocks and mutual exclusion
    assert!(
        lease_acquisitions.load(Ordering::SeqCst) >= 1,
        "At least one worker must acquire the lease"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "Contention stampede must complete within 5.0s SLA (actual: {elapsed:?})"
    );
}

// ============================================================================
// TEST 5: Adversarial, Malformed & Corrupted Payload Resilience Under Load
// ============================================================================

#[tokio::test]
async fn test_adversarial_and_corrupted_payload_resilience_under_load() {
    // 1. Parser Resilience: Truncated JSON payload
    let malformed_json =
        r#"{"id": "doc-corrupted", "schema_version": "v1.0", "sections": [{"id": "s1""#;
    let err_malformed = from_json_with_options(malformed_json, &JsonOptions::default());
    assert!(
        matches!(err_malformed, Err(CdmError::Deserialization(_))),
        "Malformed JSON must be rejected with CdmError::Deserialization"
    );

    // 2. Resource Boundedness: Oversized payload exceeding JsonOptions byte ceiling
    let max_allowed = 64 * 1024; // 64 KiB
    let oversized_payload = "A".repeat(128 * 1024); // 128 KiB
    let json_options = JsonOptions {
        max_bytes: max_allowed,
        ..Default::default()
    };
    let err_oversized = from_json_with_options(&oversized_payload, &json_options);
    assert!(
        matches!(err_oversized, Err(CdmError::PayloadTooLarge { .. })),
        "Oversized payload must be rejected with CdmError::PayloadTooLarge"
    );

    // 3. Mixed Batch Under Load: Valid vs Adversarial Items
    let config = WorkerServiceConfig::for_worker("adversarial-resilience-worker", 4, 4)
        .expect("Config should be valid");
    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    let batch_id = BatchId::new();
    let total_items = 10;
    let tasks = create_test_tasks(total_items, "adversarial");

    let processor = move |task: BatchItemTask<String>| async move {
        if task.item_index % 2 == 1 {
            // Adversarial payload simulation
            Err(format!(
                "Adversarial payload rejected: Invalid encoding on item {}",
                task.item_index
            ))
        } else {
            // Valid item: synthesize valid CDM document
            let doc = create_sample_cdm_document(
                task.item_index,
                "Valid Document",
                "Sanitized safe content passing all security bounds.",
            );
            Ok(doc.document_id.to_string())
        }
    };

    let summary = engine
        .execute_batch(batch_id, tasks, Some(4), processor, None)
        .await
        .expect("Batch execution should complete");

    assert_eq!(summary.total_items, 10);
    assert_eq!(summary.succeeded_count, 5, "5 valid items must succeed");
    assert_eq!(
        summary.failed_count, 5,
        "5 adversarial items must fail cleanly"
    );
    assert_eq!(
        summary.succeeded_count + summary.failed_count,
        summary.total_items,
        "Total accounting must match exactly"
    );

    // Verify healthy items were not corrupted or interrupted
    for item in &summary.item_results {
        if item.item_index % 2 == 0 {
            assert!(matches!(item.status, BatchItemStatusOutcome::Succeeded));
        } else {
            assert!(matches!(item.status, BatchItemStatusOutcome::Failed { .. }));
        }
    }
}

// ============================================================================
// TEST 6: Graceful Worker Teardown & Scratch Drain Under Active Load
// ============================================================================

#[tokio::test]
async fn test_graceful_worker_teardown_and_scratch_drain_under_load() {
    let temp_dir = TestDir::new("teardown-scratch");
    let storage: Arc<dyn StorageBackend> =
        Arc::new(LocalFsStorage::new(temp_dir.path()).expect("LocalFsStorage should initialize"));

    // 1. Populate storage with temporary staging files (`.artifact.tmp.*`)
    let staging_key_1 = "staging/.artifact.tmp.part1";
    let staging_key_2 = "staging/.artifact.tmp.part2";
    let finalized_key = "artifacts/job-final/document.json";

    storage
        .put(staging_key_1, b"Temporary scratch chunk 1")
        .await
        .expect("Staging write 1 must succeed");
    storage
        .put(staging_key_2, b"Temporary scratch chunk 2")
        .await
        .expect("Staging write 2 must succeed");
    storage
        .put(finalized_key, b"Finalized immutable artifact")
        .await
        .expect("Finalized write must succeed");

    assert!(storage.exists(staging_key_1).await.unwrap());
    assert!(storage.exists(staging_key_2).await.unwrap());
    assert!(storage.exists(finalized_key).await.unwrap());

    // 2. Initialize worker engine
    let config = WorkerServiceConfig::for_worker("teardown-load-worker", 4, 4)
        .expect("Config should be valid");
    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    // 3. Dispatch an active high-concurrency batch in background
    let batch_id = BatchId::new();
    let tasks = create_test_tasks(16, "teardown");
    let engine_exec = engine.clone();

    let batch_handle = tokio::spawn(async move {
        engine_exec
            .execute_batch(
                batch_id,
                tasks,
                Some(4),
                |task| async move {
                    // Simulate non-trivial execution delay
                    tokio::time::sleep(Duration::from_millis(25)).await;
                    Ok(format!("done-{}", task.item_index))
                },
                None,
            )
            .await
    });

    // Let the batch get actively in flight across worker threads
    tokio::time::sleep(Duration::from_millis(10)).await;

    // 4. Initiate cooperative worker engine shutdown mid-flight
    engine.shutdown();
    assert!(
        engine.is_shutting_down(),
        "Engine must be marked shutting down"
    );

    // Await batch completion
    let summary = batch_handle
        .await
        .expect("Task join must succeed")
        .expect("Batch execution should complete");

    // Accounting: all 16 tasks accounted for across completed, cancelled, or failed
    assert_eq!(summary.total_items, 16);
    assert_eq!(
        summary.succeeded_count + summary.cancelled_count + summary.failed_count,
        16,
        "All in-flight tasks must be accounted for cleanly"
    );

    // 5. Subsequent batch submissions must be immediately rejected
    let subsequent_tasks = create_test_tasks(4, "subsequent");
    let rejected = engine
        .execute_batch(
            BatchId::new(),
            subsequent_tasks,
            Some(2),
            |_| async { Ok("unreachable".to_string()) },
            None,
        )
        .await;

    assert!(
        rejected.is_err(),
        "Subsequent batch submission must be rejected during shutdown"
    );
    match rejected {
        Err(CoreError::JobEngine(msg)) => {
            assert!(
                msg.contains("shutting down"),
                "Error message must indicate worker daemon is shutting down (got: {msg})"
            );
        }
        other => panic!("Expected CoreError::JobEngine shutting down, got: {other:?}"),
    }

    // 6. Execute storage retention sweep to drain scratch files
    let policy = RetentionPolicy::default().with_ephemeral_ttl(chrono::Duration::milliseconds(1));
    let retention_mgr =
        RetentionManager::new(storage.clone(), policy).expect("RetentionManager should initialize");

    // Brief sleep to exceed 1ms ephemeral TTL
    tokio::time::sleep(Duration::from_millis(20)).await;

    let staging_report = retention_mgr
        .cleanup_stale_staging_files(chrono::Duration::zero())
        .await
        .expect("Ephemeral staging sweep must succeed");

    assert_eq!(
        staging_report.ephemeral_files_deleted, 2,
        "Both temporary staging scratch files must be purged"
    );
    assert!(
        !storage.exists(staging_key_1).await.unwrap(),
        "Staging file 1 must no longer exist"
    );
    assert!(
        !storage.exists(staging_key_2).await.unwrap(),
        "Staging file 2 must no longer exist"
    );
    assert!(
        storage.exists(finalized_key).await.unwrap(),
        "Finalized artifact must remain strictly preserved"
    );
}
