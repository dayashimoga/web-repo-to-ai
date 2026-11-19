-- Migration: 20261119000001_batch_jobs_and_items.sql
-- Description: Multi-URL batch conversion tracking headers and itemized URL execution outcomes
-- Schema Version: 1.2.0

CREATE TABLE IF NOT EXISTS batch_jobs (
    id UUID PRIMARY KEY,
    name TEXT NULL,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'processing', 'completed', 'partial_failure', 'failed', 'cancelled')),
    total_items INTEGER NOT NULL CHECK (total_items >= 0),
    queued_items INTEGER NOT NULL DEFAULT 0 CHECK (queued_items >= 0),
    processing_items INTEGER NOT NULL DEFAULT 0 CHECK (processing_items >= 0),
    completed_items INTEGER NOT NULL DEFAULT 0 CHECK (completed_items >= 0),
    failed_items INTEGER NOT NULL DEFAULT 0 CHECK (failed_items >= 0),
    max_parallelism INTEGER NOT NULL DEFAULT 5 CHECK (max_parallelism >= 1 AND max_parallelism <= 20),
    owner_session_id UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_batch_jobs_owner_session
    ON batch_jobs (owner_session_id) WHERE owner_session_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_batch_jobs_created_at
    ON batch_jobs (created_at DESC);

CREATE TABLE IF NOT EXISTS batch_items (
    batch_id UUID NOT NULL REFERENCES batch_jobs(id) ON DELETE CASCADE,
    item_index INTEGER NOT NULL CHECK (item_index >= 0),
    job_id UUID NOT NULL,
    url TEXT NOT NULL,
    custom_id TEXT NULL,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'processing', 'completed', 'failed', 'timed_out', 'cancelled')),
    error_code TEXT NULL,
    error_message TEXT NULL,
    duration_ms BIGINT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NULL,
    PRIMARY KEY (batch_id, item_index)
);

CREATE INDEX IF NOT EXISTS idx_batch_items_job_id
    ON batch_items (job_id);

COMMENT ON TABLE batch_jobs IS 'Parent tracking records for multi-URL batch conversion submissions';
COMMENT ON COLUMN batch_jobs.id IS 'UUID v7 primary key of the batch job';
COMMENT ON COLUMN batch_jobs.name IS 'Optional human-readable label or description';
COMMENT ON COLUMN batch_jobs.status IS 'Lifecycle state: queued, processing, completed, partial_failure, failed, cancelled';
COMMENT ON COLUMN batch_jobs.total_items IS 'Total count of URLs submitted in this batch';
COMMENT ON COLUMN batch_jobs.queued_items IS 'Count of items awaiting execution permit';
COMMENT ON COLUMN batch_jobs.processing_items IS 'Count of items currently processing in parallel';
COMMENT ON COLUMN batch_jobs.completed_items IS 'Count of items completed successfully';
COMMENT ON COLUMN batch_jobs.failed_items IS 'Count of items that failed or timed out';
COMMENT ON COLUMN batch_jobs.max_parallelism IS 'Concurrency ceiling applied to item processing';
COMMENT ON COLUMN batch_jobs.owner_session_id IS 'Client session identifier scoping ownership';

COMMENT ON TABLE batch_items IS 'Individual URL task records and execution outcomes within a batch';
COMMENT ON COLUMN batch_items.batch_id IS 'Foreign key referencing parent batch_jobs.id with CASCADE deletion';
COMMENT ON COLUMN batch_items.item_index IS 'Zero-based sequence index within the batch submission';
COMMENT ON COLUMN batch_items.job_id IS 'UUID v7 child conversion job identifier';
COMMENT ON COLUMN batch_items.url IS 'Target URL of the conversion item';
COMMENT ON COLUMN batch_items.custom_id IS 'Optional client correlation identifier';
COMMENT ON COLUMN batch_items.status IS 'Item execution state: queued, processing, completed, failed, timed_out, cancelled';
COMMENT ON COLUMN batch_items.duration_ms IS 'Execution duration in milliseconds';
