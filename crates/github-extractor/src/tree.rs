//! # Git Trees Data Model and Hierarchy Traversal
//!
//! Provides structured models and helper methods for representing GitHub Git Trees
//! (`GET /repos/{owner}/{repo}/git/trees/{tree_sha}`), typed file modes, Git object types,
//! normalized relative paths, and subtree/blob queries.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Git file modes representing file types and POSIX permission bits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitFileMode {
    /// Standard non-executable file (`100644`).
    #[serde(rename = "100644")]
    RegularFile,
    /// Executable script or binary (`100755`).
    #[serde(rename = "100755")]
    ExecutableFile,
    /// Subdirectory or subtree (`040000`).
    #[serde(rename = "040000")]
    Subdirectory,
    /// Symbolic link pointer (`120000`).
    #[serde(rename = "120000")]
    Symlink,
    /// Git submodule commit pointer (`160000`).
    #[serde(rename = "160000")]
    Submodule,
    /// Non-standard or unrecognized file mode.
    #[serde(untagged)]
    Other(String),
}

impl GitFileMode {
    /// Parses a mode string into a strongly-typed `GitFileMode`.
    #[must_use]
    pub fn from_mode_str(mode: &str) -> Self {
        match mode.trim() {
            "100644" => Self::RegularFile,
            "100755" => Self::ExecutableFile,
            "040000" => Self::Subdirectory,
            "120000" => Self::Symlink,
            "160000" => Self::Submodule,
            other => Self::Other(other.to_string()),
        }
    }

    /// Returns the standard 6-character mode string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::RegularFile => "100644",
            Self::ExecutableFile => "100755",
            Self::Subdirectory => "040000",
            Self::Symlink => "120000",
            Self::Submodule => "160000",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Returns whether this mode represents a file object (regular, executable, or symlink).
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(
            self,
            Self::RegularFile | Self::ExecutableFile | Self::Symlink
        )
    }

    /// Returns whether this mode represents a directory tree.
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Subdirectory)
    }

    /// Returns whether this mode has executable permissions (`100755`).
    #[must_use]
    pub const fn is_executable(&self) -> bool {
        matches!(self, Self::ExecutableFile)
    }

    /// Returns whether this mode represents a symbolic link (`120000`).
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        matches!(self, Self::Symlink)
    }

    /// Returns whether this mode represents a Git submodule (`160000`).
    #[must_use]
    pub const fn is_submodule(&self) -> bool {
        matches!(self, Self::Submodule)
    }
}

impl fmt::Display for GitFileMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Git object type classification returned by GitHub's Git Trees API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitTreeEntryType {
    /// File content blob.
    Blob,
    /// Directory subtree.
    Tree,
    /// Submodule commit reference.
    Commit,
}

impl GitTreeEntryType {
    /// Returns the lowercase string identifier for this object type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
        }
    }
}

impl fmt::Display for GitTreeEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single entry within a Git tree response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitTreeEntry {
    /// Repository-relative path with normalized forward slashes (e.g. `crates/core/src/lib.rs`).
    pub path: String,
    /// POSIX file mode.
    pub mode: GitFileMode,
    /// Git object type (`blob`, `tree`, or `commit`).
    #[serde(rename = "type")]
    pub entry_type: GitTreeEntryType,
    /// 40-character hexadecimal SHA-1 Git object hash.
    pub sha: String,
    /// File size in bytes (present for blobs, `None` for trees and submodules).
    pub size: Option<u64>,
    /// GitHub API URL for the direct Git object, if returned.
    pub url: Option<String>,
}

