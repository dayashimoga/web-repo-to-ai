//! # Structured Content Elements Extraction
//!
//! Subsystem for extracting HTML tables, source code blocks, syntax detection,
//! and hyperlinks with matrix normalization, GFM Markdown formatting, and link classification.

pub mod code;
pub mod extractor;
pub mod link;
pub mod model;
pub mod syntax;
pub mod table;

pub use code::CodeBlockExtractor;
pub use extractor::ElementsExtractor;
pub use link::{LinkExtractor, ASSET_EXTENSIONS};
pub use model::{
    ElementsOptions, ExtractedCodeBlock, ExtractedElements, ExtractedLink, ExtractedTable,
    LinkType, TableAlignment,
};
pub use syntax::{detect_syntax, normalize_language_tag};
pub use table::TableExtractor;
