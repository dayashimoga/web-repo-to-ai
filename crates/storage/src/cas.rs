//! # Content-Addressed Storage (CAS) Subsystem
//!
//! Provides cryptographic content-addressed blob storage with deduplication,
//! multi-hash validation (BLAKE3 and SHA-256), sharded directory partitioning,
//! LRU storage quota enforcement, and zero-mock audit verification:
//! - [`ContentAddressedStorage`] unified trait for content-addressed operations.
//! - [`CasEngine`] production implementation wrapping [`DynStorage`] and [`ContentCacheIndexRepository`].
//! - [`ContentCacheIndexRepository`] repository abstraction for the `content_cache_index` catalog.
//! - [`InMemoryContentCacheIndexRepository`] in-memory implementation for tests and standalone CLI.
//! - [`PostgresContentCacheIndexRepository`] SQLx-backed PostgreSQL persistent store.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use web_repo_to_ai_core::error::CoreError;

use crate::backend::{DynStorage, InMemoryStorage};
use crate::checksum::ArtifactDigests;
use crate::local_fs::layout;
use crate::schema::ContentCacheIndexRow;

/// Outcome of storing an artifact or blob in Content-Addressed Storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasWriteOutcome {
    /// 64-character lowercase hexadecimal BLAKE3 digest identifying the content.
    pub blake3_hex: String,
    /// 64-character lowercase hexadecimal SHA-256 digest.
    pub sha256_hex: String,
    /// Storage key within the backend driver (e.g. `blobs/af/af1349b...bin`).
    pub storage_key: String,
    /// Byte size of the payload.
    pub byte_size: u64,
    /// Whether this write was brand new (`true`) or a deduplicated reuse hit (`false`).
    pub is_new: bool,
}

/// Cryptographic verification result for a stored content-addressed blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CasIntegrityOutcome {
    /// Content is intact and matches the expected cryptographic digests.
    Valid {
        /// BLAKE3 digest.
        blake3_hex: String,
        /// SHA-256 digest.
        sha256_hex: String,
        /// Exact byte size.
        byte_size: u64,
    },
    /// Content exists in storage but does not match its expected BLAKE3 digest.
    Corrupted {
        /// Requested BLAKE3 digest.
        blake3_hex: String,
        /// Expected BLAKE3 digest.
        expected_blake3: String,
        /// Actual calculated BLAKE3 digest from the retrieved payload.
        actual_blake3: String,
        /// Retrieved payload size in bytes.
        byte_size: u64,
    },
    /// Blob was not found in storage.
    NotFound {
        /// Requested BLAKE3 digest.
        blake3_hex: String,
    },
}

impl CasIntegrityOutcome {
    /// Returns `true` if the content is verified intact.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }
}

/// Metrics report produced when pruning oldest-accessed blobs to enforce a storage capacity quota.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasPruneReport {
    /// Number of blobs pruned.
    pub pruned_count: u64,
    /// Number of bytes freed from storage.
    pub reclaimed_bytes: u64,
    /// Initial storage usage in bytes before pruning.
    pub initial_bytes: u64,
    /// Final storage usage in bytes after pruning.
    pub final_bytes: u64,
}

/// Comprehensive audit summary of all indexed content-addressed blobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasAuditReport {
    /// Total number of indexed blobs audited.
    pub total_indexed: u64,
    /// Total bytes accounted for across all indexed blobs.
    pub total_bytes: u64,
    /// Number of blobs verified completely intact.
    pub verified_valid: u64,
    /// Number of indexed entries missing from physical storage.
    pub missing: u64,
    /// Number of blobs corrupted (digest mismatch).
    pub corrupted: u64,
    /// List of specific failure messages for missing or corrupted blobs.
    pub failures: Vec<String>,
}

impl CasAuditReport {
    /// Returns `true` if every audited blob is present and intact with zero errors.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.missing == 0 && self.corrupted == 0
    }
}

