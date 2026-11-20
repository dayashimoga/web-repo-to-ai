//! # Error Types for Structured Document Extraction
//!
//! Strongly-typed errors for format detection, parsing, DoS limits, and CDM conversion.

use thiserror::Error;

/// Specialized result type for structured extractor operations.
pub type Result<T> = std::result::Result<T, StructuredError>;

/// Errors that can occur during structured document extraction.
#[derive(Debug, Error)]
pub enum StructuredError {
    /// Input exceeds configured safety limits (e.g. byte size, nesting depth).
    #[error("Safety limit exceeded: {0}")]
    LimitExceeded(String),

    /// Content cannot be recognized as a supported structured document format.
    #[error("Unrecognized or unsupported structured format: {0}")]
    UnrecognizedFormat(String),

    /// Failed to parse JSON content.
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Failed to parse YAML content.
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml_ng::Error),

    /// Malformed OpenAPI specification structure.
    #[error("Malformed OpenAPI specification: {0}")]
    InvalidOpenApi(String),

    /// Malformed JSON Schema specification structure.
    #[error("Malformed JSON Schema: {0}")]
    InvalidJsonSchema(String),

    /// Canonical Context Document Model (CDM) validation or builder error.
    #[error("CDM construction error: {0}")]
    CdmError(#[from] web_repo_to_ai_cdm::error::CdmError),

    /// I/O or input conversion error.
    #[error("I/O error: {0}")]
    IoError(String),
}
