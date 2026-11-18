//! # GitHub App Installation Authorization Engine
//!
//! Provides authorization evaluation for private GitHub repository extraction.
//! Verifies tenant session scoping, installation ownership, repository access
//! selection (`All` vs `Selected`), Rule 05 read-only permissions compliance,
//! and proactive token renewal before private tree or source extraction.
//!
//! ## Rule 05 Compliance & Architectural Invariants
//!
//! 1. **Tenant Isolation:** Access requires an active installation registered under the
//!    calling tenant's `owner_session_id`. Cross-session token access is strictly denied.
//! 2. **Repository Access Grant Verification:**
//!    - For `repository_selection: all`: All repositories under the installation's account are authorized.
//!    - For `repository_selection: selected`: The requested repository must either be present in the
//!      cached authorized repository list or verified via live `GET /repos/{owner}/{repo}` with the installation token.
//! 3. **Least Privilege Enforcement:** Validates that the installation's granted permissions comply with
//!    the configured [`PermissionPolicy`] (strictly `contents: read`, `metadata: read`; write and admin scopes are rejected).
//! 4. **Secret Masking:** Plaintext bearer tokens in [`AuthorizedInstallationContext`] are strictly masked in
//!    [`fmt::Debug`] and [`fmt::Display`] implementations.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::client::GitHubApiClient;
use crate::error::GitHubExtractorError;
use crate::installation::RepositorySelection;
use crate::permissions::{PermissionAuditor, PermissionPolicy};
use crate::token_refresh::{InstallationTokenRefresher, InstallationTokenStore};

/// Fully authorized installation context for executing private repository extraction.
#[derive(Clone, PartialEq, Eq)]
pub struct AuthorizedInstallationContext {
    /// Unique GitHub App installation identifier.
    pub installation_id: i64,
    /// Account owner or organization login handle.
    pub account_login: String,
    /// Ephemeral bearer installation token (sensitive).
    pub token: String,
    /// Scope of repository access granted to the installation (`all` or `selected`).
    pub repository_selection: RepositorySelection,
    /// Target repository owner name.
    pub owner: String,
    /// Target repository name.
    pub repo: String,
    /// Granted permissions map.
    pub permissions: HashMap<String, String>,
}

impl AuthorizedInstallationContext {
    /// Creates a new `AuthorizedInstallationContext`.
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        installation_id: i64,
        account_login: impl Into<String>,
        token: impl Into<String>,
        repository_selection: RepositorySelection,
    ) -> Self {
        Self {
            installation_id,
            account_login: account_login.into(),
            token: token.into(),
            repository_selection,
            owner: owner.into(),
            repo: repo.into(),
            permissions: HashMap::new(),
        }
    }

    /// Sets the permissions map.
    #[must_use]
    pub fn with_permissions(mut self, permissions: HashMap<String, String>) -> Self {
        self.permissions = permissions;
        self
    }
}

impl fmt::Debug for AuthorizedInstallationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedInstallationContext")
            .field("installation_id", &self.installation_id)
            .field("account_login", &self.account_login)
            .field("token", &"***[REDACTED_TOKEN]***")
            .field("repository_selection", &self.repository_selection)
            .field("owner", &self.owner)
            .field("repo", &self.repo)
            .field("permissions", &self.permissions)
            .finish()
    }
}

impl fmt::Display for AuthorizedInstallationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthorizedInstallationContext(installation_id={}, account={}, repo={}/{}, selection={})",
            self.installation_id,
            self.account_login,
            self.owner,
            self.repo,
            self.repository_selection
        )
    }
}

/// Result of an installation authorization evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationOutcome {
    /// Installation is authorized and valid token is ready for extraction.
    Authorized(AuthorizedInstallationContext),

    /// The calling session has no registered GitHub App installations.
    NoInstallationForSession {
        /// The calling session identifier.
        session_id: Uuid,
    },

    /// The session has installations, but none matching the requested repository owner.
    AccountMismatch {
        /// The calling session identifier.
        session_id: Uuid,
        /// The requested repository owner.
        requested_owner: String,
        /// Account logins available to this session.
        available_accounts: Vec<String>,
    },

    /// Target repository is not granted under the installation's repository selection.
    RepositoryNotGranted {
        /// The installation identifier.
        installation_id: i64,
        /// Account login owning the installation.
        account_login: String,
        /// The requested repository slug (`owner/repo`).
        requested_repo: String,
        /// Current repository selection mode (`all` or `selected`).
        repository_selection: RepositorySelection,
    },

    /// The installation has been revoked.
    InstallationRevoked {
        /// The installation identifier.
        installation_id: i64,
        /// Account login owning the installation.
        account_login: String,
    },

    /// The installation is currently suspended on GitHub.
    InstallationSuspended {
        /// The installation identifier.
        installation_id: i64,
        /// Account login owning the installation.
        account_login: String,
    },

    /// The installation token's permissions violate the required policy.
    InsufficientPermissions {
        /// The installation identifier.
        installation_id: i64,
        /// Account login owning the installation.
        account_login: String,
        /// List of policy violation descriptions.
        violations: Vec<String>,
    },

    /// Token retrieval or renewal failed.
    TokenUnavailable {
        /// The installation identifier.
        installation_id: i64,
        /// Account login owning the installation.
        account_login: String,
        /// Failure reason description.
        reason: String,
    },
}

