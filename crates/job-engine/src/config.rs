//! # Batch Execution Configuration
//!
//! Provides configuration models and validation rules for bounded parallel
//! batch task execution within worker nodes.

use crate::error::JobEngineError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Minimum allowable parallel concurrency per batch.
pub const MIN_PARALLELISM: usize = 1;

/// Maximum allowable parallel concurrency per batch to guard CPU and memory.
pub const MAX_PARALLELISM: usize = 20;

/// Default parallel concurrency per batch.
pub const DEFAULT_PARALLELISM: usize = 5;

/// Default per-item execution timeout in seconds (1 minute).
pub const DEFAULT_ITEM_TIMEOUT_SECONDS: u64 = 60;

/// Default cumulative batch execution timeout in seconds (10 minutes).
pub const DEFAULT_BATCH_TIMEOUT_SECONDS: u64 = 600;

/// Configuration governing bounded parallel batch execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExecutionConfig {
    /// Maximum concurrent worker permits allowed during batch execution.
    pub max_parallelism: usize,

    /// Execution timeout applied to each individual batch item task.
    pub item_timeout: Duration,

    /// Cumulative timeout applied across the entire batch processing lifecycle.
    pub batch_timeout: Duration,

    /// Whether to abort remaining queued items immediately upon encountering a failure.
    pub abort_on_first_error: bool,

    /// Maximum number of automatic retries permitted per failing item (0 = no retry).
    pub max_retries: u32,

    /// Backoff delay applied between item retry attempts.
    pub retry_backoff: Duration,
}

impl Default for BatchExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallelism: DEFAULT_PARALLELISM,
            item_timeout: Duration::from_secs(DEFAULT_ITEM_TIMEOUT_SECONDS),
            batch_timeout: Duration::from_secs(DEFAULT_BATCH_TIMEOUT_SECONDS),
            abort_on_first_error: false,
            max_retries: 0,
            retry_backoff: Duration::from_millis(500),
        }
    }
}

impl BatchExecutionConfig {
    /// Creates a new validated `BatchExecutionConfig` with the requested parallelism.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if `max_parallelism` is outside `1..=20`.
    pub fn with_parallelism(max_parallelism: usize) -> Result<Self, JobEngineError> {
        let config = Self {
            max_parallelism,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Builder method to customize per-item timeout.
    #[must_use]
    pub fn with_item_timeout(mut self, timeout: Duration) -> Self {
        self.item_timeout = timeout;
        self
    }

    /// Builder method to customize cumulative batch timeout.
    #[must_use]
    pub fn with_batch_timeout(mut self, timeout: Duration) -> Self {
        self.batch_timeout = timeout;
        self
    }

    /// Builder method to configure abort on first error behavior.
    #[must_use]
    pub fn with_abort_on_first_error(mut self, abort: bool) -> Self {
        self.abort_on_first_error = abort;
        self
    }

    /// Builder method to configure retries and backoff.
    #[must_use]
    pub fn with_retries(mut self, max_retries: u32, retry_backoff: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_backoff = retry_backoff;
        self
    }

    /// Validates the configuration invariants.
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError::InvalidConfiguration`] if invariants are violated.
    pub fn validate(&self) -> Result<(), JobEngineError> {
        if self.max_parallelism < MIN_PARALLELISM || self.max_parallelism > MAX_PARALLELISM {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "max_parallelism must be between {MIN_PARALLELISM} and {MAX_PARALLELISM} (got {})",
                    self.max_parallelism
                ),
            });
        }

        if self.item_timeout.is_zero() {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "item_timeout cannot be zero".to_string(),
            });
        }

        if self.batch_timeout.is_zero() {
            return Err(JobEngineError::InvalidConfiguration {
                reason: "batch_timeout cannot be zero".to_string(),
            });
        }

        if self.item_timeout > self.batch_timeout {
            return Err(JobEngineError::InvalidConfiguration {
                reason: format!(
                    "item_timeout ({:?}) cannot exceed batch_timeout ({:?})",
                    self.item_timeout, self.batch_timeout
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
    fn test_default_config_validates() {
        let config = BatchExecutionConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_parallelism, DEFAULT_PARALLELISM);
        assert!(!config.abort_on_first_error);
    }

    #[test]
    fn test_valid_custom_parallelism() {
        assert!(BatchExecutionConfig::with_parallelism(1).is_ok());
        assert!(BatchExecutionConfig::with_parallelism(10).is_ok());
        assert!(BatchExecutionConfig::with_parallelism(20).is_ok());
    }

    #[test]
    fn test_invalid_parallelism_rejected() {
        assert!(BatchExecutionConfig::with_parallelism(0).is_err());
        assert!(BatchExecutionConfig::with_parallelism(21).is_err());
    }

    #[test]
    fn test_item_timeout_exceeding_batch_timeout_rejected() {
        let config = BatchExecutionConfig {
            item_timeout: Duration::from_secs(600),
            batch_timeout: Duration::from_secs(100),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
