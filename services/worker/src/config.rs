//! # Worker Service Configuration
//!
//! Strongly-typed configuration options for the pipeline worker daemon,
//! including concurrency limits, polling intervals, leasing parameters, and storage paths.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use web_repo_to_ai_core::error::CoreError;

/// Minimum allowable worker concurrency ceiling.
pub const MIN_WORKER_CONCURRENCY: usize = 1;

/// Maximum allowable worker concurrency ceiling under the 16 GB RAM baseline.
pub const MAX_WORKER_CONCURRENCY: usize = 20;

/// Default worker-wide concurrency ceiling.
pub const DEFAULT_WORKER_CONCURRENCY: usize = 10;

/// Default batch-level parallelism.
pub const DEFAULT_BATCH_PARALLELISM: usize = 5;

/// Default queue polling interval in milliseconds.
pub const DEFAULT_POLL_INTERVAL_MILLIS: u64 = 500;

/// Default worker heartbeat renewal interval in seconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u32 = 15;

/// Default lease holding duration before stale reclamation in seconds.
pub const DEFAULT_LEASE_DURATION_SECONDS: u32 = 60;

/// Default graceful shutdown wait timeout in seconds.
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECONDS: u32 = 30;

/// Comprehensive configuration for the worker service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerServiceConfig {
    /// Unique identity name for this worker instance.
    pub worker_id: String,

    /// Worker-wide ceiling on concurrent executing tasks.
    pub max_concurrent_jobs: usize,

    /// Default parallelism allocated per batch job when unspecified.
    pub default_batch_parallelism: usize,

    /// Interval between job queue polls in milliseconds.
    pub poll_interval_millis: u64,

    /// Interval between heartbeat renewals for leased jobs.
    pub heartbeat_interval_seconds: u32,

    /// Expiration duration for acquired job leases.
    pub lease_duration_seconds: u32,

    /// Maximum duration to wait for in-flight tasks to finish on shutdown.
    pub shutdown_timeout_seconds: u32,

    /// PostgreSQL connection connection string, if configured.
    pub database_url: Option<String>,

    /// Local filesystem directory for storing converted artifacts.
    pub storage_dir: Option<PathBuf>,

    /// Default maximum retry attempts for failing batch items.
    pub default_max_retries: u32,

    /// Backoff delay in milliseconds between retry attempts.
    pub default_retry_backoff_millis: u64,
}

impl Default for WorkerServiceConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4()),
            max_concurrent_jobs: DEFAULT_WORKER_CONCURRENCY,
            default_batch_parallelism: DEFAULT_BATCH_PARALLELISM,
            poll_interval_millis: DEFAULT_POLL_INTERVAL_MILLIS,
            heartbeat_interval_seconds: DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
            lease_duration_seconds: DEFAULT_LEASE_DURATION_SECONDS,
            shutdown_timeout_seconds: DEFAULT_SHUTDOWN_TIMEOUT_SECONDS,
            database_url: None,
            storage_dir: None,
            default_max_retries: 0,
            default_retry_backoff_millis: 500,
        }
    }
}

impl WorkerServiceConfig {
    /// Constructs a validated `WorkerServiceConfig` for a specific worker identity.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if parameters violate worker invariants.
    pub fn for_worker(
        worker_id: impl Into<String>,
        max_concurrent_jobs: usize,
        default_batch_parallelism: usize,
    ) -> Result<Self, CoreError> {
        let config = Self {
            worker_id: worker_id.into(),
            max_concurrent_jobs,
            default_batch_parallelism,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Configures automatic item-level retry limits and backoff duration in milliseconds.
    #[must_use]
    pub fn with_retries(mut self, max_retries: u32, retry_backoff_millis: u64) -> Self {
        self.default_max_retries = max_retries;
        self.default_retry_backoff_millis = retry_backoff_millis;
        self
    }

    /// Returns the polling interval as a `Duration`.
    #[must_use]
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_millis)
    }

    /// Returns the shutdown timeout as a `Duration`.
    #[must_use]
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_seconds as u64)
    }

    /// Validates the configuration invariants.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if any boundary is violated.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.worker_id.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "worker_id".to_string(),
                reason: "worker_id cannot be blank".to_string(),
            });
        }

        if self.max_concurrent_jobs < MIN_WORKER_CONCURRENCY
            || self.max_concurrent_jobs > MAX_WORKER_CONCURRENCY
        {
            return Err(CoreError::Validation {
                field: "max_concurrent_jobs".to_string(),
                reason: format!(
                    "max_concurrent_jobs must be between {MIN_WORKER_CONCURRENCY} and {MAX_WORKER_CONCURRENCY} (got {})",
                    self.max_concurrent_jobs
                ),
            });
        }

        if self.default_batch_parallelism < 1
            || self.default_batch_parallelism > self.max_concurrent_jobs
        {
            return Err(CoreError::Validation {
                field: "default_batch_parallelism".to_string(),
                reason: format!(
                    "default_batch_parallelism ({}) cannot exceed max_concurrent_jobs ({})",
                    self.default_batch_parallelism, self.max_concurrent_jobs
                ),
            });
        }

        if self.poll_interval_millis == 0 {
            return Err(CoreError::Validation {
                field: "poll_interval_millis".to_string(),
                reason: "poll_interval_millis must be >= 1".to_string(),
            });
        }

        if self.heartbeat_interval_seconds == 0 {
            return Err(CoreError::Validation {
                field: "heartbeat_interval_seconds".to_string(),
                reason: "heartbeat_interval_seconds must be >= 1".to_string(),
            });
        }

        if self.lease_duration_seconds <= self.heartbeat_interval_seconds {
            return Err(CoreError::Validation {
                field: "lease_duration_seconds".to_string(),
                reason: format!(
                    "lease_duration_seconds ({}) must be strictly greater than heartbeat_interval_seconds ({})",
                    self.lease_duration_seconds, self.heartbeat_interval_seconds
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_worker_config_validates() {
        let config = WorkerServiceConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_concurrent_jobs, DEFAULT_WORKER_CONCURRENCY);
        assert_eq!(config.default_batch_parallelism, DEFAULT_BATCH_PARALLELISM);
    }

    #[test]
    fn test_custom_worker_config_validation() {
        let config = WorkerServiceConfig::for_worker("worker-prod-1", 16, 8).unwrap();
        assert_eq!(config.worker_id, "worker-prod-1");
        assert_eq!(config.max_concurrent_jobs, 16);
        assert_eq!(config.default_batch_parallelism, 8);
    }

    #[test]
    fn test_invalid_concurrency_rejected() {
        assert!(WorkerServiceConfig::for_worker("w1", 0, 1).is_err());
        assert!(WorkerServiceConfig::for_worker("w1", 21, 5).is_err());
        assert!(WorkerServiceConfig::for_worker("w1", 5, 10).is_err());
    }

    #[test]
    fn test_invalid_lease_duration_rejected() {
        let config = WorkerServiceConfig {
            heartbeat_interval_seconds: 30,
            lease_duration_seconds: 30, // must be > heartbeat
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
