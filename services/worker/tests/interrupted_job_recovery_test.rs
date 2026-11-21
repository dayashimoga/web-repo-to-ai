//! # Interrupted-Job Recovery & Resumption Integration Tests
//!
//! Evaluates the reliability, crash resilience, cryptographic integrity verification,
//! and failover guarantees of the web-repo-to-ai worker daemon and job engine under
//! abnormal interruptions, mid-batch worker crashes, and lease expirations:
//! - Worker process crash simulation mid-batch with durable checkpointing.
//! - Cryptographic BLAKE3 checksum verification and corrupted checkpoint quarantine.
//! - Cascading multi-stage crashes across multiple replacement worker instances.
//! - Distributed lease expiration, stale lease reclamation, and worker failover.
//! - Cooperative shutdown and cancellation with idempotent resumption.
//! - CDM document synthesis and streaming archive export integrity across recovery cycles.

use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_exporters::streaming::StreamingExportConfig;
use web_repo_to_ai_exporters::ZipArchive;
use web_repo_to_ai_job_engine::config::BatchExecutionConfig;
use web_repo_to_ai_job_engine::error::JobEngineError;
use web_repo_to_ai_job_engine::models::{BatchItemStatusOutcome, BatchItemTask};
use web_repo_to_ai_job_engine::resumable::{
    CheckpointItemResult, CheckpointStore, InMemoryCheckpointStore, JobCheckpoint, ResumableConfig,
    ResumableParallelExecutor,
};
use web_repo_to_ai_storage::leasing::{LeaseConfig, LeaseRecoveryReport};
use web_repo_to_ai_storage::schema::{JobProgressPhase, JobStatus};
use web_repo_to_ai_storage::state_machine::{JobStateMachine, JobTransitionRequest};
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

/// Helper creating a batch of N generic test conversion tasks.
fn create_test_tasks(count: usize, prefix: &str) -> Vec<BatchItemTask<()>> {
    (0..count)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{prefix}/{i}"),
                None,
                (),
            )
        })
        .collect()
}

/// Helper creating a realistic minimal CDM document for pipeline tests.
fn create_sample_cdm_document(index: usize, title: &str, body: &str) -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: format!("https://example.com/cdm/doc-{index}"),
        canonical_url: format!("https://example.com/cdm/doc-{index}"),
        domain: Some("example.com".to_string()),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    };

    let section = CdmSectionBuilder::new(
        format!("sec-{index}"),
        format!("https://example.com/cdm/doc-{index}#sec"),
        SectionKind::Prose,
        body,
    )
    .title(title)
    .original_path(format!("docs/doc_{index}.md"))
    .build();

    CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        format!("https://example.com/cdm/doc-{index}"),
    )
    .revision(RevisionInfo {
        git_tag: Some(format!("v{index}.0")),
        ..Default::default()
    })
    .add_section(section)
    .build()
}

// ============================================================================
// 1. Worker Crash & Resumption Test
// ============================================================================

