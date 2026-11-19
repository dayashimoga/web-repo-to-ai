//! # Pipeline Worker Library (`web-repo-to-ai-worker`)
//!
//! Provides the asynchronous conversion pipeline worker daemon and parallel
//! execution abstractions:
//! - Bounded concurrency worker pools respecting CPU and memory budgets
//! - Multi-URL batch execution with per-item isolation
//! - Telemetry metrics and progress notifications
//! - Graceful shutdown and lease management

#![warn(missing_docs)]

pub mod config;
pub mod engine;
pub mod metrics;

pub use config::{
    WorkerServiceConfig, DEFAULT_BATCH_PARALLELISM, DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
    DEFAULT_LEASE_DURATION_SECONDS, DEFAULT_POLL_INTERVAL_MILLIS, DEFAULT_SHUTDOWN_TIMEOUT_SECONDS,
    DEFAULT_WORKER_CONCURRENCY, MAX_WORKER_CONCURRENCY, MIN_WORKER_CONCURRENCY,
};
pub use engine::WorkerEngine;
pub use metrics::{WorkerMetricsSnapshot, WorkerServiceMetrics};

/// Returns the unique service identifier name.
#[must_use]
pub fn service_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_service_metadata() {
        assert_eq!(service_name(), "web-repo-to-ai-worker");
    }

    #[test]
    fn test_worker_reexports() {
        let config = WorkerServiceConfig::default();
        assert_eq!(config.max_concurrent_jobs, DEFAULT_WORKER_CONCURRENCY);
        let engine = WorkerEngine::new(config);
        assert!(engine.is_ok());
    }
}
