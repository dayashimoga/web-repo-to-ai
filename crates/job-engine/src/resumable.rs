//! # Resumable Processing & Durable Checkpointing Subsystem
//!
//! Provides durable checkpointing, interrupted task recovery, crash resilience,
//! and duplicate execution prevention for large repository conversions, batch jobs,
//! and website crawling pipelines:
//! - [`JobCheckpoint`]: serializable state checkpoint with BLAKE3 cryptographic integrity.
//! - [`CheckpointItemResult`]: serialized individual item outcome.
//! - [`CheckpointStore`]: pluggable asynchronous persistence trait.
//! - [`InMemoryCheckpointStore`]: thread-safe in-memory store for testing and daemons.
//! - [`ResumableConfig`]: checkpointing frequency, bounds, and retention policies.
//! - [`ResumableExecutionPlan`]: partition of already-completed vs pending items.
//! - [`ResumableParallelExecutor`]: bounded execution coordinator with automatic checkpointing.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::info;
use uuid::Uuid;
use web_repo_to_ai_core::id::{BatchId, JobId};

use crate::config::BatchExecutionConfig;
use crate::error::JobEngineError;
use crate::models::{
    BatchExecutionSummary, BatchItemExecutionResult, BatchItemStatusOutcome, BatchItemTask,
    BatchProgressUpdate,
};

/// Default frequency (in items) at which checkpoints are persisted.
pub const DEFAULT_CHECKPOINT_INTERVAL_ITEMS: usize = 5;

/// Default maximum allowable resumption attempts before a job is declared unrecoverable.
pub const DEFAULT_MAX_RESUMPTION_ATTEMPTS: usize = 3;

/// Granular execution outcome for an individual item stored in a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointItemResult {
    /// Zero-based sequence index.
    pub item_index: usize,
    /// Unique sub-job identifier.
    pub job_id: JobId,
    /// Target resource URL.
    pub url: String,
    /// Optional client correlation ID.
    pub custom_id: Option<String>,
    /// High-level execution status.
    pub status: BatchItemStatusOutcome,
    /// Serialized JSON string of the successful result value, if any.
    pub value_json: Option<String>,
    /// Error message on failure or timeout.
    pub error: Option<String>,
    /// Duration in milliseconds taken by the item execution.
    pub duration_ms: u64,
}

/// Durable, cryptographically verified checkpoint capturing the in-flight state of a job or batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobCheckpoint {
    /// Associated job or batch identifier.
    pub job_id: String,
    /// Unique identifier for this checkpoint revision.
    pub checkpoint_id: String,
    /// Monotonically increasing sequence number for ordering checkpoints.
    pub sequence_number: u64,
    /// Total number of items submitted in the job.
    pub total_items: usize,
    /// Set of item indices that completed successfully.
    pub completed_indices: BTreeSet<usize>,
    /// Set of item indices that encountered terminal failures or timeouts.
    pub failed_indices: BTreeSet<usize>,
    /// Detailed individual item results mapped by sequence index.
    pub item_results: BTreeMap<usize, CheckpointItemResult>,
    /// Optional cursor or frontier offset (e.g. for website crawling).
    pub cursor: Option<String>,
    /// Additional context metadata.
    pub metadata: BTreeMap<String, String>,
    /// Timestamp when this checkpoint was first created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the most recent checkpoint update.
    pub updated_at: DateTime<Utc>,
    /// Cryptographic BLAKE3 hex digest of the canonical state payload.
    pub checksum: String,
}

impl JobCheckpoint {
    /// Creates a new empty checkpoint for a job.
    #[must_use]
    pub fn new(job_id: impl Into<String>, total_items: usize) -> Self {
        let now = Utc::now();
        let mut checkpoint = Self {
            job_id: job_id.into(),
            checkpoint_id: Uuid::new_v4().to_string(),
            sequence_number: 0,
            total_items,
            completed_indices: BTreeSet::new(),
            failed_indices: BTreeSet::new(),
            item_results: BTreeMap::new(),
            cursor: None,
            metadata: BTreeMap::new(),
            created_at: now,
            updated_at: now,
            checksum: String::new(),
        };
        checkpoint.update_checksum();
        checkpoint
    }

