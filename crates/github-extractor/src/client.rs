//! # GitHub REST API Client
//!
//! Provides an asynchronous, rate-limit-aware HTTP client for GitHub repository
//! discovery, branch and tag enumeration, pagination, and reference disambiguation.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::time::Duration;

use bytes::Bytes;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::architecture::RepoArchitectureContext;
use crate::classification::{ClassifiedFile, TreeClassificationReport};
use crate::commit::{
    CommitAuthor, CommitSignatureVerification, GitHubCommitMetadata, PinnedRevision,
};
use crate::config_test::{ConfigTestExtractionReport, ExtractedConfigFile, ExtractedTestFile};
use crate::dependency::{DependencyExtractionReport, ExtractedDependencyFile};
use crate::documentation::{DocExtractionReport, ExtractedDocFile};
use crate::error::{redact_secrets, GitHubExtractorError};
use crate::manifest::RepoFileManifest;
use crate::metadata::{
    GitHubLanguages, GitHubLicense, GitHubRepoContext, GitHubRepositoryMetadata,
};
use crate::models::{GitHubBranch, GitHubRateLimit, GitHubTag, RefDiscoveryReport};
use crate::recovery::{
    scan_zip_archive, ArchiveFormat, SubtreeRecoveryConfig, SubtreeRecoveryReport, TreeStitcher,
};
use crate::source::{
    decode_base64, ExtractedSourceFile, GitBlob, SourceExtractionConfig, SourceExtractionReport,
};
use crate::target::GitHubTarget;
use crate::tree::{GitTreeResponse, TreeCompleteness};
use crate::tree_generator::RepoTree;

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
struct RawAuthorPayload {
    name: String,
    email: String,
    date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct RawTreeRef {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct RawVerificationPayload {
    #[serde(default)]
    verified: bool,
    reason: String,
    signature: Option<String>,
    payload: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawCommitPayload {
    author: Option<RawAuthorPayload>,
    committer: Option<RawAuthorPayload>,
    message: String,
    tree: RawTreeRef,
    #[serde(default)]
    comment_count: u32,
    verification: Option<RawVerificationPayload>,
}

#[derive(Debug, Deserialize)]
struct RawParentRef {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct RawCommitDetail {
    sha: String,
    node_id: Option<String>,
    commit: RawCommitPayload,
    #[serde(default)]
    parents: Vec<RawParentRef>,
}

#[derive(Debug, Deserialize)]
struct RawLicensePayload {
    key: String,
    name: String,
    spdx_id: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawOwnerPayload {
    login: String,
}

#[derive(Debug, Deserialize)]
struct RawRepoPayload {
    id: u64,
    node_id: Option<String>,
    name: String,
    full_name: String,
    owner: RawOwnerPayload,
    description: Option<String>,
    #[serde(default)]
    fork: bool,
    #[serde(default)]
    archived: bool,
    #[serde(default)]
    disabled: bool,
    #[serde(default = "default_visibility")]
    visibility: String,
    #[serde(default = "default_branch_name")]
    default_branch: String,
    homepage: Option<String>,
    language: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    license: Option<RawLicensePayload>,
    #[serde(default)]
    stargazers_count: u32,
    #[serde(default)]
    watchers_count: u32,
    #[serde(default)]
    forks_count: u32,
    #[serde(default)]
    open_issues_count: u32,
    #[serde(default)]
    size: u64,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pushed_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn default_visibility() -> String {
    "public".to_string()
}

fn default_branch_name() -> String {
    "main".to_string()
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

    /// Retrieves detailed commit metadata for a specific commit reference (SHA, branch, or tag).
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the commit does not exist.
    pub async fn get_commit(
        &self,
        target: &GitHubTarget,
        commit_ref: &str,
    ) -> Result<(GitHubCommitMetadata, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!(
            "{base_url}/repos/{}/{}/commits/{commit_ref}",
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

        let raw: RawCommitDetail = serde_json::from_str(&body_text).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::JsonDecodeError(sanitized)
        })?;

        let author = raw
            .commit
            .author
            .map(|a| CommitAuthor::new(a.name, a.email, a.date));
        let committer = raw
            .commit
            .committer
            .map(|c| CommitAuthor::new(c.name, c.email, c.date));
        let verification = raw
            .commit
            .verification
            .map(|v| CommitSignatureVerification {
                verified: v.verified,
                reason: v.reason,
                signature: v.signature,
                payload: v.payload,
            });

        let parents = raw.parents.into_iter().map(|p| p.sha).collect();

        let metadata = GitHubCommitMetadata {
            sha: raw.sha,
            node_id: raw.node_id,
            message: raw.commit.message,
            author,
            committer,
            tree_sha: raw.commit.tree.sha,
            parents,
            comment_count: raw.commit.comment_count,
            verification,
        };

        Ok((metadata, rate_limit))
    }

    /// Resolves any commit reference (or default branch HEAD) into a pinned 40-character commit SHA
    /// and comprehensive commit metadata.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the reference cannot be resolved or repository is missing.
    pub async fn resolve_commit_sha(
        &self,
        target: &GitHubTarget,
        ref_spec: Option<&str>,
    ) -> Result<(String, GitHubCommitMetadata, Option<GitHubRateLimit>), GitHubExtractorError> {
        let query_ref = ref_spec.or_else(|| target.ref_spec()).unwrap_or("HEAD");

        let (metadata, rate_limit) = self.get_commit(target, query_ref).await?;
        let full_sha = metadata.sha.clone();
        Ok((full_sha, metadata, rate_limit))
    }

    /// Resolves and pins a repository target to an immutable revision container (`PinnedRevision`),
    /// guaranteeing that subsequent file discovery and extraction are bound to an immutable commit SHA.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if ref resolution or commit query fails.
    pub async fn pin_revision(
        &self,
        target: &GitHubTarget,
    ) -> Result<PinnedRevision, GitHubExtractorError> {
        let effective_ref = target.ref_spec().map(ToString::to_string);
        let (full_sha, metadata, _rl) = self
            .resolve_commit_sha(target, effective_ref.as_deref())
            .await?;
        let root_tree_sha = metadata.tree_sha.clone();

        Ok(PinnedRevision::new(
            target.clone(),
            full_sha,
            root_tree_sha,
            metadata,
            effective_ref,
        ))
    }

    /// Retrieves detailed repository metadata for a given target repository.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the repository does not exist,
    /// or `GitHubExtractorError::RateLimitExceeded` if API limits are breached.
    pub async fn get_repo_metadata(
        &self,
        target: &GitHubTarget,
    ) -> Result<(GitHubRepositoryMetadata, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!("{base_url}/repos/{}/{}", target.owner, target.repo);

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

        let raw: RawRepoPayload = serde_json::from_str(&body_text).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::JsonDecodeError(sanitized)
        })?;

        let license = raw
            .license
            .map(|l| GitHubLicense::new(l.key, l.name, l.spdx_id, l.url));

        let metadata = GitHubRepositoryMetadata {
            id: raw.id,
            node_id: raw.node_id,
            name: raw.name,
            full_name: raw.full_name,
            owner: raw.owner.login,
            description: raw.description,
            fork: raw.fork,
            archived: raw.archived,
            disabled: raw.disabled,
            visibility: raw.visibility,
            default_branch: raw.default_branch,
            homepage: raw.homepage,
            primary_language: raw.language,
            topics: raw.topics,
            license,
            stargazers_count: raw.stargazers_count,
            watchers_count: raw.watchers_count,
            forks_count: raw.forks_count,
            open_issues_count: raw.open_issues_count,
            size_kb: raw.size,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            pushed_at: raw.pushed_at,
        };

        Ok((metadata, rate_limit))
    }

    /// Retrieves programming language breakdown statistics (in bytes) for a repository.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the API request fails or decoding fails.
    pub async fn get_repo_languages(
        &self,
        target: &GitHubTarget,
    ) -> Result<(GitHubLanguages, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!(
            "{base_url}/repos/{}/{}/languages",
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

        let breakdown: BTreeMap<String, u64> = serde_json::from_str(&body_text).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::JsonDecodeError(sanitized)
        })?;

        Ok((GitHubLanguages::new(breakdown), rate_limit))
    }

    /// Fetches unified repository context by retrieving metadata and language distribution in parallel,
    /// with optional immutable revision pinning.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if any underlying request fails.
    pub async fn fetch_repo_context(
        &self,
        target: &GitHubTarget,
        pin: bool,
    ) -> Result<GitHubRepoContext, GitHubExtractorError> {
        let (meta_res, lang_res) = tokio::join!(
            self.get_repo_metadata(target),
            self.get_repo_languages(target)
        );

        let (metadata, _meta_rl) = meta_res?;
        let (languages, _lang_rl) = lang_res?;

        let pinned_revision = if pin {
            Some(self.pin_revision(target).await?)
        } else {
            None
        };

        Ok(GitHubRepoContext::new(
            target.clone(),
            metadata,
            languages,
            pinned_revision,
        ))
    }

    /// Retrieves a Git tree object for a repository by tree SHA, optionally requesting
    /// full recursive traversal (`?recursive=1`).
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the tree does not exist,
    /// or `GitHubExtractorError::RateLimitExceeded` if API rate limits are breached.
    pub async fn get_tree(
        &self,
        target: &GitHubTarget,
        tree_sha: &str,
        recursive: bool,
    ) -> Result<(GitTreeResponse, Option<GitHubRateLimit>), GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = if recursive {
            format!(
                "{base_url}/repos/{}/{}/git/trees/{tree_sha}?recursive=1",
                target.owner, target.repo
            )
        } else {
            format!(
                "{base_url}/repos/{}/{}/git/trees/{tree_sha}",
                target.owner, target.repo
            )
        };

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

        let tree_res: GitTreeResponse = serde_json::from_str(&body_text).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::JsonDecodeError(sanitized)
        })?;

