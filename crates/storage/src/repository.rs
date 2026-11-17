//! # Job Persistence Repository & Transactional State Transitions
//!
//! Provides transactional state transitions, row-level locking (`FOR UPDATE`),
//! and historical transition audit logging against PostgreSQL.

use chrono::Utc;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;

use crate::schema::{JobProgressPhase, JobRow};
use crate::state_machine::{JobStateMachine, JobStateTransitionRow, JobTransitionRequest};

/// Repository executing durable job state operations and transition audits.
#[derive(Debug, Clone, Copy, Default)]
pub struct JobRepository;

impl JobRepository {
    /// Inserts a new conversion job record into the `jobs` table.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database insertion fails.
    pub async fn create_job(pool: &sqlx::PgPool, job: &JobRow) -> Result<JobRow, CoreError> {
        job.validate()?;

        let created: JobRow = sqlx::query_as(
            r#"
            INSERT INTO jobs (
                id, kind, status, source_url, config_json, progress_phase,
                progress_pct, leased_by, lease_expires_at, attempt_count,
                error_code, error_message, owner_session_id, created_at,
                updated_at, completed_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            )
            RETURNING *
            "#,
        )
        .bind(job.id)
        .bind(&job.kind)
        .bind(&job.status)
        .bind(&job.source_url)
        .bind(&job.config_json)
        .bind(&job.progress_phase)
        .bind(job.progress_pct)
        .bind(&job.leased_by)
        .bind(job.lease_expires_at)
        .bind(job.attempt_count)
        .bind(&job.error_code)
        .bind(&job.error_message)
        .bind(job.owner_session_id)
        .bind(job.created_at)
        .bind(job.updated_at)
        .bind(job.completed_at)
        .fetch_one(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to insert job {}: {err}", job.id)))?;

        Ok(created)
    }

    /// Fetches a conversion job by primary key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn get_job(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<JobRow>, CoreError> {
        let row = sqlx::query_as(
            r#"
            SELECT * FROM jobs WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to fetch job {id}: {err}")))?;

        Ok(row)
    }

    /// Atomically transitions a job's lifecycle status using a PostgreSQL transaction
    /// with row-level locking (`FOR UPDATE`) and appends a transition record to `job_state_transitions`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the job does not exist.
    /// Returns [`CoreError::Validation`] if the requested transition violates state machine rules.
    /// Returns [`CoreError::Storage`] if database transactions or locks fail.
    pub async fn transition_job(
        pool: &sqlx::PgPool,
        id: Uuid,
        req: JobTransitionRequest,
    ) -> Result<JobRow, CoreError> {
        let mut tx = pool.begin().await.map_err(|err| {
            CoreError::Storage(format!("Failed to begin transition transaction: {err}"))
        })?;

        // 1. Lock job row for update
        let current: Option<JobRow> = sqlx::query_as(
            r#"
            SELECT * FROM jobs WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to acquire lock on job {id}: {err}")))?;

        let current = current.ok_or_else(|| CoreError::NotFound {
            resource_type: "job",
            id: id.to_string(),
        })?;
        let from_status = current.status_enum()?;

        // 2. Validate transition rules and invariants
        JobStateMachine::validate_transition_request(&current, &req)?;

        // 3. Determine completed_at timestamp
        let completed_at = if req.to_status.is_terminal() {
            Some(Utc::now())
        } else {
            current.completed_at
        };

        // 4. Update job row
        let updated: JobRow = sqlx::query_as(
            r#"
            UPDATE jobs
            SET status = $2,
                progress_phase = $3,
                progress_pct = $4,
                error_code = COALESCE($5, error_code),
                error_message = COALESCE($6, error_message),
                leased_by = CASE
                    WHEN $2 = 'queued' THEN NULL
                    WHEN $7 IS NOT NULL THEN $7
                    ELSE leased_by
                END,
                lease_expires_at = CASE
                    WHEN $2 = 'queued' OR $2 = 'completed' OR $2 = 'partial' OR $2 = 'failed' OR $2 = 'cancelled' THEN NULL
                    ELSE lease_expires_at
                END,
                completed_at = $8,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.to_status.as_str())
        .bind(req.phase.as_str())
        .bind(req.progress_pct)
        .bind(&req.error_code)
        .bind(&req.message)
        .bind(&req.worker_id)
        .bind(completed_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| CoreError::Storage(format!("Failed to update job {id} status: {err}")))?;

        // 5. Append historical transition record
        let transition_row = JobStateTransitionRow::new(id, from_status, &req);
        sqlx::query(
            r#"
            INSERT INTO job_state_transitions (
                id, job_id, from_status, to_status, phase, progress_pct,
                message, error_code, worker_id, transitioned_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(transition_row.id)
        .bind(transition_row.job_id)
        .bind(&transition_row.from_status)
        .bind(&transition_row.to_status)
        .bind(&transition_row.phase)
        .bind(transition_row.progress_pct)
        .bind(&transition_row.message)
        .bind(&transition_row.error_code)
        .bind(&transition_row.worker_id)
        .bind(transition_row.transitioned_at)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to record state transition for job {id}: {err}"
            ))
        })?;

        tx.commit().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to commit transition transaction for job {id}: {err}"
            ))
        })?;

        Ok(updated)
    }

    /// Fetches the chronological transition history for a given job.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database query fails.
    pub async fn get_transition_history(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<Vec<JobStateTransitionRow>, CoreError> {
        let rows = sqlx::query_as(
            r#"
            SELECT *
            FROM job_state_transitions
            WHERE job_id = $1
            ORDER BY transitioned_at ASC
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to fetch transition history for job {id}: {err}"
            ))
        })?;

        Ok(rows)
    }

    /// Advances running progress phase and percentage.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if transition fails.
    pub async fn update_progress(
        pool: &sqlx::PgPool,
        id: Uuid,
        phase: JobProgressPhase,
        progress_pct: i16,
        message: Option<String>,
    ) -> Result<JobRow, CoreError> {
        let req = JobTransitionRequest::update_progress(phase, progress_pct, message);
        Self::transition_job(pool, id, req).await
    }

    /// Marks a job as completed or partially completed.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if transition fails.
    pub async fn complete_job(
        pool: &sqlx::PgPool,
        id: Uuid,
        is_partial: bool,
        message: Option<String>,
    ) -> Result<JobRow, CoreError> {
        let req = if is_partial {
            JobTransitionRequest::complete_partial(message)
        } else {
            JobTransitionRequest::complete(message)
        };
        Self::transition_job(pool, id, req).await
    }

    /// Fails a job with error code and description.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if transition fails.
    pub async fn fail_job(
        pool: &sqlx::PgPool,
        id: Uuid,
        error_code: &str,
        error_message: &str,
    ) -> Result<JobRow, CoreError> {
        let req = JobTransitionRequest::fail(error_code, error_message);
        Self::transition_job(pool, id, req).await
    }

    /// Cancels a job with optional reason.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if transition fails.
    pub async fn cancel_job(
        pool: &sqlx::PgPool,
        id: Uuid,
        reason: Option<&str>,
    ) -> Result<JobRow, CoreError> {
        let req = JobTransitionRequest::cancel(reason.map(ToString::to_string));
        Self::transition_job(pool, id, req).await
    }
}
