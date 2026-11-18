//! # GitHub App Installation Token Refresh & Expiration Lifecycle
//!
//! Provides proactive token expiration evaluation, automated token renewal via GitHub App access
//! token API, single-flight concurrent refresh deduplication, and memory-safe caching under Rule 05.
//!
//! ## Rule 05 & Architectural Invariants
//!
//! 1. **Secret Masking:** Plaintext bearer tokens (`ghs_*`) are strictly masked in [`fmt::Debug`]
//!    and [`fmt::Display`] implementations to prevent credential leakage into logs and traces.
//! 2. **Proactive Refresh Window:** Tokens within the safety margin (default 5 minutes / 300 seconds)
//!    are proactively refreshed prior to issuing extraction queries to prevent mid-flight 401s.
//! 3. **Thundering-Herd Single Flight:** Concurrent requests for the same installation are
//!    deduplicated via in-flight locks and double-checked locking, preventing stampedes against
//!    GitHub's `POST /app/installations/{id}/access_tokens` endpoint.
//! 4. **Strict Least Privilege:** Renewed tokens are strictly validated against the minimal read-only
//!    policy (`contents: read`, `metadata: read`) before being stored or returned to callers.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::app_manifest::GitHubAppConfig;
use crate::client::GitHubApiClient;
use crate::error::GitHubExtractorError;
use crate::installation::{GitHubInstallationToken, InstallationTokenRequest, RepositorySelection};
use crate::permissions::PermissionPolicy;

/// Freshness classification for an installation token relative to its expiration deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenFreshness {
    /// Token is valid and well beyond the proactive refresh window.
    Fresh {
        /// Seconds remaining before expiration.
        seconds_remaining: u64,
    },
    /// Token is still valid but within the proactive refresh window; proactive renewal is needed.
    ExpiringSoon {
        /// Seconds remaining before expiration.
        seconds_remaining: u64,
    },
    /// Token has expired or is revoked.
    Expired {
        /// Seconds elapsed since expiration.
        expired_ago_secs: u64,
    },
}

impl TokenFreshness {
    /// Returns `true` if the token is completely fresh and does not require renewal.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    /// Returns `true` if the token has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        matches!(self, Self::Expired { .. })
    }

    /// Returns `true` if the token needs renewal (either expiring soon or expired).
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        !self.is_fresh()
    }

    /// Evaluates token freshness given the expiration timestamp, reference time, and safety margin.
    #[must_use]
    pub fn evaluate(
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
        safety_margin_secs: u64,
    ) -> Self {
        if now >= expires_at {
            let expired_ago_secs = (now - expires_at).num_seconds().max(0) as u64;
            Self::Expired { expired_ago_secs }
        } else {
            let seconds_remaining = (expires_at - now).num_seconds().max(0) as u64;
            if seconds_remaining <= safety_margin_secs {
                Self::ExpiringSoon { seconds_remaining }
            } else {
                Self::Fresh { seconds_remaining }
            }
        }
    }
}

/// Configurable policy for token lifecycle management, safety margins, and renewal retries.
#[derive(Debug, Clone)]
pub struct TokenLifecyclePolicy {
    /// Proactive refresh buffer in seconds (default: 300s / 5 minutes).
    pub safety_margin_secs: u64,
    /// Maximum retry attempts when calling GitHub token renewal API.
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds for exponential retry.
    pub retry_backoff_ms: u64,
    /// Permission policy enforced on renewed tokens under Rule 05.
    pub permission_policy: PermissionPolicy,
}

impl Default for TokenLifecyclePolicy {
    fn default() -> Self {
        Self {
            safety_margin_secs: 300, // 5 minutes
            max_retries: 3,
            retry_backoff_ms: 500,
            permission_policy: PermissionPolicy::strict_minimal(),
        }
    }
}

impl TokenLifecyclePolicy {
    /// Creates a default `TokenLifecyclePolicy`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the proactive refresh safety margin in seconds.
    #[must_use]
    pub fn with_safety_margin_secs(mut self, secs: u64) -> Self {
        self.safety_margin_secs = secs;
        self
    }

    /// Sets the maximum retry attempts for token renewal calls.
    #[must_use]
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Sets the initial exponential backoff delay in milliseconds.
    #[must_use]
    pub fn with_retry_backoff_ms(mut self, ms: u64) -> Self {
        self.retry_backoff_ms = ms;
        self
    }

    /// Sets the permission policy required for renewed tokens.
    #[must_use]
    pub fn with_permission_policy(mut self, policy: PermissionPolicy) -> Self {
        self.permission_policy = policy;
        self
    }
}

