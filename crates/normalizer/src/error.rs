//! # Content Normalizer Errors
//!
//! Strongly-typed error definitions for document and section normalization operations.

use thiserror::Error;

/// Error conditions encountered during document or section normalization.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NormalizerError {
    /// Content could not be normalized due to invalid or malformed structure.
    #[error("Normalization processing failed: {0}")]
    ProcessingFailed(String),

    /// Character encoding repair encountered an unrecoverable sequence.
    #[error("Character encoding error: {0}")]
    EncodingError(String),

    /// Table parsing or rectangularization failed.
    #[error("Markdown table normalization error: {0}")]
    TableError(String),

    /// General internal normalizer error.
    #[error("Internal normalizer error: {0}")]
    Internal(String),
}

/// Specialized [`Result`] type for content normalizer operations.
pub type Result<T> = std::result::Result<T, NormalizerError>;
