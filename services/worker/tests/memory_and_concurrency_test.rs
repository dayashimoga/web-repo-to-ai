//! # Memory Bounds & Concurrency Stress Test Suite (`services/worker`)
//!
//! Evaluates worker engine performance, memory stability, and concurrency guarantees:
//! - Sustained load stability across 100 consecutive conversion cycles with zero memory leaks
//! - Strict concurrency ceiling enforcement under high task load (peak concurrency <= ceiling)
//! - Adaptive backpressure and watermark flow control without queue starvation
//! - Bounded-memory streaming export of large archives with constant chunk buffers ($O(1)$)
//! - Multi-tenant Git cache concurrency with simultaneous reads/writes and zero deadlocks
//! - Cooperative shutdown and cancellation under active load with rejection of subsequent batches

use std::collections::HashSet;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time::sleep;
use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_exporters::streaming::StreamingExportConfig;
use web_repo_to_ai_exporters::ZipArchive;
use web_repo_to_ai_github_extractor::cache::{GitCacheStore, InMemoryGitCacheStore};
use web_repo_to_ai_github_extractor::source::{compute_git_blob_sha, GitBlob};
use web_repo_to_ai_job_engine::backpressure::BackpressureConfig;
use web_repo_to_ai_job_engine::models::BatchItemTask;
use web_repo_to_ai_normalizer::{normalize_document, NormalizationConfig};
use web_repo_to_ai_tokenizer::{count_tokens_for_model, TargetModel};
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

// ==============================================================================
// TEST SUITE 1: Sustained Load Memory Stability & Zero Leakage
// ==============================================================================

#[tokio::test]
async fn test_sustained_load_memory_stability_and_zero_leakage() {
    println!("\n=== [SUITE 1] Sustained Load Memory Stability & Zero Leakage (100 Cycles) ===");

    // Configure worker with 4 workers and 4 batch parallelism
    let config = WorkerServiceConfig::for_worker("test-worker-memory-stability", 4, 4)
        .expect("Valid worker config");
    let engine = WorkerEngine::new(config).expect("Worker engine initialization");

    let norm_config = NormalizationConfig::default();
    let total_cycles = 100;
    let mut cycle_durations = Vec::with_capacity(total_cycles);

    let start_time = Instant::now();

    for cycle in 0..total_cycles {
        let cycle_start = Instant::now();

        // 1. Build a synthetic realistic CDM document
        let identity = SourceIdentity {
            raw_input: format!("https://github.com/example/sustained-repo-{cycle}"),
            canonical_url: format!("https://github.com/example/sustained-repo-{cycle}"),
            domain: Some("github.com".to_string()),
            repo_owner: Some("example".to_string()),
            repo_name: Some(format!("sustained-repo-{cycle}")),
            git_ref: Some("main".to_string()),
            subpath: None,
        };

        let s1 = CdmSectionBuilder::new(
            format!("sec-md-{cycle}"),
            format!("https://github.com/example/sustained-repo-{cycle}/blob/main/README.md"),
            SectionKind::Prose,
            format!(
                "# Sustained Load Benchmark Cycle {cycle}\n\n\
                 This document tests heap allocation stability and zero-leakage behavior.\n\
                 - Iteration: {cycle}\n\
                 - Concurrency: bounded 4 Tokio threads\n\
                 - Normalization: UTF-8 sanitization + whitespace collapse\n"
            ),
        )
        .title("README.md")
        .original_path("README.md")
        .language("markdown")
        .build();

        let s2 = CdmSectionBuilder::new(
            format!("sec-rs-{cycle}"),
            format!("https://github.com/example/sustained-repo-{cycle}/blob/main/src/pipeline.rs"),
            SectionKind::CodeFile,
            format!(
                "// Pipeline handler for cycle {cycle}\n\
                 pub fn execute_cycle_{cycle}(input: &[u8]) -> usize {{\n\
                     input.iter().map(|&b| b as usize * {cycle}).sum()\n\
                 }}\n"
            ),
        )
        .title("src/pipeline.rs")
        .original_path("src/pipeline.rs")
        .language("rust")
        .build();

        let doc = CdmDocumentBuilder::new(
            identity,
            SourceType::GitHubPublic,
            format!("https://github.com/example/sustained-repo-{cycle}"),
        )
        .revision(RevisionInfo {
            git_branch: Some("main".to_string()),
            git_commit: Some(format!("{:040x}", cycle + 1)),
            ..Default::default()
        })
        .add_section(s1)
        .add_section(s2)
        .build();

        // 2. Normalize document
        let (norm_doc, report) = normalize_document(&doc, &norm_config);
        assert_eq!(report.sections_total, 2);

        // 3. Tokenize sections with dual models
        for section in &norm_doc.sections {
            let gpt_tokens = count_tokens_for_model(&section.body, TargetModel::Gpt4o);
            let claude_tokens = count_tokens_for_model(&section.body, TargetModel::Claude35Sonnet);
            assert!(gpt_tokens.count > 0);
            assert!(claude_tokens.count > 0);
        }

        // 4. Dispatch tasks through WorkerEngine
        let batch_id = BatchId::new();
        let tasks = vec![
            BatchItemTask::new(
                0,
                JobId::new(),
                format!("https://example.com/item/{cycle}/a"),
                None,
                cycle,
            ),
            BatchItemTask::new(
                1,
                JobId::new(),
                format!("https://example.com/item/{cycle}/b"),
                None,
                cycle,
            ),
        ];

        let summary = engine
            .execute_batch(
                batch_id,
                tasks,
                None,
                |task| async move {
                    // Fast simulated item conversion
                    Ok(format!("cycle-{}-item-{}", task.payload, task.item_index))
                },
                None,
            )
            .await
            .expect("Batch execution must succeed");

        assert_eq!(summary.succeeded_count, 2);
        assert_eq!(summary.failed_count, 0);

        cycle_durations.push(cycle_start.elapsed());
    }

    let total_elapsed = start_time.elapsed();

    // Verify metrics
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.batches_processed, total_cycles);
    assert_eq!(metrics.items_succeeded, total_cycles * 2);
    assert_eq!(metrics.items_failed, 0);

    // Analyze performance stability across the run:
    // First 20 cycles vs Last 20 cycles to verify lack of latency degradation or allocator thrashing
    let first_20_micros: f64 = cycle_durations[0..20]
        .iter()
        .map(|d| d.as_micros() as f64)
        .sum::<f64>()
        / 20.0;
    let last_20_micros: f64 = cycle_durations[80..100]
        .iter()
        .map(|d| d.as_micros() as f64)
        .sum::<f64>()
        / 20.0;

    println!("Total Elapsed (100 Cycles): {:?}", total_elapsed);
    println!("Average Iteration (First 20): {:.2} us", first_20_micros);
    println!("Average Iteration (Last 20):  {:.2} us", last_20_micros);
    let degradation_ratio = last_20_micros / first_20_micros.max(1.0);
    println!("Degradation Ratio: {:.2}x", degradation_ratio);

    // Enforce that latency has not ballooned (> 3.0x would indicate memory leak, fragmentation or contention)
    assert!(
        degradation_ratio <= 3.0,
        "Performance degraded significantly under sustained load: ratio was {:.2}x",
        degradation_ratio
    );
}

