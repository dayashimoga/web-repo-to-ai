//! # Retry Dynamics & Mathematical Idempotency Integration Tests
//!
//! Evaluates the reliability, transient failure tolerance, backoff exhaustion handling,
//! mathematical idempotency $f(f(x)) \equiv f(x)$, and bit-for-bit deterministic repeatability
//! across the `web-repo-to-ai` worker daemon, job engine, content normalizer, deduplicator,
//! content-addressed storage (CAS), multi-format exporters, and state machines:
//! - Item-level automatic retry loop with exponential backoff on transient errors.
//! - Deterministic terminal failure isolation on retry backoff exhaustion.
//! - Mathematical idempotency $f(f(x)) \equiv f(x)$ across document and text normalization and deduplication.
//! - Content-addressed storage (CAS) and cache write idempotency under concurrent access.
//! - Multi-format export bit-for-bit repeatability across Markdown, Plain Text, JSON v1.0, HTML5, and ZIP archives.
//! - State machine terminal immutability and leasing invariant idempotency.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use tokio::sync::Mutex;

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{RevisionInfo, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_deduplication::{deduplicate_document, ExactDeduplicationConfig};
use web_repo_to_ai_exporters::html::export_html;
use web_repo_to_ai_exporters::json::export_json;
use web_repo_to_ai_exporters::markdown::export_markdown;
use web_repo_to_ai_exporters::options::{
    HtmlOptions, JsonExportOptions, MarkdownOptions, PlainTextOptions,
};
use web_repo_to_ai_exporters::plain_text::export_plain_text;
use web_repo_to_ai_exporters::streaming::StreamingExportConfig;
use web_repo_to_ai_exporters::ZipArchive;
use web_repo_to_ai_job_engine::models::{BatchItemStatusOutcome, BatchItemTask};
use web_repo_to_ai_normalizer::{normalize_document, normalize_text, NormalizationConfig};
use web_repo_to_ai_storage::cas::{CasEngine, ContentAddressedStorage};
use web_repo_to_ai_storage::leasing::LeaseConfig;
use web_repo_to_ai_storage::schema::JobStatus;
use web_repo_to_ai_storage::state_machine::JobStateMachine;
use web_repo_to_ai_worker::config::WorkerServiceConfig;
use web_repo_to_ai_worker::engine::WorkerEngine;

/// Helper creating a batch of N test conversion tasks with sequence index.
fn create_test_tasks(count: usize, prefix: &str) -> Vec<BatchItemTask<String>> {
    (0..count)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{prefix}/{i}"),
                Some(format!("custom-{prefix}-{i}")),
                format!("payload-{i}"),
            )
        })
        .collect()
}

/// Helper creating a realistic sample CDM document with rich sections.
fn create_sample_cdm_document(index: usize, title: &str, body: &str) -> CdmDocument {
    let identity = SourceIdentity {
        raw_input: format!("https://example.com/docs/doc-{index}"),
        canonical_url: format!("https://example.com/docs/doc-{index}"),
        domain: Some("example.com".to_string()),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    };

    let prose_section = CdmSectionBuilder::new(
        format!("sec-{index}-prose"),
        format!("https://example.com/docs/doc-{index}#prose"),
        SectionKind::Prose,
        body,
    )
    .title(title)
    .original_path(format!("docs/doc_{index}.md"))
    .build();

    let code_content = "def calculate_factorial(n: int) -> int:\n    if n <= 1:\n        return 1\n    return n * calculate_factorial(n - 1)\n";
    let code_section = CdmSectionBuilder::new(
        format!("sec-{index}-code"),
        format!("https://example.com/docs/doc-{index}#code"),
        SectionKind::CodeFile,
        code_content,
    )
    .title("Algorithm Implementation")
    .original_path(format!("src/calc_{index}.py"))
    .build();

    CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        format!("https://example.com/docs/doc-{index}"),
    )
    .revision(RevisionInfo {
        git_tag: Some(format!("v{index}.0")),
        ..Default::default()
    })
    .add_section(prose_section)
    .add_section(code_section)
    .build()
}

