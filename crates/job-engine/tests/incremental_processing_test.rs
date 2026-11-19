//! # Incremental Processing & Changed-File Execution Tests
//!
//! Validates changed-file diff classification, mathematical conservation invariants,
//! zero-overhead reuse resolution, and bounded concurrency task execution.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use web_repo_to_ai_core::id::BatchId;
use web_repo_to_ai_job_engine::config::BatchExecutionConfig;
use web_repo_to_ai_job_engine::executor::BoundedParallelExecutor;
use web_repo_to_ai_job_engine::incremental::{
    ChangeCategory, ChangedFileDiff, FileSignature, IncrementalCandidate, IncrementalJobCoordinator,
};

#[test]
fn test_changed_file_diff_classification_and_conservation() {
    let baseline = vec![
        FileSignature::new("src/main.rs", "hash_main_v1", 1024),
        FileSignature::new("src/lib.rs", "hash_lib_v1", 2048),
        FileSignature::new("src/old_feature.rs", "hash_old", 512),
    ];

    let current = vec![
        FileSignature::new("src/main.rs", "hash_main_v1", 1024), // Unchanged
        FileSignature::new("src/lib.rs", "hash_lib_v2", 2560),   // Modified
        FileSignature::new("src/new_feature.rs", "hash_new", 768), // Added
    ];

    let diff = ChangedFileDiff::from_manifests(&baseline, &current);

    // Classification assertions
    assert_eq!(diff.baseline_count, 3);
    assert_eq!(diff.current_count, 3);
    assert_eq!(diff.unchanged.len(), 1);
    assert_eq!(diff.modified.len(), 1);
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.deleted.len(), 1);

    assert_eq!(diff.unchanged[0].path, "src/main.rs");
    assert_eq!(diff.modified[0].path, "src/lib.rs");
    assert_eq!(diff.added[0].path, "src/new_feature.rs");
    assert_eq!(diff.deleted[0].path, "src/old_feature.rs");

    // Metrics and conservation assertions
    assert!(diff.is_balanced(), "Mathematical conservation must hold");
    assert_eq!(diff.to_process_count(), 2);
    assert_eq!(diff.reusable_count(), 1);
    assert_eq!(diff.reusable_bytes(), 1024);
    assert_eq!(diff.to_process_bytes(), 2560 + 768);

    let reusable_hashes = diff.reusable_hashes();
    assert!(reusable_hashes.contains("hash_main_v1"));
    assert!(!reusable_hashes.contains("hash_lib_v1"));

    // Category predicates
    assert!(ChangeCategory::Added.requires_processing());
    assert!(ChangeCategory::Modified.requires_processing());
    assert!(!ChangeCategory::Unchanged.requires_processing());
    assert!(ChangeCategory::Unchanged.is_reusable());
    assert!(!ChangeCategory::Added.is_reusable());
}

#[tokio::test]
async fn test_incremental_coordinator_end_to_end_execution() {
    let config = BatchExecutionConfig::with_parallelism(4).unwrap();
    let executor = BoundedParallelExecutor::new(config).unwrap();
    let coordinator = IncrementalJobCoordinator::new(executor);

    let baseline = vec![
        FileSignature::new("README.md", "hash_readme_1", 500),
        FileSignature::new("src/app.ts", "hash_app_1", 1500),
        FileSignature::new("docs/old.md", "hash_old_1", 800),
    ];

    let candidates = vec![
        IncrementalCandidate {
            item_index: 0,
            path: "README.md".to_string(), // Unchanged -> Reused
            signature: FileSignature::new("README.md", "hash_readme_1", 500),
            payload: "body_readme_v1".to_string(),
        },
        IncrementalCandidate {
            item_index: 1,
            path: "src/app.ts".to_string(), // Modified -> Reprocessed
            signature: FileSignature::new("src/app.ts", "hash_app_2", 1800),
            payload: "body_app_v2".to_string(),
        },
        IncrementalCandidate {
            item_index: 2,
            path: "src/new.ts".to_string(), // Added -> Newly processed
            signature: FileSignature::new("src/new.ts", "hash_new_1", 1200),
            payload: "body_new_v1".to_string(),
        },
    ];

    let processor_invocations = Arc::new(AtomicUsize::new(0));
    let proc_inv_clone = processor_invocations.clone();

    let reuse_invocations = Arc::new(AtomicUsize::new(0));
    let reuse_inv_clone = reuse_invocations.clone();

    let batch_id = BatchId::new();

    let report = coordinator
        .execute_incremental(
            batch_id,
            candidates,
            &baseline,
            move |task| {
                let count = proc_inv_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::Relaxed);
                    Ok(format!("PROCESSED: {} [{}]", task.url, task.payload))
                }
            },
            move |sig| {
                let count = reuse_inv_clone.clone();
                let path = sig.path.clone();
                async move {
                    count.fetch_add(1, Ordering::Relaxed);
                    Ok(format!("REUSED_FROM_CACHE: {path}"))
                }
            },
        )
        .await
        .expect("incremental execution must succeed");

    // Processor was called ONLY for added & modified (2 items)
    assert_eq!(processor_invocations.load(Ordering::Relaxed), 2);
    // Reuse resolver was called for unchanged (1 item)
    assert_eq!(reuse_invocations.load(Ordering::Relaxed), 1);

    // Report assertions
    assert_eq!(report.batch_id, batch_id);
    assert_eq!(report.total_candidates, 3);
    assert_eq!(report.reused_count, 1);
    assert_eq!(report.processed_count, 2);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.deleted_count, 1); // docs/old.md was dropped
    assert_eq!(report.reused_bytes, 500);
    assert_eq!(report.processed_bytes, 3000); // 1800 + 1200
    assert!(report.conservation_verified());
    assert!(report.is_clean());
    assert!((report.cache_hit_ratio - (1.0 / 3.0)).abs() < 0.001);

    // Results preserve exact sequence ordering
    assert_eq!(report.all_results.len(), 3);
    assert_eq!(report.all_results[0], "REUSED_FROM_CACHE: README.md");
    assert_eq!(report.all_results[1], "PROCESSED: src/app.ts [body_app_v2]");
    assert_eq!(report.all_results[2], "PROCESSED: src/new.ts [body_new_v1]");
}

