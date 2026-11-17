//! # Artifact Retention & Ephemeral Storage Cleanup Policies
//!
//! Provides automated, bounded lifecycle policies and storage cleanups for conversion artifacts:
//! - [`RetentionPolicy`]: Configurable TTL deadlines, ephemeral scratch TTLs, and high/low storage capacity watermarks.
//! - [`CleanupReport`]: Structured audit report tracking deleted artifacts, freed bytes, and encountered errors.
//! - [`ArtifactRepository`]: Relational CRUD and atomic expiration queries for `job_artifacts` in PostgreSQL.
//! - [`RetentionManager`]: Coordinated lifecycle manager executing expired DB sweeps, stale staging file purges, and watermark enforcement.

use std::fmt;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_repo_to_ai_core::config::StorageConfig;
use web_repo_to_ai_core::error::CoreError;

use crate::backend::DynStorage;
use crate::local_fs::layout;
use crate::schema::JobArtifactRow;

/// Default artifact retention duration in days (7 days).
pub const DEFAULT_RETENTION_DAYS: u32 = 7;

/// Minimum allowable retention TTL duration (60 seconds) to avoid immediate inadvertent deletion.
pub const MIN_RETENTION_SECS: i64 = 60;

/// Maximum allowable retention TTL duration (365 days).
pub const MAX_RETENTION_DAYS: u32 = 365;

/// Default TTL for ephemeral staging and temporary scratch files (1 hour).
pub const DEFAULT_EPHEMERAL_TTL_SECS: i64 = 3600;

/// Default high watermark percentage for storage capacity enforcement (85%).
pub const DEFAULT_HIGH_WATERMARK_PCT: f64 = 0.85;

/// Default low watermark percentage for storage capacity enforcement (70%).
pub const DEFAULT_LOW_WATERMARK_PCT: f64 = 0.70;

/// Default batch limit for transactional artifact cleanup queries.
pub const DEFAULT_CLEANUP_BATCH_SIZE: usize = 100;

/// Maximum permitted batch limit to avoid long table locks.
pub const MAX_CLEANUP_BATCH_SIZE: usize = 1000;

/// Configuration defining artifact expiration, ephemeral scratch lifetimes, and disk watermarks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Default time-to-live duration for finalized job artifacts.
    pub default_ttl: Duration,
    /// Absolute maximum time-to-live ceiling permitted for any artifact.
    pub max_ttl: Duration,
    /// Maximum age permitted for uncommitted temporary staging files (`.*.tmp.*`).
    pub ephemeral_ttl: Duration,
    /// High watermark threshold in bytes that triggers aggressive cleanup sweeps.
    pub high_watermark_bytes: u64,
    /// Target low watermark threshold in bytes to prune down to during capacity enforcement.
    pub low_watermark_bytes: u64,
    /// Maximum number of artifact records to process in a single cleanup query batch.
    pub batch_size: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        let max_storage_bytes: u64 = 15 * 1024 * 1024 * 1024; // 15 GB default
        Self {
            default_ttl: Duration::days(i64::from(DEFAULT_RETENTION_DAYS)),
            max_ttl: Duration::days(i64::from(MAX_RETENTION_DAYS)),
            ephemeral_ttl: Duration::seconds(DEFAULT_EPHEMERAL_TTL_SECS),
            high_watermark_bytes: (max_storage_bytes as f64 * DEFAULT_HIGH_WATERMARK_PCT) as u64,
            low_watermark_bytes: (max_storage_bytes as f64 * DEFAULT_LOW_WATERMARK_PCT) as u64,
            batch_size: DEFAULT_CLEANUP_BATCH_SIZE,
        }
    }
}

