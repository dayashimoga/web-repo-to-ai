//! # Storage Cleanup & Ephemeral Retention Verification Tests
//!
//! Evaluates the reliability, lifecycle bounds, ephemeral staging purges, capacity
//! watermarks, content-addressed storage (CAS) LRU pruning, job directory eradications,
//! and shutdown invariants across the `web-repo-to-ai` storage and worker subsystems:
//! - Ephemeral staging and temporary scratch file purges without affecting finalized artifacts.
//! - High/low storage capacity watermark enforcement preserving bounded storage budgets (<= 30 GB).
//! - Job directory eradication with zero orphan file leaks.
//! - Content-addressed storage (CAS) LRU capacity quota pruning with cryptographic audit integrity.
//! - Cooperative worker engine shutdown scratch drainage and sweep rejection under termination.
//! - Retention policy configuration boundaries and mathematical TTL monotonicity invariants.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use web_repo_to_ai_storage::backend::{DynStorage, InMemoryStorage, StorageBackend};
use web_repo_to_ai_storage::cas::{CasEngine, ContentAddressedStorage};
use web_repo_to_ai_storage::local_fs::layout;
use web_repo_to_ai_storage::local_fs::LocalFsStorage;
use web_repo_to_ai_storage::retention::{CleanupReport, RetentionManager, RetentionPolicy};
use web_repo_to_ai_storage::schema::ArtifactFormat;
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

/// RAII helper for scoped local filesystem temporary directories.
struct TestDir(PathBuf);

impl TestDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!("web-repo-to-ai-test-cleanup-{prefix}-{}", Uuid::now_v7()));
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

