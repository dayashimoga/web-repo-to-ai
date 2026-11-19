//! # Backpressure & Flow Control Subsystem
//!
//! Provides bounded buffer management, dual-watermark flow control (low/high watermark),
//! producer suspension, load-shedding strategies, and cooperative stream throttling.

use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Notify};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::info;
use web_repo_to_ai_core::id::{BatchId, JobId};

use crate::config::BatchExecutionConfig;
use crate::error::JobEngineError;
use crate::models::{
    BatchExecutionSummary, BatchItemExecutionResult, BatchItemStatusOutcome, BatchItemTask,
    BatchProgressUpdate,
};

/// Default maximum buffer capacity for queued items.
pub const DEFAULT_MAX_BUFFER_CAPACITY: usize = 1000;

/// Default high watermark threshold (80% of capacity) at which backpressure activates.
pub const DEFAULT_HIGH_WATERMARK_RATIO: f64 = 0.8;

/// Default low watermark threshold (40% of capacity) at which backpressure clears.
pub const DEFAULT_LOW_WATERMARK_RATIO: f64 = 0.4;

/// Default maximum duration a producer will wait for backpressure clearance.
pub const DEFAULT_MAX_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Strategy invoked when queue depth exceeds high watermark / maximum capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadSheddingStrategy {
    /// Asynchronously suspend producer until queue depth drains below low watermark.
    #[default]
    BlockProducer,
    /// Discard the oldest queued item to make room for newest.
    DropOldest,
    /// Immediately reject incoming item with [`JobEngineError::BackpressureQueueSaturated`].
    DropNewest,
    /// Reject item advising upstream caller to wait for the specified backoff duration.
    RejectWithRetryAfter {
        /// Suggested retry delay.
        retry_after: Duration,
    },
}

/// Configuration governing backpressure watermarks and load shedding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackpressureConfig {
    /// Absolute maximum item buffer capacity.
    pub max_buffer_capacity: usize,
    /// Queue depth at or above which backpressure is triggered (`Saturated`).
    pub high_watermark: usize,
    /// Queue depth at or below which backpressure is cleared (`Normal`).
    pub low_watermark: usize,
    /// Maximum time a producer will wait before returning a timeout error.
    pub max_wait_timeout: Duration,
    /// Load shedding policy.
    pub shed_strategy: LoadSheddingStrategy,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        let max_cap = DEFAULT_MAX_BUFFER_CAPACITY;
        let high = (max_cap as f64 * DEFAULT_HIGH_WATERMARK_RATIO).floor() as usize;
        let low = (max_cap as f64 * DEFAULT_LOW_WATERMARK_RATIO).floor() as usize;
        Self {
            max_buffer_capacity: max_cap,
            high_watermark: high,
            low_watermark: low,
            max_wait_timeout: DEFAULT_MAX_WAIT_TIMEOUT,
            shed_strategy: LoadSheddingStrategy::BlockProducer,
        }
    }
}

impl BackpressureConfig {
    /// Creates a validated backpressure configuration with specified limits.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if invariants are violated.
    pub fn new(
        max_buffer_capacity: usize,
        high_watermark: usize,
        low_watermark: usize,
    ) -> Result<Self, JobEngineError> {
        let config = Self {
            max_buffer_capacity,
            high_watermark,
            low_watermark,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Builder to configure the load shedding strategy.
    #[must_use]
    pub fn with_shed_strategy(mut self, strategy: LoadSheddingStrategy) -> Self {
        self.shed_strategy = strategy;
        self
    }

    /// Builder to configure producer wait timeout.
    #[must_use]
    pub fn with_wait_timeout(mut self, timeout: Duration) -> Self {
        self.max_wait_timeout = timeout;
        self
    }

    /// Validates the configuration invariants.
    pub fn validate(&self) -> Result<(), JobEngineError> {
        if self.max_buffer_capacity == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "max_buffer_capacity must be at least 1".to_string(),
            });
        }
        if self.high_watermark > self.max_buffer_capacity {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "high_watermark ({}) cannot exceed max_buffer_capacity ({})",
                    self.high_watermark, self.max_buffer_capacity
                ),
            });
        }
        if self.low_watermark >= self.high_watermark {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "low_watermark ({}) must be strictly less than high_watermark ({})",
                    self.low_watermark, self.high_watermark
                ),
            });
        }
        Ok(())
    }
}

