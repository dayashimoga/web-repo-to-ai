//! # Adaptive Concurrency Subsystem
//!
//! Provides dynamic AIMD (Additive Increase, Multiplicative Decrease) concurrency
//! regulation, domain-level rate-limiting isolation, latency-aware throttling,
//! and cooperative permit management for batch execution.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Notify};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use web_repo_to_ai_core::id::{BatchId, JobId};

use crate::config::BatchExecutionConfig;
use crate::error::JobEngineError;
use crate::models::{
    BatchExecutionSummary, BatchItemExecutionResult, BatchItemStatusOutcome, BatchItemTask,
    BatchProgressUpdate,
};

/// Default minimum concurrency ceiling.
pub const DEFAULT_MIN_CONCURRENCY: usize = 1;

/// Default maximum allowable concurrency ceiling.
pub const DEFAULT_MAX_CONCURRENCY: usize = 20;

/// Default initial concurrency.
pub const DEFAULT_INITIAL_CONCURRENCY: usize = 5;

/// Default additive increase step size.
pub const DEFAULT_INCREASE_STEP: usize = 1;

/// Default consecutive successes required before triggering an additive scale-up.
pub const DEFAULT_SUCCESS_THRESHOLD: usize = 3;

/// Default multiplicative decrease factor on throttling or failure signals (0.5 = halve concurrency).
pub const DEFAULT_DECREASE_FACTOR: f64 = 0.5;

/// Default cooldown period following a backoff before concurrency can increase again.
pub const DEFAULT_COOLDOWN_DURATION: Duration = Duration::from_millis(500);

/// Configuration governing adaptive concurrency dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveConcurrencyConfig {
    /// Minimum allowable concurrency limit (floor).
    pub min_concurrency: usize,
    /// Maximum allowable concurrency limit (ceiling).
    pub max_concurrency: usize,
    /// Initial starting concurrency.
    pub initial_concurrency: usize,
    /// Additive increase step added upon sustained success.
    pub increase_step: usize,
    /// Number of consecutive successes required prior to scaling up.
    pub success_threshold: usize,
    /// Multiplicative factor applied during backoff (e.g. 0.5 = 50% reduction).
    pub decrease_factor: f64,
    /// Cooldown period after backoff during which scale-ups are suppressed.
    pub cooldown_duration: Duration,
    /// Optional SLA latency threshold; latencies exceeding this trigger gentle backoff.
    pub latency_sla: Option<Duration>,
}

impl Default for AdaptiveConcurrencyConfig {
    fn default() -> Self {
        Self {
            min_concurrency: DEFAULT_MIN_CONCURRENCY,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            initial_concurrency: DEFAULT_INITIAL_CONCURRENCY,
            increase_step: DEFAULT_INCREASE_STEP,
            success_threshold: DEFAULT_SUCCESS_THRESHOLD,
            decrease_factor: DEFAULT_DECREASE_FACTOR,
            cooldown_duration: DEFAULT_COOLDOWN_DURATION,
            latency_sla: None,
        }
    }
}

impl AdaptiveConcurrencyConfig {
    /// Constructs a validated configuration with specified boundaries.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if invariants are violated.
    pub fn new(
        min_concurrency: usize,
        max_concurrency: usize,
        initial_concurrency: usize,
    ) -> Result<Self, JobEngineError> {
        let config = Self {
            min_concurrency,
            max_concurrency,
            initial_concurrency,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Builder to configure the additive increase step.
    #[must_use]
    pub fn with_increase_step(mut self, step: usize) -> Self {
        self.increase_step = step;
        self
    }

    /// Builder to configure the success threshold before scaling up.
    #[must_use]
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Builder to configure the multiplicative decrease factor.
    #[must_use]
    pub fn with_decrease_factor(mut self, factor: f64) -> Self {
        self.decrease_factor = factor;
        self
    }

    /// Builder to configure cooldown duration.
    #[must_use]
    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown_duration = cooldown;
        self
    }

    /// Builder to configure optional latency SLA threshold.
    #[must_use]
    pub fn with_latency_sla(mut self, sla: Duration) -> Self {
        self.latency_sla = Some(sla);
        self
    }

    /// Validates configuration parameters.
    pub fn validate(&self) -> Result<(), JobEngineError> {
        if self.min_concurrency == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "min_concurrency must be at least 1".to_string(),
            });
        }
        if self.max_concurrency < self.min_concurrency {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "max_concurrency ({}) cannot be less than min_concurrency ({})",
                    self.max_concurrency, self.min_concurrency
                ),
            });
        }
        if self.initial_concurrency < self.min_concurrency
            || self.initial_concurrency > self.max_concurrency
        {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "initial_concurrency ({}) must be between min ({}) and max ({})",
                    self.initial_concurrency, self.min_concurrency, self.max_concurrency
                ),
            });
        }
        if self.increase_step == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "increase_step must be at least 1".to_string(),
            });
        }
        if self.success_threshold == 0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "success_threshold must be at least 1".to_string(),
            });
        }
        if self.decrease_factor <= 0.0 || self.decrease_factor >= 1.0 {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "decrease_factor ({}) must be strictly between 0.0 and 1.0",
                    self.decrease_factor
                ),
            });
        }
        Ok(())
    }
}

