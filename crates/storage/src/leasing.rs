//! # Distributed Job Leasing, Heartbeat, and Stale Recovery
//!
//! Provides distributed worker queue polling with row-level locking (`FOR UPDATE SKIP LOCKED`),
//! periodic heartbeat lease renewals, and automatic recovery of abandoned worker leases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;

use crate::repository::JobRepository;
use crate::schema::{JobProgressPhase, JobRow, JobStatus};

/// Default lease holding duration before a worker is presumed crashed if no heartbeat is received.
pub const DEFAULT_LEASE_DURATION_SECONDS: u32 = 60;

/// Default heartbeat reporting interval for active workers.
pub const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u32 = 15;

/// Default maximum retry attempts for abandoned or failed jobs.
pub const DEFAULT_MAX_ATTEMPTS: i16 = 3;

/// Minimum allowable lease duration to prevent premature worker lease expiration.
pub const MIN_LEASE_DURATION_SECONDS: u32 = 5;

/// Configuration defining worker identity and leasing boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseConfig {
    /// Unique identifier of the worker instance holding or seeking leases.
    pub worker_id: String,
    /// Duration in seconds for which a lease remains valid before expiring.
    pub lease_duration_seconds: u32,
    /// Target interval in seconds between worker heartbeat renewals.
    pub heartbeat_interval_seconds: u32,
    /// Maximum processing attempts permitted before an unrecovered job is marked failed.
    pub max_attempts: i16,
}

impl LeaseConfig {
    /// Creates a validated `LeaseConfig`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if parameters violate leasing invariants.
    pub fn new(
        worker_id: impl Into<String>,
        lease_duration_seconds: u32,
        heartbeat_interval_seconds: u32,
        max_attempts: i16,
    ) -> Result<Self, CoreError> {
        let config = Self {
            worker_id: worker_id.into(),
            lease_duration_seconds,
            heartbeat_interval_seconds,
            max_attempts,
        };
        config.validate()?;
        Ok(config)
    }

    /// Creates a default `LeaseConfig` for a named worker instance.
    #[must_use]
    pub fn for_worker(worker_id: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            lease_duration_seconds: DEFAULT_LEASE_DURATION_SECONDS,
            heartbeat_interval_seconds: DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }

    /// Validates invariants on the leasing configuration.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if `worker_id` is blank, durations are too small,
    /// `heartbeat_interval_seconds >= lease_duration_seconds`, or `max_attempts < 1`.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.worker_id.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "worker_id".to_string(),
                reason: "worker_id cannot be blank".to_string(),
            });
        }

        if self.lease_duration_seconds < MIN_LEASE_DURATION_SECONDS {
            return Err(CoreError::Validation {
                field: "lease_duration_seconds".to_string(),
                reason: format!(
                    "lease_duration_seconds must be >= {MIN_LEASE_DURATION_SECONDS} (actual: {})",
                    self.lease_duration_seconds
                ),
            });
        }

        if self.heartbeat_interval_seconds == 0 {
            return Err(CoreError::Validation {
                field: "heartbeat_interval_seconds".to_string(),
                reason: "heartbeat_interval_seconds must be >= 1".to_string(),
            });
        }

        if self.heartbeat_interval_seconds >= self.lease_duration_seconds {
            return Err(CoreError::Validation {
                field: "heartbeat_interval_seconds".to_string(),
                reason: format!(
                    "heartbeat_interval_seconds ({}) must be strictly less than lease_duration_seconds ({})",
                    self.heartbeat_interval_seconds, self.lease_duration_seconds
                ),
            });
        }

        if self.max_attempts < 1 {
            return Err(CoreError::Validation {
                field: "max_attempts".to_string(),
                reason: format!("max_attempts must be >= 1 (actual: {})", self.max_attempts),
            });
        }

        Ok(())
    }
}