/// Cached token record stored in an [`InstallationTokenStore`].
///
/// Under Rule 05, the `token` string is strictly masked in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTokenRecord {
    /// Sensitive bearer token.
    pub token: String,
    /// Expiration deadline.
    pub expires_at: DateTime<Utc>,
    /// GitHub login handle of the owning account.
    pub account_login: String,
    /// Scope of repositories granted to the installation (All or Selected).
    #[serde(default)]
    pub repository_selection: RepositorySelection,
    /// Optional explicit list of repositories authorized for this token (e.g. `["org/repo"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<String>>,
    /// Granted permission scopes map (e.g. `{"contents": "read", "metadata": "read"}`).
    #[serde(default)]
    pub permissions: HashMap<String, String>,
    /// Whether this installation has been revoked.
    #[serde(default)]
    pub is_revoked: bool,
    /// Whether this installation is currently suspended.
    #[serde(default)]
    pub is_suspended: bool,
}

impl StoredTokenRecord {
    /// Constructs a new `StoredTokenRecord`.
    #[must_use]
    pub fn new(
        token: impl Into<String>,
        expires_at: DateTime<Utc>,
        account_login: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            expires_at,
            account_login: account_login.into(),
            repository_selection: RepositorySelection::Selected,
            repositories: None,
            permissions: HashMap::new(),
            is_revoked: false,
            is_suspended: false,
        }
    }

    /// Sets the repository selection mode.
    #[must_use]
    pub fn with_repository_selection(mut self, selection: RepositorySelection) -> Self {
        self.repository_selection = selection;
        self
    }

    /// Sets the list of authorized repository full names.
    #[must_use]
    pub fn with_repositories(mut self, repos: impl Into<Option<Vec<String>>>) -> Self {
        self.repositories = repos.into();
        self
    }

    /// Sets the permission map for this token record.
    #[must_use]
    pub fn with_permissions(mut self, permissions: HashMap<String, String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Sets the revocation state.
    #[must_use]
    pub fn with_revoked(mut self, revoked: bool) -> Self {
        self.is_revoked = revoked;
        self
    }

    /// Sets the suspension state.
    #[must_use]
    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.is_suspended = suspended;
        self
    }

    /// Returns a reference to the plaintext token string.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Evaluates the freshness of this stored token.
    #[must_use]
    pub fn freshness(&self, safety_margin_secs: u64) -> TokenFreshness {
        TokenFreshness::evaluate(self.expires_at, Utc::now(), safety_margin_secs)
    }

    /// Checks if a specific repository (e.g. "acme/repo" or "repo") is granted by this token record.
    #[must_use]
    pub fn has_repository(&self, owner: &str, repo: &str) -> bool {
        if self.repository_selection == RepositorySelection::All {
            return self.account_login.eq_ignore_ascii_case(owner);
        }
        if let Some(repos) = &self.repositories {
            let full_target = format!("{owner}/{repo}").to_ascii_lowercase();
            let repo_only = repo.to_ascii_lowercase();
            repos.iter().any(|r| {
                let lower = r.to_ascii_lowercase();
                lower == full_target || lower == repo_only
            })
        } else {
            false
        }
    }
}

impl fmt::Debug for StoredTokenRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredTokenRecord")
            .field("token", &"***[REDACTED_TOKEN]***")
            .field("expires_at", &self.expires_at)
            .field("account_login", &self.account_login)
            .field("repository_selection", &self.repository_selection)
            .field(
                "repositories_count",
                &self.repositories.as_ref().map(|r| r.len()),
            )
            .field("is_revoked", &self.is_revoked)
            .field("is_suspended", &self.is_suspended)
            .finish()
    }
}

impl fmt::Display for StoredTokenRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoredTokenRecord(account_login={}, expires_at={}, selection={}, revoked={})",
            self.account_login, self.expires_at, self.repository_selection, self.is_revoked
        )
    }
}

/// Abstract persistent or in-memory storage contract for installation access tokens.
#[async_trait]
pub trait InstallationTokenStore: Send + Sync {
    /// Retrieves a cached token record if present.
    async fn get_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<Option<StoredTokenRecord>, GitHubExtractorError>;

    /// Stores or replaces an installation token record.
    async fn save_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
        token: &GitHubInstallationToken,
    ) -> Result<(), GitHubExtractorError>;

    /// Directly saves a pre-built stored token record.
    async fn save_token_record(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        record: StoredTokenRecord,
    ) -> Result<(), GitHubExtractorError>;

    /// Explicitly revokes and deletes a cached token record.
    async fn revoke_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<bool, GitHubExtractorError>;

    /// Explicitly revokes and deletes all cached token records for an installation across all sessions.
    async fn revoke_all_for_installation(
        &self,
        installation_id: i64,
    ) -> Result<usize, GitHubExtractorError>;

    /// Lists all installation token records associated with the specified session.
    async fn list_installations_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<(i64, StoredTokenRecord)>, GitHubExtractorError>;

    /// Finds an active installation for a specific account login within a session.
    async fn find_installation_for_account(
        &self,
        session_id: &Uuid,
        account_login: &str,
    ) -> Result<Option<(i64, StoredTokenRecord)>, GitHubExtractorError>;
}

