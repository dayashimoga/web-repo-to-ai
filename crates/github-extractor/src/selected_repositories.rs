//! # Selected Repository Access & Traversal Engine
//!
//! Provides paginated repository discovery, search, filtering, access verification,
//! and dynamic grant synchronization for GitHub App installations configured with
//! `repository_selection: "selected"`.
//!
//! ## Rule 05 Compliance & Architectural Invariants
//!
//! 1. **Bounded Traversal:** Pagination is bounded by configurable `max_pages` limits
//!    (default 50 pages / 5,000 repositories) to prevent unbounded loops or memory exhaustion.
//! 2. **Dual Identity Support:** Supports repository enumeration using both installation access
//!    tokens (`GET /installation/repositories`) and user tokens (`GET /user/installations/{id}/repositories`).
//! 3. **Dynamic Grant Synchronization:** Synchronizes webhook repository change events
//!    (`installation_repositories.added` and `installation_repositories.removed`) into active
//!    cached token records across sessions, preventing stale repository access grants.
//! 4. **Secret Masking:** Plaintext bearer tokens are never logged or formatted into debug strings.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::client::GitHubApiClient;
use crate::error::GitHubExtractorError;
use crate::installation::{InstalledRepositorySummary, RepositorySelection};
use crate::token_refresh::InstallationTokenStore;

/// Default maximum number of pages to traverse when enumerating repositories.
pub const DEFAULT_MAX_PAGINATION_PAGES: usize = 50;

/// Default page size for repository listings (maximum supported by GitHub is 100).
pub const DEFAULT_REPOSITORIES_PER_PAGE: u32 = 100;

/// Paginated repository enumeration listing for a GitHub App installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedRepositoryListing {
    /// Total number of accessible repositories reported by GitHub.
    pub total_count: usize,
    /// Repository selection mode (`all` or `selected`).
    pub repository_selection: RepositorySelection,
    /// Aggregated list of accessible repositories across all traversed pages.
    pub repositories: Vec<InstalledRepositorySummary>,
    /// Number of pages retrieved during enumeration.
    pub pages_fetched: usize,
    /// True if all accessible repositories were fetched without hitting page caps.
    pub is_complete: bool,
}

impl SelectedRepositoryListing {
    /// Returns true if no repositories were retrieved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }

    /// Returns the number of repositories in this listing.
    #[must_use]
    pub fn len(&self) -> usize {
        self.repositories.len()
    }

    /// Returns a list of all repository full slugs (`owner/repo`).
    #[must_use]
    pub fn full_names(&self) -> Vec<String> {
        self.repositories
            .iter()
            .map(|r| r.full_name.clone())
            .collect()
    }

    /// Checks if a repository with the given owner and name exists in this listing (case-insensitive).
    #[must_use]
    pub fn has_repository(&self, owner: &str, repo: &str) -> bool {
        let expected_slug = format!("{}/{}", owner.trim(), repo.trim());
        self.repositories
            .iter()
            .any(|r| r.full_name.eq_ignore_ascii_case(&expected_slug))
    }

    /// Finds a repository by owner and name (case-insensitive).
    #[must_use]
    pub fn find_by_slug(&self, owner: &str, repo: &str) -> Option<&InstalledRepositorySummary> {
        let expected_slug = format!("{}/{}", owner.trim(), repo.trim());
        self.repositories
            .iter()
            .find(|r| r.full_name.eq_ignore_ascii_case(&expected_slug))
    }

    /// Performs a case-insensitive keyword search across repository name, full_name, and description.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<InstalledRepositorySummary> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|r| {
                r.name.to_ascii_lowercase().contains(&q)
                    || r.full_name.to_ascii_lowercase().contains(&q)
                    || r.description
                        .as_deref()
                        .map(|d| d.to_ascii_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Filters repositories by visibility (private only vs public only).
    #[must_use]
    pub fn filter_by_visibility(&self, private_only: bool) -> Vec<InstalledRepositorySummary> {
        self.repositories
            .iter()
            .filter(|r| r.private == private_only)
            .cloned()
            .collect()
    }
}

/// Result of verifying access to a specific repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryAccessStatus {
    /// Repository is granted and accessible under this installation.
    Granted(InstalledRepositorySummary),
    /// Repository is not granted or not found.
    NotGranted {
        /// Requested repository owner.
        owner: String,
        /// Requested repository name.
        repo: String,
        /// Reason description.
        reason: String,
    },
    /// The token is expired, revoked, or rejected by GitHub (HTTP 401).
    TokenInvalid {
        /// Error description.
        reason: String,
    },
    /// Request was blocked by GitHub rate limits (HTTP 403/429).
    RateLimited {
        /// Retry delay suggested by upstream.
        retry_after_secs: Option<u64>,
    },
}

