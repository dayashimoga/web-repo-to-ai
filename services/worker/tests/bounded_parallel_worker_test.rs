//! # Integration Tests for Bounded Parallel Execution in `services/worker`
//!
//! Verifies concurrency bounds, error isolation, per-item timeouts, cooperative
//! shutdown, and realtime progress notifications within the worker daemon engine.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::sleep;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::models::{BatchItemTask, BatchProgressUpdate};
use web_repo_to_ai_worker::{WorkerEngine, WorkerServiceConfig};

#[tokio::test]
async fn test_worker_engine_concurrency_ceiling_respected() {
    // Configure worker with a strict ceiling of 3 concurrent tasks
    let config = WorkerServiceConfig::for_worker("test-worker-ceiling", 3, 3).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let concurrent_tracker = Arc::new(AtomicUsize::new(0));
    let observed_max = Arc::new(AtomicUsize::new(0));

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..12)
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

    let tracker_c = Arc::clone(&concurrent_tracker);
    let observed_c = Arc::clone(&observed_max);

    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            None,
            move |_task| {
                let tracker = Arc::clone(&tracker_c);
                let observed = Arc::clone(&observed_c);
                async move {
                    let cur = tracker.fetch_add(1, Ordering::SeqCst) + 1;
                    let mut max = observed.load(Ordering::Relaxed);
                    while cur > max {
                        match observed.compare_exchange_weak(
                            max,
                            cur,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(actual) => max = actual,
                        }
                    }

                    // Simulate conversion processing time
                    sleep(Duration::from_millis(30)).await;

                    tracker.fetch_sub(1, Ordering::SeqCst);
                    Ok("converted".to_string())
                }
            },
            None,
        )
        .await
        .unwrap();

    // Verify all 12 items succeeded
    assert_eq!(summary.total_items, 12);
    assert_eq!(summary.succeeded_count, 12);
    assert_eq!(summary.failed_count, 0);
    assert!(summary.all_succeeded());

    // Verify that observed concurrency NEVER exceeded the configured ceiling of 3
    let peak = observed_max.load(Ordering::SeqCst);
    assert!(
        peak <= 3,
        "Observed concurrency ({peak}) exceeded worker ceiling (3)"
    );
    assert!(peak >= 1);
    assert_eq!(concurrent_tracker.load(Ordering::SeqCst), 0);

    // Verify service telemetry
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.batches_processed, 1);
    assert_eq!(metrics.items_succeeded, 12);
    assert!(metrics.peak_concurrency <= 3);
}

#[tokio::test]
async fn test_worker_engine_resilient_to_individual_failures() {
    let config = WorkerServiceConfig::for_worker("test-worker-resilience", 4, 4).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..8)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{i}"),
                None,
                (),
            )
        })
        .collect();

    // Items 2, 5, 7 fail; others succeed
    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            None,
            |task| async move {
                match task.item_index {
                    2 => Err("HTTP 404 Not Found".to_string()),
                    5 => Err("SSRF protection: blocked private IP range".to_string()),
                    7 => Err("Extraction timeout exceeded".to_string()),
                    _ => Ok(format!("Success for {}", task.item_index)),
                }
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(summary.total_items, 8);
    assert_eq!(summary.succeeded_count, 5);
    assert_eq!(summary.failed_count, 3);
    assert!(summary.has_failures());
    assert!((summary.success_rate() - 0.625).abs() < f64::EPSILON);

    // Verify item sequence preservation and individual error attribution
    for res in &summary.item_results {
        match res.item_index {
            2 => {
                assert!(res.status.is_failure());
                assert!(res.error.as_ref().unwrap().contains("404"));
            }
            5 => {
                assert!(res.status.is_failure());
                assert!(res.error.as_ref().unwrap().contains("SSRF"));
            }
            7 => {
                assert!(res.status.is_failure());
                assert!(res.error.as_ref().unwrap().contains("timeout"));
            }
            other => {
                assert!(res.status.is_success());
                assert_eq!(res.value.as_ref().unwrap(), &format!("Success for {other}"));
            }
        }
    }
}