/// Observed outcome of an individual execution unit used to guide concurrency adjustment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptiveOutcome {
    /// Task succeeded within expected performance envelope.
    Success {
        /// Time taken to execute the task.
        duration: Duration,
    },
    /// Target host or rate limiter explicitly rejected request (HTTP 429).
    RateLimited {
        /// Optional retry-after recommendation from upstream server.
        retry_after: Option<Duration>,
    },
    /// Upstream server is overloaded or unavailable (HTTP 503 / 504).
    ServerOverloaded,
    /// Task execution exceeded allocated timeout budget.
    Timeout,
    /// Low-level network or connection failure.
    ConnectionError,
    /// Other task failure.
    OtherFailure,
}

impl AdaptiveOutcome {
    /// Automatically infers an `AdaptiveOutcome` from a standard `Result<R, E>`.
    pub fn from_result<R, E: std::fmt::Display>(result: &Result<R, E>, duration: Duration) -> Self {
        match result {
            Ok(_) => Self::Success { duration },
            Err(err) => {
                let err_str = err.to_string().to_lowercase();
                if err_str.contains("429")
                    || err_str.contains("rate limit")
                    || err_str.contains("too many requests")
                {
                    Self::RateLimited { retry_after: None }
                } else if err_str.contains("503")
                    || err_str.contains("504")
                    || err_str.contains("overload")
                    || err_str.contains("service unavailable")
                {
                    Self::ServerOverloaded
                } else if err_str.contains("timeout") || err_str.contains("timed out") {
                    Self::Timeout
                } else if err_str.contains("connection")
                    || err_str.contains("dns")
                    || err_str.contains("network")
                    || err_str.contains("reset")
                {
                    Self::ConnectionError
                } else {
                    Self::OtherFailure
                }
            }
        }
    }
}

/// Decision made by the adaptive controller upon receiving an outcome signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptiveDecision {
    /// Concurrency remains unchanged.
    Unchanged {
        /// Active concurrency permit ceiling.
        current_limit: usize,
    },
    /// Concurrency was scaled up additively.
    Increased {
        /// Limit before scaling up.
        old_limit: usize,
        /// New higher limit.
        new_limit: usize,
    },
    /// Concurrency was scaled down multiplicatively.
    Decreased {
        /// Limit before backoff.
        old_limit: usize,
        /// New reduced limit.
        new_limit: usize,
        /// Reason for backoff.
        reason: &'static str,
    },
    /// Scale-up was suppressed because the controller is currently in cooldown.
    InCooldown {
        /// Active concurrency permit ceiling.
        current_limit: usize,
        /// Remaining cooldown time.
        remaining: Duration,
    },
}

/// Detailed snapshot of telemetry and performance stats for an adaptive controller.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveConcurrencyStats {
    /// Current dynamically allocated concurrency ceiling.
    pub current_limit: usize,
    /// Minimum allowed limit.
    pub min_limit: usize,
    /// Maximum allowed limit.
    pub max_limit: usize,
    /// Number of permits currently in active execution.
    pub active_permits: usize,
    /// Cumulative count of successful task executions.
    pub total_successes: u64,
    /// Cumulative count of rate limit signals observed.
    pub total_rate_limits: u64,
    /// Cumulative count of server overload signals observed.
    pub total_server_overloads: u64,
    /// Cumulative count of timeout signals observed.
    pub total_timeouts: u64,
    /// Cumulative count of other failures observed.
    pub total_other_failures: u64,
    /// Cumulative count of upward concurrency adjustments.
    pub total_scale_ups: u64,
    /// Cumulative count of downward backoff adjustments.
    pub total_backoffs: u64,
    /// Current consecutive successes observed without intervening failure.
    pub consecutive_successes: usize,
    /// Current consecutive failures observed.
    pub consecutive_failures: usize,
    /// Exponentially weighted or moving average latency in milliseconds.
    pub average_latency_ms: f64,
}

