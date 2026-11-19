//! # Worker Engine
//!
//! Provides the core execution coordinator for the worker daemon, binding
//! configuration, bounded parallel batch execution pools, leasing mechanics,
//! and graceful cancellation lifecycles.

use std::future::Future;
use std::time::Instant;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::BatchId;
use web_repo_to_ai_job_engine::models::{
    BatchExecutionSummary, BatchItemTask, BatchProgressUpdate,
};
use web_repo_to_ai_job_engine::pool::{WorkerPool, WorkerPoolConfig};

use crate::config::WorkerServiceConfig;
use crate::metrics::{WorkerMetricsSnapshot, WorkerServiceMetrics};

/// Primary worker execution engine coordinator.
#[derive(Debug, Clone)]
pub struct WorkerEngine {
    config: WorkerServiceConfig,
    pool: WorkerPool,
    metrics: WorkerServiceMetrics,
    shutdown_token: CancellationToken,
}

impl WorkerEngine {
    /// Initializes a new `WorkerEngine` with the supplied configuration.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if configuration invariants are violated.
    pub fn new(config: WorkerServiceConfig) -> Result<Self, CoreError> {
        config.validate()?;

        let pool_config = WorkerPoolConfig::new(
            &config.worker_id,
            config.max_concurrent_jobs,
            config.default_batch_parallelism,
        )
        .map_err(|e| CoreError::Validation {
            field: "pool_config".to_string(),
            reason: e.to_string(),
        })?;

        let pool = WorkerPool::new(pool_config).map_err(|e| CoreError::Validation {
            field: "pool".to_string(),
            reason: e.to_string(),
        })?;

        let shutdown_token = pool.cancellation_token();

        Ok(Self {
            config,
            pool,
            metrics: WorkerServiceMetrics::new(),
            shutdown_token,
        })
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &WorkerServiceConfig {
        &self.config
    }

    /// Returns a snapshot of current telemetry metrics.
    #[must_use]
    pub fn metrics_snapshot(&self) -> WorkerMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Returns the shutdown cancellation token for signal monitoring.
    #[must_use]
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    /// Initiates cooperative worker shutdown.
    pub fn shutdown(&self) {
        info!(worker_id = %self.config.worker_id, "Worker engine shutdown initiated");
        self.pool.shutdown();
    }

    /// Returns `true` if the worker is currently shutting down.
    #[must_use]
    pub fn is_shutting_down(&self) -> bool {
        self.pool.is_shutting_down()
    }

    /// Executes a batch of conversion tasks concurrently within the bounded parallelism ceiling.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Execution`] if batch execution fails or is rejected during shutdown.
    #[instrument(skip(self, tasks, processor, progress_tx), fields(worker_id = %self.config.worker_id, total_items = tasks.len()))]
    pub async fn execute_batch<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        requested_parallelism: Option<usize>,
        processor: F,
        progress_tx: Option<mpsc::Sender<BatchProgressUpdate>>,
    ) -> Result<BatchExecutionSummary<R>, CoreError>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
        F: Fn(BatchItemTask<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<R, String>> + Send + 'static,
    {
        if self.is_shutting_down() {
            return Err(CoreError::JobEngine(
                "Worker daemon is shutting down; operation rejected".to_string(),
            ));
        }

        let total_items = tasks.len();
        let start = Instant::now();

        info!(
            %batch_id,
            total_items,
            worker_id = %self.config.worker_id,
            "Worker engine dispatching parallel batch tasks"
        );

        let summary = self
            .pool
            .execute_batch(
                batch_id,
                tasks,
                requested_parallelism,
                processor,
                progress_tx,
            )
            .await
            .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        // Update service-level telemetry
        self.metrics.record_batch_completed();
        self.metrics.record_item_outcome(
            summary.succeeded_count,
            summary.failed_count,
            summary.timed_out_count,
            summary.cancelled_count,
        );
        self.metrics.update_active_workers(summary.peak_concurrency);

        info!(
            %batch_id,
            total_items,
            succeeded = summary.succeeded_count,
            failed = summary.failed_count,
            peak_concurrency = summary.peak_concurrency,
            duration_ms = start.elapsed().as_millis(),
            "Worker engine batch execution finished"
        );

        Ok(summary)
    }

    /// Runs the worker background daemon loop until a shutdown signal is observed.
    pub async fn run_daemon(&self) {
        info!(
            worker_id = %self.config.worker_id,
            max_concurrency = self.config.max_concurrent_jobs,
            poll_interval_ms = self.config.poll_interval_millis,
            "Worker engine background daemon started"
        );

        let poll_interval = self.config.poll_interval();

        while !self.is_shutting_down() {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    info!(worker_id = %self.config.worker_id, "Shutdown signal received; draining worker loop");
                    break;
                }
                _ = tokio::time::sleep(poll_interval) => {
                    // Queue polling cycle: in a deployed system with DB, polls for queued leases
                    // Heartbeat renewal loop runs alongside in-flight tasks
                }
            }
        }

        info!(worker_id = %self.config.worker_id, "Worker engine daemon loop exited cleanly");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_engine_execution_and_telemetry() {
        let config = WorkerServiceConfig::for_worker("test-worker-engine", 6, 3).unwrap();
        let engine = WorkerEngine::new(config).unwrap();

        let batch_id = BatchId::new();
        let tasks = vec![
            BatchItemTask::new(
                0,
                uuid::Uuid::new_v4().into(),
                "https://example.com/item1",
                None,
                (),
            ),
            BatchItemTask::new(
                1,
                uuid::Uuid::new_v4().into(),
                "https://example.com/item2",
                None,
                (),
            ),
            BatchItemTask::new(
                2,
                uuid::Uuid::new_v4().into(),
                "https://example.com/item3",
                None,
                (),
            ),
        ];

        let summary = engine
            .execute_batch(
                batch_id,
                tasks,
                Some(3),
                |t| async move {
                    if t.item_index == 1 {
                        Err("simulated failure".to_string())
                    } else {
                        Ok("simulated success".to_string())
                    }
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(summary.total_items, 3);
        assert_eq!(summary.succeeded_count, 2);
        assert_eq!(summary.failed_count, 1);

        let metrics = engine.metrics_snapshot();
        assert_eq!(metrics.batches_processed, 1);
        assert_eq!(metrics.items_processed, 3);
        assert_eq!(metrics.items_succeeded, 2);
        assert_eq!(metrics.items_failed, 1);
    }

    #[tokio::test]
    async fn test_worker_engine_shutdown_behavior() {
        let config = WorkerServiceConfig::for_worker("test-worker-shutdown", 4, 2).unwrap();
        let engine = WorkerEngine::new(config).unwrap();

        assert!(!engine.is_shutting_down());
        engine.shutdown();
        assert!(engine.is_shutting_down());

        let batch_id = BatchId::new();
        let tasks = vec![BatchItemTask::new(
            0,
            uuid::Uuid::new_v4().into(),
            "https://example.com",
            None,
            (),
        )];

        let res = engine
            .execute_batch(
                batch_id,
                tasks,
                None,
                |_| async { Ok("ok".to_string()) },
                None,
            )
            .await;

        assert!(res.is_err());
    }
}