/// Realtime operational telemetry and cache efficiency statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasStats {
    /// Total unique blobs currently stored in CAS.
    pub total_blobs: u64,
    /// Total unique content bytes in CAS.
    pub total_bytes: u64,
    /// Total write requests received.
    pub write_operations: u64,
    /// Total writes that were skipped due to content-addressed deduplication.
    pub deduplicated_writes: u64,
    /// Total read requests that resulted in a cache hit.
    pub read_hits: u64,
    /// Total read requests that resulted in a cache miss.
    pub read_misses: u64,
}

impl CasStats {
    /// Computes the deduplication savings ratio ($0.0$ to $1.0$).
    #[must_use]
    pub fn deduplication_ratio(&self) -> f64 {
        if self.write_operations == 0 {
            0.0
        } else {
            self.deduplicated_writes as f64 / self.write_operations as f64
        }
    }
}

/// Internal atomic telemetry tracker.
#[derive(Debug, Default)]
struct CasStatsTracker {
    write_operations: AtomicU64,
    deduplicated_writes: AtomicU64,
    read_hits: AtomicU64,
    read_misses: AtomicU64,
}

/// Asynchronous repository abstraction managing the `content_cache_index` relational table.
#[async_trait]
pub trait ContentCacheIndexRepository: Send + Sync + std::fmt::Debug {
    /// Looks up a cache index row by exact `cache_key`.
    async fn get_by_key(&self, cache_key: &str) -> Result<Option<ContentCacheIndexRow>, CoreError>;

    /// Looks up a cache index row by 64-character lowercase `blake3_hex` digest.
    async fn get_by_blake3(
        &self,
        blake3_hex: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError>;

    /// Inserts or updates a content cache index record.
    async fn upsert(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError>;

    /// Updates the `last_accessed_at` timestamp of a record by `cache_key`.
    async fn touch(&self, cache_key: &str) -> Result<bool, CoreError>;

    /// Updates the `last_accessed_at` timestamp of all records matching `blake3_hex`.
    async fn touch_blake3(&self, blake3_hex: &str) -> Result<bool, CoreError>;

    /// Deletes a cache index record by `cache_key`.
    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, CoreError>;

    /// Deletes all cache index records referencing the specified `blake3_hex` digest.
    async fn delete_by_blake3(&self, blake3_hex: &str) -> Result<u64, CoreError>;

    /// Lists all records stored in the cache index.
    async fn list_all(&self) -> Result<Vec<ContentCacheIndexRow>, CoreError>;

    /// Lists records sorted by oldest `last_accessed_at` first, up to `limit`.
    async fn list_oldest_first(&self, limit: usize)
        -> Result<Vec<ContentCacheIndexRow>, CoreError>;

    /// Calculates total byte size of all indexed content.
    async fn total_bytes(&self) -> Result<u64, CoreError>;

    /// Returns the total count of indexed entries.
    async fn count(&self) -> Result<u64, CoreError>;

    /// Prunes all index entries whose `last_accessed_at` is older than `cutoff`.
    async fn prune_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError>;
}

type ContentCacheMap = Arc<RwLock<HashMap<String, ContentCacheIndexRow>>>;

/// In-memory concurrent implementation of [`ContentCacheIndexRepository`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryContentCacheIndexRepository {
    entries: ContentCacheMap,
}

impl InMemoryContentCacheIndexRepository {
    /// Constructs a new, empty in-memory repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContentCacheIndexRepository for InMemoryContentCacheIndexRepository {
    async fn get_by_key(&self, cache_key: &str) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        let map = self.entries.read().await;
        Ok(map.get(cache_key).cloned())
    }

    async fn get_by_blake3(
        &self,
        blake3_hex: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        let lower = blake3_hex.to_ascii_lowercase();
        let map = self.entries.read().await;
        for row in map.values() {
            if row.blake3_hex == lower {
                return Ok(Some(row.clone()));
            }
        }
        Ok(None)
    }

