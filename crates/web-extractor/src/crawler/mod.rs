//! # Website Crawling Subsystem
//!
//! Provides modular components for web crawling, including internal hyperlink discovery,
//! link normalization, heuristic semantic prioritization, and crawling restrictions.

pub mod links;

pub use links::{
    DiscoveredInternalLink, InternalLinkDiscoverer, InternalLinkDiscoveryConfig,
    PageLinkDiscoveryReport, DEFAULT_MAX_LINKS_PER_PAGE, MAX_CONTEXT_SNIPPET_LEN,
};