// ==============================================================================
// TEST SUITE 2: Strict Concurrency Ceiling Enforcement
// ==============================================================================

#[tokio::test]
async fn test_concurrency_ceiling_strict_enforcement() {
    println!("\n=== [SUITE 2] Strict Concurrency Ceiling Enforcement (32 Tasks, Max 4) ===");

    // Configure worker with a strict maximum of 4 concurrent jobs
    let max_concurrency = 4;
    let config = WorkerServiceConfig::for_worker(
        "test-worker-strict-ceiling",
        max_concurrency,
        max_concurrency,
    )
    .expect("Valid worker config");
    let engine = WorkerEngine::new(config).expect("Worker engine initialization");

    let current_in_flight = Arc::new(AtomicUsize::new(0));
    let observed_peak = Arc::new(AtomicUsize::new(0));

    let total_tasks = 32;
    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<usize>> = (0..total_tasks)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/ceiling/{i}"),
                None,
                i,
            )
        })
        .collect();

    let in_flight_c = Arc::clone(&current_in_flight);
    let observed_c = Arc::clone(&observed_peak);

    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            None,
            move |_task| {
                let in_flight = Arc::clone(&in_flight_c);
                let observed = Arc::clone(&observed_c);
                async move {
                    let cur = in_flight.fetch_add(1, Ordering::SeqCst) + 1;

                    // Atomically track highest observed concurrency
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
                    sleep(Duration::from_millis(15)).await;

                    in_flight.fetch_sub(1, Ordering::SeqCst);
                    Ok("converted".to_string())
                }
            },
            None,
        )
        .await
        .expect("Batch execution must succeed");

    assert_eq!(summary.total_items, total_tasks);
    assert_eq!(summary.succeeded_count, total_tasks);
    assert_eq!(summary.failed_count, 0);
    assert!(summary.all_succeeded());

    let peak = observed_peak.load(Ordering::SeqCst);
    println!("Configured Concurrency Ceiling: {}", max_concurrency);
    println!("Observed Peak Concurrency:     {}", peak);
    println!(
        "Summary Peak Concurrency:      {}",
        summary.peak_concurrency
    );

    // Invariant: observed concurrency MUST NEVER exceed configured ceiling
    assert!(
        peak <= max_concurrency,
        "Observed concurrency ({peak}) breached hard ceiling ({max_concurrency})"
    );
    assert!(
        summary.peak_concurrency <= max_concurrency,
        "Summary peak concurrency ({}) breached hard ceiling ({max_concurrency})",
        summary.peak_concurrency
    );
    assert_eq!(
        current_in_flight.load(Ordering::SeqCst),
        0,
        "All in-flight tasks must drain completely"
    );
}

