//! # Content Cache Index Entity Schema
//!
//! Strongly-typed Rust models for the `content_cache_index` relational table defined in `docs/DATA_MODEL.md`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::error::CoreError;

/// Relational record representing a cached content blob in the `content_cache_index` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContentCacheIndexRow {
    /// Unique cache key (e.g., git commit blob SHA or URL validator hash).
    pub cache_key: String,
    /// 64-character lowercase hexadecimal BLAKE3 digest of the content payload.
    pub blake3_hex: String,
    /// Storage key pointing to the raw content in object store or local disk.
    pub storage_key: String,
    /// Exact byte size of the cached payload.
    pub byte_size: i64,
    /// Timestamp of most recent cache hit or access.
    pub last_accessed_at: DateTime<Utc>,
}

impl ContentCacheIndexRow {
    /// Constructs a new cache index record with current timestamp.
    #[must_use]
    pub fn new(
        cache_key: impl Into<String>,
        blake3_hex: impl Into<String>,
        storage_key: impl Into<String>,
        byte_size: i64,
    ) -> Self {
        Self {
            cache_key: cache_key.into(),
            blake3_hex: blake3_hex.into().to_ascii_lowercase(),
            storage_key: storage_key.into(),
            byte_size,
            last_accessed_at: Utc::now(),
        }
    }

    /// Updates `last_accessed_at` timestamp to the current time.
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Sets an explicit `last_accessed_at` timestamp.
    #[must_use]
    pub fn with_last_accessed_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.last_accessed_at = timestamp;
        self
    }

    /// Returns human-readable formatted byte size (e.g., "1.23 KB", "4.56 MB").
    #[must_use]
    pub fn formatted_byte_size(&self) -> String {
        let bytes = self.byte_size.max(0) as f64;
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        if bytes >= GB {
            format!("{:.2} GB", bytes / GB)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes / MB)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes / KB)
        } else {
            format!("{} B", self.byte_size)
        }
    }

    /// Validates table constraints and cryptographic hash lengths.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.cache_key.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "cache_key".to_string(),
                reason: "cache_key cannot be empty".to_string(),
            });
        }

        if self.storage_key.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "storage_key".to_string(),
                reason: "storage_key cannot be empty".to_string(),
            });
        }

        if self.byte_size < 0 {
            return Err(CoreError::Validation {
                field: "byte_size".to_string(),
                reason: format!("byte_size cannot be negative (actual: {})", self.byte_size),
            });
        }

        if self.blake3_hex.len() != 64 || !self.blake3_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "blake3_hex".to_string(),
                reason: format!(
                    "blake3_hex must be exactly 64 hexadecimal characters (actual: '{}')",
                    self.blake3_hex
                ),
            });
        }

        Ok(())
    }
}
