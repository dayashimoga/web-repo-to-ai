-- Migration: 20261119000002_http_validators.sql
-- Description: URL-level HTTP caching, ETag and Last-Modified tracking, and content cache index linkage
-- Schema Version: 1.3.0

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