impl AuthorizationOutcome {
    /// Returns true if the evaluation resulted in an authorized state.
    #[must_use]
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized(_))
    }

    /// Unwraps the authorized context, panicking if not authorized.
    #[must_use]
    pub fn unwrap_authorized(self) -> AuthorizedInstallationContext {
        match self {
            Self::Authorized(ctx) => ctx,
            other => panic!("Called unwrap_authorized on non-authorized outcome: {other:?}"),
        }
    }
}

/// Evaluator coordinating installation access authorization for private repository extraction.
pub struct InstallationAuthorizer {
    token_store: Arc<dyn InstallationTokenStore>,
    github_client: Arc<GitHubApiClient>,
    token_refresher: Option<Arc<InstallationTokenRefresher>>,
    permission_policy: PermissionPolicy,
    safety_margin_secs: u64,
}

impl InstallationAuthorizer {
    /// Creates a new `InstallationAuthorizer` with default strict minimal permission policy (Rule 05)
    /// and a 300-second (5 minute) proactive renewal margin.
    pub fn new(
        token_store: Arc<dyn InstallationTokenStore>,
        github_client: Arc<GitHubApiClient>,
    ) -> Self {
        Self {
            token_store,
            github_client,
            token_refresher: None,
            permission_policy: PermissionPolicy::strict_minimal(),
            safety_margin_secs: 300,
        }
    }

    /// Injects an optional [`InstallationTokenRefresher`] for automated renewal of expiring tokens.
    #[must_use]
    pub fn with_refresher(mut self, refresher: Arc<InstallationTokenRefresher>) -> Self {
        self.token_refresher = Some(refresher);
        self
    }

    /// Configures the permission policy to evaluate against (defaults to Rule 05 strict minimal).
    #[must_use]
    pub fn with_permission_policy(mut self, policy: PermissionPolicy) -> Self {
        self.permission_policy = policy;
        self
    }

    /// Configures the proactive expiration safety margin in seconds.
    #[must_use]
    pub fn with_safety_margin_secs(mut self, margin_secs: u64) -> Self {
        self.safety_margin_secs = margin_secs;
        self
    }

