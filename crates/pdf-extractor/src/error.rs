//! # Error Types for PDF Extraction
//!
//! Strongly-typed error variants for parsing, layout analysis, decryption, and CDM conversion.

use thiserror::Error;

/// Result type alias for operations within `web-repo-to-ai-pdf-extractor`.
pub type Result<T> = std::result::Result<T, PdfError>;

/// Errors that can occur during PDF parsing, layout reconstruction, or CDM mapping.
#[derive(Debug, Error)]
pub enum PdfError {
    /// The input bytes do not contain a valid PDF file header (`%PDF-`).
    #[error("Invalid PDF header: expected '%PDF-', got '{0}'")]
    InvalidHeader(String),

    /// The PDF file is completely empty or truncated.
    #[error("PDF document is empty (0 bytes)")]
    EmptyDocument,

    /// Document structure or xref table is malformed.
    #[error("Malformed PDF structure: {0}")]
    MalformedStructure(String),

    /// Document is protected by password or unsupported encryption.
    #[error("PDF document is encrypted or password-protected")]
    EncryptedDocument,

    /// Requested page index is outside the document's page range.
    #[error("Page index {requested} out of range (total pages: {total})")]
    PageOutOfRange {
        /// The requested 1-based page number.
        requested: u32,
        /// The total number of pages in the document.
        total: u32,
    },

    /// Content stream decompression or parsing failed.
    #[error("Failed to decode PDF content stream: {0}")]
    StreamDecompression(String),

    /// Font encoding, ToUnicode CMap, or glyph mapping failed.
    #[error("Font decoding failure: {0}")]
    FontDecoding(String),

    /// An I/O error occurred while reading from a file or stream.
    #[error("I/O error: {0}")]
    Io(String),

    /// Failure when converting parsed PDF structures into a Canonical Context Document Model (`CdmDocument`).
    #[error("CDM mapping error: {0}")]
    CdmMapping(String),

    /// A general extraction error occurred.
    #[error("PDF extraction error: {0}")]
    Other(String),
}

impl From<std::io::Error> for PdfError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<lopdf::Error> for PdfError {
    fn from(err: lopdf::Error) -> Self {
        let msg = err.to_string();
        if msg.to_lowercase().contains("encrypt") {
            Self::EncryptedDocument
        } else {
            Self::MalformedStructure(msg)
        }
    }
}
