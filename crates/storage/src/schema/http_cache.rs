//! # HTTP Validator and Cache Entity Schema
//!
//! Strongly-typed Rust entity models for the `http_validators` relational table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::error::CoreError;

/// Relational record representing cached HTTP validators and metadata in `http_validators`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct HttpValidatorRow {
    /// Normalized target URL acting as the primary key.
    pub url: String,
    /// Resolved canonical destination URL after redirects, if distinct.
    pub canonical_url: Option<String>,
    /// Exact wire ETag header value (including quotes and weak indicator W/).
    pub etag: Option<String>,
    /// Raw Last-Modified HTTP date string from server.
    pub last_modified: Option<String>,
    /// Parsed UTC timestamp from Last-Modified header if valid.
    pub last_modified_timestamp: Option<DateTime<Utc>>,
    /// MIME content type header (e.g. text/html; charset=utf-8).
    pub content_type: Option<String>,
    /// 64-character lowercase hexadecimal BLAKE3 digest of cached content payload.
    pub blake3_hex: String,
    /// Location of cached content blob in artifact storage backend.
    pub storage_key: String,
    /// Byte size of the cached payload.
    pub byte_size: i64,
    /// Raw Cache-Control header string (max-age, must-revalidate, etc.).
    pub cache_control: Option<String>,
    /// Calculated expiration timestamp based on max-age or Expires header.
    pub expires_at: Option<DateTime<Utc>>,
    /// Timestamp when this URL validator was first registered.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this URL validator was most recently validated or accessed.
    pub last_checked_at: DateTime<Utc>,
}

impl HttpValidatorRow {
    /// Constructs a new `HttpValidatorRow` with required fields and current timestamps.
    #[must_use]
    pub fn new(
        url: impl Into<String>,
        blake3_hex: impl Into<String>,
        storage_key: impl Into<String>,
        byte_size: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            url: url.into(),
            canonical_url: None,
            etag: None,
            last_modified: None,
            last_modified_timestamp: None,
            content_type: None,
            blake3_hex: blake3_hex.into().to_ascii_lowercase(),
            storage_key: storage_key.into(),
            byte_size,
            cache_control: None,
            expires_at: None,
            created_at: now,
            last_checked_at: now,
        }
    }

    /// Sets the entity tag (`ETag`).
    #[must_use]
    pub fn with_etag(mut self, etag: Option<String>) -> Self {
        self.etag = etag;
        self
    }

    /// Sets the `Last-Modified` string and parsed timestamp.
    #[must_use]
    pub fn with_last_modified(
        mut self,
        last_modified: Option<String>,
        last_modified_timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        self.last_modified = last_modified;
        self.last_modified_timestamp = last_modified_timestamp;
        self
    }

    /// Sets the MIME content type.
    #[must_use]
    pub fn with_content_type(mut self, content_type: Option<String>) -> Self {
        self.content_type = content_type;
        self
    }

    /// Sets the `Cache-Control` header string.
    #[must_use]
    pub fn with_cache_control(mut self, cache_control: Option<String>) -> Self {
        self.cache_control = cache_control;
        self
    }

    /// Sets the expiration timestamp.
    #[must_use]
    pub fn with_expires_at(mut self, expires_at: Option<DateTime<Utc>>) -> Self {
        self.expires_at = expires_at;
        self
    }

    /// Sets the canonical destination URL.
    #[must_use]
    pub fn with_canonical_url(mut self, canonical_url: Option<String>) -> Self {
        self.canonical_url = canonical_url;
        self
    }

    /// Updates `last_checked_at` timestamp and refreshes validators on successful 304 Not Modified.
    pub fn touch(
        &mut self,
        updated_etag: Option<&str>,
        updated_last_modified: Option<&str>,
        updated_expires_at: Option<DateTime<Utc>>,
    ) {
        self.last_checked_at = Utc::now();
        if let Some(etag) = updated_etag {
            self.etag = Some(etag.to_string());
        }
        if let Some(lm) = updated_last_modified {
            self.last_modified = Some(lm.to_string());
        }
        if updated_expires_at.is_some() {
            self.expires_at = updated_expires_at;
        }
    }

    /// Validates table constraints and hash invariants.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.url.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "url".to_string(),
                reason: "url cannot be empty".to_string(),
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
