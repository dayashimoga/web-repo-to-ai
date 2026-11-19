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
use web_repo_to_ai_job_engine::adaptive::{
    AdaptiveConcurrencyConfig, AdaptiveConcurrencyController, AdaptiveParallelExecutor,
};
use web_repo_to_ai_job_engine::backpressure::{BackpressureConfig, BackpressureParallelExecutor};
use web_repo_to_ai_job_engine::executor::BoundedParallelExecutor;
use web_repo_to_ai_job_engine::incremental::{
    FileSignature, IncrementalCandidate, IncrementalExecutionReport, IncrementalJobCoordinator,
};
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

    /// Executes an incremental batch of tasks comparing current candidates against a baseline manifest.
    ///
    /// Unchanged items bypass execution and are resolved directly via `reuse_resolver`,
    /// while added or modified items are executed concurrently within the worker's bounded concurrency ceiling.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] if the worker is shutting down or incremental setup/execution fails.
    #[instrument(skip(self, candidates, baseline, processor, reuse_resolver), fields(worker_id = %self.config.worker_id, total_candidates = candidates.len()))]
    pub async fn execute_incremental_batch<T, R, FP, FR, FutP, FutR>(
        &self,
        batch_id: BatchId,
        candidates: Vec<IncrementalCandidate<T>>,
        baseline: &[FileSignature],
        requested_parallelism: Option<usize>,
        processor: FP,
        reuse_resolver: FR,
    ) -> Result<IncrementalExecutionReport<R>, CoreError>
    where
        T: Send + Sync + Clone + 'static,
        R: Send + Sync + Clone + 'static,
        FP: Fn(BatchItemTask<T>) -> FutP + Send + Sync + Clone + 'static,
        FutP: Future<Output = Result<R, String>> + Send + 'static,
        FR: Fn(&FileSignature) -> FutR + Send + Sync + 'static,
        FutR: Future<Output = Result<R, String>> + Send + 'static,
    {
        if self.is_shutting_down() {
            return Err(CoreError::JobEngine(
                "Worker daemon is shutting down; incremental operation rejected".to_string(),
            ));
        }

        let parallelism = requested_parallelism
            .unwrap_or(self.config.default_batch_parallelism)
            .clamp(1, self.config.max_concurrent_jobs);

        let lease_secs = u64::from(self.config.lease_duration_seconds);
        let exec_config =
            web_repo_to_ai_job_engine::config::BatchExecutionConfig::with_parallelism(parallelism)
                .map(|c| {
                    c.with_batch_timeout(std::time::Duration::from_secs(lease_secs * 2))
                        .with_item_timeout(std::time::Duration::from_secs(lease_secs))
                })
                .map_err(|e| CoreError::Validation {
                    field: "exec_config".to_string(),
                    reason: e.to_string(),
                })?;

        let executor = BoundedParallelExecutor::new(exec_config)
            .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        let coordinator = IncrementalJobCoordinator::new(executor);

        let report = coordinator
            .execute_incremental(batch_id, candidates, baseline, processor, reuse_resolver)
            .await
            .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        self.metrics.record_batch_completed();
        self.metrics
            .record_item_outcome(report.processed_count, report.failed_count, 0, 0);
        self.metrics
            .record_incremental_reuse(report.reused_count, report.reused_bytes);

        info!(
            %batch_id,
            total_candidates = report.total_candidates,
            reused = report.reused_count,
            processed = report.processed_count,
            failed = report.failed_count,
            cache_hit_ratio = report.cache_hit_ratio,
            reused_bytes = report.reused_bytes,
            "Worker engine incremental batch execution finished"
        );

        Ok(report)
    }

    /// Executes a batch of conversion tasks using dynamic AIMD adaptive concurrency regulation.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Execution`] if batch execution fails or is rejected during shutdown.
    #[instrument(skip(self, tasks, adaptive_config, processor, progress_tx), fields(worker_id = %self.config.worker_id, total_items = tasks.len()))]
    pub async fn execute_adaptive_batch<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        adaptive_config: Option<AdaptiveConcurrencyConfig>,
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

        let config = match adaptive_config {
            Some(cfg) => cfg,
            None => AdaptiveConcurrencyConfig::new(
                1,
                self.config.max_concurrent_jobs,
                self.config.default_batch_parallelism,
            )
            .map_err(|e| CoreError::Validation {
                field: "adaptive_config".to_string(),
                reason: e.to_string(),
            })?,
        };

        let lease_secs = u64::from(self.config.lease_duration_seconds);
        let batch_config =
            web_repo_to_ai_job_engine::config::BatchExecutionConfig::with_parallelism(
                config.max_concurrency,
            )
            .map(|c| {
                c.with_batch_timeout(std::time::Duration::from_secs(lease_secs * 2))
                    .with_item_timeout(std::time::Duration::from_secs(lease_secs))
            })
            .map_err(|e| CoreError::Validation {
                field: "batch_config".to_string(),
                reason: e.to_string(),
            })?;

        let controller = AdaptiveConcurrencyController::new(config)
            .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        let executor = AdaptiveParallelExecutor::new(controller.clone(), batch_config);

        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                processor,
                progress_tx,
                Some(self.shutdown_token()),
            )
            .await
            .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        let stats = controller.stats();
        self.metrics.record_batch_completed();
        self.metrics.record_item_outcome(
            summary.succeeded_count,
            summary.failed_count,
            summary.timed_out_count,
            summary.cancelled_count,
        );
        self.metrics.record_adaptive_stats(
            stats.total_scale_ups,
            stats.total_backoffs,
            stats.total_rate_limits,
            stats.current_limit,
        );
        self.metrics.update_active_workers(summary.peak_concurrency);

        info!(
            %batch_id,
            total_items,
            succeeded = summary.succeeded_count,
            failed = summary.failed_count,
            final_limit = stats.current_limit,
            scale_ups = stats.total_scale_ups,
            backoffs = stats.total_backoffs,
            duration_ms = start.elapsed().as_millis(),
            "Worker engine adaptive batch execution finished"
        );

        Ok(summary)
    }

    /// Executes a batch of tasks applying bounded backpressure flow control across producers and workers.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Execution`] if batch execution fails or is rejected during shutdown.
    #[instrument(skip(self, tasks, backpressure_config, processor, progress_tx), fields(worker_id = %self.config.worker_id, total_items = tasks.len()))]
    pub async fn execute_flow_controlled_batch<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        backpressure_config: Option<BackpressureConfig>,
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

        let bp_config = match backpressure_config {
            Some(cfg) => cfg,
            None => {
                let max_cap = self.config.max_concurrent_jobs * 4;
                let high = (max_cap as f64 * 0.8).floor() as usize;
                let low = (max_cap as f64 * 0.4).floor() as usize;
                BackpressureConfig::new(max_cap, high, low).map_err(|e| CoreError::Validation {
                    field: "backpressure_config".to_string(),
                    reason: e.to_string(),
                })?
            }
        };

        let lease_secs = u64::from(self.config.lease_duration_seconds);
        let batch_config =
            web_repo_to_ai_job_engine::config::BatchExecutionConfig::with_parallelism(
                self.config.default_batch_parallelism,
            )
            .map(|c| {
                c.with_batch_timeout(std::time::Duration::from_secs(lease_secs * 2))
                    .with_item_timeout(std::time::Duration::from_secs(lease_secs))
            })
            .map_err(|e| CoreError::Validation {
                field: "batch_config".to_string(),
                reason: e.to_string(),
            })?;

        let summary = BackpressureParallelExecutor::execute_flow_controlled(
            batch_id,
            tasks,
            bp_config,
            batch_config,
            processor,
            progress_tx,
            Some(self.shutdown_token()),
        )
        .await
        .map_err(|e| CoreError::JobEngine(e.to_string()))?;

        self.metrics.record_batch_completed();
        self.metrics.record_item_outcome(
            summary.succeeded_count,
            summary.failed_count,
            summary.timed_out_count,
            summary.cancelled_count,
        );
        self.metrics
            .record_backpressure_stats(0, 0, summary.peak_concurrency);
        self.metrics.update_active_workers(summary.peak_concurrency);

        info!(
            %batch_id,
            total_items,
            succeeded = summary.succeeded_count,
            failed = summary.failed_count,
            peak_concurrency = summary.peak_concurrency,
            duration_ms = start.elapsed().as_millis(),
            "Worker engine flow-controlled batch execution finished"
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
