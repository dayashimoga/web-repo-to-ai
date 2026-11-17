//! # Job Persistence Repository & Transactional State Transitions
//!
//! Provides transactional state transitions, row-level locking (`FOR UPDATE`),
//! and historical transition audit logging against PostgreSQL.

use chrono::Utc;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;

use crate::leasing::{HeartbeatResult, LeaseConfig, LeaseRecoveryReport};
use crate::schema::{JobProgressPhase, JobRow, JobStatus};
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

    /// Polls the database for the next eligible job and atomically acquires a lease.
    ///
    /// Selects either a `queued` job or a stale `leased`/`running` job whose lease has expired
    /// and whose `attempt_count < config.max_attempts`.
    ///
    /// Uses PostgreSQL `FOR UPDATE SKIP LOCKED` for lock contention avoidance across multiple workers.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if database operations fail.
    pub async fn acquire_next_job(
        pool: &sqlx::PgPool,
        config: &LeaseConfig,
    ) -> Result<Option<JobRow>, CoreError> {
        config.validate()?;

        let mut tx = pool.begin().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to begin lease acquisition transaction: {err}"
            ))
        })?;

        // 1. Fetch next eligible job with row lock, skipping rows locked by other workers
        let candidate: Option<JobRow> = sqlx::query_as(
            r#"
            SELECT *
            FROM jobs
            WHERE (
                status = 'queued'
                OR (
                    status IN ('leased', 'running')
                    AND lease_expires_at IS NOT NULL
                    AND lease_expires_at < NOW()
                    AND attempt_count < $1
                )
            )
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(config.max_attempts)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to query candidate job for lease: {err}"))
        })?;

        let Some(candidate) = candidate else {
            tx.rollback().await.map_err(|err| {
                CoreError::Storage(format!("Failed to rollback empty lease transaction: {err}"))
            })?;
            return Ok(None);
        };

        let from_status = candidate.status_enum()?;
        let is_stale_reacquisition = from_status != JobStatus::Queued;
        let message = if is_stale_reacquisition {
            format!(
                "Stale worker lease reacquired (previous worker: {:?}, attempt {})",
                candidate.leased_by,
                candidate.attempt_count + 1
            )
        } else {
            "Worker lease acquired".to_string()
        };

        // 2. Update job to leased with lease expiration calculated relative to NOW()
        let updated: JobRow = sqlx::query_as(
            r#"
            UPDATE jobs
            SET status = 'leased',
                leased_by = $2,
                lease_expires_at = NOW() + ($3::bigint * INTERVAL '1 second'),
                attempt_count = attempt_count + 1,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(candidate.id)
        .bind(&config.worker_id)
        .bind(config.lease_duration_seconds as i64)
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to update leased state for job {}: {err}",
                candidate.id
            ))
        })?;

        // 3. Record historical state transition
        let transition_row = JobStateTransitionRow {
            id: Uuid::now_v7(),
            job_id: candidate.id,
            from_status: from_status.to_string(),
            to_status: JobStatus::Leased.to_string(),
            phase: candidate.progress_phase,
            progress_pct: candidate.progress_pct,
            message: Some(message),
            error_code: None,
            worker_id: Some(config.worker_id.clone()),
            transitioned_at: Utc::now(),
        };

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
                "Failed to record lease acquisition transition for job {}: {err}",
                candidate.id
            ))
        })?;

        tx.commit().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to commit lease acquisition transaction for job {}: {err}",
                candidate.id
            ))
        })?;

        Ok(Some(updated))
    }

    /// Renews a worker's active lease and optionally updates execution phase and progress.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the job is not found.
    /// Returns [`CoreError::JobEngine`] if the worker does not own the lease,
    /// if the job is in a terminal state, or if the lease has expired.
    pub async fn heartbeat_job(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        worker_id: &str,
        extension_seconds: u32,
        progress: Option<(JobProgressPhase, i16, Option<String>)>,
    ) -> Result<HeartbeatResult, CoreError> {
        let mut tx = pool.begin().await.map_err(|err| {
            CoreError::Storage(format!("Failed to begin heartbeat transaction: {err}"))
        })?;

        let current: Option<JobRow> = sqlx::query_as(
            r#"
            SELECT * FROM jobs WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to acquire lock for heartbeat on job {job_id}: {err}"
            ))
        })?;

        let current = current.ok_or_else(|| CoreError::NotFound {
            resource_type: "job",
            id: job_id.to_string(),
        })?;

        let current_status = current.status_enum()?;
        if current_status.is_terminal() {
            return Err(CoreError::JobEngine(format!(
                "Cannot renew lease: job {job_id} is in terminal state '{current_status}'"
            )));
        }

        if current.leased_by.as_deref() != Some(worker_id) {
            return Err(CoreError::JobEngine(format!(
                "Cannot renew lease: job {job_id} is leased by {:?}, not '{worker_id}'",
                current.leased_by
            )));
        }

        if let Some(expires_at) = current.lease_expires_at {
            if expires_at < Utc::now() {
                return Err(CoreError::JobEngine(format!(
                    "Cannot renew lease: job {job_id} lease expired at {expires_at}"
                )));
            }
        }

        // Validate progress bounds if specified
        if let Some((_, pct, _)) = &progress {
            if !(0..=100).contains(pct) {
                return Err(CoreError::Validation {
                    field: "progress_pct".to_string(),
                    reason: format!("progress_pct must be between 0 and 100 (actual: {pct})"),
                });
            }
        }

        let new_phase = progress.as_ref().map(|p| p.0.as_str());
        let new_pct = progress.as_ref().map(|p| p.1);
        let progress_message = progress.as_ref().and_then(|p| p.2.clone());

        let updated: JobRow = sqlx::query_as(
            r#"
            UPDATE jobs
            SET lease_expires_at = NOW() + ($2::bigint * INTERVAL '1 second'),
                progress_phase = COALESCE($3, progress_phase),
                progress_pct = COALESCE($4, progress_pct),
                status = CASE
                    WHEN $3 IS NOT NULL AND status = 'leased' THEN 'running'
                    ELSE status
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(job_id)
        .bind(extension_seconds as i64)
        .bind(new_phase)
        .bind(new_pct)
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to update heartbeat on job {job_id}: {err}"))
        })?;

        // If progress changed, record a state transition entry
        if let Some((phase, pct, _)) = progress {
            let transition_row = JobStateTransitionRow {
                id: Uuid::now_v7(),
                job_id,
                from_status: current_status.to_string(),
                to_status: updated.status.clone(),
                phase: phase.as_str().to_string(),
                progress_pct: pct,
                message: progress_message.or_else(|| Some("Heartbeat progress update".to_string())),
                error_code: None,
                worker_id: Some(worker_id.to_string()),
                transitioned_at: Utc::now(),
            };

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
                    "Failed to record heartbeat transition for job {job_id}: {err}"
                ))
            })?;
        }

        tx.commit().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to commit heartbeat transaction for job {job_id}: {err}"
            ))
        })?;

        let renewed_until = updated.lease_expires_at.unwrap_or_else(Utc::now);

        Ok(HeartbeatResult {
            job_id,
            renewed_until,
            status: updated.status_enum()?,
            phase: updated.progress_phase_enum()?,
            progress_pct: updated.progress_pct,
        })
    }

    /// Checks whether the worker holds an active, unexpired lease on the target job.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if query execution fails.
    pub async fn is_lease_valid(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        worker_id: &str,
    ) -> Result<bool, CoreError> {
        let is_valid: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM jobs
                WHERE id = $1
                  AND leased_by = $2
                  AND status IN ('leased', 'running')
                  AND (lease_expires_at IS NULL OR lease_expires_at > NOW())
            )
            "#,
        )
        .bind(job_id)
        .bind(worker_id)
        .fetch_one(pool)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to check lease validity for job {job_id}: {err}"
            ))
        })?;

        Ok(is_valid)
    }

    /// Voluntarily releases an active lease back to `queued` status.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the job is not found.
    /// Returns [`CoreError::JobEngine`] if the worker does not own the active lease or if the job is terminal.
    pub async fn release_lease(
        pool: &sqlx::PgPool,
        job_id: Uuid,
        worker_id: &str,
        reason: Option<&str>,
    ) -> Result<JobRow, CoreError> {
        let mut tx = pool.begin().await.map_err(|err| {
            CoreError::Storage(format!("Failed to begin lease release transaction: {err}"))
        })?;

        let current: Option<JobRow> = sqlx::query_as(
            r#"
            SELECT * FROM jobs WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!(
                "Failed to acquire lock for lease release on job {job_id}: {err}"
            ))
        })?;

        let current = current.ok_or_else(|| CoreError::NotFound {
            resource_type: "job",
            id: job_id.to_string(),
        })?;

        let current_status = current.status_enum()?;
        if current.leased_by.as_deref() != Some(worker_id) {
            return Err(CoreError::JobEngine(format!(
                "Cannot release lease: job {job_id} is leased by {:?}, not '{worker_id}'",
                current.leased_by
            )));
        }

        if current_status != JobStatus::Leased && current_status != JobStatus::Running {
            return Err(CoreError::JobEngine(format!(
                "Cannot release lease: job {job_id} is in status '{current_status}', expected leased or running"
            )));
        }

        let updated: JobRow = sqlx::query_as(
            r#"
            UPDATE jobs
            SET status = 'queued',
                leased_by = NULL,
                lease_expires_at = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(job_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to release lease on job {job_id}: {err}"))
        })?;

        let transition_row = JobStateTransitionRow {
            id: Uuid::now_v7(),
            job_id,
            from_status: current_status.to_string(),
            to_status: JobStatus::Queued.to_string(),
            phase: current.progress_phase,
            progress_pct: current.progress_pct,
            message: Some(
                reason
                    .unwrap_or("Worker voluntarily released lease")
                    .to_string(),
            ),
            error_code: None,
            worker_id: Some(worker_id.to_string()),
            transitioned_at: Utc::now(),
        };

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
                "Failed to record lease release transition for job {job_id}: {err}"
            ))
        })?;

        tx.commit().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to commit lease release transaction for job {job_id}: {err}"
            ))
        })?;

        Ok(updated)
    }

    /// Sweeps stale jobs whose leases have expired across all workers.
    ///
    /// Jobs with `attempt_count < max_attempts` are reset to `queued` for retry.
    /// Jobs that have exceeded `max_attempts` are marked as permanently `failed` with code `LEASE_TIMEOUT_EXCEEDED`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if query execution fails.
    pub async fn recover_stale_jobs(
        pool: &sqlx::PgPool,
        max_attempts: i16,
        batch_size: u32,
    ) -> Result<LeaseRecoveryReport, CoreError> {
        let mut tx = pool.begin().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to begin stale lease recovery transaction: {err}"
            ))
        })?;

        let stale_jobs: Vec<JobRow> = sqlx::query_as(
            r#"
            SELECT *
            FROM jobs
            WHERE status IN ('leased', 'running')
              AND lease_expires_at IS NOT NULL
              AND lease_expires_at < NOW()
            ORDER BY lease_expires_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(batch_size as i64)
        .fetch_all(&mut *tx)
        .await
        .map_err(|err| {
            CoreError::Storage(format!("Failed to query stale jobs for recovery: {err}"))
        })?;

        let mut report = LeaseRecoveryReport::default();

        for job in stale_jobs {
            let from_status = job.status.clone();
            if job.attempt_count < max_attempts {
                // Recover to queued
                sqlx::query(
                    r#"
                    UPDATE jobs
                    SET status = 'queued',
                        leased_by = NULL,
                        lease_expires_at = NULL,
                        updated_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(job.id)
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    CoreError::Storage(format!(
                        "Failed to recover stale job {} to queued: {err}",
                        job.id
                    ))
                })?;

                let message = format!(
                    "Lease expired (worker: {:?}); recovered to queued for retry (attempt {}/{})",
                    job.leased_by, job.attempt_count, max_attempts
                );

                let transition_row = JobStateTransitionRow {
                    id: Uuid::now_v7(),
                    job_id: job.id,
                    from_status,
                    to_status: JobStatus::Queued.to_string(),
                    phase: JobProgressPhase::Initializing.as_str().to_string(),
                    progress_pct: job.progress_pct,
                    message: Some(message),
                    error_code: None,
                    worker_id: job.leased_by,
                    transitioned_at: Utc::now(),
                };

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
                        "Failed to record recovery transition for job {}: {err}",
                        job.id
                    ))
                })?;

                report.recovered_to_queued += 1;
                report.recovered_job_ids.push(job.id);
            } else {
                // Permanently fail
                let error_msg = format!(
                    "Worker lease expired and job exceeded maximum retry attempts ({}/{})",
                    job.attempt_count, max_attempts
                );

                sqlx::query(
                    r#"
                    UPDATE jobs
                    SET status = 'failed',
                        progress_phase = 'failed',
                        error_code = 'LEASE_TIMEOUT_EXCEEDED',
                        error_message = $2,
                        completed_at = NOW(),
                        updated_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(job.id)
                .bind(&error_msg)
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    CoreError::Storage(format!("Failed to fail timed-out job {}: {err}", job.id))
                })?;

                let transition_row = JobStateTransitionRow {
                    id: Uuid::now_v7(),
                    job_id: job.id,
                    from_status,
                    to_status: JobStatus::Failed.to_string(),
                    phase: JobProgressPhase::Failed.as_str().to_string(),
                    progress_pct: 100,
                    message: Some(error_msg),
                    error_code: Some("LEASE_TIMEOUT_EXCEEDED".to_string()),
                    worker_id: job.leased_by,
                    transitioned_at: Utc::now(),
                };

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
                        "Failed to record failure transition for job {}: {err}",
                        job.id
                    ))
                })?;

                report.permanently_failed += 1;
                report.failed_job_ids.push(job.id);
            }
        }

        tx.commit().await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to commit stale lease recovery transaction: {err}"
            ))
        })?;

        Ok(report)
    }
}
