//! # PDF Extractor (`web-repo-to-ai-pdf-extractor`)
//!
//! Pure Rust document structure parsing, spatial layout reconstruction, text flow,
//! page bounding box computation, and Canonical Context Document Model (CDM) mapping.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod encryption;
pub mod error;
pub mod extractor;
pub mod layout;
pub mod limits;
pub mod models;
pub mod parser;
pub mod provenance;
pub mod scanned;
pub mod table;

// Top-level re-exports
pub use encryption::{
    extract_encryption_info, extract_raw_encryption_info, EncryptionInfo, PdfPermissions,
};
pub use error::{PdfError, Result};
pub use extractor::{PdfExtractionOptions, PdfExtractor};
pub use layout::{LayoutOptions, PageLayoutResult, PdfLayoutEngine};
pub use limits::PdfLimits;
pub use models::{
    BoundingBox, OutlineItem, PageDimensions, PdfDocument, PdfLine, PdfMetadata, PdfPage, PdfSpan,
    PdfTextBlock, TextBlockKind,
};
pub use parser::{PdfParser, PdfParserOptions};
pub use provenance::{PageProvenance, SectionProvenance};
pub use scanned::{
    analyze_document_scanned, analyze_page_scanned, extract_page_images, PdfImageInfo,
    ScannedDetectorOptions, ScannedDocumentAnalysis, ScannedPageAnalysis,
};
pub use table::{PdfTable, PdfTableCell, PdfTableRow, TableDetectorOptions};
