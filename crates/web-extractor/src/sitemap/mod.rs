//! # Sitemap Discovery and Ingestion Subsystem
//!
//! Provides automated sitemap discovery, `robots.txt` directive parsing, well-known endpoint probing,
//! multi-format sitemap extraction (XML `<urlset>`, `<sitemapindex>`, RSS 2.0, Atom feeds, and plain-text lists),
//! Gzip decompression with decompression-bomb protection, SSRF security validation, and cross-domain filtering.

pub mod discovery;
pub mod model;
pub mod parser;
pub mod robots;

pub use discovery::{SitemapDiscoverer, WELL_KNOWN_SITEMAP_PATHS};
pub use model::{
    ChangeFrequency, DiscoveredSitemapReport, ParsedSitemap, SitemapDiscoveryConfig, SitemapEntry,
    SitemapIndexEntry, SitemapType, DEFAULT_MAX_CHILD_SITEMAPS, DEFAULT_MAX_INDEX_DEPTH,
    DEFAULT_MAX_SITEMAP_URLS, DEFAULT_SITEMAP_TIMEOUT, MAX_PERMISSIBLE_SITEMAP_URLS,
};
pub use parser::{decode_xml_entities, is_gzipped, parse_datetime, parse_sitemap, strip_namespace};
pub use robots::parse_robots_txt;