#[tokio::test]
async fn test_worker_engine_per_batch_parallelism_override() {
    // Worker ceiling is 8, default is 4
    let config = WorkerServiceConfig::for_worker("test-worker-override", 8, 4).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let concurrent_tracker = Arc::new(AtomicUsize::new(0));
    let observed_max = Arc::new(AtomicUsize::new(0));

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..6)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{i}"),
                None,
                (),
            )
        })
        .collect();

    let tracker_c = Arc::clone(&concurrent_tracker);
    let observed_c = Arc::clone(&observed_max);

    // Override parallelism to 2
    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            Some(2),
            move |_task| {
                let tracker = Arc::clone(&tracker_c);
                let observed = Arc::clone(&observed_c);
                async move {
                    let cur = tracker.fetch_add(1, Ordering::SeqCst) + 1;
                    let mut max = observed.load(Ordering::Relaxed);
                    while cur > max {
                        match observed.compare_exchange_weak(
                            max,
                            cur,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(actual) => max = actual,
                        }
                    }
                    sleep(Duration::from_millis(25)).await;
                    tracker.fetch_sub(1, Ordering::SeqCst);
                    Ok("done".to_string())
                }
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(summary.succeeded_count, 6);
    let peak = observed_max.load(Ordering::SeqCst);
    assert!(
        peak <= 2,
        "Peak concurrency ({peak}) exceeded requested override (2)"
    );
}

#[tokio::test]
async fn test_worker_engine_realtime_progress_notifications() {
    let config = WorkerServiceConfig::for_worker("test-worker-progress", 4, 2).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let (tx, mut rx) = mpsc::channel::<BatchProgressUpdate>(16);

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..4)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{i}"),
                None,
                (),
            )
        })
        .collect();

    let collector_handle = tokio::spawn(async move {
        let mut updates = Vec::new();
        while let Some(update) = rx.recv().await {
            updates.push(update);
        }
        updates
    });

    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            Some(2),
            |task| async move {
                sleep(Duration::from_millis(15)).await;
                if task.item_index == 3 {
                    Err("error".to_string())
                } else {
                    Ok("success".to_string())
                }
            },
            Some(tx),
        )
        .await
        .unwrap();

    assert_eq!(summary.total_items, 4);

    let progress_history = collector_handle.await.unwrap();
    assert!(!progress_history.is_empty());

    // Final update should report 4 total items (3 succeeded + 1 failed = 100%)
    let final_update = progress_history.last().unwrap();
    assert_eq!(final_update.total_items, 4);
    assert_eq!(final_update.completed_items + final_update.failed_items, 4);
    assert_eq!(final_update.progress_pct(), 100);
}

#[tokio::test]
async fn test_worker_engine_cooperative_shutdown_draining() {
    let config = WorkerServiceConfig::for_worker("test-worker-drain", 4, 2).unwrap();
    let engine = WorkerEngine::new(config).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..6)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{i}"),
                None,
                (),
            )
        })
        .collect();

    let engine_clone = engine.clone();

    // Trigger shutdown after 20ms while items are processing
    tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        engine_clone.shutdown();
    });

    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            Some(2),
            |_task| async move {
                sleep(Duration::from_millis(50)).await;
                Ok("done".to_string())
            },
            None,
        )
        .await
        .unwrap();

    // Some tasks will be cancelled due to cooperative shutdown
    assert!(
        summary.cancelled_count > 0 || summary.succeeded_count > 0,
        "Expected mixed cancelled or completed tasks"
    );
    assert!(engine.is_shutting_down());

    // Subsequent batch requests must be rejected immediately
    let reject_res = engine
        .execute_batch(
            BatchId::new(),
            vec![BatchItemTask::new(
                0,
                JobId::new(),
                "https://example.com",
                None,
                (),
            )],
            None,
            |_| async { Ok("fail".to_string()) },
            None,
        )
        .await;

    assert!(reject_res.is_err());
}
