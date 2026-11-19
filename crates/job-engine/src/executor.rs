//! # Bounded Parallel Executor
//!
//! Provides resource-bounded asynchronous parallel task execution using Tokio semaphores,
//! atomic concurrency telemetry, cooperative cancellation, per-item timeouts, and
//! resilient error containment.

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, Semaphore};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use web_repo_to_ai_core::id::{BatchId, JobId};

use crate::config::BatchExecutionConfig;
use crate::error::JobEngineError;
use crate::models::{
    BatchExecutionSummary, BatchItemExecutionResult, BatchItemTask, BatchProgressUpdate,
};

/// High-performance bounded parallel executor for batch jobs.
///
/// Uses an `Arc<Semaphore>` to strictly constrain the number of concurrent in-flight
/// tasks, protecting system resources (CPU, memory, file descriptors).
#[derive(Debug, Clone)]
pub struct BoundedParallelExecutor {
    config: BatchExecutionConfig,
    active_workers: Arc<AtomicUsize>,
    peak_concurrency: Arc<AtomicUsize>,
}

impl BoundedParallelExecutor {
    /// Constructs a new `BoundedParallelExecutor` with the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if configuration parameters are invalid.
    pub fn new(config: BatchExecutionConfig) -> Result<Self, JobEngineError> {
        config.validate()?;
        Ok(Self {
            config,
            active_workers: Arc::new(AtomicUsize::new(0)),
            peak_concurrency: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &BatchExecutionConfig {
        &self.config
    }

    /// Returns the current number of concurrently executing worker tasks.
    #[must_use]
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }

    /// Returns the peak concurrency observed across executions.
    #[must_use]
    pub fn peak_concurrency(&self) -> usize {
        self.peak_concurrency.load(Ordering::Relaxed)
    }

    /// Executes a collection of batch item tasks concurrently within the configured parallelism bound.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Input task payload type (must be `Send + Sync + 'static`).
    /// - `R`: Output result value type (must be `Send + Sync + 'static`).
    /// - `F`: Async task processor closure mapping `BatchItemTask<T>` to `Result<R, String>`.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::EmptyBatch`] if the tasks list is empty.
    /// Returns [`JobEngineError::WorkerShuttingDown`] if cancelled prior to start.
    #[instrument(skip(self, tasks, processor, progress_tx, cancellation_token), fields(total_items = tasks.len()))]
    pub async fn execute_batch<T, R, F, Fut>(
        &self,
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        processor: F,
        progress_tx: Option<mpsc::Sender<BatchProgressUpdate>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<BatchExecutionSummary<R>, JobEngineError>
    where
        T: Send + Sync + 'static,
        R: Send + Sync + 'static,
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

        let total_items = tasks.len();
        let semaphore = Arc::new(Semaphore::new(self.config.max_parallelism));
        let batch_start = Instant::now();
        let internal_cancel = CancellationToken::new();

        info!(
            %batch_id,
            total_items,
            max_parallelism = self.config.max_parallelism,
            "Starting bounded parallel batch execution"
        );

        let mut join_handles = Vec::with_capacity(total_items);
        let completed_count = Arc::new(AtomicUsize::new(0));
        let failed_count = Arc::new(AtomicUsize::new(0));

        for task in tasks {
            let sem = Arc::clone(&semaphore);
            let active = Arc::clone(&self.active_workers);
            let peak_tracker = Arc::clone(&self.peak_concurrency);
            let proc = processor.clone();
            let parent_cancel = cancel_token.clone();
            let sub_cancel = internal_cancel.clone();
            let item_timeout_dur = self.config.item_timeout;
            let abort_on_first = self.config.abort_on_first_error;
            let comp_counter = Arc::clone(&completed_count);
            let fail_counter = Arc::clone(&failed_count);
            let prog_tx = progress_tx.clone();

            let handle = tokio::spawn(async move {
                let item_index = task.item_index;
                let job_id = task.job_id;
                let url = task.url.clone();
                let custom_id = task.custom_id.clone();

                // Check cancellation before acquiring permit
                if parent_cancel.is_cancelled() || sub_cancel.is_cancelled() {
                    fail_counter.fetch_add(1, Ordering::Relaxed);
                    return BatchItemExecutionResult::cancelled(
                        item_index,
                        job_id,
                        url,
                        custom_id,
                        "Batch execution cancelled before start",
                    );
                }

                // Acquire permit from bounded semaphore
                let permit = match sem.acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => {
                        fail_counter.fetch_add(1, Ordering::Relaxed);
                        return BatchItemExecutionResult::cancelled(
                            item_index,
                            job_id,
                            url,
                            custom_id,
                            "Semaphore closed",
                        );
                    }
                };

                // Track active worker count and peak concurrency
                let current_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                let mut peak = peak_tracker.load(Ordering::Relaxed);
                while current_active > peak {
                    match peak_tracker.compare_exchange_weak(
                        peak,
                        current_active,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(actual) => peak = actual,
                    }
                }

                debug!(
                    item_index,
                    %job_id,
                    url = %url,
                    active = current_active,
                    "Acquired worker permit, executing task"
                );

                let item_start = Instant::now();

                // Execute task with timeout and cooperative cancellation monitoring
                let result = tokio::select! {
                    _ = parent_cancel.cancelled() => {
                        fail_counter.fetch_add(1, Ordering::Relaxed);
                        BatchItemExecutionResult::cancelled(
                            item_index,
                            job_id,
                            url,
                            custom_id,
                            "Execution cancelled by client request",
                        )
                    }
                    _ = sub_cancel.cancelled() => {
                        fail_counter.fetch_add(1, Ordering::Relaxed);
                        BatchItemExecutionResult::cancelled(
                            item_index,
                            job_id,
                            url,
                            custom_id,
                            "Execution cancelled due to prior error in batch",
                        )
                    }
                    outcome = timeout(item_timeout_dur, proc(task)) => {
                        let duration_ms = item_start.elapsed().as_millis() as u64;
                        match outcome {
                            Ok(Ok(val)) => {
                                comp_counter.fetch_add(1, Ordering::Relaxed);
                                BatchItemExecutionResult::success(
                                    item_index,
                                    job_id,
                                    url,
                                    custom_id,
                                    duration_ms,
                                    val,
                                )
                            }
                            Ok(Err(err_msg)) => {
                                fail_counter.fetch_add(1, Ordering::Relaxed);
                                warn!(
                                    item_index,
                                    %job_id,
                                    error = %err_msg,
                                    "Item execution failed"
                                );
                                if abort_on_first {
                                    sub_cancel.cancel();
                                }
                                BatchItemExecutionResult::failure(
                                    item_index,
                                    job_id,
                                    url,
                                    custom_id,
                                    duration_ms,
                                    err_msg,
                                )
                            }
                            Err(_) => {
                                fail_counter.fetch_add(1, Ordering::Relaxed);
                                error!(
                                    item_index,
                                    %job_id,
                                    timeout_ms = item_timeout_dur.as_millis(),
                                    "Item execution timed out"
                                );
                                if abort_on_first {
                                    sub_cancel.cancel();
                                }
                                BatchItemExecutionResult::timed_out(
                                    item_index,
                                    job_id,
                                    url,
                                    custom_id,
                                    duration_ms,
                                )
                            }
                        }
                    }
                };

                // Decrement active worker counter and release permit
                active.fetch_sub(1, Ordering::SeqCst);
                drop(permit);

                // Send progress notification if channel is provided
                if let Some(tx) = prog_tx {
                    let progress = BatchProgressUpdate {
                        batch_id,
                        total_items,
                        completed_items: comp_counter.load(Ordering::Relaxed),
                        failed_items: fail_counter.load(Ordering::Relaxed),
                        active_workers: active.load(Ordering::Relaxed),
                        peak_concurrency: peak_tracker.load(Ordering::Relaxed),
                        elapsed_ms: batch_start.elapsed().as_millis() as u64,
                    };
                    let _ = tx.send(progress).await;
                }

                result
            });

            join_handles.push(handle);
        }

        // Collect all task results
        let mut item_results = Vec::with_capacity(total_items);
        for handle in join_handles {
            match handle.await {
                Ok(item_result) => item_results.push(item_result),
                Err(join_err) => {
                    // Task panicked or was aborted by Tokio runtime
                    error!(error = %join_err, "Task panicked or aborted on join");
                    item_results.push(BatchItemExecutionResult::failure(
                        item_results.len(),
                        JobId::new(),
                        "unknown",
                        None,
                        0,
                        format!("Task joined with error: {join_err}"),
                    ));
                }
            }
        }

        // Sort results by item_index to preserve deterministic sequence ordering
        item_results.sort_by_key(|r| r.item_index);

        let mut succeeded_count = 0;
        let mut failed_count = 0;
        let mut cancelled_count = 0;
        let mut timed_out_count = 0;

        for r in &item_results {
            match r.status {
                crate::models::BatchItemStatusOutcome::Succeeded => succeeded_count += 1,
                crate::models::BatchItemStatusOutcome::Failed { .. } => failed_count += 1,
                crate::models::BatchItemStatusOutcome::TimedOut { .. } => timed_out_count += 1,
                crate::models::BatchItemStatusOutcome::Cancelled { .. } => cancelled_count += 1,
            }
        }

        let duration_ms = batch_start.elapsed().as_millis() as u64;
        let peak_concurrency = self.peak_concurrency.load(Ordering::Relaxed);

        info!(
            %batch_id,
            total_items,
            succeeded_count,
            failed_count,
            cancelled_count,
            timed_out_count,
            peak_concurrency,
            duration_ms,
            "Batch execution completed"
        );

        Ok(BatchExecutionSummary {
            batch_id,
            total_items,
            succeeded_count,
            failed_count,
            cancelled_count,
            timed_out_count,
            duration_ms,
            peak_concurrency,
            configured_parallelism: self.config.max_parallelism,
            item_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_executor_strict_concurrency_ceiling() {
        // Configure max parallelism = 3
        let config = BatchExecutionConfig::with_parallelism(3).unwrap();
        let executor = BoundedParallelExecutor::new(config).unwrap();

        let batch_id = BatchId::new();
        let tasks: Vec<BatchItemTask<()>> = (0..12)
            .map(|i| {
                BatchItemTask::new(
                    i,
                    JobId::new(),
                    format!("https://example.com/{i}"),
                    None,
                    (),
                )
            })
            .collect();

        // Each task simulates 30ms of work
        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                |_task| async move {
                    sleep(Duration::from_millis(30)).await;
                    Ok::<String, String>("ok".to_string())
                },
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(summary.total_items, 12);
        assert_eq!(summary.succeeded_count, 12);
        assert!(summary.all_succeeded());
        // Peak concurrency must strictly never exceed configured max_parallelism (3)
        assert!(summary.peak_concurrency <= 3);
        assert!(summary.peak_concurrency >= 1);
        assert_eq!(executor.active_workers(), 0);
    }

    #[tokio::test]
    async fn test_executor_resilient_error_isolation() {
        // 10 items, odd items succeed, even items fail
        let config = BatchExecutionConfig::with_parallelism(4).unwrap();
        let executor = BoundedParallelExecutor::new(config).unwrap();

        let batch_id = BatchId::new();
        let tasks: Vec<BatchItemTask<()>> = (0..10)
            .map(|i| {
                BatchItemTask::new(
                    i,
                    JobId::new(),
                    format!("https://example.com/{i}"),
                    None,
                    (),
                )
            })
            .collect();

        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                |task| async move {
                    if task.item_index % 2 == 0 {
                        Err(format!("Error on item {}", task.item_index))
                    } else {
                        Ok(format!("Result {}", task.item_index))
                    }
                },
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(summary.total_items, 10);
        assert_eq!(summary.succeeded_count, 5);
        assert_eq!(summary.failed_count, 5);
        assert!(summary.has_failures());
        assert!((summary.success_rate() - 0.5).abs() < f64::EPSILON);

        // Sequence ordering preserved
        for (idx, res) in summary.item_results.iter().enumerate() {
            assert_eq!(res.item_index, idx);
            if idx % 2 == 0 {
                assert!(res.status.is_failure());
            } else {
                assert!(res.status.is_success());
            }
        }
    }

    #[tokio::test]
    async fn test_executor_timeout_enforcement() {
        let config = BatchExecutionConfig::with_parallelism(2)
            .unwrap()
            .with_item_timeout(Duration::from_millis(50))
            .with_batch_timeout(Duration::from_secs(5));
        let executor = BoundedParallelExecutor::new(config).unwrap();

        let batch_id = BatchId::new();
        let tasks: Vec<BatchItemTask<()>> = vec![
            BatchItemTask::new(0, JobId::new(), "https://example.com/fast", None, ()),
            BatchItemTask::new(1, JobId::new(), "https://example.com/slow", None, ()),
        ];

        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                |task| async move {
                    if task.item_index == 1 {
                        sleep(Duration::from_millis(200)).await;
                    }
                    Ok("completed".to_string())
                },
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(summary.succeeded_count, 1);
        assert_eq!(summary.timed_out_count, 1);
        assert!(summary.item_results[0].status.is_success());
        assert!(matches!(
            summary.item_results[1].status,
            crate::models::BatchItemStatusOutcome::TimedOut { .. }
        ));
    }

    #[tokio::test]
    async fn test_executor_cancellation_cooperation() {
        let config = BatchExecutionConfig::with_parallelism(2).unwrap();
        let executor = BoundedParallelExecutor::new(config).unwrap();
        let cancel = CancellationToken::new();

        let batch_id = BatchId::new();
        let tasks: Vec<BatchItemTask<()>> = (0..6)
            .map(|i| {
                BatchItemTask::new(
                    i,
                    JobId::new(),
                    format!("https://example.com/{i}"),
                    None,
                    (),
                )
            })
            .collect();

        // Cancel after 20ms
        let cancel_clone = cancel.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(20)).await;
            cancel_clone.cancel();
        });

        let summary = executor
            .execute_batch(
                batch_id,
                tasks,
                |_task| async move {
                    sleep(Duration::from_millis(50)).await;
                    Ok("done".to_string())
                },
                None,
                Some(cancel),
            )
            .await
            .unwrap();

        assert!(summary.cancelled_count > 0 || summary.failed_count > 0);
    }
}
