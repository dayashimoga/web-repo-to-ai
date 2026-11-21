//! # Worker Service Telemetry Metrics
//!
//! Provides thread-safe atomic metrics for tracking worker lifecycle events,
//! batch throughput, item success/failure counts, and peak concurrency.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Thread-safe atomic metrics repository for the worker daemon.
#[derive(Debug, Clone, Default)]
pub struct WorkerServiceMetrics {
    batches_processed: Arc<AtomicUsize>,
    items_processed: Arc<AtomicUsize>,
    items_succeeded: Arc<AtomicUsize>,
    items_failed: Arc<AtomicUsize>,
    items_timed_out: Arc<AtomicUsize>,
    items_cancelled: Arc<AtomicUsize>,
    active_jobs: Arc<AtomicUsize>,
    peak_concurrency: Arc<AtomicUsize>,
    incremental_items_reused: Arc<AtomicUsize>,
    incremental_bytes_saved: Arc<AtomicUsize>,
    adaptive_scale_ups: Arc<AtomicUsize>,
    adaptive_backoffs: Arc<AtomicUsize>,
    adaptive_rate_limits: Arc<AtomicUsize>,
    current_adaptive_limit: Arc<AtomicUsize>,
    backpressure_events: Arc<AtomicUsize>,
    backpressure_items_shed: Arc<AtomicUsize>,
    current_queue_depth: Arc<AtomicUsize>,
    streaming_bytes_exported: Arc<AtomicUsize>,
    streaming_exports_completed: Arc<AtomicUsize>,
    resumed_jobs_count: Arc<AtomicUsize>,
    checkpoints_saved_count: Arc<AtomicUsize>,
    items_reused_from_checkpoint: Arc<AtomicUsize>,
    retries_attempted: Arc<AtomicUsize>,
    retries_succeeded: Arc<AtomicUsize>,
}

/// Serializable snapshot of worker metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerMetricsSnapshot {
    /// Total batch submissions completed.
    pub batches_processed: usize,
    /// Total item tasks dispatched.
    pub items_processed: usize,
    /// Total items successfully converted.
    pub items_succeeded: usize,
    /// Total items that failed.
    pub items_failed: usize,
    /// Total items that timed out.
    pub items_timed_out: usize,
    /// Total items that were cancelled.
    pub items_cancelled: usize,
    /// Current count of actively executing jobs.
    pub active_jobs: usize,
    /// Highest concurrency level observed simultaneously.
    pub peak_concurrency: usize,
    /// Total items reused from cache during incremental processing.
    pub incremental_items_reused: usize,
    /// Total bytes saved through incremental reuse.
    pub incremental_bytes_saved: usize,
    /// Total additive scale-ups under adaptive concurrency.
    pub adaptive_scale_ups: usize,
    /// Total multiplicative backoffs under adaptive concurrency.
    pub adaptive_backoffs: usize,
    /// Total rate limit signals observed under adaptive concurrency.
    pub adaptive_rate_limits: usize,
    /// Current dynamically allocated concurrency ceiling.
    pub current_adaptive_limit: usize,
    /// Total backpressure throttle/suspension events.
    pub backpressure_events: usize,
    /// Total items rejected or discarded due to backpressure saturation.
    pub backpressure_items_shed: usize,
    /// Point-in-time queued item depth.
    pub current_queue_depth: usize,
    /// Total uncompressed bytes exported via streaming pipelines.
    pub streaming_bytes_exported: usize,
    /// Total streaming export packages completed.
    pub streaming_exports_completed: usize,
    /// Total jobs resumed from durable checkpoints.
    pub resumed_jobs_count: usize,
    /// Total durable checkpoints persisted during executions.
    pub checkpoints_saved_count: usize,
    /// Total items skipped and reused directly from checkpoints.
    pub items_reused_from_checkpoint: usize,
    /// Total retry attempts executed across batch tasks.
    pub retries_attempted: usize,
    /// Total retry attempts that succeeded.
    pub retries_succeeded: usize,
}

