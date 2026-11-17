-- PostgreSQL Relational Schema for web-repo-to-ai
-- Schema Version: 1.0.0
-- Compliant with PostgreSQL 16+, docs/DATA_MODEL.md Section 2, and ADR-005.

-- Utility function: automatic updated_at timestamp maintenance
CREATE OR REPLACE FUNCTION fn_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 1. JOBS TABLE: Core conversion jobs, state tracking, and distributed leasing
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('web_single', 'web_batch', 'web_crawl', 'github_public', 'github_private')),
    status TEXT NOT NULL CHECK (status IN ('queued', 'leased', 'running', 'completed', 'partial', 'failed', 'cancelled')),
    source_url TEXT NOT NULL CHECK (length(source_url) > 0 AND length(source_url) <= 2048),
    config_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    progress_phase TEXT NOT NULL CHECK (progress_phase IN ('initializing', 'fetching', 'extracting', 'optimizing', 'exporting', 'completed', 'failed')),
    progress_pct SMALLINT NOT NULL DEFAULT 0 CHECK (progress_pct >= 0 AND progress_pct <= 100),
    leased_by TEXT NULL,
    lease_expires_at TIMESTAMPTZ NULL,
    attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    error_code TEXT NULL,
    error_message TEXT NULL,
    owner_session_id UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NULL
);

-- Indexes for jobs table
CREATE INDEX IF NOT EXISTS idx_jobs_status_lease ON jobs (status, lease_expires_at);
CREATE INDEX IF NOT EXISTS idx_jobs_owner_session ON jobs (owner_session_id);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs (created_at DESC);

-- Trigger for jobs updated_at auto-update
DROP TRIGGER IF EXISTS trg_jobs_updated_at ON jobs;
CREATE TRIGGER trg_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION fn_set_updated_at();

COMMENT ON TABLE jobs IS 'Core conversion job records, progress state, and distributed worker leases';
COMMENT ON COLUMN jobs.id IS 'UUID v7 primary key of the conversion job';
COMMENT ON COLUMN jobs.kind IS 'Source extraction category';
COMMENT ON COLUMN jobs.status IS 'Job lifecycle status';
COMMENT ON COLUMN jobs.source_url IS 'Initial target URL or repository address';
COMMENT ON COLUMN jobs.config_json IS 'Job configuration JSON (token budget, filters, format preferences)';
COMMENT ON COLUMN jobs.progress_phase IS 'Active execution pipeline phase';
COMMENT ON COLUMN jobs.progress_pct IS 'Completion percentage (0 to 100)';
COMMENT ON COLUMN jobs.leased_by IS 'Worker instance identifier holding active lease';
COMMENT ON COLUMN jobs.lease_expires_at IS 'Lease expiration deadline for heartbeat recovery';
COMMENT ON COLUMN jobs.attempt_count IS 'Total execution attempts for retry tracking';
COMMENT ON COLUMN jobs.owner_session_id IS 'Anonymous web session or user identifier';

-- 2. JOB ARTIFACTS TABLE: Exported bundles and canonical intermediate representations
CREATE TABLE IF NOT EXISTS job_artifacts (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    format TEXT NOT NULL CHECK (format IN ('cdm_json', 'markdown', 'txt', 'json', 'html', 'jsonl', 'zip', 'manifest', 'quality_report')),
    storage_key TEXT NOT NULL CHECK (length(storage_key) > 0),
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    sha256_hex TEXT NOT NULL CHECK (length(sha256_hex) = 64),
    blake3_hex TEXT NOT NULL CHECK (length(blake3_hex) = 64),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_job_artifacts_job_format UNIQUE (job_id, format)
);

-- Indexes for job_artifacts table
CREATE INDEX IF NOT EXISTS idx_job_artifacts_job_id ON job_artifacts (job_id);
CREATE INDEX IF NOT EXISTS idx_job_artifacts_expires_at ON job_artifacts (expires_at);