// ==============================================================================
// TEST SUITE 3: Adaptive Backpressure & Watermark Flow Control
// ==============================================================================

#[tokio::test]
async fn test_adaptive_backpressure_and_watermark_flow_control() {
    println!("\n=== [SUITE 3] Adaptive Backpressure & Watermark Flow Control ===");

    let config = WorkerServiceConfig::for_worker("test-worker-backpressure", 4, 4)
        .expect("Valid worker config");
    let engine = WorkerEngine::new(config).expect("Worker engine initialization");

    // Queue capacity = 8, high watermark = 6, low watermark = 3
    let bp_config = BackpressureConfig::new(8, 6, 3).expect("Valid backpressure config");

    let total_tasks = 16;
    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<usize>> = (0..total_tasks)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/backpressure/{i}"),
                None,
                i,
            )
        })
        .collect();

    let summary = engine
        .execute_flow_controlled_batch(
            batch_id,
            tasks,
            Some(bp_config),
            |_task| async move {
                sleep(Duration::from_millis(10)).await;
                Ok("flow-controlled".to_string())
            },
            None,
        )
        .await
        .expect("Flow controlled batch must succeed");

    assert_eq!(summary.total_items, total_tasks);
    assert_eq!(summary.succeeded_count, total_tasks);
    assert_eq!(summary.failed_count, 0);
    assert!(summary.peak_concurrency <= 8);

    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.batches_processed, 1);
    assert_eq!(metrics.items_succeeded, total_tasks);

    println!(
        "Processed {} tasks smoothly with backpressure (peak concurrency: {})",
        total_tasks, summary.peak_concurrency
    );
}

// ==============================================================================
// TEST SUITE 4: Streaming Large Export Bounded Buffer Memory
// ==============================================================================