#[tokio::test]
async fn test_incremental_coordinator_with_failures_and_conservation() {
    let config = BatchExecutionConfig::with_parallelism(2).unwrap();
    let executor = BoundedParallelExecutor::new(config).unwrap();
    let coordinator = IncrementalJobCoordinator::new(executor);

    let baseline = vec![]; // All items are newly added

    let candidates = vec![
        IncrementalCandidate {
            item_index: 0,
            path: "file1.txt".to_string(),
            signature: FileSignature::new("file1.txt", "h1", 100),
            payload: true,
        },
        IncrementalCandidate {
            item_index: 1,
            path: "file2_fails.txt".to_string(),
            signature: FileSignature::new("file2_fails.txt", "h2", 200),
            payload: false, // will fail
        },
    ];

    let report = coordinator
        .execute_incremental(
            BatchId::new(),
            candidates,
            &baseline,
            |task| async move {
                if task.payload {
                    Ok(format!("OK: {}", task.url))
                } else {
                    Err("Simulated extraction syntax failure".to_string())
                }
            },
            |_| async move { Ok("REUSED".to_string()) },
        )
        .await
        .unwrap();

    assert_eq!(report.total_candidates, 2);
    assert_eq!(report.processed_count, 1);
    assert_eq!(report.failed_count, 1);
    assert_eq!(report.reused_count, 0);
    assert!(!report.is_clean());
    assert!(report.conservation_verified());

    assert_eq!(report.all_results.len(), 1);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].0, "file2_fails.txt");
    assert!(report.failures[0].1.contains("Simulated extraction"));
}

#[tokio::test]
async fn test_incremental_all_reused_and_empty_boundaries() {
    let config = BatchExecutionConfig::with_parallelism(2).unwrap();
    let executor = BoundedParallelExecutor::new(config).unwrap();
    let coordinator = IncrementalJobCoordinator::new(executor);

    // Empty candidates
    let empty_report = coordinator
        .execute_incremental(
            BatchId::new(),
            Vec::<IncrementalCandidate<()>>::new(),
            &[],
            |_| async move { Ok(()) },
            |_| async move { Ok(()) },
        )
        .await
        .unwrap();

    assert_eq!(empty_report.total_candidates, 0);
    assert_eq!(empty_report.reused_count, 0);
    assert_eq!(empty_report.cache_hit_ratio, 0.0);
    assert!(empty_report.conservation_verified());

    // 100% cache hit
    let baseline = vec![
        FileSignature::new("f1.txt", "hash1", 50),
        FileSignature::new("f2.txt", "hash2", 60),
    ];
    let candidates = vec![
        IncrementalCandidate {
            item_index: 0,
            path: "f1.txt".to_string(),
            signature: FileSignature::new("f1.txt", "hash1", 50),
            payload: (),
        },
        IncrementalCandidate {
            item_index: 1,
            path: "f2.txt".to_string(),
            signature: FileSignature::new("f2.txt", "hash2", 60),
            payload: (),
        },
    ];

    let full_hit_report = coordinator
        .execute_incremental(
            BatchId::new(),
            candidates,
            &baseline,
            |_| async move { panic!("Processor must NOT be called on 100% cache hit") },
            |sig| {
                let path = sig.path.clone();
                async move { Ok(format!("CACHED:{path}")) }
            },
        )
        .await
        .unwrap();

    assert_eq!(full_hit_report.total_candidates, 2);
    assert_eq!(full_hit_report.reused_count, 2);
    assert_eq!(full_hit_report.processed_count, 0);
    assert!((full_hit_report.cache_hit_ratio - 1.0).abs() < f64::EPSILON);
    assert_eq!(full_hit_report.reused_bytes, 110);
    assert!(full_hit_report.conservation_verified());
}
