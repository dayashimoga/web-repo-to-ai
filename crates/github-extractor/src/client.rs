//! # GitHub REST API Client
//!
//! Provides an asynchronous, rate-limit-aware HTTP client for GitHub repository
//! discovery, branch and tag enumeration, pagination, and reference disambiguation.

use std::time::Duration;

use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::debug;

use crate::error::{redact_secrets, GitHubExtractorError};
use crate::models::{GitHubBranch, GitHubRateLimit, GitHubTag, RefDiscoveryReport};
use crate::target::GitHubTarget;

/// Default HTTP request timeout for GitHub API calls.
pub const DEFAULT_GITHUB_TIMEOUT: Duration = Duration::from_secs(30);
/// Default User-Agent header string.
pub const DEFAULT_USER_AGENT: &str = "web-repo-to-ai/0.1.0";
/// GitHub REST API v3 Accept header.
pub const GITHUB_API_VERSION_HEADER: &str = "application/vnd.github.v3+json";
/// Default pagination page size (GitHub maximum is 100).
pub const DEFAULT_PER_PAGE: usize = 100;
/// Default maximum pagination depth (10 pages * 100 = 1,000 items ceiling).
pub const DEFAULT_MAX_PAGES: usize = 10;

#[derive(Debug, Deserialize)]
struct RawCommitRef {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct RawBranchItem {
    name: String,
    commit: RawCommitRef,
    #[serde(default)]
    protected: bool,
}

#[derive(Debug, Deserialize)]
struct RawTagItem {
    name: String,
    commit: RawCommitRef,
    zipball_url: Option<String>,
    tarball_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawApiErrorResponse {
    message: Option<String>,
}

/// Asynchronous client for querying the GitHub REST API.
#[derive(Debug, Clone)]
pub struct GitHubApiClient {
    client: Client,
    base_url_override: Option<String>,
    auth_token: Option<String>,
    timeout: Duration,
    max_pages: usize,
    per_page: usize,
}

impl Default for GitHubApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubApiClient {
    /// Creates a new `GitHubApiClient` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_GITHUB_TIMEOUT)
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url_override: None,
            auth_token: None,
            timeout: DEFAULT_GITHUB_TIMEOUT,
            max_pages: DEFAULT_MAX_PAGES,
            per_page: DEFAULT_PER_PAGE,
        }
    }

    /// Injects an optional authentication token (Personal Access Token or GitHub App installation token).
    #[must_use]
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Sets an explicit API base URL override (useful for mock servers in testing or GHES).
    #[must_use]
    pub fn with_base_url_override(mut self, base_url: impl Into<String>) -> Self {
        self.base_url_override = Some(base_url.into().trim_end_matches('/').to_string());
        self
    }

    /// Configures the request timeout duration.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Configures the maximum number of pagination pages to traverse.
    #[must_use]
    pub const fn with_max_pages(mut self, max: usize) -> Self {
        self.max_pages = max;
        self
    }

    /// Configures the number of items requested per page (maximum 100).
    #[must_use]
    pub const fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    /// Injects a preconfigured `reqwest::Client`.
    #[must_use]
    pub fn with_custom_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Computes the effective REST API base URL for a given target repository.
    #[must_use]
    pub fn effective_base_url(&self, target: &GitHubTarget) -> String {
        if let Some(base) = &self.base_url_override {
            base.clone()
        } else {
            target.api_base_url()
        }
    }