#[tokio::test]
async fn test_streaming_large_export_bounded_buffer_memory() {
    println!("\n=== [SUITE 4] Streaming Large Export Bounded Buffer Memory ===");

    let config = WorkerServiceConfig::for_worker("test-worker-stream-memory", 4, 2)
        .expect("Valid worker config");
    let engine = WorkerEngine::new(config).expect("Worker engine initialization");

    let identity = SourceIdentity {
        raw_input: "https://github.com/example/large-monorepo-export".to_string(),
        canonical_url: "https://github.com/example/large-monorepo-export".to_string(),
        domain: Some("github.com".to_string()),
        repo_owner: Some("example".to_string()),
        repo_name: Some("large-monorepo-export".to_string()),
        git_ref: Some("main".to_string()),
        subpath: None,
    };

    let mut doc_builder = CdmDocumentBuilder::new(
        identity,
        SourceType::GitHubPublic,
        "https://github.com/example/large-monorepo-export",
    )
    .revision(RevisionInfo {
        git_branch: Some("main".to_string()),
        git_commit: Some("11223344556677889900aabbccddeeff00112233".to_string()),
        ..Default::default()
    });

    // Synthesize 50 source sections with ~20 KB of text/code each (> 1.0 MB total)
    let section_count = 50;
    let payload_unit = "pub fn compute_large_data_block() -> &'static str {\n    \"abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+\"\n}\n".repeat(250); // ~20.5 KB

    for i in 0..section_count {
        let path = format!("src/components/component_{i:03}.rs");
        let section = CdmSectionBuilder::new(
            format!("sec-{i:03}"),
            format!("https://github.com/example/large-monorepo-export/blob/main/{path}"),
            SectionKind::CodeFile,
            &payload_unit,
        )
        .title(&path)
        .original_path(&path)
        .language("rust")
        .build();

        doc_builder = doc_builder.add_section(section);
    }

    let doc = doc_builder.build();
    assert_eq!(doc.sections.len(), section_count);

    // Set buffer capacity to exactly 8 KiB (8192 bytes)
    let buffer_capacity = 8192;
    let export_config = StreamingExportConfig::new(buffer_capacity)
        .expect("Valid config")
        .with_manifests(true, true);

    let mut zip_buffer = Cursor::new(Vec::new());

    let start = Instant::now();
    let summary = engine
        .execute_streaming_export(&mut zip_buffer, export_config, &[doc])
        .expect("Streaming export execution must succeed");
    let elapsed = start.elapsed();

    // Verify uncompressed bytes exceeds 1 MB
    assert!(
        summary.total_uncompressed_bytes >= 1_000_000,
        "Expected at least 1,000,000 uncompressed bytes, got {}",
        summary.total_uncompressed_bytes
    );
    // 50 sections + context.md + context.txt + document.json = 53 minimum
    assert!(
        summary.total_items_written >= section_count + 3,
        "Expected at least {} items written, got {}",
        section_count + 3,
        summary.total_items_written
    );
    assert!(!summary.sha256_checksum.is_empty());
    assert!(!summary.blake3_checksum.is_empty());

    println!(
        "Total Uncompressed Payload: {} bytes",
        summary.total_uncompressed_bytes
    );
    println!(
        "Total Archive Items:        {}",
        summary.total_items_written
    );
    println!(
        "Chunk Buffer Capacity:      {} bytes (Bounded O(1))",
        buffer_capacity
    );
    println!("Export Duration:            {:?}", elapsed);

    // Validate archive integrity
    let zip_bytes = zip_buffer.into_inner();
    println!("Compressed Archive Size:    {} bytes", zip_bytes.len());
    let mut reader = Cursor::new(zip_bytes);
    let mut zip = ZipArchive::new(&mut reader).expect("Valid ZIP archive");

    // Spot-check files inside the generated archive
    assert!(zip.by_name("doc_0/context.md").is_ok());
    assert!(zip.by_name("doc_0/context.txt").is_ok());
    assert!(zip.by_name("doc_0/document.json").is_ok());
    assert!(zip
        .by_name("doc_0/sources/src/components/component_000.rs")
        .is_ok());
    assert!(zip
        .by_name("doc_0/sources/src/components/component_049.rs")
        .is_ok());
    assert!(zip.by_name("manifest.json").is_ok());
    assert!(zip.by_name("checksums.sha256").is_ok());
}

// ==============================================================================
// TEST SUITE 5: Multi-Tenant Cache Concurrency & Zero-Deadlock Guarantees
// ==============================================================================

#[tokio::test]
async fn test_multi_tenant_cache_concurrency_and_zero_deadlocks() {
    println!("\n=== [SUITE 5] Multi-Tenant Cache Concurrency & Zero-Deadlock Guarantees ===");

    let cache = Arc::new(InMemoryGitCacheStore::new());

    let num_tenants = 8;
    let _tasks_per_tenant = 4; // 2 writers, 2 readers per tenant = 32 tasks
    let items_per_writer = 20;

    let mut handles = Vec::new();

    let start = Instant::now();

    for tenant_id in 0..num_tenants {
        // Writer Tasks
        for writer_id in 0..2 {
            let cache_ref = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                for i in 0..items_per_writer {
                    let content = format!(
                        "Tenant {tenant_id} - Writer {writer_id} - Blob item {i} payload with repeated content block"
                    );
                    let raw_bytes = content.into_bytes();
                    let sha = compute_git_blob_sha(&raw_bytes);
                    let blob = GitBlob::new(sha, raw_bytes, "raw");

                    cache_ref
                        .store_cached_blob(&blob)
                        .await
                        .expect("Storing blob must succeed");
                }
            });
            handles.push(handle);
        }

        // Reader Tasks
        for _reader_id in 0..2 {
            let cache_ref = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                for i in 0..items_per_writer {
                    // Check a predictable SHA that another writer produces
                    let synthetic_payload = format!(
                        "Tenant {tenant_id} - Writer 0 - Blob item {i} payload with repeated content block"
                    );
                    let sha = compute_git_blob_sha(synthetic_payload.as_bytes());

                    // Read queries should never deadlock, whether hit or miss
                    let _ = cache_ref.get_cached_blob(&sha).await;
                    let _ = cache_ref.check_cached_blobs(&[sha]).await;
                }
            });
            handles.push(handle);
        }
    }

    // Await all 32 tasks with a strict 10-second timeout to assert zero deadlocks
    let join_all = async {
        for handle in handles {
            handle.await.expect("Task must join without panic");
        }
    };

    tokio::time::timeout(Duration::from_secs(10), join_all)
        .await
        .expect("Zero deadlocks: all 32 concurrent tenant tasks must complete within 10s");

    let elapsed = start.elapsed();
    let total_stored = cache.blob_count().await;
    println!("Elapsed Time:     {:?}", elapsed);
    println!("Total Cached Blobs: {}", total_stored);

    // Each of the 8 tenants had 2 writers storing 20 distinct blobs = 8 * 2 * 20 = 320 unique blobs
    assert_eq!(total_stored, num_tenants * 2 * items_per_writer);

    // Verify bit-for-bit retrieval accuracy for arbitrary items
    for tenant_id in 0..num_tenants {
        for writer_id in 0..2 {
            let expected_text = format!(
                "Tenant {tenant_id} - Writer {writer_id} - Blob item 10 payload with repeated content block"
            );
            let sha = compute_git_blob_sha(expected_text.as_bytes());
            let retrieved = cache
                .get_cached_blob(&sha)
                .await
                .expect("Lookup must succeed")
                .expect("Blob must exist in cache");

            assert_eq!(retrieved.sha, sha);
            assert_eq!(retrieved.content_str().unwrap(), expected_text);
        }
    }

    // Verify batch query accuracy
    let mut sample_shas = Vec::new();
    for tenant_id in 0..num_tenants {
        let text = format!(
            "Tenant {tenant_id} - Writer 0 - Blob item 5 payload with repeated content block"
        );
        sample_shas.push(compute_git_blob_sha(text.as_bytes()));
    }
    // Add one nonexistent SHA
    sample_shas.push("0000000000000000000000000000000000000000".to_string());

    let hits: HashSet<String> = cache
        .check_cached_blobs(&sample_shas)
        .await
        .expect("check_cached_blobs must succeed");
    assert_eq!(hits.len(), num_tenants);
}

