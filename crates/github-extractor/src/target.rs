//! # GitHub Repository Target and URL Decomposition
//!
//! Provides parsing, decomposition, validation, and canonicalization of GitHub
//! URLs across repository roots, branches, tags, commit SHAs, directory trees,
//! and single-file blobs.

use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use web_repo_to_ai_cdm::SourceIdentity;
use web_repo_to_ai_url_security::{UrlNormalizer, UrlSecurityPolicy};

use crate::error::GitHubExtractorError;

/// Default public GitHub hostname.
pub const DEFAULT_GITHUB_HOST: &str = "github.com";
/// Default public GitHub WWW hostname.
pub const DEFAULT_GITHUB_WWW_HOST: &str = "www.github.com";
/// Default public GitHub API base URL.
pub const DEFAULT_GITHUB_API_BASE: &str = "https://api.github.com";
/// Default raw GitHub user content base URL.
pub const DEFAULT_RAW_GITHUB_USERCONTENT_BASE: &str = "https://raw.githubusercontent.com";

static REPO_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._-]{1,100}$").expect("static regex must compile"));

static COMMIT_SHA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-fA-F]{4,40}$").expect("static regex must compile"));

static RESERVED_GITHUB_NAMES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert("about");
    s.insert("account");
    s.insert("collections");
    s.insert("contact");
    s.insert("copilot");
    s.insert("codespaces");
    s.insert("customer-stories");
    s.insert("dashboard");
    s.insert("discussions");
    s.insert("enterprise");
    s.insert("events");
    s.insert("explore");
    s.insert("features");
    s.insert("issues");
    s.insert("join");
    s.insert("login");
    s.insert("logout");
    s.insert("marketplace");
    s.insert("new");
    s.insert("notifications");
    s.insert("organizations");
    s.insert("orgs");
    s.insert("pricing");
    s.insert("pulls");
    s.insert("readme");
    s.insert("resources");
    s.insert("search");
    s.insert("security");
    s.insert("session");
    s.insert("settings");
    s.insert("signup");
    s.insert("site");
    s.insert("sponsors");
    s.insert("status");
    s.insert("team");
    s.insert("topics");
    s.insert("trending");
    s
});

/// Classification of the target entity pointed to by the GitHub URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitHubTargetType {
    /// Full repository root at the default or specified branch.
    RepositoryRoot,
    /// Directory subtree within the repository.
    Tree {
        /// Git reference name (branch, tag, or commit).
        ref_name: String,
        /// Optional relative directory path inside the repository.
        subpath: Option<String>,
    },
    /// Single file blob inside the repository.
    Blob {
        /// Git reference name (branch, tag, or commit).
        ref_name: String,
        /// Relative path to the targeted file.
        path: String,
    },
    /// Specific immutable commit revision.
    Commit {
        /// 4 to 40 character hexadecimal commit SHA.
        commit_sha: String,
    },
    /// Specific release tag reference.
    Tag {
        /// Release tag name.
        tag_name: String,
    },
    /// Commits listing view for a branch or ref.
    Commits {
        /// Branch or ref name, if specified.
        ref_name: Option<String>,
    },
    /// Pull request view.
    PullRequest {
        /// PR sequence number.
        number: u64,
    },
    /// Issue view.
    Issue {
        /// Issue sequence number.
        number: u64,
    },
    /// Releases view.
    Releases,
}

/// Decomposed, validated, and normalized GitHub repository target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubTarget {
    /// Validated repository owner username or organization.
    pub owner: String,
    /// Validated repository project name (without `.git` suffix).
    pub repo: String,
    /// Detailed classification of the targeted entity.
    pub target_type: GitHubTargetType,
    /// Extracted Git reference (branch, tag, or commit SHA), if specified.
    pub ref_spec: Option<String>,
    /// Relative subpath filter within the repository, if targeted.
    pub subpath: Option<String>,
    /// Hostname of the GitHub server (e.g. `github.com` or custom enterprise host).
    pub host: String,
    /// Whether this target is hosted on a custom GitHub Enterprise Server.
    pub is_enterprise: bool,
}