#[tokio::test]
async fn test_interrupted_batch_worker_crash_and_resumption() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_id = BatchId::new();
    let total_items = 12;
    let tasks = create_test_tasks(total_items, "crash-resumption");

    // Phase 1: Simulate Worker 1 (crashed after 5 items checkpointed)
    let crashed_worker_id = "worker-alpha-crash";
    let mut initial_cp = JobCheckpoint::new(batch_id.to_string(), total_items);
    let items_before_crash = 5;

    for (i, task) in tasks.iter().enumerate().take(items_before_crash) {
        initial_cp.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: None,
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(format!("\"w1-processed-{i}\"")),
            error: None,
            duration_ms: 15,
        });
    }
    initial_cp
        .metadata
        .insert("last_worker_id".to_string(), crashed_worker_id.to_string());
    initial_cp.update_checksum();
    store.save_checkpoint(&initial_cp).await.unwrap();

    // Phase 2: Replacement Worker 2 boots up and claims batch execution
    let worker2_config = WorkerServiceConfig::for_worker("worker-beta-recovery", 4, 2).unwrap();
    let worker2_engine = WorkerEngine::new(worker2_config).unwrap();

    let executed_indices = Arc::new(Mutex::new(Vec::new()));
    let exec_tracker = Arc::clone(&executed_indices);

    let resumable_config = ResumableConfig::new(2).unwrap();

    let (summary, stats) = worker2_engine
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            resumable_config,
            Arc::clone(&store),
            move |t| {
                let tracker = Arc::clone(&exec_tracker);
                async move {
                    tracker.lock().await.push(t.item_index);
                    Ok(format!("w2-processed-{}", t.item_index))
                }
            },
            None,
        )
        .await
        .expect("Resumed batch execution must succeed");

    // Resumption verification
    assert!(stats.is_resumed, "Job must be identified as resumed");
    assert_eq!(
        stats.items_reused, items_before_crash,
        "Must reuse all previously checkpointed items"
    );
    assert_eq!(
        stats.items_executed,
        total_items - items_before_crash,
        "Must only execute remaining items"
    );

    // Verify exactly items 5..12 were processed by Worker 2
    let executed = executed_indices.lock().await;
    assert_eq!(
        *executed,
        (items_before_crash..total_items).collect::<Vec<_>>(),
        "Worker 2 must only execute uncompleted item indices"
    );

    // Verify consolidated summary contains all 12 items in sequence
    assert_eq!(summary.total_items, total_items);
    assert_eq!(summary.succeeded_count, total_items);
    assert_eq!(summary.failed_count, 0);
    assert_eq!(summary.item_results.len(), total_items);

    for i in 0..total_items {
        let item = &summary.item_results[i];
        assert_eq!(item.item_index, i);
        assert!(item.is_success());
        if i < items_before_crash {
            assert_eq!(item.value, Some(format!("w1-processed-{i}")));
        } else {
            assert_eq!(item.value, Some(format!("w2-processed-{i}")));
        }
    }

    // Verify worker telemetry snapshot
    let metrics = worker2_engine.metrics_snapshot();
    assert_eq!(metrics.resumed_jobs_count, 1);
    assert_eq!(metrics.items_reused_from_checkpoint, items_before_crash);
    assert_eq!(metrics.batches_processed, 1);
    assert_eq!(metrics.items_processed, total_items);
    assert_eq!(metrics.items_succeeded, total_items);

    // Verify final checkpoint stored in store
    let final_cp = store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .expect("Final checkpoint must exist");
    assert!(final_cp.is_fully_finished());
    assert_eq!(final_cp.completed_indices.len(), total_items);
    assert_eq!(final_cp.failed_indices.len(), 0);
    assert!(final_cp.verify_checksum().is_ok());
}

// ============================================================================
// 2. Cascading Multi-Stage Worker Crashes Test
// ============================================================================

