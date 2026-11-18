//! GitHub App installation token revocation and lifecycle event synchronization.
//!
//! Provides upstream token invalidation via GitHub's REST API (`DELETE /installation/token`),
//! coordinated in-memory token cache eviction, and automated webhook lifecycle synchronization
//! for uninstallation (`installation.deleted`) and suspension (`installation.suspend`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::callback::{GitHubWebhookEvent, InstallationAction};
use crate::client::GitHubApiClient;
use crate::error::GitHubExtractorError;
use crate::token_refresh::InstallationTokenStore;

/// Result outcome of an installation token revocation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevocationOutcome {
    /// Active token revoked on GitHub API and purged locally.
    RevokedUpstreamAndLocally,
    /// Token revoked/zeroized locally (upstream token was expired, already invalid, or not present).
    RevokedLocally,
    /// Installation record was not found.
    NotFound,
    /// Installation was already in a revoked state.
    AlreadyRevoked,
}

impl RevocationOutcome {
    /// Returns `true` if any local or upstream revocation occurred.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        matches!(
            self,
            Self::RevokedUpstreamAndLocally | Self::RevokedLocally | Self::AlreadyRevoked
        )
    }

    /// Returns `true` if upstream GitHub API revocation was confirmed.
    #[must_use]
    pub const fn is_upstream_revoked(&self) -> bool {
        matches!(self, Self::RevokedUpstreamAndLocally)
    }
}

/// Detailed summary report of an installation revocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationSummary {
    /// Tenant session ID performing the revocation.
    pub session_id: Uuid,
    /// GitHub installation numeric ID.
    pub installation_id: i64,
    /// Concrete outcome of the revocation attempt.
    pub outcome: RevocationOutcome,
    /// UTC timestamp when revocation was executed.
    pub revoked_at: DateTime<Utc>,
    /// GitHub user or organization login, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_login: Option<String>,
    /// Whether GitHub API was invoked and confirmed invalidation.
    pub upstream_revoked: bool,
}

impl RevocationSummary {
    /// Constructs a new `RevocationSummary`.
    #[must_use]
    pub fn new(
        session_id: Uuid,
        installation_id: i64,
        outcome: RevocationOutcome,
        account_login: Option<String>,
    ) -> Self {
        let upstream_revoked = outcome.is_upstream_revoked();
        Self {
            session_id,
            installation_id,
            outcome,
            revoked_at: Utc::now(),
            account_login,
            upstream_revoked,
        }
    }
}

/// Action taken during webhook-driven installation lifecycle synchronization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "snake_case")]
pub enum WebhookLifecycleAction {
    /// GitHub App uninstalled: all cached and stored credentials purged.
    InstallationDeleted {
        /// GitHub installation ID.
        installation_id: i64,
        /// Account login that uninstalled the app.
        account_login: String,
        /// Number of in-memory cached tokens evicted.
        evicted_cached_tokens: usize,
    },
    /// GitHub App suspended: all cached tokens evicted to prevent usage.
    InstallationSuspended {
        /// GitHub installation ID.
        installation_id: i64,
        /// Account login that suspended the app.
        account_login: String,
        /// Number of in-memory cached tokens evicted.
        evicted_cached_tokens: usize,
    },
    /// GitHub App unsuspended: installation restored.
    InstallationUnsuspended {
        /// GitHub installation ID.
        installation_id: i64,
        /// Account login that unsuspended the app.
        account_login: String,
    },
    /// Non-lifecycle or non-revocation webhook event ignored.
    Ignored {
        /// Reason or event name ignored.
        reason: String,
    },
}

/// Coordinates installation token revocation and webhook event synchronization.
#[derive(Clone)]
pub struct TokenRevocationCoordinator {
    client: Arc<GitHubApiClient>,
    token_store: Arc<dyn InstallationTokenStore>,
}

impl TokenRevocationCoordinator {
    /// Constructs a new `TokenRevocationCoordinator`.
    #[must_use]
    pub fn new(client: Arc<GitHubApiClient>, token_store: Arc<dyn InstallationTokenStore>) -> Self {
        Self {
            client,
            token_store,
        }
    }

    /// Accessor for the underlying `GitHubApiClient`.
    #[must_use]
    pub fn client(&self) -> &Arc<GitHubApiClient> {
        &self.client
    }

    /// Accessor for the underlying `InstallationTokenStore`.
    #[must_use]
    pub fn token_store(&self) -> &Arc<dyn InstallationTokenStore> {
        &self.token_store
    }