impl GitHubTarget {
    /// Constructs a root repository target for the given owner and repo on default GitHub host.
    #[must_use]
    pub fn for_repo(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            target_type: GitHubTargetType::RepositoryRoot,
            ref_spec: None,
            subpath: None,
            host: DEFAULT_GITHUB_HOST.to_string(),
            is_enterprise: false,
        }
    }

    /// Returns the repository owner username or organization.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns the repository project name.
    #[must_use]
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Returns the repository target classification.
    #[must_use]
    pub const fn target_type(&self) -> &GitHubTargetType {
        &self.target_type
    }

    /// Returns the targeted git reference (branch, tag, or commit SHA), if any.
    #[must_use]
    pub fn ref_spec(&self) -> Option<&str> {
        self.ref_spec.as_deref()
    }

    /// Returns the targeted subpath within the repository, if any.
    #[must_use]
    pub fn subpath(&self) -> Option<&str> {
        self.subpath.as_deref()
    }

    /// Returns the GitHub hostname.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns whether this target points to the default public `github.com`.
    #[must_use]
    pub fn is_default_host(&self) -> bool {
        self.host == DEFAULT_GITHUB_HOST || self.host == DEFAULT_GITHUB_WWW_HOST
    }

    /// Returns whether the target is the full repository root.
    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self.target_type, GitHubTargetType::RepositoryRoot)
    }

    /// Returns whether the target is a directory subtree.
    #[must_use]
    pub const fn is_tree(&self) -> bool {
        matches!(self.target_type, GitHubTargetType::Tree { .. })
    }

    /// Returns whether the target is a single file blob.
    #[must_use]
    pub const fn is_blob(&self) -> bool {
        matches!(self.target_type, GitHubTargetType::Blob { .. })
    }

    /// Returns whether the target is a specific commit.
    #[must_use]
    pub const fn is_commit(&self) -> bool {
        matches!(self.target_type, GitHubTargetType::Commit { .. })
    }

    /// Returns the Name-with-Owner string (`owner/repo`).
    #[must_use]
    pub fn nwo(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// Constructs the canonical repository root URL (`https://{host}/{owner}/{repo}`).
    #[must_use]
    pub fn canonical_repo_url(&self) -> String {
        let canonical_host = if self.is_default_host() {
            DEFAULT_GITHUB_HOST
        } else {
            &self.host
        };
        format!("https://{canonical_host}/{}", self.nwo())
    }

    /// Reconstructs the exact canonical HTTPS target URL.
    #[must_use]
    pub fn canonical_target_url(&self) -> String {
        let host = if self.is_default_host() {
            DEFAULT_GITHUB_HOST
        } else {
            &self.host
        };
        match &self.target_type {
            GitHubTargetType::RepositoryRoot => {
                if let Some(r) = &self.ref_spec {
                    format!("https://{host}/{}/tree/{r}", self.nwo())
                } else {
                    self.canonical_repo_url()
                }
            }
            GitHubTargetType::Tree { ref_name, subpath } => {
                if let Some(p) = subpath {
                    format!("https://{host}/{}/tree/{ref_name}/{p}", self.nwo())
                } else {
                    format!("https://{host}/{}/tree/{ref_name}", self.nwo())
                }
            }
            GitHubTargetType::Blob { ref_name, path } => {
                format!("https://{host}/{}/blob/{ref_name}/{path}", self.nwo())
            }
            GitHubTargetType::Commit { commit_sha } => {
                format!("https://{host}/{}/commit/{commit_sha}", self.nwo())
            }
            GitHubTargetType::Tag { tag_name } => {
                format!("https://{host}/{}/releases/tag/{tag_name}", self.nwo())
            }
            GitHubTargetType::Commits { ref_name } => {
                if let Some(r) = ref_name {
                    format!("https://{host}/{}/commits/{r}", self.nwo())
                } else {
                    format!("https://{host}/{}/commits", self.nwo())
                }
            }
            GitHubTargetType::PullRequest { number } => {
                format!("https://{host}/{}/pull/{number}", self.nwo())
            }
            GitHubTargetType::Issue { number } => {
                format!("https://{host}/{}/issues/{number}", self.nwo())
            }
            GitHubTargetType::Releases => {
                format!("https://{host}/{}/releases", self.nwo())
            }
        }
    }

    /// Constructs the HTTPS Git clone URL (`https://{host}/{owner}/{repo}.git`).
    #[must_use]
    pub fn clone_url_https(&self) -> String {
        let host = if self.is_default_host() {
            DEFAULT_GITHUB_HOST
        } else {
            &self.host
        };
        format!("https://{host}/{}.git", self.nwo())
    }

    /// Constructs the SSH Git clone URL (`git@{host}:{owner}/{repo}.git`).
    #[must_use]
    pub fn clone_url_ssh(&self) -> String {
        let host = if self.is_default_host() {
            DEFAULT_GITHUB_HOST
        } else {
            &self.host
        };
        format!("git@{host}:{}.git", self.nwo())
    }

    /// Constructs the GitHub API base endpoint.
    #[must_use]
    pub fn api_base_url(&self) -> String {
        if self.is_default_host() {
            DEFAULT_GITHUB_API_BASE.to_string()
        } else {
            format!("https://{}/api/v3", self.host)
        }
    }

    /// Constructs the GitHub repository REST API endpoint (`.../repos/{owner}/{repo}`).
    #[must_use]
    pub fn api_repo_url(&self) -> String {
        format!("{}/repos/{}", self.api_base_url(), self.nwo())
    }

    /// Constructs the GitHub Git Trees API endpoint.
    #[must_use]
    pub fn api_trees_url(&self, tree_sha: &str, recursive: bool) -> String {
        let base = format!("{}/git/trees/{tree_sha}", self.api_repo_url());
        if recursive {
            format!("{base}?recursive=1")
        } else {
            base
        }
    }

    /// Constructs the GitHub Git Blobs API endpoint.
    #[must_use]
    pub fn api_blob_url(&self, file_sha: &str) -> String {
        format!("{}/git/blobs/{file_sha}", self.api_repo_url())
    }

    /// Constructs the GitHub Commits REST API endpoint.
    #[must_use]
    pub fn api_commits_url(&self, ref_param: Option<&str>) -> String {
        let base = format!("{}/commits", self.api_repo_url());
        if let Some(r) = ref_param {
            format!("{base}?sha={r}")
        } else {
            base
        }
    }

    /// Constructs the GitHub Contents REST API endpoint for a file or directory.
    #[must_use]
    pub fn api_contents_url(&self, path: Option<&str>, ref_param: Option<&str>) -> String {
        let path_part = path.unwrap_or("").trim_matches('/');
        let base = if path_part.is_empty() {
            format!("{}/contents", self.api_repo_url())
        } else {
            format!("{}/contents/{path_part}", self.api_repo_url())
        };

        if let Some(r) = ref_param {
            format!("{base}?ref={r}")
        } else {
            base
        }
    }

    /// Constructs the raw content download URL for a file.
    #[must_use]
    pub fn raw_blob_url(&self, ref_name: &str, file_path: &str) -> String {
        let clean_path = file_path.trim_matches('/');
        if self.is_default_host() {
            format!(
                "{}/{}/{}/{}/{}",
                DEFAULT_RAW_GITHUB_USERCONTENT_BASE, self.owner, self.repo, ref_name, clean_path
            )
        } else {
            format!(
                "https://{}/raw/{}/{}/{}/{}",
                self.host, self.owner, self.repo, ref_name, clean_path
            )
        }
    }

    /// Maps this decomposed target into a CDM `SourceIdentity`.
    #[must_use]
    pub fn to_source_identity(&self, raw_input: &str) -> SourceIdentity {
        SourceIdentity {
            raw_input: raw_input.to_string(),
            canonical_url: self.canonical_target_url(),
            domain: Some(self.host.clone()),
            repo_owner: Some(self.owner.clone()),
            repo_name: Some(self.repo.clone()),
            git_ref: self.ref_spec.clone(),
            subpath: self.subpath.clone(),
        }
    }

    /// Returns a new `GitHubTarget` with refined ref and subpath (e.g. after resolving
    /// ambiguous branch names containing slashes against remote refs).
    #[must_use]
    pub fn with_refined_ref_and_subpath(
        &self,
        ref_name: impl Into<String>,
        subpath: Option<String>,
    ) -> Self {
        let ref_str = ref_name.into();
        let target_type = match &self.target_type {
            GitHubTargetType::Tree { .. } => GitHubTargetType::Tree {
                ref_name: ref_str.clone(),
                subpath: subpath.clone(),
            },
            GitHubTargetType::Blob { .. } => GitHubTargetType::Blob {
                ref_name: ref_str.clone(),
                path: subpath.clone().unwrap_or_default(),
            },
            other => other.clone(),
        };

        Self {
            owner: self.owner.clone(),
            repo: self.repo.clone(),
            target_type,
            ref_spec: Some(ref_str),
            subpath,
            host: self.host.clone(),
            is_enterprise: self.is_enterprise,
        }
    }
}

