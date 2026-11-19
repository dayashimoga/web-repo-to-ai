//! # Git Commit and Blob Incremental Caching Engine
//!
//! Provides content-addressed caching abstractions and incremental tree traversal
//! for GitHub repository conversions:
//! - [`TreeIncrementalDiff`] calculates fine-grained differences between two Git trees.
//! - [`GitCacheStore`] async trait for persistent or ephemeral Git commit and blob storage.
//! - [`InMemoryGitCacheStore`] thread-safe in-memory cache store.
//! - [`IncrementalBlobExtractor`] coordinated blob acquisition bypassing network I/O for cached blobs.
//! - [`IncrementalExtractionStats`] performance accounting metrics (cache hits, misses, byte savings).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info};

use crate::classification::ClassifiedFile;
use crate::error::GitHubExtractorError;
use crate::source::{ExtractedSourceFile, GitBlob, SourceExtractionConfig, SourceExtractionReport};
use crate::target::GitHubTarget;
use crate::tree::{GitTreeEntry, GitTreeEntryType, GitTreeResponse};

/// Fine-grained incremental difference between two Git tree entry snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeIncrementalDiff {
    /// Commit SHA of the reference/previous revision, if specified.
    pub previous_commit_sha: Option<String>,
    /// Commit SHA of the target/current revision, if specified.
    pub current_commit_sha: Option<String>,
    /// File blobs whose path and Git SHA-1 are identical across both revisions.
    pub unchanged_blobs: Vec<GitTreeEntry>,
    /// File blobs whose path exists in both revisions but Git SHA-1 has changed.
    pub modified_blobs: Vec<GitTreeEntry>,
    /// File blobs present in the current revision but absent from the previous revision.
    pub added_blobs: Vec<GitTreeEntry>,
    /// File blobs present in the previous revision but absent from the current revision.
    pub deleted_blobs: Vec<GitTreeEntry>,
}

impl TreeIncrementalDiff {
    /// Computes the incremental differences between a `previous` and `current` tree snapshot.
    #[must_use]
    pub fn compute(previous: &[GitTreeEntry], current: &[GitTreeEntry]) -> Self {
        let prev_blobs: HashMap<&str, &GitTreeEntry> = previous
            .iter()
            .filter(|e| matches!(e.entry_type, GitTreeEntryType::Blob) || e.mode.is_file())
            .map(|e| (e.path.as_str(), e))
            .collect();

        let curr_blobs: HashMap<&str, &GitTreeEntry> = current
            .iter()
            .filter(|e| matches!(e.entry_type, GitTreeEntryType::Blob) || e.mode.is_file())
            .map(|e| (e.path.as_str(), e))
            .collect();

        let mut unchanged_blobs = Vec::new();
        let mut modified_blobs = Vec::new();
        let mut added_blobs = Vec::new();
        let mut deleted_blobs = Vec::new();

        for curr in curr_blobs.values() {
            if let Some(prev) = prev_blobs.get(curr.path.as_str()) {
                if prev.sha.eq_ignore_ascii_case(&curr.sha) {
                    unchanged_blobs.push((*curr).clone());
                } else {
                    modified_blobs.push((*curr).clone());
                }
            } else {
                added_blobs.push((*curr).clone());
            }
        }

        for prev in prev_blobs.values() {
            if !curr_blobs.contains_key(prev.path.as_str()) {
                deleted_blobs.push((*prev).clone());
            }
        }

        // Sort deterministic order by relative path
        unchanged_blobs.sort_by(|a, b| a.path.cmp(&b.path));
        modified_blobs.sort_by(|a, b| a.path.cmp(&b.path));
        added_blobs.sort_by(|a, b| a.path.cmp(&b.path));
        deleted_blobs.sort_by(|a, b| a.path.cmp(&b.path));

        Self {
            previous_commit_sha: None,
            current_commit_sha: None,
            unchanged_blobs,
            modified_blobs,
            added_blobs,
            deleted_blobs,
        }
    }