impl RetentionPolicy {
    /// Constructs a validated retention policy from explicit parameters.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if any duration or watermark invariant is violated.
    pub fn new(
        default_ttl: Duration,
        max_ttl: Duration,
        ephemeral_ttl: Duration,
        high_watermark_bytes: u64,
        low_watermark_bytes: u64,
        batch_size: usize,
    ) -> Result<Self, CoreError> {
        let policy = Self {
            default_ttl,
            max_ttl,
            ephemeral_ttl,
            high_watermark_bytes,
            low_watermark_bytes,
            batch_size,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Derives a validated retention policy from application [`StorageConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if derived parameters fail validation.
    pub fn from_config(config: &StorageConfig) -> Result<Self, CoreError> {
        let default_days = if config.artifact_retention_days == 0 {
            DEFAULT_RETENTION_DAYS
        } else {
            config.artifact_retention_days
        };

        let default_ttl = Duration::days(i64::from(default_days));
        let max_ttl = Duration::days(i64::from(MAX_RETENTION_DAYS));
        let ephemeral_ttl = Duration::seconds(DEFAULT_EPHEMERAL_TTL_SECS);

        let high_watermark_bytes =
            (config.max_storage_bytes as f64 * DEFAULT_HIGH_WATERMARK_PCT) as u64;
        let low_watermark_bytes =
            (config.max_storage_bytes as f64 * DEFAULT_LOW_WATERMARK_PCT) as u64;

        Self::new(
            default_ttl,
            max_ttl,
            ephemeral_ttl,
            high_watermark_bytes,
            low_watermark_bytes,
            DEFAULT_CLEANUP_BATCH_SIZE,
        )
    }

    /// Overrides the default artifact TTL.
    #[must_use]
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Overrides the maximum artifact TTL ceiling.
    #[must_use]
    pub fn with_max_ttl(mut self, ttl: Duration) -> Self {
        self.max_ttl = ttl;
        self
    }

    /// Overrides the ephemeral staging file TTL.
    #[must_use]
    pub fn with_ephemeral_ttl(mut self, ttl: Duration) -> Self {
        self.ephemeral_ttl = ttl;
        self
    }

    /// Overrides storage capacity watermark thresholds.
    #[must_use]
    pub fn with_watermarks(mut self, high: u64, low: u64) -> Self {
        self.high_watermark_bytes = high;
        self.low_watermark_bytes = low;
        self
    }

    /// Overrides the batch size for database cleanup queries.
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Validates retention policy invariants.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if TTL durations or watermarks are malformed.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.default_ttl < Duration::seconds(MIN_RETENTION_SECS) {
            return Err(CoreError::Validation {
                field: "default_ttl".to_string(),
                reason: format!(
                    "default_ttl must be at least {} seconds (actual: {}s)",
                    MIN_RETENTION_SECS,
                    self.default_ttl.num_seconds()
                ),
            });
        }

        if self.max_ttl < self.default_ttl {
            return Err(CoreError::Validation {
                field: "max_ttl".to_string(),
                reason: format!(
                    "max_ttl ({}s) cannot be less than default_ttl ({}s)",
                    self.max_ttl.num_seconds(),
                    self.default_ttl.num_seconds()
                ),
            });
        }

        if self.ephemeral_ttl <= Duration::zero() {
            return Err(CoreError::Validation {
                field: "ephemeral_ttl".to_string(),
                reason: "ephemeral_ttl must be strictly positive".to_string(),
            });
        }

        if self.low_watermark_bytes >= self.high_watermark_bytes {
            return Err(CoreError::Validation {
                field: "low_watermark_bytes".to_string(),
                reason: format!(
                    "low_watermark_bytes ({}) must be strictly less than high_watermark_bytes ({})",
                    self.low_watermark_bytes, self.high_watermark_bytes
                ),
            });
        }

        if self.batch_size == 0 || self.batch_size > MAX_CLEANUP_BATCH_SIZE {
            return Err(CoreError::Validation {
                field: "batch_size".to_string(),
                reason: format!(
                    "batch_size must be between 1 and {MAX_CLEANUP_BATCH_SIZE} (actual: {})",
                    self.batch_size
                ),
            });
        }

        Ok(())
    }

    /// Computes the exact expiration deadline for an artifact, bounding any caller-requested TTL.
    #[must_use]
    pub fn calculate_expires_at(
        &self,
        now: DateTime<Utc>,
        requested_ttl: Option<Duration>,
    ) -> DateTime<Utc> {
        let effective_ttl = match requested_ttl {
            Some(req) if req <= Duration::zero() => self.default_ttl,
            Some(req) if req > self.max_ttl => self.max_ttl,
            Some(req) => req,
            None => self.default_ttl,
        };

        now + effective_ttl
    }
}

/// Audit report detailing the results of a storage cleanup operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupReport {
    /// Number of artifact records evaluated during the cleanup cycle.
    pub artifacts_evaluated: usize,
    /// Number of finalized job artifacts successfully deleted.
    pub artifacts_deleted: usize,
    /// Total bytes freed from deleted finalized job artifacts.
    pub bytes_freed: u64,
    /// Number of orphaned or stale staging files removed.
    pub ephemeral_files_deleted: usize,
    /// Total bytes freed from removed ephemeral and staging files.
    pub ephemeral_bytes_freed: u64,
    /// Non-fatal error descriptions encountered during the cleanup pass.
    pub errors: Vec<String>,
}

impl CleanupReport {
    /// Instantiates an empty cleanup report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the successful deletion of a finalized artifact.
    pub fn record_artifact_deleted(&mut self, byte_size: u64) {
        self.artifacts_deleted += 1;
        self.bytes_freed += byte_size;
    }

    /// Records the successful removal of an ephemeral/staging file.
    pub fn record_ephemeral_deleted(&mut self, byte_size: u64) {
        self.ephemeral_files_deleted += 1;
        self.ephemeral_bytes_freed += byte_size;
    }

    /// Records a non-fatal error encountered during cleanup.
    pub fn record_error(&mut self, error: impl fmt::Display) {
        self.errors.push(error.to_string());
    }

    /// Merges another cleanup report into this one.
    pub fn merge(&mut self, other: Self) {
        self.artifacts_evaluated += other.artifacts_evaluated;
        self.artifacts_deleted += other.artifacts_deleted;
        self.bytes_freed += other.bytes_freed;
        self.ephemeral_files_deleted += other.ephemeral_files_deleted;
        self.ephemeral_bytes_freed += other.ephemeral_bytes_freed;
        self.errors.extend(other.errors);
    }

    /// Returns the combined count of finalized artifacts and ephemeral files deleted.
    #[must_use]
    pub fn total_files_deleted(&self) -> usize {
        self.artifacts_deleted + self.ephemeral_files_deleted
    }

    /// Returns the combined bytes freed across all deleted files.
    #[must_use]
    pub fn total_bytes_freed(&self) -> u64 {
        self.bytes_freed + self.ephemeral_bytes_freed
    }

    /// Returns `true` if no files were deleted and no errors occurred.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_files_deleted() == 0 && self.errors.is_empty()
    }

    /// Returns `true` if one or more non-fatal errors were recorded.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns `true` if the cleanup completed without any errors.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Relational repository executing SQLx operations on the `job_artifacts` table.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArtifactRepository;

impl ArtifactRepository {
    /// Inserts a new artifact record into the `job_artifacts` table.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if the entity fails validation.
    /// Returns [`CoreError::Storage`] if database insertion fails.
    pub async fn create_artifact(
        pool: &sqlx::PgPool,
        artifact: &JobArtifactRow,
    ) -> Result<JobArtifactRow, CoreError> {
        artifact.validate()?;

        let created: JobArtifactRow = sqlx::query_as(
            r#"
            INSERT INTO job_artifacts (
                id, job_id, format, storage_key, byte_size,
                sha256_hex, blake3_hex, expires_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(artifact.id)
        .bind(artifact.job_id)
        .bind(&artifact.format)
        .bind(&artifact.storage_key)
        .bind(artifact.byte_size)
        .bind(&artifact.sha256_hex)
        .bind(&artifact.blake3_hex)
        .bind(artifact.expires_at)
        .bind(artifact.created_at)
        .fetch_one(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to insert job artifact '{}': {err}",
                artifact.id
            ))
        })?;

        Ok(created)
    }

    /// Fetches an artifact record by its primary key UUID.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn get_artifact(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<Option<JobArtifactRow>, CoreError> {
        let row = sqlx::query_as(
            r#"
            SELECT * FROM job_artifacts WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to fetch artifact {id}: {err}")))?;

        Ok(row)
    }

    /// Fetches an artifact record by parent job ID and format string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn get_artifact_by_format(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        format: &str,
    ) -> Result<Option<JobArtifactRow>, CoreError> {
        let row = sqlx::query_as(
            r#"
            SELECT * FROM job_artifacts
            WHERE job_id = $1 AND format = $2
            "#,
        )
        .bind(job_id)
        .bind(format)
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to fetch artifact for job {job_id} format '{format}': {err}"
            ))
        })?;

        Ok(row)
    }

    /// Lists all artifact records associated with a specific job ID.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn list_artifacts_for_job(
        pool: &sqlx::PgPool,
        job_id: Uuid,
    ) -> Result<Vec<JobArtifactRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM job_artifacts
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to list artifacts for job {job_id}: {err}"))
        })?;

        Ok(rows)
    }

    /// Queries expired artifacts whose `expires_at` is less than or equal to `now`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn find_expired_artifacts(
        pool: &sqlx::PgPool,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<JobArtifactRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM job_artifacts
            WHERE expires_at <= $1
            ORDER BY expires_at ASC
            LIMIT $2
            "#,
        )
        .bind(now)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to find expired artifacts: {err}")))?;

        Ok(rows)
    }

    /// Atomically deletes a single artifact record by ID.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database deletion fails.
    pub async fn delete_artifact(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<Option<JobArtifactRow>, CoreError> {
        let row = sqlx::query_as(
            r#"
            DELETE FROM job_artifacts
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to delete artifact {id}: {err}")))?;

        Ok(row)
    }

    /// Deletes all artifact records for a job and returns the deleted records.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database deletion fails.
    pub async fn delete_job_artifacts(
        pool: &sqlx::PgPool,
        job_id: Uuid,
    ) -> Result<Vec<JobArtifactRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            DELETE FROM job_artifacts
            WHERE job_id = $1
            RETURNING *
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to delete artifacts for job {job_id}: {err}"
            ))
        })?;

        Ok(rows)
    }

    /// Atomically deletes a batch of expired artifacts using row-level skip-locking.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database transaction fails.
    pub async fn delete_expired_artifacts_batch(
        pool: &sqlx::PgPool,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<JobArtifactRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            DELETE FROM job_artifacts
            WHERE id IN (
                SELECT id FROM job_artifacts
                WHERE expires_at <= $1
                ORDER BY expires_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .bind(now)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to delete expired artifacts batch: {err}"))
        })?;

        Ok(rows)
    }

    /// Retrieves the oldest artifacts ordered chronologically by creation timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn find_oldest_artifacts(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> Result<Vec<JobArtifactRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM job_artifacts
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to query oldest artifacts: {err}")))?;

        Ok(rows)
    }

    /// Counts the total number of artifact records currently stored in PostgreSQL.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database count fails.
    pub async fn count_artifacts(pool: &sqlx::PgPool) -> Result<i64, CoreError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM job_artifacts
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to count artifacts: {err}")))?;

        Ok(count.0)
    }

    /// Sums the total bytes consumed by all stored artifact records in PostgreSQL.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn total_artifact_bytes(pool: &sqlx::PgPool) -> Result<i64, CoreError> {
        let total: (Option<i64>,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(byte_size), 0) FROM job_artifacts
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to query total artifact bytes: {err}"))
        })?;

        Ok(total.0.unwrap_or(0))
    }
}

/// Central lifecycle and retention coordinator executing storage purges.
#[derive(Debug, Clone)]
pub struct RetentionManager {
    storage: DynStorage,
    policy: RetentionPolicy,
}

impl RetentionManager {
    /// Instantiates a new retention manager with validated policy.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if policy fails validation.
    pub fn new(storage: DynStorage, policy: RetentionPolicy) -> Result<Self, CoreError> {
        policy.validate()?;
        Ok(Self { storage, policy })
    }

    /// Returns a reference to the active storage backend.
    #[must_use]
    pub fn storage(&self) -> &DynStorage {
        &self.storage
    }

    /// Returns a reference to the active retention policy.
    #[must_use]
    pub fn policy(&self) -> &RetentionPolicy {
        &self.policy
    }

    /// Sweeps and deletes expired artifacts from both the database and the storage backend.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database deletion fails.
    pub async fn cleanup_expired_artifacts(
        &self,
        pool: &sqlx::PgPool,
        now: DateTime<Utc>,
    ) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();
        let batch_size = self.policy.batch_size as i64;

        loop {
            let expired_batch =
                ArtifactRepository::delete_expired_artifacts_batch(pool, now, batch_size).await?;
            if expired_batch.is_empty() {
                break;
            }

            report.artifacts_evaluated += expired_batch.len();

            for artifact in expired_batch {
                let key = &artifact.storage_key;
                match self.storage.delete(key).await {
                    Ok(deleted) => {
                        if deleted {
                            report.record_artifact_deleted(artifact.byte_size as u64);
                        } else {
                            // Already deleted from storage or missing
                            report.record_artifact_deleted(0);
                        }
                    }
                    Err(err) => {
                        report.record_error(format!(
                            "Failed to delete artifact file '{key}' from storage: {err}"
                        ));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Deletes all artifacts and storage files associated with a specific job.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database operations fail.
    pub async fn cleanup_job_artifacts(
        &self,
        pool: &sqlx::PgPool,
        job_id: Uuid,
    ) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();

        // 1. Delete relational records
        let deleted_records = ArtifactRepository::delete_job_artifacts(pool, job_id).await?;
        report.artifacts_evaluated += deleted_records.len();

        for record in deleted_records {
            report.record_artifact_deleted(record.byte_size as u64);
        }

        // 2. Delete storage directory
        let dir_key = layout::job_dir_key(job_id);
        match self.storage.delete_directory(&dir_key).await {
            Ok(_) => {}
            Err(err) => {
                report.record_error(format!("Failed to delete job directory '{dir_key}': {err}"));
            }
        }

        Ok(report)
    }

    /// Directly deletes a job's artifact directory from storage without requiring a database pool.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if storage deletion fails.
    pub async fn cleanup_job_storage(&self, job_id: Uuid) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();
        let dir_key = layout::job_dir_key(job_id);

        match self.storage.delete_directory(&dir_key).await {
            Ok(deleted) => {
                if deleted {
                    report.record_artifact_deleted(0);
                }
            }
            Err(err) => {
                report.record_error(format!(
                    "Failed to delete job storage directory '{dir_key}': {err}"
                ));
            }
        }

        Ok(report)
    }

    /// Scans the storage backend for stale staging files older than `max_age` and removes them.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if scanning fails.
    pub async fn cleanup_stale_staging_files(
        &self,
        max_age: Duration,
    ) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();
        match self.storage.cleanup_stale_staging_files(max_age).await {
            Ok((count, bytes)) => {
                report.ephemeral_files_deleted = count;
                report.ephemeral_bytes_freed = bytes;
            }
            Err(err) => {
                report.record_error(format!("Failed to cleanup stale staging files: {err}"));
            }
        }
        Ok(report)
    }

    /// Enforces storage capacity limits by pruning oldest files when usage exceeds the high watermark.
    ///
    /// If storage usage exceeds `high_watermark_bytes`:
    /// 1. Sweeps expired database artifacts first (if `pool` is provided).
    /// 2. If still above `low_watermark_bytes`, prunes oldest artifacts until below `low_watermark_bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if storage capacity query fails.
    pub async fn enforce_capacity_watermark(
        &self,
        pool: Option<&sqlx::PgPool>,
        now: DateTime<Utc>,
    ) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();
        let usage = self.storage.storage_capacity().await?;

        if usage.total_artifact_bytes <= self.policy.high_watermark_bytes {
            return Ok(report);
        }

        // 1. Sweep expired DB artifacts if pool is available
        if let Some(pg) = pool {
            let expired_report = self.cleanup_expired_artifacts(pg, now).await?;
            report.merge(expired_report);
        }

        // 2. Re-check capacity
        let current_usage = self.storage.storage_capacity().await?;
        if current_usage.total_artifact_bytes <= self.policy.low_watermark_bytes {
            return Ok(report);
        }

        let mut bytes_to_free =
            current_usage.total_artifact_bytes - self.policy.low_watermark_bytes;

        // 3. Prune oldest artifacts to reach low watermark
        if let Some(pg) = pool {
            let oldest_batch =
                ArtifactRepository::find_oldest_artifacts(pg, self.policy.batch_size as i64)
                    .await?;

            for artifact in oldest_batch {
                if bytes_to_free == 0 {
                    break;
                }

                if let Ok(Some(deleted_row)) =
                    ArtifactRepository::delete_artifact(pg, artifact.id).await
                {
                    match self.storage.delete(&deleted_row.storage_key).await {
                        Ok(deleted) => {
                            if deleted {
                                let freed = deleted_row.byte_size as u64;
                                report.record_artifact_deleted(freed);
                                bytes_to_free = bytes_to_free.saturating_sub(freed);
                            }
                        }
                        Err(err) => {
                            report.record_error(format!(
                                "Failed to delete storage file '{}': {err}",
                                deleted_row.storage_key
                            ));
                        }
                    }
                }
            }
        } else {
            // Pure storage mode: list and sort keys by modified timestamp
            let keys = self.storage.list(None).await?;
            let mut key_metas = Vec::new();

            for key in keys {
                if let Ok(meta) = self.storage.metadata(&key).await {
                    key_metas.push(meta);
                }
            }

            key_metas.sort_by_key(|m| m.modified_at);

            for meta in key_metas {
                if bytes_to_free == 0 {
                    break;
                }

                match self.storage.delete(&meta.key).await {
                    Ok(deleted) => {
                        if deleted {
                            let freed = meta.byte_size;
                            report.record_artifact_deleted(freed);
                            bytes_to_free = bytes_to_free.saturating_sub(freed);
                        }
                    }
                    Err(err) => {
                        report.record_error(format!(
                            "Failed to delete storage file '{}': {err}",
                            meta.key
                        ));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Executes a complete retention and cleanup sweep cycle.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if catastrophic storage errors occur.
    pub async fn run_full_sweep(
        &self,
        pool: Option<&sqlx::PgPool>,
        now: DateTime<Utc>,
    ) -> Result<CleanupReport, CoreError> {
        let mut report = CleanupReport::new();

        // 1. Stale temporary staging files
        let staging_report = self
            .cleanup_stale_staging_files(self.policy.ephemeral_ttl)
            .await?;
        report.merge(staging_report);

        // 2. Expired database artifacts (if pool provided)
        if let Some(pg) = pool {
            let expired_report = self.cleanup_expired_artifacts(pg, now).await?;
            report.merge(expired_report);
        }

        // 3. Watermark capacity enforcement
        let watermark_report = self.enforce_capacity_watermark(pool, now).await?;
        report.merge(watermark_report);

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{InMemoryStorage, StorageBackend};
    use crate::local_fs::LocalFsStorage;
    use crate::schema::ArtifactFormat;
    use std::path::PathBuf;
    use std::sync::Arc;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "web_repo_to_ai_retention_{name}_{}",
                Uuid::now_v7()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("Failed to create test dir");
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_retention_policy_defaults_and_validation() {
        let policy = RetentionPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.default_ttl, Duration::days(7));
        assert_eq!(policy.max_ttl, Duration::days(365));
        assert_eq!(policy.ephemeral_ttl, Duration::seconds(3600));
        assert!(policy.low_watermark_bytes < policy.high_watermark_bytes);
        assert_eq!(policy.batch_size, 100);

        // Inverted watermarks
        let invalid_watermarks = policy.clone().with_watermarks(100, 200);
        assert!(invalid_watermarks.validate().is_err());

        // Zero TTL
        let zero_ttl = policy.clone().with_default_ttl(Duration::seconds(10));
        assert!(zero_ttl.validate().is_err());

        // Max TTL < default TTL
        let invalid_max = policy.clone().with_max_ttl(Duration::days(1));
        assert!(invalid_max.validate().is_err());

        // Excessive batch size
        let invalid_batch = policy.with_batch_size(2000);
        assert!(invalid_batch.validate().is_err());
    }

    #[test]
    fn test_retention_policy_calculate_expires_at() {
        let policy = RetentionPolicy::default();
        let now = Utc::now();

        // Default TTL applied
        let default_exp = policy.calculate_expires_at(now, None);
        assert_eq!(default_exp, now + Duration::days(7));

        // Custom valid TTL requested
        let custom_ttl = Duration::days(14);
        let custom_exp = policy.calculate_expires_at(now, Some(custom_ttl));
        assert_eq!(custom_exp, now + Duration::days(14));

        // Requested TTL exceeding max TTL clamped
        let excess_ttl = Duration::days(500);
        let clamped_exp = policy.calculate_expires_at(now, Some(excess_ttl));
        assert_eq!(clamped_exp, now + Duration::days(365));

        // Negative or zero requested TTL falls back to default
        let zero_exp = policy.calculate_expires_at(now, Some(Duration::seconds(0)));
        assert_eq!(zero_exp, now + Duration::days(7));
    }

    #[test]
    fn test_cleanup_report_metrics_and_merge() {
        let mut report1 = CleanupReport::new();
        assert!(report1.is_empty());
        assert!(report1.is_success());

        report1.record_artifact_deleted(1024);
        report1.record_ephemeral_deleted(512);
        assert_eq!(report1.total_files_deleted(), 2);
        assert_eq!(report1.total_bytes_freed(), 1536);

        let mut report2 = CleanupReport::new();
        report2.record_artifact_deleted(2048);
        report2.record_error("Storage backend timeout");
        assert!(report2.has_errors());
        assert!(!report2.is_success());

        report1.merge(report2);
        assert_eq!(report1.artifacts_deleted, 2);
        assert_eq!(report1.ephemeral_files_deleted, 1);
        assert_eq!(report1.total_bytes_freed(), 3584);
        assert_eq!(report1.errors.len(), 1);
        assert!(!report1.is_success());
    }

    #[tokio::test]
    async fn test_in_memory_stale_staging_cleanup() {
        let storage = Arc::new(InMemoryStorage::new());
        let policy = RetentionPolicy::default().with_ephemeral_ttl(Duration::seconds(1));
        let manager = RetentionManager::new(storage.clone(), policy).unwrap();

        // Put legitimate file and staging file
        storage
            .put("jobs/job1/artifact.md", b"# Final Artifact")
            .await
            .unwrap();
        storage
            .put("jobs/job1/.artifact.tmp.12345", b"# Staging Artifact")
            .await
            .unwrap();

        // Staging cleanup with high max_age does nothing
        let report_high = manager
            .cleanup_stale_staging_files(Duration::hours(1))
            .await
            .unwrap();
        assert_eq!(report_high.ephemeral_files_deleted, 0);
        assert!(storage
            .exists("jobs/job1/.artifact.tmp.12345")
            .await
            .unwrap());

        // Staging cleanup with 0 max_age removes the temp file
        let report_zero = manager
            .cleanup_stale_staging_files(Duration::zero())
            .await
            .unwrap();
        assert_eq!(report_zero.ephemeral_files_deleted, 1);
        assert_eq!(
            report_zero.ephemeral_bytes_freed,
            b"# Staging Artifact".len() as u64
        );
        assert!(!storage
            .exists("jobs/job1/.artifact.tmp.12345")
            .await
            .unwrap());
        assert!(storage.exists("jobs/job1/artifact.md").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_watermark_capacity_enforcement() {
        let storage = Arc::new(InMemoryStorage::new());
        // Set low high-watermark of 30 bytes, low watermark of 15 bytes
        let policy = RetentionPolicy::default().with_watermarks(30, 15);
        let manager = RetentionManager::new(storage.clone(), policy).unwrap();

        // Put 3 files of 10 bytes each = 30 bytes (at high watermark)
        storage.put("jobs/j1/f1.txt", b"0123456789").await.unwrap();
        storage.put("jobs/j2/f2.txt", b"0123456789").await.unwrap();
        storage.put("jobs/j3/f3.txt", b"0123456789").await.unwrap();

        let report_at_limit = manager
            .enforce_capacity_watermark(None, Utc::now())
            .await
            .unwrap();
        assert_eq!(report_at_limit.artifacts_deleted, 0);

        // Put a 4th file = 40 bytes (exceeds high watermark)
        storage.put("jobs/j4/f4.txt", b"0123456789").await.unwrap();

        let report_exceeded = manager
            .enforce_capacity_watermark(None, Utc::now())
            .await
            .unwrap();
        assert!(report_exceeded.artifacts_deleted >= 2);
        assert!(report_exceeded.bytes_freed >= 20);

        let usage = storage.storage_capacity().await.unwrap();
        assert!(usage.total_artifact_bytes <= 15);
    }

    #[tokio::test]
    async fn test_local_fs_stale_staging_cleanup() {
        let test_dir = TestDir::new("stale_staging");
        let storage = Arc::new(LocalFsStorage::new(&test_dir.0).unwrap());
        let policy = RetentionPolicy::default();
        let manager = RetentionManager::new(storage.clone(), policy).unwrap();

        let job_id = Uuid::now_v7();
        let valid_key = layout::job_artifact_key(job_id, ArtifactFormat::Markdown);
        storage
            .put(&valid_key, b"Valid artifact content")
            .await
            .unwrap();

        // Manually write an orphaned staging file
        let job_dir = test_dir.0.join(format!("jobs/{job_id}"));
        let staging_file = job_dir.join(".artifact.tmp.9999");
        tokio::fs::write(&staging_file, b"Orphaned staging data")
            .await
            .unwrap();
        assert!(tokio::fs::try_exists(&staging_file).await.unwrap());

        // Run staging cleanup with zero max_age
        let report = manager
            .cleanup_stale_staging_files(Duration::zero())
            .await
            .unwrap();
        assert_eq!(report.ephemeral_files_deleted, 1);
        assert_eq!(
            report.ephemeral_bytes_freed,
            b"Orphaned staging data".len() as u64
        );

        assert!(!tokio::fs::try_exists(&staging_file).await.unwrap());
        assert!(storage.exists(&valid_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_job_storage_directory() {
        let storage = Arc::new(InMemoryStorage::new());
        let policy = RetentionPolicy::default();
        let manager = RetentionManager::new(storage.clone(), policy).unwrap();

        let job_id = Uuid::now_v7();
        let key1 = format!("jobs/{job_id}/artifact.md");
        let key2 = format!("jobs/{job_id}/manifest.json");
        storage.put(&key1, b"# Content").await.unwrap();
        storage.put(&key2, b"{}").await.unwrap();

        assert!(storage.exists(&key1).await.unwrap());
        assert!(storage.exists(&key2).await.unwrap());

        let report = manager.cleanup_job_storage(job_id).await.unwrap();
        assert_eq!(report.artifacts_deleted, 1);
        assert!(!storage.exists(&key1).await.unwrap());
        assert!(!storage.exists(&key2).await.unwrap());
    }
}
