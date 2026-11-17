//! # CDM Error Taxonomy
//!
//! Strongly-typed error types for Canonical Context Document Model (CDM)
//! serialization, deserialization, parsing, validation, and payload bounding.

use thiserror::Error;
use web_repo_to_ai_core::CoreError;

/// Standard result alias for CDM operations.
pub type Result<T> = std::result::Result<T, CdmError>;

/// Error variants encountered during CDM serialization, deserialization, or validation.
#[derive(Debug, Error)]
pub enum CdmError {
    /// Failure during JSON serialization.
    #[error("CDM serialization error: {0}")]
    Serialization(#[source] serde_json::Error),

    /// Failure during JSON deserialization.
    #[error("CDM deserialization error: {0}")]
    Deserialization(#[source] serde_json::Error),

    /// Incoming JSON payload exceeds the maximum permitted memory ceiling.
    #[error("CDM payload size ({actual_bytes} bytes) exceeds maximum limit of {max_bytes} bytes")]
    PayloadTooLarge {
        /// Encountered byte count.
        actual_bytes: usize,
        /// Configured or default byte limit.
        max_bytes: usize,
    },

    /// Deserialized document schema version does not match supported major version.
    #[error("Unsupported CDM schema version '{found}'; supported version is '{supported}'")]
    UnsupportedSchemaVersion {
        /// Version string encountered in the payload.
        found: String,
        /// Expected/supported version string.
        supported: String,
    },

    /// Cryptographic hash mismatch indicating data corruption or tampering.
    #[error(
        "BLAKE3 hash mismatch for section '{section_id}': expected {expected}, actual {actual}"
    )]
    HashMismatch {
        /// Section ID with mismatched hash.
        section_id: String,
        /// Recorded digest.
        expected: String,
        /// Computed digest.
        actual: String,
    },

    /// CDM model invariant or structural validation failure.
    #[error("CDM validation failure: {0}")]
    Validation(String),

    /// Standard I/O error during streaming read or write.
    #[error("CDM I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<CdmError> for CoreError {
    fn from(err: CdmError) -> Self {
        match err {
            CdmError::Serialization(e) => Self::Serialization(e),
            CdmError::Deserialization(e) => Self::InvalidRequestSchema(e.to_string()),
            CdmError::PayloadTooLarge {
                actual_bytes,
                max_bytes,
            } => Self::PayloadTooLarge {
                actual_bytes: actual_bytes as u64,
                max_bytes: max_bytes as u64,
            },
            CdmError::UnsupportedSchemaVersion { found, supported } => Self::Validation {
                field: "schema_version".to_string(),
                reason: format!("unsupported version '{found}', expected '{supported}'"),
            },
            CdmError::HashMismatch { section_id, .. } => Self::Validation {
                field: format!("sections[{section_id}].content_hash"),
                reason: "cryptographic BLAKE3 digest verification failed".to_string(),
            },
            CdmError::Validation(reason) => Self::Validation {
                field: "cdm_document".to_string(),
                reason,
            },
            CdmError::Io(e) => Self::Io(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdm_error_core_error_conversion() {
        let err = CdmError::PayloadTooLarge {
            actual_bytes: 1000,
            max_bytes: 500,
        };
        let core: CoreError = err.into();
        assert_eq!(
            core.error_code(),
            web_repo_to_ai_core::ErrorCode::PayloadTooLarge
        );

        let v_err = CdmError::UnsupportedSchemaVersion {
            found: "2.0.0".to_string(),
            supported: "1.0.0".to_string(),
        };
        let core_v: CoreError = v_err.into();
        assert_eq!(
            core_v.error_code(),
            web_repo_to_ai_core::ErrorCode::ValidationError
        );
    }
}