#[tokio::test]
async fn test_multi_stage_cascading_worker_crashes_and_resumption() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_id = BatchId::new();
    let total_items = 20;
    let tasks = create_test_tasks(total_items, "cascading-crashes");

    // Stage 1: Worker 1 completes items 0..6 (6 items) and crashes
    let mut cp1 = JobCheckpoint::new(batch_id.to_string(), total_items);
    for (i, task) in tasks.iter().enumerate().take(6) {
        cp1.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: None,
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(format!("\"worker-1-val-{i}\"")),
            error: None,
            duration_ms: 10,
        });
    }
    store.save_checkpoint(&cp1).await.unwrap();

    // Stage 2: Worker 2 resumes, completes items 6..13 (7 more items, total 13), then crashes
    let mut cp2 = store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .unwrap();

    for (i, task) in tasks.iter().enumerate().skip(6).take(7) {
        cp2.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: None,
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(format!("\"worker-2-val-{i}\"")),
            error: None,
            duration_ms: 12,
        });
    }
    store.save_checkpoint(&cp2).await.unwrap();

    // Stage 3: Worker 3 resumes and finishes the remaining 7 items (13..20)
    let worker3_cfg = WorkerServiceConfig::for_worker("worker-stage-3", 4, 2).unwrap();
    let worker3 = WorkerEngine::new(worker3_cfg).unwrap();

    let stage3_executed = Arc::new(Mutex::new(Vec::new()));
    let stage3_tracker = Arc::clone(&stage3_executed);

    let (summary, stats) = worker3
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            ResumableConfig::new(2).unwrap(),
            Arc::clone(&store),
            move |t| {
                let tracker = Arc::clone(&stage3_tracker);
                async move {
                    tracker.lock().await.push(t.item_index);
                    Ok(format!("worker-3-val-{}", t.item_index))
                }
            },
            None,
        )
        .await
        .expect("Stage 3 recovery must complete the batch");

    // Assertions on Stage 3 execution
    assert!(stats.is_resumed);
    assert_eq!(stats.items_reused, 13, "Must reuse 13 accumulated items");
    assert_eq!(
        stats.items_executed, 7,
        "Must execute the 7 remaining items"
    );

    let executed = stage3_executed.lock().await;
    assert_eq!(*executed, (13..20).collect::<Vec<_>>());

    // Verify all 20 items in final summary
    assert_eq!(summary.total_items, 20);
    assert_eq!(summary.succeeded_count, 20);
    assert_eq!(summary.failed_count, 0);

    for i in 0..20 {
        let res = &summary.item_results[i];
        assert_eq!(res.item_index, i);
        if i < 6 {
            assert_eq!(res.value, Some(format!("worker-1-val-{i}")));
        } else if i < 13 {
            assert_eq!(res.value, Some(format!("worker-2-val-{i}")));
        } else {
            assert_eq!(res.value, Some(format!("worker-3-val-{i}")));
        }
    }

    // Verify store checkpoint state
    let final_cp = store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert!(final_cp.is_fully_finished());
    assert_eq!(final_cp.completed_indices.len(), 20);
    assert_eq!(final_cp.sequence_number, cp2.sequence_number + 7);
    assert!(final_cp.verify_checksum().is_ok());
}

// ============================================================================
// 3. Corrupted Checkpoint Quarantine & Clean Recovery Fallback Test
// ============================================================================

#[tokio::test]
async fn test_interrupted_job_tampered_or_corrupted_checkpoint_recovery() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_id = BatchId::new();
    let tasks = create_test_tasks(6, "tamper-recovery");

    // 1. Create a checkpoint and manually tamper with internal state without recalculating checksum
    let mut corrupted_cp = JobCheckpoint::new(batch_id.to_string(), 6);
    corrupted_cp.record_item(CheckpointItemResult {
        item_index: 0,
        job_id: tasks[0].job_id,
        url: tasks[0].url.clone(),
        custom_id: None,
        status: BatchItemStatusOutcome::Succeeded,
        value_json: Some("\"val-0\"".to_string()),
        error: None,
        duration_ms: 10,
    });

    // Tamper with completed indices directly (simulating disk bit rot or payload truncation)
    corrupted_cp.completed_indices.insert(999);
    store.save_checkpoint(&corrupted_cp).await.unwrap();

    // 2. Direct verification must detect the tampering
    let verify_res = corrupted_cp.verify_checksum();
    assert!(
        matches!(verify_res, Err(JobEngineError::CheckpointCorrupted { .. })),
        "Checksum mismatch must be recognized as CheckpointCorrupted"
    );

    // 3. Resumable parallel executor loading with verify_checksum_on_load: true must reject the corrupted checkpoint
    let batch_cfg = BatchExecutionConfig {
        max_parallelism: 2,
        ..Default::default()
    };
    let resumable_cfg = ResumableConfig::new(2).unwrap(); // verify_checksum_on_load is true by default
    let executor =
        ResumableParallelExecutor::new(batch_cfg, resumable_cfg, Arc::clone(&store)).unwrap();

    let exec_err = executor
        .execute_resumable(
            batch_id,
            tasks.clone(),
            |t| async move { Ok(format!("recovered-{}", t.item_index)) },
            None,
            None,
        )
        .await;

    assert!(
        matches!(exec_err, Err(JobEngineError::CheckpointCorrupted { .. })),
        "Execution must be aborted upon detecting corrupted checkpoint"
    );

    // 4. Safe recovery quarantine: Delete corrupted checkpoint and execute freshly
    store
        .delete_checkpoint(&batch_id.to_string())
        .await
        .expect("Quarantine deletion must succeed");

    assert!(
        store
            .load_checkpoint(&batch_id.to_string())
            .await
            .unwrap()
            .is_none(),
        "Quarantined checkpoint must be purged"
    );

    // Re-execute cleanly: all 6 items succeed from a clean state
    let worker_cfg = WorkerServiceConfig::for_worker("quarantine-recovery-worker", 4, 2).unwrap();
    let worker = WorkerEngine::new(worker_cfg).unwrap();

    let (summary, stats) = worker
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            ResumableConfig::new(2).unwrap(),
            Arc::clone(&store),
            |t| async move { Ok(format!("clean-val-{}", t.item_index)) },
            None,
        )
        .await
        .expect("Fresh execution after quarantine must succeed");

    assert!(!stats.is_resumed, "Execution must be fresh, not resumed");
    assert_eq!(stats.items_reused, 0);
    assert_eq!(stats.items_executed, 6);
    assert_eq!(summary.succeeded_count, 6);

    let clean_cp = store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert!(clean_cp.verify_checksum().is_ok());
    assert_eq!(clean_cp.completed_indices.len(), 6);
}