impl Default for LeaseConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", Uuid::now_v7()),
            lease_duration_seconds: DEFAULT_LEASE_DURATION_SECONDS,
            heartbeat_interval_seconds: DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }
}

/// Result of a successful worker heartbeat renewal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatResult {
    /// Unique identifier of the renewed conversion job.
    pub job_id: Uuid,
    /// Absolute timestamp until which the worker's lease is guaranteed.
    pub renewed_until: DateTime<Utc>,
    /// Current lifecycle status of the job.
    pub status: JobStatus,
    /// Current pipeline phase of the job.
    pub phase: JobProgressPhase,
    /// Current completion percentage (0..=100).
    pub progress_pct: i16,
}

/// Structured summary of a stale lease recovery sweep.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseRecoveryReport {
    /// Number of stale jobs reset back to `queued` for worker retry.
    pub recovered_to_queued: usize,
    /// Number of jobs permanently marked as `failed` because `attempt_count >= max_attempts`.
    pub permanently_failed: usize,
    /// List of job UUIDs recovered back to `queued`.
    pub recovered_job_ids: Vec<Uuid>,
    /// List of job UUIDs permanently marked `failed`.
    pub failed_job_ids: Vec<Uuid>,
}

impl LeaseRecoveryReport {
    /// Total number of stale jobs processed during the sweep.
    #[must_use]
    pub fn total_processed(&self) -> usize {
        self.recovered_to_queued + self.permanently_failed
    }

    /// Returns `true` if no stale jobs were found or modified.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_processed() == 0
    }
}

/// Coordinator managing distributed worker leases, periodic heartbeats, and queue polling.
#[derive(Debug, Clone)]
pub struct LeaseManager {
    pool: sqlx::PgPool,
    config: LeaseConfig,
}

impl LeaseManager {
    /// Instantiates a new `LeaseManager` after validating the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if the configuration invariants fail.
    pub fn new(pool: sqlx::PgPool, config: LeaseConfig) -> Result<Self, CoreError> {
        config.validate()?;
        Ok(Self { pool, config })
    }

    /// Returns a reference to the active `PgPool`.
    #[must_use]
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// Returns a reference to the active `LeaseConfig`.
    #[must_use]
    pub fn config(&self) -> &LeaseConfig {
        &self.config
    }

    /// Returns the worker identifier configured for this manager.
    #[must_use]
    pub fn worker_id(&self) -> &str {
        &self.config.worker_id
    }

    /// Polls the database for the next eligible job and atomically acquires a lease.
    ///
    /// Uses PostgreSQL `FOR UPDATE SKIP LOCKED` to prevent worker contention and deadlocks.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database operations fail.
    pub async fn acquire_next_job(&self) -> Result<Option<JobRow>, CoreError> {
        JobRepository::acquire_next_job(&self.pool, &self.config).await
    }

    /// Renews the worker's lease on a job and optionally records execution progress.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the job does not exist.
    /// Returns [`CoreError::JobEngine`] if the worker does not own the active lease,
    /// or if the job has entered a terminal state or timed out.
    pub async fn heartbeat(
        &self,
        job_id: Uuid,
        progress: Option<(JobProgressPhase, i16, Option<String>)>,
    ) -> Result<HeartbeatResult, CoreError> {
        JobRepository::heartbeat_job(
            &self.pool,
            job_id,
            &self.config.worker_id,
            self.config.lease_duration_seconds,
            progress,
        )
        .await
    }

    /// Checks whether the worker currently holds an unexpired lease on the target job.
    ///
    /// Useful for long-running pipeline tasks to detect early cancellation or lease revocation.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if query execution fails.
    pub async fn is_lease_valid(&self, job_id: Uuid) -> Result<bool, CoreError> {
        JobRepository::is_lease_valid(&self.pool, job_id, &self.config.worker_id).await
    }