    async fn upsert(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError> {
        row.validate()?;
        let mut map = self.entries.write().await;
        map.insert(row.cache_key.clone(), row.clone());
        Ok(())
    }

    async fn touch(&self, cache_key: &str) -> Result<bool, CoreError> {
        let mut map = self.entries.write().await;
        if let Some(row) = map.get_mut(cache_key) {
            row.touch();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn touch_blake3(&self, blake3_hex: &str) -> Result<bool, CoreError> {
        let lower = blake3_hex.to_ascii_lowercase();
        let mut map = self.entries.write().await;
        let mut touched = false;
        for row in map.values_mut() {
            if row.blake3_hex == lower {
                row.touch();
                touched = true;
            }
        }
        Ok(touched)
    }

    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, CoreError> {
        let mut map = self.entries.write().await;
        Ok(map.remove(cache_key).is_some())
    }

    async fn delete_by_blake3(&self, blake3_hex: &str) -> Result<u64, CoreError> {
        let lower = blake3_hex.to_ascii_lowercase();
        let mut map = self.entries.write().await;
        let before = map.len();
        map.retain(|_, row| row.blake3_hex != lower);
        let removed = (before - map.len()) as u64;
        Ok(removed)
    }

    async fn list_all(&self) -> Result<Vec<ContentCacheIndexRow>, CoreError> {
        let map = self.entries.read().await;
        Ok(map.values().cloned().collect())
    }

    async fn list_oldest_first(
        &self,
        limit: usize,
    ) -> Result<Vec<ContentCacheIndexRow>, CoreError> {
        let map = self.entries.read().await;
        let mut rows: Vec<_> = map.values().cloned().collect();
        rows.sort_by_key(|r| r.last_accessed_at);
        if rows.len() > limit {
            rows.truncate(limit);
        }
        Ok(rows)
    }

    async fn total_bytes(&self) -> Result<u64, CoreError> {
        let map = self.entries.read().await;
        let sum = map.values().map(|r| r.byte_size.max(0) as u64).sum();
        Ok(sum)
    }

    async fn count(&self) -> Result<u64, CoreError> {
        let map = self.entries.read().await;
        Ok(map.len() as u64)
    }

    async fn prune_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut map = self.entries.write().await;
        let before = map.len();
        map.retain(|_, row| row.last_accessed_at >= cutoff);
        let removed = (before - map.len()) as u64;
        Ok(removed)
    }
}

/// Production PostgreSQL implementation of [`ContentCacheIndexRepository`] backed by SQLx.
#[derive(Debug, Clone)]
pub struct PostgresContentCacheIndexRepository {
    pool: sqlx::PgPool,
}

impl PostgresContentCacheIndexRepository {
    /// Constructs a new PostgreSQL repository wrapping a connection pool.
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentCacheIndexRepository for PostgresContentCacheIndexRepository {
    async fn get_by_key(&self, cache_key: &str) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        sqlx::query_as::<_, ContentCacheIndexRow>(
            "SELECT * FROM content_cache_index WHERE cache_key = $1",
        )
        .bind(cache_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to query content_cache_index by key: {err}"))
        })
    }

    async fn get_by_blake3(
        &self,
        blake3_hex: &str,
    ) -> Result<Option<ContentCacheIndexRow>, CoreError> {
        sqlx::query_as::<_, ContentCacheIndexRow>(
            "SELECT * FROM content_cache_index WHERE blake3_hex = $1 ORDER BY last_accessed_at DESC LIMIT 1",
        )
        .bind(blake3_hex.to_ascii_lowercase())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to query content_cache_index by blake3: {err}")))
    }

    async fn upsert(&self, row: &ContentCacheIndexRow) -> Result<(), CoreError> {
        row.validate()?;
        sqlx::query(
            r#"
            INSERT INTO content_cache_index (
                cache_key, blake3_hex, storage_key, byte_size, last_accessed_at
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (cache_key) DO UPDATE SET
                blake3_hex = EXCLUDED.blake3_hex,
                storage_key = EXCLUDED.storage_key,
                byte_size = EXCLUDED.byte_size,
                last_accessed_at = EXCLUDED.last_accessed_at
            "#,
        )
        .bind(&row.cache_key)
        .bind(&row.blake3_hex)
        .bind(&row.storage_key)
        .bind(row.byte_size)
        .bind(row.last_accessed_at)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to upsert content_cache_index: {err}"))
        })?;

        Ok(())
    }