    /// Computes the canonical BLAKE3 hexadecimal digest over the checkpoint's content fields.
    #[must_use]
    pub fn compute_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.job_id.as_bytes());
        hasher.update(&self.sequence_number.to_le_bytes());
        hasher.update(&(self.total_items as u64).to_le_bytes());

        for idx in &self.completed_indices {
            hasher.update(b"c:");
            hasher.update(&(*idx as u64).to_le_bytes());
        }
        for idx in &self.failed_indices {
            hasher.update(b"f:");
            hasher.update(&(*idx as u64).to_le_bytes());
        }
        for (idx, res) in &self.item_results {
            hasher.update(&(*idx as u64).to_le_bytes());
            hasher.update(res.url.as_bytes());
            if let Some(err) = &res.error {
                hasher.update(err.as_bytes());
            }
            if let Some(val) = &res.value_json {
                hasher.update(val.as_bytes());
            }
        }
        if let Some(c) = &self.cursor {
            hasher.update(b"cursor:");
            hasher.update(c.as_bytes());
        }
        for (k, v) in &self.metadata {
            hasher.update(k.as_bytes());
            hasher.update(b"=");
            hasher.update(v.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Recalculates and updates the internal checksum field.
    pub fn update_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    /// Verifies that the internal checksum matches the recomputed state digest.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::CheckpointCorrupted`] if the checksum does not match.
    pub fn verify_checksum(&self) -> Result<(), JobEngineError> {
        let expected = self.compute_checksum();
        if self.checksum != expected {
            return Err(JobEngineError::CheckpointCorrupted {
                job_id: self.job_id.clone(),
                reason: format!(
                    "Checksum mismatch: stored '{}', recomputed '{}'",
                    self.checksum, expected
                ),
            });
        }
        Ok(())
    }

    /// Records an item execution result into the checkpoint, incrementing the sequence number.
    pub fn record_item(&mut self, result: CheckpointItemResult) {
        let idx = result.item_index;
        if result.status.is_success() {
            self.completed_indices.insert(idx);
            self.failed_indices.remove(&idx);
        } else {
            self.failed_indices.insert(idx);
            self.completed_indices.remove(&idx);
        }
        self.item_results.insert(idx, result);
        self.sequence_number += 1;
        self.updated_at = Utc::now();
        self.update_checksum();
    }

    /// Returns `true` if all submitted items have reached a terminal outcome (completed or failed).
    #[must_use]
    pub fn is_fully_finished(&self) -> bool {
        self.total_items > 0
            && (self.completed_indices.len() + self.failed_indices.len()) >= self.total_items
    }

    /// Returns the count of remaining unfinished items.
    #[must_use]
    pub fn remaining_count(&self) -> usize {
        let finished = self.completed_indices.len() + self.failed_indices.len();
        self.total_items.saturating_sub(finished)
    }
}

/// Configuration governing checkpoint generation, frequency, and resumption policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumableConfig {
    /// Save a checkpoint every N items completed.
    pub checkpoint_interval_items: usize,
    /// Maximum permitted resumption attempts.
    pub max_resumption_attempts: usize,
    /// Whether to delete the checkpoint upon 100% completion.
    pub delete_checkpoint_on_complete: bool,
    /// Whether to strictly verify the cryptographic BLAKE3 checksum upon loading.
    pub verify_checksum_on_load: bool,
}

impl Default for ResumableConfig {
    fn default() -> Self {
        Self {
            checkpoint_interval_items: DEFAULT_CHECKPOINT_INTERVAL_ITEMS,
            max_resumption_attempts: DEFAULT_MAX_RESUMPTION_ATTEMPTS,
            delete_checkpoint_on_complete: false,
            verify_checksum_on_load: true,
        }
    }
}

