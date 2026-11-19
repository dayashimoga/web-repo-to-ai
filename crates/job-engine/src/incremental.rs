//! # Changed-File Processing & Incremental Execution Subsystem
//!
//! Provides deterministic change classification, diff calculation against baseline
//! signatures, incremental execution planning, bounded parallel execution of only
//! added/modified items, zero-overhead cache reuse for unchanged files, and mathematical
//! conservation verification:
//! - [`FileSignature`]: cryptographic path, hash, byte size, and timestamp snapshot.
//! - [`ChangeCategory`]: categorization (`Unchanged`, `Added`, `Modified`, `Deleted`).
//! - [`ChangedFileDiff`]: set difference partition between baseline and current states.
//! - [`IncrementalExecutionPlan`]: partition of items to process vs items to reuse.
//! - [`IncrementalJobCoordinator`]: coordinator running bounded parallel execution on changed items.
//! - [`IncrementalExecutionReport`]: comprehensive report detailing execution and savings metrics.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use web_repo_to_ai_core::id::{BatchId, JobId};

use crate::error::JobEngineError;
use crate::executor::BoundedParallelExecutor;
use crate::models::{BatchItemStatusOutcome, BatchItemTask};

/// Cryptographic and structural signature of an extractable file or resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileSignature {
    /// Relative normalized path or canonical identifier of the resource.
    pub path: String,
    /// Cryptographic digest (e.g. BLAKE3, SHA-256, or Git blob SHA).
    pub content_hash: String,
    /// Byte size of the resource payload.
    pub byte_size: u64,
    /// Optional timestamp of last modification.
    pub last_modified: Option<DateTime<Utc>>,
}

impl FileSignature {
    /// Constructs a new `FileSignature`.
    #[must_use]
    pub fn new(path: impl Into<String>, content_hash: impl Into<String>, byte_size: u64) -> Self {
        Self {
            path: path.into(),
            content_hash: content_hash.into().to_ascii_lowercase(),
            byte_size,
            last_modified: None,
        }
    }

    /// Sets the optional `last_modified` timestamp.
    #[must_use]
    pub fn with_last_modified(mut self, timestamp: DateTime<Utc>) -> Self {
        self.last_modified = Some(timestamp);
        self
    }
}

/// Categorical change classification between baseline and current states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeCategory {
    /// Content hash matches baseline; eligible for direct cache reuse.
    Unchanged,
    /// Brand new item not present in baseline; requires processing.
    Added,
    /// Item existed in baseline but content hash or size changed; requires reprocessing.
    Modified,
    /// Item existed in baseline but is no longer present; dropped from output.
    Deleted,
}

impl ChangeCategory {
    /// Returns `true` if this item must be processed (either Added or Modified).
    #[must_use]
    pub fn requires_processing(&self) -> bool {
        matches!(self, Self::Added | Self::Modified)
    }

    /// Returns `true` if this item can be reused without processing.
    #[must_use]
    pub fn is_reusable(&self) -> bool {
        matches!(self, Self::Unchanged)
    }
}

/// Granular change record for a specific resource path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedChange {
    /// Target path.
    pub path: String,
    /// Categorical classification.
    pub category: ChangeCategory,
    /// Current signature if present in current state.
    pub current: Option<FileSignature>,
    /// Baseline signature if present in baseline state.
    pub baseline: Option<FileSignature>,
}

/// Structural set diff comparing baseline and current resource manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFileDiff {
    /// Total count of items in baseline.
    pub baseline_count: usize,
    /// Total count of items in current candidate set.
    pub current_count: usize,
    /// New items to be processed.
    pub added: Vec<FileSignature>,
    /// Modified items to be reprocessed.
    pub modified: Vec<FileSignature>,
    /// Unchanged items to be reused directly from cache.
    pub unchanged: Vec<FileSignature>,
    /// Items dropped since baseline.
    pub deleted: Vec<FileSignature>,
}

impl ChangedFileDiff {
    /// Computes $O(N)$ set difference partition comparing baseline and current manifests.
    #[must_use]
    pub fn from_manifests(baseline: &[FileSignature], current: &[FileSignature]) -> Self {
        let mut baseline_map: HashMap<&str, &FileSignature> =
            HashMap::with_capacity(baseline.len());
        for sig in baseline {
            baseline_map.insert(sig.path.as_str(), sig);
        }

        let mut current_map: HashMap<&str, &FileSignature> = HashMap::with_capacity(current.len());
        for sig in current {
            current_map.insert(sig.path.as_str(), sig);
        }

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut unchanged = Vec::new();
        let mut deleted = Vec::new();

        // Classify current items
        for sig in current {
            match baseline_map.get(sig.path.as_str()) {
                None => {
                    added.push(sig.clone());
                }
                Some(base) => {
                    if base.content_hash == sig.content_hash && base.byte_size == sig.byte_size {
                        unchanged.push(sig.clone());
                    } else {
                        modified.push(sig.clone());
                    }
                }
            }
        }

        // Identify deleted items
        for sig in baseline {
            if !current_map.contains_key(sig.path.as_str()) {
                deleted.push(sig.clone());
            }
        }

        Self {
            baseline_count: baseline.len(),
            current_count: current.len(),
            added,
            modified,
            unchanged,
            deleted,
        }
    }