    /// Revokes an installation token upstream via the GitHub REST API (`DELETE /installation/token`).
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if the HTTP request encounters network or server errors.
    pub async fn revoke_upstream(&self, plaintext_token: &str) -> Result<(), GitHubExtractorError> {
        self.client
            .revoke_installation_access_token(plaintext_token)
            .await
    }

    /// Revokes and deletes a cached token record from the in-memory store.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if storage operation fails.
    pub async fn revoke_cached_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<bool, GitHubExtractorError> {
        self.token_store
            .revoke_token(session_id, installation_id)
            .await
    }

    /// Revokes all cached token records for an installation ID across all tenant sessions.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if storage operation fails.
    pub async fn revoke_all_cached_for_installation(
        &self,
        installation_id: i64,
    ) -> Result<usize, GitHubExtractorError> {
        self.token_store
            .revoke_all_for_installation(installation_id)
            .await
    }

    /// Executes coordinated revocation for an installation:
    /// 1. If an active cached token exists, revokes it upstream on GitHub.
    /// 2. Evicts the token from the in-memory cache.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] on unrecoverable failure.
    pub async fn revoke_token_and_cache(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<RevocationOutcome, GitHubExtractorError> {
        let cached = self
            .token_store
            .get_token(session_id, installation_id)
            .await?;

        let mut upstream_revoked = false;
        if let Some(record) = cached {
            // Attempt upstream revocation on GitHub
            match self.revoke_upstream(record.token()).await {
                Ok(()) => {
                    upstream_revoked = true;
                    info!(
                        "Revoked token upstream on GitHub for installation {installation_id} (session {session_id})"
                    );
                }
                Err(err) => {
                    warn!(
                        "Upstream GitHub token revocation returned non-fatal error: {err}; proceeding with local eviction"
                    );
                }
            }
        }

        let evicted = self
            .token_store
            .revoke_token(session_id, installation_id)
            .await?;

        if upstream_revoked {
            Ok(RevocationOutcome::RevokedUpstreamAndLocally)
        } else if evicted {
            Ok(RevocationOutcome::RevokedLocally)
        } else {
            Ok(RevocationOutcome::NotFound)
        }
    }

    /// Synchronizes webhook lifecycle events with token caches.
    ///
    /// When an `installation` event with `action: deleted` or `action: suspend` is received,
    /// all corresponding cached tokens are immediately purged.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] on storage or event parsing errors.
    pub async fn handle_webhook_event(
        &self,
        event: &GitHubWebhookEvent,
    ) -> Result<WebhookLifecycleAction, GitHubExtractorError> {
        match event {
            GitHubWebhookEvent::Installation {
                action,
                installation,
            } => {
                let inst_id = installation.id as i64;
                let login = installation.account.login.clone();

                match action {
                    InstallationAction::Deleted => {
                        let count = self.revoke_all_cached_for_installation(inst_id).await?;
                        info!(
                            "Webhook processed: installation {inst_id} ({login}) deleted; evicted {count} cached token(s)"
                        );
                        Ok(WebhookLifecycleAction::InstallationDeleted {
                            installation_id: inst_id,
                            account_login: login,
                            evicted_cached_tokens: count,
                        })
                    }
                    InstallationAction::Suspend => {
                        let count = self.revoke_all_cached_for_installation(inst_id).await?;
                        warn!(
                            "Webhook processed: installation {inst_id} ({login}) suspended; evicted {count} cached token(s)"
                        );
                        Ok(WebhookLifecycleAction::InstallationSuspended {
                            installation_id: inst_id,
                            account_login: login,
                            evicted_cached_tokens: count,
                        })
                    }
                    InstallationAction::Unsuspend => {
                        info!("Webhook processed: installation {inst_id} ({login}) unsuspended");
                        Ok(WebhookLifecycleAction::InstallationUnsuspended {
                            installation_id: inst_id,
                            account_login: login,
                        })
                    }
                    other => {
                        debug!("Ignoring non-revocation installation action: {other:?}");
                        Ok(WebhookLifecycleAction::Ignored {
                            reason: format!("Non-revocation installation action: {other:?}"),
                        })
                    }
                }
            }
            other => {
                debug!("Ignoring non-installation webhook event: {other:?}");
                Ok(WebhookLifecycleAction::Ignored {
                    reason: "Non-installation webhook event".to_string(),
                })
            }
        }
    }
}
