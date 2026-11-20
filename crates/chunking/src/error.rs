//! # Chunking Error Types
//!
//! Error definitions for document and section chunking operations.

use thiserror::Error;

/// Result alias for chunking operations.
pub type ChunkResult<T> = Result<T, ChunkingError>;

/// Errors that can occur during document and section chunking.
#[derive(Debug, Error)]
pub enum ChunkingError {
    /// Provided chunking configuration contains invalid parameters.
    #[error("invalid chunking configuration: {0}")]
    InvalidConfig(String),

    /// The target document or section has no content to chunk.
    #[error("empty document: document '{0}' contains no chunkable sections or content")]
    EmptyDocument(String),

    /// Tokenizer configuration or token counting failure.
    #[error("tokenizer error: {0}")]
    TokenizerError(String),

    /// Internal error occurred while partitioning or splitting text.
    #[error("chunking failed for section '{section_id}': {message}")]
    ProcessingError {
        /// Identifier of the section that failed to chunk.
        section_id: String,
        /// Description of the error condition.
        message: String,
    },

    /// Export or serialization failure.
    #[error("chunk export error: {0}")]
    ExportError(String),

    /// Schema validation failed during export.
    #[error("vector schema validation error: {0}")]
    ValidationError(String),

    /// I/O error during streaming export.
    #[error("I/O error during chunk export: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}
