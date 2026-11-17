//! Path parameter models and URI segment extractors.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_core::{CoreError, JobId, Result, Validatable};

use super::jobs::VALID_EXPORT_FORMATS;

/// Path parameters for job-specific endpoints (`/api/v1/jobs/{job_id}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobPathParams {
    /// Strongly-typed UUID v7 job identifier.
    pub job_id: JobId,
}

impl Validatable for JobPathParams {
    fn validate(&self) -> Result<()> {
        // JobId is strongly typed and already guaranteed to be a valid UUID v7.
        Ok(())
    }
}

/// Path parameters for artifact retrieval (`/api/v1/jobs/{job_id}/artifacts/{format}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactPathParams {
    /// Target conversion job identifier.
    pub job_id: JobId,
    /// Target export artifact format (e.g. `markdown`, `json`, `zip`).
    pub format: String,
}

impl Validatable for ArtifactPathParams {
    fn validate(&self) -> Result<()> {
        let lower = self.format.trim().to_ascii_lowercase();
        if !VALID_EXPORT_FORMATS.contains(&lower.as_str()) {
            return Err(CoreError::Validation {
                field: "format".to_string(),
                reason: format!(
                    "Unsupported artifact format '{}': must be one of {:?}",
                    self.format, VALID_EXPORT_FORMATS
                ),
            });
        }
        Ok(())
    }
}
