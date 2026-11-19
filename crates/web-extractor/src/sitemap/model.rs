//! # Sitemap Domain Models and Data Structures
//!
//! Strongly-typed domain models for parsed XML/RSS/Atom/plain-text sitemaps, sitemap index hierarchies,
//! discovery configurations, and discovery report aggregates.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// Default maximum allowable URLs discovered in a single sitemap crawl.
pub const DEFAULT_MAX_SITEMAP_URLS: usize = 5_000;

/// Hard ceiling for URLs per crawl conforming to official sitemap specifications (50,000).
pub const MAX_PERMISSIBLE_SITEMAP_URLS: usize = 50_000;

/// Default maximum recursion depth for `<sitemapindex>` child references.
pub const DEFAULT_MAX_INDEX_DEPTH: usize = 3;

/// Default maximum child sitemaps inspected per index file.
pub const DEFAULT_MAX_CHILD_SITEMAPS: usize = 50;

/// Default timeout for single sitemap or robots.txt HTTP requests.
pub const DEFAULT_SITEMAP_TIMEOUT: Duration = Duration::from_secs(15);

/// Change frequency hint indicating how frequently content at this URL is likely to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeFrequency {
    /// Document changes each time it is accessed.
    Always,
    /// Changes hourly.
    Hourly,
    /// Changes daily.
    Daily,
    /// Changes weekly.
    Weekly,
    /// Changes monthly.
    Monthly,
    /// Changes yearly.
    Yearly,
    /// Archived content that never changes.
    Never,
}

impl fmt::Display for ChangeFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Yearly => write!(f, "yearly"),
            Self::Never => write!(f, "never"),
        }
    }
}

impl FromStr for ChangeFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "always" => Ok(Self::Always),
            "hourly" => Ok(Self::Hourly),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "yearly" => Ok(Self::Yearly),
            "never" => Ok(Self::Never),
            other => Err(format!("Unknown change frequency: '{other}'")),
        }
    }
}

/// A single discovered URL entry extracted from a sitemap, RSS feed, or Atom feed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapEntry {
    /// Fully-qualified canonical URL for the page.
    pub url: Url,
    /// Optional last modification timestamp.
    pub last_modified: Option<DateTime<Utc>>,
    /// Optional change frequency indicator.
    pub change_frequency: Option<ChangeFrequency>,
    /// Optional priority relative to other URLs on the site (0.0 to 1.0).
    pub priority: Option<f32>,
    /// URL of the sitemap file from which this entry was extracted.
    pub source_sitemap: Option<Url>,
}

impl SitemapEntry {
    /// Constructs a new minimal `SitemapEntry` with just the URL.
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self {
            url,
            last_modified: None,
            change_frequency: None,
            priority: None,
            source_sitemap: None,
        }
    }

    /// Sets the last modified timestamp.
    #[must_use]
    pub fn with_last_modified(mut self, dt: DateTime<Utc>) -> Self {
        self.last_modified = Some(dt);
        self
    }

    /// Sets the change frequency.
    #[must_use]
    pub fn with_change_frequency(mut self, freq: ChangeFrequency) -> Self {
        self.change_frequency = Some(freq);
        self
    }

    /// Sets the priority, automatically clamping to 0.0..=1.0.
    #[must_use]
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = Some(priority.clamp(0.0, 1.0));
        self
    }

    /// Sets the source sitemap URL.
    #[must_use]
    pub fn with_source_sitemap(mut self, source: Url) -> Self {
        self.source_sitemap = Some(source);
        self
    }
}

/// A reference to a child sitemap extracted from a `<sitemapindex>` XML file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapIndexEntry {
    /// URL of the child sitemap file.
    pub sitemap_url: Url,
    /// Optional last modification timestamp of the child sitemap.
    pub last_modified: Option<DateTime<Utc>>,
}

/// Format classification of an inspected sitemap source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SitemapType {
    /// Standard XML `<urlset>` document.
    UrlSet,
    /// XML `<sitemapindex>` referencing child sitemaps.
    SitemapIndex,
    /// Plain text document containing one URL per line.
    PlainText,
    /// RSS 2.0 XML syndication feed.
    RssFeed,
    /// Atom XML syndication feed.
    AtomFeed,
}

impl fmt::Display for SitemapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UrlSet => write!(f, "urlset"),
            Self::SitemapIndex => write!(f, "sitemapindex"),
            Self::PlainText => write!(f, "plain_text"),
            Self::RssFeed => write!(f, "rss_feed"),
            Self::AtomFeed => write!(f, "atom_feed"),
        }
    }
}

