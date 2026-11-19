//! # Website Crawling Subsystem
//!
//! Provides modular components for web crawling, including internal hyperlink discovery,
//! link normalization, heuristic semantic prioritization, and crawling restrictions.

pub mod frontier;
pub mod links;
pub mod loop_detector;
pub mod restrictions;

pub use frontier::{
    CanonicalUrlNormalizerConfig, CrawlFrontier, CrawlQueueItem, CrawlerUrlNormalizer,
    FrontierEnqueueDecision, FrontierStats, DEFAULT_INDEX_FILENAMES,
};
pub use links::{
    DiscoveredInternalLink, InternalLinkDiscoverer, InternalLinkDiscoveryConfig,
    PageLinkDiscoveryReport, DEFAULT_MAX_LINKS_PER_PAGE, MAX_CONTEXT_SNIPPET_LEN,
};
pub use loop_detector::{
    CrawlLoopDetector, CrawlTrapReason, LoopDetectorConfig, DEFAULT_MAX_PATH_SEGMENTS,
    DEFAULT_MAX_QUERY_KEY_REPEATS, DEFAULT_MAX_QUERY_PARAMS, DEFAULT_MAX_REDIRECT_CHAIN_LEN,
    DEFAULT_MAX_SEGMENT_REPEATS,
};
pub use restrictions::{
    CrawlBudgetStats, CrawlDecision, CrawlRejectionReason, CrawlRestrictions,
    CrawlRestrictionsEvaluator, CrawlScope, QueryParamPolicy, ABSOLUTE_MAX_CRAWL_DEPTH,
    DEFAULT_MAX_CRAWL_DEPTH, DEFAULT_MAX_CRAWL_PAGES,
};