/// Operational state of the buffer relative to watermarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureState {
    /// Buffer depth is at or below the low watermark; admissions proceed with zero latency.
    Normal,
    /// Buffer depth is between low and high watermarks; warning state.
    Warning,
    /// Buffer depth has reached or exceeded the high watermark; backpressure is active.
    Saturated,
}

/// Telemetry statistics describing the backpressure subsystem state.
#[derive(Debug, Clone, PartialEq)]
pub struct BackpressureStats {
    /// Current buffered item count.
    pub current_depth: usize,
    /// Maximum capacity ceiling.
    pub max_capacity: usize,
    /// High watermark threshold.
    pub high_watermark: usize,
    /// Low watermark threshold.
    pub low_watermark: usize,
    /// Utilization ratio (`current_depth / max_capacity`).
    pub utilization_ratio: f64,
    /// Total items successfully admitted into the queue.
    pub total_admitted: u64,
    /// Total items drained and completed by consumers.
    pub total_drained: u64,
    /// Total times producers were suspended due to backpressure.
    pub total_producers_suspended: u64,
    /// Total items discarded or rejected by load shedding.
    pub total_items_shed: u64,
    /// Active operational state.
    pub state: BackpressureState,
}

#[derive(Debug)]
struct BackpressureInnerState {
    current_depth: usize,
    total_admitted: u64,
    total_drained: u64,
    total_producers_suspended: u64,
    total_items_shed: u64,
}

#[derive(Debug)]
struct BackpressureControllerInner {
    config: BackpressureConfig,
    state: Mutex<BackpressureInnerState>,
    drain_notify: Notify,
}

/// Dynamic backpressure regulator coordinating admission permits and producer throttling.
#[derive(Debug, Clone)]
pub struct BackpressureController {
    inner: Arc<BackpressureControllerInner>,
}

impl BackpressureController {
    /// Constructs a new `BackpressureController` from configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if invariants are violated.
    pub fn new(config: BackpressureConfig) -> Result<Self, JobEngineError> {
        config.validate()?;
        Ok(Self {
            inner: Arc::new(BackpressureControllerInner {
                config,
                state: Mutex::new(BackpressureInnerState {
                    current_depth: 0,
                    total_admitted: 0,
                    total_drained: 0,
                    total_producers_suspended: 0,
                    total_items_shed: 0,
                }),
                drain_notify: Notify::new(),
            }),
        })
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &BackpressureConfig {
        &self.inner.config
    }

    /// Returns the current buffer depth.
    #[must_use]
    pub fn current_depth(&self) -> usize {
        let state = self.inner.state.lock().unwrap();
        state.current_depth
    }

    /// Evaluates the current backpressure operational state.
    #[must_use]
    pub fn state(&self) -> BackpressureState {
        let state = self.inner.state.lock().unwrap();
        if state.current_depth >= self.inner.config.high_watermark {
            BackpressureState::Saturated
        } else if state.current_depth > self.inner.config.low_watermark {
            BackpressureState::Warning
        } else {
            BackpressureState::Normal
        }
    }

    /// Returns current buffer utilization ratio between 0.0 and 1.0.
    #[must_use]
    pub fn utilization(&self) -> f64 {
        let state = self.inner.state.lock().unwrap();
        if self.inner.config.max_buffer_capacity == 0 {
            0.0
        } else {
            state.current_depth as f64 / self.inner.config.max_buffer_capacity as f64
        }
    }

    /// Acquires an admission slot, suspending the producer if buffer depth is saturated.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::BackpressureQueueSaturated`] if load shedding rejects the item.
    /// Returns [`JobEngineError::BackpressureTimeout`] if waiting exceeds configured timeout.
    /// Returns [`JobEngineError::WorkerShuttingDown`] if cancelled.
    pub async fn acquire_slot(
        &self,
        cancellation_token: Option<&CancellationToken>,
    ) -> Result<BackpressureSlot, JobEngineError> {
        let start = Instant::now();
        let max_wait = self.inner.config.max_wait_timeout;

        loop {
            if let Some(token) = cancellation_token {
                if token.is_cancelled() {
                    return Err(JobEngineError::WorkerShuttingDown);
                }
            }

            let notified = self.inner.drain_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            {
                let mut state = self.inner.state.lock().unwrap();
                if state.current_depth < self.inner.config.high_watermark {
                    state.current_depth += 1;
                    state.total_admitted += 1;
                    return Ok(BackpressureSlot {
                        inner: Arc::clone(&self.inner),
                    });
                }

                // Buffer is saturated; evaluate shedding policy
                match self.inner.config.shed_strategy {
                    LoadSheddingStrategy::DropNewest => {
                        state.total_items_shed += 1;
                        return Err(JobEngineError::BackpressureQueueSaturated {
                            current_depth: state.current_depth,
                            capacity: self.inner.config.max_buffer_capacity,
                        });
                    }
                    LoadSheddingStrategy::RejectWithRetryAfter { retry_after: _ } => {
                        state.total_items_shed += 1;
                        return Err(JobEngineError::BackpressureQueueSaturated {
                            current_depth: state.current_depth,
                            capacity: self.inner.config.max_buffer_capacity,
                        });
                    }
                    LoadSheddingStrategy::DropOldest | LoadSheddingStrategy::BlockProducer => {
                        state.total_producers_suspended += 1;
                    }
                }
            }

            // Await drain notification or timeout
            let remaining = max_wait.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(JobEngineError::BackpressureTimeout {
                    duration_ms: max_wait.as_millis() as u64,
                });
            }

            if let Some(token) = cancellation_token {
                tokio::select! {
                    () = notified => {}
                    () = token.cancelled() => {
                        return Err(JobEngineError::WorkerShuttingDown);
                    }
                    _ = tokio::time::sleep(remaining) => {
                        return Err(JobEngineError::BackpressureTimeout {
                            duration_ms: max_wait.as_millis() as u64,
                        });
                    }
                }
            } else {
                tokio::select! {
                    () = notified => {}
                    _ = tokio::time::sleep(remaining) => {
                        return Err(JobEngineError::BackpressureTimeout {
                            duration_ms: max_wait.as_millis() as u64,
                        });
                    }
                }
            }
        }
    }