impl RepositoryAccessStatus {
    /// Returns true if access to the repository was verified and granted.
    #[must_use]
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted(_))
    }

    /// Unwraps the granted repository summary, panicking if not granted.
    #[must_use]
    pub fn unwrap_granted(self) -> InstalledRepositorySummary {
        match self {
            Self::Granted(summary) => summary,
            other => panic!("Called unwrap_granted on non-granted status: {other:?}"),
        }
    }
}

/// Summary of synchronizing dynamic repository grant changes (e.g. from webhook events).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncSummary {
    /// GitHub App installation ID.
    pub installation_id: i64,
    /// Number of repositories added.
    pub added_count: usize,
    /// Number of repositories removed.
    pub removed_count: usize,
    /// Number of session token records updated.
    pub updated_sessions_count: usize,
}

/// Service coordinating paginated repository discovery, search, verification,
/// and dynamic grant synchronization for GitHub App installations.
pub struct SelectedRepositoryManager {
    client: Arc<GitHubApiClient>,
    max_pages: usize,
    per_page: u32,
}

impl SelectedRepositoryManager {
    /// Creates a new `SelectedRepositoryManager` with default pagination bounds.
    pub fn new(client: Arc<GitHubApiClient>) -> Self {
        Self {
            client,
            max_pages: DEFAULT_MAX_PAGINATION_PAGES,
            per_page: DEFAULT_REPOSITORIES_PER_PAGE,
        }
    }