    /// Associates immutable commit SHAs with the diff report.
    #[must_use]
    pub fn with_commits(
        mut self,
        previous_commit_sha: impl Into<String>,
        current_commit_sha: impl Into<String>,
    ) -> Self {
        self.previous_commit_sha = Some(previous_commit_sha.into());
        self.current_commit_sha = Some(current_commit_sha.into());
        self
    }

    /// Number of file blobs with identical content across revisions.
    #[must_use]
    pub fn unchanged_count(&self) -> usize {
        self.unchanged_blobs.len()
    }

    /// Number of file blobs modified between revisions.
    #[must_use]
    pub fn modified_count(&self) -> usize {
        self.modified_blobs.len()
    }

    /// Number of file blobs added in the current revision.
    #[must_use]
    pub fn added_count(&self) -> usize {
        self.added_blobs.len()
    }

    /// Number of file blobs removed from the current revision.
    #[must_use]
    pub fn deleted_count(&self) -> usize {
        self.deleted_blobs.len()
    }

    /// Total number of blobs in the current revision ($N = \text{unchanged} + \text{modified} + \text{added}$).
    #[must_use]
    pub fn total_current_blobs(&self) -> usize {
        self.unchanged_blobs.len() + self.modified_blobs.len() + self.added_blobs.len()
    }

    /// Ratio of unchanged blobs in the current revision ($0.0$ to $1.0$).
    #[must_use]
    pub fn unchanged_ratio(&self) -> f64 {
        let total = self.total_current_blobs();
        if total == 0 {
            1.0
        } else {
            self.unchanged_blobs.len() as f64 / total as f64
        }
    }

    /// Returns a set of unique Git object SHAs that can be reused from cache.
    #[must_use]
    pub fn reusable_shas(&self) -> HashSet<String> {
        self.unchanged_blobs.iter().map(|e| e.sha.clone()).collect()
    }
}

/// Asynchronous abstraction for caching Git commits, trees, and blobs.
#[async_trait]
pub trait GitCacheStore: Send + Sync {
    /// Retrieves a cached Git blob by its Git SHA-1.
    async fn get_cached_blob(&self, sha: &str) -> Result<Option<GitBlob>, GitHubExtractorError>;

    /// Stores a Git blob in the cache.
    async fn store_cached_blob(&self, blob: &GitBlob) -> Result<(), GitHubExtractorError>;

    /// Batch-queries a slice of Git SHAs, returning the set of SHAs present in the cache.
    async fn check_cached_blobs(
        &self,
        shas: &[String],
    ) -> Result<HashSet<String>, GitHubExtractorError>;

    /// Retrieves a cached Git tree response for a repository and commit SHA.
    async fn get_cached_tree(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<Option<GitTreeResponse>, GitHubExtractorError>;

    /// Stores a Git tree response in the cache for a repository and commit SHA.
    async fn store_cached_tree(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
        tree: &GitTreeResponse,
    ) -> Result<(), GitHubExtractorError>;
}

/// Cache key identifying a repository commit tree: `(owner, repo, commit_sha)`.
pub type TreeCacheKey = (String, String, String);

type BlobCacheMap = Arc<RwLock<HashMap<String, GitBlob>>>;
type TreeCacheMap = Arc<RwLock<HashMap<TreeCacheKey, GitTreeResponse>>>;

/// Thread-safe in-memory cache store implementing [`GitCacheStore`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryGitCacheStore {
    blobs: BlobCacheMap,
    trees: TreeCacheMap,
}

impl InMemoryGitCacheStore {
    /// Constructs a new, empty `InMemoryGitCacheStore`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blobs: Arc::new(RwLock::new(HashMap::new())),
            trees: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Number of cached blobs.
    pub async fn blob_count(&self) -> usize {
        self.blobs.read().await.len()
    }

    /// Number of cached trees.
    pub async fn tree_count(&self) -> usize {
        self.trees.read().await.len()
    }
}

#[async_trait]
impl GitCacheStore for InMemoryGitCacheStore {
    async fn get_cached_blob(&self, sha: &str) -> Result<Option<GitBlob>, GitHubExtractorError> {
        let key = sha.to_lowercase();
        let map = self.blobs.read().await;
        Ok(map.get(&key).cloned())
    }

