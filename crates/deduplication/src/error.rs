//! # Deduplication Errors
//!
//! Strongly-typed error definitions for exact and semantic document deduplication operations.

use thiserror::Error;

/// Error conditions encountered during deduplication processing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DeduplicationError {
    /// Content could not be deduplicated due to processing error.
    #[error("Deduplication processing failed: {0}")]
    ProcessingFailed(String),

    /// Invalid configuration parameters provided.
    #[error("Invalid deduplication configuration: {0}")]
    InvalidConfiguration(String),

    /// General internal deduplication error.
    #[error("Internal deduplication error: {0}")]
    Internal(String),
}

/// Specialized [`Result`] type for deduplication operations.
pub type Result<T> = std::result::Result<T, DeduplicationError>;
