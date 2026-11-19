//! # Content-Addressed Storage (CAS) Integration Test Suite
//!
//! Validates deduplication, multi-hash verification, custom key routing,
//! corruption detection, LRU capacity pruning, and schema integration.

use chrono::{Duration, Utc};
use std::sync::Arc;
use web_repo_to_ai_storage::{
    layout, split_ddl_statements, ArtifactDigests, CasEngine, CasIntegrityOutcome,
    ContentAddressedStorage, ContentCacheIndexRepository, ContentCacheIndexRow, DynStorage,
    InMemoryContentCacheIndexRepository, InMemoryStorage, EXPECTED_INDEXES, EXPECTED_TABLES,
    POSTGRES_SCHEMA_DDL,
};

#[test]
fn test_cas_schema_and_migration_catalog_contains_cas_index() {
    assert!(!POSTGRES_SCHEMA_DDL.is_empty());
    assert!(EXPECTED_INDEXES.contains(&"idx_content_cache_blake3"));
    assert!(EXPECTED_INDEXES.contains(&"idx_content_cache_last_accessed"));
    assert!(EXPECTED_TABLES.contains(&"content_cache_index"));

    assert!(
        POSTGRES_SCHEMA_DDL.contains("idx_content_cache_blake3"),
        "ddl.sql must declare idx_content_cache_blake3"
    );

    let statements = split_ddl_statements(POSTGRES_SCHEMA_DDL);
    assert!(!statements.is_empty());
}