    /// Evaluates installation authorization for a specific repository under a tenant session.
    pub async fn authorize(
        &self,
        session_id: &Uuid,
        owner: &str,
        repo: &str,
    ) -> Result<AuthorizationOutcome, GitHubExtractorError> {
        let trimmed_owner = owner.trim();
        let trimmed_repo = repo.trim();

        debug!(
            "Evaluating installation authorization for session {session_id} on {trimmed_owner}/{trimmed_repo}"
        );

        // 1. Resolve installation for this session matching the requested owner
        let installation_match = self
            .token_store
            .find_installation_for_account(session_id, trimmed_owner)
            .await?;

        let (installation_id, mut record) = match installation_match {
            Some(pair) => pair,
            None => {
                // Check if session has any installations to provide actionable mismatch diagnostics
                let session_installations = self
                    .token_store
                    .list_installations_for_session(session_id)
                    .await?;

                if session_installations.is_empty() {
                    info!("Session {session_id} has no registered GitHub App installations");
                    return Ok(AuthorizationOutcome::NoInstallationForSession {
                        session_id: *session_id,
                    });
                }

                let available_accounts: Vec<String> = session_installations
                    .into_iter()
                    .map(|(_, rec)| rec.account_login)
                    .collect();

                info!(
                    "Session {session_id} has installations for accounts {:?}, but none for '{trimmed_owner}'",
                    available_accounts
                );

                return Ok(AuthorizationOutcome::AccountMismatch {
                    session_id: *session_id,
                    requested_owner: trimmed_owner.to_string(),
                    available_accounts,
                });
            }
        };

        // 2. Validate installation state (revoked / suspended)
        if record.is_revoked {
            warn!(
                "Installation {installation_id} for {trimmed_owner} is revoked for session {session_id}"
            );
            return Ok(AuthorizationOutcome::InstallationRevoked {
                installation_id,
                account_login: record.account_login.clone(),
            });
        }

        if record.is_suspended {
            warn!("Installation {installation_id} for {trimmed_owner} is suspended on GitHub");
            return Ok(AuthorizationOutcome::InstallationSuspended {
                installation_id,
                account_login: record.account_login.clone(),
            });
        }

        // 3. Proactive token freshness evaluation and renewal
        let freshness = record.freshness(self.safety_margin_secs);
        if freshness.needs_refresh() {
            if let Some(refresher) = &self.token_refresher {
                debug!(
                    "Token for installation {installation_id} needs refresh ({freshness:?}); attempting proactive renewal"
                );
                match refresher
                    .get_or_refresh_token(session_id, installation_id, &record.account_login)
                    .await
                {
                    Ok(renewed_token) => {
                        debug!("Proactively renewed token for installation {installation_id}");
                        if let Ok(Some(updated)) = self
                            .token_store
                            .get_token(session_id, installation_id)
                            .await
                        {
                            record = updated;
                        } else {
                            record.token = renewed_token;
                        }
                    }
                    Err(e) => {
                        if freshness.is_expired() {
                            warn!(
                                "Token for installation {installation_id} is expired and renewal failed: {e}"
                            );
                            return Ok(AuthorizationOutcome::TokenUnavailable {
                                installation_id,
                                account_login: record.account_login.clone(),
                                reason: format!("Token renewal failed: {e}"),
                            });
                        }
                        warn!(
                            "Proactive renewal failed for expiring token (installation {installation_id}): {e}; proceeding with existing token"
                        );
                    }
                }
            } else if freshness.is_expired() {
                warn!(
                    "Token for installation {installation_id} is expired and no refresher configured"
                );
                return Ok(AuthorizationOutcome::TokenUnavailable {
                    installation_id,
                    account_login: record.account_login.clone(),
                    reason: "Token is expired and no refresher configured".to_string(),
                });
            }
        }

        // 4. Verify repository access grant (All vs Selected)
        let is_granted = match record.repository_selection {
            RepositorySelection::All => {
                // All repos under account are granted
                record.account_login.eq_ignore_ascii_case(trimmed_owner)
            }
            RepositorySelection::Selected => {
                if record.has_repository(trimmed_owner, trimmed_repo) {
                    true
                } else {
                    // Cached list did not match; verify live with GitHub API
                    debug!(
                        "Repository {trimmed_owner}/{trimmed_repo} not in cached list for installation {installation_id}; querying GitHub"
                    );
                    match self
                        .github_client
                        .check_repository_access(&record.token, trimmed_owner, trimmed_repo)
                        .await
                    {
                        Ok(Some(details)) => {
                            debug!(
                                "Verified repository access via GitHub API for {} (id: {})",
                                details.full_name, details.id
                            );
                            true
                        }
                        Ok(None) => false,
                        Err(e) => {
                            warn!(
                                "Failed to check repository access on GitHub for {trimmed_owner}/{trimmed_repo}: {e}"
                            );
                            false
                        }
                    }
                }
            }
        };

        if !is_granted {
            let requested_slug = format!("{trimmed_owner}/{trimmed_repo}");
            info!(
                "Repository '{requested_slug}' is not granted to installation {installation_id} (selection: {})",
                record.repository_selection
            );
            return Ok(AuthorizationOutcome::RepositoryNotGranted {
                installation_id,
                account_login: record.account_login.clone(),
                requested_repo: requested_slug,
                repository_selection: record.repository_selection,
            });
        }

        // 5. Rule 05 permission policy compliance verification
        let audit =
            PermissionAuditor::audit_permissions(&record.permissions, &self.permission_policy);

        if !audit.is_compliant() {
            let violations: Vec<String> = audit.violations.iter().map(|v| v.to_string()).collect();
            warn!(
                "Installation {installation_id} permissions violate policy: {:?}",
                violations
            );
            return Ok(AuthorizationOutcome::InsufficientPermissions {
                installation_id,
                account_login: record.account_login.clone(),
                violations,
            });
        }

        // 6. Access authorized
        info!(
            "Successfully authorized installation {installation_id} for {trimmed_owner}/{trimmed_repo} (session {session_id})"
        );

        Ok(AuthorizationOutcome::Authorized(
            AuthorizedInstallationContext {
                installation_id,
                account_login: record.account_login,
                token: record.token,
                repository_selection: record.repository_selection,
                owner: trimmed_owner.to_string(),
                repo: trimmed_repo.to_string(),
                permissions: record.permissions,
            },
        ))
    }
}