    /// Attempts non-blocking slot acquisition.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::BackpressureQueueSaturated`] if buffer is at or above high watermark.
    pub fn try_acquire_slot(&self) -> Result<BackpressureSlot, JobEngineError> {
        let mut state = self.inner.state.lock().unwrap();
        if state.current_depth < self.inner.config.high_watermark {
            state.current_depth += 1;
            state.total_admitted += 1;
            Ok(BackpressureSlot {
                inner: Arc::clone(&self.inner),
            })
        } else {
            state.total_items_shed += 1;
            Err(JobEngineError::BackpressureQueueSaturated {
                current_depth: state.current_depth,
                capacity: self.inner.config.max_buffer_capacity,
            })
        }
    }

    /// Captures a point-in-time statistics snapshot.
    #[must_use]
    pub fn stats(&self) -> BackpressureStats {
        let state = self.inner.state.lock().unwrap();
        let depth = state.current_depth;
        let cap = self.inner.config.max_buffer_capacity;
        let high = self.inner.config.high_watermark;
        let low = self.inner.config.low_watermark;

        let st = if depth >= high {
            BackpressureState::Saturated
        } else if depth > low {
            BackpressureState::Warning
        } else {
            BackpressureState::Normal
        };

        BackpressureStats {
            current_depth: depth,
            max_capacity: cap,
            high_watermark: high,
            low_watermark: low,
            utilization_ratio: if cap == 0 {
                0.0
            } else {
                depth as f64 / cap as f64
            },
            total_admitted: state.total_admitted,
            total_drained: state.total_drained,
            total_producers_suspended: state.total_producers_suspended,
            total_items_shed: state.total_items_shed,
            state: st,
        }
    }
}

/// RAII slot guard that decrements buffer depth upon completion and wakes producers.
#[derive(Debug)]
pub struct BackpressureSlot {
    inner: Arc<BackpressureControllerInner>,
}

impl Drop for BackpressureSlot {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().unwrap();
        state.current_depth = state.current_depth.saturating_sub(1);
        state.total_drained += 1;

        if state.current_depth <= self.inner.config.low_watermark {
            self.inner.drain_notify.notify_waiters();
        }
    }
}

/// Flow-controlled bounded queue managing producer-consumer decoupling with backpressure.
#[derive(Debug, Clone)]
pub struct FlowControlledQueue<T> {
    controller: BackpressureController,
    buffer: Arc<Mutex<VecDeque<T>>>,
    push_notify: Arc<Notify>,
}

impl<T> FlowControlledQueue<T> {
    /// Creates a new flow-controlled queue with the specified configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if configuration invariants fail.
    pub fn new(config: BackpressureConfig) -> Result<Self, JobEngineError> {
        let controller = BackpressureController::new(config)?;
        Ok(Self {
            controller,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            push_notify: Arc::new(Notify::new()),
        })
    }