#[derive(Debug)]
struct AdaptiveState {
    current_limit: usize,
    active_permits: usize,
    total_successes: u64,
    total_rate_limits: u64,
    total_server_overloads: u64,
    total_timeouts: u64,
    total_other_failures: u64,
    total_scale_ups: u64,
    total_backoffs: u64,
    consecutive_successes: usize,
    consecutive_failures: usize,
    total_latency_ms: f64,
    last_backoff: Option<Instant>,
}

#[derive(Debug)]
struct AdaptiveConcurrencyInner {
    config: AdaptiveConcurrencyConfig,
    state: Mutex<AdaptiveState>,
    notify: Notify,
}

/// Dynamic concurrency controller implementing Additive Increase / Multiplicative Decrease.
#[derive(Debug, Clone)]
pub struct AdaptiveConcurrencyController {
    inner: Arc<AdaptiveConcurrencyInner>,
}

impl AdaptiveConcurrencyController {
    /// Creates a new `AdaptiveConcurrencyController` from configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if configuration invariants fail.
    pub fn new(config: AdaptiveConcurrencyConfig) -> Result<Self, JobEngineError> {
        config.validate()?;
        let initial_limit = config.initial_concurrency;
        Ok(Self {
            inner: Arc::new(AdaptiveConcurrencyInner {
                config,
                state: Mutex::new(AdaptiveState {
                    current_limit: initial_limit,
                    active_permits: 0,
                    total_successes: 0,
                    total_rate_limits: 0,
                    total_server_overloads: 0,
                    total_timeouts: 0,
                    total_other_failures: 0,
                    total_scale_ups: 0,
                    total_backoffs: 0,
                    consecutive_successes: 0,
                    consecutive_failures: 0,
                    total_latency_ms: 0.0,
                    last_backoff: None,
                }),
                notify: Notify::new(),
            }),
        })
    }

    /// Returns a reference to the configuration.
    #[must_use]
    pub fn config(&self) -> &AdaptiveConcurrencyConfig {
        &self.inner.config
    }

    /// Returns the current dynamic concurrency limit ceiling.
    #[must_use]
    pub fn current_limit(&self) -> usize {
        let state = self.inner.state.lock().unwrap();
        state.current_limit
    }

    /// Returns the number of currently active in-flight permits.
    #[must_use]
    pub fn active_permits(&self) -> usize {
        let state = self.inner.state.lock().unwrap();
        state.active_permits
    }

