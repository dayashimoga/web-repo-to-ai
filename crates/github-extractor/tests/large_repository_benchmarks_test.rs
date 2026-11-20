//! # Large Repository Performance Benchmarks (`large_repository_benchmarks_test.rs`)
//!
//! Subphase 11.2.2: Automated large repository benchmarks, latency percentiles (P50, P90, P95, P99),
//! throughput metrics (MB/s, files/s), and SLA compliance verification across:
//! - Large Git trees traversal and recursive truncation subtree stitching (15,000+ items)
//! - Polyglot monorepo file classification, ignore filtering, and category breakdown (12,500+ items)
//! - Multi-algorithm content hashing (BLAKE3, SHA-256, Git blob SHA-1)
//! - In-memory ZIP archive ingestion and directory scanning
//! - File accounting balance sheet, discrepancy detection, and mathematical completeness conservation (20,000+ files)
//! - Concurrent multi-repository pipeline execution scaling (1, 2, 4 worker tasks)

#![allow(missing_docs)]

use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use web_repo_to_ai_github_extractor::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, scan_zip_archive,
    FileAccountingAuditor, FileCategory, FileClassificationConfig, FileClassifier,
    FileExtractionStatus, GitFileMode, GitTreeEntry, GitTreeEntryType, GitTreeResponse,
    ManifestFileEntry, RepoFileManifest, TreeStitcher,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ==============================================================================
// STATISTICAL BENCHMARK HARNESS
// ==============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub name: String,
    pub iterations: usize,
    pub total_items: usize,
    pub total_bytes: usize,
    pub total_duration: Duration,
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub throughput_mb_s: f64,
    pub items_per_s: f64,
}

impl BenchmarkStats {
    pub fn calculate(
        name: impl Into<String>,
        total_items: usize,
        total_bytes: usize,
        mut durations: Vec<Duration>,
    ) -> Self {
        assert!(!durations.is_empty(), "Durations must not be empty");
        durations.sort();

        let iterations = durations.len();
        let total_duration: Duration = durations.iter().sum();
        let min = durations[0];
        let max = durations[iterations - 1];
        let mean = total_duration / (iterations as u32);

        let p50_idx = ((iterations as f64) * 0.50).floor() as usize;
        let p90_idx = ((iterations as f64) * 0.90).floor() as usize;
        let p95_idx = ((iterations as f64) * 0.95).floor() as usize;
        let p99_idx = ((iterations as f64) * 0.99).floor() as usize;

        let p50 = durations[p50_idx.min(iterations - 1)];
        let p90 = durations[p90_idx.min(iterations - 1)];
        let p95 = durations[p95_idx.min(iterations - 1)];
        let p99 = durations[p99_idx.min(iterations - 1)];

        let total_secs = total_duration.as_secs_f64();
        let throughput_mb_s = if total_secs > 0.0 && total_bytes > 0 {
            (total_bytes as f64 / (1024.0 * 1024.0)) / total_secs
        } else {
            0.0
        };

        let items_per_s = if total_secs > 0.0 && total_items > 0 {
            total_items as f64 / total_secs
        } else {
            0.0
        };

        let stats = Self {
            name: name.into(),
            iterations,
            total_items,
            total_bytes,
            total_duration,
            min,
            max,
            mean,
            p50,
            p90,
            p95,
            p99,
            throughput_mb_s,
            items_per_s,
        };
        stats.print_summary();
        stats
    }

    pub fn print_summary(&self) {
        println!(
            "\n================================================================================"
        );
        println!("BENCHMARK: {}", self.name);
        println!(
            "  Iterations: {:<5} | Items: {:<10} | Total Data: {:<8.2} KB | Total Time: {:.2} ms",
            self.iterations,
            self.total_items,
            self.total_bytes as f64 / 1024.0,
            self.total_duration.as_secs_f64() * 1000.0
        );
        println!(
            "  Latency:    Min: {:.2} ms | Mean: {:.2} ms | P50: {:.2} ms | P90: {:.2} ms | P95: {:.2} ms | P99: {:.2} ms | Max: {:.2} ms",
            self.min.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1000.0,
            self.p50.as_secs_f64() * 1000.0,
            self.p90.as_secs_f64() * 1000.0,
            self.p95.as_secs_f64() * 1000.0,
            self.p99.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0
        );
        println!(
            "  Throughput: {:.2} MB/s | Processing Rate: {:.1} items/s",
            self.throughput_mb_s, self.items_per_s
        );
        println!(
            "================================================================================"
        );
    }

