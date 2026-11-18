//! Data Transfer Objects (DTOs) for GitHub App installation management, revocation, and webhooks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_repo_to_ai_core::{CoreError, Validatable};
use web_repo_to_ai_storage::schema::GitHubInstallationRow;

/// Path parameters for operations targeting a specific GitHub installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationPathParams {
    /// Numeric GitHub installation identifier.
    pub installation_id: i64,
}

impl Validatable for InstallationPathParams {
    fn validate(&self) -> Result<(), CoreError> {
        if self.installation_id <= 0 {
            return Err(CoreError::Validation {
                field: "installation_id".to_string(),
                reason: "installation_id must be a positive non-zero integer".to_string(),
            });
        }
        Ok(())
    }
}

/// Response payload returned upon successful revocation of an installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeInstallationResponse {
    /// Operation status string (e.g. `"revoked"`).
    pub status: String,
    /// Tenant session ID that initiated the revocation.
    pub session_id: Uuid,
    /// Numeric GitHub installation ID.
    pub installation_id: i64,
    /// UTC timestamp of revocation.
    pub revoked_at: DateTime<Utc>,
    /// Whether GitHub API was invoked and confirmed invalidation.
    pub upstream_revoked: bool,
}

/// Safe public summary of an active GitHub installation (strictly zero secret leakage).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationSummaryResponse {
    /// Numeric GitHub installation ID.
    pub installation_id: i64,
    /// Account handle or organization name.
    pub account_login: String,
    /// Whether the installation has been revoked.
    pub is_revoked: bool,
    /// Record creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Record last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Expiration timestamp of the current access token, if active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<DateTime<Utc>>,
}

impl From<&GitHubInstallationRow> for InstallationSummaryResponse {
    fn from(row: &GitHubInstallationRow) -> Self {
        Self {
            installation_id: row.github_installation_id,
            account_login: row.account_login.clone(),
            is_revoked: row.is_revoked(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            token_expires_at: row.token_expires_at,
        }
    }
}

/// Response payload returned after processing an incoming GitHub webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookReceiptResponse {
    /// Status of the receipt (`"processed"`, `"acknowledged"`, or `"ignored"`).
    pub status: String,
    /// Event name from `X-GitHub-Event` (e.g. `"installation"`, `"ping"`).
    pub event: String,
    /// Action name if applicable (e.g. `"deleted"`, `"suspend"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Installation ID affected, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<i64>,
    /// Number of storage records revoked as a result of this webhook.
    pub revoked_count: u64,
    /// UTC timestamp of webhook receipt and processing.
    pub received_at: DateTime<Utc>,
}
