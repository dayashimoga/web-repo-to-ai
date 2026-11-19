//! # Worker Pool Management
//!
//! Provides worker-level concurrency budgeting, lifecycle supervision,
//! and batch dispatching.

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;
use web_repo_to_ai_core::id::BatchId;

use crate::config::BatchExecutionConfig;
use crate::error::JobEngineError;
use crate::executor::BoundedParallelExecutor;
use crate::models::{BatchExecutionSummary, BatchItemTask, BatchProgressUpdate};

/// Default worker-wide maximum concurrent task permits.
pub const DEFAULT_MAX_WORKER_CONCURRENCY: usize = 10;

/// Default graceful shutdown wait timeout.
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECONDS: u64 = 30;

/// Configuration for the worker execution pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPoolConfig {
    /// Unique identity of this worker instance.
    pub worker_id: String,

    /// Global concurrency ceiling across all concurrently processing batches on this worker.
    pub max_worker_concurrency: usize,

    /// Default parallelism allocated per batch when unspecified.
    pub default_batch_parallelism: usize,

    /// Maximum duration allowed for active tasks to complete during graceful shutdown.
    pub shutdown_timeout: Duration,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4()),
            max_worker_concurrency: DEFAULT_MAX_WORKER_CONCURRENCY,
            default_batch_parallelism: crate::config::DEFAULT_PARALLELISM,
            shutdown_timeout: Duration::from_secs(DEFAULT_SHUTDOWN_TIMEOUT_SECONDS),
        }
    }
}

impl WorkerPoolConfig {
    /// Creates a validated `WorkerPoolConfig`.
    pub fn new(
        worker_id: impl Into<String>,
        max_worker_concurrency: usize,
        default_batch_parallelism: usize,
    ) -> Result<Self, JobEngineError> {
        let config = Self {
            worker_id: worker_id.into(),
            max_worker_concurrency,
            default_batch_parallelism,
            shutdown_timeout: Duration::from_secs(DEFAULT_SHUTDOWN_TIMEOUT_SECONDS),
        };
        config.validate()?;
        Ok(config)
    }

    /// Validates the pool configuration.
    pub fn validate(&self) -> Result<(), JobEngineError> {
        if self.worker_id.trim().is_empty() {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "worker_id cannot be blank".to_string(),
            });
        }
        if self.max_worker_concurrency == 0 || self.max_worker_concurrency > 50 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "max_worker_concurrency must be between 1 and 50 (got {})",
                    self.max_worker_concurrency
                ),
            });
        }
        if self.default_batch_parallelism == 0
            || self.default_batch_parallelism > self.max_worker_concurrency
        {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "default_batch_parallelism ({}) cannot exceed max_worker_concurrency ({})",
                    self.default_batch_parallelism, self.max_worker_concurrency
                ),
            });
        }
        Ok(())
    }
}

/// Worker telemetry metrics snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPoolMetrics {
    /// Worker identifier.
    pub worker_id: String,
    /// Total batches processed.
    pub batches_processed: usize,
    /// Total items processed.
    pub items_processed: usize,
    /// Total items succeeded.
    pub items_succeeded: usize,
    /// Total items failed.
    pub items_failed: usize,
    /// Is worker currently shutting down.
    pub is_shutting_down: bool,
}

/// Manages parallel batch task dispatch and worker-level lifecycle coordination.
#[derive(Debug, Clone)]
pub struct WorkerPool {
    config: WorkerPoolConfig,
    cancellation_token: CancellationToken,
    batches_processed: Arc<AtomicUsize>,
    items_processed: Arc<AtomicUsize>,
    items_succeeded: Arc<AtomicUsize>,
    items_failed: Arc<AtomicUsize>,
}

