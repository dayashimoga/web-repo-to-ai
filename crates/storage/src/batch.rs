//! # Batch Job and Item Tracking Persistence
//!
//! Provides relational and in-memory persistence abstractions for tracking multi-URL batch
//! conversions, progress metrics, and individual URL item execution outcomes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::{BatchId, JobId};

/// Lifecycle status of a multi-URL batch conversion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchJobStatus {
    /// Batch is queued awaiting worker leasing.
    Queued,
    /// Batch items are currently processing in parallel.
    Processing,
    /// All items in the batch completed successfully.
    Completed,
    /// Some items completed, but one or more items failed.
    PartialFailure,
    /// All items in the batch failed.
    Failed,
    /// The batch was cancelled by client request or system intervention.
    Cancelled,
}

impl BatchJobStatus {
    /// Returns the database string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::PartialFailure => "partial_failure",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parses from database string representation.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if string does not match any valid variant.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s.to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "partial_failure" => Ok(Self::PartialFailure),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(CoreError::Validation {
                field: "status".to_string(),
                reason: format!("Unknown batch status: '{other}'"),
            }),
        }
    }

    /// Returns `true` if the status is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::PartialFailure | Self::Failed | Self::Cancelled
        )
    }
}

/// Lifecycle status of an individual item within a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchItemStatusKind {
    /// Item is queued waiting for an available worker permit.
    Queued,
    /// Item is actively undergoing acquisition, extraction, or optimization.
    Processing,
    /// Item completed conversion successfully.
    Completed,
    /// Item encountered an unrecoverable extraction or conversion error.
    Failed,
    /// Item exceeded its allocated execution timeout.
    TimedOut,
    /// Item was cancelled cooperatively before completion.
    Cancelled,
}

impl BatchItemStatusKind {
    /// Returns the database string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parses from database string representation.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if string does not match any valid variant.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        match s.to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "timed_out" => Ok(Self::TimedOut),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(CoreError::Validation {
                field: "status".to_string(),
                reason: format!("Unknown batch item status: '{other}'"),
            }),
        }
    }

    /// Returns `true` if the item is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::TimedOut | Self::Cancelled
        )
    }
}

/// Represents a persistent parent batch conversion record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRecord {
    /// Strongly-typed UUID v7 batch identifier.
    pub id: BatchId,
    /// Optional human-readable batch name or label.
    pub name: Option<String>,
    /// High-level lifecycle state.
    pub status: BatchJobStatus,
    /// Total count of member URLs in this batch.
    pub total_items: usize,
    /// Count of items currently waiting in queue.
    pub queued_items: usize,
    /// Count of items actively processing in parallel.
    pub processing_items: usize,
    /// Count of items successfully completed.
    pub completed_items: usize,
    /// Count of items that failed or timed out.
    pub failed_items: usize,
    /// Maximum configured concurrent tasks.
    pub max_parallelism: usize,
    /// Client web session identifier, if supplied.
    pub owner_session_id: Option<Uuid>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
    /// Terminal completion timestamp, if finished.
    pub completed_at: Option<DateTime<Utc>>,
}

/// Represents a persistent individual URL task record within a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemRecord {
    /// Parent batch identifier.
    pub batch_id: BatchId,
    /// Zero-based sequence index within the batch submission.
    pub item_index: usize,
    /// Strongly-typed UUID v7 sub-job identifier.
    pub job_id: JobId,
    /// Target resource URL.
    pub url: String,
    /// Optional client-provided correlation identifier.
    pub custom_id: Option<String>,
    /// Execution status of this item.
    pub status: BatchItemStatusKind,
    /// Machine-readable error code if failed.
    pub error_code: Option<String>,
    /// Detailed error message if failed.
    pub error_message: Option<String>,
    /// Total execution duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// Count of exported artifacts produced.
    pub artifact_count: usize,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp.
    pub completed_at: Option<DateTime<Utc>>,
}

