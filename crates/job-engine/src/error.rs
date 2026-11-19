//! # Job Engine Errors
//!
//! Strongly-typed domain errors for job concurrency, batch task execution,
//! timeouts, and worker pool operations.

use thiserror::Error;

/// Specific error conditions encountered during job engine and batch operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JobEngineError {
    /// Concurrency limit was exceeded or an invalid concurrency boundary was supplied.
    #[error("Concurrency limit violation: requested {requested}, maximum permitted is {limit}")]
    ConcurrencyLimitExceeded {
        /// Requested concurrency level.
        requested: usize,
        /// Maximum allowable concurrency limit.
        limit: usize,
    },

    /// An item or batch task exceeded its allocated execution timeout.
    #[error("Task for item '{item_index}' (job '{job_id}') timed out after {duration_ms}ms")]
    TaskTimeout {
        /// Index of the item within the batch.
        item_index: usize,
        /// Associated job identifier.
        job_id: String,
        /// Configured timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// Task was cooperatively cancelled before or during execution.
    #[error("Task for item '{item_index}' (job '{job_id}') cancelled: {reason}")]
    TaskCancelled {
        /// Index of the item within the batch.
        item_index: usize,
        /// Associated job identifier.
        job_id: String,
        /// Human-readable cancellation rationale.
        reason: String,
    },

    /// Task execution failed with an unrecoverable error.
    #[error("Task for item '{item_index}' (job '{job_id}') failed: {reason}")]
    TaskFailed {
        /// Index of the item within the batch.
        item_index: usize,
        /// Associated job identifier.
        job_id: String,
        /// Error description.
        reason: String,
    },

    /// The worker daemon is in the process of shutting down and cannot accept new tasks.
    #[error("Worker daemon is shutting down; operation rejected")]
    WorkerShuttingDown,

    /// Configuration parameters violate domain boundaries or constraints.
    #[error("Invalid job engine configuration: {reason}")]
    InvalidConfiguration {
        /// Explanation of the configuration invariant violation.
        reason: String,
    },

    /// A batch contains no actionable items to execute.
    #[error("Batch submission is empty; at least 1 item is required")]
    EmptyBatch,
}
