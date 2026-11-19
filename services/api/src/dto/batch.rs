//! Data Transfer Objects for multi-URL batch conversion submissions and lifecycle tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;
use web_repo_to_ai_core::{BatchId, CoreError, JobId, Result, Validatable};
use web_repo_to_ai_storage::batch::{
    BatchItemRecord, BatchItemStatusKind, BatchJobStatus, BatchRecord,
};

use super::jobs::{JobStatus, VALID_EXPORT_FORMATS};

/// Maximum allowable distinct target URLs in a single batch submission.
pub const MAX_BATCH_URLS: usize = 100;

/// Minimum required target URLs in a batch submission.
pub const MIN_BATCH_URLS: usize = 1;

/// Maximum allowed concurrent item worker parallelism.
pub const MAX_BATCH_PARALLELISM: usize = 20;

/// Default concurrent item worker parallelism.
pub const DEFAULT_BATCH_PARALLELISM: usize = 5;

/// Valid context profile identifiers supported across the engine.
pub const VALID_CONTEXT_PROFILES: &[&str] = &[
    "general_ai",
    "code_review",
    "security_review",
    "debugging_and_documentation",
    "debugging",
];

/// Specification for an individual target URL within a batch conversion request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemRequest {
    /// Target web URL or GitHub repository identifier.
    pub url: String,
    /// Optional caller-supplied identifier for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// Optional Git branch, tag, or commit reference override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    /// Optional path inclusion/exclusion patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_patterns: Option<Vec<String>>,
    /// Custom token budget ceiling override (1,000 to 2,000,000 tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<usize>,
    /// Target LLM model identifier override (e.g. "gpt-4o", "claude-3-5-sonnet").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_model: Option<String>,
    /// Formats to generate for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_formats: Option<Vec<String>>,
    /// Specialized context optimization profile override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_profile: Option<String>,
}

impl Validatable for BatchItemRequest {
    fn validate(&self) -> Result<()> {
        let trimmed = self.url.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Validation {
                field: "url".to_string(),
                reason: "Batch item target URL cannot be empty or whitespace".to_string(),
            });
        }

        if trimmed.len() > 2048 {
            return Err(CoreError::Validation {
                field: "url".to_string(),
                reason: format!(
                    "Target URL exceeds 2,048 characters maximum limit (length: {})",
                    trimmed.len()
                ),
            });
        }

        match Url::parse(trimmed) {
            Ok(parsed) => {
                let scheme = parsed.scheme().to_ascii_lowercase();
                if scheme != "http" && scheme != "https" {
                    return Err(CoreError::Validation {
                        field: "url".to_string(),
                        reason: format!(
                            "Unsupported URL scheme '{scheme}' for '{trimmed}': only http and https are permitted"
                        ),
                    });
                }
            }
            Err(e) => {
                return Err(CoreError::Validation {
                    field: "url".to_string(),
                    reason: format!("Malformed target URL '{trimmed}': {e}"),
                });
            }
        }

        if let Some(ref custom_id) = self.custom_id {
            if custom_id.trim().is_empty() || custom_id.len() > 256 {
                return Err(CoreError::Validation {
                    field: "custom_id".to_string(),
                    reason: "Custom identifier must be between 1 and 256 characters".to_string(),
                });
            }
        }

        if let Some(budget) = self.token_budget {
            if !(1_000..=2_000_000).contains(&budget) {
                return Err(CoreError::Validation {
                    field: "token_budget".to_string(),
                    reason: format!(
                        "Token budget {budget} out of bounds: must be between 1,000 and 2,000,000 tokens"
                    ),
                });
            }
        }

        if let Some(ref formats) = self.export_formats {
            if formats.is_empty() {
                return Err(CoreError::Validation {
                    field: "export_formats".to_string(),
                    reason: "Export formats array cannot be empty when specified".to_string(),
                });
            }
            for fmt in formats {
                let lower = fmt.trim().to_ascii_lowercase();
                if !VALID_EXPORT_FORMATS.contains(&lower.as_str()) {
                    return Err(CoreError::Validation {
                        field: "export_formats".to_string(),
                        reason: format!(
                            "Unsupported export format '{fmt}': must be one of {VALID_EXPORT_FORMATS:?}"
                        ),
                    });
                }
            }
        }

        if let Some(ref profile) = self.context_profile {
            let lower = profile.trim().to_ascii_lowercase();
            if !VALID_CONTEXT_PROFILES.contains(&lower.as_str()) {
                return Err(CoreError::Validation {
                    field: "context_profile".to_string(),
                    reason: format!(
                        "Unsupported context profile '{profile}': must be one of {VALID_CONTEXT_PROFILES:?}"
                    ),
                });
            }
        }

        Ok(())
    }
}

