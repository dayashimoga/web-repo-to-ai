//! Integration test for worker resumable batch execution.

use std::sync::Arc;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::models::{BatchItemStatusOutcome, BatchItemTask};
use web_repo_to_ai_job_engine::resumable::{
    CheckpointItemResult, CheckpointStore, InMemoryCheckpointStore, JobCheckpoint, ResumableConfig,
};
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

#[tokio::test]
async fn test_worker_engine_resumable_batch_execution() {
    let config = WorkerServiceConfig::for_worker("test-worker-resumable", 4, 2).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..5)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/item/{i}"),
                None,
                (),
            )
        })
        .collect();

    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    // Pre-populate checkpoint simulating crash after items 0 and 1 finished
    let mut cp = JobCheckpoint::new(batch_id.to_string(), 5);
    for (i, task) in tasks.iter().enumerate().take(2) {
        cp.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: None,
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(format!("\"reused-{i}\"")),
            error: None,
            duration_ms: 8,
        });
    }
    store.save_checkpoint(&cp).await.unwrap();

    let executed_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let exec_track = Arc::clone(&executed_counter);

    let resumable_config = ResumableConfig::new(2).unwrap();

    let (summary, stats) = engine
        .execute_resumable_batch(
            batch_id,
            tasks,
            Some(2),
            resumable_config,
            store,
            move |t| {
                let track = Arc::clone(&exec_track);
                async move {
                    track.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(format!("newly-processed-{}", t.item_index))
                }
            },
            None,
        )
        .await
        .expect("Resumable batch execution must succeed");

    assert!(stats.is_resumed);
    assert_eq!(stats.items_reused, 2);
    assert_eq!(stats.items_executed, 3);
    assert_eq!(
        executed_counter.load(std::sync::atomic::Ordering::SeqCst),
        3
    );

    // Full 5-item summary
    assert_eq!(summary.total_items, 5);
    assert_eq!(summary.succeeded_count, 5);
    assert_eq!(summary.failed_count, 0);

    // Verify worker telemetry snapshot
    let snap = engine.metrics_snapshot();
    assert_eq!(snap.resumed_jobs_count, 1);
    assert_eq!(snap.items_reused_from_checkpoint, 2);
    assert_eq!(snap.batches_processed, 1);
    assert_eq!(snap.items_processed, 5);
    assert_eq!(snap.items_succeeded, 5);
}
