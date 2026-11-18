//! # GitHub Repository Metadata, Language Breakdown & Context Consolidation
//!
//! Provides structured models and converters for GitHub repository information
//! (`GET /repos/{owner}/{repo}`) and language statistics (`GET /repos/{owner}/{repo}/languages`),
//! integrating with the Canonical Context Document Model (`DocumentMetadata`).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use web_repo_to_ai_cdm::DocumentMetadata;

use crate::commit::PinnedRevision;
use crate::target::GitHubTarget;

/// Software license metadata declared for a GitHub repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubLicense {
    /// Machine-readable license key (e.g. `mit`, `apache-2.0`).
    pub key: String,
    /// Human-readable license title (e.g. `MIT License`).
    pub name: String,
    /// Canonical SPDX license identifier (e.g. `MIT`, `Apache-2.0`, `GPL-3.0`), if known.
    pub spdx_id: Option<String>,
    /// GitHub API or authoritative URL for license details.
    pub url: Option<String>,
}

impl GitHubLicense {
    /// Constructs a new `GitHubLicense`.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        name: impl Into<String>,
        spdx_id: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            spdx_id,
            url,
        }
    }
}

/// Programming language breakdown with byte volumes and percentage calculations.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GitHubLanguages {
    /// Map of language name to byte count detected in the repository.
    pub breakdown: BTreeMap<String, u64>,
}

impl GitHubLanguages {
    /// Constructs a new `GitHubLanguages` instance from a byte distribution map.
    #[must_use]
    pub fn new(breakdown: BTreeMap<String, u64>) -> Self {
        Self { breakdown }
    }

    /// Returns the total volume in bytes across all detected languages.
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.breakdown.values().copied().sum()
    }

    /// Returns the fractional percentage (0.0 to 100.0) for a given language, or None if absent.
    #[must_use]
    pub fn percentage(&self, language: &str) -> Option<f64> {
        let total = self.total_bytes();
        if total == 0 {
            return None;
        }

        self.breakdown
            .get(language)
            .map(|&bytes| (bytes as f64 / total as f64) * 100.0)
    }

    /// Returns a list of (language, percentage) pairs sorted in descending order of byte volume.
    #[must_use]
    pub fn percentages(&self) -> Vec<(String, f64)> {
        let total = self.total_bytes();
        if total == 0 {
            return Vec::new();
        }

        let mut items: Vec<(String, u64)> = self
            .breakdown
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();

        // Sort descending by bytes, ascending by name on tie
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        items
            .into_iter()
            .map(|(lang, bytes)| {
                let pct = (bytes as f64 / total as f64) * 100.0;
                (lang, pct)
            })
            .collect()
    }

    /// Returns the primary language (language with highest byte volume), if any.
    #[must_use]
    pub fn primary_language(&self) -> Option<&str> {
        self.breakdown
            .iter()
            .max_by_key(|entry| entry.1)
            .map(|(k, _)| k.as_str())
    }
}

/// Comprehensive repository metadata extracted from GitHub's Repository REST API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRepositoryMetadata {
    /// GitHub internal repository numeric ID.
    pub id: u64,
    /// GraphQL Node ID.
    pub node_id: Option<String>,
    /// Repository short name (e.g. `web-repo-to-ai`).
    pub name: String,
    /// Fully qualified owner and name (`owner/repo`).
    pub full_name: String,
    /// Owner login username or organization.
    pub owner: String,
    /// Repository description.
    pub description: Option<String>,
    /// Whether this repository is a fork.
    pub fork: bool,
    /// Whether this repository is archived.
    pub archived: bool,
    /// Whether this repository is disabled.
    pub disabled: bool,
    /// Repository visibility (`public`, `private`, `internal`).
    pub visibility: String,
    /// Configured default branch (e.g. `main` or `master`).
    pub default_branch: String,
    /// Project homepage or documentation URL, if set.
    pub homepage: Option<String>,
    /// Primary language recognized by GitHub's Linguist.
    pub primary_language: Option<String>,
    /// Repository topics / keywords.
    pub topics: Vec<String>,
    /// Declared repository license, if present.
    pub license: Option<GitHubLicense>,
    /// Star count (`stargazers_count`).
    pub stargazers_count: u32,
    /// Watcher count (`watchers_count`).
    pub watchers_count: u32,
    /// Fork count (`forks_count`).
    pub forks_count: u32,
    /// Open issues count (`open_issues_count`).
    pub open_issues_count: u32,
    /// Repository size in kilobytes.
    pub size_kb: u64,
    /// Creation timestamp.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp.
    pub updated_at: Option<DateTime<Utc>>,
    /// Last push timestamp.
    pub pushed_at: Option<DateTime<Utc>>,
}

impl GitHubRepositoryMetadata {
    /// Returns a one-line summary string of the repository.
    #[must_use]
    pub fn summary(&self) -> String {
        let desc = self.description.as_deref().unwrap_or("No description");
        let lang = self
            .primary_language
            .as_deref()
            .unwrap_or("Unknown language");
        let license = self
            .license
            .as_ref()
            .and_then(|l| l.spdx_id.as_deref())
            .unwrap_or("No license");

        format!(
            "{} ({}) | {} | {} | {} stars",
            self.full_name, lang, license, desc, self.stargazers_count
        )
    }

