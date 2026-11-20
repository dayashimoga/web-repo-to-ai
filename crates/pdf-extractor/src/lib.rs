//! # PDF Extractor (`web-repo-to-ai-pdf-extractor`)
//!
//! Pure Rust document structure parsing, spatial layout reconstruction, text flow,
//! page bounding box computation, and Canonical Context Document Model (CDM) mapping.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod extractor;
pub mod layout;
pub mod models;
pub mod parser;

// Top-level re-exports
pub use error::{PdfError, Result};
pub use extractor::{PdfExtractionOptions, PdfExtractor};
pub use layout::{LayoutOptions, PdfLayoutEngine};
pub use models::{
    BoundingBox, OutlineItem, PageDimensions, PdfDocument, PdfLine, PdfMetadata, PdfPage, PdfSpan,
    PdfTextBlock, TextBlockKind,
};
pub use parser::PdfParser;
