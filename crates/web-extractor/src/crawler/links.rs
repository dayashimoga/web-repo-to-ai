//! # Internal Hyperlink Discovery Engine
//!
//! Provides automated discovery, normalization, semantic classification, and security validation
//! of internal hyperlinks from HTML documents during website crawling.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::{form_urlencoded, Url};
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy, ValidatedUrl};

use crate::dom::parser::decode_html_entities;
use crate::elements::link::ASSET_EXTENSIONS;
use crate::error::WebExtractorError;
use crate::http::WebFetcher;
use crate::url::WebTarget;

/// Default maximum allowable internal links collected from a single webpage.
pub const DEFAULT_MAX_LINKS_PER_PAGE: usize = 1_000;

/// Default maximum character length of surrounding contextual text snippet.
pub const MAX_CONTEXT_SNIPPET_LEN: usize = 140;

/// Representation of an internal hyperlink discovered on a webpage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredInternalLink {
    /// Fully resolved, canonical, normalized destination URL.
    pub url: Url,
    /// Absolute URL of the webpage where this link was discovered.
    pub source_url: Url,
    /// Visible anchor text normalized without redundant whitespace.
    pub anchor_text: String,
    /// Optional HTML `title` attribute value.
    pub title: Option<String>,
    /// Whitespace-split `rel` attribute tokens (e.g., `["nofollow", "next"]`).
    pub rel: Vec<String>,
    /// Whether the link was located inside primary content (`<main>`, `<article>`, etc.)
    /// rather than boilerplate navigation (`<nav>`, `<header>`, `<footer>`, sidebar).
    pub is_content_link: bool,
    /// Whether the link represents a pagination transition (`rel="next"`, `rel="prev"`, or pagination classes).
    pub is_pagination: bool,
    /// Surrounding sentence or heading text providing semantic relevance.
    pub context_snippet: Option<String>,
}

impl DiscoveredInternalLink {
    /// Constructs a minimal `DiscoveredInternalLink`.
    #[must_use]
    pub fn new(url: Url, source_url: Url) -> Self {
        Self {
            url,
            source_url,
            anchor_text: String::new(),
            title: None,
            rel: Vec::new(),
            is_content_link: true,
            is_pagination: false,
            context_snippet: None,
        }
    }
}

/// Configuration settings governing internal link extraction, filtering, and normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalLinkDiscoveryConfig {
    /// Strip URL fragment `#anchor` identifiers to deduplicate page targets.
    pub strip_fragments: bool,
    /// Strip analytics / marketing query tracking parameters (e.g. `utm_*`, `fbclid`).
    pub strip_tracking_params: bool,
    /// Exclude links targeting static media and binary downloadable asset extensions (`.pdf`, `.zip`, etc.).
    pub exclude_assets: bool,
    /// Drop links flagged with `rel="nofollow"`.
    pub respect_nofollow: bool,
    /// Treat subdomains of the target domain as internal links (e.g. `docs.example.com` vs `example.com`).
    pub allow_subdomains: bool,
    /// Maximum internal links to collect per page.
    pub max_links_per_page: usize,
    /// Extract surrounding text snippets for semantic context.
    pub extract_context_snippets: bool,
}

impl Default for InternalLinkDiscoveryConfig {
    fn default() -> Self {
        Self {
            strip_fragments: true,
            strip_tracking_params: true,
            exclude_assets: true,
            respect_nofollow: false,
            allow_subdomains: false,
            max_links_per_page: DEFAULT_MAX_LINKS_PER_PAGE,
            extract_context_snippets: true,
        }
    }
}

impl InternalLinkDiscoveryConfig {
    /// Controls fragment stripping.
    #[must_use]
    pub fn with_strip_fragments(mut self, strip: bool) -> Self {
        self.strip_fragments = strip;
        self
    }

    /// Controls marketing query tracking parameter stripping.
    #[must_use]
    pub fn with_strip_tracking_params(mut self, strip: bool) -> Self {
        self.strip_tracking_params = strip;
        self
    }

    /// Controls static asset file exclusion.
    #[must_use]
    pub fn with_exclude_assets(mut self, exclude: bool) -> Self {
        self.exclude_assets = exclude;
        self
    }

    /// Controls whether to respect `rel="nofollow"`.
    #[must_use]
    pub fn with_respect_nofollow(mut self, respect: bool) -> Self {
        self.respect_nofollow = respect;
        self
    }

    /// Controls whether subdomains are classified as internal.
    #[must_use]
    pub fn with_allow_subdomains(mut self, allow: bool) -> Self {
        self.allow_subdomains = allow;
        self
    }