// ============================================================================
// 4. Distributed Lease Expiration & Worker Failover Test
// ============================================================================

#[test]
fn test_distributed_lease_expiration_and_worker_failover() {
    // 1. Lease configuration validation
    let lease_alpha = LeaseConfig::new("worker-node-alpha", 10, 3, 3).unwrap();
    assert_eq!(lease_alpha.worker_id, "worker-node-alpha");
    assert_eq!(lease_alpha.lease_duration_seconds, 10);
    assert_eq!(lease_alpha.heartbeat_interval_seconds, 3);
    assert_eq!(lease_alpha.max_attempts, 3);

    // 2. State Machine Lifecycle Transitions for Stale Lease Recovery
    // Queued -> Leased -> Running -> (Crash / Lease Expiry) -> Queued (requeue) -> Leased -> Running -> Completed
    assert!(JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Leased).is_ok());
    assert!(JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Running).is_ok());

    // Worker 1 crashes: Lease expires -> requeue transition back to Queued
    assert!(JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Leased).is_ok());
    assert!(JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Queued).is_ok());

    let requeue_req = JobTransitionRequest::requeue("Worker lease expired without heartbeat");
    assert_eq!(requeue_req.to_status, JobStatus::Queued);
    assert_eq!(requeue_req.phase, JobProgressPhase::Initializing);
    assert_eq!(requeue_req.progress_pct, 0);
    assert_eq!(requeue_req.worker_id, None);

    // Worker 2 acquires reclaimed job: Queued -> Leased -> Running -> Completed
    assert!(JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Leased).is_ok());
    assert!(JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Running).is_ok());
    assert!(JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Completed).is_ok());

    // Terminal statuses are immutable
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Completed, JobStatus::Running).is_err()
    );
    assert!(JobStateMachine::evaluate_transition(JobStatus::Failed, JobStatus::Queued).is_err());

    // 3. Lease Recovery Report helper evaluation
    let mut report = LeaseRecoveryReport::default();
    assert!(report.is_empty());
    assert_eq!(report.total_processed(), 0);

    let job_id_1 = Uuid::now_v7();
    let job_id_2 = Uuid::now_v7();

    // Job 1 recovered to queued for retry (attempt < max_attempts)
    report.recovered_to_queued += 1;
    report.recovered_job_ids.push(job_id_1);

    // Job 2 exceeded max_attempts -> permanently marked failed
    report.permanently_failed += 1;
    report.failed_job_ids.push(job_id_2);

    assert!(!report.is_empty());
    assert_eq!(report.total_processed(), 2);
    assert_eq!(report.recovered_to_queued, 1);
    assert_eq!(report.permanently_failed, 1);
}

