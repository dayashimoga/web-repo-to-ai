//! # Subtree Recovery and Archive Fallback
//!
//! Provides automated remediation for truncated Git Trees API responses:
//! - Configurable limits for recursive subtree expansion (`SubtreeRecoveryConfig`)
//! - Path-aware tree stitching merging subtree results into the parent tree (`TreeStitcher`)
//! - In-memory shallow archive (zipball) parsing reconstructing complete tree responses (`scan_zip_archive`)
//! - Comprehensive recovery reporting detailing recovered entries, counts, and fallback status (`SubtreeRecoveryReport`)

use std::fmt;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;
use zip::ZipArchive;

use crate::error::{redact_secrets, GitHubExtractorError};
use crate::tree::{
    GitFileMode, GitTreeEntry, GitTreeEntryType, GitTreeResponse, TreeCompleteness,
    SUBTREE_TRAVERSAL_LIMIT,
};

/// Supported archive packaging formats for shallow fallback downloads from GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    /// Compressed tarball archive (`.tar.gz`).
    Tarball,
    /// Standard zipball archive (`.zip`).
    Zipball,
}

impl ArchiveFormat {
    /// Returns the GitHub REST API URL path segment for this archive format.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tarball => "tarball",
            Self::Zipball => "zipball",
        }
    }

    /// Returns the standard file extension with leading period.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Tarball => ".tar.gz",
            Self::Zipball => ".zip",
        }
    }

    /// Returns the standard HTTP MIME media type.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Tarball => "application/x-gzip",
            Self::Zipball => "application/zip",
        }
    }
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration governing automated subtree recovery traversal and fallback thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtreeRecoveryConfig {
    /// Maximum count of unexpanded subtrees to fetch via individual Git Trees API queries.
    /// Exceeding this triggers automatic archive fallback if enabled.
    pub max_subtrees: usize,
    /// Maximum recursion depth allowed during nested subtree expansion (root = 0).
    pub max_depth: usize,
    /// Whether to automatically fall back to shallow zipball archive downloading
    /// when subtree traversal exceeds bounds or fails.
    pub auto_fallback: bool,
    /// Maximum concurrent API requests allowed during recovery traversal.
    pub concurrency: usize,
}

impl Default for SubtreeRecoveryConfig {
    fn default() -> Self {
        Self {
            max_subtrees: SUBTREE_TRAVERSAL_LIMIT,
            max_depth: 10,
            auto_fallback: true,
            concurrency: 1,
        }
    }
}

impl SubtreeRecoveryConfig {
    /// Creates a new `SubtreeRecoveryConfig` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum count of subtrees to traverse.
    #[must_use]
    pub const fn with_max_subtrees(mut self, max: usize) -> Self {
        self.max_subtrees = max;
        self
    }

    /// Sets the maximum recursion depth.
    #[must_use]
    pub const fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Configures whether shallow archive fallback is automatically triggered.
    #[must_use]
    pub const fn with_auto_fallback(mut self, auto: bool) -> Self {
        self.auto_fallback = auto;
        self
    }

    /// Sets the maximum concurrency for subtree queries.
    #[must_use]
    pub const fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }
}

/// Diagnostic report detailing the results of an automated subtree recovery operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtreeRecoveryReport {
    /// Tree completeness state prior to beginning recovery.
    pub initial_completeness: TreeCompleteness,
    /// Count of individual subtree API queries attempted.
    pub subtrees_attempted: usize,
    /// Count of subtree API queries successfully resolved and stitched.
    pub subtrees_succeeded: usize,
    /// Count of subtree API queries that failed or could not be completed.
    pub subtrees_failed: usize,
    /// Count of new file blob entries recovered and stitched into the tree.
    pub additional_blobs_recovered: usize,
    /// Count of new directory subtree entries recovered and stitched into the tree.
    pub additional_trees_recovered: usize,
    /// Total entry count in the tree after recovery completion.
    pub final_total_entries: usize,
    /// Tree completeness evaluation following recovery.
    pub final_completeness: TreeCompleteness,
    /// Whether shallow archive fallback was triggered to complete the recovery.
    pub fallback_triggered: bool,
    /// Remaining unexpanded subtree paths, if any could not be resolved.
    pub remaining_unexpanded: Vec<String>,
}

impl SubtreeRecoveryReport {
    /// Returns whether the tree was successfully recovered to 100% verified completeness.
    #[must_use]
    pub const fn is_fully_recovered(&self) -> bool {
        self.final_completeness.is_complete()
    }
}

/// Helper utility for normalizing and stitching relative paths and subtree responses.
#[derive(Debug, Clone, Copy)]
pub struct TreeStitcher;

impl TreeStitcher {
    /// Joins a parent directory path and a child relative path, normalizing separators
    /// to clean forward slashes and stripping redundant delimiters.
    ///
    /// # Examples
    /// ```
    /// use web_repo_to_ai_github_extractor::stitch_child_path;
    ///
    /// assert_eq!(stitch_child_path("crates/core", "src/lib.rs"), "crates/core/src/lib.rs");
    /// assert_eq!(stitch_child_path("crates/core/", "/src/lib.rs"), "crates/core/src/lib.rs");
    /// assert_eq!(stitch_child_path("", "README.md"), "README.md");
    /// ```
    #[must_use]
    pub fn stitch_child_path(parent_path: &str, child_path: &str) -> String {
        let clean_parent = parent_path
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .trim_end_matches('/')
            .trim_end_matches('\\')
            .replace('\\', "/");

        let clean_child = child_path
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .trim_end_matches('/')
            .trim_end_matches('\\')
            .replace('\\', "/");

        if clean_parent.is_empty() {
            clean_child
        } else if clean_child.is_empty() {
            clean_parent
        } else {
            format!("{clean_parent}/{clean_child}")
        }
    }