    /// Number of items that require processing (Added + Modified).
    #[must_use]
    pub fn to_process_count(&self) -> usize {
        self.added.len() + self.modified.len()
    }

    /// Number of unchanged items eligible for reuse.
    #[must_use]
    pub fn reusable_count(&self) -> usize {
        self.unchanged.len()
    }

    /// Total bytes of unchanged items saved from processing.
    #[must_use]
    pub fn reusable_bytes(&self) -> u64 {
        self.unchanged.iter().map(|s| s.byte_size).sum()
    }

    /// Total bytes of items requiring processing.
    #[must_use]
    pub fn to_process_bytes(&self) -> u64 {
        self.added
            .iter()
            .chain(self.modified.iter())
            .map(|s| s.byte_size)
            .sum()
    }

    /// Unique content hashes that can be reused directly from cache.
    #[must_use]
    pub fn reusable_hashes(&self) -> HashSet<String> {
        self.unchanged
            .iter()
            .map(|s| s.content_hash.clone())
            .collect()
    }

    /// Mathematical conservation invariant:
    /// $\text{current\_count} == \text{unchanged} + \text{added} + \text{modified}$.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.current_count == self.unchanged.len() + self.added.len() + self.modified.len()
    }
}

/// Item to be processed or reused during an incremental execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementalCandidate<T> {
    /// Sequential index within the submission.
    pub item_index: usize,
    /// Target path or identifier.
    pub path: String,
    /// Cryptographic signature of this candidate.
    pub signature: FileSignature,
    /// Task-specific payload.
    pub payload: T,
}

/// An individual item reused directly from prior cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReusedItem<R> {
    /// Original item index.
    pub item_index: usize,
    /// Path of the resource.
    pub path: String,
    /// Content hash.
    pub content_hash: String,
    /// Byte size.
    pub byte_size: u64,
    /// Retrieved cached result.
    pub result: R,
}

/// Execution report summarizing an incremental conversion run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncrementalExecutionReport<R> {
    /// Batch identifier.
    pub batch_id: BatchId,
    /// Total candidates evaluated.
    pub total_candidates: usize,
    /// Items reused directly from cache without re-processing.
    pub reused_count: usize,
    /// Items newly executed or re-processed.
    pub processed_count: usize,
    /// Items that failed during processing.
    pub failed_count: usize,
    /// Items deleted compared to baseline.
    pub deleted_count: usize,
    /// Bytes reused without network transfer or recomputation.
    pub reused_bytes: u64,
    /// Bytes processed through the execution engine.
    pub processed_bytes: u64,
    /// Cache hit ratio ($0.0$ to $1.0$).
    pub cache_hit_ratio: f64,
    /// Total execution duration in milliseconds.
    pub elapsed_ms: u64,
    /// All output results (both reused and processed), sorted deterministically by item index.
    pub all_results: Vec<R>,
    /// Item-level failures as `(path, error_message)`.
    pub failures: Vec<(String, String)>,
}

impl<R> IncrementalExecutionReport<R> {
    /// Conservation law verification:
    /// $\text{total\_candidates} == \text{reused\_count} + \text{processed\_count} + \text{failed\_count}$.
    #[must_use]
    pub fn conservation_verified(&self) -> bool {
        self.total_candidates == self.reused_count + self.processed_count + self.failed_count
    }

    /// Returns `true` if all candidates succeeded or were reused with zero failures.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.failed_count == 0
    }
}

/// Coordinator orchestrating incremental changed-file processing.
#[derive(Debug, Clone)]
pub struct IncrementalJobCoordinator {
    executor: BoundedParallelExecutor,
}

impl IncrementalJobCoordinator {
    /// Constructs a new `IncrementalJobCoordinator` with the provided bounded parallel executor.
    #[must_use]
    pub fn new(executor: BoundedParallelExecutor) -> Self {
        Self { executor }
    }

    /// Returns a reference to the inner parallel executor.
    #[must_use]
    pub fn executor(&self) -> &BoundedParallelExecutor {
        &self.executor
    }