// ============================================================================
// 5. Cooperative Cancellation & Safe Resumption Test
// ============================================================================

#[tokio::test]
async fn test_interrupted_batch_cancellation_and_safe_drain() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_id = BatchId::new();
    let total_items = 8;
    let tasks = create_test_tasks(total_items, "cancel-drain");

    let cancel_token = CancellationToken::new();
    let completed_counter = Arc::new(AtomicUsize::new(0));

    let batch_cfg = BatchExecutionConfig {
        max_parallelism: 2,
        ..Default::default()
    };
    let resumable_cfg = ResumableConfig::new(2).unwrap();
    let executor =
        ResumableParallelExecutor::new(batch_cfg, resumable_cfg, Arc::clone(&store)).unwrap();

    let trigger_cancel = cancel_token.clone();
    let task_counter = Arc::clone(&completed_counter);

    // Run batch with cooperative cancellation triggered after 2 items complete
    let (summary, _stats) = executor
        .execute_resumable(
            batch_id,
            tasks.clone(),
            move |t| {
                let cnt = Arc::clone(&task_counter);
                let cancel = trigger_cancel.clone();
                async move {
                    let cur = cnt.fetch_add(1, Ordering::SeqCst) + 1;
                    if cur == 2 {
                        // Trigger cancellation after 2 items finish
                        cancel.cancel();
                    }
                    tokio::time::sleep(Duration::from_millis(15)).await;
                    Ok(format!("done-{}", t.item_index))
                }
            },
            None,
            Some(cancel_token),
        )
        .await
        .expect("Execution must return cleanly even when cancelled");

    // Partial results: some completed, remainder cancelled
    assert!(
        summary.cancelled_count > 0,
        "Cancelled items must be recorded"
    );
    assert!(
        summary.succeeded_count >= 2,
        "At least 2 items succeeded before cancellation"
    );
    assert_eq!(
        summary.succeeded_count + summary.cancelled_count + summary.failed_count,
        total_items
    );

    // Checkpoint in store reflects completed items
    let cp = store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .expect("Partial checkpoint must be persisted");
    let initial_succeeded = cp.completed_indices.len();
    assert!(initial_succeeded >= 2);

    // Now resume the job with a fresh worker engine and fresh uncancelled token
    let worker_cfg = WorkerServiceConfig::for_worker("resumption-worker", 4, 2).unwrap();
    let worker = WorkerEngine::new(worker_cfg).unwrap();

    let (resumed_summary, resumed_stats) = worker
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            ResumableConfig::new(2).unwrap(),
            Arc::clone(&store),
            |t| async move { Ok(format!("resumed-done-{}", t.item_index)) },
            None,
        )
        .await
        .expect("Resuming cancelled batch must succeed");

    assert!(resumed_stats.is_resumed);
    assert_eq!(resumed_stats.items_reused, initial_succeeded);
    assert_eq!(
        resumed_stats.items_executed,
        total_items - initial_succeeded
    );
    assert_eq!(resumed_summary.total_items, total_items);
    assert_eq!(resumed_summary.succeeded_count, total_items);
    assert_eq!(resumed_summary.cancelled_count, 0);
    assert_eq!(resumed_summary.failed_count, 0);
}

// ============================================================================
// 6. End-to-End CDM Synthesis & Streaming Export Integrity Test
// ============================================================================