    /// Sets the maximum allowable links to discover from a single page.
    #[must_use]
    pub fn with_max_links_per_page(mut self, max: usize) -> Self {
        self.max_links_per_page = max.max(1);
        self
    }

    /// Controls context snippet extraction.
    #[must_use]
    pub fn with_context_snippets(mut self, extract: bool) -> Self {
        self.extract_context_snippets = extract;
        self
    }
}

/// Comprehensive report detailing all hyperlinks discovered on a webpage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageLinkDiscoveryReport {
    /// Webpage URL that was inspected.
    pub source_url: Url,
    /// Base URL actually used for relative link resolution (accounting for `<base href="...">`).
    pub base_url_used: Url,
    /// Normalized, validated internal hyperlinks ready for crawling.
    pub internal_links: Vec<DiscoveredInternalLink>,
    /// Count of links rejected because they target external domains.
    pub external_links_count: usize,
    /// Count of links rejected because they target static assets (`.pdf`, `.zip`, etc.).
    pub assets_count: usize,
    /// Count of in-page anchor links inspected (`#section`).
    pub anchors_count: usize,
    /// Count of links dropped due to SSRF or zero-trust URL security policy violations.
    pub filtered_ssrf_count: usize,
    /// Count of links dropped due to `rel="nofollow"`.
    pub dropped_nofollow_count: usize,
    /// Informational or diagnostic warning messages encountered during discovery.
    pub warnings: Vec<String>,
}

impl PageLinkDiscoveryReport {
    /// Returns the total count of discovered internal links.
    #[must_use]
    pub fn internal_count(&self) -> usize {
        self.internal_links.len()
    }

    /// Returns `true` if no internal links were discovered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.internal_links.is_empty()
    }

    /// Extracts a list of all unique destination [`Url`] instances.
    #[must_use]
    pub fn destination_urls(&self) -> Vec<Url> {
        let mut urls = Vec::new();
        let mut seen = HashSet::new();
        for link in &self.internal_links {
            let s = link.url.as_str();
            if !seen.contains(s) {
                seen.insert(s.to_string());
                urls.push(link.url.clone());
            }
        }
        urls
    }
}

/// Automated internal hyperlink discovery and classification engine.
#[derive(Debug, Clone)]
pub struct InternalLinkDiscoverer {
    /// URL security policy enforcing scheme and SSRF constraints.
    policy: UrlSecurityPolicy,
    /// Web content fetcher.
    fetcher: Arc<WebFetcher>,
    /// Extraction and filtering configuration.
    config: InternalLinkDiscoveryConfig,
}

impl InternalLinkDiscoverer {
    /// Constructs a new `InternalLinkDiscoverer` with default settings and a standard `WebFetcher`.
    #[must_use]
    pub fn new(policy: UrlSecurityPolicy, dns_resolver: SecureDnsResolver) -> Self {
        let fetcher = Arc::new(WebFetcher::new(policy.clone(), dns_resolver));
        Self {
            policy,
            fetcher,
            config: InternalLinkDiscoveryConfig::default(),
        }
    }

    /// Overrides the `WebFetcher` (e.g. for deterministic mock testing).
    #[must_use]
    pub fn with_fetcher(mut self, fetcher: Arc<WebFetcher>) -> Self {
        self.fetcher = fetcher;
        self
    }