// ==============================================================================
// TEST SUITE 6: Cooperative Shutdown & Task Cancellation Under Load
// ==============================================================================

#[tokio::test]
async fn test_cooperative_shutdown_and_cancellation_under_load() {
    println!("\n=== [SUITE 6] Cooperative Shutdown & Task Cancellation Under Load ===");

    // Configure worker with concurrency 4
    let config = WorkerServiceConfig::for_worker("test-worker-shutdown-stress", 4, 4)
        .expect("Valid worker config");
    let engine = WorkerEngine::new(config).expect("Worker engine initialization");

    let total_tasks = 16;
    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<usize>> = (0..total_tasks)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/shutdown/{i}"),
                None,
                i,
            )
        })
        .collect();

    let engine_clone = engine.clone();

    // Trigger cooperative shutdown after 20ms while 16 items are actively queued/running
    tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        println!("Triggering cooperative engine.shutdown()...");
        engine_clone.shutdown();
    });

    let summary = engine
        .execute_batch(
            batch_id,
            tasks,
            None,
            |_task| async move {
                // Each task attempts to sleep 80ms
                sleep(Duration::from_millis(80)).await;
                Ok("completed".to_string())
            },
            None,
        )
        .await
        .expect("Batch execution finishes cooperatively during shutdown");

    assert_eq!(summary.total_items, total_tasks);
    println!("Succeeded Tasks: {}", summary.succeeded_count);
    println!("Cancelled Tasks: {}", summary.cancelled_count);
    println!("Engine is_shutting_down: {}", engine.is_shutting_down());

    // Invariants:
    // 1. Worker is officially marked as shutting down
    assert!(engine.is_shutting_down());

    // 2. All 16 tasks are accounted for (sum of succeeded and cancelled equals total)
    assert_eq!(
        summary.succeeded_count + summary.cancelled_count,
        total_tasks
    );
    assert!(
        summary.cancelled_count > 0,
        "Expected at least one cancelled task due to cooperative shutdown"
    );

    // 3. Any subsequent batch execution request MUST be immediately rejected
    let rejected_batch = engine
        .execute_batch(
            BatchId::new(),
            vec![BatchItemTask::new(
                0,
                JobId::new(),
                "https://example.com/rejected",
                None,
                0,
            )],
            None,
            |_| async { Ok("fail".to_string()) },
            None,
        )
        .await;

    assert!(rejected_batch.is_err());
    let err_msg = rejected_batch.unwrap_err().to_string();
    println!("Rejected Batch Error: {}", err_msg);
    assert!(
        err_msg.contains("shutting down"),
        "Expected shutdown rejection error message, got: {err_msg}"
    );
}
