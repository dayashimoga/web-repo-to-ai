//! # Authorization Failure Recovery Engine
//!
//! Provides intelligent error classification, root-cause diagnosis, automated token eviction,
//! proactive renewal retries, and step-by-step remediation guidance for private repository
//! authorization failures under Rule 05.
//!
//! ## Rule 05 Compliance & Security Invariants
//!
//! 1. **Zero Token Leakage:** All diagnosis strings, recovery plans, and remediation messages
//!    never expose raw bearer tokens or private keys.
//! 2. **Tenant Isolation:** Cache invalidation and token refresh are strictly bound to the
//!    calling session ID and installation ID.
//! 3. **Actionable Remediation:** For failures requiring human intervention (such as repository
//!    grant selection, app suspension, or organization SSO approval), direct, verified GitHub
//!    URLs are provided with precise instructions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::app_manifest::GitHubAppConfig;
use crate::authorization::AuthorizationOutcome;
use crate::error::{redact_secrets, GitHubExtractorError};
use crate::installation::RepositorySelection;
use crate::oauth::build_app_installation_url;
use crate::token_refresh::{InstallationTokenRefresher, InstallationTokenStore};

/// Categorization of reasons why private repository authorization or extraction failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationFailureKind {
    /// Bearer token is rejected upstream by GitHub (HTTP 401 Bad credentials).
    TokenInvalid,
    /// Token expired and could not be renewed automatically.
    TokenExpired,
    /// Requested private repository is not in the granted repository selection (HTTP 404).
    RepositoryNotGranted,
    /// Target repository owner does not match any connected GitHub App installation.
    AccountMismatch,
    /// Calling session has zero registered GitHub App installations.
    NoInstallationForSession,
    /// GitHub App installation is suspended by the organization or user.
    InstallationSuspended,
    /// GitHub App installation has been revoked, uninstalled, or deleted.
    InstallationRevoked,
    /// Token permissions violate Rule 05 least-privilege policy.
    InsufficientPermissions,
    /// GitHub API rate limit was reached (HTTP 403 / 429).
    RateLimited,
    /// Upstream GitHub API internal server error or network connectivity failure.
    UpstreamApiError,
}

impl AuthorizationFailureKind {
    /// Returns the snake_case identifier for this failure kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TokenInvalid => "token_invalid",
            Self::TokenExpired => "token_expired",
            Self::RepositoryNotGranted => "repository_not_granted",
            Self::AccountMismatch => "account_mismatch",
            Self::NoInstallationForSession => "no_installation_for_session",
            Self::InstallationSuspended => "installation_suspended",
            Self::InstallationRevoked => "installation_revoked",
            Self::InsufficientPermissions => "insufficient_permissions",
            Self::RateLimited => "rate_limited",
            Self::UpstreamApiError => "upstream_api_error",
        }
    }
}

impl fmt::Display for AuthorizationFailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Prescribed remediation action to resolve an authorization failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RecoveryAction {
    /// System can automatically evict the invalid token and retry with a freshly issued token.
    AutomaticRetryWithFreshToken,
    /// User must install the GitHub App on their account or organization.
    PromptAppInstallation {
        /// 1-click URL to install the GitHub App.
        installation_url: String,
        /// Suggested target account handle, if known.
        target_account: Option<String>,
    },
    /// User must update repository access in their GitHub App installation settings.
    PromptRepositorySelectionUpdate {
        /// Direct GitHub App installation settings URL.
        settings_url: String,
        /// Numeric GitHub App installation ID.
        installation_id: i64,
        /// Missing repository slug (`owner/repo`).
        repository: String,
    },
    /// User or organization admin must un-suspend the GitHub App installation.
    PromptUnsuspendInstallation {
        /// Direct GitHub App installation settings URL.
        settings_url: String,
        /// Numeric GitHub App installation ID.
        installation_id: i64,
        /// Account login handle.
        account_login: String,
    },
    /// User must re-authenticate their session grant.
    PromptReauthenticateUser {
        /// OAuth authorization URL.
        authorize_url: String,
    },
    /// Caller must pause until rate limit reset window expires.
    WaitRateLimitReset {
        /// Duration in seconds until rate limit resets.
        retry_after_secs: u64,
        /// Timestamp when rate limit resets.
        reset_at: DateTime<Utc>,
    },
    /// Organization administrator approval or SSO authorization is required.
    ContactAdministrator {
        /// Account login handle.
        account_login: String,
        /// Specific explanation of why administrator action is needed.
        details: String,
    },
    /// Failure cannot be recovered automatically.
    Unrecoverable {
        /// Detail explanation of why recovery is not possible.
        reason: String,
    },
}

