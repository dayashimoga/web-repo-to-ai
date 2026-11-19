//! # Job Engine (`web-repo-to-ai-job-engine`)
//!
//! Provides durable job execution, bounded concurrency pools, and parallel batch task orchestration:
//! - Strictly bounded asynchronous parallel execution using Tokio semaphores
//! - Cooperative cancellation propagation via `CancellationToken`
//! - Resilient per-item timeout handling and error isolation
//! - Item ordering preservation and deterministic execution summaries
//! - Realtime batch progress tracking and telemetry

#![warn(missing_docs)]

pub mod adaptive;
pub mod backpressure;
pub mod config;
pub mod error;
pub mod executor;
pub mod incremental;
pub mod models;
pub mod pool;
pub mod resumable;

// Re-export primary types for ergonomic crate-level access
pub use adaptive::{
    AdaptiveConcurrencyConfig, AdaptiveConcurrencyController, AdaptiveConcurrencyStats,
    AdaptiveDecision, AdaptiveOutcome, AdaptiveParallelExecutor, AdaptivePermit,
    DomainAdaptiveConcurrencyManager, DEFAULT_COOLDOWN_DURATION, DEFAULT_DECREASE_FACTOR,
    DEFAULT_INCREASE_STEP, DEFAULT_INITIAL_CONCURRENCY, DEFAULT_MAX_CONCURRENCY,
    DEFAULT_MIN_CONCURRENCY, DEFAULT_SUCCESS_THRESHOLD,
};
pub use backpressure::{
    BackpressureConfig, BackpressureController, BackpressureParallelExecutor, BackpressureSlot,
    BackpressureState, BackpressureStats, FlowControlledQueue, LoadSheddingStrategy,
    DEFAULT_HIGH_WATERMARK_RATIO, DEFAULT_LOW_WATERMARK_RATIO, DEFAULT_MAX_BUFFER_CAPACITY,
    DEFAULT_MAX_WAIT_TIMEOUT,
};
pub use config::{
    BatchExecutionConfig, DEFAULT_BATCH_TIMEOUT_SECONDS, DEFAULT_ITEM_TIMEOUT_SECONDS,
    DEFAULT_PARALLELISM, MAX_PARALLELISM, MIN_PARALLELISM,
};
pub use error::JobEngineError;
pub use executor::BoundedParallelExecutor;
pub use incremental::{
    ChangeCategory, ChangedFileDiff, ClassifiedChange, FileSignature, IncrementalCandidate,
    IncrementalExecutionReport, IncrementalJobCoordinator, ReusedItem,
};
pub use models::{
    BatchExecutionSummary, BatchItemExecutionResult, BatchItemStatusOutcome, BatchItemTask,
    BatchProgressUpdate,
};
pub use pool::{
    WorkerPool, WorkerPoolConfig, WorkerPoolMetrics, DEFAULT_MAX_WORKER_CONCURRENCY,
    DEFAULT_SHUTDOWN_TIMEOUT_SECONDS,
};
pub use resumable::{
    CheckpointItemResult, CheckpointStore, InMemoryCheckpointStore, JobCheckpoint, ResumableConfig,
    ResumableExecutionPlan, ResumableParallelExecutor, ResumableStats,
    DEFAULT_CHECKPOINT_INTERVAL_ITEMS, DEFAULT_MAX_RESUMPTION_ATTEMPTS,
};

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_engine_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-job-engine");
    }

    #[test]
    fn test_crate_reexports() {
        let config = BatchExecutionConfig::default();
        assert_eq!(config.max_parallelism, DEFAULT_PARALLELISM);
        let error = JobEngineError::EmptyBatch;
        assert_eq!(
            error.to_string(),
            "Batch submission is empty; at least 1 item is required"
        );
    }
}