    pub fn assert_p95_under(&self, limit: Duration) {
        assert!(
            self.p95 <= limit,
            "SLA VIOLATION for '{}': P95 latency {:?} exceeded ceiling {:?}",
            self.name,
            self.p95,
            limit
        );
    }

    pub fn assert_items_rate_over(&self, min_items_s: f64) {
        assert!(
            self.items_per_s >= min_items_s,
            "SLA VIOLATION for '{}': Rate {:.1} items/s was below floor {:.1} items/s",
            self.name,
            self.items_per_s,
            min_items_s
        );
    }

    pub fn assert_throughput_over(&self, min_mb_s: f64) {
        assert!(
            self.throughput_mb_s >= min_mb_s,
            "SLA VIOLATION for '{}': Throughput {:.2} MB/s was below floor {:.2} MB/s",
            self.name,
            self.throughput_mb_s,
            min_mb_s
        );
    }
}

// ==============================================================================
// SYNTHETIC GENERATORS
// ==============================================================================

fn generate_large_synthetic_monorepo_tree(
    package_count: usize,
    modules_per_package: usize,
    files_per_module: usize,
) -> Vec<GitTreeEntry> {
    let extensions = [
        "rs", "ts", "tsx", "py", "go", "cpp", "java", "md", "json", "yaml", "toml", "lock",
    ];
    let total_entries = package_count * modules_per_package * files_per_module;
    let mut entries = Vec::with_capacity(total_entries + package_count * 2);

    for p in 0..package_count {
        // Root package manifests
        entries.push(GitTreeEntry::new(
            format!("packages/pkg_{p:02}/Cargo.toml"),
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            format!("{p:040x}"),
            Some(450),
        ));
        entries.push(GitTreeEntry::new(
            format!("packages/pkg_{p:02}/package.json"),
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            format!("{:040x}", p + 100),
            Some(600),
        ));

        for m in 0..modules_per_package {
            for f in 0..files_per_module {
                let idx = p * (modules_per_package * files_per_module) + m * files_per_module + f;
                let ext = extensions[idx % extensions.len()];
                let path = format!("packages/pkg_{p:02}/src/mod_{m:02}/file_{f:03}.{ext}");
                let sha = format!("{idx:040x}");
                let size = (100 + (idx % 2000)) as u64;

                entries.push(GitTreeEntry::new(
                    path,
                    GitFileMode::RegularFile,
                    GitTreeEntryType::Blob,
                    sha,
                    Some(size),
                ));
            }
        }
    }
    entries
}

fn generate_synthetic_zip_archive(files_count: usize) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(buf);

    let base_dir = "org-repo-c0ffee1/";
    let dir_options = SimpleFileOptions::default()
        .system(zip::System::Unix)
        .unix_permissions(0o755);
    let file_options = SimpleFileOptions::default()
        .system(zip::System::Unix)
        .unix_permissions(0o644);

    writer
        .add_directory(base_dir, dir_options)
        .expect("Failed to create base dir in zip");

    for i in 0..files_count {
        let pkg = i % 10;
        let sub = (i / 10) % 5;
        let path = format!("{base_dir}packages/pkg_{pkg:02}/src/sub_{sub:02}/component_{i:04}.rs");
        let content = format!(
            "// Synthetic source file {i}\npub fn component_{i}() -> usize {{ {i} * 42 }}\n"
        );

        writer
            .start_file(path, file_options)
            .expect("Failed to start zip file");
        writer
            .write_all(content.as_bytes())
            .expect("Failed to write zip content");
    }

    writer
        .finish()
        .expect("Failed to finalize zip archive")
        .into_inner()
}

