//! # Worker Engine Incremental Batch Processing Tests
//!
//! Validates worker daemon orchestration of incremental task batches,
//! change classification, metrics telemetry, and zero-leakage error handling.

use web_repo_to_ai_core::id::BatchId;
use web_repo_to_ai_job_engine::incremental::{FileSignature, IncrementalCandidate};
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

#[tokio::test]
async fn test_worker_engine_execute_incremental_batch() {
    let config = WorkerServiceConfig {
        worker_id: "test-worker-inc-01".to_string(),
        max_concurrent_jobs: 4,
        default_batch_parallelism: 2,
        ..Default::default()
    };

    let engine = WorkerEngine::new(config).expect("Worker engine must initialize");

    let baseline = vec![
        FileSignature::new("page1.html", "hash_p1_v1", 2048),
        FileSignature::new("page2.html", "hash_p2_v1", 4096),
    ];

    let candidates = vec![
        IncrementalCandidate {
            item_index: 0,
            path: "page1.html".to_string(), // Unchanged
            signature: FileSignature::new("page1.html", "hash_p1_v1", 2048),
            payload: "content1",
        },
        IncrementalCandidate {
            item_index: 1,
            path: "page2.html".to_string(), // Modified
            signature: FileSignature::new("page2.html", "hash_p2_v2", 5120),
            payload: "content2_updated",
        },
        IncrementalCandidate {
            item_index: 2,
            path: "page3.html".to_string(), // Added
            signature: FileSignature::new("page3.html", "hash_p3_v1", 1024),
            payload: "content3_new",
        },
    ];

    let batch_id = BatchId::new();

    let report = engine
        .execute_incremental_batch(
            batch_id,
            candidates,
            &baseline,
            Some(2),
            |task| async move { Ok(format!("CONVERTED: {} ({})", task.url, task.payload)) },
            |sig| {
                let path = sig.path.clone();
                async move { Ok(format!("CACHED_CONVERSION: {path}")) }
            },
        )
        .await
        .expect("Worker incremental execution must succeed");

    assert_eq!(report.batch_id, batch_id);
    assert_eq!(report.total_candidates, 3);
    assert_eq!(report.reused_count, 1);
    assert_eq!(report.processed_count, 2);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.reused_bytes, 2048);
    assert_eq!(report.processed_bytes, 5120 + 1024);
    assert!(report.conservation_verified());

    // Assert results
    assert_eq!(report.all_results.len(), 3);
    assert_eq!(report.all_results[0], "CACHED_CONVERSION: page1.html");
    assert_eq!(
        report.all_results[1],
        "CONVERTED: page2.html (content2_updated)"
    );
    assert_eq!(
        report.all_results[2],
        "CONVERTED: page3.html (content3_new)"
    );

    // Metrics snapshot validation
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.batches_processed, 1);
    assert_eq!(metrics.items_succeeded, 2);
    assert_eq!(metrics.incremental_items_reused, 1);
    assert_eq!(metrics.incremental_bytes_saved, 2048);
}