    async fn touch(&self, cache_key: &str) -> Result<bool, CoreError> {
        let result = sqlx::query(
            "UPDATE content_cache_index SET last_accessed_at = NOW() WHERE cache_key = $1",
        )
        .bind(cache_key)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to touch content_cache_index by key: {err}"))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn touch_blake3(&self, blake3_hex: &str) -> Result<bool, CoreError> {
        let result = sqlx::query(
            "UPDATE content_cache_index SET last_accessed_at = NOW() WHERE blake3_hex = $1",
        )
        .bind(blake3_hex.to_ascii_lowercase())
        .execute(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to touch content_cache_index by blake3: {err}"
            ))
        })?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, CoreError> {
        let result = sqlx::query("DELETE FROM content_cache_index WHERE cache_key = $1")
            .bind(cache_key)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to delete content_cache_index by key: {err}"
                ))
            })?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_by_blake3(&self, blake3_hex: &str) -> Result<u64, CoreError> {
        let result = sqlx::query("DELETE FROM content_cache_index WHERE blake3_hex = $1")
            .bind(blake3_hex.to_ascii_lowercase())
            .execute(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to delete content_cache_index by blake3: {err}"
                ))
            })?;

        Ok(result.rows_affected())
    }

    async fn list_all(&self) -> Result<Vec<ContentCacheIndexRow>, CoreError> {
        sqlx::query_as::<_, ContentCacheIndexRow>(
            "SELECT * FROM content_cache_index ORDER BY last_accessed_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to list content_cache_index: {err}")))
    }

    async fn list_oldest_first(
        &self,
        limit: usize,
    ) -> Result<Vec<ContentCacheIndexRow>, CoreError> {
        sqlx::query_as::<_, ContentCacheIndexRow>(
            "SELECT * FROM content_cache_index ORDER BY last_accessed_at ASC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to query oldest content_cache_index: {err}"))
        })
    }

    async fn total_bytes(&self) -> Result<u64, CoreError> {
        let row: (Option<i64>,) =
            sqlx::query_as("SELECT COALESCE(SUM(byte_size), 0) FROM content_cache_index")
                .fetch_one(&self.pool)
                .await
                .map_err(|err| {
                    CoreError::Storage(format!("Failed to sum content_cache_index bytes: {err}"))
                })?;

        Ok(row.0.unwrap_or(0).max(0) as u64)
    }

    async fn count(&self) -> Result<u64, CoreError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM content_cache_index")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!("Failed to count content_cache_index: {err}"))
            })?;

        Ok(row.0.max(0) as u64)
    }

    async fn prune_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = sqlx::query("DELETE FROM content_cache_index WHERE last_accessed_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                CoreError::Storage(format!("Failed to prune stale content_cache_index: {err}"))
            })?;

        Ok(result.rows_affected())
    }
}

/// Unified asynchronous abstraction for Content-Addressed Storage (CAS).
#[async_trait]
pub trait ContentAddressedStorage: Send + Sync + std::fmt::Debug {
    /// Stores raw content bytes in Content-Addressed Storage under its canonical BLAKE3 address.
    ///
    /// If content already exists, skips rewriting and returns a deduplication outcome.
    async fn put(&self, data: &[u8]) -> Result<CasWriteOutcome, CoreError>;

    /// Stores raw content bytes under a custom caller-provided cache key (e.g. `url:<url>` or `git:blob:<sha>`).
    async fn put_with_key(
        &self,
        cache_key: &str,
        data: &[u8],
    ) -> Result<CasWriteOutcome, CoreError>;

    /// Retrieves raw content bytes by its 64-character BLAKE3 digest.
    ///
    /// Cryptographically verifies the content hash before returning bytes.
    async fn get(&self, blake3_hex: &str) -> Result<Option<Vec<u8>>, CoreError>;

    /// Retrieves raw content bytes by custom caller cache key.
    ///
    /// Cryptographically verifies the content hash before returning bytes.
    async fn get_by_key(&self, cache_key: &str) -> Result<Option<Vec<u8>>, CoreError>;

    /// Fast existence check for a BLAKE3 content digest.
    async fn contains(&self, blake3_hex: &str) -> Result<bool, CoreError>;

    /// Fast existence check for a custom caller cache key.
    async fn contains_key(&self, cache_key: &str) -> Result<bool, CoreError>;

