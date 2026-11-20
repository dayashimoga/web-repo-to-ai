//! # OCR Extractor Error Definitions
//!
//! Structured error types for Tesseract process execution, TSV parsing,
//! image validation, and resource limits.

use thiserror::Error;

/// Error types occurring during OCR processing.
#[derive(Debug, Error)]
pub enum OcrError {
    /// Tesseract CLI binary could not be found or executed.
    #[error("Tesseract binary not found at '{path}': {source}")]
    TesseractNotFound {
        /// Configured binary path attempted.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The OCR adapter is unavailable on the host system.
    #[error("OCR engine is unavailable on system: {0}")]
    AdapterUnavailable(String),

    /// OCR subprocess timed out.
    #[error("OCR execution timed out after {elapsed_ms}ms (configured limit: {limit_ms}ms)")]
    ExecutionTimeout {
        /// Elapsed milliseconds before cancellation.
        elapsed_ms: u64,
        /// Configured maximum timeout limit in milliseconds.
        limit_ms: u64,
    },

    /// Subprocess exited with a non-zero status.
    #[error("OCR process exited with code {}: {}", .exit_code.map_or("unknown".to_string(), |c| c.to_string()), .stderr.trim())]
    ProcessFailed {
        /// Subprocess exit status code if available.
        exit_code: Option<i32>,
        /// Subprocess standard error output.
        stderr: String,
    },

    /// Input image exceeds resource or DoS limits.
    #[error("Input limit exceeded: {0}")]
    InputLimitExceeded(String),

    /// Image format or structure is invalid.
    #[error("Invalid image format or data: {0}")]
    ImageFormat(String),

    /// Image preprocessing failure (e.g. decoding, deskewing, binarization).
    #[error("Image preprocessing failed: {0}")]
    PreprocessingError(String),

    /// Requested OCR recognition language is unsupported or not installed.
    #[error("Unsupported language '{requested}'. Available languages: {available:?}")]
    UnsupportedLanguage {
        /// Language requested by caller.
        requested: String,
        /// List of languages supported by the active engine.
        available: Vec<String>,
    },

    /// Failed to parse Tesseract output (e.g. TSV formatting error).
    #[error("Failed to parse OCR TSV output: {0}")]
    OutputParseError(String),

    /// Low-level filesystem or child process I/O error.
    #[error("I/O error during OCR execution: {0}")]
    Io(#[from] std::io::Error),

    /// Error constructing or serializing CDM representations.
    #[error("CDM conversion failed: {0}")]
    CdmConversion(String),
}

/// Specialized `Result` type for OCR operations.
pub type Result<T> = std::result::Result<T, OcrError>;
