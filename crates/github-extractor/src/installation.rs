//! # GitHub App Installation & Token Models
//!
//! Strongly-typed domain representations for GitHub App installations, authorized
//! account profiles, installed repository summaries, and short-lived installation access tokens.
//!
//! ## Rule 05 Compliance
//! - Installation access tokens (`ghs_*`) are strictly masked in [`fmt::Debug`] and
//!   [`fmt::Display`] implementations to prevent token leakage in traces and log entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

/// Scope of repositories granted to the GitHub App installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RepositorySelection {
    /// App has access to all current and future repositories in the account.
    All,
    /// App has access only to explicitly selected repositories.
    #[default]
    Selected,
}

impl RepositorySelection {
    /// Returns the string representation expected by GitHub API.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Selected => "selected",
        }
    }
}

impl fmt::Display for RepositorySelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for RepositorySelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RepositorySelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(Self::All),
            "selected" => Ok(Self::Selected),
            other => Err(serde::de::Error::custom(format!(
                "Unknown repository selection '{other}'; expected 'all' or 'selected'"
            ))),
        }
    }
}

/// Concise summary of an account (User or Organization) owning an installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAccountSummary {
    /// Numeric GitHub account ID.
    pub id: u64,
    /// Login handle / organization name.
    pub login: String,
    /// Account type (`"User"` or `"Organization"`).
    #[serde(rename = "type")]
    pub account_type: String,
    /// Profile avatar image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Web profile URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
}

/// A registered installation of the GitHub App on a user or organization account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubInstallation {
    /// Unique installation identifier.
    pub id: u64,
    /// Account where the app is installed.
    pub account: GitHubAccountSummary,
    /// Repository access mode (`all` or `selected`).
    pub repository_selection: RepositorySelection,
    /// Direct API endpoint for requesting installation access tokens.
    #[serde(default)]
    pub access_tokens_url: String,
    /// Direct API endpoint for listing repositories accessible to this installation.
    #[serde(default)]
    pub repositories_url: String,
    /// Web settings URL for managing this installation on GitHub.
    #[serde(default)]
    pub html_url: String,
    /// Numeric ID of the installed App.
    pub app_id: u64,
    /// URL slug of the installed App.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_slug: Option<String>,
    /// Target numeric ID (user or org ID).
    pub target_id: u64,
    /// Target type (`"User"` or `"Organization"`).
    pub target_type: String,
    /// Granted permission scopes map (e.g. `{"contents": "read", "metadata": "read"}`).
    #[serde(default)]
    pub permissions: HashMap<String, String>,
    /// Webhook events subscribed by this installation.
    #[serde(default)]
    pub events: Vec<String>,
    /// Timestamp when the installation was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the installation was last updated.
    pub updated_at: DateTime<Utc>,
    /// Optional suspension timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_at: Option<DateTime<Utc>>,
    /// Account that suspended the installation, if suspended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended_by: Option<GitHubAccountSummary>,
}

impl GitHubInstallation {
    /// Returns true if this installation is currently suspended.
    #[must_use]
    pub fn is_suspended(&self) -> bool {
        self.suspended_at.is_some()
    }

    /// Returns true if installed on an organization account.
    #[must_use]
    pub fn is_organization(&self) -> bool {
        self.target_type.eq_ignore_ascii_case("organization")
            || self
                .account
                .account_type
                .eq_ignore_ascii_case("organization")
    }

    /// Audits this installation's permissions against the strict minimal permission policy under Rule 05.
    #[must_use]
    pub fn audit_permissions(&self) -> crate::permissions::PermissionAuditReport {
        crate::permissions::PermissionAuditor::audit_installation(
            self,
            &crate::permissions::PermissionPolicy::strict_minimal(),
        )
    }

    /// Audits this installation's permissions against the specified permission policy.
    #[must_use]
    pub fn audit_permissions_with_policy(
        &self,
        policy: &crate::permissions::PermissionPolicy,
    ) -> crate::permissions::PermissionAuditReport {
        crate::permissions::PermissionAuditor::audit_installation(self, policy)
    }