// ==============================================================================
// 1. LARGE GIT TREE TRAVERSAL & TRUNCATION SUBTREE STITCHING BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_large_git_tree_traversal_and_truncation_stitching() {
    // 50 packages x 10 modules x 30 files = 15,000 files
    let full_entries = generate_large_synthetic_monorepo_tree(50, 10, 30);
    let total_items = full_entries.len();
    assert_eq!(total_items, 15_100); // 15,000 files + 100 package manifests

    // Simulate root tree returning first 10,000 entries with truncated = true
    let root_chunk_size = 10_000;
    let root_entries = full_entries[..root_chunk_size].to_vec();

    // 5 subtrees recovering the remaining 5,100 items (1,000 or 1,100 items each)
    let subtree_chunks: Vec<GitTreeResponse> = full_entries[root_chunk_size..]
        .chunks(1_020)
        .enumerate()
        .map(|(idx, chunk)| {
            GitTreeResponse::new(format!("sub_commit_{idx}"), chunk.to_vec(), false)
        })
        .collect();

    let iterations = 20;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let mut base_tree = GitTreeResponse::new("root_commit_sha", root_entries.clone(), true);
        assert!(base_tree.truncated);

        let mut total_blobs_added = 0;
        for subtree in &subtree_chunks {
            let (blobs, _) = TreeStitcher::stitch_subtree(&mut base_tree, "", subtree);
            total_blobs_added += blobs;
        }

        assert_eq!(total_blobs_added, full_entries.len() - root_chunk_size);
        assert_eq!(base_tree.tree.len(), full_entries.len());
        durations.push(start.elapsed());
    }

    let stats = BenchmarkStats::calculate(
        "Large Git Tree Traversal & Truncation Subtree Stitching (15,100 items)",
        total_items * iterations,
        0,
        durations,
    );

    // Stitching 5,100 subtrees into a 10,000-item base tree must complete in under 75 ms
    stats.assert_p95_under(Duration::from_millis(75));
    stats.assert_items_rate_over(150_000.0);
}

// ==============================================================================
// 2. POLYGLOT MONOREPO FILE CLASSIFICATION & FILTERING BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_polyglot_monorepo_classification_and_filtering() {
    // 50 packages x 10 modules x 25 files = 12,500 files + 100 manifests
    let full_entries = generate_large_synthetic_monorepo_tree(50, 10, 25);
    let total_files = full_entries.len();
    let total_bytes: usize = full_entries
        .iter()
        .map(|e| e.size.unwrap_or(0) as usize)
        .sum();

    let tree_response = GitTreeResponse::new("root_commit_sha", full_entries, false);
    let config = FileClassificationConfig {
        max_file_size_bytes: 1024 * 1024,
        ..Default::default()
    };

    // Warmup
    for _ in 0..3 {
        let _ = FileClassifier::classify_tree(&tree_response, &config);
    }

    let iterations = 25;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let report = FileClassifier::classify_tree(&tree_response, &config);
        assert_eq!(report.total_files, total_files);
        assert!(report.eligible_count > 8_000);
        assert!(report.category_counts.contains_key("source_code"));
        assert!(report.category_counts.contains_key("configuration"));
        assert!(report.category_counts.contains_key("documentation"));

        // Verify mathematical conservation of the classification report
        let completeness = report.to_expected_completeness_report();
        assert!(completeness.formula_valid());
        durations.push(start.elapsed());
    }

    let stats = BenchmarkStats::calculate(
        "Polyglot Monorepo File Classification & Category Filtering (12,600 files)",
        total_files * iterations,
        total_bytes * iterations,
        durations,
    );

    // Classifying 12,600 multi-language files must complete in under 180 ms
    stats.assert_p95_under(Duration::from_millis(180));
    stats.assert_items_rate_over(80_000.0);
}

// ==============================================================================
// 3. MULTI-ALGORITHM CONTENT HASHING BENCHMARK (BLAKE3, SHA-256, GIT BLOB)
// ==============================================================================