// ============================================================================
// TEST 1: Ephemeral Staging & Temporary Scratch File Purging
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_ephemeral_staging_purging() {
    // ------------------------------------------------------------------------
    // Part A: In-Memory Storage Staging Purging
    // ------------------------------------------------------------------------
    let mem_storage = Arc::new(InMemoryStorage::new());
    let policy = RetentionPolicy::default().with_ephemeral_ttl(Duration::hours(1));
    let manager = RetentionManager::new(mem_storage.clone(), policy).unwrap();

    let job_id = Uuid::now_v7();
    let final_key = format!("jobs/{job_id}/artifact.md");
    let staging_key1 = format!("jobs/{job_id}/.artifact.tmp.1001");
    let staging_key2 = format!("jobs/{job_id}/.export.tmp.1002");

    mem_storage
        .put(&final_key, b"# Finalized Markdown Document Content")
        .await
        .unwrap();
    mem_storage
        .put(&staging_key1, b"Incomplete write-in-progress chunk 1")
        .await
        .unwrap();
    mem_storage
        .put(&staging_key2, b"Incomplete write-in-progress chunk 2")
        .await
        .unwrap();

    // Verify all 3 files exist initially
    assert!(mem_storage.exists(&final_key).await.unwrap());
    assert!(mem_storage.exists(&staging_key1).await.unwrap());
    assert!(mem_storage.exists(&staging_key2).await.unwrap());

    // Run staging cleanup with zero max_age (purges all staging files)
    let staging_report = manager
        .cleanup_stale_staging_files(Duration::zero())
        .await
        .unwrap();

    assert_eq!(staging_report.ephemeral_files_deleted, 2);
    assert!(staging_report.ephemeral_bytes_freed > 0);
    assert!(staging_report.is_success());

    // Staging files must be eradicated while final artifact remains intact
    assert!(!mem_storage.exists(&staging_key1).await.unwrap());
    assert!(!mem_storage.exists(&staging_key2).await.unwrap());
    assert!(mem_storage.exists(&final_key).await.unwrap());
    let final_content = mem_storage.get(&final_key).await.unwrap();
    assert_eq!(final_content, b"# Finalized Markdown Document Content");

    // ------------------------------------------------------------------------
    // Part B: Local Filesystem Storage Staging Purging & Worker Engine Sweep
    // ------------------------------------------------------------------------
    let test_dir = TestDir::new("ephemeral_fs");
    let fs_storage: DynStorage = Arc::new(LocalFsStorage::new(test_dir.path()).unwrap());
    let fs_policy = RetentionPolicy::default();
    let fs_manager = RetentionManager::new(fs_storage.clone(), fs_policy).unwrap();

    let fs_job_id = Uuid::now_v7();
    let fs_valid_key = layout::job_artifact_key(fs_job_id, ArtifactFormat::Markdown);
    fs_storage
        .put(&fs_valid_key, b"# Production LocalFs Artifact Content")
        .await
        .unwrap();

    // Create orphaned staging temporary files directly on disk
    let job_fs_dir = test_dir.path().join(format!("jobs/{fs_job_id}"));
    std::fs::create_dir_all(&job_fs_dir).unwrap();
    let tmp_file_1 = job_fs_dir.join(".artifact.tmp.9991");
    let tmp_file_2 = job_fs_dir.join(".streaming.tmp.9992");
    tokio::fs::write(&tmp_file_1, b"Temporary aborted stream 1")
        .await
        .unwrap();
    tokio::fs::write(&tmp_file_2, b"Temporary aborted stream 2")
        .await
        .unwrap();

    assert!(tokio::fs::try_exists(&tmp_file_1).await.unwrap());
    assert!(tokio::fs::try_exists(&tmp_file_2).await.unwrap());

    // Execute retention sweep via WorkerEngine
    let worker_config = WorkerServiceConfig::for_worker("test-worker-retention", 2, 2).unwrap();
    let engine = WorkerEngine::new(worker_config).unwrap();

    // Run sweep with zero ephemeral TTL via custom manager
    let sweep_policy = RetentionPolicy::default().with_ephemeral_ttl(Duration::zero());
    let sweep_manager = RetentionManager::new(fs_storage.clone(), sweep_policy).unwrap();

    let sweep_report = engine
        .execute_retention_sweep(&sweep_manager, None, Utc::now())
        .await
        .expect("Worker retention sweep must succeed");

    assert_eq!(sweep_report.ephemeral_files_deleted, 2);
    assert_eq!(
        sweep_report.ephemeral_bytes_freed,
        b"Temporary aborted stream 1".len() as u64 + b"Temporary aborted stream 2".len() as u64
    );

    // Staging files must be removed, valid artifact preserved
    assert!(!tokio::fs::try_exists(&tmp_file_1).await.unwrap());
    assert!(!tokio::fs::try_exists(&tmp_file_2).await.unwrap());
    assert!(fs_storage.exists(&fs_valid_key).await.unwrap());

    // Verify engine telemetry recording
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.storage_cleanup_cycles, 1);
    assert_eq!(metrics.storage_files_purged, 2);
    assert!(metrics.storage_bytes_reclaimed > 0);
}

