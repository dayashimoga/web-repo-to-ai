//! Integration test suite for Git commit and blob cache storage schema and repository.

use chrono::{Duration, Utc};
use web_repo_to_ai_storage::{
    split_ddl_statements, GitBlobCacheRow, GitCacheRepository, GitCommitCacheRow,
    InMemoryGitCacheRepository, EXPECTED_INDEXES, EXPECTED_TABLES, POSTGRES_SCHEMA_DDL,
};

#[test]
fn test_schema_and_migration_catalog_contains_git_cache() {
    assert!(
        EXPECTED_TABLES.contains(&"git_commit_cache"),
        "EXPECTED_TABLES must include git_commit_cache"
    );
    assert!(
        EXPECTED_TABLES.contains(&"git_blob_cache"),
        "EXPECTED_TABLES must include git_blob_cache"
    );
    assert!(
        EXPECTED_INDEXES.contains(&"idx_git_commit_cache_repo"),
        "EXPECTED_INDEXES must include idx_git_commit_cache_repo"
    );
    assert!(
        EXPECTED_INDEXES.contains(&"idx_git_commit_cache_last_accessed"),
        "EXPECTED_INDEXES must include idx_git_commit_cache_last_accessed"
    );
    assert!(
        EXPECTED_INDEXES.contains(&"idx_git_blob_cache_blake3"),
        "EXPECTED_INDEXES must include idx_git_blob_cache_blake3"
    );
    assert!(
        EXPECTED_INDEXES.contains(&"idx_git_blob_cache_last_accessed"),
        "EXPECTED_INDEXES must include idx_git_blob_cache_last_accessed"
    );

    let statements = split_ddl_statements(POSTGRES_SCHEMA_DDL);
    assert!(
        !statements.is_empty(),
        "DDL statements must split successfully"
    );
    assert!(
        POSTGRES_SCHEMA_DDL.contains("CREATE TABLE IF NOT EXISTS git_commit_cache"),
        "DDL must declare git_commit_cache table"
    );
    assert!(
        POSTGRES_SCHEMA_DDL.contains("CREATE TABLE IF NOT EXISTS git_blob_cache"),
        "DDL must declare git_blob_cache table"
    );
}

#[test]
fn test_git_commit_and_blob_cache_row_validations() {
    let valid_commit = GitCommitCacheRow::new(
        "octocat",
        "Hello-World",
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        "b4e687810e0474ab49327a1713820d7305949d36",
        42,
        10240,
    )
    .with_tree_entries_json("[]");
    assert!(valid_commit.validate().is_ok());

    let empty_owner = GitCommitCacheRow::new(
        "",
        "Hello-World",
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        "b4e687810e0474ab49327a1713820d7305949d36",
        42,
        10240,
    );
    assert!(empty_owner.validate().is_err());

    let invalid_sha = GitCommitCacheRow::new(
        "octocat",
        "Hello-World",
        "12345", // too short (< 7)
        "b4e687810e0474ab49327a1713820d7305949d36",
        42,
        10240,
    );
    assert!(invalid_sha.validate().is_err());

    let negative_files = GitCommitCacheRow::new(
        "octocat",
        "Hello-World",
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        "b4e687810e0474ab49327a1713820d7305949d36",
        -1,
        10240,
    );
    assert!(negative_files.validate().is_err());

    // Blob validations
    let blake3_hex = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    let valid_blob = GitBlobCacheRow::new(
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        blake3_hex,
        "blobs/7f/d1a60b01f91b314f59955a4e4d4e80d8edf11d",
        512,
    )
    .with_binary(false);
    assert!(valid_blob.validate().is_ok());

    let invalid_blake3 = GitBlobCacheRow::new(
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        "short_digest",
        "blobs/7f/d1a60b01f91b314f59955a4e4d4e80d8edf11d",
        512,
    );
    assert!(invalid_blake3.validate().is_err());

    let empty_storage_key = GitBlobCacheRow::new(
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        blake3_hex,
        "",
        512,
    );
    assert!(empty_storage_key.validate().is_err());
}

#[tokio::test]
async fn test_in_memory_git_cache_repository_commit_lifecycle() {
    let repo = InMemoryGitCacheRepository::new();
    let commit = GitCommitCacheRow::new(
        "Dayashimoga",
        "web-repo-to-ai",
        "c0ffee1234567890abcdef1234567890abcdef12",
        "tree01234567890abcdef1234567890abcdef12",
        15,
        25600,
    );

    // Initial state: not found
    let fetched = repo
        .get_commit(
            "Dayashimoga",
            "web-repo-to-ai",
            "c0ffee1234567890abcdef1234567890abcdef12",
        )
        .await
        .expect("query should succeed");
    assert!(fetched.is_none());

    // Upsert commit
    repo.upsert_commit(&commit)
        .await
        .expect("upsert should succeed");

    // Case-insensitive retrieval
    let fetched = repo
        .get_commit(
            "dayashimoga",
            "WEB-REPO-TO-AI",
            "C0FFEE1234567890ABCDEF1234567890ABCDEF12",
        )
        .await
        .expect("query should succeed")
        .expect("commit should exist");
    assert_eq!(fetched.total_files, 15);
    assert_eq!(fetched.total_bytes, 25600);

    // Touch commit
    let touched = repo
        .touch_commit(
            "dayashimoga",
            "web-repo-to-ai",
            "c0ffee1234567890abcdef1234567890abcdef12",
        )
        .await
        .expect("touch should succeed");
    assert!(touched);

    // Delete commit
    let deleted = repo
        .delete_commit(
            "dayashimoga",
            "web-repo-to-ai",
            "c0ffee1234567890abcdef1234567890abcdef12",
        )
        .await
        .expect("delete should succeed");
    assert!(deleted);

    let re_fetched = repo
        .get_commit(
            "dayashimoga",
            "web-repo-to-ai",
            "c0ffee1234567890abcdef1234567890abcdef12",
        )
        .await
        .expect("query should succeed");
    assert!(re_fetched.is_none());
}

