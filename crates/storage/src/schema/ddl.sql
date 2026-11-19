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

-- 5. JOB STATE TRANSITIONS TABLE: Append-only audit trail of job status transitions
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

-- ============================================================================
-- 5. BATCH JOBS AND BATCH ITEMS SCHEMA
-- ============================================================================

-- Parent tracking records for multi-URL batch conversion submissions
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

-- Index for querying batches by owner session
CREATE INDEX IF NOT EXISTS idx_batch_jobs_owner_session
    ON batch_jobs (owner_session_id) WHERE owner_session_id IS NOT NULL;

-- Index for ordering batches chronologically
CREATE INDEX IF NOT EXISTS idx_batch_jobs_created_at
    ON batch_jobs (created_at DESC);

-- Individual URL task records and execution outcomes within a batch
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

-- Index for fast lookup of a batch item by child job_id
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

-- 7. HTTP VALIDATORS TABLE: URL-level HTTP caching, ETag, Last-Modified, and cache control
CREATE TABLE IF NOT EXISTS http_validators (
    url TEXT PRIMARY KEY CHECK (length(url) > 0),
    canonical_url TEXT NULL,
    etag TEXT NULL,
    last_modified TEXT NULL,
    last_modified_timestamp TIMESTAMPTZ NULL,
    content_type TEXT NULL,
    blake3_hex TEXT NOT NULL CHECK (length(blake3_hex) = 64),
    storage_key TEXT NOT NULL CHECK (length(storage_key) > 0),
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    cache_control TEXT NULL,
    expires_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for http_validators table
CREATE INDEX IF NOT EXISTS idx_http_validators_blake3
    ON http_validators (blake3_hex);

CREATE INDEX IF NOT EXISTS idx_http_validators_last_checked
    ON http_validators (last_checked_at);

CREATE INDEX IF NOT EXISTS idx_http_validators_expires
    ON http_validators (expires_at) WHERE expires_at IS NOT NULL;

COMMENT ON TABLE http_validators IS 'URL-level HTTP caching validators (ETag, Last-Modified, Cache-Control) for conditional HTTP requests and incremental extraction';
COMMENT ON COLUMN http_validators.url IS 'Normalized target URL as primary key';
COMMENT ON COLUMN http_validators.canonical_url IS 'Resolved canonical destination URL after redirects';
COMMENT ON COLUMN http_validators.etag IS 'Exact wire ETag header value (including quotes and weak indicator W/)';
COMMENT ON COLUMN http_validators.last_modified IS 'Raw Last-Modified HTTP date string from server';
COMMENT ON COLUMN http_validators.last_modified_timestamp IS 'Parsed UTC timestamp from Last-Modified header';
COMMENT ON COLUMN http_validators.content_type IS 'MIME content type header (e.g. text/html; charset=utf-8)';
COMMENT ON COLUMN http_validators.blake3_hex IS '64-character lowercase hexadecimal BLAKE3 digest of cached content payload';
COMMENT ON COLUMN http_validators.storage_key IS 'Location of cached content blob in artifact storage backend';
COMMENT ON COLUMN http_validators.byte_size IS 'Byte size of the cached payload';
COMMENT ON COLUMN http_validators.cache_control IS 'Raw Cache-Control header string (max-age, must-revalidate, etc.)';
COMMENT ON COLUMN http_validators.expires_at IS 'Calculated expiration timestamp based on max-age or Expires header';
COMMENT ON COLUMN http_validators.created_at IS 'Timestamp when this URL validator was first registered';
COMMENT ON COLUMN http_validators.last_checked_at IS 'Timestamp when this URL validator was most recently validated or accessed';

-- 8. GIT CACHE TABLES: Repository commit metadata, tree manifests, and content-addressed Git blobs
CREATE TABLE IF NOT EXISTS git_commit_cache (
    owner TEXT NOT NULL CHECK (length(owner) > 0),
    repo TEXT NOT NULL CHECK (length(repo) > 0),
    commit_sha TEXT NOT NULL CHECK (length(commit_sha) >= 7 AND length(commit_sha) <= 64),
    tree_sha TEXT NOT NULL CHECK (length(tree_sha) >= 7 AND length(tree_sha) <= 64),
    tree_entries_json TEXT,
    total_files INTEGER NOT NULL CHECK (total_files >= 0),
    total_bytes BIGINT NOT NULL CHECK (total_bytes >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner, repo, commit_sha)
);

CREATE INDEX IF NOT EXISTS idx_git_commit_cache_repo ON git_commit_cache (owner, repo);
CREATE INDEX IF NOT EXISTS idx_git_commit_cache_last_accessed ON git_commit_cache (last_accessed_at);

CREATE TABLE IF NOT EXISTS git_blob_cache (
    blob_sha TEXT PRIMARY KEY CHECK (length(blob_sha) >= 7 AND length(blob_sha) <= 64),
    blake3_hex TEXT NOT NULL CHECK (length(blake3_hex) = 64),
    storage_key TEXT NOT NULL CHECK (length(storage_key) > 0),
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    is_binary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_git_blob_cache_blake3 ON git_blob_cache (blake3_hex);
CREATE INDEX IF NOT EXISTS idx_git_blob_cache_last_accessed ON git_blob_cache (last_accessed_at);

COMMENT ON TABLE git_commit_cache IS 'Cached Git commit revisions, root tree SHAs, and serialized tree manifests';
COMMENT ON TABLE git_blob_cache IS 'Content-addressed Git blob cache mapping Git SHA-1 hashes to storage keys and BLAKE3 digests';