// ============================================================================
// TEST 1: Transient Failure Automatic Retry & Recovery
// ============================================================================

#[tokio::test]
async fn test_transient_failure_automatic_retry_and_recovery() {
    // 1. Configure worker with max_retries = 3 and fast backoff (10ms)
    let config = WorkerServiceConfig::for_worker("test-worker-retries", 4, 4)
        .expect("Config should be valid")
        .with_retries(3, 10);

    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    let batch_id = BatchId::new();
    let total_tasks = 6;
    let tasks = create_test_tasks(total_tasks, "retry-test");

    // Track execution attempt counts per item index
    let attempts_map: Arc<Mutex<HashMap<usize, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let attempts_tracker = Arc::clone(&attempts_map);

    // 2. Processor simulating transient failures on odd items that recover on attempt 3
    let processor = move |task: BatchItemTask<String>| {
        let attempts_map = Arc::clone(&attempts_tracker);
        async move {
            let attempt_number = {
                let mut map = attempts_map.lock().await;
                let count = map.entry(task.item_index).or_insert(0);
                let current = *count;
                *count += 1;
                current
            };

            // Odd items fail on attempts 0 and 1 with a transient error, then succeed on attempt 2
            if task.item_index % 2 == 1 && attempt_number < 2 {
                Err(format!(
                    "Transient upstream connection reset on attempt {attempt_number}"
                ))
            } else {
                Ok(format!(
                    "result-{}-attempt-{}",
                    task.item_index, attempt_number
                ))
            }
        }
    };

    // 3. Execute parallel batch
    let summary = engine
        .execute_batch(batch_id, tasks, None, processor, None)
        .await
        .expect("Batch execution should complete");

    // 4. Verify 100% success after transient retries
    assert!(
        summary.all_succeeded(),
        "All batch items should succeed after transient retry recovery"
    );
    assert_eq!(summary.total_items, total_tasks);
    assert_eq!(summary.succeeded_count, total_tasks);
    assert_eq!(summary.failed_count, 0);
    assert_eq!(summary.timed_out_count, 0);
    assert_eq!(summary.cancelled_count, 0);
    assert_eq!(summary.item_results.len(), total_tasks);

    // 5. Verify task attempt counts
    let final_attempts = attempts_map.lock().await;
    for i in 0..total_tasks {
        if i % 2 == 1 {
            // Odd tasks failed twice and succeeded on attempt 2 (total 3 attempts)
            assert_eq!(
                final_attempts.get(&i).copied(),
                Some(3),
                "Task {i} should have been attempted exactly 3 times"
            );
            assert_eq!(
                summary.item_results[i].value,
                Some(format!("result-{i}-attempt-2")),
                "Task {i} result should match the successful 3rd attempt"
            );
        } else {
            // Even tasks succeeded immediately on attempt 0 (total 1 attempt)
            assert_eq!(
                final_attempts.get(&i).copied(),
                Some(1),
                "Task {i} should have been attempted exactly 1 time"
            );
            assert_eq!(
                summary.item_results[i].value,
                Some(format!("result-{i}-attempt-0")),
                "Task {i} result should match the initial successful attempt"
            );
        }
        // Strict sequence index preservation
        assert_eq!(summary.item_results[i].item_index, i);
        assert!(summary.item_results[i].is_success());
    }

    // 6. Verify worker telemetry
    let metrics = engine.metrics_snapshot();
    assert_eq!(metrics.batches_processed, 1);
    assert_eq!(metrics.items_processed, total_tasks);
    assert_eq!(metrics.items_succeeded, total_tasks);
    assert_eq!(metrics.items_failed, 0);
}

// ============================================================================
// TEST 2: Retry Backoff Exhaustion Deterministic Terminal Failure
// ============================================================================