    /// Batch existence check across multiple BLAKE3 content digests.
    async fn batch_contains(
        &self,
        blake3_hexes: &[&str],
    ) -> Result<HashMap<String, bool>, CoreError>;

    /// Batch retrieval of multiple content digests.
    async fn batch_get(&self, blake3_hexes: &[&str])
        -> Result<HashMap<String, Vec<u8>>, CoreError>;

    /// Deletes a content blob from storage and index by its BLAKE3 digest.
    async fn delete(&self, blake3_hex: &str) -> Result<bool, CoreError>;

    /// Deletes an entry by its custom caller cache key.
    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, CoreError>;

    /// Cryptographically verifies the integrity of a stored blob against its address.
    async fn verify_integrity(&self, blake3_hex: &str) -> Result<CasIntegrityOutcome, CoreError>;

    /// Enforces an upper storage capacity ceiling (in bytes) by pruning oldest-accessed blobs.
    async fn prune_lru(&self, max_allowed_bytes: u64) -> Result<CasPruneReport, CoreError>;

    /// Audits all indexed blobs against physical storage for cryptographic integrity.
    async fn audit(&self) -> Result<CasAuditReport, CoreError>;

    /// Returns live operational telemetry and deduplication metrics.
    async fn stats(&self) -> Result<CasStats, CoreError>;
}

/// Production Content-Addressed Storage coordinator implementation.
#[derive(Debug, Clone)]
pub struct CasEngine {
    storage: DynStorage,
    index: Arc<dyn ContentCacheIndexRepository>,
    stats: Arc<CasStatsTracker>,
}

impl CasEngine {
    /// Constructs a new `CasEngine` wrapping a storage backend and an index repository.
    #[must_use]
    pub fn new(storage: DynStorage, index: Arc<dyn ContentCacheIndexRepository>) -> Self {
        Self {
            storage,
            index,
            stats: Arc::new(CasStatsTracker::default()),
        }
    }

    /// Constructs a standalone, thread-safe in-memory `CasEngine` for tests and CLI usage.
    #[must_use]
    pub fn in_memory() -> Self {
        let storage: DynStorage = Arc::new(InMemoryStorage::new());
        let index = Arc::new(InMemoryContentCacheIndexRepository::new());
        Self::new(storage, index)
    }

    /// Generates the standard canonical cache key for a BLAKE3 digest: `blake3:<blake3_hex>`.
    #[must_use]
    pub fn canonical_cache_key(blake3_hex: &str) -> String {
        format!("blake3:{}", blake3_hex.to_ascii_lowercase())
    }
}

#[async_trait]
impl ContentAddressedStorage for CasEngine {
    async fn put(&self, data: &[u8]) -> Result<CasWriteOutcome, CoreError> {
        let digests = ArtifactDigests::compute(data);
        let canonical_key = Self::canonical_cache_key(&digests.blake3_hex);
        self.put_with_key(&canonical_key, data).await
    }

    async fn put_with_key(
        &self,
        cache_key: &str,
        data: &[u8],
    ) -> Result<CasWriteOutcome, CoreError> {
        self.stats.write_operations.fetch_add(1, Ordering::Relaxed);
        let digests = ArtifactDigests::compute(data);
        let storage_key = layout::blob_key(&digests.blake3_hex);

        // Check if content is already stored
        if self.storage.exists(&storage_key).await? {
            // Deduplication hit!
            self.stats
                .deduplicated_writes
                .fetch_add(1, Ordering::Relaxed);
            // Ensure index row exists/is updated
            let row = ContentCacheIndexRow::new(
                cache_key,
                &digests.blake3_hex,
                &storage_key,
                digests.byte_size as i64,
            );
            self.index.upsert(&row).await?;
            self.index.touch_blake3(&digests.blake3_hex).await?;

            return Ok(CasWriteOutcome {
                blake3_hex: digests.blake3_hex,
                sha256_hex: digests.sha256_hex,
                storage_key,
                byte_size: digests.byte_size,
                is_new: false,
            });
        }

        // New content write
        self.storage.put(&storage_key, data).await?;

        let row = ContentCacheIndexRow::new(
            cache_key,
            &digests.blake3_hex,
            &storage_key,
            digests.byte_size as i64,
        );
        self.index.upsert(&row).await?;

        // If custom cache_key was passed, also bind the canonical blake3: key
        let canonical_key = Self::canonical_cache_key(&digests.blake3_hex);
        if cache_key != canonical_key {
            let canonical_row = ContentCacheIndexRow::new(
                &canonical_key,
                &digests.blake3_hex,
                &storage_key,
                digests.byte_size as i64,
            );
            self.index.upsert(&canonical_row).await?;
        }

        Ok(CasWriteOutcome {
            blake3_hex: digests.blake3_hex,
            sha256_hex: digests.sha256_hex,
            storage_key,
            byte_size: digests.byte_size,
            is_new: true,
        })
    }

