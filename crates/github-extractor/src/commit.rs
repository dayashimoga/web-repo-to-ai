//! # GitHub Commit Metadata and Immutable Revision Pinning
//!
//! Provides commit metadata extraction, author/committer information, parent SHAs,
//! root Git tree resolution, and the immutable `PinnedRevision` container guaranteeing
//! zero ref-drift during downstream repository ingestion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use web_repo_to_ai_cdm::{RevisionInfo, SourceIdentity};

use crate::target::{GitHubTarget, GitHubTargetType};

/// Git commit author or committer identity and timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAuthor {
    /// Contributor display name.
    pub name: String,
    /// Contributor email address.
    pub email: String,
    /// Commit authoring or committing timestamp.
    pub date: DateTime<Utc>,
}

impl CommitAuthor {
    /// Constructs a new `CommitAuthor` record.
    #[must_use]
    pub fn new(name: impl Into<String>, email: impl Into<String>, date: DateTime<Utc>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            date,
        }
    }
}

/// Cryptographic GPG or SSH commit signature verification details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSignatureVerification {
    /// Whether the commit signature was cryptographically verified.
    pub verified: bool,
    /// Verification reason (e.g. `valid`, `unsigned`, `bad_signature`).
    pub reason: String,
    /// Raw ASCII-armored signature string, if present.
    pub signature: Option<String>,
    /// Signed payload content, if present.
    pub payload: Option<String>,
}

/// Comprehensive commit metadata extracted from GitHub's Commit REST API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubCommitMetadata {
    /// Full 40-character hexadecimal commit SHA.
    pub sha: String,
    /// GraphQL Node ID for the commit.
    pub node_id: Option<String>,
    /// Full commit message.
    pub message: String,
    /// Commit author identity and authoring timestamp.
    pub author: Option<CommitAuthor>,
    /// Committer identity and commit timestamp.
    pub committer: Option<CommitAuthor>,
    /// 40-character Git root tree SHA associated with this commit revision.
    pub tree_sha: String,
    /// Parent commit SHAs.
    pub parents: Vec<String>,
    /// Number of comments on this commit.
    pub comment_count: u32,
    /// Cryptographic signature verification status, if available.
    pub verification: Option<CommitSignatureVerification>,
}

impl GitHubCommitMetadata {
    /// Returns the short 7-character commit SHA prefix.
    #[must_use]
    pub fn short_sha(&self) -> &str {
        if self.sha.len() >= 7 {
            &self.sha[..7]
        } else {
            &self.sha
        }
    }

    /// Returns the first line (subject) of the commit message.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.message.lines().next().unwrap_or("").trim()
    }

    /// Returns the primary commit date (committer date preferred, author date fallback).
    #[must_use]
    pub fn commit_date(&self) -> Option<DateTime<Utc>> {
        self.committer
            .as_ref()
            .map(|c| c.date)
            .or_else(|| self.author.as_ref().map(|a| a.date))
    }
}

/// Immutable, pinned Git revision binding a repository target to an immutable
/// 40-character commit SHA and root tree SHA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedRevision {
    /// Original or refined repository target.
    pub target: GitHubTarget,
    /// Resolved immutable 40-character hexadecimal commit SHA.
    pub commit_sha: String,
    /// Resolved 40-character Git root tree SHA for tree traversal.
    pub root_tree_sha: String,
    /// Detailed commit metadata.
    pub metadata: GitHubCommitMetadata,
    /// The effective branch, tag, or ref name requested, if any.
    pub effective_ref: Option<String>,
    /// UTC timestamp when this revision was resolved and pinned.
    pub pinned_at: DateTime<Utc>,
}

impl PinnedRevision {
    /// Constructs a new `PinnedRevision`.
    #[must_use]
    pub fn new(
        target: GitHubTarget,
        commit_sha: impl Into<String>,
        root_tree_sha: impl Into<String>,
        metadata: GitHubCommitMetadata,
        effective_ref: Option<String>,
    ) -> Self {
        Self {
            target,
            commit_sha: commit_sha.into(),
            root_tree_sha: root_tree_sha.into(),
            metadata,
            effective_ref,
            pinned_at: Utc::now(),
        }
    }

    /// Returns the repository target.
    #[must_use]
    pub const fn target(&self) -> &GitHubTarget {
        &self.target
    }

    /// Returns the full 40-character immutable commit SHA.
    #[must_use]
    pub fn commit_sha(&self) -> &str {
        &self.commit_sha
    }

    /// Returns the short 7-character commit SHA prefix.
    #[must_use]
    pub fn short_sha(&self) -> &str {
        if self.commit_sha.len() >= 7 {
            &self.commit_sha[..7]
        } else {
            &self.commit_sha
        }
    }

    /// Returns the Git root tree SHA required for Git Trees API traversal.
    #[must_use]
    pub fn root_tree_sha(&self) -> &str {
        &self.root_tree_sha
    }

    /// Returns the detailed commit metadata.
    #[must_use]
    pub const fn metadata(&self) -> &GitHubCommitMetadata {
        &self.metadata
    }

    /// Returns the original effective reference name (branch, tag, or ref), if any.
    #[must_use]
    pub fn effective_ref(&self) -> Option<&str> {
        self.effective_ref.as_deref()
    }

    /// Returns whether this pinned revision matches the specified commit SHA or prefix.
    #[must_use]
    pub fn matches_sha(&self, query_sha: &str) -> bool {
        self.commit_sha.starts_with(query_sha)
    }

    /// Converts this pinned revision into a CDM `RevisionInfo` record.
    #[must_use]
    pub fn to_revision_info(&self) -> RevisionInfo {
        let (git_branch, git_tag) = match &self.target.target_type {
            GitHubTargetType::Tag { tag_name } => (None, Some(tag_name.clone())),
            GitHubTargetType::Tree { ref_name, .. } | GitHubTargetType::Blob { ref_name, .. } => {
                (Some(ref_name.clone()), None)
            }
            _ => {
                if let Some(r) = &self.effective_ref {
                    if r.starts_with('v') || r.contains('.') {
                        (None, Some(r.clone()))
                    } else {
                        (Some(r.clone()), None)
                    }
                } else {
                    (None, None)
                }
            }
        };

        RevisionInfo {
            git_commit: Some(self.commit_sha.clone()),
            git_branch,
            git_tag,
            http_etag: None,
            http_last_modified: self.metadata.commit_date().map(|d| d.to_rfc2822()),
            content_hash_blake3: None,
            content_hash_sha256: None,
        }
    }

    /// Maps this pinned revision into a CDM `SourceIdentity`.
    #[must_use]
    pub fn to_source_identity(&self, raw_input: &str) -> SourceIdentity {
        let mut id = self.target.to_source_identity(raw_input);
        // Ensure git_ref uses the effective ref or falls back to the pinned commit SHA
        id.git_ref = self
            .effective_ref
            .clone()
            .or_else(|| Some(self.commit_sha.clone()));
        id
    }
}