/// Parsed contents of an individual sitemap or feed document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedSitemap {
    /// Originating URL of this sitemap document.
    pub sitemap_url: Url,
    /// Format classification of the sitemap.
    pub sitemap_type: SitemapType,
    /// Discovered URL entries.
    pub entries: Vec<SitemapEntry>,
    /// Child sitemap references if this document was a `<sitemapindex>`.
    pub child_sitemaps: Vec<SitemapIndexEntry>,
    /// Non-fatal parsing warnings or dropped item notices.
    pub warnings: Vec<String>,
}

/// Configuration settings governing sitemap discovery, limits, and domain constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapDiscoveryConfig {
    /// Maximum number of unique URLs to discover before stopping.
    pub max_urls: usize,
    /// Maximum recursion depth for child sitemaps referenced in `<sitemapindex>`.
    pub max_index_depth: usize,
    /// Maximum child sitemaps to inspect per index file.
    pub max_child_sitemaps: usize,
    /// Enforces that discovered URLs belong strictly to the target URL's host.
    pub enforce_same_host: bool,
    /// Allows discovered URLs on subdomains of the target domain.
    pub allow_subdomains: bool,
    /// Look up declared sitemaps in `robots.txt` before well-known paths.
    pub check_robots_txt: bool,
    /// Probe well-known endpoints (`/sitemap.xml`, etc.) when `robots.txt` has no sitemaps.
    pub probe_well_known: bool,
    /// HTTP timeout per request.
    pub request_timeout: Duration,
}

impl Default for SitemapDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_urls: DEFAULT_MAX_SITEMAP_URLS,
            max_index_depth: DEFAULT_MAX_INDEX_DEPTH,
            max_child_sitemaps: DEFAULT_MAX_CHILD_SITEMAPS,
            enforce_same_host: true,
            allow_subdomains: false,
            check_robots_txt: true,
            probe_well_known: true,
            request_timeout: DEFAULT_SITEMAP_TIMEOUT,
        }
    }
}

impl SitemapDiscoveryConfig {
    /// Sets the maximum allowable URLs to collect.
    #[must_use]
    pub fn with_max_urls(mut self, max: usize) -> Self {
        self.max_urls = max.clamp(1, MAX_PERMISSIBLE_SITEMAP_URLS);
        self
    }

    /// Sets the maximum recursion depth for index files.
    #[must_use]
    pub fn with_max_index_depth(mut self, depth: usize) -> Self {
        self.max_index_depth = depth;
        self
    }

    /// Sets the maximum child sitemaps inspected per index file.
    #[must_use]
    pub fn with_max_child_sitemaps(mut self, count: usize) -> Self {
        self.max_child_sitemaps = count;
        self
    }

    /// Sets whether to enforce exact host match or allow subdomains.
    #[must_use]
    pub fn with_host_policy(mut self, enforce_same_host: bool, allow_subdomains: bool) -> Self {
        self.enforce_same_host = enforce_same_host;
        self.allow_subdomains = allow_subdomains;
        self
    }

    /// Controls whether to query `robots.txt`.
    #[must_use]
    pub fn with_check_robots_txt(mut self, check: bool) -> Self {
        self.check_robots_txt = check;
        self
    }

    /// Controls whether to probe well-known candidate paths.
    #[must_use]
    pub fn with_probe_well_known(mut self, probe: bool) -> Self {
        self.probe_well_known = probe;
        self
    }

    /// Sets request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

/// Comprehensive report produced after running sitemap discovery against a website.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveredSitemapReport {
    /// Root target URL provided for discovery.
    pub target_url: Url,
    /// Target host/domain name.
    pub target_domain: String,
    /// List of sitemap URLs fetched and inspected during discovery.
    pub sitemaps_inspected: Vec<Url>,
    /// Total URLs parsed across all inspected sitemaps before deduplication.
    pub total_urls_discovered: usize,
    /// Deduplicated, validated unique URL entries.
    pub unique_urls: Vec<SitemapEntry>,
    /// Number of URLs dropped due to cross-domain policy or SSRF restrictions.
    pub filtered_cross_domain_count: usize,
    /// Warning or informational messages encountered during discovery.
    pub warning_messages: Vec<String>,
}

impl DiscoveredSitemapReport {
    /// Returns the number of unique validated URLs discovered.
    #[must_use]
    pub fn url_count(&self) -> usize {
        self.unique_urls.len()
    }

    /// Returns `true` if no unique URLs were discovered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.unique_urls.is_empty()
    }

    /// Extracts a list of all unique [`Url`] instances.
    #[must_use]
    pub fn urls(&self) -> Vec<Url> {
        self.unique_urls.iter().map(|e| e.url.clone()).collect()
    }
}
