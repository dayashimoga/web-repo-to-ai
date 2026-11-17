//! Data Transfer Objects for conversion job creation and status tracking.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;
use web_repo_to_ai_core::{CoreError, JobId, Result, Validatable};

/// Allowed export artifact formats.
pub const VALID_EXPORT_FORMATS: &[&str] = &[
    "markdown", "txt", "json", "html", "jsonl", "zip", "manifest",
];

/// Execution state of a conversion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job created and waiting for background worker lease.
    #[default]
    Queued,
    /// Actively extracting, normalizing, and budgeting content.
    Processing,
    /// Successfully processed and ready for download.
    Completed,
    /// Failed due to extraction, SSRF, budget, or system error.
    Failed,
    /// Cancelled upon user or administrator request.
    Cancelled,
}

/// Request payload to create a new repository or URL conversion job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// One or more target URLs or repository identifiers.
    pub sources: Vec<String>,
    /// Target Git branch, tag, or commit reference for repository sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    /// Optional file glob inclusion/exclusion patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_patterns: Option<Vec<String>>,
    /// Custom token budget ceiling (1,000 to 2,000,000 tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<usize>,
    /// Target LLM model identifier for budgeting (e.g. "claude-3-5-sonnet").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_model: Option<String>,
    /// Formats to generate (defaults to `["markdown"]` if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_formats: Option<Vec<String>>,
    /// Whether to perform exact file-level content deduplication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicate_exact: Option<bool>,
    /// Whether to perform semantic near-duplicate suppression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicate_semantic: Option<bool>,
    /// Whether to strip code comments during normalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,
}

impl Validatable for CreateJobRequest {
    fn validate(&self) -> Result<()> {
        // 1. Sources validation
        if self.sources.is_empty() {
            return Err(CoreError::Validation {
                field: "sources".to_string(),
                reason: "Must provide at least one source URL (received empty array)".to_string(),
            });
        }

        if self.sources.len() > 50 {
            return Err(CoreError::Validation {
                field: "sources".to_string(),
                reason: format!(
                    "Maximum allowable sources per job is 50 (received {})",
                    self.sources.len()
                ),
            });
        }

        for (idx, source) in self.sources.iter().enumerate() {
            let trimmed = source.trim();
            if trimmed.is_empty() {
                return Err(CoreError::Validation {
                    field: "sources".to_string(),
                    reason: format!("Source at index {idx} cannot be empty or whitespace"),
                });
            }

            if trimmed.len() > 2048 {
                return Err(CoreError::Validation {
                    field: "sources".to_string(),
                    reason: format!(
                        "Source at index {idx} exceeds maximum length of 2048 characters"
                    ),
                });
            }

            let parsed = Url::parse(trimmed).map_err(|err| CoreError::Validation {
                field: "sources".to_string(),
                reason: format!("Source at index {idx} ('{trimmed}') is malformed: {err}"),
            })?;

            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err(CoreError::UnsupportedUrlScheme(format!(
                    "Source at index {idx} has unsupported scheme '{}': only http and https are allowed",
                    parsed.scheme()
                )));
            }
        }

        // 2. Git ref validation
        if let Some(ref git_ref) = self.git_ref {
            let trimmed = git_ref.trim();
            if trimmed.is_empty() || trimmed.len() > 255 {
                return Err(CoreError::Validation {
                    field: "git_ref".to_string(),
                    reason: "Git reference must be between 1 and 255 characters".to_string(),
                });
            }

            if trimmed.contains("..")
                || trimmed.contains('~')
                || trimmed.contains('^')
                || trimmed.contains(':')
                || trimmed.contains('?')
                || trimmed.contains('*')
                || trimmed.contains('[')
                || trimmed.contains('\\')
                || trimmed
                    .chars()
                    .any(|c| c.is_ascii_whitespace() || c.is_ascii_control())
            {
                return Err(CoreError::Validation {
                    field: "git_ref".to_string(),
                    reason: format!("Git reference '{trimmed}' contains invalid characters"),
                });
            }
        }

