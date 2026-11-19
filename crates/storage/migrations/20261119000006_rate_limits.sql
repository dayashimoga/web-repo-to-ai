-- Migration: 20261119000006_rate_limits.sql
-- Description: Sliding window and persistent rate limiting state table
-- Enforces per-client key and time bucket request counting with TTL indexing.

CREATE TABLE IF NOT EXISTS rate_limit_counters (
    client_key VARCHAR(128) NOT NULL,
    window_bucket BIGINT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (client_key, window_bucket)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_expires ON rate_limit_counters(expires_at);