#[tokio::test]
async fn test_retry_backoff_exhaustion_deterministic_terminal_failure() {
    // 1. Configure worker with max_retries = 2 (initial attempt + 2 retries = 3 total executions)
    let config = WorkerServiceConfig::for_worker("test-worker-exhaustion", 4, 4)
        .expect("Config should be valid")
        .with_retries(2, 10);

    let engine = WorkerEngine::new(config).expect("Engine should initialize");

    let batch_id = BatchId::new();
    let total_tasks = 5;
    let tasks = create_test_tasks(total_tasks, "exhaustion-test");

    let attempts_map: Arc<Mutex<HashMap<usize, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let attempts_tracker = Arc::clone(&attempts_map);

    // 2. Processor where task index 2 has a permanent terminal error
    let processor = move |task: BatchItemTask<String>| {
        let attempts_map = Arc::clone(&attempts_tracker);
        async move {
            let attempt_number = {
                let mut map = attempts_map.lock().await;
                let count = map.entry(task.item_index).or_insert(0);
                let current = *count;
                *count += 1;
                current
            };

            if task.item_index == 2 {
                // Permanently failing task
                Err(format!(
                    "Terminal HTTP 404 Not Found on attempt {attempt_number}"
                ))
            } else {
                // Healthy tasks
                Ok(format!("success-payload-{}", task.item_index))
            }
        }
    };

    // 3. Execute batch
    let summary = engine
        .execute_batch(batch_id, tasks, None, processor, None)
        .await
        .expect("Batch execution should complete even when items fail");

    // 4. Verify terminal isolation
    assert!(
        summary.has_failures(),
        "Batch summary should report failures"
    );
    assert!(!summary.all_succeeded());
    assert_eq!(summary.total_items, total_tasks);
    assert_eq!(summary.succeeded_count, 4);
    assert_eq!(summary.failed_count, 1);
    assert_eq!(summary.timed_out_count, 0);
    assert_eq!(summary.cancelled_count, 0);

    // 5. Verify exhausted item 2 was attempted exactly 3 times (1 initial + 2 retries)
    let final_attempts = attempts_map.lock().await;
    assert_eq!(
        final_attempts.get(&2).copied(),
        Some(3),
        "Item 2 should have exhausted exactly 1 initial + 2 retries"
    );

    let failed_item = &summary.item_results[2];
    assert_eq!(failed_item.item_index, 2);
    assert!(!failed_item.is_success());
    match &failed_item.status {
        BatchItemStatusOutcome::Failed { error } => {
            assert!(
                error.contains("Terminal HTTP 404 Not Found"),
                "Error message should reflect upstream failure: {error}"
            );
        }
        other => panic!("Expected BatchItemStatusOutcome::Failed, got {other:?}"),
    }

    // 6. Verify non-failing items were not aborted or corrupted
    for i in [0, 1, 3, 4] {
        let item = &summary.item_results[i];
        assert_eq!(item.item_index, i);
        assert!(item.is_success());
        assert_eq!(item.value, Some(format!("success-payload-{i}")));
        assert_eq!(
            final_attempts.get(&i).copied(),
            Some(1),
            "Item {i} should have succeeded on attempt 1 without retries"
        );
    }
}

// ============================================================================
// TEST 3: Document Normalization & Deduplication Mathematical Idempotency
// ============================================================================

