//! # Website Crawling Subsystem
//!
//! Provides modular components for web crawling, including internal hyperlink discovery,
//! link normalization, heuristic semantic prioritization, and crawling restrictions.

pub mod links;
pub mod restrictions;

pub use links::{
    DiscoveredInternalLink, InternalLinkDiscoverer, InternalLinkDiscoveryConfig,
    PageLinkDiscoveryReport, DEFAULT_MAX_LINKS_PER_PAGE, MAX_CONTEXT_SNIPPET_LEN,
};
pub use restrictions::{
    CrawlBudgetStats, CrawlDecision, CrawlRejectionReason, CrawlRestrictions,
    CrawlRestrictionsEvaluator, CrawlScope, QueryParamPolicy, ABSOLUTE_MAX_CRAWL_DEPTH,
    DEFAULT_MAX_CRAWL_DEPTH, DEFAULT_MAX_CRAWL_PAGES,
};
