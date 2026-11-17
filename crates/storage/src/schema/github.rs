//! # GitHub Installation Entity Schema
//!
//! Strongly-typed Rust models for the `github_installations` relational table defined in `docs/DATA_MODEL.md`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;

/// Relational record representing a GitHub App installation in the `github_installations` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitHubInstallationRow {
    /// Unique UUID identifier for the installation record.
    pub id: Uuid,
    /// Anonymous web session or user account owning this grant.
    pub owner_session_id: Uuid,
    /// Upstream numeric installation ID issued by GitHub App platform.
    pub github_installation_id: i64,
    /// GitHub user or organization login handle.
    pub account_login: String,
    /// AES-256-GCM ciphertext bytes of short-lived installation access token.
    pub encrypted_token: Option<Vec<u8>>,
    /// 12-byte initialization vector / nonce used for AES-256-GCM encryption.
    pub token_nonce: Option<Vec<u8>>,
    /// Token expiration deadline from GitHub API.
    pub token_expires_at: Option<DateTime<Utc>>,
    /// Revocation timestamp if authorization was revoked.
    pub revoked_at: Option<DateTime<Utc>>,
    /// Record creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Record last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl GitHubInstallationRow {
    /// Constructs a new `GitHubInstallationRow` for an authorized GitHub account.
    #[must_use]
    pub fn new(
        owner_session_id: Uuid,
        github_installation_id: i64,
        account_login: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            owner_session_id,
            github_installation_id,
            account_login: account_login.into(),
            encrypted_token: None,
            token_nonce: None,
            token_expires_at: None,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Attaches envelope-encrypted installation credentials.
    #[must_use]
    pub fn with_encrypted_token(
        mut self,
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        self.encrypted_token = Some(ciphertext);
        self.token_nonce = Some(nonce);
        self.token_expires_at = Some(expires_at);
        self
    }

    /// Returns `true` if the installation is active (not revoked and token not expired).
    #[must_use]
    pub fn is_active(&self) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        match self.token_expires_at {
            Some(exp) => exp > Utc::now(),
            None => true,
        }
    }

    /// Returns `true` if this installation grant has been explicitly revoked.
    #[must_use]
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Marks the installation as revoked.
    pub fn revoke(&mut self) {
        self.revoked_at = Some(Utc::now());
        self.encrypted_token = None;
        self.token_nonce = None;
        self.token_expires_at = None;
    }

    /// Validates table constraints and invariants on this installation row.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.github_installation_id <= 0 {
            return Err(CoreError::Validation {
                field: "github_installation_id".to_string(),
                reason: format!(
                    "github_installation_id must be a positive integer (actual: {})",
                    self.github_installation_id
                ),
            });
        }

        if self.account_login.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "account_login".to_string(),
                reason: "account_login cannot be empty".to_string(),
            });
        }

        if let (Some(token), Some(nonce)) = (&self.encrypted_token, &self.token_nonce) {
            if token.is_empty() {
                return Err(CoreError::Validation {
                    field: "encrypted_token".to_string(),
                    reason: "encrypted_token bytes cannot be empty when present".to_string(),
                });
            }
            if nonce.len() != 12 {
                return Err(CoreError::Validation {
                    field: "token_nonce".to_string(),
                    reason: format!(
                        "AES-256-GCM token_nonce must be exactly 12 bytes (actual: {})",
                        nonce.len()
                    ),
                });
            }
        }

        Ok(())
    }
}