#[tokio::test]
async fn test_in_memory_git_cache_repository_blob_lifecycle_and_batch() {
    let repo = InMemoryGitCacheRepository::new();
    let blake3_hex = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

    let blob1 = GitBlobCacheRow::new(
        "1111111111111111111111111111111111111111",
        blake3_hex,
        "storage/blob1",
        100,
    );
    let blob2 = GitBlobCacheRow::new(
        "2222222222222222222222222222222222222222",
        blake3_hex,
        "storage/blob2",
        200,
    );
    let blob3 = GitBlobCacheRow::new(
        "3333333333333333333333333333333333333333",
        blake3_hex,
        "storage/blob3",
        300,
    );

    // Batch upsert
    repo.upsert_blobs_batch(&[blob1.clone(), blob2.clone()])
        .await
        .expect("batch upsert should succeed");

    // Single upsert
    repo.upsert_blob(&blob3)
        .await
        .expect("single upsert should succeed");

    // Batch query: two existing, one non-existing
    let queries = vec![
        "1111111111111111111111111111111111111111".to_string(),
        "3333333333333333333333333333333333333333".to_string(),
        "4444444444444444444444444444444444444444".to_string(),
    ];
    let batch_result = repo
        .get_blobs_batch(&queries)
        .await
        .expect("batch query should succeed");
    assert_eq!(batch_result.len(), 2);
    assert_eq!(
        batch_result
            .get("1111111111111111111111111111111111111111")
            .unwrap()
            .byte_size,
        100
    );
    assert_eq!(
        batch_result
            .get("3333333333333333333333333333333333333333")
            .unwrap()
            .byte_size,
        300
    );

    // Touch and delete
    let touched = repo
        .touch_blob("1111111111111111111111111111111111111111")
        .await
        .expect("touch should succeed");
    assert!(touched);

    let deleted = repo
        .delete_blob("1111111111111111111111111111111111111111")
        .await
        .expect("delete should succeed");
    assert!(deleted);

    let fetched = repo
        .get_blob("1111111111111111111111111111111111111111")
        .await
        .expect("query should succeed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_in_memory_git_cache_repository_stale_pruning() {
    let repo = InMemoryGitCacheRepository::new();
    let now = Utc::now();
    let old_timestamp = now - Duration::days(10);
    let recent_timestamp = now - Duration::hours(1);

    let mut old_commit = GitCommitCacheRow::new(
        "octocat",
        "repo1",
        "oldcommit1234567890abcdef1234567890abcdef",
        "tree1234567890abcdef1234567890abcdef12",
        5,
        1000,
    );
    old_commit.last_accessed_at = old_timestamp;

    let mut recent_commit = GitCommitCacheRow::new(
        "octocat",
        "repo1",
        "newcommit1234567890abcdef1234567890abcdef",
        "tree1234567890abcdef1234567890abcdef12",
        5,
        1000,
    );
    recent_commit.last_accessed_at = recent_timestamp;

    repo.upsert_commit(&old_commit).await.unwrap();
    repo.upsert_commit(&recent_commit).await.unwrap();

    let blake3_hex = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    let mut old_blob = GitBlobCacheRow::new(
        "oldblob1234567890abcdef1234567890abcdef12",
        blake3_hex,
        "storage/old",
        100,
    );
    old_blob.last_accessed_at = old_timestamp;

    let mut recent_blob = GitBlobCacheRow::new(
        "newblob1234567890abcdef1234567890abcdef12",
        blake3_hex,
        "storage/new",
        100,
    );
    recent_blob.last_accessed_at = recent_timestamp;

    repo.upsert_blob(&old_blob).await.unwrap();
    repo.upsert_blob(&recent_blob).await.unwrap();

    // Prune cutoff 3 days ago
    let cutoff = now - Duration::days(3);
    let pruned_commits = repo.prune_stale_commits(cutoff).await.unwrap();
    let pruned_blobs = repo.prune_stale_blobs(cutoff).await.unwrap();

    assert_eq!(pruned_commits, 1);
    assert_eq!(pruned_blobs, 1);

    // Old entries pruned, recent remain
    assert!(repo
        .get_commit(
            "octocat",
            "repo1",
            "oldcommit1234567890abcdef1234567890abcdef"
        )
        .await
        .unwrap()
        .is_none());
    assert!(repo
        .get_commit(
            "octocat",
            "repo1",
            "newcommit1234567890abcdef1234567890abcdef"
        )
        .await
        .unwrap()
        .is_some());

    assert!(repo
        .get_blob("oldblob1234567890abcdef1234567890abcdef12")
        .await
        .unwrap()
        .is_none());
    assert!(repo
        .get_blob("newblob1234567890abcdef1234567890abcdef12")
        .await
        .unwrap()
        .is_some());
}
