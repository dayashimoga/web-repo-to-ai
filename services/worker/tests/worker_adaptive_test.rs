//! # Worker Engine Adaptive Concurrency Batch Execution Tests
//!
//! Validates worker daemon orchestration of adaptive task batches,
//! dynamic AIMD regulation, rate limit backoffs, and metrics telemetry.

use std::time::Duration;
use tokio::time::sleep;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::adaptive::AdaptiveConcurrencyConfig;
use web_repo_to_ai_job_engine::models::BatchItemTask;
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

#[tokio::test]
async fn test_worker_engine_execute_adaptive_batch() {
    let config = WorkerServiceConfig {
        worker_id: "test-worker-adaptive-01".to_string(),
        max_concurrent_jobs: 6,
        default_batch_parallelism: 2,
        ..Default::default()
    };

    let engine = WorkerEngine::new(config).expect("Worker engine must initialize");

    let adaptive_config = AdaptiveConcurrencyConfig::new(1, 4, 2)
        .expect("Valid adaptive config")
        .with_success_threshold(2)
        .with_increase_step(1)
        .with_decrease_factor(0.5)
        .with_cooldown(Duration::from_millis(0));

    let tasks = (0..5)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/api/item/{i}"),
                None,
                i,
            )
        })
        .collect::<Vec<_>>();

    let batch_id = BatchId::new();

    let summary = engine
        .execute_adaptive_batch(
            batch_id,
            tasks,
            Some(adaptive_config),
            |task| async move {
                if task.payload == 2 {
                    // Simulate HTTP 429 rate limit
                    Err("HTTP 429 Too Many Requests: Upstream throttling".to_string())
                } else {
                    sleep(Duration::from_millis(15)).await;
                    Ok(format!("PROCESSED_ITEM_{}", task.payload))
                }
            },
            None,
        )
        .await
        .expect("Adaptive batch execution must complete");

    assert_eq!(summary.batch_id, batch_id);
    assert_eq!(summary.total_items, 5);
    assert_eq!(summary.succeeded_count, 4);
    assert_eq!(summary.failed_count, 1);
    assert_eq!(summary.timed_out_count, 0);

    // Verify worker telemetry snapshot
    let snap = engine.metrics_snapshot();
    assert_eq!(snap.batches_processed, 1);
    assert_eq!(snap.items_processed, 5);
    assert_eq!(snap.items_succeeded, 4);
    assert_eq!(snap.items_failed, 1);
    assert!(snap.adaptive_scale_ups >= 1);
    assert_eq!(snap.adaptive_backoffs, 1);
    assert_eq!(snap.adaptive_rate_limits, 1);
}
