//! # Job State Machine & Transition Logic
//!
//! Enforces deterministic lifecycle transitions, terminal status immutability,
//! progress invariants, and transition audit records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::JobId;

use crate::schema::{JobProgressPhase, JobRow, JobStatus};

/// Evaluator enforcing legal job lifecycle transitions and progress invariants.
#[derive(Debug, Clone, Copy, Default)]
pub struct JobStateMachine;

impl JobStateMachine {
    /// Validates whether a state transition from `current` to `target` is permitted.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if the transition is disallowed or attempts
    /// to mutate a terminal job.
    pub fn evaluate_transition(current: JobStatus, target: JobStatus) -> Result<(), CoreError> {
        if current.is_terminal() {
            return Err(CoreError::Validation {
                field: "status".to_string(),
                reason: format!(
                    "Cannot transition job from terminal state '{current}' to '{target}'"
                ),
            });
        }

        if !current.can_transition_to(target) {
            return Err(CoreError::Validation {
                field: "status".to_string(),
                reason: format!("Illegal job state transition from '{current}' to '{target}'"),
            });
        }

        Ok(())
    }

    /// Validates an incoming transition request against the current state of a `JobRow`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if the target status or progress values violate invariants.
    pub fn validate_transition_request(
        current: &JobRow,
        req: &JobTransitionRequest,
    ) -> Result<(), CoreError> {
        let current_status = current.status_enum()?;
        Self::evaluate_transition(current_status, req.to_status)?;

        if !(0..=100).contains(&req.progress_pct) {
            return Err(CoreError::Validation {
                field: "progress_pct".to_string(),
                reason: format!(
                    "progress_pct must be between 0 and 100 (actual: {})",
                    req.progress_pct
                ),
            });
        }

        // Completion invariants
        if req.to_status == JobStatus::Completed {
            if req.progress_pct != 100 {
                return Err(CoreError::Validation {
                    field: "progress_pct".to_string(),
                    reason: format!(
                        "Completed job must have 100% progress (actual: {})",
                        req.progress_pct
                    ),
                });
            }
            if req.phase != JobProgressPhase::Completed {
                return Err(CoreError::Validation {
                    field: "phase".to_string(),
                    reason: format!(
                        "Completed job must have phase 'completed' (actual: '{}')",
                        req.phase
                    ),
                });
            }
        }

        // Failure invariants
        if req.to_status == JobStatus::Failed && req.phase != JobProgressPhase::Failed {
            return Err(CoreError::Validation {
                field: "phase".to_string(),
                reason: format!(
                    "Failed job must have phase 'failed' (actual: '{}')",
                    req.phase
                ),
            });
        }

        Ok(())
    }
}

/// Request to transition a job into a new state with progress and audit metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobTransitionRequest {
    /// Target lifecycle status.
    pub to_status: JobStatus,
    /// Execution pipeline phase.
    pub phase: JobProgressPhase,
    /// Completion percentage (0 to 100).
    pub progress_pct: i16,
    /// Informational message or transition reason.
    pub message: Option<String>,
    /// Machine-readable error code if transitioning to failure.
    pub error_code: Option<String>,
    /// Identifier of worker responsible for this transition.
    pub worker_id: Option<String>,
}

impl JobTransitionRequest {
    /// Transition request for worker lease acquisition.
    #[must_use]
    pub fn lease(worker_id: impl Into<String>) -> Self {
        Self {
            to_status: JobStatus::Leased,
            phase: JobProgressPhase::Initializing,
            progress_pct: 0,
            message: Some("Worker lease acquired".to_string()),
            error_code: None,
            worker_id: Some(worker_id.into()),
        }
    }

    /// Transition request for advancing to active processing.
    #[must_use]
    pub fn start_running(worker_id: Option<String>, initial_phase: JobProgressPhase) -> Self {
        Self {
            to_status: JobStatus::Running,
            phase: initial_phase,
            progress_pct: 5,
            message: Some("Job execution started".to_string()),
            error_code: None,
            worker_id,
        }
    }

    /// Transition request for intermediate progress updates during running state.
    #[must_use]
    pub fn update_progress(
        phase: JobProgressPhase,
        progress_pct: i16,
        message: Option<String>,
    ) -> Self {
        Self {
            to_status: JobStatus::Running,
            phase,
            progress_pct,
            message,
            error_code: None,
            worker_id: None,
        }
    }