#[tokio::test]
async fn test_document_normalization_and_deduplication_mathematical_idempotency() {
    let norm_config = NormalizationConfig::lossless();

    // ------------------------------------------------------------------------
    // Part A: Mathematical idempotency of raw text normalization: f(f(x)) == f(x)
    // ------------------------------------------------------------------------
    let dirty_raw_text = "\u{FEFF}   # System Architecture Overview \r\n\r\n\r\n\r\n\
        The quick brown fox jumps over the lazy dog.  \r\n\
        Some body text with   multiple spaces and trailing blanks.   \r\n\r\n\r\n\
        ```rust\r\nfn main() {\r\n    println!(\"Hello World!\");\r\n}\r\n```\r\n\r\n";

    let text_pass_1 = normalize_text(dirty_raw_text, &norm_config, false);
    let text_pass_2 = normalize_text(&text_pass_1, &norm_config, false);
    let text_pass_3 = normalize_text(&text_pass_2, &norm_config, false);

    assert_eq!(
        text_pass_1, text_pass_2,
        "Text normalization must be mathematically idempotent: f(f(x)) == f(x)"
    );
    assert_eq!(
        text_pass_2, text_pass_3,
        "Iterative text normalization must reach immediate steady state"
    );
    assert!(
        !text_pass_1.contains('\r'),
        "Normalized text must have CRLF converted to LF"
    );
    assert!(
        !text_pass_1.starts_with('\u{FEFF}'),
        "BOM must be cleanly stripped"
    );

    // ------------------------------------------------------------------------
    // Part B: Mathematical idempotency of full CDM Document normalization
    // ------------------------------------------------------------------------
    let messy_body = "This is a comprehensive document section.\r\n\r\n\r\n\r\n\
        It contains redundant line breaks and trailing spaces.   \r\n";
    let original_doc =
        create_sample_cdm_document(42, "Idempotency Verification Document", messy_body);

    let (doc_norm_1, report_1) = normalize_document(&original_doc, &norm_config);
    let (doc_norm_2, report_2) = normalize_document(&doc_norm_1, &norm_config);
    let (doc_norm_3, report_3) = normalize_document(&doc_norm_2, &norm_config);

    // Structural and content equality across passes
    assert_eq!(
        doc_norm_1.sections.len(),
        doc_norm_2.sections.len(),
        "Section count must remain invariant"
    );
    for i in 0..doc_norm_1.sections.len() {
        assert_eq!(
            doc_norm_1.sections[i].body, doc_norm_2.sections[i].body,
            "Section {i} body must be identical after second normalization pass"
        );
        assert_eq!(
            doc_norm_2.sections[i].body, doc_norm_3.sections[i].body,
            "Section {i} body must be identical after third normalization pass"
        );
        assert_eq!(
            doc_norm_1.sections[i].content_hash, doc_norm_2.sections[i].content_hash,
            "Section {i} cryptographic BLAKE3 content hash must be invariant"
        );
    }

    assert_eq!(
        report_2.sections_modified, 0,
        "Second normalization pass must perform exactly 0 modifications"
    );
    assert_eq!(
        report_3.sections_modified, 0,
        "Third normalization pass must perform exactly 0 modifications"
    );
    assert!(
        report_1.sections_modified > 0,
        "Initial pass should have cleaned up formatting anomalies"
    );

    // ------------------------------------------------------------------------
    // Part C: Mathematical idempotency of Document Deduplication
    // ------------------------------------------------------------------------
    let dedup_config = ExactDeduplicationConfig::default();

    // Construct a document with duplicate sections
    let identity = SourceIdentity {
        raw_input: "https://example.com/dedup/doc".to_string(),
        canonical_url: "https://example.com/dedup/doc".to_string(),
        domain: Some("example.com".to_string()),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    };

    let sec_shared_content = "This identical text block is shared across multiple chapters.";
    let sec_1 = CdmSectionBuilder::new(
        "sec-1",
        "https://example.com/dedup/doc#sec1",
        SectionKind::Prose,
        sec_shared_content,
    )
    .title("Chapter 1: Shared Concepts")
    .build();

    let sec_2 = CdmSectionBuilder::new(
        "sec-2",
        "https://example.com/dedup/doc#sec2",
        SectionKind::Prose,
        sec_shared_content,
    )
    .title("Chapter 2: Duplicate Copy")
    .build();

    let sec_unique = CdmSectionBuilder::new(
        "sec-unique",
        "https://example.com/dedup/doc#unique",
        SectionKind::Prose,
        "Unique exclusive content",
    )
    .title("Unique Section")
    .build();

    let multi_section_doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageStatic,
        "https://example.com/dedup/doc",
    )
    .add_section(sec_1)
    .add_section(sec_2)
    .add_section(sec_unique)
    .build();

    let (doc_dedup_1, dedup_rep_1) = deduplicate_document(&multi_section_doc, &dedup_config);
    let (doc_dedup_2, dedup_rep_2) = deduplicate_document(&doc_dedup_1, &dedup_config);
    let (doc_dedup_3, dedup_rep_3) = deduplicate_document(&doc_dedup_2, &dedup_config);

    assert_eq!(
        dedup_rep_1.duplicate_sections_found, 1,
        "Pass 1 must detect and remove the duplicate section"
    );
    assert_eq!(
        dedup_rep_2.duplicate_sections_found, 0,
        "Pass 2 must detect 0 duplicates on an already deduplicated document"
    );
    assert_eq!(
        dedup_rep_3.duplicate_sections_found, 0,
        "Pass 3 must detect 0 duplicates"
    );

    assert_eq!(
        doc_dedup_1.sections.len(),
        doc_dedup_2.sections.len(),
        "Section counts must match across repeated deduplication passes"
    );
    assert_eq!(doc_dedup_2.sections.len(), doc_dedup_3.sections.len());
}