/// Comprehensive diagnostic report and remediation plan for an authorization failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationRecoveryPlan {
    /// Classified category of failure.
    pub failure_kind: AuthorizationFailureKind,
    /// Whether the failure can be resolved automatically without user intervention.
    pub is_auto_recoverable: bool,
    /// Prescribed recovery action.
    pub prescribed_action: RecoveryAction,
    /// Human-readable diagnosis explaining why authorization failed.
    pub diagnosis: String,
    /// Concrete step-by-step remediation instructions for the user or administrator.
    pub remediation_guidance: Vec<String>,
    /// Target repository slug (`owner/repo`), if applicable.
    pub target_slug: Option<String>,
    /// Associated installation ID, if known.
    pub installation_id: Option<i64>,
    /// Associated account login handle, if known.
    pub account_login: Option<String>,
}

impl AuthorizationRecoveryPlan {
    /// Returns true if this failure can be resolved automatically.
    #[must_use]
    pub const fn is_auto_recoverable(&self) -> bool {
        self.is_auto_recoverable
    }

    /// Returns a short machine-readable action code string.
    #[must_use]
    pub const fn action_code(&self) -> &'static str {
        match &self.prescribed_action {
            RecoveryAction::AutomaticRetryWithFreshToken => "auto_retry",
            RecoveryAction::PromptAppInstallation { .. } => "prompt_app_installation",
            RecoveryAction::PromptRepositorySelectionUpdate { .. } => {
                "prompt_repository_selection_update"
            }
            RecoveryAction::PromptUnsuspendInstallation { .. } => "prompt_unsuspend_installation",
            RecoveryAction::PromptReauthenticateUser { .. } => "prompt_reauthenticate_user",
            RecoveryAction::WaitRateLimitReset { .. } => "wait_rate_limit_reset",
            RecoveryAction::ContactAdministrator { .. } => "contact_administrator",
            RecoveryAction::Unrecoverable { .. } => "unrecoverable",
        }
    }

    /// Formats a Markdown summary card suitable for user-facing UI or error messages.
    #[must_use]
    pub fn summary_card(&self) -> String {
        let mut card = String::with_capacity(512);
        card.push_str("### GitHub App Authorization Diagnostic\n\n");
        card.push_str(&format!("**Status:** `{}`\n", self.failure_kind.as_str()));

        if let Some(target) = &self.target_slug {
            card.push_str(&format!("**Target Repository:** `{target}`\n"));
        }
        if let Some(inst_id) = self.installation_id {
            card.push_str(&format!("**Installation ID:** `{inst_id}`\n"));
        }
        if let Some(acct) = &self.account_login {
            card.push_str(&format!("**Account Login:** `{acct}`\n"));
        }

        card.push_str(&format!("\n**Diagnosis:** {}\n\n", self.diagnosis));
        card.push_str("**Remediation Steps:**\n");
        for (idx, step) in self.remediation_guidance.iter().enumerate() {
            card.push_str(&format!("{}. {}\n", idx + 1, step));
        }

        match &self.prescribed_action {
            RecoveryAction::PromptAppInstallation {
                installation_url, ..
            } => {
                card.push_str(&format!(
                    "\n[Install GitHub App on your account]({installation_url})\n"
                ));
            }
            RecoveryAction::PromptRepositorySelectionUpdate { settings_url, .. } => {
                card.push_str(&format!(
                    "\n[Update Repository Access Settings]({settings_url})\n"
                ));
            }
            RecoveryAction::PromptUnsuspendInstallation { settings_url, .. } => {
                card.push_str(&format!(
                    "\n[Manage Installation on GitHub]({settings_url})\n"
                ));
            }
            RecoveryAction::PromptReauthenticateUser { authorize_url } => {
                card.push_str(&format!(
                    "\n[Re-authenticate with GitHub]({authorize_url})\n"
                ));
            }
            RecoveryAction::WaitRateLimitReset {
                retry_after_secs,
                reset_at,
            } => {
                card.push_str(&format!(
                    "\n*Rate limit resets in {retry_after_secs}s at {reset_at}.*\n"
                ));
            }
            _ => {}
        }

        card
    }
}