/// Request payload to initiate a new multi-URL batch conversion job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreateBatchJobRequest {
    /// Itemized target URLs with optional per-item parameter overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<BatchItemRequest>>,
    /// Convenience shorthand: list of URL strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    /// Optional human-readable batch job name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Global default Git branch or tag for repository sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_git_ref: Option<String>,
    /// Global default token budget ceiling (1,000 to 2,000,000 tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_token_budget: Option<usize>,
    /// Global default target LLM model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_target_model: Option<String>,
    /// Global default export formats (defaults to `["markdown"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_export_formats: Option<Vec<String>>,
    /// Global default context optimization profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_context_profile: Option<String>,
    /// Whether to perform exact file deduplication across the batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicate_exact: Option<bool>,
    /// Whether to perform semantic near-duplicate suppression across the batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicate_semantic: Option<bool>,
    /// Whether to strip comments during normalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,
    /// Maximum concurrent worker tasks for this batch (1 to 20, defaults to 5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallelism: Option<usize>,
    /// Whether to abort remaining batch items if any individual item fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_on_error: Option<bool>,
}

impl CreateBatchJobRequest {
    /// Resolves both `items` and shorthand `urls` into a unified list of [`BatchItemRequest`]s.
    ///
    /// Applies global defaults and deduplicates identical URLs within the batch,
    /// preserving the first occurrence.
    #[must_use]
    pub fn effective_items(&self) -> Vec<BatchItemRequest> {
        let mut result = Vec::new();
        let mut seen_urls = HashSet::new();

        // 1. Process structured items first
        if let Some(ref items) = self.items {
            for item in items {
                let trimmed_url = item.url.trim().to_string();
                if seen_urls.insert(trimmed_url.to_ascii_lowercase()) {
                    let mut resolved = item.clone();
                    resolved.url = trimmed_url;
                    if resolved.git_ref.is_none() {
                        resolved.git_ref = self.default_git_ref.clone();
                    }
                    if resolved.token_budget.is_none() {
                        resolved.token_budget = self.default_token_budget;
                    }
                    if resolved.target_model.is_none() {
                        resolved.target_model = self.default_target_model.clone();
                    }
                    if resolved.export_formats.is_none() {
                        resolved.export_formats = self.default_export_formats.clone();
                    }
                    if resolved.context_profile.is_none() {
                        resolved.context_profile = self.default_context_profile.clone();
                    }
                    result.push(resolved);
                }
            }
        }

        // 2. Process shorthand URLs
        if let Some(ref urls) = self.urls {
            for url in urls {
                let trimmed_url = url.trim().to_string();
                if seen_urls.insert(trimmed_url.to_ascii_lowercase()) {
                    result.push(BatchItemRequest {
                        url: trimmed_url,
                        custom_id: None,
                        git_ref: self.default_git_ref.clone(),
                        path_patterns: None,
                        token_budget: self.default_token_budget,
                        target_model: self.default_target_model.clone(),
                        export_formats: self.default_export_formats.clone(),
                        context_profile: self.default_context_profile.clone(),
                    });
                }
            }
        }

        result
    }
}

