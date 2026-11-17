//! # Technical Documentation & Navigation Hierarchy Engine
//!
//! Provides documentation framework detection, breadcrumb extraction, nested table-of-contents
//! generation, callout/admonition discovery, and section-by-section extraction.

pub mod admonition;
pub mod extractor;
pub mod framework;
pub mod model;
pub mod navigation;
pub mod toc;

pub use admonition::AdmonitionExtractor;
pub use extractor::DocExtractor;
pub use framework::detect_framework;
pub use model::{
    Admonition, AdmonitionType, BreadcrumbItem, DocFramework, DocLink, DocPagination, DocSection,
    DocVersionInfo, ExtractedDocumentation, TocItem,
};
pub use navigation::DocNavigationExtractor;
pub use toc::extract_table_of_contents;
