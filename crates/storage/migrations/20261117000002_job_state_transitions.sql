-- Migration: 20261117000002_job_state_transitions.sql
-- Description: Append-only historical audit trail of all job lifecycle transitions
-- Schema Version: 1.1.0

CREATE TABLE IF NOT EXISTS job_state_transitions (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    from_status TEXT NOT NULL,
    to_status TEXT NOT NULL,
    phase TEXT NOT NULL,
    progress_pct SMALLINT NOT NULL CHECK (progress_pct >= 0 AND progress_pct <= 100),
    message TEXT NULL,
    error_code TEXT NULL,
    worker_id TEXT NULL,
    transitioned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for retrieving chronological history of transitions per job
CREATE INDEX IF NOT EXISTS idx_job_state_transitions_job_id
    ON job_state_transitions (job_id, transitioned_at DESC);

COMMENT ON TABLE job_state_transitions IS 'Append-only historical audit trail of all job lifecycle transitions';
COMMENT ON COLUMN job_state_transitions.id IS 'UUID v7 primary key of the transition record';
COMMENT ON COLUMN job_state_transitions.job_id IS 'Foreign key referencing parent jobs.id with CASCADE deletion';
COMMENT ON COLUMN job_state_transitions.from_status IS 'Lifecycle status before transition';
COMMENT ON COLUMN job_state_transitions.to_status IS 'Lifecycle status after transition';
COMMENT ON COLUMN job_state_transitions.phase IS 'Pipeline progress phase at time of transition';
COMMENT ON COLUMN job_state_transitions.progress_pct IS 'Integer completion percentage (0-100)';
COMMENT ON COLUMN job_state_transitions.message IS 'Human-readable description or log reason';
COMMENT ON COLUMN job_state_transitions.error_code IS 'Optional machine-readable error code';
COMMENT ON COLUMN job_state_transitions.worker_id IS 'Worker instance identifier responsible for transition';
COMMENT ON COLUMN job_state_transitions.transitioned_at IS 'UTC timestamp when the transition took effect';