impl fmt::Display for GitHubTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_target_url())
    }
}

impl FromStr for GitHubTarget {
    type Err = GitHubExtractorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GitHubUrlParser::parse_url(s)
    }
}

/// Configurable parser and validator for GitHub repository URLs and identifiers.
#[derive(Debug, Clone)]
pub struct GitHubUrlParser {
    allowed_hosts: HashSet<String>,
    allow_ssh: bool,
    allow_git_transport: bool,
    allow_shorthand: bool,
    enforce_ssrf_protection: bool,
}

impl Default for GitHubUrlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubUrlParser {
    /// Creates a new parser configured with default public GitHub settings.
    #[must_use]
    pub fn new() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(DEFAULT_GITHUB_HOST.to_string());
        allowed.insert(DEFAULT_GITHUB_WWW_HOST.to_string());

        Self {
            allowed_hosts: allowed,
            allow_ssh: true,
            allow_git_transport: true,
            allow_shorthand: true,
            enforce_ssrf_protection: true,
        }
    }

    /// Adds an allowed enterprise or custom GitHub host (e.g. `github.mycompany.com`).
    #[must_use]
    pub fn with_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.insert(host.into().to_ascii_lowercase());
        self
    }

    /// Configures whether SSH Git URLs (`git@github.com:...`) are allowed.
    #[must_use]
    pub const fn with_ssh(mut self, allow: bool) -> Self {
        self.allow_ssh = allow;
        self
    }

    /// Configures whether `git://` transport URLs are allowed.
    #[must_use]
    pub const fn with_git_transport(mut self, allow: bool) -> Self {
        self.allow_git_transport = allow;
        self
    }

    /// Configures whether shorthand strings (`owner/repo` or `owner/repo@ref`) are allowed.
    #[must_use]
    pub const fn with_shorthand(mut self, allow: bool) -> Self {
        self.allow_shorthand = allow;
        self
    }

    /// Configures whether zero-trust SSRF protections are enforced.
    #[must_use]
    pub const fn with_ssrf_protection(mut self, enforce: bool) -> Self {
        self.enforce_ssrf_protection = enforce;
        self
    }

    /// Static convenience method to parse a GitHub URL with default configuration.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the URL is invalid, malicious, or unparseable.
    pub fn parse_url(input: &str) -> Result<GitHubTarget, GitHubExtractorError> {
        Self::new().parse(input)
    }

    /// Parses and decomposes a raw input string into a validated `GitHubTarget`.
    ///
    /// # Errors
    /// Returns `GitHubExtractorError` if the input fails syntax, security, or naming checks.
    pub fn parse(&self, raw_input: &str) -> Result<GitHubTarget, GitHubExtractorError> {
        let trimmed = raw_input.trim();
        if trimmed.is_empty() {
            return Err(GitHubExtractorError::InvalidUrl(
                "Input URL or repository identifier cannot be empty".to_string(),
            ));
        }

        if trimmed.len() > 4096 {
            return Err(GitHubExtractorError::InvalidUrl(
                "Input URL exceeds maximum permitted length of 4096 characters".to_string(),
            ));
        }

        // Detect path traversal attempts early before WHATWG URL resolution normalizes them
        if trimmed.contains("/../")
            || trimmed.ends_with("/..")
            || trimmed.contains("\\..\\")
            || trimmed.ends_with("\\..")
            || trimmed.contains("%2e%2e")
            || trimmed.contains("%2E%2E")
            || trimmed.contains("..%2f")
            || trimmed.contains("..%2F")
        {
            return Err(GitHubExtractorError::InvalidSubpath {
                path: trimmed.to_string(),
                reason: "Directory traversal ('..') is strictly prohibited".to_string(),
            });
        }

        // 1. Check for SSH syntax: git@host:owner/repo(.git)?(/tree/...)
        if self.allow_ssh && (trimmed.starts_with("git@") || trimmed.starts_with("ssh://git@")) {
            return self.parse_ssh_target(trimmed);
        }

        // 2. Check for shorthand syntax: owner/repo or owner/repo@ref
        if self.allow_shorthand && self.looks_like_shorthand(trimmed) {
            return self.parse_shorthand_target(trimmed);
        }

        // 3. Standard URL parsing (HTTPS, HTTP, GIT)
        self.parse_http_or_git_url(trimmed)
    }

    fn looks_like_shorthand(&self, input: &str) -> bool {
        // Shorthand must not contain scheme delimiters '://' or '@' at start or whitespace
        if input.contains("://") || input.contains(' ') || input.contains('\\') {
            return false;
        }

        let base = input.split('@').next().unwrap_or(input);
        let slash_count = base.chars().filter(|&c| c == '/').count();
        slash_count == 1 && !base.starts_with('/') && !base.ends_with('/')
    }

    fn parse_shorthand_target(&self, input: &str) -> Result<GitHubTarget, GitHubExtractorError> {
        let (nwo, ref_part) = if let Some((base, ref_spec)) = input.split_once('@') {
            (base, Some(ref_spec))
        } else {
            (input, None)
        };

        let (owner, repo) = nwo.split_once('/').ok_or_else(|| {
            GitHubExtractorError::InvalidUrl("Malformed shorthand repository format".to_string())
        })?;

        validate_owner(owner)?;
        let clean_repo = sanitize_repo_name(repo)?;
        validate_repo(&clean_repo)?;

        let ref_spec = match ref_part {
            Some(r) => {
                let trimmed_ref = r.trim();
                if trimmed_ref.is_empty() {
                    return Err(GitHubExtractorError::InvalidRef {
                        ref_name: r.to_string(),
                        reason: "Git ref specified after '@' cannot be empty".to_string(),
                    });
                }
                Some(trimmed_ref.to_string())
            }
            None => None,
        };

        Ok(GitHubTarget {
            owner: owner.to_string(),
            repo: clean_repo,
            target_type: GitHubTargetType::RepositoryRoot,
            ref_spec,
            subpath: None,
            host: DEFAULT_GITHUB_HOST.to_string(),
            is_enterprise: false,
        })
    }

    fn parse_ssh_target(&self, input: &str) -> Result<GitHubTarget, GitHubExtractorError> {
        let stripped = input.strip_prefix("ssh://").unwrap_or(input);
        let after_user = stripped.strip_prefix("git@").ok_or_else(|| {
            GitHubExtractorError::InvalidUrl("SSH URL must begin with git@".to_string())
        })?;

        let (host, path_part) = after_user.split_once(':').ok_or_else(|| {
            GitHubExtractorError::InvalidUrl("SSH URL missing host/path separator ':'".to_string())
        })?;

        let clean_host = host.to_ascii_lowercase();
        self.validate_host(&clean_host)?;

        let clean_path = path_part.trim_start_matches('/');
        let segments: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();

        if segments.len() < 2 {
            return Err(GitHubExtractorError::InvalidUrl(
                "SSH URL must contain at least owner and repo path segments".to_string(),
            ));
        }

        let owner = segments[0];
        validate_owner(owner)?;

        let clean_repo = sanitize_repo_name(segments[1])?;
        validate_repo(&clean_repo)?;

        let is_enterprise = !is_default_host(&clean_host);

        // Optional subpath/tree support in SSH URL
        if segments.len() > 3 && segments[2] == "tree" {
            let ref_name = segments[3].to_string();
            let subpath = if segments.len() > 4 {
                Some(segments[4..].join("/"))
            } else {
                None
            };

            return Ok(GitHubTarget {
                owner: owner.to_string(),
                repo: clean_repo,
                target_type: GitHubTargetType::Tree {
                    ref_name: ref_name.clone(),
                    subpath: subpath.clone(),
                },
                ref_spec: Some(ref_name),
                subpath,
                host: clean_host,
                is_enterprise,
            });
        }

        Ok(GitHubTarget {
            owner: owner.to_string(),
            repo: clean_repo,
            target_type: GitHubTargetType::RepositoryRoot,
            ref_spec: None,
            subpath: None,
            host: clean_host,
            is_enterprise,
        })
    }

    fn parse_http_or_git_url(&self, input: &str) -> Result<GitHubTarget, GitHubExtractorError> {
        let mut policy = UrlSecurityPolicy {
            max_url_length: 4096,
            ..Default::default()
        };
        if self.allow_git_transport {
            policy.allowed_schemes.push("git".to_string());
        }

        let parsed_url = if self.enforce_ssrf_protection {
            UrlNormalizer::normalize(input, &policy).map_err(GitHubExtractorError::UrlSecurity)?
        } else {
            Url::parse(input)
                .map_err(|e| GitHubExtractorError::InvalidUrl(format!("Parse error: {e}")))?
        };

        let scheme = parsed_url.scheme().to_ascii_lowercase();
        if scheme == "git" && !self.allow_git_transport {
            return Err(GitHubExtractorError::InvalidUrl(
                "git:// transport scheme is disallowed by configuration".to_string(),
            ));
        }

        if !parsed_url.username().is_empty() || parsed_url.password().is_some() {
            return Err(GitHubExtractorError::EmbeddedCredentialsForbidden);
        }

        let host = parsed_url
            .host_str()
            .ok_or_else(|| GitHubExtractorError::InvalidUrl("Missing host in URL".to_string()))?
            .to_ascii_lowercase();

        self.validate_host(&host)?;

        let is_enterprise = !is_default_host(&host);

        // Path segments decomposition
        let path = parsed_url.path();
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if segments.is_empty() {
            return Err(GitHubExtractorError::InvalidUrl(
                "GitHub URL path is empty; owner and repository required".to_string(),
            ));
        }

        let owner = segments[0];
        if is_reserved_github_path(owner) {
            return Err(GitHubExtractorError::ReservedPath(format!("/{owner}")));
        }
        validate_owner(owner)?;

        if segments.len() == 1 {
            return Err(GitHubExtractorError::InvalidUrl(format!(
                "URL points to GitHub user or organization profile '{owner}', not a repository"
            )));
        }

        let raw_repo = segments[1];
        let clean_repo = sanitize_repo_name(raw_repo)?;
        validate_repo(&clean_repo)?;

        // Segment 2+: Target type classification
        if segments.len() == 2 {
            return Ok(GitHubTarget {
                owner: owner.to_string(),
                repo: clean_repo,
                target_type: GitHubTargetType::RepositoryRoot,
                ref_spec: None,
                subpath: None,
                host,
                is_enterprise,
            });
        }

        let verb = segments[2];
        match verb {
            "tree" => {
                if segments.len() == 3 {
                    return Err(GitHubExtractorError::InvalidUrl(
                        "URL contains '/tree/' but no ref name was specified".to_string(),
                    ));
                }
                let ref_name = segments[3].to_string();
                let subpath = if segments.len() > 4 {
                    let raw_subpath = segments[4..].join("/");
                    validate_and_sanitize_subpath(&raw_subpath)?
                } else {
                    None
                };

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::Tree {
                        ref_name: ref_name.clone(),
                        subpath: subpath.clone(),
                    },
                    ref_spec: Some(ref_name),
                    subpath,
                    host,
                    is_enterprise,
                })
            }
            "blob" | "raw" => {
                if segments.len() == 3 {
                    return Err(GitHubExtractorError::InvalidUrl(
                        "URL contains '/blob/' or '/raw/' but no ref was specified".to_string(),
                    ));
                }
                let ref_name = segments[3].to_string();
                if segments.len() == 4 {
                    return Err(GitHubExtractorError::InvalidUrl(
                        "URL contains '/blob/' or '/raw/' but no file path was specified"
                            .to_string(),
                    ));
                }
                let raw_subpath = segments[4..].join("/");
                let file_path = validate_and_sanitize_subpath(&raw_subpath)?.ok_or_else(|| {
                    GitHubExtractorError::InvalidSubpath {
                        path: raw_subpath,
                        reason: "File path cannot resolve to an empty string".to_string(),
                    }
                })?;

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::Blob {
                        ref_name: ref_name.clone(),
                        path: file_path.clone(),
                    },
                    ref_spec: Some(ref_name),
                    subpath: Some(file_path),
                    host,
                    is_enterprise,
                })
            }
            "commit" => {
                if segments.len() == 3 {
                    return Err(GitHubExtractorError::InvalidUrl(
                        "URL contains '/commit/' but no commit SHA was specified".to_string(),
                    ));
                }
                let sha = segments[3];
                if !COMMIT_SHA_REGEX.is_match(sha) {
                    return Err(GitHubExtractorError::InvalidRef {
                        ref_name: sha.to_string(),
                        reason: "Commit SHA must contain 4 to 40 hexadecimal characters"
                            .to_string(),
                    });
                }

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::Commit {
                        commit_sha: sha.to_string(),
                    },
                    ref_spec: Some(sha.to_string()),
                    subpath: None,
                    host,
                    is_enterprise,
                })
            }
            "commits" => {
                let ref_name = if segments.len() > 3 {
                    Some(segments[3..].join("/"))
                } else {
                    None
                };

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::Commits {
                        ref_name: ref_name.clone(),
                    },
                    ref_spec: ref_name,
                    subpath: None,
                    host,
                    is_enterprise,
                })
            }
            "releases" => {
                if segments.len() >= 5 && segments[3] == "tag" {
                    let tag_name = segments[4..].join("/");
                    Ok(GitHubTarget {
                        owner: owner.to_string(),
                        repo: clean_repo,
                        target_type: GitHubTargetType::Tag {
                            tag_name: tag_name.clone(),
                        },
                        ref_spec: Some(tag_name),
                        subpath: None,
                        host,
                        is_enterprise,
                    })
                } else {
                    Ok(GitHubTarget {
                        owner: owner.to_string(),
                        repo: clean_repo,
                        target_type: GitHubTargetType::Releases,
                        ref_spec: None,
                        subpath: None,
                        host,
                        is_enterprise,
                    })
                }
            }
            "pull" => {
                if segments.len() == 3 {
                    return Err(GitHubExtractorError::InvalidUrl(
                        "URL contains '/pull/' but no pull request number was specified"
                            .to_string(),
                    ));
                }
                let num_str = segments[3];
                let number = num_str.parse::<u64>().map_err(|_| {
                    GitHubExtractorError::InvalidUrl(format!(
                        "Invalid pull request number '{num_str}'"
                    ))
                })?;

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::PullRequest { number },
                    ref_spec: None,
                    subpath: None,
                    host,
                    is_enterprise,
                })
            }
            "issues" => {
                if segments.len() > 3 {
                    let num_str = segments[3];
                    if let Ok(number) = num_str.parse::<u64>() {
                        return Ok(GitHubTarget {
                            owner: owner.to_string(),
                            repo: clean_repo,
                            target_type: GitHubTargetType::Issue { number },
                            ref_spec: None,
                            subpath: None,
                            host,
                            is_enterprise,
                        });
                    }
                }

                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::RepositoryRoot,
                    ref_spec: None,
                    subpath: None,
                    host,
                    is_enterprise,
                })
            }
            _ => {
                // If the segment is not a recognized verb, treat as repository root
                // with no ref or subpath.
                Ok(GitHubTarget {
                    owner: owner.to_string(),
                    repo: clean_repo,
                    target_type: GitHubTargetType::RepositoryRoot,
                    ref_spec: None,
                    subpath: None,
                    host,
                    is_enterprise,
                })
            }
        }
    }

    fn validate_host(&self, host: &str) -> Result<(), GitHubExtractorError> {
        let clean = host.to_ascii_lowercase();
        if self.allowed_hosts.contains(&clean) {
            return Ok(());
        }

        Err(GitHubExtractorError::UnsupportedHost(format!(
            "Host '{host}' is not in the allowed GitHub domains list"
        )))
    }
}

