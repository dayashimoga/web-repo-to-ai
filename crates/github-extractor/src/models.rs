//! # GitHub Ref Discovery Models
//!
//! Strongly-typed domain structures for Git branches, tags, rate limiting metrics,
//! and repository reference discovery reports.

use chrono::{DateTime, TimeZone, Utc};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

/// GitHub REST API rate limiting status extracted from HTTP headers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRateLimit {
    /// Maximum allowed requests per hourly window.
    pub limit: u32,
    /// Number of requests remaining in the current window.
    pub remaining: u32,
    /// UTC timestamp when the rate limit window resets.
    pub reset_at: DateTime<Utc>,
    /// Number of requests consumed in the current window, if provided.
    pub used: Option<u32>,
}

impl GitHubRateLimit {
    /// Attempts to parse GitHub rate limit headers from an HTTP response header map.
    #[must_use]
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let limit = headers
            .get("x-ratelimit-limit")?
            .to_str()
            .ok()?
            .parse::<u32>()
            .ok()?;

        let remaining = headers
            .get("x-ratelimit-remaining")?
            .to_str()
            .ok()?
            .parse::<u32>()
            .ok()?;

        let reset_epoch = headers
            .get("x-ratelimit-reset")?
            .to_str()
            .ok()?
            .parse::<i64>()
            .ok()?;

        let reset_at = Utc
            .timestamp_opt(reset_epoch, 0)
            .single()
            .unwrap_or_else(Utc::now);

        let used = headers
            .get("x-ratelimit-used")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        Some(Self {
            limit,
            remaining,
            reset_at,
            used,
        })
    }

    /// Returns whether the rate limit quota is fully exhausted (`remaining == 0`).
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }

    /// Returns the number of seconds remaining until the rate limit window resets.
    #[must_use]
    pub fn seconds_until_reset(&self) -> i64 {
        let diff = self.reset_at.signed_duration_since(Utc::now());
        diff.num_seconds().max(0)
    }
}

/// Information representing a Git branch discovered from the GitHub REST API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubBranch {
    /// Branch name (e.g. `main`, `master`, `feature/pipeline-v2`).
    pub name: String,
    /// 40-character hexadecimal commit SHA at the tip of the branch.
    pub commit_sha: String,
    /// Whether the branch is protected by branch protection rules.
    pub protected: bool,
}

impl GitHubBranch {
    /// Constructs a new `GitHubBranch` descriptor.
    #[must_use]
    pub fn new(name: impl Into<String>, commit_sha: impl Into<String>, protected: bool) -> Self {
        Self {
            name: name.into(),
            commit_sha: commit_sha.into(),
            protected,
        }
    }
}

/// Information representing a Git tag discovered from the GitHub REST API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubTag {
    /// Release tag name (e.g. `v1.0.0`, `v0.9.2-beta.1`).
    pub name: String,
    /// 40-character hexadecimal commit SHA tagged by this ref.
    pub commit_sha: String,
    /// Download URL for the zipball archive of this tag, if available.
    pub zipball_url: Option<String>,
    /// Download URL for the tarball archive of this tag, if available.
    pub tarball_url: Option<String>,
}

impl GitHubTag {
    /// Constructs a new `GitHubTag` descriptor.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        commit_sha: impl Into<String>,
        zipball_url: Option<String>,
        tarball_url: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            commit_sha: commit_sha.into(),
            zipball_url,
            tarball_url,
        }
    }
}

/// Comprehensive report aggregating discovered branches, tags, and rate limit status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefDiscoveryReport {
    /// Repository owner username or organization.
    pub owner: String,
    /// Repository project name.
    pub repo: String,
    /// Default branch name (e.g. `main` or `master`), if identified.
    pub default_branch: Option<String>,
    /// All discovered Git branches in the repository.
    pub branches: Vec<GitHubBranch>,
    /// All discovered release tags in the repository.
    pub tags: Vec<GitHubTag>,
    /// GitHub API rate limiting metrics at time of discovery.
    pub rate_limit: Option<GitHubRateLimit>,
    /// UTC timestamp when ref discovery was performed.
    pub discovered_at: DateTime<Utc>,
}

impl RefDiscoveryReport {
    /// Finds a branch by exact name match.
    #[must_use]
    pub fn find_branch(&self, name: &str) -> Option<&GitHubBranch> {
        self.branches.iter().find(|b| b.name == name)
    }

    /// Finds a release tag by exact name match.
    #[must_use]
    pub fn find_tag(&self, name: &str) -> Option<&GitHubTag> {
        self.tags.iter().find(|t| t.name == name)
    }

    /// Resolves the commit SHA for a given reference name (branch, tag, or direct SHA).
    #[must_use]
    pub fn find_commit_sha_for_ref<'a>(&'a self, ref_name: &'a str) -> Option<&'a str> {
        if let Some(b) = self.find_branch(ref_name) {
            return Some(&b.commit_sha);
        }
        if let Some(t) = self.find_tag(ref_name) {
            return Some(&t.commit_sha);
        }
        // If ref is already a 40-character hex commit SHA
        if ref_name.len() == 40 && ref_name.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(ref_name);
        }
        None
    }

    /// Disambiguates a slash-separated URL path tail against known branches and tags.
    ///
    /// For example, given `feature/new-pipeline/crates/core`, this function checks
    /// if `feature/new-pipeline` matches an existing branch. If so, it returns
    /// `Some(("feature/new-pipeline".to_string(), Some("crates/core".to_string())))`.
    ///
    /// Uses longest-prefix matching to prefer the most specific matching branch/tag.
    #[must_use]
    pub fn disambiguate_ref_and_subpath(&self, raw_tail: &str) -> Option<(String, Option<String>)> {
        let clean = raw_tail.trim_matches('/');
        if clean.is_empty() {
            return None;
        }

        let segments: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return None;
        }

        // Try longest candidate ref prefix first (from segments.len() down to 1)
        for prefix_len in (1..=segments.len()).rev() {
            let candidate_ref = segments[..prefix_len].join("/");
            let is_match = self.branches.iter().any(|b| b.name == candidate_ref)
                || self.tags.iter().any(|t| t.name == candidate_ref);

            if is_match {
                let subpath = if prefix_len < segments.len() {
                    Some(segments[prefix_len..].join("/"))
                } else {
                    None
                };
                return Some((candidate_ref, subpath));
            }
        }

        // Fallback: first segment as ref, remainder as subpath
        let ref_name = segments[0].to_string();
        let subpath = if segments.len() > 1 {
            Some(segments[1..].join("/"))
        } else {
            None
        };
        Some((ref_name, subpath))
    }
}