/// Thread-safe in-memory implementation of [`InstallationTokenStore`].
#[derive(Clone, Default)]
pub struct InMemoryInstallationTokenStore {
    records: Arc<RwLock<HashMap<(Uuid, i64), StoredTokenRecord>>>,
}

impl InMemoryInstallationTokenStore {
    /// Creates a new empty `InMemoryInstallationTokenStore`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl InstallationTokenStore for InMemoryInstallationTokenStore {
    async fn get_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<Option<StoredTokenRecord>, GitHubExtractorError> {
        let lock = self.records.read().await;
        Ok(lock.get(&(*session_id, installation_id)).cloned())
    }

    async fn save_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
        token: &GitHubInstallationToken,
    ) -> Result<(), GitHubExtractorError> {
        let mut lock = self.records.write().await;
        let mut record = StoredTokenRecord::new(&token.token, token.expires_at, account_login)
            .with_repository_selection(token.repository_selection)
            .with_permissions(token.permissions.clone());

        if let Some(repos) = &token.repositories {
            record =
                record.with_repositories(Some(repos.iter().map(|r| r.full_name.clone()).collect()));
        }

        lock.insert((*session_id, installation_id), record);
        Ok(())
    }

    async fn save_token_record(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        record: StoredTokenRecord,
    ) -> Result<(), GitHubExtractorError> {
        let mut lock = self.records.write().await;
        lock.insert((*session_id, installation_id), record);
        Ok(())
    }

    async fn revoke_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<bool, GitHubExtractorError> {
        let mut lock = self.records.write().await;
        Ok(lock.remove(&(*session_id, installation_id)).is_some())
    }

    async fn revoke_all_for_installation(
        &self,
        installation_id: i64,
    ) -> Result<usize, GitHubExtractorError> {
        let mut lock = self.records.write().await;
        let initial_len = lock.len();
        lock.retain(|(_session_id, inst_id), _| *inst_id != installation_id);
        Ok(initial_len - lock.len())
    }

    async fn list_installations_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<(i64, StoredTokenRecord)>, GitHubExtractorError> {
        let guard = self.records.read().await;
        let mut list = Vec::new();
        for ((sess, inst_id), record) in guard.iter() {
            if sess == session_id {
                list.push((*inst_id, record.clone()));
            }
        }
        list.sort_by_key(|(id, _)| *id);
        Ok(list)
    }

    async fn find_installation_for_account(
        &self,
        session_id: &Uuid,
        account_login: &str,
    ) -> Result<Option<(i64, StoredTokenRecord)>, GitHubExtractorError> {
        let guard = self.records.read().await;
        for ((sess, inst_id), record) in guard.iter() {
            if sess == session_id && record.account_login.eq_ignore_ascii_case(account_login) {
                return Ok(Some((*inst_id, record.clone())));
            }
        }
        Ok(None)
    }
}

/// Key representing a tenant session and GitHub installation ID.
pub type InstallationKey = (Uuid, i64);

/// Per-installation concurrency lock for deduplicating token renewals.
type InFlightLock = Arc<Mutex<()>>;

/// Shared in-flight lock registry mapping installation keys to synchronization locks.
type InFlightRegistry = Arc<Mutex<HashMap<InstallationKey, InFlightLock>>>;

/// Core token lifecycle orchestrator providing proactive refresh and single-flight deduplication.
#[derive(Clone)]
pub struct InstallationTokenRefresher {
    store: Arc<dyn InstallationTokenStore>,
    client: Arc<GitHubApiClient>,
    app_config: Arc<GitHubAppConfig>,
    policy: TokenLifecyclePolicy,
    in_flight: InFlightRegistry,
}

