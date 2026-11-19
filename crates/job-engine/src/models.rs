//! # Batch Execution Domain Models
//!
//! Strongly-typed models representing individual batch item tasks, execution outcomes,
//! runtime progress updates, and consolidated batch summaries.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::id::{BatchId, JobId};

/// Encapsulates an individual task within a parallel batch run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemTask<T> {
    /// Zero-based index of this item within the batch submission sequence.
    pub item_index: usize,

    /// Unique job identifier assigned to this batch sub-task.
    pub job_id: JobId,

    /// Target resource URL to be processed.
    pub url: String,

    /// Optional client-provided correlation identifier.
    pub custom_id: Option<String>,

    /// Task-specific payload or configuration options.
    pub payload: T,
}

impl<T> BatchItemTask<T> {
    /// Constructs a new `BatchItemTask`.
    pub fn new(
        item_index: usize,
        job_id: JobId,
        url: impl Into<String>,
        custom_id: Option<String>,
        payload: T,
    ) -> Self {
        Self {
            item_index,
            job_id,
            url: url.into(),
            custom_id,
            payload,
        }
    }
}

/// Categorical outcome status of a batch item execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BatchItemStatusOutcome {
    /// The task completed successfully.
    Succeeded,

    /// The task failed with an error.
    Failed {
        /// Human-readable explanation of the failure.
        error: String,
    },

    /// The task exceeded its allocated execution timeout.
    TimedOut {
        /// Duration in milliseconds before timeout occurred.
        duration_ms: u64,
    },

    /// The task was cancelled cooperatively before completion.
    Cancelled {
        /// Reason for cancellation.
        reason: String,
    },
}

impl BatchItemStatusOutcome {
    /// Returns `true` if the status represents a successful outcome.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Succeeded)
    }

    /// Returns `true` if the status represents any form of failure or incompletion.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

/// Consolidated execution result for a single batch item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemExecutionResult<R> {
    /// Zero-based sequence index.
    pub item_index: usize,

    /// Unique sub-job identifier.
    pub job_id: JobId,

    /// Target URL.
    pub url: String,

    /// Optional client-supplied correlation ID.
    pub custom_id: Option<String>,

    /// Execution duration in milliseconds.
    pub duration_ms: u64,

    /// High-level execution status.
    pub status: BatchItemStatusOutcome,

    /// Output result on success.
    pub value: Option<R>,

    /// Error message on failure, if any.
    pub error: Option<String>,
}

impl<R> BatchItemExecutionResult<R> {
    /// Constructs a successful item result.
    pub fn success(
        item_index: usize,
        job_id: JobId,
        url: impl Into<String>,
        custom_id: Option<String>,
        duration_ms: u64,
        value: R,
    ) -> Self {
        Self {
            item_index,
            job_id,
            url: url.into(),
            custom_id,
            duration_ms,
            status: BatchItemStatusOutcome::Succeeded,
            value: Some(value),
            error: None,
        }
    }

    /// Constructs a failed item result.
    pub fn failure(
        item_index: usize,
        job_id: JobId,
        url: impl Into<String>,
        custom_id: Option<String>,
        duration_ms: u64,
        error: impl Into<String>,
    ) -> Self {
        let err_str = error.into();
        Self {
            item_index,
            job_id,
            url: url.into(),
            custom_id,
            duration_ms,
            status: BatchItemStatusOutcome::Failed {
                error: err_str.clone(),
            },
            value: None,
            error: Some(err_str),
        }
    }

    /// Constructs a timed-out item result.
    pub fn timed_out(
        item_index: usize,
        job_id: JobId,
        url: impl Into<String>,
        custom_id: Option<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            item_index,
            job_id,
            url: url.into(),
            custom_id,
            duration_ms,
            status: BatchItemStatusOutcome::TimedOut { duration_ms },
            value: None,
            error: Some(format!("Execution timed out after {duration_ms}ms")),
        }
    }

    /// Constructs a cancelled item result.
    pub fn cancelled(
        item_index: usize,
        job_id: JobId,
        url: impl Into<String>,
        custom_id: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        let reason_str = reason.into();
        Self {
            item_index,
            job_id,
            url: url.into(),
            custom_id,
            duration_ms: 0,
            status: BatchItemStatusOutcome::Cancelled {
                reason: reason_str.clone(),
            },
            value: None,
            error: Some(reason_str),
        }
    }

    /// Returns `true` if this item succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