impl WorkerPool {
    /// Initializes a new `WorkerPool`.
    pub fn new(config: WorkerPoolConfig) -> Result<Self, JobEngineError> {
        config.validate()?;
        Ok(Self {
            config,
            cancellation_token: CancellationToken::new(),
            batches_processed: Arc::new(AtomicUsize::new(0)),
            items_processed: Arc::new(AtomicUsize::new(0)),
            items_succeeded: Arc::new(AtomicUsize::new(0)),
            items_failed: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Returns the worker pool configuration.
    #[must_use]
    pub fn config(&self) -> &WorkerPoolConfig {
        &self.config
    }

    /// Returns the cancellation token associated with this worker pool.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Initiates cooperative worker shutdown.
    pub fn shutdown(&self) {
        info!(worker_id = %self.config.worker_id, "Initiating worker pool shutdown");
        self.cancellation_token.cancel();
    }

    /// Returns `true` if the worker pool is shutting down.
    #[must_use]
    pub fn is_shutting_down(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Returns a telemetry snapshot of pool execution statistics.
    #[must_use]
    pub fn metrics(&self) -> WorkerPoolMetrics {
        WorkerPoolMetrics {
            worker_id: self.config.worker_id.clone(),
            batches_processed: self.batches_processed.load(Ordering::Relaxed),
            items_processed: self.items_processed.load(Ordering::Relaxed),
            items_succeeded: self.items_succeeded.load(Ordering::Relaxed),
            items_failed: self.items_failed.load(Ordering::Relaxed),
            is_shutting_down: self.is_shutting_down(),
        }
    }

    /// Dispatches and executes a batch within the worker pool.
    pub async fn execute_batch<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        requested_parallelism: Option<usize>,
        processor: F,
        progress_tx: Option<mpsc::Sender<BatchProgressUpdate>>,
    ) -> Result<BatchExecutionSummary<R>, JobEngineError>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
        F: Fn(BatchItemTask<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<R, String>> + Send + 'static,
    {
        if self.is_shutting_down() {
            return Err(JobEngineError::WorkerShuttingDown);
        }

        // Bounded parallelism: min(requested, pool_concurrency_ceiling)
        let effective_parallelism = requested_parallelism
            .unwrap_or(self.config.default_batch_parallelism)
            .min(self.config.max_worker_concurrency);

        let execution_config = BatchExecutionConfig::with_parallelism(effective_parallelism)?;
        let executor = BoundedParallelExecutor::new(execution_config)?;

        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                processor,
                progress_tx,
                Some(self.cancellation_token.clone()),
            )
            .await?;

        // Update telemetry counters
        self.batches_processed.fetch_add(1, Ordering::Relaxed);
        self.items_processed
            .fetch_add(summary.total_items, Ordering::Relaxed);
        self.items_succeeded
            .fetch_add(summary.succeeded_count, Ordering::Relaxed);
        self.items_failed.fetch_add(
            summary.failed_count + summary.timed_out_count,
            Ordering::Relaxed,
        );

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_config_validation() {
        let default_cfg = WorkerPoolConfig::default();
        assert!(default_cfg.validate().is_ok());

        let invalid_blank_id = WorkerPoolConfig {
            worker_id: "  ".to_string(),
            ..Default::default()
        };
        assert!(invalid_blank_id.validate().is_err());

        let invalid_parallelism_exceeding_ceiling = WorkerPoolConfig {
            max_worker_concurrency: 5,
            default_batch_parallelism: 10,
            ..Default::default()
        };
        assert!(invalid_parallelism_exceeding_ceiling.validate().is_err());
    }

    #[tokio::test]
    async fn test_worker_pool_batch_execution() {
        let config = WorkerPoolConfig::new("test-worker-1", 8, 4).unwrap();
        let pool = WorkerPool::new(config).unwrap();

        let batch_id = BatchId::new();
        let tasks = vec![
            BatchItemTask::new(
                0,
                uuid::Uuid::new_v4().into(),
                "https://example.com/1",
                None,
                (),
            ),
            BatchItemTask::new(
                1,
                uuid::Uuid::new_v4().into(),
                "https://example.com/2",
                None,
                (),
            ),
        ];

        let summary = pool
            .execute_batch(
                batch_id,
                tasks,
                Some(2),
                |_t| async move { Ok("done".to_string()) },
                None,
            )
            .await
            .unwrap();

        assert_eq!(summary.succeeded_count, 2);
        let metrics = pool.metrics();
        assert_eq!(metrics.batches_processed, 1);
        assert_eq!(metrics.items_succeeded, 2);
    }
}