    /// Transition request for successful terminal completion.
    #[must_use]
    pub fn complete(message: Option<String>) -> Self {
        Self {
            to_status: JobStatus::Completed,
            phase: JobProgressPhase::Completed,
            progress_pct: 100,
            message: message.or_else(|| Some("Job completed successfully".to_string())),
            error_code: None,
            worker_id: None,
        }
    }

    /// Transition request for partial terminal completion (e.g. some excluded items).
    #[must_use]
    pub fn complete_partial(message: Option<String>) -> Self {
        Self {
            to_status: JobStatus::Partial,
            phase: JobProgressPhase::Completed,
            progress_pct: 100,
            message: message
                .or_else(|| Some("Job partially completed with warnings/exclusions".to_string())),
            error_code: None,
            worker_id: None,
        }
    }

    /// Transition request for failure with error code and description.
    #[must_use]
    pub fn fail(error_code: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            to_status: JobStatus::Failed,
            phase: JobProgressPhase::Failed,
            progress_pct: 100,
            message: Some(error_message.into()),
            error_code: Some(error_code.into()),
            worker_id: None,
        }
    }

    /// Transition request for explicit cancellation.
    #[must_use]
    pub fn cancel(reason: Option<String>) -> Self {
        Self {
            to_status: JobStatus::Cancelled,
            phase: JobProgressPhase::Failed,
            progress_pct: 100,
            message: reason.or_else(|| Some("Job cancelled by user or operator".to_string())),
            error_code: Some("JOB_CANCELLED".to_string()),
            worker_id: None,
        }
    }

    /// Transition request for re-queuing after lease expiration or retry.
    #[must_use]
    pub fn requeue(reason: impl Into<String>) -> Self {
        Self {
            to_status: JobStatus::Queued,
            phase: JobProgressPhase::Initializing,
            progress_pct: 0,
            message: Some(reason.into()),
            error_code: None,
            worker_id: None,
        }
    }
}

/// Relational record representing a historical transition event in `job_state_transitions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobStateTransitionRow {
    /// Unique UUID v7 primary key of the transition event.
    pub id: Uuid,
    /// Reference to parent `jobs.id`.
    pub job_id: Uuid,
    /// State before transition.
    pub from_status: String,
    /// State after transition.
    pub to_status: String,
    /// Pipeline phase at transition time.
    pub phase: String,
    /// Progress percentage at transition time.
    pub progress_pct: i16,
    /// Log message or reason.
    pub message: Option<String>,
    /// Machine-readable error code if transitioning to failure.
    pub error_code: Option<String>,
    /// Identifier of worker responsible for transition.
    pub worker_id: Option<String>,
    /// UTC timestamp of transition event.
    pub transitioned_at: DateTime<Utc>,
}

impl JobStateTransitionRow {
    /// Constructs a new historical transition record with current timestamp and UUID v7.
    #[must_use]
    pub fn new(job_id: Uuid, from_status: JobStatus, req: &JobTransitionRequest) -> Self {
        Self {
            id: Uuid::now_v7(),
            job_id,
            from_status: from_status.as_str().to_string(),
            to_status: req.to_status.as_str().to_string(),
            phase: req.phase.as_str().to_string(),
            progress_pct: req.progress_pct,
            message: req.message.clone(),
            error_code: req.error_code.clone(),
            worker_id: req.worker_id.clone(),
            transitioned_at: Utc::now(),
        }
    }

    /// Returns the strongly-typed `JobId` domain identifier.
    #[must_use]
    pub fn job_id(&self) -> JobId {
        JobId::from(self.job_id)
    }

    /// Parses and returns the typed `from_status`.
    pub fn from_status_enum(&self) -> Result<JobStatus, CoreError> {
        self.from_status.parse()
    }

    /// Parses and returns the typed `to_status`.
    pub fn to_status_enum(&self) -> Result<JobStatus, CoreError> {
        self.to_status.parse()
    }

