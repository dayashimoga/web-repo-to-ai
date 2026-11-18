//! # GitHub Extractor (`web-repo-to-ai-github-extractor`)
//!
//! Provides deep repository ingestion for public and private GitHub sources:
//! - GitHub URL parsing, target decomposition, and validation (`GitHubTarget`, `GitHubUrlParser`)
//! - Branch and release tag discovery with pagination and rate limit tracking (`GitHubApiClient`, `RefDiscoveryReport`)
//! - Git Trees API traversal with recursive subtree recovery
//! - Completeness manifest accounting (Discovered = Extracted + Excluded + Failed)
//! - Ephemeral GitHub App installation token lifecycle management

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod commit;
pub mod error;
pub mod metadata;
pub mod models;
pub mod target;

pub use client::{
    GitHubApiClient, DEFAULT_GITHUB_TIMEOUT, DEFAULT_MAX_PAGES, DEFAULT_PER_PAGE,
    DEFAULT_USER_AGENT, GITHUB_API_VERSION_HEADER,
};
pub use commit::{CommitAuthor, CommitSignatureVerification, GitHubCommitMetadata, PinnedRevision};
pub use error::{redact_secrets, GitHubExtractorError};
pub use metadata::{GitHubLanguages, GitHubLicense, GitHubRepoContext, GitHubRepositoryMetadata};
pub use models::{GitHubBranch, GitHubRateLimit, GitHubTag, RefDiscoveryReport};
pub use target::{
    is_default_host, is_reserved_github_path, validate_and_sanitize_subpath, validate_owner,
    validate_repo, GitHubTarget, GitHubTargetType, GitHubUrlParser, DEFAULT_GITHUB_API_BASE,
    DEFAULT_GITHUB_HOST, DEFAULT_GITHUB_WWW_HOST, DEFAULT_RAW_GITHUB_USERCONTENT_BASE,
};

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_extractor_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-github-extractor");
    }
}