    async fn store_cached_blob(&self, blob: &GitBlob) -> Result<(), GitHubExtractorError> {
        let key = blob.sha.to_lowercase();
        let mut map = self.blobs.write().await;
        map.insert(key, blob.clone());
        Ok(())
    }

    async fn check_cached_blobs(
        &self,
        shas: &[String],
    ) -> Result<HashSet<String>, GitHubExtractorError> {
        let map = self.blobs.read().await;
        let mut hits = HashSet::with_capacity(shas.len());
        for sha in shas {
            let key = sha.to_lowercase();
            if map.contains_key(&key) {
                hits.insert(sha.clone());
            }
        }
        Ok(hits)
    }

    async fn get_cached_tree(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<Option<GitTreeResponse>, GitHubExtractorError> {
        let key = (
            owner.to_lowercase(),
            repo.to_lowercase(),
            commit_sha.to_lowercase(),
        );
        let map = self.trees.read().await;
        Ok(map.get(&key).cloned())
    }

    async fn store_cached_tree(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
        tree: &GitTreeResponse,
    ) -> Result<(), GitHubExtractorError> {
        let key = (
            owner.to_lowercase(),
            repo.to_lowercase(),
            commit_sha.to_lowercase(),
        );
        let mut map = self.trees.write().await;
        map.insert(key, tree.clone());
        Ok(())
    }
}

/// Quantitative performance metrics from an incremental blob extraction run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncrementalExtractionStats {
    /// Total eligible files submitted for extraction.
    pub total_files: usize,
    /// Files successfully retrieved from local cache (zero network requests).
    pub cached_files: usize,
    /// Files fetched from network / GitHub API (cache misses).
    pub fetched_files: usize,
    /// Files that encountered fetch or verification failures.
    pub failed_files: usize,
    /// Files skipped as unsupported (e.g. binary).
    pub unsupported_files: usize,
    /// Total byte size served directly from cache.
    pub cached_bytes: u64,
    /// Total byte size downloaded from network.
    pub fetched_bytes: u64,
    /// Elapsed duration of the extraction operation.
    pub duration: Duration,
}

impl Default for IncrementalExtractionStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            cached_files: 0,
            fetched_files: 0,
            failed_files: 0,
            unsupported_files: 0,
            cached_bytes: 0,
            fetched_bytes: 0,
            duration: Duration::ZERO,
        }
    }
}

impl IncrementalExtractionStats {
    /// Creates a new empty `IncrementalExtractionStats`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ratio of files retrieved from cache over total processed eligible files ($0.0$ to $1.0$).
    #[must_use]
    pub fn cache_hit_ratio(&self) -> f64 {
        let processed = self.cached_files + self.fetched_files;
        if processed == 0 {
            0.0
        } else {
            self.cached_files as f64 / processed as f64
        }
    }

    /// Ratio of bytes retrieved from cache over total bytes acquired.
    #[must_use]
    pub fn byte_hit_ratio(&self) -> f64 {
        let total = self.cached_bytes + self.fetched_bytes;
        if total == 0 {
            0.0
        } else {
            self.cached_bytes as f64 / total as f64
        }
    }
}

/// Orchestrates incremental repository file extraction leveraging [`GitCacheStore`].
pub struct IncrementalBlobExtractor;

