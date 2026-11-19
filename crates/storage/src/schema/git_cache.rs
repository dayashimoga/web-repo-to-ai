//! # Git Cache Entity Models
//!
//! Strongly-typed Rust models for `git_commit_cache` and `git_blob_cache` relational tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::error::CoreError;

/// Relational record representing a cached Git commit revision in `git_commit_cache`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitCommitCacheRow {
    /// Repository owner login or organization name.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Git commit SHA-1 or SHA-256 hex string.
    pub commit_sha: String,
    /// Root Git tree SHA associated with the commit.
    pub tree_sha: String,
    /// Optional serialized JSON of GitTreeEntry items for fast tree hits.
    pub tree_entries_json: Option<String>,
    /// Total count of file blobs in the commit revision.
    pub total_files: i32,
    /// Aggregate byte size across all blobs in the commit.
    pub total_bytes: i64,
    /// Timestamp when the commit cache record was first created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of most recent cache access for LRU eviction.
    pub last_accessed_at: DateTime<Utc>,
}

impl GitCommitCacheRow {
    /// Creates a new `GitCommitCacheRow` instance with default timestamps.
    #[must_use]
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        commit_sha: impl Into<String>,
        tree_sha: impl Into<String>,
        total_files: i32,
        total_bytes: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            owner: owner.into(),
            repo: repo.into(),
            commit_sha: commit_sha.into(),
            tree_sha: tree_sha.into(),
            tree_entries_json: None,
            total_files,
            total_bytes,
            created_at: now,
            last_accessed_at: now,
        }
    }

    /// Sets optional serialized JSON for tree entries.
    #[must_use]
    pub fn with_tree_entries_json(mut self, json: impl Into<String>) -> Self {
        self.tree_entries_json = Some(json.into());
        self
    }

    /// Validates field invariants and database constraints.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if any constraint is violated.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.owner.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "owner".to_string(),
                reason: "owner cannot be empty in GitCommitCacheRow".to_string(),
            });
        }
        if self.repo.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "repo".to_string(),
                reason: "repo cannot be empty in GitCommitCacheRow".to_string(),
            });
        }
        if self.commit_sha.len() < 7 || self.commit_sha.len() > 64 {
            return Err(CoreError::Validation {
                field: "commit_sha".to_string(),
                reason: format!(
                    "commit_sha must be between 7 and 64 characters (got {})",
                    self.commit_sha.len()
                ),
            });
        }
        if self.tree_sha.len() < 7 || self.tree_sha.len() > 64 {
            return Err(CoreError::Validation {
                field: "tree_sha".to_string(),
                reason: format!(
                    "tree_sha must be between 7 and 64 characters (got {})",
                    self.tree_sha.len()
                ),
            });
        }
        if self.total_files < 0 {
            return Err(CoreError::Validation {
                field: "total_files".to_string(),
                reason: "total_files cannot be negative".to_string(),
            });
        }
        if self.total_bytes < 0 {
            return Err(CoreError::Validation {
                field: "total_bytes".to_string(),
                reason: "total_bytes cannot be negative".to_string(),
            });
        }
        Ok(())
    }
}

/// Relational record representing a cached Git blob in `git_blob_cache`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitBlobCacheRow {
    /// 40-character Git object SHA-1 hash.
    pub blob_sha: String,
    /// 64-character lowercase hexadecimal BLAKE3 digest of the content.
    pub blake3_hex: String,
    /// Storage key path where blob contents are preserved.
    pub storage_key: String,
    /// Byte length of the raw content.
    pub byte_size: i64,
    /// Whether content was detected as binary.
    pub is_binary: bool,
    /// Timestamp when the blob was first cached.
    pub created_at: DateTime<Utc>,
    /// Timestamp of most recent cache access for LRU eviction.
    pub last_accessed_at: DateTime<Utc>,
}

impl GitBlobCacheRow {
    /// Creates a new `GitBlobCacheRow` instance with default timestamps.
    #[must_use]
    pub fn new(
        blob_sha: impl Into<String>,
        blake3_hex: impl Into<String>,
        storage_key: impl Into<String>,
        byte_size: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            blob_sha: blob_sha.into(),
            blake3_hex: blake3_hex.into(),
            storage_key: storage_key.into(),
            byte_size,
            is_binary: false,
            created_at: now,
            last_accessed_at: now,
        }
    }

    /// Sets the binary content indicator.
    #[must_use]
    pub fn with_binary(mut self, is_binary: bool) -> Self {
        self.is_binary = is_binary;
        self
    }

    /// Validates field invariants and database constraints.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if any constraint is violated.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.blob_sha.len() < 7 || self.blob_sha.len() > 64 {
            return Err(CoreError::Validation {
                field: "blob_sha".to_string(),
                reason: format!(
                    "blob_sha must be between 7 and 64 characters (got {})",
                    self.blob_sha.len()
                ),
            });
        }
        if self.blake3_hex.len() != 64 {
            return Err(CoreError::Validation {
                field: "blake3_hex".to_string(),
                reason: format!(
                    "blake3_hex must be exactly 64 characters (got {})",
                    self.blake3_hex.len()
                ),
            });
        }
        if self.storage_key.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "storage_key".to_string(),
                reason: "storage_key cannot be empty in GitBlobCacheRow".to_string(),
            });
        }
        if self.byte_size < 0 {
            return Err(CoreError::Validation {
                field: "byte_size".to_string(),
                reason: "byte_size cannot be negative in GitBlobCacheRow".to_string(),
            });
        }
        Ok(())
    }
}