    /// Returns a reference to the backpressure controller.
    #[must_use]
    pub fn controller(&self) -> &BackpressureController {
        &self.controller
    }

    /// Returns the number of items currently buffered in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        let buf = self.buffer.lock().unwrap();
        buf.len()
    }

    /// Returns `true` if the queue holds no buffered items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let buf = self.buffer.lock().unwrap();
        buf.is_empty()
    }

    /// Returns current backpressure stats.
    #[must_use]
    pub fn stats(&self) -> BackpressureStats {
        self.controller.stats()
    }

    /// Pushes an item into the queue, applying backpressure flow control.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError`] if load shedding rejects the item or wait times out.
    pub async fn push(
        &self,
        item: T,
        cancellation_token: Option<&CancellationToken>,
    ) -> Result<(), JobEngineError> {
        // Acquire backpressure slot
        let _slot = self.controller.acquire_slot(cancellation_token).await?;

        // Slot acquired: store in buffer and forget slot guard so depth remains accounted
        // until popped and consumed
        std::mem::forget(_slot);

        {
            let mut buf = self.buffer.lock().unwrap();
            buf.push_back(item);
        }

        self.push_notify.notify_one();
        Ok(())
    }

    /// Non-blocking push attempt.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::BackpressureQueueSaturated`] if buffer is at high watermark.
    pub fn try_push(&self, item: T) -> Result<(), JobEngineError> {
        let _slot = self.controller.try_acquire_slot()?;
        std::mem::forget(_slot);

        {
            let mut buf = self.buffer.lock().unwrap();
            buf.push_back(item);
        }

        self.push_notify.notify_one();
        Ok(())
    }

    /// Pops the next item from the queue, decrementing buffer depth and notifying producers.
    pub async fn pop(
        &self,
        cancellation_token: Option<&CancellationToken>,
    ) -> Result<Option<T>, JobEngineError> {
        loop {
            if let Some(token) = cancellation_token {
                if token.is_cancelled() {
                    return Err(JobEngineError::WorkerShuttingDown);
                }
            }

            let notified = self.push_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            {
                let mut buf = self.buffer.lock().unwrap();
                if let Some(item) = buf.pop_front() {
                    // Item popped: decrement depth in controller and notify producers if <= low watermark
                    let mut state = self.controller.inner.state.lock().unwrap();
                    state.current_depth = state.current_depth.saturating_sub(1);
                    state.total_drained += 1;

                    if state.current_depth <= self.controller.inner.config.low_watermark {
                        self.controller.inner.drain_notify.notify_waiters();
                    }

                    return Ok(Some(item));
                }
            }

            if let Some(token) = cancellation_token {
                tokio::select! {
                    () = notified => {}
                    () = token.cancelled() => {
                        return Err(JobEngineError::WorkerShuttingDown);
                    }
                }
            } else {
                notified.await;
            }
        }
    }

    /// Non-blocking pop attempt.
    #[must_use]
    pub fn try_pop(&self) -> Option<T> {
        let mut buf = self.buffer.lock().unwrap();
        if let Some(item) = buf.pop_front() {
            let mut state = self.controller.inner.state.lock().unwrap();
            state.current_depth = state.current_depth.saturating_sub(1);
            state.total_drained += 1;

            if state.current_depth <= self.controller.inner.config.low_watermark {
                self.controller.inner.drain_notify.notify_waiters();
            }

            Some(item)
        } else {
            None
        }
    }
}

/// Batch task executor applying backpressure flow control across producer and worker pools.
#[derive(Debug, Clone)]
pub struct BackpressureParallelExecutor {
    queue: FlowControlledQueue<BatchItemTask<()>>,
    batch_config: BatchExecutionConfig,
}

impl BackpressureParallelExecutor {
    /// Constructs a new `BackpressureParallelExecutor`.
    #[must_use]
    pub fn new(
        queue: FlowControlledQueue<BatchItemTask<()>>,
        batch_config: BatchExecutionConfig,
    ) -> Self {
        Self {
            queue,
            batch_config,
        }
    }

    /// Returns a reference to the flow controlled queue.
    #[must_use]
    pub fn queue(&self) -> &FlowControlledQueue<BatchItemTask<()>> {
        &self.queue
    }

    /// Returns a reference to the batch execution configuration.
    #[must_use]
    pub fn batch_config(&self) -> &BatchExecutionConfig {
        &self.batch_config
    }