impl ResumableConfig {
    /// Creates a new validated `ResumableConfig`.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if parameters are invalid.
    pub fn new(checkpoint_interval_items: usize) -> Result<Self, JobEngineError> {
        let config = Self {
            checkpoint_interval_items,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Configures whether to delete checkpoints upon 100% job completion.
    #[must_use]
    pub fn with_cleanup(mut self, delete_on_complete: bool) -> Self {
        self.delete_checkpoint_on_complete = delete_on_complete;
        self
    }

    /// Configures checksum verification.
    #[must_use]
    pub fn with_checksum_verification(mut self, verify: bool) -> Self {
        self.verify_checksum_on_load = verify;
        self
    }

    /// Validates configuration parameters.
    pub fn validate(&self) -> Result<(), JobEngineError> {
        if self.checkpoint_interval_items == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "checkpoint_interval_items must be at least 1".to_string(),
            });
        }
        if self.max_resumption_attempts == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "max_resumption_attempts must be at least 1".to_string(),
            });
        }
        Ok(())
    }
}

/// Asynchronous persistence trait for storing and retrieving job checkpoints.
#[async_trait]
pub trait CheckpointStore: Send + Sync + std::fmt::Debug {
    /// Persists a checkpoint to durable storage.
    async fn save_checkpoint(&self, checkpoint: &JobCheckpoint) -> Result<(), JobEngineError>;

    /// Loads the most recent checkpoint for the specified job, if one exists.
    async fn load_checkpoint(&self, job_id: &str) -> Result<Option<JobCheckpoint>, JobEngineError>;

    /// Permanently removes the checkpoint for a job.
    async fn delete_checkpoint(&self, job_id: &str) -> Result<(), JobEngineError>;
}

/// Thread-safe in-memory implementation of [`CheckpointStore`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryCheckpointStore {
    storage: Arc<RwLock<HashMap<String, JobCheckpoint>>>,
}

impl InMemoryCheckpointStore {
    /// Creates a new empty in-memory checkpoint store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of checkpoints currently stored.
    pub async fn len(&self) -> usize {
        self.storage.read().await.len()
    }

    /// Returns `true` if the store contains no checkpoints.
    pub async fn is_empty(&self) -> bool {
        self.storage.read().await.is_empty()
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn save_checkpoint(&self, checkpoint: &JobCheckpoint) -> Result<(), JobEngineError> {
        let mut map = self.storage.write().await;
        map.insert(checkpoint.job_id.clone(), checkpoint.clone());
        Ok(())
    }

    async fn load_checkpoint(&self, job_id: &str) -> Result<Option<JobCheckpoint>, JobEngineError> {
        let map = self.storage.read().await;
        Ok(map.get(job_id).cloned())
    }

    async fn delete_checkpoint(&self, job_id: &str) -> Result<(), JobEngineError> {
        let mut map = self.storage.write().await;
        map.remove(job_id);
        Ok(())
    }
}

/// Structural plan partitioning a batch into previously completed items vs pending items.
#[derive(Debug, Clone)]
pub struct ResumableExecutionPlan<T> {
    /// Total item count originally submitted.
    pub total_items: usize,
    /// Items that were already completed in the checkpoint and do not need re-execution.
    pub already_completed_indices: Vec<usize>,
    /// Tasks that must be executed in this run.
    pub pending_tasks: Vec<BatchItemTask<T>>,
    /// `true` if this run is resuming from an existing checkpoint.
    pub is_resumed: bool,
    /// Starting sequence number from the loaded checkpoint (or 0 if fresh).
    pub starting_sequence: u64,
}

impl<T> ResumableExecutionPlan<T> {
    /// Constructs an execution plan by comparing submitted tasks against a checkpoint.
    pub fn create(tasks: Vec<BatchItemTask<T>>, checkpoint: Option<&JobCheckpoint>) -> Self {
        let total_items = tasks.len();
        if let Some(cp) = checkpoint {
            let mut pending = Vec::new();
            let mut completed = Vec::new();

            for task in tasks {
                if cp.completed_indices.contains(&task.item_index) {
                    completed.push(task.item_index);
                } else {
                    pending.push(task);
                }
            }

            Self {
                total_items,
                already_completed_indices: completed,
                pending_tasks: pending,
                is_resumed: true,
                starting_sequence: cp.sequence_number,
            }
        } else {
            Self {
                total_items,
                already_completed_indices: Vec::new(),
                pending_tasks: tasks,
                is_resumed: false,
                starting_sequence: 0,
            }
        }
    }
}

/// Statistical report capturing resumption execution dynamics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResumableStats {
    /// Whether this execution was resumed from a previous checkpoint.
    pub is_resumed: bool,
    /// Number of items reused directly from the checkpoint without reprocessing.
    pub items_reused: usize,
    /// Number of pending items executed in this run.
    pub items_executed: usize,
    /// Total checkpoints saved during this run.
    pub checkpoints_saved: usize,
}