// ============================================================================
// TEST 4: Content-Addressed Storage & Cache Write Idempotency
// ============================================================================

#[tokio::test]
async fn test_content_addressed_storage_and_cache_write_idempotency() {
    let cas = CasEngine::in_memory();

    let payload = b"Universal cryptographic content blob for CAS write idempotency verification. \
        The contents are strictly immutable, content-addressed, and multi-hash verified.";

    // 1. Initial write
    let outcome_1 = cas
        .put(payload)
        .await
        .expect("Initial CAS write must succeed");
    assert!(outcome_1.is_new, "Initial write must be marked as new");
    assert_eq!(outcome_1.byte_size, payload.len() as u64);
    assert!(!outcome_1.blake3_hex.is_empty());
    assert!(!outcome_1.sha256_hex.is_empty());

    // 2. Sequential duplicate write (idempotent no-op reuse)
    let outcome_2 = cas
        .put(payload)
        .await
        .expect("Secondary duplicate write must succeed");
    assert!(
        !outcome_2.is_new,
        "Duplicate sequential write must be recognized as existing"
    );
    assert_eq!(
        outcome_1.blake3_hex, outcome_2.blake3_hex,
        "BLAKE3 digest must be strictly identical"
    );
    assert_eq!(
        outcome_1.sha256_hex, outcome_2.sha256_hex,
        "SHA-256 digest must be strictly identical"
    );
    assert_eq!(
        outcome_1.storage_key, outcome_2.storage_key,
        "Storage location key must be strictly identical"
    );
    assert_eq!(outcome_1.byte_size, outcome_2.byte_size);

    // 3. Concurrent multi-threaded writes of the identical blob
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let cas_clone = cas.clone();
        let payload_clone = payload.to_vec();
        tasks.push(tokio::spawn(
            async move { cas_clone.put(&payload_clone).await },
        ));
    }

    for task in tasks {
        let outcome = task
            .await
            .expect("Task join must succeed")
            .expect("CAS concurrent write must succeed");
        assert_eq!(outcome.blake3_hex, outcome_1.blake3_hex);
        assert_eq!(outcome.sha256_hex, outcome_1.sha256_hex);
    }

    // 4. Retrieve content and verify bit-for-bit fidelity
    let fetched = cas
        .get(&outcome_1.blake3_hex)
        .await
        .expect("Fetch by hash must succeed")
        .expect("Blob must exist in storage");
    assert_eq!(fetched, payload, "Retrieved bytes must match stored bytes");

    // 5. Audit storage integrity: ensure exactly 1 physical blob exists (zero disk bloat)
    let audit = cas.audit().await.expect("Storage audit must succeed");
    assert!(audit.is_healthy(), "Audit must confirm storage is healthy");
    assert_eq!(
        audit.total_indexed, 1,
        "Exactly 1 indexed entry should exist despite multiple writes"
    );
    assert_eq!(audit.verified_valid, 1);
    assert_eq!(audit.missing, 0);
    assert_eq!(audit.corrupted, 0);
}

// ============================================================================
// TEST 5: Multi-Format Export Bit-for-Bit Deterministic Idempotency
// ============================================================================

