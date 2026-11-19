//! # HTTP Cache and Validator Storage Integration Tests
//!
//! Validates `HttpValidatorRow` constraints, `HttpCacheRepository` operations
//! (`InMemoryHttpCacheRepository`), schema DDL, and migration catalog synchronization.

use chrono::Utc;
use web_repo_to_ai_storage::{
    embedded_migrations, ContentCacheIndexRow, HttpCacheRepository, HttpValidatorRow,
    InMemoryHttpCacheRepository, EXPECTED_INDEXES, EXPECTED_TABLES, POSTGRES_SCHEMA_DDL,
};

#[test]
fn test_http_validator_row_validation() {
    let blake3_valid = "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865";

    // 1. Valid row
    let valid_row = HttpValidatorRow::new(
        "https://example.com/api/v1/resource",
        blake3_valid,
        "content/blobs/4355a46b.json",
        1024,
    )
    .with_etag(Some("\"etag-12345\"".to_string()))
    .with_last_modified(
        Some("Wed, 21 Oct 2026 07:28:00 GMT".to_string()),
        Some(Utc::now()),
    )
    .with_content_type(Some("application/json".to_string()))
    .with_cache_control(Some("max-age=3600".to_string()))
    .with_expires_at(Some(Utc::now() + chrono::Duration::hours(1)));
    assert!(valid_row.validate().is_ok());

    // 2. Empty URL fails
    let mut invalid = valid_row.clone();
    invalid.url = "   ".to_string();
    assert!(invalid.validate().is_err());

    // 3. Empty storage key fails
    let mut invalid_sk = valid_row.clone();
    invalid_sk.storage_key = "".to_string();
    assert!(invalid_sk.validate().is_err());

    // 4. Negative byte size fails
    let mut invalid_size = valid_row.clone();
    invalid_size.byte_size = -1;
    assert!(invalid_size.validate().is_err());

    // 5. Malformed BLAKE3 hash fails (not 64 chars)
    let mut invalid_hash = valid_row.clone();
    invalid_hash.blake3_hex = "short_hash".to_string();
    assert!(invalid_hash.validate().is_err());
}

#[tokio::test]
async fn test_in_memory_http_cache_repository_crud_lifecycle() {
    let repo = InMemoryHttpCacheRepository::new();
    let url = "https://example.com/docs/guide";
    let blake3 = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

    // 1. Initial lookup returns None
    let missing = repo.get_validator(url).await.unwrap();
    assert!(missing.is_none());

    // 2. Upsert initial validator
    let row = HttpValidatorRow::new(url, blake3, "content/blobs/abcdef.html", 2048)
        .with_etag(Some("\"v1.0\"".to_string()))
        .with_last_modified(Some("Thu, 19 Nov 2026 08:00:00 GMT".to_string()), None)
        .with_content_type(Some("text/html".to_string()))
        .with_cache_control(Some("max-age=600".to_string()))
        .with_expires_at(Some(Utc::now() + chrono::Duration::minutes(10)))
        .with_canonical_url(Some("https://example.com/docs/guide/canonical".to_string()));

    repo.upsert_validator(&row).await.unwrap();

    // 3. Retrieve and verify fields
    let stored = repo.get_validator(url).await.unwrap().expect("Must exist");
    assert_eq!(stored.url, url);
    assert_eq!(
        stored.canonical_url.as_deref(),
        Some("https://example.com/docs/guide/canonical")
    );
    assert_eq!(stored.etag.as_deref(), Some("\"v1.0\""));
    assert_eq!(stored.byte_size, 2048);

    // 4. Touch validator on 304 Not Modified
    let touched = repo
        .touch_validator(
            url,
            Some("\"v1.0-refreshed\""),
            Some("Thu, 19 Nov 2026 09:00:00 GMT"),
            Some(Utc::now() + chrono::Duration::hours(1)),
        )
        .await
        .unwrap();
    assert!(touched);

    let updated = repo.get_validator(url).await.unwrap().unwrap();
    assert_eq!(updated.etag.as_deref(), Some("\"v1.0-refreshed\""));
    assert_eq!(
        updated.last_modified.as_deref(),
        Some("Thu, 19 Nov 2026 09:00:00 GMT")
    );

    // 5. Touch non-existent URL returns false
    let touch_missing = repo
        .touch_validator("https://example.com/missing", None, None, None)
        .await
        .unwrap();
    assert!(!touch_missing);

    // 6. Delete validator
    let deleted = repo.delete_validator(url).await.unwrap();
    assert!(deleted);
    assert!(repo.get_validator(url).await.unwrap().is_none());
}

