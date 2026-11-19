-- Migration: 20261119000003_git_cache.sql
-- Subphase: 8.3.2 (Git commit/blob caching in crates/storage and crates/github-extractor)
-- Purpose: Relational persistence for Git commit metadata, cached tree manifests, and content-addressed blob digests.

-- 1. GIT COMMIT CACHE TABLE
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

-- Indexes for git_commit_cache
CREATE INDEX IF NOT EXISTS idx_git_commit_cache_repo ON git_commit_cache (owner, repo);
CREATE INDEX IF NOT EXISTS idx_git_commit_cache_last_accessed ON git_commit_cache (last_accessed_at);

COMMENT ON TABLE git_commit_cache IS 'Cached Git commit revisions, root tree SHAs, and serialized tree manifests';
COMMENT ON COLUMN git_commit_cache.owner IS 'Repository owner login or organization name';
COMMENT ON COLUMN git_commit_cache.repo IS 'Repository name';
COMMENT ON COLUMN git_commit_cache.commit_sha IS 'Immutable Git commit SHA-1 or SHA-256 hex string';
COMMENT ON COLUMN git_commit_cache.tree_sha IS 'Root Git tree SHA associated with the commit';
COMMENT ON COLUMN git_commit_cache.tree_entries_json IS 'Optional serialized JSON of GitTreeEntry items for fast tree hits';
COMMENT ON COLUMN git_commit_cache.total_files IS 'Total count of file blobs in the commit revision';
COMMENT ON COLUMN git_commit_cache.total_bytes IS 'Aggregate byte size across all blobs in the commit';
COMMENT ON COLUMN git_commit_cache.created_at IS 'Timestamp when the commit cache record was first created';
COMMENT ON COLUMN git_commit_cache.last_accessed_at IS 'Timestamp of most recent cache access for pruning';

-- 2. GIT BLOB CACHE TABLE
CREATE TABLE IF NOT EXISTS git_blob_cache (
    blob_sha TEXT PRIMARY KEY CHECK (length(blob_sha) >= 7 AND length(blob_sha) <= 64),
    blake3_hex TEXT NOT NULL CHECK (length(blake3_hex) = 64),
    storage_key TEXT NOT NULL CHECK (length(storage_key) > 0),
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    is_binary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for git_blob_cache
CREATE INDEX IF NOT EXISTS idx_git_blob_cache_blake3 ON git_blob_cache (blake3_hex);
CREATE INDEX IF NOT EXISTS idx_git_blob_cache_last_accessed ON git_blob_cache (last_accessed_at);

COMMENT ON TABLE git_blob_cache IS 'Content-addressed Git blob cache mapping Git SHA-1 hashes to storage keys and BLAKE3 digests';
COMMENT ON COLUMN git_blob_cache.blob_sha IS '40-character Git object SHA-1 hash';
COMMENT ON COLUMN git_blob_cache.blake3_hex IS '64-character lowercase hexadecimal BLAKE3 digest of the blob bytes';
COMMENT ON COLUMN git_blob_cache.storage_key IS 'Durable storage key path where blob contents are preserved';
COMMENT ON COLUMN git_blob_cache.byte_size IS 'Raw blob payload size in bytes';
COMMENT ON COLUMN git_blob_cache.is_binary IS 'Whether content was detected as binary / null bytes';
COMMENT ON COLUMN git_blob_cache.created_at IS 'Timestamp when the blob was first cached';
COMMENT ON COLUMN git_blob_cache.last_accessed_at IS 'Timestamp of most recent cache hit for LRU eviction';