// ============================================================================
// TEST 2: Storage Capacity Watermark Enforcement (<= 30 GB Bound)
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_capacity_watermark_enforcement() {
    let storage = Arc::new(InMemoryStorage::new());

    // Configure low high-watermark of 40,000 bytes and low-watermark of 20,000 bytes
    let high_watermark = 40_000;
    let low_watermark = 20_000;
    let policy = RetentionPolicy::default().with_watermarks(high_watermark, low_watermark);
    let manager = RetentionManager::new(storage.clone(), policy).unwrap();

    // 1. Store 3 files of 10,000 bytes each = 30,000 bytes (below 40,000 high watermark)
    let payload = vec![b'A'; 10_000];
    storage.put("jobs/j1/artifact1.bin", &payload).await.unwrap();
    storage.put("jobs/j2/artifact2.bin", &payload).await.unwrap();
    storage.put("jobs/j3/artifact3.bin", &payload).await.unwrap();

    let initial_usage = storage.storage_capacity().await.unwrap();
    assert_eq!(initial_usage.total_artifact_bytes, 30_000);
    assert_eq!(initial_usage.artifact_count, 3);

    // Sweep while within budget: no files pruned
    let report_within_budget = manager
        .enforce_capacity_watermark(None, Utc::now())
        .await
        .unwrap();
    assert_eq!(report_within_budget.artifacts_deleted, 0);
    assert_eq!(report_within_budget.bytes_freed, 0);

    // 2. Add 2 more files of 10,000 bytes = 50,000 bytes (breaches high watermark 40,000)
    storage.put("jobs/j4/artifact4.bin", &payload).await.unwrap();
    storage.put("jobs/j5/artifact5.bin", &payload).await.unwrap();

    let exceeded_usage = storage.storage_capacity().await.unwrap();
    assert_eq!(exceeded_usage.total_artifact_bytes, 50_000);
    assert_eq!(exceeded_usage.artifact_count, 5);

    // 3. Enforce capacity watermark: must prune oldest files down to <= low watermark (20,000)
    let prune_report = manager
        .enforce_capacity_watermark(None, Utc::now())
        .await
        .unwrap();

    assert!(
        prune_report.artifacts_deleted >= 3,
        "Must prune at least 3 files (30,000 bytes) to drop from 50,000 down to <= 20,000"
    );
    assert!(
        prune_report.bytes_freed >= 30_000,
        "Must reclaim at least 30,000 bytes"
    );

    // 4. Validate final storage usage is strictly within target bound
    let final_usage = storage.storage_capacity().await.unwrap();
    assert!(
        final_usage.total_artifact_bytes <= low_watermark,
        "Final storage bytes ({}) must be <= low watermark ({})",
        final_usage.total_artifact_bytes,
        low_watermark
    );
    assert!(final_usage.artifact_count <= 2);
}

// ============================================================================
// TEST 3: Job Directory Purging & Zero-Orphan Isolation
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_job_directory_purging_and_zero_orphans() {
    let storage = Arc::new(InMemoryStorage::new());
    let policy = RetentionPolicy::default();
    let manager = RetentionManager::new(storage.clone(), policy).unwrap();

    let target_job_id = Uuid::now_v7();
    let sibling_job_id = Uuid::now_v7();

    // 1. Populate target job directory with all 7 primary output formats
    let target_artifacts = [
        format!("jobs/{target_job_id}/context.md"),
        format!("jobs/{target_job_id}/context.txt"),
        format!("jobs/{target_job_id}/document.json"),
        format!("jobs/{target_job_id}/index.html"),
        format!("jobs/{target_job_id}/package.zip"),
        format!("jobs/{target_job_id}/manifest.json"),
        format!("jobs/{target_job_id}/checksums.sha256"),
    ];

    for key in &target_artifacts {
        storage
            .put(key, format!("Payload for {key}").as_bytes())
            .await
            .unwrap();
        assert!(storage.exists(key).await.unwrap());
    }

    // 2. Populate sibling job directory
    let sibling_artifacts = [
        format!("jobs/{sibling_job_id}/context.md"),
        format!("jobs/{sibling_job_id}/document.json"),
        format!("jobs/{sibling_job_id}/manifest.json"),
    ];

    for key in &sibling_artifacts {
        storage
            .put(key, format!("Payload for {key}").as_bytes())
            .await
            .unwrap();
        assert!(storage.exists(key).await.unwrap());
    }

    // 3. Purge target job storage directory
    let purge_report = manager
        .cleanup_job_storage(target_job_id)
        .await
        .expect("Target job cleanup must succeed");

    assert!(purge_report.is_success());

    // 4. Assert zero orphan files remain under target job prefix
    for key in &target_artifacts {
        assert!(
            !storage.exists(key).await.unwrap(),
            "Target artifact '{key}' must no longer exist"
        );
    }

    let remaining_target_files = storage
        .list(Some(&format!("jobs/{target_job_id}")))
        .await
        .unwrap();
    assert!(
        remaining_target_files.is_empty(),
        "Target job directory must have 0 remaining files (zero orphans)"
    );

    // 5. Assert sibling job was completely untouched
    for key in &sibling_artifacts {
        assert!(
            storage.exists(key).await.unwrap(),
            "Sibling artifact '{key}' must remain undisturbed"
        );
    }
}