#[tokio::test]
async fn test_in_memory_http_cache_repository_stale_pruning() {
    let repo = InMemoryHttpCacheRepository::new();
    let blake3 = "1111111111111111111111111111111111111111111111111111111111111111";

    let old_time = Utc::now() - chrono::Duration::days(10);
    let mut old_row =
        HttpValidatorRow::new("https://example.com/old-page", blake3, "content/old", 100)
            .with_etag(Some("\"old\"".to_string()));
    old_row.last_checked_at = old_time;

    let fresh_row = HttpValidatorRow::new(
        "https://example.com/fresh-page",
        blake3,
        "content/fresh",
        200,
    )
    .with_etag(Some("\"fresh\"".to_string()));

    repo.upsert_validator(&old_row).await.unwrap();
    repo.upsert_validator(&fresh_row).await.unwrap();

    // Prune entries older than 5 days ago
    let cutoff = Utc::now() - chrono::Duration::days(5);
    let pruned = repo.prune_stale_validators(cutoff).await.unwrap();
    assert_eq!(pruned, 1);

    assert!(repo
        .get_validator("https://example.com/old-page")
        .await
        .unwrap()
        .is_none());
    assert!(repo
        .get_validator("https://example.com/fresh-page")
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn test_in_memory_content_cache_index_crud() {
    let repo = InMemoryHttpCacheRepository::new();
    let cache_key = "https://example.com/docs#etag:abc123";
    let blake3 = "2222222222222222222222222222222222222222222222222222222222222222";

    let entry = ContentCacheIndexRow::new(cache_key, blake3, "content/blobs/222222.html", 4096);
    repo.upsert_content_cache(&entry).await.unwrap();

    let fetched = repo
        .get_content_cache(cache_key)
        .await
        .unwrap()
        .expect("Cache entry must exist");
    assert_eq!(fetched.cache_key, cache_key);
    assert_eq!(fetched.blake3_hex, blake3);
    assert_eq!(fetched.byte_size, 4096);
    assert_eq!(fetched.storage_key, "content/blobs/222222.html");
}

#[test]
fn test_schema_and_migration_catalog_synchronization() {
    // 1. Table catalog contains http_validators
    assert!(
        EXPECTED_TABLES.contains(&"http_validators"),
        "EXPECTED_TABLES must contain http_validators"
    );

    // 2. Indexes catalog contains http_validators indexes
    assert!(EXPECTED_INDEXES.contains(&"idx_http_validators_blake3"));
    assert!(EXPECTED_INDEXES.contains(&"idx_http_validators_last_checked"));
    assert!(EXPECTED_INDEXES.contains(&"idx_http_validators_expires"));

    // 3. Embedded DDL contains table creation
    assert!(
        POSTGRES_SCHEMA_DDL.contains("CREATE TABLE IF NOT EXISTS http_validators"),
        "POSTGRES_SCHEMA_DDL must define http_validators"
    );

    // 4. Embedded migrations contains new migration
    let migrations = embedded_migrations();
    let has_http_validators_migration = migrations.iter().any(|m| m.version == 20261119000002);
    assert!(
        has_http_validators_migration,
        "Embedded migrations must include version 20261119000002"
    );
}
