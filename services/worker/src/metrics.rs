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
    }
}