/// Realtime progress notification emitted during parallel batch execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProgressUpdate {
    /// Identifier of the batch being processed.
    pub batch_id: BatchId,

    /// Total number of items in the batch.
    pub total_items: usize,

    /// Cumulative count of items that completed successfully.
    pub completed_items: usize,

    /// Cumulative count of items that failed or timed out.
    pub failed_items: usize,

    /// Number of concurrent tasks actively executing at this moment.
    pub active_workers: usize,

    /// Highest concurrent task count observed so far.
    pub peak_concurrency: usize,

    /// Elapsed execution time since batch start in milliseconds.
    pub elapsed_ms: u64,
}

impl BatchProgressUpdate {
    /// Computes percentage of completed (succeeded + failed) items.
    #[must_use]
    pub fn progress_pct(&self) -> u8 {
        if self.total_items == 0 {
            return 100;
        }
        let finished = self.completed_items + self.failed_items;
        ((finished as f64 / self.total_items as f64) * 100.0).min(100.0) as u8
    }
}

/// Comprehensive summary report returned upon completion of a batch run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExecutionSummary<R> {
    /// Batch identifier.
    pub batch_id: BatchId,

    /// Total count of items submitted in the batch.
    pub total_items: usize,

    /// Count of items that succeeded.
    pub succeeded_count: usize,

    /// Count of items that failed.
    pub failed_count: usize,

    /// Count of items that were cancelled.
    pub cancelled_count: usize,

    /// Count of items that timed out.
    pub timed_out_count: usize,

    /// Total wall-clock execution duration in milliseconds.
    pub duration_ms: u64,

    /// Maximum concurrent worker permits simultaneously in flight.
    pub peak_concurrency: usize,

    /// Maximum configured parallelism ceiling for this batch.
    pub configured_parallelism: usize,

    /// Detailed execution results for each item, preserving sequence order.
    pub item_results: Vec<BatchItemExecutionResult<R>>,
}

impl<R> BatchExecutionSummary<R> {
    /// Returns `true` if all items in the batch completed successfully.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.total_items > 0 && self.succeeded_count == self.total_items
    }

    /// Returns `true` if one or more items failed, timed out, or were cancelled.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.failed_count > 0 || self.timed_out_count > 0 || self.cancelled_count > 0
    }

    /// Computes the success rate as a float between 0.0 and 1.0.
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            self.succeeded_count as f64 / self.total_items as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_item_status_predicates() {
        assert!(BatchItemStatusOutcome::Succeeded.is_success());
        assert!(!BatchItemStatusOutcome::Succeeded.is_failure());

        let failed = BatchItemStatusOutcome::Failed {
            error: "bad url".to_string(),
        };
        assert!(!failed.is_success());
        assert!(failed.is_failure());
    }

    #[test]
    fn test_batch_progress_calculation() {
        let update = BatchProgressUpdate {
            batch_id: BatchId::new(),
            total_items: 10,
            completed_items: 4,
            failed_items: 1,
            active_workers: 2,
            peak_concurrency: 5,
            elapsed_ms: 1200,
        };
        assert_eq!(update.progress_pct(), 50);
    }

    #[test]
    fn test_batch_execution_summary_metrics() {
        let summary: BatchExecutionSummary<()> = BatchExecutionSummary {
            batch_id: BatchId::new(),
            total_items: 4,
            succeeded_count: 3,
            failed_count: 1,
            cancelled_count: 0,
            timed_out_count: 0,
            duration_ms: 350,
            peak_concurrency: 3,
            configured_parallelism: 5,
            item_results: vec![],
        };
        assert!(!summary.all_succeeded());
        assert!(summary.has_failures());
        assert!((summary.success_rate() - 0.75).abs() < f64::EPSILON);
    }
}