    /// Validates that this installation complies with the strict minimal permission policy.
    ///
    /// # Errors
    /// Returns [`crate::error::GitHubExtractorError::PermissionViolation`] if non-compliant.
    pub fn validate_permissions(&self) -> Result<(), crate::error::GitHubExtractorError> {
        crate::permissions::PermissionAuditor::validate_installation(
            self,
            &crate::permissions::PermissionPolicy::strict_minimal(),
        )
    }

    /// Validates that this installation complies with the specified permission policy.
    ///
    /// # Errors
    /// Returns [`crate::error::GitHubExtractorError::PermissionViolation`] if non-compliant.
    pub fn validate_permissions_with_policy(
        &self,
        policy: &crate::permissions::PermissionPolicy,
    ) -> Result<(), crate::error::GitHubExtractorError> {
        crate::permissions::PermissionAuditor::validate_installation(self, policy)
    }
}

/// Summary of a repository accessible to an installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledRepositorySummary {
    /// Unique numeric repository ID.
    pub id: u64,
    /// GraphQL node ID.
    #[serde(default)]
    pub node_id: String,
    /// Repository name (e.g. `"core-engine"`).
    pub name: String,
    /// Full repository slug (e.g. `"acme-corp/core-engine"`).
    pub full_name: String,
    /// True if the repository is private.
    #[serde(default)]
    pub private: bool,
    /// Default branch name (e.g. `"main"`).
    #[serde(default = "default_main")]
    pub default_branch: String,
    /// Repository description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Web URL on GitHub.
    #[serde(default)]
    pub html_url: String,
}

fn default_main() -> String {
    "main".to_string()
}

/// Repository permission flags returned by GitHub REST API for an authenticated identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RepositoryPermissions {
    /// Read/clone/pull access.
    #[serde(default)]
    pub pull: bool,
    /// Push/write access.
    #[serde(default)]
    pub push: bool,
    /// Administrative access.
    #[serde(default)]
    pub admin: bool,
    /// Triage access.
    #[serde(default)]
    pub triage: bool,
    /// Maintain access.
    #[serde(default)]
    pub maintain: bool,
}

/// Detailed repository metadata and access permissions verified for an installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryAccessDetails {
    /// Numeric repository ID.
    pub id: u64,
    /// Short repository name (e.g. "core-engine").
    pub name: String,
    /// Full repository slug (e.g. "acme/core-engine").
    pub full_name: String,
    /// Whether the repository is private.
    #[serde(default)]
    pub private: bool,
    /// Default branch name.
    #[serde(default = "default_main")]
    pub default_branch: String,
    /// Repository description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Web URL on GitHub.
    #[serde(default)]
    pub html_url: String,
    /// Specific permissions granted to the requesting token.
    #[serde(default)]
    pub permissions: Option<RepositoryPermissions>,
}

impl From<RepositoryAccessDetails> for InstalledRepositorySummary {
    fn from(details: RepositoryAccessDetails) -> Self {
        Self {
            id: details.id,
            node_id: String::new(),
            name: details.name,
            full_name: details.full_name,
            private: details.private,
            default_branch: details.default_branch,
            description: details.description,
            html_url: details.html_url,
        }
    }
}

/// Short-lived installation access token for executing authenticated repository operations.
///
/// Under Rule 05, the `token` string is strictly masked in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone, Serialize, Deserialize)]
pub struct GitHubInstallationToken {
    /// The ephemeral bearer token (sensitive).
    pub token: String,
    /// UTC timestamp when the token expires (typically 60 minutes from issuance).
    pub expires_at: DateTime<Utc>,
    /// Granted permission map for this token.
    #[serde(default)]
    pub permissions: HashMap<String, String>,
    /// Repository selection mode.
    #[serde(default)]
    pub repository_selection: RepositorySelection,
    /// Optional explicit list of repositories authorized for this token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<InstalledRepositorySummary>>,
}

impl fmt::Debug for GitHubInstallationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitHubInstallationToken")
            .field("token", &"***[REDACTED_TOKEN]***")
            .field("expires_at", &self.expires_at)
            .field("permissions", &self.permissions)
            .field("repository_selection", &self.repository_selection)
            .field(
                "repositories_count",
                &self.repositories.as_ref().map(|r| r.len()),
            )
            .finish()
    }
}