    async fn get(&self, blake3_hex: &str) -> Result<Option<Vec<u8>>, CoreError> {
        let storage_key = layout::blob_key(blake3_hex);
        if !self.storage.exists(&storage_key).await? {
            self.stats.read_misses.fetch_add(1, Ordering::Relaxed);
            return Ok(None);
        }

        let bytes = self.storage.get(&storage_key).await?;
        let digests = ArtifactDigests::compute(&bytes);

        if digests.blake3_hex != blake3_hex.to_ascii_lowercase() {
            return Err(CoreError::Storage(format!(
                "Content-addressed corruption detected for blob {blake3_hex}: actual digest is {}",
                digests.blake3_hex
            )));
        }

        self.stats.read_hits.fetch_add(1, Ordering::Relaxed);
        let _ = self.index.touch_blake3(blake3_hex).await;
        Ok(Some(bytes))
    }

    async fn get_by_key(&self, cache_key: &str) -> Result<Option<Vec<u8>>, CoreError> {
        if let Some(row) = self.index.get_by_key(cache_key).await? {
            let res = self.get(&row.blake3_hex).await?;
            let _ = self.index.touch(cache_key).await;
            Ok(res)
        } else {
            self.stats.read_misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn contains(&self, blake3_hex: &str) -> Result<bool, CoreError> {
        let storage_key = layout::blob_key(blake3_hex);
        self.storage.exists(&storage_key).await
    }

    async fn contains_key(&self, cache_key: &str) -> Result<bool, CoreError> {
        Ok(self.index.get_by_key(cache_key).await?.is_some())
    }

    async fn batch_contains(
        &self,
        blake3_hexes: &[&str],
    ) -> Result<HashMap<String, bool>, CoreError> {
        let mut results = HashMap::with_capacity(blake3_hexes.len());
        for hex in blake3_hexes {
            let exists = self.contains(hex).await?;
            results.insert((*hex).to_string(), exists);
        }
        Ok(results)
    }

    async fn batch_get(
        &self,
        blake3_hexes: &[&str],
    ) -> Result<HashMap<String, Vec<u8>>, CoreError> {
        let mut results = HashMap::with_capacity(blake3_hexes.len());
        for hex in blake3_hexes {
            if let Some(bytes) = self.get(hex).await? {
                results.insert((*hex).to_string(), bytes);
            }
        }
        Ok(results)
    }

    async fn delete(&self, blake3_hex: &str) -> Result<bool, CoreError> {
        let storage_key = layout::blob_key(blake3_hex);
        let storage_deleted = self.storage.delete(&storage_key).await.unwrap_or(false);
        let index_deleted = self.index.delete_by_blake3(blake3_hex).await.unwrap_or(0) > 0;
        Ok(storage_deleted || index_deleted)
    }

    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, CoreError> {
        if let Some(row) = self.index.get_by_key(cache_key).await? {
            self.index.delete_by_key(cache_key).await?;
            // Check if any other index entries still reference this blake3
            if self.index.get_by_blake3(&row.blake3_hex).await?.is_none() {
                self.storage.delete(&row.storage_key).await?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn verify_integrity(&self, blake3_hex: &str) -> Result<CasIntegrityOutcome, CoreError> {
        let storage_key = layout::blob_key(blake3_hex);
        if !self.storage.exists(&storage_key).await? {
            return Ok(CasIntegrityOutcome::NotFound {
                blake3_hex: blake3_hex.to_string(),
            });
        }

        let bytes = self.storage.get(&storage_key).await?;
        let digests = ArtifactDigests::compute(&bytes);

        if digests.blake3_hex == blake3_hex.to_ascii_lowercase() {
            Ok(CasIntegrityOutcome::Valid {
                blake3_hex: digests.blake3_hex,
                sha256_hex: digests.sha256_hex,
                byte_size: digests.byte_size,
            })
        } else {
            Ok(CasIntegrityOutcome::Corrupted {
                blake3_hex: blake3_hex.to_string(),
                expected_blake3: blake3_hex.to_ascii_lowercase(),
                actual_blake3: digests.blake3_hex,
                byte_size: digests.byte_size,
            })
        }
    }

    async fn prune_lru(&self, max_allowed_bytes: u64) -> Result<CasPruneReport, CoreError> {
        let initial_bytes = self.index.total_bytes().await?;
        if initial_bytes <= max_allowed_bytes {
            return Ok(CasPruneReport {
                pruned_count: 0,
                reclaimed_bytes: 0,
                initial_bytes,
                final_bytes: initial_bytes,
            });
        }

        let mut current_bytes = initial_bytes;
        let mut pruned_count = 0u64;
        let mut reclaimed_bytes = 0u64;

        // Fetch batches of oldest-accessed records
        const BATCH_SIZE: usize = 64;
        while current_bytes > max_allowed_bytes {
            let oldest = self.index.list_oldest_first(BATCH_SIZE).await?;
            if oldest.is_empty() {
                break;
            }

            for row in oldest {
                if current_bytes <= max_allowed_bytes {
                    break;
                }
                let blob_bytes = row.byte_size.max(0) as u64;
                let _ = self.delete(&row.blake3_hex).await;
                pruned_count += 1;
                reclaimed_bytes += blob_bytes;
                current_bytes = current_bytes.saturating_sub(blob_bytes);
            }
        }

        let final_bytes = self.index.total_bytes().await.unwrap_or(current_bytes);

        Ok(CasPruneReport {
            pruned_count,
            reclaimed_bytes,
            initial_bytes,
            final_bytes,
        })
    }

    async fn audit(&self) -> Result<CasAuditReport, CoreError> {
        let all_records = self.index.list_all().await?;
        let mut verified_valid = 0u64;
        let mut missing = 0u64;
        let mut corrupted = 0u64;
        let mut failures = Vec::new();
        let mut total_bytes = 0u64;

        // Deduplicate records by blake3_hex for audit
        let mut seen_blake3 = HashMap::new();
        for row in all_records {
            total_bytes += row.byte_size.max(0) as u64;
            seen_blake3.entry(row.blake3_hex.clone()).or_insert(row);
        }

        for (blake3, row) in &seen_blake3 {
            match self.verify_integrity(blake3).await? {
                CasIntegrityOutcome::Valid { .. } => {
                    verified_valid += 1;
                }
                CasIntegrityOutcome::NotFound { .. } => {
                    missing += 1;
                    failures.push(format!(
                        "Missing blob for key {}: storage key {}",
                        row.cache_key, row.storage_key
                    ));
                }
                CasIntegrityOutcome::Corrupted { actual_blake3, .. } => {
                    corrupted += 1;
                    failures.push(format!(
                        "Corrupted blob for key {}: expected {}, got {}",
                        row.cache_key, blake3, actual_blake3
                    ));
                }
            }
        }

        Ok(CasAuditReport {
            total_indexed: seen_blake3.len() as u64,
            total_bytes,
            verified_valid,
            missing,
            corrupted,
            failures,
        })
    }

    async fn stats(&self) -> Result<CasStats, CoreError> {
        let total_blobs = self.index.count().await?;
        let total_bytes = self.index.total_bytes().await?;

        Ok(CasStats {
            total_blobs,
            total_bytes,
            write_operations: self.stats.write_operations.load(Ordering::Relaxed),
            deduplicated_writes: self.stats.deduplicated_writes.load(Ordering::Relaxed),
            read_hits: self.stats.read_hits.load(Ordering::Relaxed),
            read_misses: self.stats.read_misses.load(Ordering::Relaxed),
        })
    }
}