    /// Overrides discovery configuration.
    #[must_use]
    pub fn with_config(mut self, config: InternalLinkDiscoveryConfig) -> Self {
        self.config = config;
        self
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &InternalLinkDiscoveryConfig {
        &self.config
    }

    /// Fetches a target webpage over HTTP and extracts all internal links.
    pub async fn discover_from_url(
        &self,
        target_url_str: &str,
    ) -> Result<PageLinkDiscoveryReport, WebExtractorError> {
        let validated_target =
            ValidatedUrl::parse(target_url_str, &self.policy).map_err(WebExtractorError::from)?;
        let target = WebTarget::new(validated_target);

        let response = self.fetcher.fetch(&target).await?;
        let body_text = String::from_utf8_lossy(&response.body);

        self.discover_from_html(&body_text, response.url.url())
    }

    /// Parses raw HTML markup and extracts all internal links relative to `source_url`.
    pub fn discover_from_html(
        &self,
        html: &str,
        source_url: &Url,
    ) -> Result<PageLinkDiscoveryReport, WebExtractorError> {
        let dom = tl::parse(html, tl::ParserOptions::default())
            .map_err(|e| WebExtractorError::HtmlParseFailed(e.to_string()))?;

        self.discover_from_vdom(&dom, source_url)
    }

    /// Extracts all internal links from a parsed `tl::VDom`.
    pub fn discover_from_vdom(
        &self,
        vdom: &tl::VDom<'_>,
        source_url: &Url,
    ) -> Result<PageLinkDiscoveryReport, WebExtractorError> {
        let parser = vdom.parser();
        let mut warnings = Vec::new();

        // 1. Determine effective base URL (check for <base href="..."> in <head>)
        let effective_base_url = resolve_html_base_url(vdom, source_url);

        let source_host = match source_url.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => {
                return Err(WebExtractorError::Sitemap(format!(
                    "Source URL '{source_url}' has no host"
                )))
            }
        };

        let parent_map = build_parent_map(vdom);

        let mut internal_links_map: HashMap<String, DiscoveredInternalLink> = HashMap::new();
        let mut external_links_count = 0;
        let mut assets_count = 0;
        let mut anchors_count = 0;
        let mut filtered_ssrf_count = 0;
        let mut dropped_nofollow_count = 0;

        // Query candidate link elements: <a>, <area>, <link rel="alternate|next|prev">
        let a_nodes = vdom.query_selector("a[href], area[href], link[href]");
        if let Some(nodes) = a_nodes {
            for handle in nodes {
                if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                    let attrs = tag.attributes();

                    let raw_href = match attrs.get("href").flatten() {
                        Some(b) => b.as_utf8_str(),
                        None => continue,
                    };

                    let href_trimmed = raw_href.trim();
                    if href_trimmed.is_empty() {
                        continue;
                    }

                    // Special & Javascript schemes check
                    let href_lower = href_trimmed.to_ascii_lowercase();
                    if href_lower.starts_with("javascript:")
                        || href_lower.starts_with("vbscript:")
                        || href_lower.starts_with("mailto:")
                        || href_lower.starts_with("tel:")
                        || href_lower.starts_with("sms:")
                        || href_lower.starts_with("data:")
                    {
                        continue;
                    }

                    // In-page section anchor check
                    if href_trimmed.starts_with('#') {
                        anchors_count += 1;
                        continue;
                    }

                    // Extract rel tokens
                    let rel: Vec<String> = attrs
                        .get("rel")
                        .flatten()
                        .map(|b| b.as_utf8_str())
                        .map(|s| s.split_whitespace().map(str::to_ascii_lowercase).collect())
                        .unwrap_or_default();

                    // Check nofollow policy
                    if self.config.respect_nofollow && rel.contains(&"nofollow".to_string()) {
                        dropped_nofollow_count += 1;
                        continue;
                    }

                    // Resolve relative link against effective base URL
                    let resolved_url = match effective_base_url.join(href_trimmed) {
                        Ok(u) => u,
                        Err(e) => {
                            warnings.push(format!("Could not join link '{href_trimmed}': {e}"));
                            continue;
                        }
                    };

                    // Scheme check: only http and https
                    if resolved_url.scheme() != "http" && resolved_url.scheme() != "https" {
                        continue;
                    }

                    // Asset extension check
                    if self.config.exclude_assets && is_asset_url(&resolved_url) {
                        assets_count += 1;
                        continue;
                    }

                    // Host / domain matching check
                    if !self.is_internal_host(&resolved_url, &source_host) {
                        external_links_count += 1;
                        continue;
                    }

                    // Normalize URL (strip fragments, strip tracking parameters)
                    let normalized_url = self.normalize_url(&resolved_url);

                    // SSRF and Zero-Trust policy validation
                    if ValidatedUrl::parse(normalized_url.as_str(), &self.policy).is_err() {
                        filtered_ssrf_count += 1;
                        continue;
                    }

                    // Extract visible anchor text
                    let mut text_buf = String::new();
                    collect_tag_text(tag, parser, &mut text_buf);
                    let anchor_text = text_buf.split_whitespace().collect::<Vec<_>>().join(" ");

                    // Extract title attribute
                    let title = attrs
                        .get("title")
                        .flatten()
                        .map(|b| decode_html_entities(b.as_utf8_str().trim()))
                        .filter(|s| !s.is_empty());

                    // Heuristics: content link vs boilerplate navigation
                    let is_content_link = is_content_area_link(handle, parser, &parent_map);

                    // Heuristics: pagination detection
                    let is_pagination = is_pagination_link(&rel, &anchor_text, attrs);

                    // Context snippet extraction
                    let context_snippet = if self.config.extract_context_snippets {
                        extract_surrounding_context(handle, parser, &parent_map)
                    } else {
                        None
                    };

                    let key = normalized_url.as_str().to_string();

                    // If already present, prefer content links or links with anchor text
                    if let Some(existing) = internal_links_map.get_mut(&key) {
                        if !existing.is_content_link && is_content_link {
                            existing.is_content_link = true;
                        }
                        if existing.anchor_text.is_empty() && !anchor_text.is_empty() {
                            existing.anchor_text = anchor_text;
                        }
                        if existing.context_snippet.is_none() && context_snippet.is_some() {
                            existing.context_snippet = context_snippet;
                        }
                    } else {
                        if internal_links_map.len() >= self.config.max_links_per_page {
                            warnings.push(format!(
                                "Reached maximum internal links per page limit ({})",
                                self.config.max_links_per_page
                            ));
                            break;
                        }

                        let link_entry = DiscoveredInternalLink {
                            url: normalized_url,
                            source_url: source_url.clone(),
                            anchor_text,
                            title,
                            rel,
                            is_content_link,
                            is_pagination,
                            context_snippet,
                        };
                        internal_links_map.insert(key, link_entry);
                    }
                }
            }
        }