impl GitTreeEntry {
    /// Constructs a new `GitTreeEntry` ensuring paths are normalized.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        mode: GitFileMode,
        entry_type: GitTreeEntryType,
        sha: impl Into<String>,
        size: Option<u64>,
    ) -> Self {
        let raw_path = path.into();
        let normalized = raw_path
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .replace('\\', "/");

        Self {
            path: normalized,
            mode,
            entry_type,
            sha: sha.into(),
            size,
            url: None,
        }
    }

    /// Returns the file name or last path segment.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// Returns the lowercase file extension, if any.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name();
        name.rsplit_once('.').map(|(_, ext)| ext)
    }

    /// Returns the parent directory path, or `None` if in the repository root.
    #[must_use]
    pub fn parent_path(&self) -> Option<&str> {
        self.path.rsplit_once('/').map(|(parent, _)| parent)
    }

    /// Returns whether this entry represents a blob file.
    #[must_use]
    pub const fn is_blob(&self) -> bool {
        matches!(self.entry_type, GitTreeEntryType::Blob)
    }

    /// Returns whether this entry represents a subtree directory.
    #[must_use]
    pub const fn is_tree(&self) -> bool {
        matches!(self.entry_type, GitTreeEntryType::Tree)
    }

    /// Returns whether this entry represents a submodule commit.
    #[must_use]
    pub const fn is_submodule(&self) -> bool {
        matches!(self.entry_type, GitTreeEntryType::Commit)
    }

    /// Returns the nesting depth within the directory tree (root = 0).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.path.chars().filter(|&c| c == '/').count()
    }
}

/// Structured response from GitHub's Git Trees REST API (`GET /repos/{owner}/{repo}/git/trees/{tree_sha}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitTreeResponse {
    /// SHA of the root tree object queried.
    pub sha: String,
    /// GitHub API URL for this tree.
    pub url: Option<String>,
    /// Array of tree objects (blobs, subtrees, and submodules).
    pub tree: Vec<GitTreeEntry>,
    /// Whether the tree response was truncated by GitHub (>100,000 entries or 7 MB response).
    #[serde(default)]
    pub truncated: bool,
}

impl GitTreeResponse {
    /// Constructs a new `GitTreeResponse`.
    #[must_use]
    pub fn new(sha: impl Into<String>, tree: Vec<GitTreeEntry>, truncated: bool) -> Self {
        Self {
            sha: sha.into(),
            url: None,
            tree,
            truncated,
        }
    }

    /// Returns total count of blob (file) entries in this tree.
    #[must_use]
    pub fn blob_count(&self) -> usize {
        self.tree.iter().filter(|e| e.is_blob()).count()
    }

    /// Returns total count of subtree (directory) entries in this tree.
    #[must_use]
    pub fn tree_count(&self) -> usize {
        self.tree.iter().filter(|e| e.is_tree()).count()
    }

    /// Returns total count of submodule commit entries in this tree.
    #[must_use]
    pub fn submodule_count(&self) -> usize {
        self.tree.iter().filter(|e| e.is_submodule()).count()
    }

    /// Returns aggregate byte size of all blobs for which size was reported.
    #[must_use]
    pub fn total_blob_size(&self) -> u64 {
        self.tree
            .iter()
            .filter_map(|e| if e.is_blob() { e.size } else { None })
            .sum()
    }

    /// Returns an iterator yielding only blob (file) entries.
    pub fn blobs(&self) -> impl Iterator<Item = &GitTreeEntry> {
        self.tree.iter().filter(|e| e.is_blob())
    }

    /// Finds a tree entry by exact relative path.
    #[must_use]
    pub fn find_by_path(&self, path: &str) -> Option<&GitTreeEntry> {
        let clean = path.trim_start_matches('/').trim_start_matches('\\');
        self.tree.iter().find(|e| e.path == clean)
    }

    /// Returns all entries whose path starts with the given prefix.
    #[must_use]
    pub fn filter_by_prefix(&self, prefix: &str) -> Vec<&GitTreeEntry> {
        let clean = prefix.trim_start_matches('/').trim_start_matches('\\');
        let with_slash = if clean.ends_with('/') || clean.is_empty() {
            clean.to_string()
        } else {
            format!("{clean}/")
        };

        self.tree
            .iter()
            .filter(|e| e.path == clean || e.path.starts_with(&with_slash))
            .collect()
    }

    /// Returns all entries matching the given file extension (case-insensitive).
    #[must_use]
    pub fn filter_by_extension(&self, extension: &str) -> Vec<&GitTreeEntry> {
        let target_ext = extension.trim_start_matches('.').to_ascii_lowercase();
        self.tree
            .iter()
            .filter(|e| {
                e.extension()
                    .map(|ext| ext.eq_ignore_ascii_case(&target_ext))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Collects all entry paths into a `HashSet` for O(1) set membership and completeness auditing.
    #[must_use]
    pub fn to_path_set(&self) -> HashSet<String> {
        self.tree.iter().map(|e| e.path.clone()).collect()
    }
}