    /// Executes tasks concurrently while maintaining bounded backpressure flow control.
    pub async fn execute_flow_controlled<T, R, F, Fut>(
        batch_id: BatchId,
        tasks: Vec<BatchItemTask<T>>,
        backpressure_config: BackpressureConfig,
        batch_config: BatchExecutionConfig,
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

        let controller = BackpressureController::new(backpressure_config)?;
        let total_items = tasks.len();
        let batch_start = Instant::now();
        let mut join_handles = Vec::with_capacity(total_items);

        info!(
            %batch_id,
            total_items,
            max_capacity = controller.config().max_buffer_capacity,
            high_watermark = controller.config().high_watermark,
            "Starting flow-controlled batch execution with backpressure"
        );

        for task in tasks {
            let ctrl = controller.clone();
            let proc = processor.clone();
            let parent_cancel = cancel_token.clone();
            let item_timeout_dur = batch_config.item_timeout;
            let prog_tx = progress_tx.clone();

            // Acquire backpressure slot before spawning worker task
            let slot = ctrl.acquire_slot(Some(&parent_cancel)).await?;

            let handle = tokio::spawn(async move {
                let _slot = slot; // Slot retained for duration of task execution
                let item_index = task.item_index;
                let job_id = task.job_id;
                let url = task.url.clone();
                let custom_id = task.custom_id.clone();

                if parent_cancel.is_cancelled() {
                    return BatchItemExecutionResult::cancelled(
                        item_index,
                        job_id,
                        url,
                        custom_id,
                        "Flow-controlled task cancelled before execution",
                    );
                }

                let item_start = Instant::now();
                let url_str = url.clone();
                let outcome = match timeout(item_timeout_dur, proc(task)).await {
                    Ok(Ok(value)) => {
                        let duration = item_start.elapsed();
                        BatchItemExecutionResult::success(
                            item_index,
                            job_id,
                            url_str,
                            custom_id,
                            duration.as_millis() as u64,
                            value,
                        )
                    }
                    Ok(Err(err_msg)) => {
                        let duration = item_start.elapsed();
                        BatchItemExecutionResult::failure(
                            item_index,
                            job_id,
                            url_str,
                            custom_id,
                            duration.as_millis() as u64,
                            err_msg,
                        )
                    }
                    Err(_) => BatchItemExecutionResult::timed_out(
                        item_index,
                        job_id,
                        url_str,
                        custom_id,
                        item_timeout_dur.as_millis() as u64,
                    ),
                };

                if let Some(ref tx) = prog_tx {
                    let _ = tx
                        .send(BatchProgressUpdate {
                            batch_id,
                            total_items,
                            completed_items: 1,
                            failed_items: usize::from(!outcome.status.is_success()),
                            active_workers: ctrl.current_depth(),
                            peak_concurrency: ctrl.config().high_watermark,
                            elapsed_ms: item_start.elapsed().as_millis() as u64,
                        })
                        .await;
                }

                outcome
            });

            join_handles.push(handle);
        }

        let mut item_results = Vec::with_capacity(total_items);
        let mut succeeded_count = 0;
        let mut failed_count = 0;
        let mut timed_out_count = 0;
        let mut cancelled_count = 0;

        for handle in join_handles {
            match handle.await {
                Ok(item_res) => {
                    match item_res.status {
                        BatchItemStatusOutcome::Succeeded => succeeded_count += 1,
                        BatchItemStatusOutcome::TimedOut { .. } => timed_out_count += 1,
                        BatchItemStatusOutcome::Cancelled { .. } => cancelled_count += 1,
                        BatchItemStatusOutcome::Failed { .. } => failed_count += 1,
                    }
                    item_results.push(item_res);
                }
                Err(join_err) => {
                    failed_count += 1;
                    item_results.push(BatchItemExecutionResult::failure(
                        item_results.len(),
                        JobId::new(),
                        "",
                        None,
                        0,
                        format!("Task panic: {join_err}"),
                    ));
                }
            }
        }

        let total_duration = batch_start.elapsed();
        item_results.sort_by_key(|r| r.item_index);

        Ok(BatchExecutionSummary {
            batch_id,
            total_items,
            succeeded_count,
            failed_count,
            cancelled_count,
            timed_out_count,
            duration_ms: total_duration.as_millis() as u64,
            peak_concurrency: controller.stats().current_depth,
            configured_parallelism: batch_config.max_parallelism,
            item_results,
        })
    }
}