    /// Plans and executes an incremental batch job:
    /// 1. Computes diff between baseline signatures and candidates.
    /// 2. Bypasses execution for unchanged items by invoking `reuse_resolver`.
    /// 3. Concurrently processes added/modified items through the bounded parallel executor.
    /// 4. Merges and sorts all results, returning a verified [`IncrementalExecutionReport`].
    ///
    /// # Errors
    ///
    /// Returns [`JobEngineError`] if batch setup or execution fails.
    #[instrument(skip(self, candidates, baseline, processor, reuse_resolver))]
    pub async fn execute_incremental<T, R, FP, FR, FutP, FutR>(
        &self,
        batch_id: BatchId,
        candidates: Vec<IncrementalCandidate<T>>,
        baseline: &[FileSignature],
        processor: FP,
        reuse_resolver: FR,
    ) -> Result<IncrementalExecutionReport<R>, JobEngineError>
    where
        T: Send + Sync + Clone + 'static,
        R: Send + Sync + Clone + 'static,
        FP: Fn(BatchItemTask<T>) -> FutP + Send + Sync + Clone + 'static,
        FutP: Future<Output = Result<R, String>> + Send + 'static,
        FR: Fn(&FileSignature) -> FutR + Send + Sync + 'static,
        FutR: Future<Output = Result<R, String>> + Send + 'static,
    {
        let start_time = Instant::now();
        let total_candidates = candidates.len();

        // 1. Extract current signatures
        let current_signatures: Vec<FileSignature> =
            candidates.iter().map(|c| c.signature.clone()).collect();

        // 2. Compute structural diff
        let diff = ChangedFileDiff::from_manifests(baseline, &current_signatures);
        debug!(
            batch_id = %batch_id,
            added = diff.added.len(),
            modified = diff.modified.len(),
            unchanged = diff.unchanged.len(),
            deleted = diff.deleted.len(),
            "Incremental diff calculated"
        );

        let unchanged_paths: HashSet<&str> =
            diff.unchanged.iter().map(|s| s.path.as_str()).collect();

        let mut to_process_tasks = Vec::new();
        let mut reused_items = Vec::new();
        let mut failures = Vec::new();

        let mut reused_bytes = 0u64;
        let mut processed_bytes = 0u64;

        // 3. Partition candidates
        for candidate in candidates {
            if unchanged_paths.contains(candidate.path.as_str()) {
                // Resolve reused item from cache
                match reuse_resolver(&candidate.signature).await {
                    Ok(cached_res) => {
                        reused_bytes += candidate.signature.byte_size;
                        reused_items.push((candidate.item_index, cached_res));
                    }
                    Err(err) => {
                        failures.push((
                            candidate.path.clone(),
                            format!("Reuse resolution failed: {err}"),
                        ));
                    }
                }
            } else {
                // Must be executed
                processed_bytes += candidate.signature.byte_size;
                let task = BatchItemTask::new(
                    candidate.item_index,
                    JobId::new(),
                    candidate.path.clone(),
                    Some(candidate.signature.content_hash.clone()),
                    candidate.payload,
                );
                to_process_tasks.push(task);
            }
        }

        // 4. Concurrently execute only added/modified items within bounded concurrency
        let mut newly_processed_items = Vec::new();
        if !to_process_tasks.is_empty() {
            info!(
                batch_id = %batch_id,
                tasks_to_process = to_process_tasks.len(),
                reused_tasks = reused_items.len(),
                "Executing incremental delta tasks"
            );

            let summary = self
                .executor
                .execute_batch(batch_id, to_process_tasks, processor, None, None)
                .await?;

            for item_res in summary.item_results {
                match item_res.status {
                    BatchItemStatusOutcome::Succeeded => {
                        if let Some(res) = item_res.value {
                            newly_processed_items.push((item_res.item_index, res));
                        }
                    }
                    BatchItemStatusOutcome::Failed { error } => {
                        failures.push((item_res.url, error));
                    }
                    BatchItemStatusOutcome::TimedOut { .. } => {
                        failures.push((item_res.url, "Item execution timed out".to_string()));
                    }
                    BatchItemStatusOutcome::Cancelled { .. } => {
                        failures.push((item_res.url, "Item execution cancelled".to_string()));
                    }
                }
            }
        }

        // 5. Merge all results deterministically sorted by item_index
        let mut merged: Vec<(usize, R)> =
            Vec::with_capacity(reused_items.len() + newly_processed_items.len());
        merged.extend(reused_items);
        merged.extend(newly_processed_items);
        merged.sort_by_key(|(idx, _)| *idx);

        let all_results: Vec<R> = merged.into_iter().map(|(_, r)| r).collect();

        let reused_count = diff.unchanged.len()
            - failures
                .iter()
                .filter(|(p, _)| unchanged_paths.contains(p.as_str()))
                .count();
        let processed_count = all_results.len().saturating_sub(reused_count);
        let failed_count = failures.len();
        let cache_hit_ratio = if total_candidates == 0 {
            0.0
        } else {
            reused_count as f64 / total_candidates as f64
        };

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(IncrementalExecutionReport {
            batch_id,
            total_candidates,
            reused_count,
            processed_count,
            failed_count,
            deleted_count: diff.deleted.len(),
            reused_bytes,
            processed_bytes,
            cache_hit_ratio,
            elapsed_ms,
            all_results,
            failures,
        })
    }
}