impl fmt::Display for GitHubInstallationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GitHubInstallationToken(expires_at={}, selection={})",
            self.expires_at, self.repository_selection
        )
    }
}

impl GitHubInstallationToken {
    /// Accessor for the token string.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns true if the token is already expired according to current UTC time.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Returns true if the token will expire within the given safety margin (in seconds).
    #[must_use]
    pub fn is_expiring_soon(&self, safety_margin_secs: u64) -> bool {
        let margin = chrono::Duration::seconds(safety_margin_secs as i64);
        Utc::now() + margin >= self.expires_at
    }

    /// Audits this installation token's permissions against the strict minimal permission policy under Rule 05.
    #[must_use]
    pub fn audit_permissions(&self) -> crate::permissions::PermissionAuditReport {
        crate::permissions::PermissionAuditor::audit_installation_token(
            self,
            &crate::permissions::PermissionPolicy::strict_minimal(),
        )
    }

    /// Audits this installation token's permissions against the specified permission policy.
    #[must_use]
    pub fn audit_permissions_with_policy(
        &self,
        policy: &crate::permissions::PermissionPolicy,
    ) -> crate::permissions::PermissionAuditReport {
        crate::permissions::PermissionAuditor::audit_installation_token(self, policy)
    }

    /// Validates that this installation token complies with the strict minimal permission policy.
    ///
    /// # Errors
    /// Returns [`crate::error::GitHubExtractorError::PermissionViolation`] if non-compliant.
    pub fn validate_permissions(&self) -> Result<(), crate::error::GitHubExtractorError> {
        crate::permissions::PermissionAuditor::validate_installation_token(
            self,
            &crate::permissions::PermissionPolicy::strict_minimal(),
        )
    }

    /// Validates that this installation token complies with the specified permission policy.
    ///
    /// # Errors
    /// Returns [`crate::error::GitHubExtractorError::PermissionViolation`] if non-compliant.
    pub fn validate_permissions_with_policy(
        &self,
        policy: &crate::permissions::PermissionPolicy,
    ) -> Result<(), crate::error::GitHubExtractorError> {
        crate::permissions::PermissionAuditor::validate_installation_token(self, policy)
    }
}

/// Request payload for creating a scoped installation access token via `POST /app/installations/{id}/access_tokens`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InstallationTokenRequest {
    /// Explicit list of repository names to scope the token to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<String>>,
    /// Explicit list of repository IDs to scope the token to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_ids: Option<Vec<u64>>,
    /// Explicit permissions to scope the token to (strictly read-only under Rule 05).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<crate::app_manifest::GitHubAppPermissions>,
}

impl InstallationTokenRequest {
    /// Creates a token request enforcing minimal read-only permissions (`contents: read`, `metadata: read`).
    #[must_use]
    pub fn minimal_read_only() -> Self {
        Self {
            repositories: None,
            repository_ids: None,
            permissions: Some(crate::app_manifest::GitHubAppPermissions::minimal_read_only()),
        }
    }

    /// Creates a token request scoped to specific repository names with minimal read-only permissions.
    #[must_use]
    pub fn for_repositories(repositories: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            repositories: Some(repositories.into_iter().map(Into::into).collect()),
            repository_ids: None,
            permissions: Some(crate::app_manifest::GitHubAppPermissions::minimal_read_only()),
        }
    }

    /// Sets explicit repository IDs for this request.
    #[must_use]
    pub fn with_repository_ids(mut self, ids: Vec<u64>) -> Self {
        self.repository_ids = Some(ids);
        self
    }
}

/// Response payload from `GET /user/installations`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInstallationsResponse {
    /// Total number of installations accessible to the user.
    pub total_count: usize,
    /// List of installations.
    pub installations: Vec<GitHubInstallation>,
}

/// Response payload from `GET /installation/repositories`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationRepositoriesResponse {
    /// Total number of accessible repositories.
    pub total_count: usize,
    /// Repository selection mode (`all` or `selected`).
    pub repository_selection: RepositorySelection,
    /// List of accessible repositories.
    pub repositories: Vec<InstalledRepositorySummary>,
}