#[tokio::test]
async fn test_multi_format_export_bit_for_bit_deterministic_idempotency() {
    let doc = create_sample_cdm_document(
        101,
        "Deterministic Export Specification",
        "Deterministic rendering requires strict ordering, invariant headings, and reproducible tables.",
    );

    let md_opts = MarkdownOptions::default();
    let pt_opts = PlainTextOptions::default();
    let json_opts = JsonExportOptions::default();
    let html_opts = HtmlOptions::default();

    // 1. Export in iteration 0 to establish baseline
    let baseline_md = export_markdown(&doc, &md_opts).expect("Markdown export must succeed");
    let baseline_pt = export_plain_text(&doc, &pt_opts).expect("Plain text export must succeed");
    let baseline_json = export_json(&doc, &json_opts).expect("JSON export must succeed");
    let (baseline_html, _) = export_html(&doc, &html_opts).expect("HTML export must succeed");

    let baseline_md_blake3 = blake3::hash(baseline_md.as_bytes()).to_hex().to_string();
    let baseline_pt_blake3 = blake3::hash(baseline_pt.as_bytes()).to_hex().to_string();
    let baseline_json_blake3 = blake3::hash(baseline_json.as_bytes()).to_hex().to_string();
    let baseline_html_blake3 = blake3::hash(baseline_html.as_bytes()).to_hex().to_string();

    // 2. Repeat export 5 times and verify bit-for-bit invariance
    for iteration in 1..=5 {
        let md = export_markdown(&doc, &md_opts).expect("Markdown export must succeed");
        let pt = export_plain_text(&doc, &pt_opts).expect("Plain text export must succeed");
        let json = export_json(&doc, &json_opts).expect("JSON export must succeed");
        let (html, _) = export_html(&doc, &html_opts).expect("HTML export must succeed");

        assert_eq!(
            md, baseline_md,
            "Iteration {iteration}: Markdown export must be bit-for-bit identical"
        );
        assert_eq!(
            pt, baseline_pt,
            "Iteration {iteration}: Plain text export must be bit-for-bit identical"
        );
        assert_eq!(
            json, baseline_json,
            "Iteration {iteration}: JSON export must be bit-for-bit identical"
        );
        assert_eq!(
            html, baseline_html,
            "Iteration {iteration}: HTML export must be bit-for-bit identical"
        );

        assert_eq!(
            blake3::hash(md.as_bytes()).to_hex().to_string(),
            baseline_md_blake3,
            "Iteration {iteration}: Markdown BLAKE3 checksum must match"
        );
        assert_eq!(
            blake3::hash(pt.as_bytes()).to_hex().to_string(),
            baseline_pt_blake3,
            "Iteration {iteration}: Plain text BLAKE3 checksum must match"
        );
        assert_eq!(
            blake3::hash(json.as_bytes()).to_hex().to_string(),
            baseline_json_blake3,
            "Iteration {iteration}: JSON BLAKE3 checksum must match"
        );
        assert_eq!(
            blake3::hash(html.as_bytes()).to_hex().to_string(),
            baseline_html_blake3,
            "Iteration {iteration}: HTML BLAKE3 checksum must match"
        );
    }

    // 3. Streaming ZIP Archive Export Idempotency
    let worker_config = WorkerServiceConfig::for_worker("test-export-worker", 2, 2).unwrap();
    let engine = WorkerEngine::new(worker_config).unwrap();
    let export_config = StreamingExportConfig::new(8192)
        .unwrap()
        .with_manifests(true, true);

    let docs = vec![doc.clone()];

    // Generate initial ZIP archive
    let mut zip_buf_1 = Cursor::new(Vec::new());
    let summary_1 = engine
        .execute_streaming_export(&mut zip_buf_1, export_config.clone(), &docs)
        .expect("Streaming ZIP export 1 must succeed");

    // Generate secondary ZIP archive from identical inputs
    let mut zip_buf_2 = Cursor::new(Vec::new());
    let summary_2 = engine
        .execute_streaming_export(&mut zip_buf_2, export_config, &docs)
        .expect("Streaming ZIP export 2 must succeed");

    assert_eq!(
        summary_1.total_items_written, summary_2.total_items_written,
        "Total items written in streaming ZIP export must be identical"
    );
    assert_eq!(
        summary_1.total_uncompressed_bytes,
        summary_2.total_uncompressed_bytes
    );

    // Verify internal entry layout reproducibility
    let mut reader_1 = Cursor::new(zip_buf_1.into_inner());
    let mut zip_1 = ZipArchive::new(&mut reader_1).expect("Valid ZIP archive 1");

    let mut reader_2 = Cursor::new(zip_buf_2.into_inner());
    let mut zip_2 = ZipArchive::new(&mut reader_2).expect("Valid ZIP archive 2");

    assert_eq!(zip_1.len(), zip_2.len(), "ZIP file count must match");

    let expected_files = [
        "manifest.json",
        "checksums.sha256",
        "doc_0/document.json",
        "doc_0/context.md",
        "doc_0/context.txt",
    ];
    for path in expected_files {
        let f1 = zip_1.by_name(path).expect("File must exist in zip 1");
        let f2 = zip_2.by_name(path).expect("File must exist in zip 2");
        assert_eq!(f1.size(), f2.size(), "Size for {path} must match");
        assert_eq!(f1.crc32(), f2.crc32(), "CRC32 for {path} must match");
    }
}