// ============================================================================
// TEST 4: Content-Addressed Storage (CAS) LRU Quota Pruning
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_cas_lru_quota_pruning() {
    let cas = CasEngine::in_memory();

    // 1. Store 8 unique blobs of 1,000 bytes each (total 8,000 bytes)
    let mut blake3_digests = Vec::new();
    for i in 0..8 {
        let payload = format!("CAS blob payload {i} padded to 1000 bytes: {:0<950}", i);
        let outcome = cas.put(payload.as_bytes()).await.unwrap();
        assert!(outcome.is_new);
        blake3_digests.push(outcome.blake3_hex);
    }

    let initial_stats = cas.stats().await.unwrap();
    assert_eq!(initial_stats.write_operations, 8);

    // 2. Enforce LRU quota ceiling of 3,500 bytes (must prune down from 8,000 to <= 3,500)
    let prune_report = cas
        .prune_lru(3500)
        .await
        .expect("CAS LRU prune must succeed");

    assert!(
        prune_report.pruned_count >= 5,
        "Must prune at least 5 blobs (5,000 bytes) to satisfy 3,500 byte quota (actual: {})",
        prune_report.pruned_count
    );
    assert!(
        prune_report.reclaimed_bytes >= 5000,
        "Must reclaim at least 5,000 bytes (actual: {})",
        prune_report.reclaimed_bytes
    );
    assert!(
        prune_report.final_bytes <= 3500,
        "Final CAS storage usage ({}) must be <= 3,500 bytes",
        prune_report.final_bytes
    );

    // 3. Audit CAS storage integrity after pruning: zero corruption permitted
    let audit_report = cas.audit().await.expect("CAS audit must succeed");
    assert!(
        audit_report.is_healthy(),
        "CAS storage must be healthy after LRU pruning"
    );
    assert_eq!(audit_report.missing, 0);
    assert_eq!(audit_report.corrupted, 0);
    assert!(audit_report.verified_valid <= 3);
}

// ============================================================================
// TEST 5: Cooperative Worker Engine Shutdown Scratch Drainage
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_cooperative_worker_shutdown_scratch_drain() {
    let storage = Arc::new(InMemoryStorage::new());
    let policy = RetentionPolicy::default();
    let manager = RetentionManager::new(storage.clone(), policy).unwrap();

    let worker_config = WorkerServiceConfig::for_worker("test-worker-shutdown-clean", 2, 2).unwrap();
    let engine = WorkerEngine::new(worker_config).unwrap();

    // Verify initial operational state
    assert!(!engine.is_shutting_down());

    // Initiate cooperative shutdown
    engine.shutdown();
    assert!(engine.is_shutting_down());

    // Attempting a retention sweep during shutdown must be cleanly rejected
    let sweep_result = engine
        .execute_retention_sweep(&manager, None, Utc::now())
        .await;

    assert!(
        sweep_result.is_err(),
        "Retention sweep must be rejected during worker shutdown"
    );

    // Metrics snapshot must remain coherent and accessible
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.active_jobs, 0);
}

// ============================================================================
// TEST 6: Retention Policy Boundaries & Monotonic TTL Invariants
// ============================================================================