/// Asynchronous repository trait for batch jobs and item result persistence.
#[async_trait]
pub trait BatchRepository: Send + Sync {
    /// Creates and persists a new batch header along with its initial item records.
    ///
    /// # Errors
    /// Returns [`CoreError`] if persistence fails.
    async fn create_batch(
        &self,
        batch: BatchRecord,
        items: Vec<BatchItemRecord>,
    ) -> Result<BatchRecord, CoreError>;

    /// Retrieves a batch record by its unique identifier.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database query failure.
    async fn get_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError>;

    /// Retrieves all item records for a given batch, ordered deterministically by `item_index`.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database query failure.
    async fn get_batch_items(&self, batch_id: BatchId) -> Result<Vec<BatchItemRecord>, CoreError>;

    /// Retrieves an individual item record within a batch by its sequence index.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database query failure.
    async fn get_batch_item(
        &self,
        batch_id: BatchId,
        item_index: usize,
    ) -> Result<Option<BatchItemRecord>, CoreError>;

    /// Retrieves an individual batch item record by its child `JobId`.
    ///
    /// # Errors
    /// Returns [`CoreError`] on database query failure.
    async fn get_batch_item_by_job_id(
        &self,
        job_id: JobId,
    ) -> Result<Option<BatchItemRecord>, CoreError>;

    /// Updates the execution status of a specific item within a batch, automatically
    /// recalculating the parent batch's progress counters and status.
    ///
    /// # Errors
    /// Returns [`CoreError`] if update fails.
    async fn update_item_status(
        &self,
        batch_id: BatchId,
        item_index: usize,
        status: BatchItemStatusKind,
        error_code: Option<String>,
        error_message: Option<String>,
        duration_ms: Option<u64>,
    ) -> Result<Option<BatchItemRecord>, CoreError>;

    /// Cancels all remaining queued/processing items in a batch and sets batch status to `Cancelled`.
    ///
    /// # Errors
    /// Returns [`CoreError`] on failure.
    async fn cancel_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError>;

    /// Lists batch jobs for an optional owner session with pagination.
    ///
    /// # Errors
    /// Returns [`CoreError`] on query failure.
    async fn list_batches(
        &self,
        owner_session_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BatchRecord>, CoreError>;
}

type InMemoryBatchStore = Arc<RwLock<HashMap<BatchId, (BatchRecord, Vec<BatchItemRecord>)>>>;

/// In-memory thread-safe implementation of [`BatchRepository`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryBatchRepository {
    storage: InMemoryBatchStore,
}

impl InMemoryBatchRepository {
    /// Creates a new empty `InMemoryBatchRepository`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl BatchRepository for InMemoryBatchRepository {
    async fn create_batch(
        &self,
        batch: BatchRecord,
        items: Vec<BatchItemRecord>,
    ) -> Result<BatchRecord, CoreError> {
        let mut map = self.storage.write().await;
        let id = batch.id;
        map.insert(id, (batch.clone(), items));
        Ok(batch)
    }

    async fn get_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError> {
        let map = self.storage.read().await;
        Ok(map.get(&batch_id).map(|(b, _)| b.clone()))
    }