/// Orchestrator for analyzing authorization failures, evicting invalid tokens,
/// and executing recovery workflows.
pub struct AuthorizationRecoveryCoordinator {
    token_store: Option<Arc<dyn InstallationTokenStore>>,
    token_refresher: Option<Arc<InstallationTokenRefresher>>,
    app_config: Option<Arc<GitHubAppConfig>>,
}

impl Default for AuthorizationRecoveryCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthorizationRecoveryCoordinator {
    /// Creates a new `AuthorizationRecoveryCoordinator`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            token_store: None,
            token_refresher: None,
            app_config: None,
        }
    }

    /// Sets the installation token store.
    #[must_use]
    pub fn with_token_store(mut self, store: Arc<dyn InstallationTokenStore>) -> Self {
        self.token_store = Some(store);
        self
    }

    /// Sets the installation token refresher.
    #[must_use]
    pub fn with_token_refresher(mut self, refresher: Arc<InstallationTokenRefresher>) -> Self {
        self.token_refresher = Some(refresher);
        self
    }

    /// Sets the GitHub App configuration.
    #[must_use]
    pub fn with_app_config(mut self, config: Arc<GitHubAppConfig>) -> Self {
        self.app_config = Some(config);
        self
    }

    /// Diagnoses an [`AuthorizationOutcome`] evaluation and produces an actionable [`AuthorizationRecoveryPlan`].
    #[must_use]
    pub fn diagnose_outcome(
        &self,
        outcome: &AuthorizationOutcome,
        owner: &str,
        repo: &str,
    ) -> AuthorizationRecoveryPlan {
        let target_slug = format!("{owner}/{repo}");
        let base_html = self
            .app_config
            .as_ref()
            .map(|c| c.html_url.as_str())
            .unwrap_or("https://github.com");

        match outcome {
            AuthorizationOutcome::Authorized(ctx) => AuthorizationRecoveryPlan {
                failure_kind: AuthorizationFailureKind::TokenInvalid,
                is_auto_recoverable: true,
                prescribed_action: RecoveryAction::AutomaticRetryWithFreshToken,
                diagnosis: format!(
                    "Authorization for '{target_slug}' succeeded (installation_id={})",
                    ctx.installation_id
                ),
                remediation_guidance: vec!["No remediation required; context is valid.".to_string()],
                target_slug: Some(target_slug),
                installation_id: Some(ctx.installation_id),
                account_login: Some(ctx.account_login.clone()),
            },

            AuthorizationOutcome::NoInstallationForSession { session_id } => {
                let install_url = self.build_installation_url(Some(owner));
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::NoInstallationForSession,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptAppInstallation {
                        installation_url: install_url,
                        target_account: Some(owner.to_string()),
                    },
                    diagnosis: format!(
                        "Session {session_id} has no registered GitHub App installations."
                    ),
                    remediation_guidance: vec![
                        format!("Install the GitHub App on your personal account or '{owner}' organization."),
                        "Grant read-only access to repository contents and metadata.".to_string(),
                        "Retry repository extraction after installation completes.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id: None,
                    account_login: None,
                }
            }

            AuthorizationOutcome::AccountMismatch {
                requested_owner,
                available_accounts,
                ..
            } => {
                let install_url = self.build_installation_url(Some(requested_owner));
                let acct_list = if available_accounts.is_empty() {
                    "none".to_string()
                } else {
                    available_accounts.join(", ")
                };
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::AccountMismatch,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptAppInstallation {
                        installation_url: install_url,
                        target_account: Some(requested_owner.clone()),
                    },
                    diagnosis: format!(
                        "Requested repository belongs to account '{requested_owner}', but current session only has installations for: {acct_list}."
                    ),
                    remediation_guidance: vec![
                        format!("Install the GitHub App on organization/user '{requested_owner}'."),
                        "Ensure the repository is selected during the installation flow.".to_string(),
                        "Refresh session authorization after installation.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id: None,
                    account_login: Some(requested_owner.clone()),
                }
            }

            AuthorizationOutcome::RepositoryNotGranted {
                installation_id,
                account_login,
                repository_selection,
                ..
            } => {
                let settings_url = format!("{base_html}/settings/installations/{installation_id}");
                let guidance = match repository_selection {
                    RepositorySelection::Selected => vec![
                        format!("Open the installation settings for '{account_login}' on GitHub."),
                        format!("Under 'Repository access', select 'Only select repositories' and add '{repo}'."),
                        "Save changes and retry the extraction.".to_string(),
                    ],
                    RepositorySelection::All => vec![
                        format!("Verify that repository '{target_slug}' exists and is owned by '{account_login}'."),
                        "Check that your GitHub account has permission to view this repository.".to_string(),
                    ],
                };
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::RepositoryNotGranted,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptRepositorySelectionUpdate {
                        settings_url,
                        installation_id: *installation_id,
                        repository: target_slug.clone(),
                    },
                    diagnosis: format!(
                        "Repository '{target_slug}' is not granted to installation {installation_id} (selection mode: {repository_selection})."
                    ),
                    remediation_guidance: guidance,
                    target_slug: Some(target_slug),
                    installation_id: Some(*installation_id),
                    account_login: Some(account_login.clone()),
                }
            }

            AuthorizationOutcome::InstallationRevoked {
                installation_id,
                account_login,
            } => {
                let install_url = self.build_installation_url(Some(account_login));
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::InstallationRevoked,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptAppInstallation {
                        installation_url: install_url,
                        target_account: Some(account_login.clone()),
                    },
                    diagnosis: format!(
                        "GitHub App installation {installation_id} for '{account_login}' has been revoked or uninstalled."
                    ),
                    remediation_guidance: vec![
                        format!("Reinstall the GitHub App on '{account_login}'."),
                        "Grant access to the required repositories.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id: Some(*installation_id),
                    account_login: Some(account_login.clone()),
                }
            }

            AuthorizationOutcome::InstallationSuspended {
                installation_id,
                account_login,
            } => {
                let settings_url = format!("{base_html}/settings/installations/{installation_id}");
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::InstallationSuspended,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptUnsuspendInstallation {
                        settings_url,
                        installation_id: *installation_id,
                        account_login: account_login.clone(),
                    },
                    diagnosis: format!(
                        "GitHub App installation {installation_id} on '{account_login}' is currently suspended."
                    ),
                    remediation_guidance: vec![
                        format!("Navigate to installation settings on GitHub ({base_html}/settings/installations/{installation_id})."),
                        "Click 'Unsuspend' to restore API access.".to_string(),
                        "Retry extraction once access is resumed.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id: Some(*installation_id),
                    account_login: Some(account_login.clone()),
                }
            }

            AuthorizationOutcome::InsufficientPermissions {
                installation_id,
                account_login,
                violations,
            } => AuthorizationRecoveryPlan {
                failure_kind: AuthorizationFailureKind::InsufficientPermissions,
                is_auto_recoverable: false,
                prescribed_action: RecoveryAction::ContactAdministrator {
                    account_login: account_login.clone(),
                    details: violations.join("; "),
                },
                diagnosis: format!(
                    "Installation {installation_id} on '{account_login}' violates Rule 05 permission policy: {}.",
                    violations.join(", ")
                ),
                remediation_guidance: vec![
                    "Verify that the GitHub App requested only 'Contents: Read' and 'Metadata: Read'.".to_string(),
                    "Ensure no administrative or write scopes were added.".to_string(),
                ],
                target_slug: Some(target_slug),
                installation_id: Some(*installation_id),
                account_login: Some(account_login.clone()),
            },

            AuthorizationOutcome::TokenUnavailable {
                installation_id,
                account_login,
                reason,
            } => {
                let sanitized_reason = redact_secrets(reason);
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::TokenExpired,
                    is_auto_recoverable: self.token_refresher.is_some(),
                    prescribed_action: if self.token_refresher.is_some() {
                        RecoveryAction::AutomaticRetryWithFreshToken
                    } else {
                        RecoveryAction::Unrecoverable {
                            reason: sanitized_reason.clone(),
                        }
                    },
                    diagnosis: format!(
                        "Token retrieval or renewal failed for installation {installation_id} on '{account_login}': {sanitized_reason}"
                    ),
                    remediation_guidance: vec![
                        "Check that the GitHub App private key and credentials are valid.".to_string(),
                        "Ensure the server has outbound internet access to GitHub's API.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id: Some(*installation_id),
                    account_login: Some(account_login.clone()),
                }
            }
        }
    }

    /// Diagnoses a runtime [`GitHubExtractorError`] returned during private repository extraction
    /// and maps it to an actionable [`AuthorizationRecoveryPlan`].
    #[must_use]
    pub fn diagnose_extractor_error(
        &self,
        error: &GitHubExtractorError,
        owner: &str,
        repo: &str,
        installation_id: Option<i64>,
        account_login: Option<&str>,
    ) -> AuthorizationRecoveryPlan {
        let target_slug = format!("{owner}/{repo}");
        let base_html = self
            .app_config
            .as_ref()
            .map(|c| c.html_url.as_str())
            .unwrap_or("https://github.com");

        match error {
            GitHubExtractorError::BadCredentials { message } => {
                let sanitized = redact_secrets(message);
                let can_retry = self.token_refresher.is_some() && self.token_store.is_some();
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::TokenInvalid,
                    is_auto_recoverable: can_retry,
                    prescribed_action: if can_retry {
                        RecoveryAction::AutomaticRetryWithFreshToken
                    } else {
                        RecoveryAction::PromptAppInstallation {
                            installation_url: self.build_installation_url(Some(owner)),
                            target_account: Some(owner.to_string()),
                        }
                    },
                    diagnosis: format!(
                        "GitHub rejected the installation token (HTTP 401 Bad credentials): {sanitized}."
                    ),
                    remediation_guidance: vec![
                        "The cached installation token is expired or revoked upstream.".to_string(),
                        "Purge stale credentials and issue a fresh ephemeral token.".to_string(),
                        "If error persists, re-authorize the GitHub App.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id,
                    account_login: account_login.map(ToString::to_string),
                }
            }

            GitHubExtractorError::ResourceNotFound { url, message } => {
                let sanitized_url = redact_secrets(url);
                let sanitized_msg = redact_secrets(message);
                let settings_url = installation_id.map_or_else(
                    || format!("{base_html}/settings/installations"),
                    |id| format!("{base_html}/settings/installations/{id}"),
                );
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::RepositoryNotGranted,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::PromptRepositorySelectionUpdate {
                        settings_url,
                        installation_id: installation_id.unwrap_or(0),
                        repository: target_slug.clone(),
                    },
                    diagnosis: format!(
                        "GitHub returned HTTP 404 Not Found at '{sanitized_url}': {sanitized_msg}. For private repositories, 404 indicates the repository does not exist or access has not been granted."
                    ),
                    remediation_guidance: vec![
                        format!("Check that the repository '{target_slug}' exists and is spelled correctly."),
                        format!("If '{target_slug}' is private, verify that the GitHub App installation has access to it."),
                        "In your GitHub App settings, ensure this repository is included under 'Selected repositories'.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id,
                    account_login: account_login.map(ToString::to_string),
                }
            }

            GitHubExtractorError::RateLimitExceeded {
                limit,
                reset_at,
                message,
            } => {
                let sanitized = redact_secrets(message);
                let now = Utc::now();
                let retry_after_secs = (*reset_at - now).num_seconds().max(0) as u64;
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::RateLimited,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::WaitRateLimitReset {
                        retry_after_secs,
                        reset_at: *reset_at,
                    },
                    diagnosis: format!(
                        "GitHub REST API rate limit reached (limit={limit}, reset_at={reset_at}): {sanitized}."
                    ),
                    remediation_guidance: vec![
                        format!("Wait {retry_after_secs} seconds until the rate limit window resets at {reset_at}."),
                        "Retry extraction after the reset window.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id,
                    account_login: account_login.map(ToString::to_string),
                }
            }

            GitHubExtractorError::Forbidden { message } => {
                let sanitized = redact_secrets(message);
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::InsufficientPermissions,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::ContactAdministrator {
                        account_login: account_login.unwrap_or(owner).to_string(),
                        details: sanitized.clone(),
                    },
                    diagnosis: format!("GitHub request forbidden (HTTP 403): {sanitized}."),
                    remediation_guidance: vec![
                        "Check if the repository has organization IP allowlist or SAML SSO restrictions.".to_string(),
                        "Ensure the GitHub App permissions include 'Contents: Read' and 'Metadata: Read'.".to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id,
                    account_login: account_login.map(ToString::to_string),
                }
            }

            other => {
                let sanitized = redact_secrets(&other.to_string());
                AuthorizationRecoveryPlan {
                    failure_kind: AuthorizationFailureKind::UpstreamApiError,
                    is_auto_recoverable: false,
                    prescribed_action: RecoveryAction::Unrecoverable {
                        reason: sanitized.clone(),
                    },
                    diagnosis: format!("Extraction failed with upstream error: {sanitized}."),
                    remediation_guidance: vec![
                        "Verify network connectivity to GitHub API.".to_string(),
                        "Check GitHub Status (https://www.githubstatus.com) for service incidents."
                            .to_string(),
                    ],
                    target_slug: Some(target_slug),
                    installation_id,
                    account_login: account_login.map(ToString::to_string),
                }
            }
        }
    }

    /// Evicts an invalid token from the store and proactively issues a fresh token.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if store eviction or token refresh fails.
    pub async fn attempt_eviction_and_refresh(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
    ) -> Result<String, GitHubExtractorError> {
        info!(
            "Evicting stale token for session {session_id} / installation {installation_id} and renewing"
        );

        if let Some(store) = &self.token_store {
            let _ = store.revoke_token(session_id, installation_id).await;
        }

        let Some(refresher) = &self.token_refresher else {
            return Err(GitHubExtractorError::BadCredentials {
                message: "No token refresher configured to renew invalid installation token"
                    .to_string(),
            });
        };

        refresher
            .get_or_refresh_token(session_id, installation_id, account_login)
            .await
    }

    /// Executes an async operation with automatic token eviction and one-time retry
    /// if the first attempt returns an HTTP 401 Unauthorized / Bad credentials error.
    pub async fn execute_with_retry_on_401<F, Fut, T>(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
        mut operation: F,
    ) -> Result<T, GitHubExtractorError>
    where
        F: FnMut(Option<&str>) -> Fut,
        Fut: Future<Output = Result<T, GitHubExtractorError>>,
    {
        // First attempt with currently active/cached token
        match operation(None).await {
            Ok(val) => Ok(val),
            Err(e) if is_401_error(&e) => {
                warn!(
                    "Operation failed with 401 Unauthorized; attempting eviction and proactive token renewal"
                );
                let fresh_token = self
                    .attempt_eviction_and_refresh(session_id, installation_id, account_login)
                    .await?;

                // Second attempt with the renewed token
                debug!("Retrying operation with freshly renewed token");
                operation(Some(&fresh_token)).await
            }
            Err(e) => Err(e),
        }
    }

    fn build_installation_url(&self, target_account: Option<&str>) -> String {
        let base_html = self
            .app_config
            .as_ref()
            .map(|c| c.html_url.as_str())
            .unwrap_or("https://github.com");

        if let Some(config) = &self.app_config {
            let identifier = config.app_slug.as_deref().or(config.app_name.as_deref());
            if let Some(id) = identifier {
                if let Ok(mut url) = build_app_installation_url(id, None, Some(base_html)) {
                    if let Some(acct) = target_account {
                        url.push_str(&format!("?target_account={acct}"));
                    }
                    return url;
                }
            }
        }

        target_account.map_or_else(
            || format!("{base_html}/apps"),
            |acct| format!("{base_html}/apps?target_account={acct}"),
        )
    }
}

/// Helper detecting whether an error represents an HTTP 401 Unauthorized.
fn is_401_error(err: &GitHubExtractorError) -> bool {
    matches!(
        err,
        GitHubExtractorError::BadCredentials { .. }
            | GitHubExtractorError::HttpError { status: 401, .. }
    )
}