        // Deterministic 3-tier priority sorting:
        // Tier 0: Primary content links (is_content_link && !is_pagination)
        // Tier 1: Pagination transitions (is_pagination)
        // Tier 2: Boilerplate navigation (nav, header, footer, sidebar)
        // Tie-breaker: Alphabetical URL string
        let mut internal_links: Vec<DiscoveredInternalLink> =
            internal_links_map.into_values().collect();
        internal_links.sort_by(|a, b| {
            link_priority_tier(a)
                .cmp(&link_priority_tier(b))
                .then_with(|| a.url.as_str().cmp(b.url.as_str()))
        });

        Ok(PageLinkDiscoveryReport {
            source_url: source_url.clone(),
            base_url_used: effective_base_url,
            internal_links,
            external_links_count,
            assets_count,
            anchors_count,
            filtered_ssrf_count,
            dropped_nofollow_count,
            warnings,
        })
    }

    /// Verifies if a resolved URL belongs to the internal site boundary.
    fn is_internal_host(&self, url: &Url, source_host: &str) -> bool {
        let host = match url.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => return false,
        };

        if host == source_host {
            return true;
        }

        if self.config.allow_subdomains {
            let dot_target = format!(".{source_host}");
            if host.ends_with(&dot_target) {
                return true;
            }
        }

        false
    }

    /// Normalizes candidate link URL (stripping fragments and analytics query tracking parameters).
    fn normalize_url(&self, raw: &Url) -> Url {
        let mut url = raw.clone();

        // 1. Strip in-page section fragment `#section`
        if self.config.strip_fragments {
            url.set_fragment(None);
        }

        // 2. Strip tracking query parameters if configured
        if self.config.strip_tracking_params && url.query().is_some() {
            let pairs: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(k, _)| !is_tracking_param(k))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if pairs.is_empty() {
                url.set_query(None);
            } else {
                let mut serializer = form_urlencoded::Serializer::new(String::new());
                for (k, v) in pairs {
                    serializer.append_pair(&k, &v);
                }
                let new_query = serializer.finish();
                url.set_query(Some(&new_query));
            }
        }

        url
    }
}

/// Checks if a query parameter key represents an analytics or campaign tracking parameter.
fn is_tracking_param(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.starts_with("utm_")
        || lower == "fbclid"
        || lower == "gclid"
        || lower == "msclkid"
        || lower == "mc_cid"
        || lower == "mc_eid"
        || lower == "_hsenc"
        || lower == "_hsmi"
        || lower == "ref"
        || lower == "ref_src"
}

/// Identifies if a URL targets a known downloadable asset or media file extension.
fn is_asset_url(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    ASSET_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

/// Resolves HTML `<base href="...">` declaration if present in `<head>`.
fn resolve_html_base_url(vdom: &tl::VDom<'_>, fallback: &Url) -> Url {
    let parser = vdom.parser();
    if let Some(nodes) = vdom.query_selector("base[href]") {
        for handle in nodes {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                if let Some(raw_href) = tag.attributes().get("href").flatten() {
                    let href_cow = raw_href.as_utf8_str();
                    let href_str = href_cow.trim();
                    if !href_str.is_empty() {
                        if let Ok(base_url) = fallback.join(href_str) {
                            return base_url;
                        }
                    }
                }
            }
        }
    }
    fallback.clone()
}

/// Builds a child-to-parent handle lookup map for DOM hierarchy traversal.
fn build_parent_map(vdom: &tl::VDom<'_>) -> HashMap<u32, tl::NodeHandle> {
    let mut map = HashMap::new();
    let parser = vdom.parser();
    for &child in vdom.children() {
        populate_parents(child, None, parser, &mut map);
    }
    map
}