    /// Voluntarily releases an active lease back to `queued` status (e.g. on graceful worker shutdown).
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the job does not exist.
    /// Returns [`CoreError::JobEngine`] if the worker does not own the lease or if the job is terminal.
    pub async fn release_lease(
        &self,
        job_id: Uuid,
        reason: Option<&str>,
    ) -> Result<JobRow, CoreError> {
        JobRepository::release_lease(&self.pool, job_id, &self.config.worker_id, reason).await
    }

    /// Sweeps expired leases across all workers and either recovers them to `queued` or marks them `failed`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if query execution fails.
    pub async fn recover_stale_jobs(
        &self,
        batch_size: u32,
    ) -> Result<LeaseRecoveryReport, CoreError> {
        JobRepository::recover_stale_jobs(&self.pool, self.config.max_attempts, batch_size).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lease_config_default_and_validation() {
        let config = LeaseConfig::default();
        assert!(config.worker_id.starts_with("worker-"));
        assert_eq!(config.lease_duration_seconds, 60);
        assert_eq!(config.heartbeat_interval_seconds, 15);
        assert_eq!(config.max_attempts, 3);
        assert!(config.validate().is_ok());

        let custom = LeaseConfig::for_worker("worker-prod-1");
        assert_eq!(custom.worker_id, "worker-prod-1");
        assert!(custom.validate().is_ok());
    }

    #[test]
    fn test_lease_config_validation_rejections() {
        // Blank worker_id
        assert!(LeaseConfig::new("", 60, 15, 3).is_err());
        assert!(LeaseConfig::new("   ", 60, 15, 3).is_err());

        // Lease duration too short (<5s)
        assert!(LeaseConfig::new("w1", 4, 2, 3).is_err());

        // Heartbeat interval 0
        assert!(LeaseConfig::new("w1", 60, 0, 3).is_err());

        // Heartbeat interval >= lease duration
        assert!(LeaseConfig::new("w1", 30, 30, 3).is_err());
        assert!(LeaseConfig::new("w1", 30, 35, 3).is_err());

        // Max attempts < 1
        assert!(LeaseConfig::new("w1", 60, 15, 0).is_err());
        assert!(LeaseConfig::new("w1", 60, 15, -1).is_err());
    }

    #[test]
    fn test_heartbeat_result_serde() {
        let job_id = Uuid::now_v7();
        let now = Utc::now();
        let res = HeartbeatResult {
            job_id,
            renewed_until: now,
            status: JobStatus::Running,
            phase: JobProgressPhase::Extracting,
            progress_pct: 45,
        };

        let json = serde_json::to_string(&res).expect("Serialization failed");
        let decoded: HeartbeatResult = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(res, decoded);
    }

    #[test]
    fn test_lease_recovery_report_helpers_and_serde() {
        let mut report = LeaseRecoveryReport::default();
        assert!(report.is_empty());
        assert_eq!(report.total_processed(), 0);

        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        report.recovered_to_queued += 1;
        report.recovered_job_ids.push(id1);
        report.permanently_failed += 1;
        report.failed_job_ids.push(id2);

        assert!(!report.is_empty());
        assert_eq!(report.total_processed(), 2);

        let json = serde_json::to_string(&report).expect("Serialization failed");
        let decoded: LeaseRecoveryReport =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(report, decoded);
    }

    #[test]
    fn test_lease_constants_invariants() {
        const { assert!(MIN_LEASE_DURATION_SECONDS <= DEFAULT_LEASE_DURATION_SECONDS) };
        const { assert!(DEFAULT_HEARTBEAT_INTERVAL_SECONDS < DEFAULT_LEASE_DURATION_SECONDS) };
        const { assert!(DEFAULT_MAX_ATTEMPTS >= 1) };
    }

    #[test]
    fn test_lease_config_serde_roundtrip() {
        let config = LeaseConfig::new("worker-test-alpha", 45, 10, 5).expect("Valid config");
        let json = serde_json::to_string(&config).expect("Serialization failed");
        let decoded: LeaseConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(config, decoded);
    }
}