/// Checks whether a host is default public `github.com` or `www.github.com`.
#[must_use]
pub fn is_default_host(host: &str) -> bool {
    host.eq_ignore_ascii_case(DEFAULT_GITHUB_HOST)
        || host.eq_ignore_ascii_case(DEFAULT_GITHUB_WWW_HOST)
}

/// Checks whether a path segment is a reserved GitHub top-level namespace.
#[must_use]
pub fn is_reserved_github_path(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    RESERVED_GITHUB_NAMES.contains(lower.as_str())
}

/// Validates GitHub owner (username or organization) naming constraints.
///
/// # Errors
/// Returns `GitHubExtractorError::InvalidOwner` if the owner string violates GitHub naming rules.
pub fn validate_owner(owner: &str) -> Result<(), GitHubExtractorError> {
    if owner.is_empty() {
        return Err(GitHubExtractorError::InvalidOwner {
            owner: owner.to_string(),
            reason: "Owner name cannot be empty".to_string(),
        });
    }

    if owner.len() > 39 {
        return Err(GitHubExtractorError::InvalidOwner {
            owner: owner.to_string(),
            reason: "Owner name cannot exceed 39 characters".to_string(),
        });
    }

    if owner.starts_with('-') || owner.ends_with('-') {
        return Err(GitHubExtractorError::InvalidOwner {
            owner: owner.to_string(),
            reason: "Owner name cannot begin or end with a hyphen".to_string(),
        });
    }

    if owner.contains("--") {
        return Err(GitHubExtractorError::InvalidOwner {
            owner: owner.to_string(),
            reason: "Owner name cannot contain consecutive hyphens".to_string(),
        });
    }

    for ch in owner.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' {
            return Err(GitHubExtractorError::InvalidOwner {
                owner: owner.to_string(),
                reason: "Owner name may only contain alphanumeric characters and single hyphens"
                    .to_string(),
            });
        }
    }

    Ok(())
}