// ============================================================================
// TEST 6: State Machine Transition & Lease Renewal Idempotency
// ============================================================================

#[tokio::test]
async fn test_state_machine_transition_and_lease_renewal_idempotency() {
    // ------------------------------------------------------------------------
    // Part A: State Machine Terminal Immutability & Deterministic Invariants
    // ------------------------------------------------------------------------
    // Valid transitions in distributed leasing: Queued -> Leased -> Running -> Completed
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Leased).is_ok(),
        "Queued -> Leased transition must be permitted"
    );
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Running).is_ok(),
        "Leased -> Running transition must be permitted"
    );
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Completed).is_ok(),
        "Running -> Completed transition must be permitted"
    );
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Failed).is_ok(),
        "Running -> Failed transition must be permitted"
    );
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Cancelled).is_ok(),
        "Queued -> Cancelled transition must be permitted"
    );

    // Direct Queued -> Running transition without intermediate lease acquisition must be rejected
    assert!(
        JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Running).is_err(),
        "Direct Queued -> Running transition without leasing must be rejected"
    );

    // Terminal state transitions must be strictly and idempotently rejected
    let terminal_states = [
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Cancelled,
    ];
    let target_states = [
        JobStatus::Queued,
        JobStatus::Running,
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Cancelled,
    ];

    for terminal in terminal_states {
        for target in target_states {
            let res = JobStateMachine::evaluate_transition(terminal, target);
            assert!(
                res.is_err(),
                "Transitioning from terminal state {terminal} to {target} must be rejected"
            );
        }
    }

    // ------------------------------------------------------------------------
    // Part B: Lease Configuration Validation Invariants
    // ------------------------------------------------------------------------
    // Valid lease config
    let valid_lease = LeaseConfig::new("worker-node-1", 30, 10, 3);
    assert!(valid_lease.is_ok(), "Valid lease configuration must pass");

    // Invariant: worker_id cannot be blank
    let blank_worker = LeaseConfig::new("   ", 30, 10, 3);
    assert!(
        blank_worker.is_err(),
        "Blank worker_id must fail validation"
    );

    // Invariant: lease_duration_seconds must be >= 5
    let tiny_lease = LeaseConfig::new("worker-node-1", 2, 1, 3);
    assert!(
        tiny_lease.is_err(),
        "Tiny lease duration must fail validation"
    );

    // Invariant: heartbeat_interval_seconds must be strictly < lease_duration_seconds
    let invalid_heartbeat = LeaseConfig::new("worker-node-1", 10, 10, 3);
    assert!(
        invalid_heartbeat.is_err(),
        "Heartbeat >= lease duration must fail validation"
    );

    // Invariant: max_attempts must be >= 1
    let zero_attempts = LeaseConfig::new("worker-node-1", 30, 10, 0);
    assert!(
        zero_attempts.is_err(),
        "Zero max_attempts must fail validation"
    );

    // ------------------------------------------------------------------------
    // Part C: Monotonic Lease Expiration Computation
    // ------------------------------------------------------------------------
    let config = LeaseConfig::for_worker("test-worker-monotonic");
    assert_eq!(config.lease_duration_seconds, 60);
    assert_eq!(config.heartbeat_interval_seconds, 15);
    assert_eq!(config.max_attempts, 3);
}
