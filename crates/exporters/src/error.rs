//! # Exporter Errors
//!
//! Strongly-typed error definitions for multi-format artifact generation.

use thiserror::Error;

/// Errors that can occur during document export.
#[derive(Debug, Error)]
pub enum ExporterError {
    /// An underlying I/O error occurred while writing output.
    #[error("I/O error during export: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization or deserialization error occurred.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// A formatting or serialization error occurred.
    #[error("Formatting error: {0}")]
    Formatting(String),

    /// The specified token budget was exceeded during export.
    #[error("Token budget of {budget} exceeded: current count {actual}")]
    BudgetExceeded {
        /// Maximum allowed token budget.
        budget: usize,
        /// Actual token count attempted.
        actual: usize,
    },

    /// The CDM document is structurally invalid or missing required elements.
    #[error("Invalid CDM document: {0}")]
    InvalidDocument(String),

    /// A checksum or manifest validation error occurred.
    #[error("Checksum validation error: {0}")]
    ChecksumValidation(String),

    /// Export exceeded maximum allowed package size in bytes.
    #[error(
        "Export package exceeded maximum allowed size of {max_bytes} bytes: current {actual_bytes}"
    )]
    PackageSizeExceeded {
        /// Maximum allowable bytes.
        max_bytes: u64,
        /// Actual byte count.
        actual_bytes: u64,
    },
}

/// Convenience result type for exporter operations.
pub type Result<T> = std::result::Result<T, ExporterError>;