/// Strips `.git` suffix from repository name if present.
fn sanitize_repo_name(raw_repo: &str) -> Result<String, GitHubExtractorError> {
    let trimmed = raw_repo.trim();
    let stripped = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    if stripped.is_empty() {
        return Err(GitHubExtractorError::InvalidRepository {
            repo: raw_repo.to_string(),
            reason: "Repository name cannot be empty".to_string(),
        });
    }
    Ok(stripped.to_string())
}

/// Validates GitHub repository name constraints.
///
/// # Errors
/// Returns `GitHubExtractorError::InvalidRepository` if the repository violates GitHub rules.
pub fn validate_repo(repo: &str) -> Result<(), GitHubExtractorError> {
    if repo.is_empty() {
        return Err(GitHubExtractorError::InvalidRepository {
            repo: repo.to_string(),
            reason: "Repository name cannot be empty".to_string(),
        });
    }

    if repo == "." || repo == ".." {
        return Err(GitHubExtractorError::InvalidRepository {
            repo: repo.to_string(),
            reason: "Repository name cannot be '.' or '..'".to_string(),
        });
    }

    if repo.len() > 100 {
        return Err(GitHubExtractorError::InvalidRepository {
            repo: repo.to_string(),
            reason: "Repository name cannot exceed 100 characters".to_string(),
        });
    }

    if !REPO_REGEX.is_match(repo) {
        return Err(GitHubExtractorError::InvalidRepository {
            repo: repo.to_string(),
            reason: "Repository name may only contain alphanumeric characters, hyphens, underscores, and dots"
                .to_string(),
        });
    }

    Ok(())
}

/// Validates and sanitizes a relative repository subpath against path traversal attacks.
///
/// # Errors
/// Returns `GitHubExtractorError::InvalidSubpath` if the subpath contains path traversal sequences
/// or illegal characters.
pub fn validate_and_sanitize_subpath(
    raw_path: &str,
) -> Result<Option<String>, GitHubExtractorError> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if trimmed.contains('\0') {
        return Err(GitHubExtractorError::InvalidSubpath {
            path: trimmed.to_string(),
            reason: "Subpath contains null bytes".to_string(),
        });
    }

    // Normalize Windows backslashes to forward slashes
    let normalized = trimmed.replace('\\', "/");

    let mut clean_segments = Vec::new();
    for segment in normalized.split('/') {
        let seg = segment.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }

        if seg == ".." || seg.contains("%2e%2e") || seg.contains("%2E%2E") {
            return Err(GitHubExtractorError::InvalidSubpath {
                path: trimmed.to_string(),
                reason: "Directory traversal ('..') is strictly prohibited".to_string(),
            });
        }

        clean_segments.push(seg);
    }

    if clean_segments.is_empty() {
        Ok(None)
    } else {
        Ok(Some(clean_segments.join("/")))
    }
}