impl IncrementalBlobExtractor {
    /// Concurrently extracts source files, checking the cache first and falling back to a fetcher.
    pub async fn extract_with_fetcher<F, Fut>(
        target: &GitHubTarget,
        files: &[ClassifiedFile],
        config: &SourceExtractionConfig,
        cache: &dyn GitCacheStore,
        fetch_fn: F,
    ) -> (SourceExtractionReport, IncrementalExtractionStats)
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<GitBlob, GitHubExtractorError>> + Send + 'static,
    {
        let start = Instant::now();
        let mut report = SourceExtractionReport::new();
        let mut stats = IncrementalExtractionStats::new();

        let mut eligible_files = Vec::new();
        for file in files {
            if !file.is_eligible {
                if file.category.is_binary() {
                    report.add_unsupported(file.clone());
                    stats.unsupported_files += 1;
                }
            } else {
                eligible_files.push(file.clone());
            }
        }

        stats.total_files = eligible_files.len();

        if eligible_files.is_empty() {
            stats.duration = start.elapsed();
            report.duration = stats.duration;
            return (report, stats);
        }

        // 1. Batch query cache for existing blob SHAs
        let shas: Vec<String> = eligible_files.iter().map(|f| f.sha.clone()).collect();
        let cached_shas = cache.check_cached_blobs(&shas).await.unwrap_or_default();

        debug!(
            "Incremental cache lookup: {}/{} blobs present in cache for {}/{}",
            cached_shas.len(),
            eligible_files.len(),
            target.owner,
            target.repo
        );

        let mut to_fetch = Vec::new();

        // 2. Fast-path cached blobs
        for file in eligible_files {
            if cached_shas.contains(&file.sha) {
                match cache.get_cached_blob(&file.sha).await {
                    Ok(Some(blob)) => {
                        match ExtractedSourceFile::from_blob(&file.path, &blob, config) {
                            Ok(extracted) => {
                                stats.cached_files += 1;
                                stats.cached_bytes += blob.size as u64;
                                report.add_extracted(extracted);
                            }
                            Err(e) => {
                                // Cache entry corrupted or binary; record failure
                                stats.failed_files += 1;
                                report.add_failed(file.path, file.sha, e.to_string());
                            }
                        }
                    }
                    _ => {
                        // Fallback to fetch if cache read missed
                        to_fetch.push(file);
                    }
                }
            } else {
                to_fetch.push(file);
            }
        }

        // 3. Concurrently fetch cache misses
        if !to_fetch.is_empty() {
            let concurrency = config.concurrency.clamp(1, 50);
            let semaphore = Arc::new(Semaphore::new(concurrency));
            let fetch_fn = Arc::new(fetch_fn);

            let mut tasks = Vec::with_capacity(to_fetch.len());
            for file in to_fetch {
                let sem = Arc::clone(&semaphore);
                let fetcher = Arc::clone(&fetch_fn);
                let cfg = config.clone();

                tasks.push(tokio::spawn(async move {
                    let _permit = match sem.acquire().await {
                        Ok(p) => p,
                        Err(e) => {
                            return (
                                file.clone(),
                                Err(GitHubExtractorError::RequestFailed(format!(
                                    "Semaphore permit acquisition failed: {e}"
                                ))),
                            );
                        }
                    };

                    let res = fetcher(file.sha.clone()).await;
                    match res {
                        Ok(blob) => {
                            let extracted_res =
                                ExtractedSourceFile::from_blob(&file.path, &blob, &cfg);
                            (file, Ok((blob, extracted_res)))
                        }
                        Err(e) => (file, Err(e)),
                    }
                }));
            }

            for task in tasks {
                match task.await {
                    Ok((_file, Ok((blob, Ok(extracted))))) => {
                        stats.fetched_files += 1;
                        stats.fetched_bytes += blob.size as u64;
                        // Populate cache
                        let _ = cache.store_cached_blob(&blob).await;
                        report.add_extracted(extracted);
                    }
                    Ok((file, Ok((_blob, Err(err))))) => {
                        stats.failed_files += 1;
                        report.add_failed(file.path, file.sha, err.to_string());
                    }
                    Ok((file, Err(err))) => {
                        stats.failed_files += 1;
                        report.add_failed(file.path, file.sha, err.to_string());
                    }
                    Err(join_err) => {
                        stats.failed_files += 1;
                        report.add_failed(
                            "unknown".to_string(),
                            String::new(),
                            format!("Tokio task join error: {join_err}"),
                        );
                    }
                }
            }
        }

        stats.duration = start.elapsed();
        report.duration = stats.duration;

        info!(
            "Incremental extraction complete: {} cached, {} fetched, {} failed (hit ratio: {:.1}%)",
            stats.cached_files,
            stats.fetched_files,
            stats.failed_files,
            stats.cache_hit_ratio() * 100.0
        );

        (report, stats)
    }
}
