-- Migration: Add index on blake3_hex for content_cache_index table
-- Subphase 8.3.3: Content-addressed storage (CAS)

CREATE INDEX IF NOT EXISTS idx_content_cache_blake3 ON content_cache_index (blake3_hex);