#[tokio::test]
async fn test_cas_put_get_deduplication_and_integrity() {
    let cas = CasEngine::in_memory();

    let data = b"Hello, Content-Addressed Storage World!";
    let digests = ArtifactDigests::compute(data);

    // Initial put: must be is_new = true
    let outcome1 = cas.put(data).await.expect("initial put must succeed");
    assert_eq!(outcome1.blake3_hex, digests.blake3_hex);
    assert_eq!(outcome1.sha256_hex, digests.sha256_hex);
    assert_eq!(outcome1.byte_size, data.len() as u64);
    assert!(outcome1.is_new, "first write should be new");

    // Repeat put: must be deduplication hit (is_new = false)
    let outcome2 = cas.put(data).await.expect("repeat put must succeed");
    assert_eq!(outcome2.blake3_hex, digests.blake3_hex);
    assert!(!outcome2.is_new, "repeat write must be deduplicated");

    // Retrieve by BLAKE3
    let retrieved = cas
        .get(&digests.blake3_hex)
        .await
        .expect("get must succeed")
        .expect("blob must exist");
    assert_eq!(retrieved, data);

    // Non-existent get
    let non_existent = cas
        .get("0000000000000000000000000000000000000000000000000000000000000000")
        .await
        .expect("query should succeed");
    assert!(non_existent.is_none());

    // Check stats
    let stats = cas.stats().await.expect("stats must succeed");
    assert_eq!(stats.write_operations, 2);
    assert_eq!(stats.deduplicated_writes, 1);
    assert_eq!(stats.read_hits, 1);
    assert_eq!(stats.read_misses, 1);
    assert!((stats.deduplication_ratio() - 0.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_cas_put_with_custom_key_and_retrieval() {
    let cas = CasEngine::in_memory();

    let content = b"fn main() { println!(\"Hello Rust\"); }";
    let custom_key = "url:https://crates.io/api/v1/crates/tokio";

    let outcome = cas
        .put_with_key(custom_key, content)
        .await
        .expect("put with key should succeed");
    assert!(outcome.is_new);

    // Query by custom key
    assert!(cas.contains_key(custom_key).await.unwrap());
    let fetched = cas
        .get_by_key(custom_key)
        .await
        .expect("get_by_key should succeed")
        .expect("content should exist");
    assert_eq!(fetched, content);

    // Delete by key
    let deleted = cas
        .delete_by_key(custom_key)
        .await
        .expect("delete should succeed");
    assert!(deleted);
    assert!(!cas.contains_key(custom_key).await.unwrap());
}

#[tokio::test]
async fn test_cas_batch_contains_and_batch_get() {
    let cas = CasEngine::in_memory();

    let blob1 = b"Blob number one payload";
    let blob2 = b"Blob number two payload";
    let blob3 = b"Blob number three payload";

    let out1 = cas.put(blob1).await.unwrap();
    let out2 = cas.put(blob2).await.unwrap();
    let out3 = cas.put(blob3).await.unwrap();

    let dummy_hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    // Batch contains
    let checks = cas
        .batch_contains(&[
            &out1.blake3_hex,
            &out2.blake3_hex,
            &out3.blake3_hex,
            dummy_hex,
        ])
        .await
        .expect("batch_contains should succeed");

    assert_eq!(checks.get(&out1.blake3_hex), Some(&true));
    assert_eq!(checks.get(&out2.blake3_hex), Some(&true));
    assert_eq!(checks.get(&out3.blake3_hex), Some(&true));
    assert_eq!(checks.get(dummy_hex), Some(&false));

    // Batch get
    let batch = cas
        .batch_get(&[&out1.blake3_hex, &out2.blake3_hex, dummy_hex])
        .await
        .expect("batch_get should succeed");

    assert_eq!(batch.len(), 2);
    assert_eq!(batch.get(&out1.blake3_hex).unwrap(), blob1);
    assert_eq!(batch.get(&out2.blake3_hex).unwrap(), blob2);
}

#[tokio::test]
async fn test_cas_corruption_detection_and_integrity_audit() {
    let storage: DynStorage = Arc::new(InMemoryStorage::new());
    let index = Arc::new(InMemoryContentCacheIndexRepository::new());
    let cas = CasEngine::new(storage.clone(), index);

    let good_payload = b"Legitimate, uncorrupted content payload.";
    let outcome = cas.put(good_payload).await.unwrap();

    // Verify integrity passes initially
    let audit_initial = cas.audit().await.unwrap();
    assert!(audit_initial.is_healthy());
    assert_eq!(audit_initial.verified_valid, 1);
    assert_eq!(audit_initial.corrupted, 0);
    assert_eq!(audit_initial.missing, 0);

    // Deliberately corrupt storage payload behind CAS's back
    let storage_key = layout::blob_key(&outcome.blake3_hex);
    let tampered_payload = b"Tampered and maliciously corrupted payload.";
    storage.put(&storage_key, tampered_payload).await.unwrap();

    // Read attempt detects corruption and returns Storage error
    let get_res = cas.get(&outcome.blake3_hex).await;
    assert!(get_res.is_err(), "get must fail when payload is corrupted");

    // verify_integrity returns Corrupted outcome
    let verified = cas.verify_integrity(&outcome.blake3_hex).await.unwrap();
    assert!(!verified.is_valid());
    assert!(matches!(verified, CasIntegrityOutcome::Corrupted { .. }));

    // audit detects corruption
    let audit = cas.audit().await.unwrap();
    assert!(!audit.is_healthy());
    assert_eq!(audit.corrupted, 1);
    assert_eq!(audit.failures.len(), 1);

    // Verify NotFound
    let missing = cas
        .verify_integrity("1111111111111111111111111111111111111111111111111111111111111111")
        .await
        .unwrap();
    assert!(matches!(missing, CasIntegrityOutcome::NotFound { .. }));
}

#[tokio::test]
async fn test_cas_lru_pruning_and_capacity_enforcement() {
    let cas = CasEngine::in_memory();

    let blob1 = vec![1u8; 1000];
    let blob2 = vec![2u8; 2000];
    let blob3 = vec![3u8; 3000];

    let out1 = cas.put(&blob1).await.unwrap();
    let out2 = cas.put(&blob2).await.unwrap();
    let out3 = cas.put(&blob3).await.unwrap();

    let stats_before = cas.stats().await.unwrap();
    assert_eq!(stats_before.total_bytes, 6000);

    // Prune down to 3500 bytes (should evict blob1 [1000] and leave blob2 + blob3 [5000 -> 5000 > 3500, so evict blob2 [2000] leaving blob3 [3000 <= 3500])
    let report = cas.prune_lru(3500).await.expect("prune should succeed");
    assert_eq!(report.pruned_count, 2);
    assert_eq!(report.reclaimed_bytes, 3000);
    assert_eq!(report.initial_bytes, 6000);
    assert_eq!(report.final_bytes, 3000);

    // Blob 1 and Blob 2 are pruned; Blob 3 remains
    assert!(!cas.contains(&out1.blake3_hex).await.unwrap());
    assert!(!cas.contains(&out2.blake3_hex).await.unwrap());
    assert!(cas.contains(&out3.blake3_hex).await.unwrap());
}

#[tokio::test]
async fn test_in_memory_content_cache_index_repository_crud() {
    let repo = InMemoryContentCacheIndexRepository::new();

    let blake3_1 = "1111111111111111111111111111111111111111111111111111111111111111";
    let blake3_2 = "2222222222222222222222222222222222222222222222222222222222222222";

    let row1 = ContentCacheIndexRow::new("key1", blake3_1, "blobs/11/1111.bin", 500);
    let row2 = ContentCacheIndexRow::new("key2", blake3_2, "blobs/22/2222.bin", 1500)
        .with_last_accessed_at(Utc::now() - Duration::hours(5));

    repo.upsert(&row1).await.unwrap();
    repo.upsert(&row2).await.unwrap();

    assert_eq!(repo.count().await.unwrap(), 2);
    assert_eq!(repo.total_bytes().await.unwrap(), 2000);

    // Query by key and by blake3
    let fetched1 = repo.get_by_key("key1").await.unwrap().expect("must exist");
    assert_eq!(fetched1.byte_size, 500);
    assert_eq!(fetched1.formatted_byte_size(), "500 B");

    let fetched_blake = repo
        .get_by_blake3(blake3_2)
        .await
        .unwrap()
        .expect("must exist");
    assert_eq!(fetched_blake.cache_key, "key2");

    // Oldest first
    let oldest = repo.list_oldest_first(1).await.unwrap();
    assert_eq!(oldest.len(), 1);
    assert_eq!(oldest[0].cache_key, "key2");

    // Touch
    assert!(repo.touch("key1").await.unwrap());
    assert!(repo.touch_blake3(blake3_1).await.unwrap());

    // Prune older than 1 hour (should prune key2)
    let pruned = repo
        .prune_older_than(Utc::now() - Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(pruned, 1);
    assert_eq!(repo.count().await.unwrap(), 1);
    assert!(repo.get_by_key("key2").await.unwrap().is_none());

    // Delete by key
    assert!(repo.delete_by_key("key1").await.unwrap());
    assert_eq!(repo.count().await.unwrap(), 0);
}