/// High-performance resumable parallel executor.
#[derive(Debug, Clone)]
pub struct ResumableParallelExecutor {
    batch_config: BatchExecutionConfig,
    resumable_config: ResumableConfig,
    checkpoint_store: Arc<dyn CheckpointStore>,
}

impl ResumableParallelExecutor {
    /// Initializes a new `ResumableParallelExecutor`.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if configurations are invalid.
    pub fn new(
        batch_config: BatchExecutionConfig,
        resumable_config: ResumableConfig,
        checkpoint_store: Arc<dyn CheckpointStore>,
    ) -> Result<Self, JobEngineError> {
        batch_config.validate()?;
        resumable_config.validate()?;
        Ok(Self {
            batch_config,
            resumable_config,
            checkpoint_store,
        })
    }

    /// Returns a reference to the active batch configuration.
    #[must_use]
    pub fn batch_config(&self) -> &BatchExecutionConfig {
        &self.batch_config
    }

    /// Returns a reference to the active resumable configuration.
    #[must_use]
    pub fn resumable_config(&self) -> &ResumableConfig {
        &self.resumable_config
    }

    /// Executes a collection of batch item tasks with durable checkpointing and resume support.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::EmptyBatch`] if task list is empty.
    /// Returns [`JobEngineError`] if checkpoint loading, verification, or execution fails.
    pub async fn execute_resumable<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        processor: F,
        progress_tx: Option<mpsc::Sender<BatchProgressUpdate>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<(BatchExecutionSummary<R>, ResumableStats), JobEngineError>
    where
        T: Clone + Send + Sync + 'static,
        R: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
        F: Fn(BatchItemTask<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<R, String>> + Send + 'static,
    {
        if tasks.is_empty() {
            return Err(JobEngineError::EmptyBatch);
        }

        let cancel_token = cancellation_token.unwrap_or_default();
        if cancel_token.is_cancelled() {
            return Err(JobEngineError::WorkerShuttingDown);
        }

        let job_id_str = batch_id.to_string();
        let start_time = Instant::now();

        // 1. Load existing checkpoint if available
        let mut checkpoint = match self.checkpoint_store.load_checkpoint(&job_id_str).await? {
            Some(cp) => {
                if self.resumable_config.verify_checksum_on_load {
                    cp.verify_checksum()?;
                }
                info!(
                    job_id = %job_id_str,
                    sequence = cp.sequence_number,
                    completed = cp.completed_indices.len(),
                    total = cp.total_items,
                    "Loaded existing valid checkpoint for resumption"
                );
                cp
            }
            None => {
                let cp = JobCheckpoint::new(&job_id_str, tasks.len());
                self.checkpoint_store.save_checkpoint(&cp).await?;
                cp
            }
        };

        // 2. Formulate execution plan
        let plan = ResumableExecutionPlan::create(tasks, Some(&checkpoint));
        let mut stats = ResumableStats {
            is_resumed: plan.is_resumed && !plan.already_completed_indices.is_empty(),
            items_reused: plan.already_completed_indices.len(),
            items_executed: 0,
            checkpoints_saved: 0,
        };

        info!(
            job_id = %job_id_str,
            is_resumed = stats.is_resumed,
            items_reused = stats.items_reused,
            pending = plan.pending_tasks.len(),
            "Formulated resumable execution plan"
        );

        // 3. Reconstruct results for already completed items
        let mut combined_results: BTreeMap<usize, BatchItemExecutionResult<R>> = BTreeMap::new();
        for idx in &plan.already_completed_indices {
            if let Some(cp_item) = checkpoint.item_results.get(idx) {
                let val: Option<R> = cp_item
                    .value_json
                    .as_deref()
                    .and_then(|json| serde_json::from_str(json).ok());

                combined_results.insert(
                    *idx,
                    BatchItemExecutionResult {
                        item_index: cp_item.item_index,
                        job_id: cp_item.job_id,
                        url: cp_item.url.clone(),
                        custom_id: cp_item.custom_id.clone(),
                        duration_ms: cp_item.duration_ms,
                        status: cp_item.status.clone(),
                        value: val,
                        error: cp_item.error.clone(),
                    },
                );
            }
        }

        // 4. If all items already completed, return immediately
        if plan.pending_tasks.is_empty() {
            let sorted_results: Vec<_> = combined_results.into_values().collect();
            let summary = BatchExecutionSummary {
                batch_id,
                total_items: plan.total_items,
                succeeded_count: sorted_results.iter().filter(|r| r.is_success()).count(),
                failed_count: sorted_results
                    .iter()
                    .filter(|r| matches!(r.status, BatchItemStatusOutcome::Failed { .. }))
                    .count(),
                cancelled_count: 0,
                timed_out_count: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                peak_concurrency: 0,
                configured_parallelism: self.batch_config.max_parallelism,
                item_results: sorted_results,
            };
            return Ok((summary, stats));
        }

        // 5. Execute pending items with bounded concurrency and incremental checkpointing
        let semaphore = Arc::new(Semaphore::new(self.batch_config.max_parallelism));
        let active_workers = Arc::new(AtomicUsize::new(0));
        let peak_concurrency = Arc::new(AtomicUsize::new(0));
        let (item_tx, mut item_rx) = mpsc::channel(self.batch_config.max_parallelism * 2);

        let pending_count = plan.pending_tasks.len();
        let total_items = plan.total_items;

        // Spawn dispatcher
        let spawn_tasks = plan.pending_tasks;
        let spawn_sem = Arc::clone(&semaphore);
        let spawn_active = Arc::clone(&active_workers);
        let spawn_peak = Arc::clone(&peak_concurrency);
        let spawn_cancel = cancel_token.clone();
        let item_timeout = self.batch_config.item_timeout;
        let max_retries = self.batch_config.max_retries;
        let retry_backoff = self.batch_config.retry_backoff;

        tokio::spawn(async move {
            for task in spawn_tasks {
                if spawn_cancel.is_cancelled() {
                    let _ = item_tx
                        .send(BatchItemExecutionResult::<R>::cancelled(
                            task.item_index,
                            task.job_id,
                            task.url,
                            task.custom_id,
                            "Operation cancelled",
                        ))
                        .await;
                    continue;
                }

                let permit = match spawn_sem.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
                };

                let cur = spawn_active.fetch_add(1, Ordering::SeqCst) + 1;
                let mut peak = spawn_peak.load(Ordering::Relaxed);
                while cur > peak {
                    match spawn_peak.compare_exchange_weak(
                        peak,
                        cur,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(actual) => peak = actual,
                    }
                }

                let task_tx = item_tx.clone();
                let task_proc = processor.clone();
                let task_active = Arc::clone(&spawn_active);
                let task_cancel = spawn_cancel.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let item_start = Instant::now();

                    let mut attempt: u32 = 0;
                    let mut final_res = None;

                    while attempt <= max_retries {
                        if task_cancel.is_cancelled() {
                            final_res = Some(BatchItemExecutionResult::cancelled(
                                task.item_index,
                                task.job_id,
                                task.url.clone(),
                                task.custom_id.clone(),
                                "Cancelled before execution",
                            ));
                            break;
                        }

                        match timeout(item_timeout, task_proc(task.clone())).await {
                            Ok(Ok(val)) => {
                                final_res = Some(BatchItemExecutionResult::success(
                                    task.item_index,
                                    task.job_id,
                                    task.url.clone(),
                                    task.custom_id.clone(),
                                    item_start.elapsed().as_millis() as u64,
                                    val,
                                ));
                                break;
                            }
                            Ok(Err(err)) => {
                                if attempt < max_retries && !task_cancel.is_cancelled() {
                                    attempt += 1;
                                    let backoff = retry_backoff * attempt;
                                    tokio::time::sleep(backoff).await;
                                    continue;
                                }
                                final_res = Some(BatchItemExecutionResult::failure(
                                    task.item_index,
                                    task.job_id,
                                    task.url.clone(),
                                    task.custom_id.clone(),
                                    item_start.elapsed().as_millis() as u64,
                                    err,
                                ));
                                break;
                            }
                            Err(_) => {
                                if attempt < max_retries && !task_cancel.is_cancelled() {
                                    attempt += 1;
                                    let backoff = retry_backoff * attempt;
                                    tokio::time::sleep(backoff).await;
                                    continue;
                                }
                                final_res = Some(BatchItemExecutionResult::timed_out(
                                    task.item_index,
                                    task.job_id,
                                    task.url.clone(),
                                    task.custom_id.clone(),
                                    item_start.elapsed().as_millis() as u64,
                                ));
                                break;
                            }
                        }
                    }

                    let res = final_res.unwrap_or_else(|| {
                        BatchItemExecutionResult::failure(
                            task.item_index,
                            task.job_id,
                            task.url,
                            task.custom_id,
                            item_start.elapsed().as_millis() as u64,
                            "Execution exhausted retries without outcome",
                        )
                    });

                    task_active.fetch_sub(1, Ordering::SeqCst);
                    let _ = task_tx.send(res).await;
                });
            }
        });

        // 6. Collector loop updating checkpoint periodically
        let mut items_since_last_checkpoint = 0;

        while let Some(res) = item_rx.recv().await {
            stats.items_executed += 1;

            // Serialize value if present
            let value_json = res
                .value
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok());

            let cp_item = CheckpointItemResult {
                item_index: res.item_index,
                job_id: res.job_id,
                url: res.url.clone(),
                custom_id: res.custom_id.clone(),
                status: res.status.clone(),
                value_json,
                error: res.error.clone(),
                duration_ms: res.duration_ms,
            };

            checkpoint.record_item(cp_item);
            combined_results.insert(res.item_index, res);

            items_since_last_checkpoint += 1;
            if items_since_last_checkpoint >= self.resumable_config.checkpoint_interval_items {
                self.checkpoint_store.save_checkpoint(&checkpoint).await?;
                stats.checkpoints_saved += 1;
                items_since_last_checkpoint = 0;
            }

            // Emit progress update
            if let Some(ref tx) = progress_tx {
                let _ = tx
                    .send(BatchProgressUpdate {
                        batch_id,
                        total_items,
                        completed_items: checkpoint.completed_indices.len(),
                        failed_items: checkpoint.failed_indices.len(),
                        active_workers: active_workers.load(Ordering::Relaxed),
                        peak_concurrency: peak_concurrency.load(Ordering::Relaxed),
                        elapsed_ms: start_time.elapsed().as_millis() as u64,
                    })
                    .await;
            }

            if stats.items_executed == pending_count {
                break;
            }
        }

        // 7. Save final checkpoint or delete on completion
        if self.resumable_config.delete_checkpoint_on_complete && checkpoint.is_fully_finished() {
            self.checkpoint_store.delete_checkpoint(&job_id_str).await?;
        } else {
            self.checkpoint_store.save_checkpoint(&checkpoint).await?;
            stats.checkpoints_saved += 1;
        }

        // 8. Consolidate results in original item order
        let sorted_results: Vec<_> = combined_results.into_values().collect();
        let summary = BatchExecutionSummary {
            batch_id,
            total_items,
            succeeded_count: sorted_results.iter().filter(|r| r.is_success()).count(),
            failed_count: sorted_results
                .iter()
                .filter(|r| matches!(r.status, BatchItemStatusOutcome::Failed { .. }))
                .count(),
            cancelled_count: sorted_results
                .iter()
                .filter(|r| matches!(r.status, BatchItemStatusOutcome::Cancelled { .. }))
                .count(),
            timed_out_count: sorted_results
                .iter()
                .filter(|r| matches!(r.status, BatchItemStatusOutcome::TimedOut { .. }))
                .count(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            peak_concurrency: peak_concurrency.load(Ordering::Relaxed),
            configured_parallelism: self.batch_config.max_parallelism,
            item_results: sorted_results,
        };

        Ok((summary, stats))
    }
}