#[test]
fn test_benchmark_multi_algorithm_content_hashing() {
    // Create 5,000 synthetic files with non-trivial source contents (~5 KB each = ~25 MB total)
    let file_count = 5_000;
    let mut corpus: Vec<Vec<u8>> = Vec::with_capacity(file_count);
    for i in 0..file_count {
        let content = format!(
            "// Module {i}\npub struct Record{i} {{\n    pub id: u64,\n    pub payload: [u8; 64],\n}}\nimpl Record{i} {{\n    pub fn process(&self) -> u64 {{\n        self.id * {i} + 17\n    }}\n}}\n"
        );
        corpus.push(content.into_bytes());
    }

    let corpus_bytes: usize = corpus.iter().map(|c| c.len()).sum();
    let iterations = 15;

    // 1. BLAKE3 Hashing Benchmark
    let mut blake3_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        for file_bytes in &corpus {
            let hash = compute_blake3_hash(file_bytes);
            assert_eq!(hash.len(), 64);
        }
        blake3_durations.push(start.elapsed());
    }

    let blake3_stats = BenchmarkStats::calculate(
        "Cryptographic Digest: BLAKE3 (256-bit SIMD)",
        file_count * iterations,
        corpus_bytes * iterations,
        blake3_durations,
    );
    blake3_stats.assert_throughput_over(80.0);

    // 2. SHA-256 Hashing Benchmark
    let mut sha256_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        for file_bytes in &corpus {
            let hash = compute_sha256_hash(file_bytes);
            assert_eq!(hash.len(), 64);
        }
        sha256_durations.push(start.elapsed());
    }

    let sha256_stats = BenchmarkStats::calculate(
        "Cryptographic Digest: SHA-256 Standard",
        file_count * iterations,
        corpus_bytes * iterations,
        sha256_durations,
    );
    sha256_stats.assert_throughput_over(20.0);

    // 3. Git Blob Object SHA-1 Benchmark
    let mut git_sha_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        for file_bytes in &corpus {
            let hash = compute_git_blob_sha(file_bytes);
            assert_eq!(hash.len(), 40);
        }
        git_sha_durations.push(start.elapsed());
    }

    let git_stats = BenchmarkStats::calculate(
        "Git Object Hash: Git Blob SHA-1 (blob <len>\\0<content>)",
        file_count * iterations,
        corpus_bytes * iterations,
        git_sha_durations,
    );
    git_stats.assert_throughput_over(20.0);
}

// ==============================================================================
// 4. STREAMING ZIP ARCHIVE INGESTION BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_streaming_zip_archive_ingestion() {
    let zip_files_count = 300;
    let archive_bytes = generate_synthetic_zip_archive(zip_files_count);
    let archive_len = archive_bytes.len();
    assert!(
        archive_len > 10_000,
        "Synthetic zip archive must be populated"
    );

    // Warmup
    for _ in 0..3 {
        let _ = scan_zip_archive(&archive_bytes, true);
    }

    let iterations = 30;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let tree_resp =
            scan_zip_archive(&archive_bytes, true).expect("Zip archive scanning must succeed");
        assert_eq!(tree_resp.tree.len(), zip_files_count);
        // Assert root directory stripping
        for entry in &tree_resp.tree {
            assert!(
                !entry.path.starts_with("org-repo-c0ffee1/"),
                "Root directory must be stripped"
            );
        }
        durations.push(start.elapsed());
    }

    let stats = BenchmarkStats::calculate(
        "Streaming ZIP Archive Decompression & Tree Reconstruction (300 files)",
        zip_files_count * iterations,
        archive_len * iterations,
        durations,
    );

    stats.assert_p95_under(Duration::from_millis(25));
    stats.assert_throughput_over(5.0);
}