    /// Configures the maximum number of pagination pages to traverse.
    #[must_use]
    pub fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages.max(1);
        self
    }

    /// Configures the page size per request (clamped between 1 and 100).
    #[must_use]
    pub fn with_per_page(mut self, per_page: u32) -> Self {
        self.per_page = per_page.clamp(1, 100);
        self
    }

    /// Enumerates all repositories accessible to an installation access token across pagination pages.
    ///
    /// Traverses `GET /installation/repositories` until all repositories are retrieved or
    /// `max_pages` is reached.
    pub async fn fetch_all(
        &self,
        installation_token: &str,
    ) -> Result<SelectedRepositoryListing, GitHubExtractorError> {
        debug!(
            "Starting paginated repository enumeration with max_pages={}, per_page={}",
            self.max_pages, self.per_page
        );

        let mut accumulated = Vec::new();
        let mut total_count = 0;
        let mut selection = RepositorySelection::Selected;
        let mut pages_fetched = 0;
        let mut is_complete = true;

        for page in 1..=self.max_pages {
            let resp = self
                .client
                .list_installation_repositories(
                    installation_token,
                    Some(page as u32),
                    Some(self.per_page),
                )
                .await?;

            pages_fetched += 1;
            total_count = resp.total_count;
            selection = resp.repository_selection;

            let batch_len = resp.repositories.len();
            debug!("Fetched page {page}: {batch_len} repositories (total: {total_count})");

            accumulated.extend(resp.repositories);

            if batch_len == 0 || accumulated.len() >= total_count {
                break;
            }

            if page == self.max_pages && accumulated.len() < total_count {
                warn!(
                    "Reached maximum pagination pages cap ({}) with {}/{} repositories fetched",
                    self.max_pages,
                    accumulated.len(),
                    total_count
                );
                is_complete = false;
            }
        }

        info!(
            "Completed repository enumeration: {} repositories fetched across {pages_fetched} page(s) (selection={selection})",
            accumulated.len()
        );

        Ok(SelectedRepositoryListing {
            total_count,
            repository_selection: selection,
            repositories: accumulated,
            pages_fetched,
            is_complete,
        })
    }

    /// Enumerates all repositories accessible to a user for an installation across pagination pages.
    ///
    /// Traverses `GET /user/installations/{installation_id}/repositories` until all repositories
    /// are retrieved or `max_pages` is reached.
    pub async fn fetch_all_for_user(
        &self,
        user_token: &str,
        installation_id: u64,
    ) -> Result<SelectedRepositoryListing, GitHubExtractorError> {
        debug!(
            "Starting user repository enumeration for installation {installation_id} with max_pages={}, per_page={}",
            self.max_pages, self.per_page
        );

        let mut accumulated = Vec::new();
        let mut total_count = 0;
        let mut selection = RepositorySelection::Selected;
        let mut pages_fetched = 0;
        let mut is_complete = true;

        for page in 1..=self.max_pages {
            let resp = self
                .client
                .list_user_installation_repositories(
                    user_token,
                    installation_id,
                    Some(page as u32),
                    Some(self.per_page),
                )
                .await?;

            pages_fetched += 1;
            total_count = resp.total_count;
            selection = resp.repository_selection;

            let batch_len = resp.repositories.len();
            debug!("Fetched user page {page}: {batch_len} repositories (total: {total_count})");

            accumulated.extend(resp.repositories);

            if batch_len == 0 || accumulated.len() >= total_count {
                break;
            }

            if page == self.max_pages && accumulated.len() < total_count {
                warn!(
                    "Reached maximum user pagination cap ({}) with {}/{} repositories fetched",
                    self.max_pages,
                    accumulated.len(),
                    total_count
                );
                is_complete = false;
            }
        }

        info!(
            "Completed user repository enumeration: {} repositories fetched across {pages_fetched} page(s) for installation {installation_id}",
            accumulated.len()
        );

        Ok(SelectedRepositoryListing {
            total_count,
            repository_selection: selection,
            repositories: accumulated,
            pages_fetched,
            is_complete,
        })
    }

    /// Verifies live whether a specific repository is accessible to an installation token.
    pub async fn verify_repository_access(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RepositoryAccessStatus, GitHubExtractorError> {
        let trimmed_owner = owner.trim();
        let trimmed_repo = repo.trim();

        match self
            .client
            .check_repository_access(token, trimmed_owner, trimmed_repo)
            .await
        {
            Ok(Some(details)) => Ok(RepositoryAccessStatus::Granted(details.into())),
            Ok(None) => Ok(RepositoryAccessStatus::NotGranted {
                owner: trimmed_owner.to_string(),
                repo: trimmed_repo.to_string(),
                reason: "Repository not accessible or not found on GitHub".to_string(),
            }),
            Err(GitHubExtractorError::BadCredentials { message }) => {
                Ok(RepositoryAccessStatus::TokenInvalid { reason: message })
            }
            Err(GitHubExtractorError::RateLimitExceeded { reset_at, .. }) => {
                let now = Utc::now();
                let retry_after_secs = if reset_at > now {
                    #[allow(clippy::cast_sign_loss)]
                    Some((reset_at - now).num_seconds().max(0) as u64)
                } else {
                    None
                };
                Ok(RepositoryAccessStatus::RateLimited { retry_after_secs })
            }
            Err(e) => Err(e),
        }
    }

    /// Synchronizes dynamic repository grant additions and removals from webhook events into the token store.
    pub async fn sync_grant_change(
        &self,
        store: &dyn InstallationTokenStore,
        installation_id: i64,
        added: &[InstalledRepositorySummary],
        removed: &[InstalledRepositorySummary],
    ) -> Result<SyncSummary, GitHubExtractorError> {
        info!(
            "Synchronizing repository grants for installation {installation_id}: +{} added, -{} removed",
            added.len(),
            removed.len()
        );

        let updated_sessions_count = store
            .sync_installation_repositories(installation_id, added, removed)
            .await?;

        debug!(
            "Updated {updated_sessions_count} session records for installation {installation_id}"
        );

        Ok(SyncSummary {
            installation_id,
            added_count: added.len(),
            removed_count: removed.len(),
            updated_sessions_count,
        })
    }
}