fn populate_parents(
    handle: tl::NodeHandle,
    parent: Option<tl::NodeHandle>,
    parser: &tl::Parser<'_>,
    map: &mut HashMap<u32, tl::NodeHandle>,
) {
    let id = handle.get_inner();
    if let Some(p) = parent {
        map.insert(id, p);
    }
    if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
        for &child in tag.children().top().iter() {
            populate_parents(child, Some(handle), parser, map);
        }
    }
}

/// Checks whether an `<a>` element was situated in primary content rather than navigation boilerplate.
fn is_content_area_link(
    handle: tl::NodeHandle,
    parser: &tl::Parser<'_>,
    parent_map: &HashMap<u32, tl::NodeHandle>,
) -> bool {
    let mut current = parent_map.get(&handle.get_inner()).copied();
    while let Some(parent_handle) = current {
        if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
            let parent_name = parent_tag.name().as_utf8_str();
            match parent_name.as_ref() {
                "nav" | "header" | "footer" | "aside" => return false,
                "main" | "article" => return true,
                _ => {
                    // Check class attributes for navigation cues
                    if let Some(class_bytes) = parent_tag.attributes().get("class").flatten() {
                        let class_str = class_bytes.as_utf8_str().to_ascii_lowercase();
                        if class_str.contains("nav")
                            || class_str.contains("menu")
                            || class_str.contains("sidebar")
                            || class_str.contains("footer")
                            || class_str.contains("header")
                        {
                            return false;
                        }
                        if class_str.contains("content")
                            || class_str.contains("article")
                            || class_str.contains("post")
                            || class_str.contains("markdown")
                        {
                            return true;
                        }
                    }
                }
            }
            current = parent_map.get(&parent_handle.get_inner()).copied();
        } else {
            break;
        }
    }

    // Default to true if not within identifiable navigation containers
    true
}

/// Heuristics detecting pagination hyperlinks.
fn is_pagination_link(rel: &[String], anchor_text: &str, attrs: &tl::Attributes<'_>) -> bool {
    // 1. Rel attribute check: rel="next" or rel="prev"
    if rel.contains(&"next".to_string()) || rel.contains(&"prev".to_string()) {
        return true;
    }

    // 2. Class / ID check
    for attr_name in &["class", "id", "aria-label"] {
        if let Some(val_bytes) = attrs.get(*attr_name).flatten() {
            let val = val_bytes.as_utf8_str().to_ascii_lowercase();
            if val.contains("pagination")
                || val.contains("pager")
                || val.contains("next-page")
                || val.contains("prev-page")
            {
                return true;
            }
        }
    }

    // 3. Anchor text patterns
    let text_lower = anchor_text.trim().to_ascii_lowercase();
    matches!(
        text_lower.as_str(),
        "next"
            | "previous"
            | "prev"
            | "next page"
            | "previous page"
            | "older"
            | "newer"
            | "»"
            | "«"
            | ">"
            | "<"
    )
}

/// Extracts surrounding contextual text from parent container.
fn extract_surrounding_context(
    handle: tl::NodeHandle,
    parser: &tl::Parser<'_>,
    parent_map: &HashMap<u32, tl::NodeHandle>,
) -> Option<String> {
    if let Some(parent_handle) = parent_map.get(&handle.get_inner()) {
        if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
            let mut buf = String::new();
            collect_tag_text(parent_tag, parser, &mut buf);
            let normalized = buf.split_whitespace().collect::<Vec<_>>().join(" ");
            if !normalized.is_empty() {
                if normalized.chars().count() > MAX_CONTEXT_SNIPPET_LEN {
                    let truncated: String =
                        normalized.chars().take(MAX_CONTEXT_SNIPPET_LEN).collect();
                    return Some(format!("{truncated}..."));
                }
                return Some(normalized);
            }
        }
    }
    None
}

/// Recursively collects visible anchor text from tag children.
fn collect_tag_text(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>, buf: &mut String) {
    for child_handle in tag.children().top().iter() {
        if let Some(node) = child_handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    let decoded = decode_html_entities(bytes.as_utf8_str().as_ref());
                    buf.push_str(&decoded);
                }
                tl::Node::Tag(child_tag) => {
                    collect_tag_text(child_tag, parser, buf);
                }
                tl::Node::Comment(_) => {}
            }
        }
    }
}

/// Evaluates the priority tier of a discovered internal hyperlink for crawling order.
#[inline]
fn link_priority_tier(link: &DiscoveredInternalLink) -> u8 {
    if link.is_content_link && !link.is_pagination {
        0
    } else if link.is_pagination {
        1
    } else {
        2
    }
}