        // 3. Path patterns validation
        if let Some(ref patterns) = self.path_patterns {
            if patterns.len() > 100 {
                return Err(CoreError::Validation {
                    field: "path_patterns".to_string(),
                    reason: format!(
                        "Maximum 100 path patterns allowed (received {})",
                        patterns.len()
                    ),
                });
            }

            for (idx, pattern) in patterns.iter().enumerate() {
                if pattern.trim().is_empty() || pattern.len() > 255 {
                    return Err(CoreError::Validation {
                        field: "path_patterns".to_string(),
                        reason: format!(
                            "Pattern at index {idx} must be between 1 and 255 characters"
                        ),
                    });
                }
                if pattern.contains('\0') {
                    return Err(CoreError::Validation {
                        field: "path_patterns".to_string(),
                        reason: format!("Pattern at index {idx} contains null byte"),
                    });
                }
            }
        }

        // 4. Token budget validation
        if let Some(budget) = self.token_budget {
            if !(1_000..=2_000_000).contains(&budget) {
                return Err(CoreError::Validation {
                    field: "token_budget".to_string(),
                    reason: format!(
                        "Token budget must be between 1,000 and 2,000,000 tokens (received {budget})"
                    ),
                });
            }
        }

        // 5. Target model validation
        if let Some(ref model) = self.target_model {
            let trimmed = model.trim();
            if trimmed.is_empty() || trimmed.len() > 100 {
                return Err(CoreError::Validation {
                    field: "target_model".to_string(),
                    reason: "Target model must be between 1 and 100 characters".to_string(),
                });
            }
            if !trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
            {
                return Err(CoreError::Validation {
                    field: "target_model".to_string(),
                    reason: format!(
                        "Target model '{trimmed}' contains invalid characters (only alphanumeric, '-', '_', '.' allowed)"
                    ),
                });
            }
        }

        // 6. Export formats validation
        if let Some(ref formats) = self.export_formats {
            if formats.is_empty() {
                return Err(CoreError::Validation {
                    field: "export_formats".to_string(),
                    reason:
                        "At least one export format must be specified when the array is present"
                            .to_string(),
                });
            }

            for format in formats {
                let fmt_lower = format.trim().to_ascii_lowercase();
                if !VALID_EXPORT_FORMATS.contains(&fmt_lower.as_str()) {
                    return Err(CoreError::Validation {
                        field: "export_formats".to_string(),
                        reason: format!(
                            "Unsupported export format '{format}': permitted formats are {:?}",
                            VALID_EXPORT_FORMATS
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Representation of a conversion job returned by API endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobResponse {
    /// Unique strongly-typed UUID v7 job identifier.
    pub job_id: JobId,
    /// Current job status.
    pub status: JobStatus,
    /// List of source URLs being processed.
    pub sources: Vec<String>,
    /// Effective token budget applied to this job.
    pub token_budget: usize,
    /// Target LLM model architecture for token counting.
    pub target_model: String,
    /// Configured export formats.
    pub export_formats: Vec<String>,
    /// UTC timestamp of job creation.
    pub created_at: String,
}

impl JobResponse {
    /// Creates a newly queued job response from a validated request and default configurations.
    #[must_use]
    pub fn new_queued(
        job_id: JobId,
        request: CreateJobRequest,
        default_model: &str,
        default_budget: usize,
    ) -> Self {
        let export_formats = request
            .export_formats
            .unwrap_or_else(|| vec!["markdown".to_string()]);
        let token_budget = request.token_budget.unwrap_or(default_budget);
        let target_model = request
            .target_model
            .unwrap_or_else(|| default_model.to_string());

        Self {
            job_id,
            status: JobStatus::Queued,
            sources: request.sources,
            token_budget,
            target_model,
            export_formats,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}