impl Validatable for CreateBatchJobRequest {
    fn validate(&self) -> Result<()> {
        let items = self.effective_items();

        if items.is_empty() {
            return Err(CoreError::Validation {
                field: "items".to_string(),
                reason: "Must provide at least one target URL in 'items' or 'urls' (received empty batch)"
                    .to_string(),
            });
        }

        if items.len() > MAX_BATCH_URLS {
            return Err(CoreError::Validation {
                field: "items".to_string(),
                reason: format!(
                    "Maximum allowable URLs per batch is {} (received {})",
                    MAX_BATCH_URLS,
                    items.len()
                ),
            });
        }

        if let Some(parallelism) = self.max_parallelism {
            if !(1..=MAX_BATCH_PARALLELISM).contains(&parallelism) {
                return Err(CoreError::Validation {
                    field: "max_parallelism".to_string(),
                    reason: format!(
                        "Requested max parallelism {parallelism} out of bounds: must be between 1 and {MAX_BATCH_PARALLELISM}"
                    ),
                });
            }
        }

        if let Some(budget) = self.default_token_budget {
            if !(1_000..=2_000_000).contains(&budget) {
                return Err(CoreError::Validation {
                    field: "default_token_budget".to_string(),
                    reason: format!(
                        "Default token budget {budget} out of bounds: must be between 1,000 and 2,000,000 tokens"
                    ),
                });
            }
        }

        if let Some(ref formats) = self.default_export_formats {
            if formats.is_empty() {
                return Err(CoreError::Validation {
                    field: "default_export_formats".to_string(),
                    reason: "Default export formats array cannot be empty when specified"
                        .to_string(),
                });
            }
            for fmt in formats {
                let lower = fmt.trim().to_ascii_lowercase();
                if !VALID_EXPORT_FORMATS.contains(&lower.as_str()) {
                    return Err(CoreError::Validation {
                        field: "default_export_formats".to_string(),
                        reason: format!(
                            "Unsupported export format '{fmt}': must be one of {VALID_EXPORT_FORMATS:?}"
                        ),
                    });
                }
            }
        }

        if let Some(ref profile) = self.default_context_profile {
            let lower = profile.trim().to_ascii_lowercase();
            if !VALID_CONTEXT_PROFILES.contains(&lower.as_str()) {
                return Err(CoreError::Validation {
                    field: "default_context_profile".to_string(),
                    reason: format!(
                        "Unsupported context profile '{profile}': must be one of {VALID_CONTEXT_PROFILES:?}"
                    ),
                });
            }
        }

        // Validate each resolved item
        for (idx, item) in items.iter().enumerate() {
            item.validate().map_err(|e| match e {
                CoreError::Validation { field, reason } => CoreError::Validation {
                    field: format!("items[{idx}].{field}"),
                    reason,
                },
                other => other,
            })?;
        }

        Ok(())
    }
}

/// Execution state of a multi-URL batch conversion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Batch job created and items are queued for worker execution.
    #[default]
    Queued,
    /// Batch is actively processing member URL conversion tasks.
    Processing,
    /// All batch member tasks completed successfully.
    Completed,
    /// Some member tasks completed, but one or more failed.
    PartialFailure,
    /// All member tasks failed or batch initialization failed.
    Failed,
    /// Batch job was explicitly cancelled by the user.
    Cancelled,
}

impl BatchStatus {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::PartialFailure => "partial_failure",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Status descriptor for an individual URL item within a batch job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemStatus {
    /// Zero-based index within the submitted batch.
    pub item_index: usize,
    /// Strongly-typed UUID v7 conversion job identifier assigned to this item.
    pub job_id: JobId,
    /// Target URL assigned to this item.
    pub url: String,
    /// Optional caller-supplied correlation identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    /// Current lifecycle status of this item.
    pub status: JobStatus,
    /// Machine-readable error code if item execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Detailed error message if item execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Execution duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// UTC timestamp when this item was queued.
    pub created_at: DateTime<Utc>,
    /// UTC timestamp when this item reached terminal state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<BatchItemRecord> for BatchItemStatus {
    fn from(rec: BatchItemRecord) -> Self {
        let status = match rec.status {
            BatchItemStatusKind::Queued => JobStatus::Queued,
            BatchItemStatusKind::Processing => JobStatus::Processing,
            BatchItemStatusKind::Completed => JobStatus::Completed,
            BatchItemStatusKind::Failed | BatchItemStatusKind::TimedOut => JobStatus::Failed,
            BatchItemStatusKind::Cancelled => JobStatus::Cancelled,
        };
        Self {
            item_index: rec.item_index,
            job_id: rec.job_id,
            url: rec.url,
            custom_id: rec.custom_id,
            status,
            error_code: rec.error_code,
            error_message: rec.error_message,
            duration_ms: rec.duration_ms,
            created_at: rec.created_at,
            completed_at: rec.completed_at,
        }
    }
}

/// Progress metrics for an executing or completed batch job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Total itemized URLs in the batch.
    pub total_items: usize,
    /// Number of items completed successfully.
    pub completed_items: usize,
    /// Number of items that encountered an error.
    pub failed_items: usize,
    /// Number of items currently processing in parallel.
    pub processing_items: usize,
    /// Number of items waiting in queue.
    pub queued_items: usize,
    /// Overall percentage completed (0.0 to 100.0).
    pub percentage: f64,
}