    async fn get_batch_items(&self, batch_id: BatchId) -> Result<Vec<BatchItemRecord>, CoreError> {
        let map = self.storage.read().await;
        if let Some((_, items)) = map.get(&batch_id) {
            let mut sorted = items.clone();
            sorted.sort_by_key(|i| i.item_index);
            Ok(sorted)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_batch_item(
        &self,
        batch_id: BatchId,
        item_index: usize,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let map = self.storage.read().await;
        if let Some((_, items)) = map.get(&batch_id) {
            Ok(items.iter().find(|i| i.item_index == item_index).cloned())
        } else {
            Ok(None)
        }
    }

    async fn get_batch_item_by_job_id(
        &self,
        job_id: JobId,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let map = self.storage.read().await;
        for (_, items) in map.values() {
            if let Some(item) = items.iter().find(|i| i.job_id == job_id) {
                return Ok(Some(item.clone()));
            }
        }
        Ok(None)
    }

    async fn update_item_status(
        &self,
        batch_id: BatchId,
        item_index: usize,
        status: BatchItemStatusKind,
        error_code: Option<String>,
        error_message: Option<String>,
        duration_ms: Option<u64>,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let mut map = self.storage.write().await;
        let entry = match map.get_mut(&batch_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        let now = Utc::now();
        let (ref mut batch, ref mut items) = *entry;

        let item = match items.iter_mut().find(|i| i.item_index == item_index) {
            Some(it) => it,
            None => return Ok(None),
        };

        item.status = status;
        item.updated_at = now;
        if let Some(code) = error_code {
            item.error_code = Some(code);
        }
        if let Some(msg) = error_message {
            item.error_message = Some(msg);
        }
        if let Some(dur) = duration_ms {
            item.duration_ms = Some(dur);
        }
        if status.is_terminal() {
            item.completed_at = Some(now);
        }

        let updated_item = item.clone();

        // Recalculate parent batch counters
        let mut queued = 0;
        let mut processing = 0;
        let mut completed = 0;
        let mut failed = 0;

        for it in items.iter() {
            match it.status {
                BatchItemStatusKind::Queued => queued += 1,
                BatchItemStatusKind::Processing => processing += 1,
                BatchItemStatusKind::Completed => completed += 1,
                BatchItemStatusKind::Failed | BatchItemStatusKind::TimedOut => failed += 1,
                BatchItemStatusKind::Cancelled => failed += 1,
            }
        }

        batch.queued_items = queued;
        batch.processing_items = processing;
        batch.completed_items = completed;
        batch.failed_items = failed;
        batch.updated_at = now;

        // Auto-transition parent status when all items finish
        let total = batch.total_items;
        if completed + failed == total && total > 0 {
            batch.completed_at = Some(now);
            if failed == 0 {
                batch.status = BatchJobStatus::Completed;
            } else if completed > 0 {
                batch.status = BatchJobStatus::PartialFailure;
            } else {
                batch.status = BatchJobStatus::Failed;
            }
        } else if processing > 0 {
            batch.status = BatchJobStatus::Processing;
        }

        Ok(Some(updated_item))
    }

    async fn cancel_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError> {
        let mut map = self.storage.write().await;
        let entry = match map.get_mut(&batch_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        let now = Utc::now();
        let (ref mut batch, ref mut items) = *entry;

        for it in items.iter_mut() {
            if !it.status.is_terminal() {
                it.status = BatchItemStatusKind::Cancelled;
                it.updated_at = now;
                it.completed_at = Some(now);
            }
        }

        batch.status = BatchJobStatus::Cancelled;
        batch.updated_at = now;
        batch.completed_at = Some(now);
        batch.queued_items = 0;
        batch.processing_items = 0;

        Ok(Some(batch.clone()))
    }

    async fn list_batches(
        &self,
        owner_session_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BatchRecord>, CoreError> {
        let map = self.storage.read().await;
        let mut list: Vec<BatchRecord> = map
            .values()
            .map(|(b, _)| b.clone())
            .filter(|b| match (owner_session_id, b.owner_session_id) {
                (Some(expected), Some(actual)) => expected == actual,
                (Some(_), None) => false,
                (None, _) => true,
            })
            .collect();

        list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        let paged = list.into_iter().skip(offset).take(limit).collect();
        Ok(paged)
    }
}

/// PostgreSQL transactional implementation of [`BatchRepository`].
#[derive(Debug, Clone)]
pub struct PostgresBatchRepository {
    pool: sqlx::PgPool,
}

impl PostgresBatchRepository {
    /// Creates a new `PostgresBatchRepository` backed by the given connection pool.
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BatchRepository for PostgresBatchRepository {
    async fn create_batch(
        &self,
        batch: BatchRecord,
        items: Vec<BatchItemRecord>,
    ) -> Result<BatchRecord, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO batch_jobs (
                id, name, status, total_items, queued_items, processing_items,
                completed_items, failed_items, max_parallelism, owner_session_id,
                created_at, updated_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(batch.id.into_inner())
        .bind(&batch.name)
        .bind(batch.status.as_str())
        .bind(batch.total_items as i32)
        .bind(batch.queued_items as i32)
        .bind(batch.processing_items as i32)
        .bind(batch.completed_items as i32)
        .bind(batch.failed_items as i32)
        .bind(batch.max_parallelism as i32)
        .bind(batch.owner_session_id)
        .bind(batch.created_at)
        .bind(batch.updated_at)
        .bind(batch.completed_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        for item in &items {
            sqlx::query(
                r#"
                INSERT INTO batch_items (
                    batch_id, item_index, job_id, url, custom_id, status,
                    error_code, error_message, duration_ms, created_at, updated_at, completed_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(item.batch_id.into_inner())
            .bind(item.item_index as i32)
            .bind(item.job_id.into_inner())
            .bind(&item.url)
            .bind(&item.custom_id)
            .bind(item.status.as_str())
            .bind(&item.error_code)
            .bind(&item.error_message)
            .bind(item.duration_ms.map(|d| d as i64))
            .bind(item.created_at)
            .bind(item.updated_at)
            .bind(item.completed_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(batch)
    }

    async fn get_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query(
            r#"
            SELECT id, name, status, total_items, queued_items, processing_items,
                   completed_items, failed_items, max_parallelism, owner_session_id,
                   created_at, updated_at, completed_at
            FROM batch_jobs
            WHERE id = $1
            "#,
        )
        .bind(batch_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let status_str: String = r.get("status");
            let status = BatchJobStatus::parse(&status_str)?;

            Ok(Some(BatchRecord {
                id: BatchId::from(r.get::<Uuid, _>("id")),
                name: r.get("name"),
                status,
                total_items: r.get::<i32, _>("total_items") as usize,
                queued_items: r.get::<i32, _>("queued_items") as usize,
                processing_items: r.get::<i32, _>("processing_items") as usize,
                completed_items: r.get::<i32, _>("completed_items") as usize,
                failed_items: r.get::<i32, _>("failed_items") as usize,
                max_parallelism: r.get::<i32, _>("max_parallelism") as usize,
                owner_session_id: r.get("owner_session_id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_batch_items(&self, batch_id: BatchId) -> Result<Vec<BatchItemRecord>, CoreError> {
        let rows = sqlx::query(
            r#"
            SELECT batch_id, item_index, job_id, url, custom_id, status,
                   error_code, error_message, duration_ms, created_at, updated_at, completed_at
            FROM batch_items
            WHERE batch_id = $1
            ORDER BY item_index ASC
            "#,
        )
        .bind(batch_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        use sqlx::Row;
        let mut results = Vec::with_capacity(rows.len());
        for r in rows {
            let status_str: String = r.get("status");
            let status = BatchItemStatusKind::parse(&status_str)?;
            let duration_ms: Option<i64> = r.get("duration_ms");

            results.push(BatchItemRecord {
                batch_id: BatchId::from(r.get::<Uuid, _>("batch_id")),
                item_index: r.get::<i32, _>("item_index") as usize,
                job_id: JobId::from(r.get::<Uuid, _>("job_id")),
                url: r.get("url"),
                custom_id: r.get("custom_id"),
                status,
                error_code: r.get("error_code"),
                error_message: r.get("error_message"),
                duration_ms: duration_ms.map(|d| d as u64),
                artifact_count: 0,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            });
        }

        Ok(results)
    }

    async fn get_batch_item(
        &self,
        batch_id: BatchId,
        item_index: usize,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let row = sqlx::query(
            r#"
            SELECT batch_id, item_index, job_id, url, custom_id, status,
                   error_code, error_message, duration_ms, created_at, updated_at, completed_at
            FROM batch_items
            WHERE batch_id = $1 AND item_index = $2
            "#,
        )
        .bind(batch_id.into_inner())
        .bind(item_index as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let status_str: String = r.get("status");
            let status = BatchItemStatusKind::parse(&status_str)?;
            let duration_ms: Option<i64> = r.get("duration_ms");

            Ok(Some(BatchItemRecord {
                batch_id: BatchId::from(r.get::<Uuid, _>("batch_id")),
                item_index: r.get::<i32, _>("item_index") as usize,
                job_id: JobId::from(r.get::<Uuid, _>("job_id")),
                url: r.get("url"),
                custom_id: r.get("custom_id"),
                status,
                error_code: r.get("error_code"),
                error_message: r.get("error_message"),
                duration_ms: duration_ms.map(|d| d as u64),
                artifact_count: 0,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_batch_item_by_job_id(
        &self,
        job_id: JobId,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let row = sqlx::query(
            r#"
            SELECT batch_id, item_index, job_id, url, custom_id, status,
                   error_code, error_message, duration_ms, created_at, updated_at, completed_at
            FROM batch_items
            WHERE job_id = $1
            "#,
        )
        .bind(job_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let status_str: String = r.get("status");
            let status = BatchItemStatusKind::parse(&status_str)?;
            let duration_ms: Option<i64> = r.get("duration_ms");

            Ok(Some(BatchItemRecord {
                batch_id: BatchId::from(r.get::<Uuid, _>("batch_id")),
                item_index: r.get::<i32, _>("item_index") as usize,
                job_id: JobId::from(r.get::<Uuid, _>("job_id")),
                url: r.get("url"),
                custom_id: r.get("custom_id"),
                status,
                error_code: r.get("error_code"),
                error_message: r.get("error_message"),
                duration_ms: duration_ms.map(|d| d as u64),
                artifact_count: 0,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_item_status(
        &self,
        batch_id: BatchId,
        item_index: usize,
        status: BatchItemStatusKind,
        error_code: Option<String>,
        error_message: Option<String>,
        duration_ms: Option<u64>,
    ) -> Result<Option<BatchItemRecord>, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let now = Utc::now();
        let completed_at = if status.is_terminal() {
            Some(now)
        } else {
            None
        };

        let update_res = sqlx::query(
            r#"
            UPDATE batch_items
            SET status = $3,
                error_code = COALESCE($4, error_code),
                error_message = COALESCE($5, error_message),
                duration_ms = COALESCE($6, duration_ms),
                completed_at = COALESCE($7, completed_at),
                updated_at = $8
            WHERE batch_id = $1 AND item_index = $2
            RETURNING batch_id, item_index, job_id, url, custom_id, status,
                      error_code, error_message, duration_ms, created_at, updated_at, completed_at
            "#,
        )
        .bind(batch_id.into_inner())
        .bind(item_index as i32)
        .bind(status.as_str())
        .bind(&error_code)
        .bind(&error_message)
        .bind(duration_ms.map(|d| d as i64))
        .bind(completed_at)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        let updated_item = if let Some(r) = update_res {
            use sqlx::Row;
            let status_str: String = r.get("status");
            let it_status = BatchItemStatusKind::parse(&status_str)?;
            let dur: Option<i64> = r.get("duration_ms");

            BatchItemRecord {
                batch_id: BatchId::from(r.get::<Uuid, _>("batch_id")),
                item_index: r.get::<i32, _>("item_index") as usize,
                job_id: JobId::from(r.get::<Uuid, _>("job_id")),
                url: r.get("url"),
                custom_id: r.get("custom_id"),
                status: it_status,
                error_code: r.get("error_code"),
                error_message: r.get("error_message"),
                duration_ms: dur.map(|d| d as u64),
                artifact_count: 0,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }
        } else {
            return Ok(None);
        };

        // Recalculate parent batch counters
        sqlx::query(
            r#"
            WITH counts AS (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'queued') AS queued,
                    COUNT(*) FILTER (WHERE status = 'processing') AS processing,
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status IN ('failed', 'timed_out', 'cancelled')) AS failed,
                    COUNT(*) AS total
                FROM batch_items
                WHERE batch_id = $1
            )
            UPDATE batch_jobs b
            SET queued_items = c.queued,
                processing_items = c.processing,
                completed_items = c.completed,
                failed_items = c.failed,
                status = CASE
                    WHEN c.completed + c.failed = c.total AND c.failed = 0 THEN 'completed'
                    WHEN c.completed + c.failed = c.total AND c.completed > 0 THEN 'partial_failure'
                    WHEN c.completed + c.failed = c.total AND c.completed = 0 THEN 'failed'
                    WHEN c.processing > 0 THEN 'processing'
                    ELSE b.status
                END,
                completed_at = CASE
                    WHEN c.completed + c.failed = c.total THEN $2
                    ELSE b.completed_at
                END,
                updated_at = $2
            FROM counts c
            WHERE b.id = $1
            "#,
        )
        .bind(batch_id.into_inner())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Some(updated_item))
    }

    async fn cancel_batch(&self, batch_id: BatchId) -> Result<Option<BatchRecord>, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE batch_items
            SET status = 'cancelled',
                completed_at = COALESCE(completed_at, $2),
                updated_at = $2
            WHERE batch_id = $1 AND status NOT IN ('completed', 'failed', 'timed_out', 'cancelled')
            "#,
        )
        .bind(batch_id.into_inner())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        let res = sqlx::query(
            r#"
            UPDATE batch_jobs
            SET status = 'cancelled',
                queued_items = 0,
                processing_items = 0,
                completed_at = COALESCE(completed_at, $2),
                updated_at = $2
            WHERE id = $1
            RETURNING id, name, status, total_items, queued_items, processing_items,
                      completed_items, failed_items, max_parallelism, owner_session_id,
                      created_at, updated_at, completed_at
            "#,
        )
        .bind(batch_id.into_inner())
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        if let Some(r) = res {
            use sqlx::Row;
            let status_str: String = r.get("status");
            let status = BatchJobStatus::parse(&status_str)?;

            Ok(Some(BatchRecord {
                id: BatchId::from(r.get::<Uuid, _>("id")),
                name: r.get("name"),
                status,
                total_items: r.get::<i32, _>("total_items") as usize,
                queued_items: r.get::<i32, _>("queued_items") as usize,
                processing_items: r.get::<i32, _>("processing_items") as usize,
                completed_items: r.get::<i32, _>("completed_items") as usize,
                failed_items: r.get::<i32, _>("failed_items") as usize,
                max_parallelism: r.get::<i32, _>("max_parallelism") as usize,
                owner_session_id: r.get("owner_session_id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_batches(
        &self,
        owner_session_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<BatchRecord>, CoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, status, total_items, queued_items, processing_items,
                   completed_items, failed_items, max_parallelism, owner_session_id,
                   created_at, updated_at, completed_at
            FROM batch_jobs
            WHERE ($1::uuid IS NULL OR owner_session_id = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(owner_session_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;

        use sqlx::Row;
        let mut results = Vec::with_capacity(rows.len());
        for r in rows {
            let status_str: String = r.get("status");
            let status = BatchJobStatus::parse(&status_str)?;

            results.push(BatchRecord {
                id: BatchId::from(r.get::<Uuid, _>("id")),
                name: r.get("name"),
                status,
                total_items: r.get::<i32, _>("total_items") as usize,
                queued_items: r.get::<i32, _>("queued_items") as usize,
                processing_items: r.get::<i32, _>("processing_items") as usize,
                completed_items: r.get::<i32, _>("completed_items") as usize,
                failed_items: r.get::<i32, _>("failed_items") as usize,
                max_parallelism: r.get::<i32, _>("max_parallelism") as usize,
                owner_session_id: r.get("owner_session_id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                completed_at: r.get("completed_at"),
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_batch_repository_crud_lifecycle() {
        let repo = InMemoryBatchRepository::new();
        let batch_id = BatchId::new();
        let now = Utc::now();

        let batch = BatchRecord {
            id: batch_id,
            name: Some("test-batch".to_string()),
            status: BatchJobStatus::Queued,
            total_items: 2,
            queued_items: 2,
            processing_items: 0,
            completed_items: 0,
            failed_items: 0,
            max_parallelism: 5,
            owner_session_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        let items = vec![
            BatchItemRecord {
                batch_id,
                item_index: 0,
                job_id: JobId::new(),
                url: "https://example.com/1".to_string(),
                custom_id: Some("item-1".to_string()),
                status: BatchItemStatusKind::Queued,
                error_code: None,
                error_message: None,
                duration_ms: None,
                artifact_count: 0,
                created_at: now,
                updated_at: now,
                completed_at: None,
            },
            BatchItemRecord {
                batch_id,
                item_index: 1,
                job_id: JobId::new(),
                url: "https://example.com/2".to_string(),
                custom_id: Some("item-2".to_string()),
                status: BatchItemStatusKind::Queued,
                error_code: None,
                error_message: None,
                duration_ms: None,
                artifact_count: 0,
                created_at: now,
                updated_at: now,
                completed_at: None,
            },
        ];

        // 1. Create batch
        let created = repo.create_batch(batch, items).await.unwrap();
        assert_eq!(created.id, batch_id);

        // 2. Query batch & items
        let fetched_batch = repo.get_batch(batch_id).await.unwrap().unwrap();
        assert_eq!(fetched_batch.status, BatchJobStatus::Queued);
        assert_eq!(fetched_batch.total_items, 2);

        let fetched_items = repo.get_batch_items(batch_id).await.unwrap();
        assert_eq!(fetched_items.len(), 2);
        assert_eq!(fetched_items[0].url, "https://example.com/1");

        // 3. Update item 0 -> completed
        let updated_item0 = repo
            .update_item_status(
                batch_id,
                0,
                BatchItemStatusKind::Completed,
                None,
                None,
                Some(120),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_item0.status, BatchItemStatusKind::Completed);
        assert_eq!(updated_item0.duration_ms, Some(120));

        let batch_after_first = repo.get_batch(batch_id).await.unwrap().unwrap();
        assert_eq!(batch_after_first.completed_items, 1);
        assert_eq!(batch_after_first.status, BatchJobStatus::Queued); // still 1 queued

        // 4. Update item 1 -> failed
        let updated_item1 = repo
            .update_item_status(
                batch_id,
                1,
                BatchItemStatusKind::Failed,
                Some("EXTRACTION_ERROR".to_string()),
                Some("timeout".to_string()),
                Some(500),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_item1.status, BatchItemStatusKind::Failed);

        // Both items are terminal, 1 completed, 1 failed -> PartialFailure
        let batch_final = repo.get_batch(batch_id).await.unwrap().unwrap();
        assert_eq!(batch_final.status, BatchJobStatus::PartialFailure);
        assert_eq!(batch_final.completed_items, 1);
        assert_eq!(batch_final.failed_items, 1);
        assert!(batch_final.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_in_memory_batch_cancellation() {
        let repo = InMemoryBatchRepository::new();
        let batch_id = BatchId::new();
        let now = Utc::now();

        let batch = BatchRecord {
            id: batch_id,
            name: None,
            status: BatchJobStatus::Queued,
            total_items: 2,
            queued_items: 2,
            processing_items: 0,
            completed_items: 0,
            failed_items: 0,
            max_parallelism: 5,
            owner_session_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        let items = vec![
            BatchItemRecord {
                batch_id,
                item_index: 0,
                job_id: JobId::new(),
                url: "https://example.com/1".to_string(),
                custom_id: None,
                status: BatchItemStatusKind::Queued,
                error_code: None,
                error_message: None,
                duration_ms: None,
                artifact_count: 0,
                created_at: now,
                updated_at: now,
                completed_at: None,
            },
            BatchItemRecord {
                batch_id,
                item_index: 1,
                job_id: JobId::new(),
                url: "https://example.com/2".to_string(),
                custom_id: None,
                status: BatchItemStatusKind::Queued,
                error_code: None,
                error_message: None,
                duration_ms: None,
                artifact_count: 0,
                created_at: now,
                updated_at: now,
                completed_at: None,
            },
        ];

        repo.create_batch(batch, items).await.unwrap();

        let cancelled = repo.cancel_batch(batch_id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, BatchJobStatus::Cancelled);

        let items_after = repo.get_batch_items(batch_id).await.unwrap();
        assert_eq!(items_after[0].status, BatchItemStatusKind::Cancelled);
        assert_eq!(items_after[1].status, BatchItemStatusKind::Cancelled);
    }
}