    /// Stitches all entries from a child `subtree` into `base_tree`, prepending `parent_dir_path`
    /// to each entry path and deduplicating by normalized relative path.
    ///
    /// Returns `(additional_blobs, additional_trees)`.
    pub fn stitch_subtree(
        base_tree: &mut GitTreeResponse,
        parent_dir_path: &str,
        subtree: &GitTreeResponse,
    ) -> (usize, usize) {
        let mut existing_paths = base_tree.to_path_set();
        let mut blobs_added = 0;
        let mut trees_added = 0;

        for entry in &subtree.tree {
            let stitched_path = Self::stitch_child_path(parent_dir_path, &entry.path);

            if !existing_paths.contains(&stitched_path) {
                if entry.is_blob() {
                    blobs_added += 1;
                } else if entry.is_tree() {
                    trees_added += 1;
                }

                existing_paths.insert(stitched_path.clone());
                base_tree.tree.push(GitTreeEntry::new(
                    stitched_path,
                    entry.mode.clone(),
                    entry.entry_type,
                    &entry.sha,
                    entry.size,
                ));
            }
        }

        (blobs_added, trees_added)
    }
}

/// Convenience standalone function wrapping `TreeStitcher::stitch_child_path`.
#[must_use]
pub fn stitch_child_path(parent_path: &str, child_path: &str) -> String {
    TreeStitcher::stitch_child_path(parent_path, child_path)
}

/// Parses an in-memory GitHub zipball archive byte stream, reconstructing a clean `GitTreeResponse`.
///
/// In GitHub zipball archives, all entries are nested under a single top-level directory
/// formatted as `{owner}-{repo}-{short_sha}/`. When `strip_root_dir` is true, this top-level
/// container is stripped, leaving clean repository-relative paths matching Git Trees API conventions.
///
/// # Errors
/// Returns `GitHubExtractorError::ArchiveError` if the zip stream is malformed or invalid.
pub fn scan_zip_archive(
    zip_bytes: &[u8],
    strip_root_dir: bool,
) -> Result<GitTreeResponse, GitHubExtractorError> {
    let reader = Cursor::new(zip_bytes);
    let mut zip = ZipArchive::new(reader).map_err(|e| {
        let sanitized = redact_secrets(&e.to_string());
        GitHubExtractorError::ArchiveError(format!("Failed to open zip archive: {sanitized}"))
    })?;

    let mut entries = Vec::new();
    let mut root_dir_prefix: Option<String> = None;

    // Detect root directory prefix if required
    if strip_root_dir && !zip.is_empty() {
        if let Ok(first_entry) = zip.by_index(0) {
            if let Ok(name_cow) = first_entry.name() {
                let name = name_cow.replace('\\', "/");
                if let Some((first_segment, _)) = name.split_once('/') {
                    root_dir_prefix = Some(format!("{first_segment}/"));
                }
            }
        }
    }

    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|e| {
            let sanitized = redact_secrets(&e.to_string());
            GitHubExtractorError::ArchiveError(format!(
                "Failed to read entry {i} in zip archive: {sanitized}"
            ))
        })?;

        let raw_name = file
            .name()
            .map_err(|e| {
                let sanitized = redact_secrets(&e.to_string());
                GitHubExtractorError::ArchiveError(format!(
                    "Failed to decode entry {i} name: {sanitized}"
                ))
            })?
            .replace('\\', "/");

        let path = if let Some(ref prefix) = root_dir_prefix {
            if let Some(stripped) = raw_name.strip_prefix(prefix) {
                stripped.to_string()
            } else {
                continue;
            }
        } else {
            raw_name
        };

        let trimmed_path = path.trim_matches('/').to_string();
        if trimmed_path.is_empty() {
            continue;
        }

        let is_dir = file.is_dir() || path.ends_with('/');
        let mode = if is_dir {
            GitFileMode::Subdirectory
        } else if let Some(unix_mode) = file.unix_mode() {
            if unix_mode & 0o111 != 0 {
                GitFileMode::ExecutableFile
            } else {
                GitFileMode::RegularFile
            }
        } else {
            GitFileMode::RegularFile
        };

        let entry_type = if is_dir {
            GitTreeEntryType::Tree
        } else {
            GitTreeEntryType::Blob
        };

        let size = if is_dir { None } else { Some(file.size()) };

        // Generate a deterministic 40-character hex identifier from path and size
        let mut hasher = Sha256::new();
        hasher.update(trimmed_path.as_bytes());
        hasher.update(file.size().to_le_bytes());
        let hash = hasher.finalize();
        let sha = format!("{hash:x}")[..40].to_string();

        entries.push(GitTreeEntry::new(trimmed_path, mode, entry_type, sha, size));
    }

    info!(
        "Reconstructed GitTreeResponse from zip archive with {} total entries",
        entries.len()
    );

    Ok(GitTreeResponse::new(
        "recovered-from-zip-archive",
        entries,
        false,
    ))
}