impl BatchProgress {
    /// Calculates progress statistics from a slice of [`BatchItemStatus`].
    #[must_use]
    pub fn from_items(items: &[BatchItemStatus]) -> Self {
        let total = items.len();
        let mut completed = 0;
        let mut failed = 0;
        let mut processing = 0;
        let mut queued = 0;

        for item in items {
            match item.status {
                JobStatus::Completed => completed += 1,
                JobStatus::Failed => failed += 1,
                JobStatus::Processing => processing += 1,
                JobStatus::Queued | JobStatus::Cancelled => queued += 1,
            }
        }

        let percentage = if total > 0 {
            ((completed + failed) as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };

        Self {
            total_items: total,
            completed_items: completed,
            failed_items: failed,
            processing_items: processing,
            queued_items: queued,
            percentage,
        }
    }
}

/// Response payload returned upon successful batch submission (`202 Accepted`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBatchJobResponse {
    /// Strongly-typed UUID v7 batch job identifier.
    pub batch_id: BatchId,
    /// Optional user-supplied batch title or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Initial lifecycle status (`queued`).
    pub status: BatchStatus,
    /// Total number of distinct URLs scheduled for processing.
    pub total_items: usize,
    /// Number of items currently queued.
    pub queued_items: usize,
    /// Bounded concurrency limit assigned to this batch.
    pub max_parallelism: usize,
    /// Initial status records for each assigned URL item.
    pub items: Vec<BatchItemStatus>,
    /// UTC timestamp of batch creation.
    pub created_at: DateTime<Utc>,
}

/// Detailed response descriptor for querying batch status (`GET /api/v1/jobs/batch/{batch_id}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchJobResponse {
    /// Strongly-typed UUID v7 batch job identifier.
    pub batch_id: BatchId,
    /// Optional user-supplied batch title or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Current lifecycle state of the batch.
    pub status: BatchStatus,
    /// Quantitative completion progress across member tasks.
    pub progress: BatchProgress,
    /// Bounded concurrency limit.
    pub max_parallelism: usize,
    /// Itemized status records for each URL in the batch.
    pub items: Vec<BatchItemStatus>,
    /// UTC timestamp of batch creation.
    pub created_at: DateTime<Utc>,
    /// UTC timestamp of last status update.
    pub updated_at: DateTime<Utc>,
    /// UTC timestamp of final completion, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Path parameter extractor for batch-specific endpoints (`/api/v1/jobs/batch/{batch_id}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPathParams {
    /// Strongly-typed UUID v7 batch identifier.
    pub batch_id: BatchId,
}

impl Validatable for BatchPathParams {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Path parameter extractor for individual batch item endpoints (`/api/v1/jobs/batch/{batch_id}/items/{item_index}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemPathParams {
    /// Strongly-typed UUID v7 batch identifier.
    pub batch_id: BatchId,
    /// Zero-based sequence index within the batch.
    pub item_index: usize,
}

impl Validatable for BatchItemPathParams {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Response descriptor for querying the full item list of a batch (`GET /api/v1/jobs/batch/{batch_id}/items`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchItemsListResponse {
    /// Strongly-typed UUID v7 batch identifier.
    pub batch_id: BatchId,
    /// Total count of items in the batch.
    pub total_items: usize,
    /// Itemized status records.
    pub items: Vec<BatchItemStatus>,
}

/// Summary descriptor for an individual batch within a list query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchSummaryItem {
    /// Strongly-typed UUID v7 batch identifier.
    pub batch_id: BatchId,
    /// Optional user-supplied batch title or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// High-level lifecycle state.
    pub status: BatchStatus,
    /// Total URLs in batch.
    pub total_items: usize,
    /// Successfully completed items count.
    pub completed_items: usize,
    /// Failed or timed-out items count.
    pub failed_items: usize,
    /// Items currently processing in parallel.
    pub processing_items: usize,
    /// Items currently queued.
    pub queued_items: usize,
    /// Bounded concurrency ceiling.
    pub max_parallelism: usize,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp, if finished.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<BatchRecord> for BatchSummaryItem {
    fn from(rec: BatchRecord) -> Self {
        let status = match rec.status {
            BatchJobStatus::Queued => BatchStatus::Queued,
            BatchJobStatus::Processing => BatchStatus::Processing,
            BatchJobStatus::Completed => BatchStatus::Completed,
            BatchJobStatus::PartialFailure => BatchStatus::PartialFailure,
            BatchJobStatus::Failed => BatchStatus::Failed,
            BatchJobStatus::Cancelled => BatchStatus::Cancelled,
        };
        Self {
            batch_id: rec.id,
            name: rec.name,
            status,
            total_items: rec.total_items,
            completed_items: rec.completed_items,
            failed_items: rec.failed_items,
            processing_items: rec.processing_items,
            queued_items: rec.queued_items,
            max_parallelism: rec.max_parallelism,
            created_at: rec.created_at,
            updated_at: rec.updated_at,
            completed_at: rec.completed_at,
        }
    }
}

/// Response payload for listing batch jobs (`GET /api/v1/jobs/batch`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListBatchesResponse {
    /// Paged batch summary items.
    pub batches: Vec<BatchSummaryItem>,
    /// Total count of batches returned in this page.
    pub count: usize,
}