    /// Converts this repository metadata and language breakdown into CDM `DocumentMetadata`.
    #[must_use]
    pub fn to_cdm_metadata(&self, languages: &GitHubLanguages) -> DocumentMetadata {
        let license_str = self
            .license
            .as_ref()
            .and_then(|l| l.spdx_id.clone().or_else(|| Some(l.name.clone())));

        let mut extra = BTreeMap::new();
        extra.insert("github.repo_id".to_string(), self.id.to_string());
        extra.insert(
            "github.default_branch".to_string(),
            self.default_branch.clone(),
        );
        extra.insert("github.visibility".to_string(), self.visibility.clone());
        extra.insert("github.is_fork".to_string(), self.fork.to_string());
        extra.insert("github.is_archived".to_string(), self.archived.to_string());
        extra.insert("github.is_disabled".to_string(), self.disabled.to_string());
        extra.insert(
            "github.stargazers_count".to_string(),
            self.stargazers_count.to_string(),
        );
        extra.insert(
            "github.watchers_count".to_string(),
            self.watchers_count.to_string(),
        );
        extra.insert(
            "github.forks_count".to_string(),
            self.forks_count.to_string(),
        );
        extra.insert(
            "github.open_issues_count".to_string(),
            self.open_issues_count.to_string(),
        );
        extra.insert("github.size_kb".to_string(), self.size_kb.to_string());

        if let Some(homepage) = &self.homepage {
            if !homepage.trim().is_empty() {
                extra.insert("github.homepage".to_string(), homepage.clone());
            }
        }

        DocumentMetadata {
            title: Some(self.full_name.clone()),
            description: self.description.clone(),
            authors: vec![self.owner.clone()],
            license: license_str,
            primary_language: self.primary_language.clone(),
            language_breakdown: languages.breakdown.clone(),
            published_at: self.created_at,
            modified_at: self.pushed_at.or(self.updated_at),
            tags: self.topics.clone(),
            extra,
        }
    }
}

/// Consolidated context binding a repository target, metadata, language distribution,
/// and optional pinned revision into a unified extraction container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRepoContext {
    /// The original or disambiguated repository target.
    pub target: GitHubTarget,
    /// Detailed repository metadata.
    pub metadata: GitHubRepositoryMetadata,
    /// Programming language distribution.
    pub languages: GitHubLanguages,
    /// Pinned commit revision, if resolved.
    pub pinned_revision: Option<PinnedRevision>,
    /// UTC timestamp when this context was retrieved.
    pub fetched_at: DateTime<Utc>,
}

impl GitHubRepoContext {
    /// Constructs a new `GitHubRepoContext`.
    #[must_use]
    pub fn new(
        target: GitHubTarget,
        metadata: GitHubRepositoryMetadata,
        languages: GitHubLanguages,
        pinned_revision: Option<PinnedRevision>,
    ) -> Self {
        Self {
            target,
            metadata,
            languages,
            pinned_revision,
            fetched_at: Utc::now(),
        }
    }

    /// Returns a reference to the repository target.
    #[must_use]
    pub const fn target(&self) -> &GitHubTarget {
        &self.target
    }

    /// Returns a reference to the repository metadata.
    #[must_use]
    pub const fn metadata(&self) -> &GitHubRepositoryMetadata {
        &self.metadata
    }

    /// Returns a reference to the language breakdown.
    #[must_use]
    pub const fn languages(&self) -> &GitHubLanguages {
        &self.languages
    }

    /// Returns an optional reference to the pinned revision.
    #[must_use]
    pub const fn pinned_revision(&self) -> Option<&PinnedRevision> {
        self.pinned_revision.as_ref()
    }

    /// Converts this repository context into standard CDM `DocumentMetadata`.
    #[must_use]
    pub fn to_cdm_metadata(&self) -> DocumentMetadata {
        let mut cdm_meta = self.metadata.to_cdm_metadata(&self.languages);

        if let Some(pinned) = &self.pinned_revision {
            cdm_meta
                .extra
                .insert("git.commit_sha".to_string(), pinned.commit_sha.clone());
            cdm_meta.extra.insert(
                "git.root_tree_sha".to_string(),
                pinned.root_tree_sha.clone(),
            );
            if let Some(r) = &pinned.effective_ref {
                cdm_meta
                    .extra
                    .insert("git.effective_ref".to_string(), r.clone());
            }
        }

        cdm_meta
    }

    /// Formats a multi-line markdown summary card suitable for documentation frontmatter.
    #[must_use]
    pub fn summary_card(&self) -> String {
        let desc = self
            .metadata
            .description
            .as_deref()
            .unwrap_or("No description provided.");
        let lang = self
            .metadata
            .primary_language
            .as_deref()
            .unwrap_or("Unknown");
        let license = self
            .metadata
            .license
            .as_ref()
            .and_then(|l| l.spdx_id.as_deref())
            .unwrap_or("None");

        let mut lines = Vec::new();
        lines.push(format!("# {}", self.metadata.full_name));
        lines.push(desc.to_string());
        lines.push(String::new());
        lines.push(format!("- **Primary Language:** {lang}"));
        lines.push(format!("- **License:** {license}"));
        lines.push(format!(
            "- **Default Branch:** {}",
            self.metadata.default_branch
        ));
        lines.push(format!(
            "- **Stats:** {} stars | {} forks | {} open issues",
            self.metadata.stargazers_count,
            self.metadata.forks_count,
            self.metadata.open_issues_count
        ));

        if let Some(pinned) = &self.pinned_revision {
            lines.push(format!(
                "- **Pinned Revision:** `{}` (Tree: `{}`)",
                pinned.short_sha(),
                &pinned.root_tree_sha[..7.min(pinned.root_tree_sha.len())]
            ));
        }

        lines.join("\n")
    }
}