impl InstallationTokenRefresher {
    /// Constructs a new `InstallationTokenRefresher`.
    #[must_use]
    pub fn new(
        store: Arc<dyn InstallationTokenStore>,
        client: Arc<GitHubApiClient>,
        app_config: Arc<GitHubAppConfig>,
        policy: TokenLifecyclePolicy,
    ) -> Self {
        Self {
            store,
            client,
            app_config,
            policy,
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reference to the active lifecycle policy.
    #[must_use]
    pub fn policy(&self) -> &TokenLifecyclePolicy {
        &self.policy
    }

    /// Retrieves an active installation token, proactively refreshing it if expired or expiring soon.
    ///
    /// Concurrent calls for the same installation are single-flight deduplicated to prevent
    /// thundering-herd API calls against GitHub.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if token renewal fails or the issued token violates policy.
    pub async fn get_or_refresh_token(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
    ) -> Result<String, GitHubExtractorError> {
        // Fast path: Check store without acquisition lock
        if let Some(record) = self.store.get_token(session_id, installation_id).await? {
            let freshness = record.freshness(self.policy.safety_margin_secs);
            if freshness.is_fresh() {
                debug!(
                    "Reusing fresh cached token for installation {installation_id} (expires at {})",
                    record.expires_at
                );
                return Ok(record.token);
            }
            debug!(
                "Cached token for installation {installation_id} requires renewal (status: {freshness:?})"
            );
        }

        // Slow path: Single-flight per-installation synchronization
        let lock = self
            .get_installation_lock(session_id, installation_id)
            .await;
        let _guard = lock.lock().await;

        // Double-check store inside the lock: Another task may have just finished refreshing
        if let Some(record) = self.store.get_token(session_id, installation_id).await? {
            let freshness = record.freshness(self.policy.safety_margin_secs);
            if freshness.is_fresh() {
                debug!(
                    "Token for installation {installation_id} was refreshed concurrently; reusing"
                );
                return Ok(record.token);
            }
        }

        // Execute token renewal
        self.execute_renewal(session_id, installation_id, account_login)
            .await
    }

    /// Forces immediate token renewal regardless of remaining TTL.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] on renewal failure.
    pub async fn force_refresh(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
    ) -> Result<String, GitHubExtractorError> {
        let lock = self
            .get_installation_lock(session_id, installation_id)
            .await;
        let _guard = lock.lock().await;
        self.execute_renewal(session_id, installation_id, account_login)
            .await
    }

    /// Evaluates the current freshness status of a cached token without renewing it.
    ///
    /// Returns `None` if no token is cached for this installation.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] on storage failure.
    pub async fn check_freshness(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<Option<TokenFreshness>, GitHubExtractorError> {
        let record = self.store.get_token(session_id, installation_id).await?;
        Ok(record.map(|r| r.freshness(self.policy.safety_margin_secs)))
    }

    /// Revokes and deletes a cached token record.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] on storage failure.
    pub async fn revoke(
        &self,
        session_id: &Uuid,
        installation_id: i64,
    ) -> Result<bool, GitHubExtractorError> {
        self.store.revoke_token(session_id, installation_id).await
    }

    /// Retrieves or registers a per-installation mutex for single-flight deduplication.
    async fn get_installation_lock(&self, session_id: &Uuid, installation_id: i64) -> InFlightLock {
        let mut map = self.in_flight.lock().await;
        map.entry((*session_id, installation_id))
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Executes token renewal with retries and exponential backoff.
    async fn execute_renewal(
        &self,
        session_id: &Uuid,
        installation_id: i64,
        account_login: &str,
    ) -> Result<String, GitHubExtractorError> {
        info!("Renewing installation access token for {account_login} (installation {installation_id})");

        let mut last_err = None;

        for attempt in 1..=self.policy.max_retries {
            match self.renew_once(installation_id).await {
                Ok(new_token) => {
                    // Validate least-privilege permissions under Rule 05
                    new_token.validate_permissions_with_policy(&self.policy.permission_policy)?;

                    let raw_token = new_token.token.clone();
                    self.store
                        .save_token(session_id, installation_id, account_login, &new_token)
                        .await?;

                    info!(
                        "Successfully renewed installation token for {account_login} (valid until {})",
                        new_token.expires_at
                    );
                    return Ok(raw_token);
                }
                Err(err) => {
                    warn!(
                        "Token renewal attempt {attempt}/{} for installation {installation_id} failed: {err}",
                        self.policy.max_retries
                    );
                    last_err = Some(err);

                    if attempt < self.policy.max_retries {
                        let delay_ms = self.policy.retry_backoff_ms * (1 << (attempt - 1));
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        let final_err = last_err.unwrap_or_else(|| GitHubExtractorError::BadCredentials {
            message: "Token renewal retry limit exhausted".to_string(),
        });
        error!("All token renewal attempts failed for installation {installation_id}: {final_err}");
        Err(final_err)
    }

    /// Issues a single renewal request to GitHub API.
    async fn renew_once(
        &self,
        installation_id: i64,
    ) -> Result<GitHubInstallationToken, GitHubExtractorError> {
        let app_jwt = crate::oauth::generate_app_jwt(&self.app_config)?;
        let request = InstallationTokenRequest::minimal_read_only();
        self.client
            .create_scoped_installation_access_token(installation_id as u64, &app_jwt, &request)
            .await
    }
}