    /// Prepares default headers for GitHub REST API requests.
    fn prepare_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static(GITHUB_API_VERSION_HEADER));

        if let Some(token) = &self.auth_token {
            let auth_val = format!("Bearer {token}");
            if let Ok(hv) = HeaderValue::from_str(&auth_val) {
                headers.insert(AUTHORIZATION, hv);
            }
        }

        headers
    }

    /// Executes an HTTP request with error decoding, rate limit inspection, and token redaction.
    async fn execute_request(
        &self,
        req: reqwest::RequestBuilder,
        request_url: &str,
    ) -> Result<(reqwest::Response, Option<GitHubRateLimit>), GitHubExtractorError> {
        let res = req.send().await.map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::RequestFailed(sanitized)
        })?;

        let rate_limit = GitHubRateLimit::from_headers(res.headers());
        let status = res.status();

        if status.is_success() {
            return Ok((res, rate_limit));
        }

        // Decode error payload
        let body_text = res.text().await.unwrap_or_default();
        let sanitized_body = redact_secrets(&body_text);
        let error_msg = serde_json::from_str::<RawApiErrorResponse>(&body_text)
            .ok()
            .and_then(|r| r.message)
            .map(|m| redact_secrets(&m))
            .unwrap_or(sanitized_body);

        let sanitized_url = redact_secrets(request_url);

        // Check for rate limit exhaustion
        if (status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS)
            && rate_limit
                .as_ref()
                .is_some_and(GitHubRateLimit::is_exhausted)
        {
            let rl = rate_limit.unwrap();
            return Err(GitHubExtractorError::RateLimitExceeded {
                limit: rl.limit,
                reset_at: rl.reset_at,
                message: error_msg,
            });
        }

        match status {
            StatusCode::UNAUTHORIZED => {
                Err(GitHubExtractorError::BadCredentials { message: error_msg })
            }
            StatusCode::FORBIDDEN => Err(GitHubExtractorError::Forbidden { message: error_msg }),
            StatusCode::NOT_FOUND => Err(GitHubExtractorError::ResourceNotFound {
                url: sanitized_url,
                message: error_msg,
            }),
            _ => Err(GitHubExtractorError::HttpError {
                status: status.as_u16(),
                message: error_msg,
            }),
        }
    }

    /// Lists branches for a target repository with automatic pagination up to `max_pages`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the API request fails, returns 404, or exceeds rate limits.
    pub async fn list_branches(
        &self,
        target: &GitHubTarget,
    ) -> Result<(Vec<GitHubBranch>, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let mut all_branches = Vec::new();
        let mut last_rate_limit = None;

        for page in 1..=self.max_pages {
            let url = format!(
                "{base_url}/repos/{}/{}/branches?per_page={}&page={page}",
                target.owner, target.repo, self.per_page
            );

            debug!("Fetching GitHub branches page {page} from {url}");
            let req = self
                .client
                .get(&url)
                .headers(self.prepare_headers())
                .timeout(self.timeout);

            let (res, rate_limit) = self.execute_request(req, &url).await?;
            last_rate_limit = rate_limit;

            let body_text = res.text().await.map_err(|e| {
                let sanitized = redact_secrets(&e.to_string());
                GitHubExtractorError::RequestFailed(sanitized)
            })?;

            let items: Vec<RawBranchItem> = serde_json::from_str(&body_text).map_err(|e| {
                let sanitized = redact_secrets(&e.to_string());
                GitHubExtractorError::JsonDecodeError(sanitized)
            })?;

            let count = items.len();
            for raw in items {
                all_branches.push(GitHubBranch::new(raw.name, raw.commit.sha, raw.protected));
            }

            if count < self.per_page {
                break;
            }
        }

        Ok((all_branches, last_rate_limit))
    }

    /// Retrieves detailed branch information for a single specific branch.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the branch does not exist.
    pub async fn get_branch(
        &self,
        target: &GitHubTarget,
        branch_name: &str,
    ) -> Result<(GitHubBranch, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!(
            "{base_url}/repos/{}/{}/branches/{branch_name}",
            target.owner, target.repo
        );

        let req = self
            .client
            .get(&url)
            .headers(self.prepare_headers())
            .timeout(self.timeout);

        let (res, rate_limit) = self.execute_request(req, &url).await?;

        let body_text = res.text().await.map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::RequestFailed(sanitized)
        })?;

        let raw: RawBranchItem = serde_json::from_str(&body_text).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::JsonDecodeError(sanitized)
        })?;

        Ok((
            GitHubBranch::new(raw.name, raw.commit.sha, raw.protected),
            rate_limit,
        ))
    }

    /// Lists release tags for a target repository with automatic pagination up to `max_pages`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the API request fails, returns 404, or exceeds rate limits.
    pub async fn list_tags(
        &self,
        target: &GitHubTarget,
    ) -> Result<(Vec<GitHubTag>, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let mut all_tags = Vec::new();
        let mut last_rate_limit = None;

        for page in 1..=self.max_pages {
            let url = format!(
                "{base_url}/repos/{}/{}/tags?per_page={}&page={page}",
                target.owner, target.repo, self.per_page
            );

            debug!("Fetching GitHub tags page {page} from {url}");
            let req = self
                .client
                .get(&url)
                .headers(self.prepare_headers())
                .timeout(self.timeout);

            let (res, rate_limit) = self.execute_request(req, &url).await?;
            last_rate_limit = rate_limit;

            let body_text = res.text().await.map_err(|e| {
                let sanitized = redact_secrets(&e.to_string());
                GitHubExtractorError::RequestFailed(sanitized)
            })?;

            let items: Vec<RawTagItem> = serde_json::from_str(&body_text).map_err(|e| {
                let sanitized = redact_secrets(&e.to_string());
                GitHubExtractorError::JsonDecodeError(sanitized)
            })?;

            let count = items.len();
            for raw in items {
                all_tags.push(GitHubTag::new(
                    raw.name,
                    raw.commit.sha,
                    raw.zipball_url,
                    raw.tarball_url,
                ));
            }

            if count < self.per_page {
                break;
            }
        }

        Ok((all_tags, last_rate_limit))
    }

    /// Discovers all available branches and tags for a repository in parallel, returning
    /// a consolidated `RefDiscoveryReport`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the API requests fail or repository is not found.
    pub async fn discover_refs(
        &self,
        target: &GitHubTarget,
    ) -> Result<RefDiscoveryReport, GitHubExtractorError> {
        let (branches_res, tags_res) =
            tokio::join!(self.list_branches(target), self.list_tags(target));

        let (branches, branch_rl) = branches_res?;
        let (tags, tag_rl) = tags_res?;

        let rate_limit = tag_rl.or(branch_rl);

        // Determine default branch if among branches
        let default_branch = if branches.iter().any(|b| b.name == "main") {
            Some("main".to_string())
        } else if branches.iter().any(|b| b.name == "master") {
            Some("master".to_string())
        } else {
            branches.first().map(|b| b.name.clone())
        };

        Ok(RefDiscoveryReport {
            owner: target.owner.clone(),
            repo: target.repo.clone(),
            default_branch,
            branches,
            tags,
            rate_limit,
            discovered_at: Utc::now(),
        })
    }

    /// Disambiguates a parsed `GitHubTarget` against discovered remote references.
    ///
    /// If the target URL was originally decomposed into a single-segment ref (e.g. `feature`)
    /// and a subpath (e.g. `pipeline/crates`), but remote branches contain `feature/pipeline`,
    /// this method produces a refined `GitHubTarget` with the exact ref and subpath.
    #[must_use]
    pub fn disambiguate_target(
        &self,
        target: &GitHubTarget,
        report: &RefDiscoveryReport,
    ) -> GitHubTarget {
        // Only refine Tree or Blob targets that have an ambiguous path
        let raw_tail = match (&target.ref_spec, &target.subpath) {
            (Some(r), Some(p)) => format!("{r}/{p}"),
            (Some(r), None) => r.clone(),
            _ => return target.clone(),
        };

        if let Some((exact_ref, exact_subpath)) = report.disambiguate_ref_and_subpath(&raw_tail) {
            target.with_refined_ref_and_subpath(exact_ref, exact_subpath)
        } else {
            target.clone()
        }
    }
}