COMMENT ON TABLE job_artifacts IS 'Metadata and storage references for generated job output artifacts';
COMMENT ON COLUMN job_artifacts.id IS 'UUID v7 identifier of the artifact record';
COMMENT ON COLUMN job_artifacts.job_id IS 'Foreign key referencing parent jobs.id with CASCADE deletion';
COMMENT ON COLUMN job_artifacts.format IS 'Export file format identifier';
COMMENT ON COLUMN job_artifacts.storage_key IS 'Relative path or object store key in crates/storage';
COMMENT ON COLUMN job_artifacts.byte_size IS 'Exact artifact file size in bytes';
COMMENT ON COLUMN job_artifacts.sha256_hex IS '64-character lowercase hexadecimal SHA-256 digest';
COMMENT ON COLUMN job_artifacts.blake3_hex IS '64-character lowercase hexadecimal BLAKE3 digest';
COMMENT ON COLUMN job_artifacts.expires_at IS 'TTL timestamp for ephemeral storage cleanup';

-- 3. GITHUB INSTALLATIONS TABLE: GitHub App authorizations and encrypted credentials
CREATE TABLE IF NOT EXISTS github_installations (
    id UUID PRIMARY KEY,
    owner_session_id UUID NOT NULL,
    github_installation_id BIGINT NOT NULL CHECK (github_installation_id > 0),
    account_login TEXT NOT NULL CHECK (length(account_login) > 0),
    encrypted_token BYTEA NULL,
    token_nonce BYTEA NULL,
    token_expires_at TIMESTAMPTZ NULL,
    revoked_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_github_installations_owner_gh_id UNIQUE (owner_session_id, github_installation_id)
);

-- Indexes for github_installations table
CREATE INDEX IF NOT EXISTS idx_github_installations_owner ON github_installations (owner_session_id);

-- Trigger for github_installations updated_at auto-update
DROP TRIGGER IF EXISTS trg_github_installations_updated_at ON github_installations;
CREATE TRIGGER trg_github_installations_updated_at
    BEFORE UPDATE ON github_installations
    FOR EACH ROW
    EXECUTE FUNCTION fn_set_updated_at();

COMMENT ON TABLE github_installations IS 'GitHub App user installation grants and envelope-encrypted tokens';
COMMENT ON COLUMN github_installations.id IS 'UUID primary key of the installation link';
COMMENT ON COLUMN github_installations.owner_session_id IS 'Session or user owning the installation grant';
COMMENT ON COLUMN github_installations.github_installation_id IS 'Upstream GitHub App installation numeric ID';
COMMENT ON COLUMN github_installations.account_login IS 'GitHub organization or user login handle';
COMMENT ON COLUMN github_installations.encrypted_token IS 'AES-256-GCM ciphertext of short-lived installation token';
COMMENT ON COLUMN github_installations.token_nonce IS 'Cryptographic nonce used for AES-256-GCM encryption';
COMMENT ON COLUMN github_installations.token_expires_at IS 'Token expiration deadline from GitHub API';
COMMENT ON COLUMN github_installations.revoked_at IS 'Revocation timestamp if disconnected by user';

-- 4. CONTENT CACHE INDEX TABLE: Content-addressed deduplication and caching
CREATE TABLE IF NOT EXISTS content_cache_index (
    cache_key TEXT PRIMARY KEY CHECK (length(cache_key) > 0),
    blake3_hex TEXT NOT NULL CHECK (length(blake3_hex) = 64),
    storage_key TEXT NOT NULL CHECK (length(storage_key) > 0),
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for content_cache_index table
CREATE INDEX IF NOT EXISTS idx_content_cache_last_accessed ON content_cache_index (last_accessed_at);

COMMENT ON TABLE content_cache_index IS 'Content-addressed cache index mapping blob signatures to storage keys';
COMMENT ON COLUMN content_cache_index.cache_key IS 'Composite cache key (e.g., git commit blob SHA or URL validator)';
COMMENT ON COLUMN content_cache_index.blake3_hex IS '64-character lowercase hexadecimal BLAKE3 digest';
COMMENT ON COLUMN content_cache_index.storage_key IS 'Location of cached content blob in crates/storage';
COMMENT ON COLUMN content_cache_index.byte_size IS 'Cached payload size in bytes';
COMMENT ON COLUMN content_cache_index.last_accessed_at IS 'Timestamp of most recent cache hit for LRU eviction';