    /// Acquires an execution permit, waiting asynchronously if active permits reach current limit.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::WorkerShuttingDown`] or [`JobEngineError::TaskCancelled`]
    /// if `cancellation_token` is cancelled while waiting.
    pub async fn acquire_permit(
        &self,
        cancellation_token: Option<&CancellationToken>,
    ) -> Result<AdaptivePermit, JobEngineError> {
        loop {
            if let Some(token) = cancellation_token {
                if token.is_cancelled() {
                    return Err(JobEngineError::WorkerShuttingDown);
                }
            }

            let notified = self.inner.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            {
                let mut state = self.inner.state.lock().unwrap();
                if state.active_permits < state.current_limit {
                    state.active_permits += 1;
                    return Ok(AdaptivePermit {
                        inner: Arc::clone(&self.inner),
                    });
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

    /// Evaluates an outcome signal and adjusts concurrency accordingly.
    pub fn record_outcome(&self, outcome: AdaptiveOutcome) -> AdaptiveDecision {
        let mut state = self.inner.state.lock().unwrap();
        let now = Instant::now();

        match outcome {
            AdaptiveOutcome::Success { duration } => {
                state.total_successes += 1;
                state.consecutive_successes += 1;
                state.consecutive_failures = 0;
                state.total_latency_ms += duration.as_secs_f64() * 1000.0;

                // Check SLA breach
                if let Some(sla) = self.inner.config.latency_sla {
                    if duration > sla {
                        // Latency SLA breached: do not scale up, consider gentle backoff
                        return AdaptiveDecision::Unchanged {
                            current_limit: state.current_limit,
                        };
                    }
                }

                // Check cooldown window
                if let Some(last_backoff) = state.last_backoff {
                    let elapsed = now.saturating_duration_since(last_backoff);
                    if elapsed < self.inner.config.cooldown_duration {
                        let remaining = self.inner.config.cooldown_duration.saturating_sub(elapsed);
                        return AdaptiveDecision::InCooldown {
                            current_limit: state.current_limit,
                            remaining,
                        };
                    }
                }

                // Check consecutive success threshold
                if state.consecutive_successes >= self.inner.config.success_threshold {
                    state.consecutive_successes = 0;
                    if state.current_limit < self.inner.config.max_concurrency {
                        let old_limit = state.current_limit;
                        let new_limit = (old_limit + self.inner.config.increase_step)
                            .min(self.inner.config.max_concurrency);
                        state.current_limit = new_limit;
                        state.total_scale_ups += 1;
                        self.inner.notify.notify_waiters();
                        info!(
                            old_limit,
                            new_limit, "Adaptive concurrency increased additively"
                        );
                        return AdaptiveDecision::Increased {
                            old_limit,
                            new_limit,
                        };
                    }
                }

                AdaptiveDecision::Unchanged {
                    current_limit: state.current_limit,
                }
            }
            AdaptiveOutcome::RateLimited { .. } => {
                state.total_rate_limits += 1;
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;
                self.apply_multiplicative_decrease(&mut state, now, "rate limit (HTTP 429)")
            }
            AdaptiveOutcome::ServerOverloaded => {
                state.total_server_overloads += 1;
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;
                self.apply_multiplicative_decrease(
                    &mut state,
                    now,
                    "server overload (HTTP 503/504)",
                )
            }
            AdaptiveOutcome::Timeout => {
                state.total_timeouts += 1;
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;
                self.apply_multiplicative_decrease(&mut state, now, "task timeout")
            }
            AdaptiveOutcome::ConnectionError => {
                state.total_other_failures += 1;
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;
                self.apply_multiplicative_decrease(&mut state, now, "connection/network failure")
            }
            AdaptiveOutcome::OtherFailure => {
                state.total_other_failures += 1;
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;
                // For non-overload errors, apply backoff only if consecutive failures exceed threshold
                if state.consecutive_failures >= 3 {
                    self.apply_multiplicative_decrease(
                        &mut state,
                        now,
                        "sustained consecutive errors",
                    )
                } else {
                    AdaptiveDecision::Unchanged {
                        current_limit: state.current_limit,
                    }
                }
            }
        }
    }

    fn apply_multiplicative_decrease(
        &self,
        state: &mut AdaptiveState,
        now: Instant,
        reason: &'static str,
    ) -> AdaptiveDecision {
        let old_limit = state.current_limit;
        let scaled = (old_limit as f64 * self.inner.config.decrease_factor).floor() as usize;
        let new_limit = scaled.max(self.inner.config.min_concurrency);

        state.current_limit = new_limit;
        state.last_backoff = Some(now);
        state.total_backoffs += 1;

        warn!(
            old_limit,
            new_limit, reason, "Adaptive concurrency backed off multiplicatively"
        );

        AdaptiveDecision::Decreased {
            old_limit,
            new_limit,
            reason,
        }
    }

    /// Captures a point-in-time statistics snapshot.
    #[must_use]
    pub fn stats(&self) -> AdaptiveConcurrencyStats {
        let state = self.inner.state.lock().unwrap();
        let avg_latency = if state.total_successes > 0 {
            state.total_latency_ms / state.total_successes as f64
        } else {
            0.0
        };

        AdaptiveConcurrencyStats {
            current_limit: state.current_limit,
            min_limit: self.inner.config.min_concurrency,
            max_limit: self.inner.config.max_concurrency,
            active_permits: state.active_permits,
            total_successes: state.total_successes,
            total_rate_limits: state.total_rate_limits,
            total_server_overloads: state.total_server_overloads,
            total_timeouts: state.total_timeouts,
            total_other_failures: state.total_other_failures,
            total_scale_ups: state.total_scale_ups,
            total_backoffs: state.total_backoffs,
            consecutive_successes: state.consecutive_successes,
            consecutive_failures: state.consecutive_failures,
            average_latency_ms: avg_latency,
        }
    }
}

/// RAII execution permit that releases capacity upon drop.
#[derive(Debug)]
pub struct AdaptivePermit {
    inner: Arc<AdaptiveConcurrencyInner>,
}

impl Drop for AdaptivePermit {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().unwrap();
        state.active_permits = state.active_permits.saturating_sub(1);
        self.inner.notify.notify_waiters();
    }
}

/// Registry providing domain-isolated adaptive concurrency controllers.
///
/// Prevents a throttling or failing remote origin from starving or constraining
/// concurrent processing directed at other healthy hosts.
#[derive(Debug, Clone)]
pub struct DomainAdaptiveConcurrencyManager {
    default_config: AdaptiveConcurrencyConfig,
    domains: Arc<Mutex<HashMap<String, Arc<AdaptiveConcurrencyController>>>>,
}

impl DomainAdaptiveConcurrencyManager {
    /// Creates a new manager with standard default configuration.
    #[must_use]
    pub fn new(default_config: AdaptiveConcurrencyConfig) -> Self {
        Self {
            default_config,
            domains: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Retrieves or instantiates the controller for the specified domain host.
    pub fn get_or_create(&self, domain: &str) -> Arc<AdaptiveConcurrencyController> {
        let normalized = domain.trim().to_lowercase();
        let mut domains = self.domains.lock().unwrap();
        domains
            .entry(normalized)
            .or_insert_with(|| {
                Arc::new(
                    AdaptiveConcurrencyController::new(self.default_config.clone())
                        .expect("Default config is valid"),
                )
            })
            .clone()
    }

    /// Records an execution outcome for a specific domain.
    pub fn record_outcome(&self, domain: &str, outcome: AdaptiveOutcome) -> AdaptiveDecision {
        let controller = self.get_or_create(domain);
        controller.record_outcome(outcome)
    }

    /// Collects a snapshot of stats across all tracked domains.
    #[must_use]
    pub fn domains_stats(&self) -> HashMap<String, AdaptiveConcurrencyStats> {
        let domains = self.domains.lock().unwrap();
        domains
            .iter()
            .map(|(domain, controller)| (domain.clone(), controller.stats()))
            .collect()
    }
}

/// High-performance batch executor that leverages adaptive concurrency regulation.
#[derive(Debug, Clone)]
pub struct AdaptiveParallelExecutor {
    controller: AdaptiveConcurrencyController,
    batch_config: BatchExecutionConfig,
}

impl AdaptiveParallelExecutor {
    /// Constructs a new `AdaptiveParallelExecutor`.
    #[must_use]
    pub fn new(
        controller: AdaptiveConcurrencyController,
        batch_config: BatchExecutionConfig,
    ) -> Self {
        Self {
            controller,
            batch_config,
        }
    }

    /// Returns a reference to the underlying adaptive controller.
    #[must_use]
    pub fn controller(&self) -> &AdaptiveConcurrencyController {
        &self.controller
    }

    /// Executes tasks concurrently while dynamically adapting concurrency to task outcomes.
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
        let batch_start = Instant::now();
        let mut join_handles = Vec::with_capacity(total_items);

        info!(
            %batch_id,
            total_items,
            initial_limit = self.controller.current_limit(),
            "Starting adaptive parallel batch execution"
        );

        for task in tasks {
            let controller = self.controller.clone();
            let proc = processor.clone();
            let parent_cancel = cancel_token.clone();
            let item_timeout_dur = self.batch_config.item_timeout;
            let prog_tx = progress_tx.clone();

            let handle = tokio::spawn(async move {
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
                        "Batch execution cancelled before start",
                    );
                }

                // Acquire adaptive permit
                let _permit = match controller.acquire_permit(Some(&parent_cancel)).await {
                    Ok(p) => p,
                    Err(_) => {
                        return BatchItemExecutionResult::cancelled(
                            item_index,
                            job_id,
                            url,
                            custom_id,
                            "Cancelled while awaiting adaptive permit",
                        );
                    }
                };

                let item_start = Instant::now();
                let url_str = url.clone();
                let outcome = match timeout(item_timeout_dur, proc(task)).await {
                    Ok(Ok(value)) => {
                        let duration = item_start.elapsed();
                        controller.record_outcome(AdaptiveOutcome::Success { duration });
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
                        let outcome =
                            AdaptiveOutcome::from_result::<(), _>(&Err(&err_msg), duration);
                        controller.record_outcome(outcome);
                        BatchItemExecutionResult::failure(
                            item_index,
                            job_id,
                            url_str,
                            custom_id,
                            duration.as_millis() as u64,
                            err_msg,
                        )
                    }
                    Err(_) => {
                        controller.record_outcome(AdaptiveOutcome::Timeout);
                        BatchItemExecutionResult::timed_out(
                            item_index,
                            job_id,
                            url_str,
                            custom_id,
                            item_timeout_dur.as_millis() as u64,
                        )
                    }
                };

                if let Some(ref tx) = prog_tx {
                    let _ = tx
                        .send(BatchProgressUpdate {
                            batch_id,
                            total_items,
                            completed_items: 1,
                            failed_items: usize::from(!outcome.status.is_success()),
                            active_workers: controller.active_permits(),
                            peak_concurrency: controller.current_limit(),
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
            peak_concurrency: self.controller.current_limit(),
            configured_parallelism: self.controller.config().max_concurrency,
            item_results,
        })
    }
}