// ==============================================================================
// 5. MANIFEST ACCOUNTING & MATHEMATICAL CONSERVATION AUDIT BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_manifest_accounting_and_conservation_audit() {
    let file_count = 20_000;
    let mut manifest_entries = Vec::with_capacity(file_count);
    let mut tree_entries = Vec::with_capacity(file_count);

    for i in 0..file_count {
        let pkg = i % 100;
        let path = format!("packages/pkg_{pkg:02}/src/file_{i:05}.rs");
        let git_sha = format!("{i:040x}");
        let size = (200 + (i % 800)) as u64;

        tree_entries.push(GitTreeEntry::new(
            &path,
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            &git_sha,
            Some(size),
        ));

        // Partition 85% extracted, 10% excluded, 3% failed, 2% unsupported
        let status = match i % 100 {
            0..=84 => FileExtractionStatus::Extracted,
            85..=94 => FileExtractionStatus::Excluded {
                reason: "Vendor dependency".to_string(),
            },
            95..=97 => FileExtractionStatus::Failed {
                error: "Corrupted upstream blob".to_string(),
            },
            _ => FileExtractionStatus::Unsupported {
                reason: "Binary asset".to_string(),
            },
        };

        manifest_entries.push(ManifestFileEntry::new(
            &path,
            &git_sha,
            FileCategory::SourceCode,
            size,
            status,
        ));
    }

    let manifest = RepoFileManifest::from_entries(manifest_entries);
    let tree_response = GitTreeResponse::new("test_commit_sha", tree_entries, false);

    // Warmup
    for _ in 0..3 {
        let _ = FileAccountingAuditor::audit_manifest(
            &tree_response,
            None,
            &manifest,
            "org/large-repo",
            Some("main".to_string()),
            Some("c0ffee1".to_string()),
        );
    }

    let iterations = 25;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let balance_sheet = FileAccountingAuditor::audit_manifest(
            &tree_response,
            None,
            &manifest,
            "org/large-repo",
            Some("main".to_string()),
            Some("c0ffee1".to_string()),
        );
        assert!(balance_sheet.is_balanced());
        assert!(balance_sheet.is_fully_accounted());
        let completeness = balance_sheet.to_completeness_report();
        assert!(completeness.formula_valid());
        assert_eq!(completeness.discovered_count, file_count as u64);
        durations.push(start.elapsed());
    }

    let stats = BenchmarkStats::calculate(
        "File Accounting Balance Sheet & Completeness Audit (20,000 files)",
        file_count * iterations,
        0,
        durations,
    );

    // Auditing 20,000 files across balance sheet invariants must finish in under 100 ms
    stats.assert_p95_under(Duration::from_millis(100));
    stats.assert_items_rate_over(250_000.0);
}

// ==============================================================================
// 6. CONCURRENT MULTI-REPOSITORY PIPELINE SCALING BENCHMARK
// ==============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_benchmark_concurrent_multi_repo_pipeline_scaling() {
    let repo_size = 2_500;
    let tree_data = Arc::new(generate_large_synthetic_monorepo_tree(25, 5, 20));
    let classification_config = Arc::new(FileClassificationConfig::default());

    let task_count = 16;
    let thread_tiers = [1, 2, 4];
    let mut tier_throughput = Vec::new();

    for &concurrency in &thread_tiers {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let counter = Arc::new(AtomicUsize::new(0));

        let start = Instant::now();
        let mut handles = Vec::with_capacity(task_count);

        for _ in 0..task_count {
            let sem = Arc::clone(&semaphore);
            let done = Arc::clone(&counter);
            let tree = Arc::clone(&tree_data);
            let cfg = Arc::clone(&classification_config);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // Pipeline simulation: Tree instantiation + Classification + Completeness report
                let response = GitTreeResponse::new("commit_sha", (*tree).clone(), false);
                let report = FileClassifier::classify_tree(&response, &cfg);
                assert_eq!(report.total_files, tree.len());
                let completeness = report.to_expected_completeness_report();
                assert!(completeness.formula_valid());

                done.fetch_add(1, Ordering::Relaxed);
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.unwrap();
        }

        let elapsed = start.elapsed();
        let rate = task_count as f64 / elapsed.as_secs_f64();
        tier_throughput.push((concurrency, elapsed, rate));
    }

    println!("\n================================================================================");
    println!(
        "MULTI-REPO CONCURRENCY SCALING BENCHMARK (16 Concurrent Pipelines of {} files):",
        repo_size
    );
    let baseline_rate = tier_throughput[0].2;
    for (threads, duration, rate) in &tier_throughput {
        let speedup = rate / baseline_rate;
        println!(
            "  Threads: {:<2} | Duration: {:>7.2} ms | Rate: {:>6.1} repos/s | Speedup: {:.2}x",
            threads,
            duration.as_secs_f64() * 1000.0,
            rate,
            speedup
        );
    }
    println!("================================================================================");

    // Verify speedup scaling across worker threads
    let speedup_2 = tier_throughput[1].2 / baseline_rate;
    let speedup_4 = tier_throughput[2].2 / baseline_rate;
    assert!(speedup_2 >= 1.4, "2-thread speedup must be at least 1.4x");
    assert!(speedup_4 >= 2.0, "4-thread speedup must be at least 2.0x");
}