#[tokio::test]
async fn test_interrupted_job_cdm_synthesis_and_export_integrity() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_id = BatchId::new();
    let total_docs = 6;

    let tasks: Vec<BatchItemTask<()>> = (0..total_docs)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/cdm/doc-{i}"),
                Some(format!("custom-doc-{i}")),
                (),
            )
        })
        .collect();

    // 1. Worker 1 executes and checkpoints the first 3 CDM documents, then aborts
    let mut initial_cp = JobCheckpoint::new(batch_id.to_string(), total_docs);
    for (i, task) in tasks.iter().enumerate().take(3) {
        let cdm = create_sample_cdm_document(
            i,
            &format!("Document {i} Title"),
            &format!("This is the verified content of document {i} processed by Worker 1."),
        );
        let cdm_json = serde_json::to_string(&cdm).expect("Serialization must succeed");

        initial_cp.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: task.custom_id.clone(),
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(cdm_json),
            error: None,
            duration_ms: 25,
        });
    }
    store.save_checkpoint(&initial_cp).await.unwrap();

    // 2. Worker 2 resumes and processes the remaining 3 CDM documents
    let worker2_cfg = WorkerServiceConfig::for_worker("worker-cdm-exporter", 4, 2).unwrap();
    let worker2 = WorkerEngine::new(worker2_cfg).unwrap();

    let (summary, stats) = worker2
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            ResumableConfig::new(2).unwrap(),
            Arc::clone(&store),
            |t| async move {
                let cdm = create_sample_cdm_document(
                    t.item_index,
                    &format!("Document {} Title", t.item_index),
                    &format!(
                        "This is the verified content of document {} processed by Worker 2.",
                        t.item_index
                    ),
                );
                Ok(cdm)
            },
            None,
        )
        .await
        .expect("Resumed batch conversion to CDM must succeed");

    assert!(stats.is_resumed);
    assert_eq!(stats.items_reused, 3);
    assert_eq!(stats.items_executed, 3);
    assert_eq!(summary.total_items, total_docs);
    assert_eq!(summary.succeeded_count, total_docs);

    // 3. Extract all 6 CDM documents from the consolidated summary
    let mut reconstructed_docs: Vec<CdmDocument> = Vec::new();
    for item in &summary.item_results {
        assert!(item.is_success());
        let doc = item
            .value
            .as_ref()
            .expect("CDM document must be present")
            .clone();
        reconstructed_docs.push(doc);
    }

    assert_eq!(reconstructed_docs.len(), total_docs);

    // Verify mathematical integrity of each document
    for (i, doc) in reconstructed_docs.iter().enumerate() {
        assert_eq!(doc.sections.len(), 1);
        let expected_blake3 = blake3::hash(doc.sections[0].body.as_bytes())
            .to_hex()
            .to_string();
        assert_eq!(doc.sections[0].content_hash, expected_blake3);
        assert_eq!(
            doc.canonical_url,
            format!("https://example.com/cdm/doc-{i}")
        );
    }

    // 4. Stream all 6 documents into a destination ZIP package via WorkerEngine
    let mut zip_buffer = Cursor::new(Vec::new());
    let export_config = StreamingExportConfig::new(8192)
        .unwrap()
        .with_manifests(true, true);

    let export_summary = worker2
        .execute_streaming_export(&mut zip_buffer, export_config, &reconstructed_docs)
        .expect("Streaming export of recovered CDM documents must succeed");

    assert!(export_summary.total_items_written >= total_docs * 4 + 2);
    assert!(export_summary.total_uncompressed_bytes > 0);
    assert!(!export_summary.sha256_checksum.is_empty());
    assert!(!export_summary.blake3_checksum.is_empty());

    // Verify the resulting zip archive contents
    let mut reader = Cursor::new(zip_buffer.into_inner());
    let mut zip = ZipArchive::new(&mut reader).expect("Valid ZIP archive");

    assert!(
        zip.by_name("manifest.json").is_ok(),
        "Archive must contain manifest.json"
    );
    assert!(
        zip.by_name("checksums.sha256").is_ok(),
        "Archive must contain checksums.sha256"
    );

    for i in 0..total_docs {
        assert!(
            zip.by_name(&format!("doc_{i}/document.json")).is_ok(),
            "Archive must contain doc_{i}/document.json"
        );
        assert!(
            zip.by_name(&format!("doc_{i}/context.md")).is_ok(),
            "Archive must contain doc_{i}/context.md"
        );
    }
}