        Ok((tree_res, rate_limit))
    }

    /// Fetches the complete Git tree corresponding to an immutable `PinnedRevision`,
    /// guaranteeing zero ref-drift by querying the pinned `root_tree_sha`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the tree query fails.
    pub async fn fetch_pinned_tree(
        &self,
        pinned: &PinnedRevision,
        recursive: bool,
    ) -> Result<GitTreeResponse, GitHubExtractorError> {
        let (tree_res, _rl) = self
            .get_tree(pinned.target(), pinned.root_tree_sha(), recursive)
            .await?;
        Ok(tree_res)
    }

    /// Downloads a repository snapshot archive (tarball or zipball) for a specified ref or commit SHA.
    ///
    /// Requests `GET /repos/{owner}/{repo}/{archive_format}/{git_ref}`, following redirects
    /// to the underlying download endpoint, with credentials and token redaction.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the ref does not exist,
    /// or `GitHubExtractorError::RequestFailed` on network failure.
    pub async fn download_archive(
        &self,
        target: &GitHubTarget,
        git_ref: &str,
        format: ArchiveFormat,
    ) -> Result<Bytes, GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!(
            "{base_url}/repos/{}/{}/{}/{}",
            target.owner,
            target.repo,
            format.as_str(),
            git_ref
        );

        let mut headers = self.prepare_headers();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        let req = self.client.get(&url).headers(headers).timeout(self.timeout);

        let (res, _rate_limit) = self.execute_request(req, &url).await?;

        let bytes = res.bytes().await.map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::RequestFailed(sanitized)
        })?;

        Ok(bytes)
    }

    /// Orchestrates automated subtree recovery on a potentially truncated `GitTreeResponse`.
    ///
    /// If the tree is already complete, returns immediately with zero recovery queries.
    /// If unexpanded subtrees exist or truncation is detected, recursively queries unexpanded
    /// subtrees up to `config.max_subtrees` and `config.max_depth`.
    ///
    /// If the subtree count exceeds bounds or traversal fails to achieve 100% completeness,
    /// and `config.auto_fallback` is enabled, falls back to downloading a shallow zipball archive
    /// (if `fallback_ref` is provided, e.g. the commit SHA or target ref) to reconstruct the full tree.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if recovery fails.
    pub async fn recover_tree(
        &self,
        target: &GitHubTarget,
        mut base_tree: GitTreeResponse,
        config: &SubtreeRecoveryConfig,
        fallback_ref: Option<&str>,
    ) -> Result<(GitTreeResponse, SubtreeRecoveryReport), GitHubExtractorError> {
        let initial_completeness = base_tree.completeness();
        if initial_completeness.is_complete() {
            let total = base_tree.tree.len();
            return Ok((
                base_tree,
                SubtreeRecoveryReport {
                    initial_completeness: TreeCompleteness::Complete,
                    subtrees_attempted: 0,
                    subtrees_succeeded: 0,
                    subtrees_failed: 0,
                    additional_blobs_recovered: 0,
                    additional_trees_recovered: 0,
                    final_total_entries: total,
                    final_completeness: TreeCompleteness::Complete,
                    fallback_triggered: false,
                    remaining_unexpanded: Vec::new(),
                },
            ));
        }

        let mut subtrees_attempted = 0;
        let mut subtrees_succeeded = 0;
        let mut subtrees_failed = 0;
        let mut additional_blobs_recovered = 0;
        let mut additional_trees_recovered = 0;
        let mut fallback_triggered = false;

        // Collect initial unexpanded subtrees
        let initial_unexpanded = base_tree.find_unexpanded_subtrees();
        let initial_unexpanded_count = initial_unexpanded.len();

        // Check if we should immediately fallback to archive download
        if config.auto_fallback
            && (initial_unexpanded_count > config.max_subtrees
                || base_tree.tree.len() >= crate::tree::GITHUB_TREE_ENTRY_LIMIT)
            && fallback_ref.is_some()
        {
            let git_ref = fallback_ref.unwrap_or_default();
            info!(
                "Initial tree truncation too large ({initial_unexpanded_count} subtrees); triggering shallow archive fallback for {git_ref}",
            );
            match self
                .download_archive(target, git_ref, ArchiveFormat::Zipball)
                .await
            {
                Ok(bytes) => match scan_zip_archive(&bytes, true) {
                    Ok(recovered_tree) => {
                        let total = recovered_tree.tree.len();
                        let final_completeness = recovered_tree.completeness();
                        return Ok((
                            recovered_tree,
                            SubtreeRecoveryReport {
                                initial_completeness,
                                subtrees_attempted: 0,
                                subtrees_succeeded: 0,
                                subtrees_failed: 0,
                                additional_blobs_recovered: 0,
                                additional_trees_recovered: 0,
                                final_total_entries: total,
                                final_completeness,
                                fallback_triggered: true,
                                remaining_unexpanded: Vec::new(),
                            },
                        ));
                    }
                    Err(e) => {
                        warn!("Failed to parse zip archive fallback: {e}");
                    }
                },
                Err(e) => {
                    warn!("Failed to download archive fallback: {e}");
                }
            }
        }

        // Bounded queue for subtree queries: (parent_dir_path, subtree_sha, current_depth)
        let mut queue: VecDeque<(String, String, usize)> = VecDeque::new();
        let mut visited_shas: HashSet<String> = HashSet::new();

        for entry in initial_unexpanded {
            if visited_shas.insert(entry.sha.clone()) {
                queue.push_back((entry.path.clone(), entry.sha.clone(), 0));
            }
        }

        while let Some((parent_dir, sha, depth)) = queue.pop_front() {
            if subtrees_attempted >= config.max_subtrees {
                warn!(
                    "Subtree recovery reached max_subtrees limit ({}); stopping traversal",
                    config.max_subtrees
                );
                break;
            }

            subtrees_attempted += 1;
            debug!("Querying subtree {parent_dir} (sha: {sha}, depth: {depth})");

            match self.get_tree(target, &sha, true).await {
                Ok((subtree_response, _rate_limit)) => {
                    subtrees_succeeded += 1;

                    // Stitch recovered entries into base_tree
                    let (blobs, trees) = TreeStitcher::stitch_subtree(
                        &mut base_tree,
                        &parent_dir,
                        &subtree_response,
                    );
                    additional_blobs_recovered += blobs;
                    additional_trees_recovered += trees;

                    // If the subtree itself contains unexpanded subtrees, and depth + 1 < max_depth, queue them
                    if depth + 1 < config.max_depth {
                        for sub in subtree_response.find_unexpanded_subtrees() {
                            let stitched_sub_path =
                                TreeStitcher::stitch_child_path(&parent_dir, &sub.path);
                            if visited_shas.insert(sub.sha.clone()) {
                                queue.push_back((stitched_sub_path, sub.sha.clone(), depth + 1));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch subtree {parent_dir} (sha: {sha}): {e}");
                    subtrees_failed += 1;
                }
            }
        }

        // If all queued subtrees succeeded and no unexpanded subtrees remain, the tree is fully expanded
        let remaining = base_tree.find_unexpanded_subtrees();
        if queue.is_empty()
            && subtrees_failed == 0
            && remaining.is_empty()
            && base_tree.tree.len() < crate::tree::GITHUB_TREE_ENTRY_LIMIT
        {
            base_tree.truncated = false;
        }

        // Re-analyze completeness after stitching
        let mut final_completeness = base_tree.completeness();
        let mut remaining_unexpanded = Vec::new();

        if let TreeCompleteness::Truncated(ref report) = final_completeness {
            remaining_unexpanded = report.unexpanded_subtrees.clone();

            // If still incomplete and auto_fallback enabled with fallback_ref, attempt archive fallback
            if config.auto_fallback && fallback_ref.is_some() && !remaining_unexpanded.is_empty() {
                let git_ref = fallback_ref.unwrap_or_default();
                info!("Subtree recovery left unexpanded subtrees; triggering archive fallback for {git_ref}");
                if let Ok(bytes) = self
                    .download_archive(target, git_ref, ArchiveFormat::Zipball)
                    .await
                {
                    if let Ok(archive_tree) = scan_zip_archive(&bytes, true) {
                        base_tree = archive_tree;
                        final_completeness = base_tree.completeness();
                        fallback_triggered = true;
                        remaining_unexpanded.clear();
                    }
                }
            }
        }

        // If tree is now complete, clear truncated flag on base_tree
        if final_completeness.is_complete() {
            base_tree.truncated = false;
        }

        let report = SubtreeRecoveryReport {
            initial_completeness,
            subtrees_attempted,
            subtrees_succeeded,
            subtrees_failed,
            additional_blobs_recovered,
            additional_trees_recovered,
            final_total_entries: base_tree.tree.len(),
            final_completeness,
            fallback_triggered,
            remaining_unexpanded,
        };

        Ok((base_tree, report))
    }

    /// Fetches and recovers the Git tree corresponding to an immutable `PinnedRevision`.
    ///
    /// Guarantees zero ref-drift by using the pinned commit SHA as immutable anchor and fallback ref.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the initial fetch or recovery fails.
    pub async fn recover_pinned_tree(
        &self,
        pinned: &PinnedRevision,
        config: &SubtreeRecoveryConfig,
    ) -> Result<(GitTreeResponse, SubtreeRecoveryReport), GitHubExtractorError> {
        let base_tree = self.fetch_pinned_tree(pinned, true).await?;
        self.recover_tree(
            pinned.target(),
            base_tree,
            config,
            Some(pinned.commit_sha()),
        )
        .await
    }

    /// Fetches a Git blob by its SHA-1 object hash.
    ///
    /// Supports both raw payload streaming (`Accept: application/vnd.github.raw`)
    /// and standard JSON base64-encoded responses.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::ResourceNotFound` if the blob does not exist,
    /// or `GitHubExtractorError::BlobError` if decoding fails.
    pub async fn get_blob(
        &self,
        target: &GitHubTarget,
        file_sha: &str,
    ) -> Result<GitBlob, GitHubExtractorError> {
        let base_url = self.effective_base_url(target);
        let url = format!(
            "{base_url}/repos/{}/{}/git/blobs/{file_sha}",
            target.owner, target.repo
        );

        let mut headers = self.prepare_headers();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "application/vnd.github.raw, application/vnd.github.v3.raw, application/vnd.github.v3+json;q=0.9",
            ),
        );

        let req = self.client.get(&url).headers(headers).timeout(self.timeout);

        let (res, _rate_limit) = self.execute_request(req, &url).await?;

        let content_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = res.bytes().await.map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::RequestFailed(sanitized)
        })?;

        // If JSON is returned or payload looks like JSON, check for base64 encoding
        #[derive(Deserialize)]
        struct RawBlobPayload {
            sha: Option<String>,
            encoding: Option<String>,
            content: Option<String>,
        }

        if content_type.contains("json") || bytes.starts_with(b"{") {
            if let Ok(payload) = serde_json::from_slice::<RawBlobPayload>(&bytes) {
                if payload.encoding.as_deref() == Some("base64") {
                    if let Some(encoded) = payload.content {
                        let decoded = decode_base64(&encoded).map_err(|e| {
                            GitHubExtractorError::BlobError {
                                sha: file_sha.to_string(),
                                message: format!("Base64 decoding failed: {e}"),
                            }
                        })?;
                        let sha = payload.sha.unwrap_or_else(|| file_sha.to_string());
                        return Ok(GitBlob::new(sha, decoded, "base64"));
                    }
                }
            }
        }

        Ok(GitBlob::new(file_sha, bytes.to_vec(), "raw"))
    }

    /// Fetches and verifies a single classified file, producing an `ExtractedSourceFile`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub async fn fetch_source_file(
        &self,
        target: &GitHubTarget,
        file: &ClassifiedFile,
        config: &SourceExtractionConfig,
    ) -> Result<ExtractedSourceFile, GitHubExtractorError> {
        let blob = self.get_blob(target, &file.sha).await?;
        ExtractedSourceFile::from_blob(&file.path, &blob, config)
    }

    /// Concurrently extracts and verifies multiple classified source code files with bounded concurrency.
    ///
    /// Uses a `tokio::sync::Semaphore` to strictly limit concurrent HTTP requests, recording
    /// individual successes, failures, and unsupported files into a consolidated `SourceExtractionReport`.
    pub async fn extract_source_files(
        &self,
        target: &GitHubTarget,
        files: &[ClassifiedFile],
        config: &SourceExtractionConfig,
    ) -> SourceExtractionReport {
        let start = std::time::Instant::now();
        let mut report = SourceExtractionReport::new();

        let mut eligible_files = Vec::new();
        for file in files {
            if !file.is_eligible {
                if file.category.is_binary() {
                    report.add_unsupported(file.clone());
                }
            } else {
                eligible_files.push(file.clone());
            }
        }

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency));
        let mut task_handles = Vec::with_capacity(eligible_files.len());

        for file in eligible_files {
            let sem = semaphore.clone();
            let client = self.clone();
            let target = target.clone();
            let f = file.clone();
            let cfg = config.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| GitHubExtractorError::RequestFailed(e.to_string()))?;
                client.fetch_source_file(&target, &f, &cfg).await
            });

            task_handles.push((file, handle));
        }

        for (file, handle) in task_handles {
            match handle.await {
                Ok(Ok(extracted)) => {
                    report.add_extracted(extracted);
                }
                Ok(Err(err)) => {
                    let err_str = err.to_string();
                    if err_str.contains("Binary or null bytes") {
                        report.add_unsupported(file);
                    } else {
                        report.add_failed(file.path, file.sha, err_str);
                    }
                }
                Err(join_err) => {
                    report.add_failed(file.path, file.sha, format!("Task join error: {join_err}"));
                }
            }
        }

        report.duration = start.elapsed();
        report
    }

    /// Fetches and verifies a single classified documentation file, producing an `ExtractedDocFile`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub async fn fetch_doc_file(
        &self,
        target: &GitHubTarget,
        file: &ClassifiedFile,
        config: &SourceExtractionConfig,
    ) -> Result<ExtractedDocFile, GitHubExtractorError> {
        let blob = self.get_blob(target, &file.sha).await?;
        ExtractedDocFile::from_blob(&file.path, &blob, config)
    }

    /// Concurrently extracts and parses repository documentation files with bounded concurrency.
    ///
    /// Identifies document titles, frontmatter, and formats, collecting extracted docs, failures,
    /// and primary README/LICENSE pointers into a consolidated `DocExtractionReport`.
    pub async fn extract_doc_files(
        &self,
        target: &GitHubTarget,
        files: &[ClassifiedFile],
        config: &SourceExtractionConfig,
    ) -> DocExtractionReport {
        let start = std::time::Instant::now();
        let mut report = DocExtractionReport::new();

        let mut eligible_files = Vec::new();
        for file in files {
            if !file.is_eligible {
                if file.category.is_binary() {
                    report.add_unsupported(file.clone());
                }
            } else {
                eligible_files.push(file.clone());
            }
        }

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency));
        let mut task_handles = Vec::with_capacity(eligible_files.len());

        for file in eligible_files {
            let sem = semaphore.clone();
            let client = self.clone();
            let target = target.clone();
            let f = file.clone();
            let cfg = config.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| GitHubExtractorError::RequestFailed(e.to_string()))?;
                client.fetch_doc_file(&target, &f, &cfg).await
            });

            task_handles.push((file, handle));
        }

        for (file, handle) in task_handles {
            match handle.await {
                Ok(Ok(extracted)) => {
                    report.add_extracted(extracted);
                }
                Ok(Err(err)) => {
                    let err_str = err.to_string();
                    if err_str.contains("Binary or null bytes") {
                        report.add_unsupported(file);
                    } else {
                        report.add_failed(file.path, file.sha, err_str);
                    }
                }
                Err(join_err) => {
                    report.add_failed(file.path, file.sha, format!("Task join error: {join_err}"));
                }
            }
        }

        report.duration = start.elapsed();
        report
    }

    /// Fetches and parses a single classified dependency manifest or lockfile, producing an `ExtractedDependencyFile`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub async fn fetch_dependency_file(
        &self,
        target: &GitHubTarget,
        file: &ClassifiedFile,
        config: &SourceExtractionConfig,
    ) -> Result<ExtractedDependencyFile, GitHubExtractorError> {
        let blob = self.get_blob(target, &file.sha).await?;
        ExtractedDependencyFile::from_blob(&file.path, &blob, config)
    }

    /// Concurrently extracts and parses repository dependency manifests and lockfiles with bounded concurrency.
    ///
    /// Identifies ecosystems, parses declared dependencies, and collects extracted manifests, lockfiles,
    /// and failures into a consolidated `DependencyExtractionReport`.
    pub async fn extract_dependency_files(
        &self,
        target: &GitHubTarget,
        files: &[ClassifiedFile],
        config: &SourceExtractionConfig,
    ) -> DependencyExtractionReport {
        let start = std::time::Instant::now();
        let mut report = DependencyExtractionReport::new();

        let mut eligible_files = Vec::new();
        for file in files {
            if !file.is_eligible {
                if file.category.is_binary() {
                    report.add_unsupported(file.clone());
                }
            } else {
                eligible_files.push(file.clone());
            }
        }

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency));
        let mut task_handles = Vec::with_capacity(eligible_files.len());

        for file in eligible_files {
            let sem = semaphore.clone();
            let client = self.clone();
            let target = target.clone();
            let f = file.clone();
            let cfg = config.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| GitHubExtractorError::RequestFailed(e.to_string()))?;
                client.fetch_dependency_file(&target, &f, &cfg).await
            });

            task_handles.push((file, handle));
        }

        for (file, handle) in task_handles {
            match handle.await {
                Ok(Ok(extracted)) => {
                    report.add_extracted(extracted);
                }
                Ok(Err(err)) => {
                    let err_str = err.to_string();
                    if err_str.contains("Binary or null bytes") {
                        report.add_unsupported(file);
                    } else {
                        report.add_failed(file.path, file.sha, err_str);
                    }
                }
                Err(join_err) => {
                    report.add_failed(file.path, file.sha, format!("Task join error: {join_err}"));
                }
            }
        }

        report.duration = start.elapsed();
        report
    }

    /// Fetches and verifies a single classified configuration file, producing an `ExtractedConfigFile`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub async fn fetch_config_file(
        &self,
        target: &GitHubTarget,
        file: &ClassifiedFile,
        config: &SourceExtractionConfig,
    ) -> Result<ExtractedConfigFile, GitHubExtractorError> {
        let blob = self.get_blob(target, &file.sha).await?;
        ExtractedConfigFile::from_blob(&file.path, &blob, config)
    }

    /// Fetches and verifies a single classified test file or fixture, producing an `ExtractedTestFile`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError::BlobError` if Git SHA verification fails or binary content is detected.
    pub async fn fetch_test_file(
        &self,
        target: &GitHubTarget,
        file: &ClassifiedFile,
        config: &SourceExtractionConfig,
    ) -> Result<ExtractedTestFile, GitHubExtractorError> {
        let blob = self.get_blob(target, &file.sha).await?;
        ExtractedTestFile::from_blob(&file.path, &blob, config, file.language.clone())
    }

    /// Concurrently extracts and parses repository configuration and test/fixture files with bounded concurrency.
    ///
    /// Classifies configs and tests, collecting extracted configs, test files, and failures into a consolidated
    /// `ConfigTestExtractionReport`.
    pub async fn extract_config_and_test_files(
        &self,
        target: &GitHubTarget,
        files: &[ClassifiedFile],
        config: &SourceExtractionConfig,
    ) -> ConfigTestExtractionReport {
        let start = std::time::Instant::now();
        let mut report = ConfigTestExtractionReport::new();

        let mut eligible_files = Vec::new();
        for file in files {
            if !file.is_eligible {
                if file.category.is_binary() {
                    report.add_unsupported(file.clone());
                }
            } else {
                eligible_files.push(file.clone());
            }
        }

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency));
        let mut task_handles = Vec::with_capacity(eligible_files.len());

        for file in eligible_files {
            let sem = semaphore.clone();
            let client = self.clone();
            let target = target.clone();
            let f = file.clone();
            let cfg = config.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| GitHubExtractorError::RequestFailed(e.to_string()))?;

                if f.category.is_test() {
                    client
                        .fetch_test_file(&target, &f, &cfg)
                        .await
                        .map(EitherConfigOrTest::Test)
                } else {
                    client
                        .fetch_config_file(&target, &f, &cfg)
                        .await
                        .map(EitherConfigOrTest::Config)
                }
            });

            task_handles.push((file, handle));
        }

        for (file, handle) in task_handles {
            match handle.await {
                Ok(Ok(EitherConfigOrTest::Config(c))) => {
                    report.add_config(c);
                }
                Ok(Ok(EitherConfigOrTest::Test(t))) => {
                    report.add_test(t);
                }
                Ok(Err(err)) => {
                    let err_str = err.to_string();
                    if err_str.contains("Binary or null bytes") {
                        report.add_unsupported(file);
                    } else {
                        report.add_failed(file.path, file.sha, err_str);
                    }
                }
                Err(join_err) => {
                    report.add_failed(file.path, file.sha, format!("Task join error: {join_err}"));
                }
            }
        }

        report.duration = start.elapsed();
        report
    }

    /// Generates a structured repository directory tree from a `GitTreeResponse`.
    #[must_use]
    pub fn generate_repo_tree(&self, tree_response: &GitTreeResponse) -> RepoTree {
        RepoTree::from_tree_response(tree_response)
    }

    /// Generates a structured repository directory tree from classified repository files.
    #[must_use]
    pub fn generate_repo_tree_classified(&self, classified_files: &[ClassifiedFile]) -> RepoTree {
        RepoTree::from_classified_files(classified_files)
    }

    /// Assembles a consolidated file manifest and completeness accounting index.
    #[must_use]
    pub fn generate_file_manifest(
        &self,
        classification: &TreeClassificationReport,
        sources: Option<&SourceExtractionReport>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
    ) -> RepoFileManifest {
        RepoFileManifest::from_classification_and_reports(
            classification,
            sources,
            docs,
            dependencies,
            configs_tests,
        )
    }

    /// Synthesizes automated architecture context, workspace topology, component catalog, and entry points.
    #[must_use]
    pub fn generate_architecture_context(
        &self,
        classification: &TreeClassificationReport,
        manifest: Option<&RepoFileManifest>,
        docs: Option<&DocExtractionReport>,
        dependencies: Option<&DependencyExtractionReport>,
        configs_tests: Option<&ConfigTestExtractionReport>,
        metadata: Option<&GitHubRepositoryMetadata>,
    ) -> RepoArchitectureContext {
        RepoArchitectureContext::from_repository_data(
            classification,
            manifest,
            docs,
            dependencies,
            configs_tests,
            metadata,
        )
    }
}

enum EitherConfigOrTest {
    Config(ExtractedConfigFile),
    Test(ExtractedTestFile),
}
