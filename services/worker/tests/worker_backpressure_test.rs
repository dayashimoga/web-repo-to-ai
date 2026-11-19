//! # Worker Engine Backpressure Batch Execution Tests
//!
//! Validates worker daemon orchestration of flow-controlled task batches,
//! bounded buffer watermarks, and telemetry snapshots.

use std::time::Duration;
use tokio::time::sleep;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::backpressure::BackpressureConfig;
use web_repo_to_ai_job_engine::models::BatchItemTask;
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

#[tokio::test]
async fn test_worker_engine_execute_flow_controlled_batch() {
    let config = WorkerServiceConfig {
        worker_id: "test-worker-bp-01".to_string(),
        max_concurrent_jobs: 6,
        default_batch_parallelism: 3,
        ..Default::default()
    };

    let engine = WorkerEngine::new(config).expect("Worker engine must initialize");

    let bp_config = BackpressureConfig::new(8, 6, 3)
        .expect("Valid backpressure config")
        .with_wait_timeout(Duration::from_secs(5));

    let tasks = (0..6)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/flow/item/{i}"),
                None,
                i,
            )
        })
        .collect::<Vec<_>>();

    let batch_id = BatchId::new();

    let summary = engine
        .execute_flow_controlled_batch(
            batch_id,
            tasks,
            Some(bp_config),
            |task| async move {
                sleep(Duration::from_millis(15)).await;
                Ok(format!("FLOW_ITEM_{}", task.payload))
            },
            None,
        )
        .await
        .expect("Flow controlled batch execution must complete");

    assert_eq!(summary.batch_id, batch_id);
    assert_eq!(summary.total_items, 6);
    assert_eq!(summary.succeeded_count, 6);
    assert_eq!(summary.failed_count, 0);
    assert_eq!(summary.timed_out_count, 0);

    // Verify worker telemetry snapshot
    let snap = engine.metrics_snapshot();
    assert_eq!(snap.batches_processed, 1);
    assert_eq!(snap.items_processed, 6);
    assert_eq!(snap.items_succeeded, 6);
    assert_eq!(snap.items_failed, 0);
}