#[tokio::test]
async fn test_storage_cleanup_ttl_boundary_and_monotonicity_invariants() {
    // ------------------------------------------------------------------------
    // Part A: Configuration Boundary Invariants
    // ------------------------------------------------------------------------
    let valid_policy = RetentionPolicy::default();
    assert!(valid_policy.validate().is_ok());
    assert_eq!(valid_policy.default_ttl, Duration::days(7));
    assert_eq!(valid_policy.max_ttl, Duration::days(365));
    assert_eq!(valid_policy.ephemeral_ttl, Duration::seconds(3600));

    // Invariant: default_ttl must be >= 60 seconds
    let tiny_ttl_policy = valid_policy.clone().with_default_ttl(Duration::seconds(30));
    assert!(tiny_ttl_policy.validate().is_err());

    // Invariant: max_ttl cannot be less than default_ttl
    let inverted_ttl_policy = valid_policy
        .clone()
        .with_default_ttl(Duration::days(30))
        .with_max_ttl(Duration::days(15));
    assert!(inverted_ttl_policy.validate().is_err());

    // Invariant: low_watermark_bytes must be strictly < high_watermark_bytes
    let inverted_watermarks = valid_policy.clone().with_watermarks(5000, 6000);
    assert!(inverted_watermarks.validate().is_err());

    let equal_watermarks = valid_policy.clone().with_watermarks(5000, 5000);
    assert!(equal_watermarks.validate().is_err());

    // Invariant: batch_size between 1 and 1000
    let zero_batch = valid_policy.clone().with_batch_size(0);
    assert!(zero_batch.validate().is_err());

    let excess_batch = valid_policy.clone().with_batch_size(2000);
    assert!(excess_batch.validate().is_err());

    // ------------------------------------------------------------------------
    // Part B: Mathematical TTL Monotonicity & Expiration Calculations
    // ------------------------------------------------------------------------
    let now = Utc::now();

    // 1. Unspecified requested TTL applies default 7 days
    let exp_default = valid_policy.calculate_expires_at(now, None);
    assert_eq!(exp_default, now + Duration::days(7));
    assert!(exp_default > now);

    // 2. Custom valid TTL within bounds (14 days)
    let exp_custom = valid_policy.calculate_expires_at(now, Some(Duration::days(14)));
    assert_eq!(exp_custom, now + Duration::days(14));
    assert!(exp_custom > exp_default);

    // 3. Requested TTL exceeding max TTL (400 days) is clamped to max TTL (365 days)
    let exp_clamped = valid_policy.calculate_expires_at(now, Some(Duration::days(400)));
    assert_eq!(exp_clamped, now + Duration::days(365));
    assert!(exp_clamped > exp_custom);

    // 4. Negative or zero requested TTL safely falls back to default TTL
    let exp_zero = valid_policy.calculate_expires_at(now, Some(Duration::zero()));
    assert_eq!(exp_zero, now + Duration::days(7));

    let exp_neg = valid_policy.calculate_expires_at(now, Some(Duration::days(-5)));
    assert_eq!(exp_neg, now + Duration::days(7));

    // ------------------------------------------------------------------------
    // Part C: CleanupReport Arithmetic & Audit Invariants
    // ------------------------------------------------------------------------
    let mut r1 = CleanupReport::new();
    assert!(r1.is_empty());
    assert!(r1.is_success());

    r1.record_artifact_deleted(2048);
    r1.record_ephemeral_deleted(1024);
    assert_eq!(r1.total_files_deleted(), 2);
    assert_eq!(r1.total_bytes_freed(), 3072);

    let mut r2 = CleanupReport::new();
    r2.record_artifact_deleted(4096);
    r2.record_error("Permission denied on storage volume");
    assert!(r2.has_errors());
    assert!(!r2.is_success());

    r1.merge(r2);
    assert_eq!(r1.artifacts_deleted, 2);
    assert_eq!(r1.ephemeral_files_deleted, 1);
    assert_eq!(r1.total_files_deleted(), 3);
    assert_eq!(r1.total_bytes_freed(), 7168);
    assert_eq!(r1.errors.len(), 1);
    assert!(!r1.is_success());
}