    /// Parses and returns the typed `phase`.
    pub fn phase_enum(&self) -> Result<JobProgressPhase, CoreError> {
        self.phase.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::JobKind;

    #[test]
    fn test_legal_state_transitions() {
        assert!(JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Leased).is_ok());
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Cancelled).is_ok()
        );

        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Running).is_ok()
        );
        assert!(JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Queued).is_ok());
        assert!(JobStateMachine::evaluate_transition(JobStatus::Leased, JobStatus::Failed).is_ok());

        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Completed).is_ok()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Partial).is_ok()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Failed).is_ok()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Running, JobStatus::Queued).is_ok()
        );
    }

    #[test]
    fn test_illegal_state_transitions_rejected() {
        // Direct jump from Queued to Completed
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Completed).is_err()
        );
        // Direct jump from Queued to Running without lease
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Queued, JobStatus::Running).is_err()
        );

        // Terminal transitions disallowed
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Completed, JobStatus::Running).is_err()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Failed, JobStatus::Queued).is_err()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Cancelled, JobStatus::Leased).is_err()
        );
        assert!(
            JobStateMachine::evaluate_transition(JobStatus::Partial, JobStatus::Running).is_err()
        );
    }

    #[test]
    fn test_transition_request_builders() {
        let lease_req = JobTransitionRequest::lease("worker-node-1");
        assert_eq!(lease_req.to_status, JobStatus::Leased);
        assert_eq!(lease_req.worker_id.as_deref(), Some("worker-node-1"));

        let run_req = JobTransitionRequest::start_running(
            Some("worker-node-1".into()),
            JobProgressPhase::Fetching,
        );
        assert_eq!(run_req.to_status, JobStatus::Running);
        assert_eq!(run_req.phase, JobProgressPhase::Fetching);

        let complete_req = JobTransitionRequest::complete(None);
        assert_eq!(complete_req.to_status, JobStatus::Completed);
        assert_eq!(complete_req.progress_pct, 100);
        assert_eq!(complete_req.phase, JobProgressPhase::Completed);

        let fail_req = JobTransitionRequest::fail("TIMEOUT", "Extraction exceeded timeout");
        assert_eq!(fail_req.to_status, JobStatus::Failed);
        assert_eq!(fail_req.error_code.as_deref(), Some("TIMEOUT"));

        let cancel_req = JobTransitionRequest::cancel(Some("User requested stop".into()));
        assert_eq!(cancel_req.to_status, JobStatus::Cancelled);

        let requeue_req = JobTransitionRequest::requeue("Lease expired without heartbeat");
        assert_eq!(requeue_req.to_status, JobStatus::Queued);
    }

    #[test]
    fn test_validate_transition_request_invariants() {
        let job = JobRow::new(
            JobKind::WebSingle,
            "https://example.com",
            serde_json::json!({}),
        );

        // Valid lease transition from Queued
        let lease_req = JobTransitionRequest::lease("w-1");
        assert!(JobStateMachine::validate_transition_request(&job, &lease_req).is_ok());

        // Invalid direct completion from Queued
        let complete_req = JobTransitionRequest::complete(None);
        assert!(JobStateMachine::validate_transition_request(&job, &complete_req).is_err());

        // Invalid completion progress < 100
        let mut running_job = job;
        running_job.status = JobStatus::Running.as_str().to_string();
        let mut bad_complete = JobTransitionRequest::complete(None);
        bad_complete.progress_pct = 95;
        assert!(JobStateMachine::validate_transition_request(&running_job, &bad_complete).is_err());
    }

    #[test]
    fn test_job_state_transition_row_creation() {
        let job_id = Uuid::new_v4();
        let req =
            JobTransitionRequest::start_running(Some("w-1".into()), JobProgressPhase::Extracting);
        let row = JobStateTransitionRow::new(job_id, JobStatus::Leased, &req);

        assert_eq!(row.job_id, job_id);
        assert_eq!(row.from_status, "leased");
        assert_eq!(row.to_status, "running");
        assert_eq!(row.phase, "extracting");
        assert_eq!(row.from_status_enum().unwrap(), JobStatus::Leased);
        assert_eq!(row.to_status_enum().unwrap(), JobStatus::Running);
        assert_eq!(row.phase_enum().unwrap(), JobProgressPhase::Extracting);
    }
}