impl WorkerServiceMetrics {
    /// Creates a new metrics repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the count of completed batches.
    pub fn record_batch_completed(&self) {
        self.batches_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an item execution outcome.
    pub fn record_item_outcome(
        &self,
        succeeded: usize,
        failed: usize,
        timed_out: usize,
        cancelled: usize,
    ) {
        self.items_processed.fetch_add(
            succeeded + failed + timed_out + cancelled,
            Ordering::Relaxed,
        );
        self.items_succeeded.fetch_add(succeeded, Ordering::Relaxed);
        self.items_failed.fetch_add(failed, Ordering::Relaxed);
        self.items_timed_out.fetch_add(timed_out, Ordering::Relaxed);
        self.items_cancelled.fetch_add(cancelled, Ordering::Relaxed);
    }

    /// Updates active worker concurrency tracking.
    pub fn update_active_workers(&self, current: usize) {
        self.active_jobs.store(current, Ordering::Relaxed);
        let mut peak = self.peak_concurrency.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_concurrency.compare_exchange_weak(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }

    /// Records incremental cache reuse savings.
    pub fn record_incremental_reuse(&self, reused_count: usize, bytes_saved: u64) {
        self.incremental_items_reused
            .fetch_add(reused_count, Ordering::Relaxed);
        self.incremental_bytes_saved
            .fetch_add(bytes_saved as usize, Ordering::Relaxed);
    }

    /// Records adaptive concurrency events and dynamic limit.
    pub fn record_adaptive_stats(
        &self,
        scale_ups: u64,
        backoffs: u64,
        rate_limits: u64,
        current_limit: usize,
    ) {
        self.adaptive_scale_ups
            .fetch_add(scale_ups as usize, Ordering::Relaxed);
        self.adaptive_backoffs
            .fetch_add(backoffs as usize, Ordering::Relaxed);
        self.adaptive_rate_limits
            .fetch_add(rate_limits as usize, Ordering::Relaxed);
        self.current_adaptive_limit
            .store(current_limit, Ordering::Relaxed);
    }

    /// Records backpressure flow control events and queue depth.
    pub fn record_backpressure_stats(&self, events: u64, items_shed: u64, queue_depth: usize) {
        self.backpressure_events
            .fetch_add(events as usize, Ordering::Relaxed);
        self.backpressure_items_shed
            .fetch_add(items_shed as usize, Ordering::Relaxed);
        self.current_queue_depth
            .store(queue_depth, Ordering::Relaxed);
    }

    /// Records streaming export metrics.
    pub fn record_streaming_export(&self, bytes: u64) {
        self.streaming_bytes_exported
            .fetch_add(bytes as usize, Ordering::Relaxed);
        self.streaming_exports_completed
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Records resumable execution metrics.
    pub fn record_resumable_stats(
        &self,
        resumed: bool,
        checkpoints_saved: usize,
        items_reused: usize,
    ) {
        if resumed {
            self.resumed_jobs_count.fetch_add(1, Ordering::Relaxed);
        }
        self.checkpoints_saved_count
            .fetch_add(checkpoints_saved, Ordering::Relaxed);
        self.items_reused_from_checkpoint
            .fetch_add(items_reused, Ordering::Relaxed);
    }

    /// Records retry attempts and outcomes.
    pub fn record_retry(&self, succeeded: bool) {
        self.retries_attempted.fetch_add(1, Ordering::Relaxed);
        if succeeded {
            self.retries_succeeded.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Produces an immutable metrics snapshot.
    #[must_use]
    pub fn snapshot(&self) -> WorkerMetricsSnapshot {
        WorkerMetricsSnapshot {
            batches_processed: self.batches_processed.load(Ordering::Relaxed),
            items_processed: self.items_processed.load(Ordering::Relaxed),
            items_succeeded: self.items_succeeded.load(Ordering::Relaxed),
            items_failed: self.items_failed.load(Ordering::Relaxed),
            items_timed_out: self.items_timed_out.load(Ordering::Relaxed),
            items_cancelled: self.items_cancelled.load(Ordering::Relaxed),
            active_jobs: self.active_jobs.load(Ordering::Relaxed),
            peak_concurrency: self.peak_concurrency.load(Ordering::Relaxed),
            incremental_items_reused: self.incremental_items_reused.load(Ordering::Relaxed),
            incremental_bytes_saved: self.incremental_bytes_saved.load(Ordering::Relaxed),
            adaptive_scale_ups: self.adaptive_scale_ups.load(Ordering::Relaxed),
            adaptive_backoffs: self.adaptive_backoffs.load(Ordering::Relaxed),
            adaptive_rate_limits: self.adaptive_rate_limits.load(Ordering::Relaxed),
            current_adaptive_limit: self.current_adaptive_limit.load(Ordering::Relaxed),
            backpressure_events: self.backpressure_events.load(Ordering::Relaxed),
            backpressure_items_shed: self.backpressure_items_shed.load(Ordering::Relaxed),
            current_queue_depth: self.current_queue_depth.load(Ordering::Relaxed),
            streaming_bytes_exported: self.streaming_bytes_exported.load(Ordering::Relaxed),
            streaming_exports_completed: self.streaming_exports_completed.load(Ordering::Relaxed),
            resumed_jobs_count: self.resumed_jobs_count.load(Ordering::Relaxed),
            checkpoints_saved_count: self.checkpoints_saved_count.load(Ordering::Relaxed),
            items_reused_from_checkpoint: self.items_reused_from_checkpoint.load(Ordering::Relaxed),
            retries_attempted: self.retries_attempted.load(Ordering::Relaxed),
            retries_succeeded: self.retries_succeeded.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording_and_snapshot() {
        let metrics = WorkerServiceMetrics::new();
        metrics.record_batch_completed();
        metrics.record_item_outcome(4, 1, 1, 0);
        metrics.record_incremental_reuse(5, 10240);
        metrics.record_adaptive_stats(3, 1, 1, 6);
        metrics.record_backpressure_stats(2, 0, 8);
        metrics.record_streaming_export(4096);
        metrics.record_resumable_stats(true, 3, 2);
        metrics.record_retry(true);
        metrics.record_retry(false);
        metrics.update_active_workers(3);

        let snap = metrics.snapshot();
        assert_eq!(snap.batches_processed, 1);
        assert_eq!(snap.items_processed, 6);
        assert_eq!(snap.items_succeeded, 4);
        assert_eq!(snap.items_failed, 1);
        assert_eq!(snap.items_timed_out, 1);
        assert_eq!(snap.items_cancelled, 0);
        assert_eq!(snap.active_jobs, 3);
        assert_eq!(snap.peak_concurrency, 3);
        assert_eq!(snap.incremental_items_reused, 5);
        assert_eq!(snap.incremental_bytes_saved, 10240);
        assert_eq!(snap.adaptive_scale_ups, 3);
        assert_eq!(snap.adaptive_backoffs, 1);
        assert_eq!(snap.adaptive_rate_limits, 1);
        assert_eq!(snap.current_adaptive_limit, 6);
        assert_eq!(snap.backpressure_events, 2);
        assert_eq!(snap.backpressure_items_shed, 0);
        assert_eq!(snap.current_queue_depth, 8);
        assert_eq!(snap.streaming_bytes_exported, 4096);
        assert_eq!(snap.streaming_exports_completed, 1);
        assert_eq!(snap.resumed_jobs_count, 1);
        assert_eq!(snap.checkpoints_saved_count, 3);
        assert_eq!(snap.items_reused_from_checkpoint, 2);
        assert_eq!(snap.retries_attempted, 2);
        assert_eq!(snap.retries_succeeded, 1);
    }
}
