//! # Web Extraction Target and URL Classification
//!
//! Encapsulates target URLs for web extraction, integrating SSRF validation from
//! `web-repo-to-ai-url-security` with heuristic document-type classification.

use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::{UrlSecurityError, UrlSecurityPolicy, ValidatedUrl};

/// Heuristic classification of webpage content structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebPageType {
    /// Technical documentation, developer guides, API references, wikis, or manual pages.
    Documentation,
    /// Blog posts, news articles, press releases, or narrative essays.
    Article,
    /// Direct downloadable content or static files (e.g. PDF, Markdown, JSON, plain text).
    FileResource,
    /// Uncategorized or standard web page (e.g., homepages, product marketing, general portals).
    GenericWeb,
}

impl fmt::Display for WebPageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Documentation => write!(f, "documentation"),
            Self::Article => write!(f, "article"),
            Self::FileResource => write!(f, "file_resource"),
            Self::GenericWeb => write!(f, "generic_web"),
        }
    }
}

/// Web acquisition target encapsulating a validated URL and its content classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebTarget {
    /// SSRF-validated, normalized target URL.
    pub url: ValidatedUrl,
    /// Inferred or explicit webpage category.
    pub page_type: WebPageType,
}

impl WebTarget {
    /// Creates a new `WebTarget` from a `ValidatedUrl`, automatically determining content classification.
    #[must_use]
    pub fn new(url: ValidatedUrl) -> Self {
        let page_type = classify_url(&url);
        Self { url, page_type }
    }

    /// Creates a `WebTarget` with an explicitly specified content classification.
    #[must_use]
    pub fn with_page_type(url: ValidatedUrl, page_type: WebPageType) -> Self {
        Self { url, page_type }
    }

    /// Parses and validates a raw URL string using the specified security policy.
    pub fn from_str(raw: &str, policy: &UrlSecurityPolicy) -> Result<Self, UrlSecurityError> {
        let validated = ValidatedUrl::parse(raw, policy)?;
        Ok(Self::new(validated))
    }

    /// Parses and validates a raw URL string using default zero-trust security policy.
    pub fn from_str_default(raw: &str) -> Result<Self, UrlSecurityError> {
        let validated = ValidatedUrl::parse_default(raw)?;
        Ok(Self::new(validated))
    }

    /// Returns a reference to the underlying `ValidatedUrl`.
    #[must_use]
    pub fn url(&self) -> &ValidatedUrl {
        &self.url
    }

    /// Returns the classified webpage type.
    #[must_use]
    pub fn page_type(&self) -> WebPageType {
        self.page_type
    }

    /// Returns canonical URL string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    /// Returns the origin string (`scheme://host[:port]`).
    #[must_use]
    pub fn origin(&self) -> String {
        self.url.origin()
    }

    /// Returns the path string.
    #[must_use]
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Resolves and validates a relative link against this web target.
    pub fn resolve_link(
        &self,
        link: &str,
        policy: &UrlSecurityPolicy,
    ) -> Result<ValidatedUrl, UrlSecurityError> {
        self.url.join(link, policy)
    }

    /// Checks if another URL shares the exact origin with this target.
    #[must_use]
    pub fn is_same_origin(&self, other: &ValidatedUrl) -> bool {
        self.url.is_same_origin(other)
    }

    /// Checks if another URL shares the registered domain with this target.
    #[must_use]
    pub fn is_same_domain(&self, other: &ValidatedUrl) -> bool {
        self.url.is_same_domain(other)
    }

    /// Checks if another URL is a subpath of this target.
    #[must_use]
    pub fn is_subpath(&self, other: &ValidatedUrl) -> bool {
        other.is_subpath_of(&self.url)
    }
}

impl Deref for WebTarget {
    type Target = ValidatedUrl;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl fmt::Display for WebTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.url, self.page_type)
    }
}

/// Heuristically classifies a URL into a `WebPageType`.
#[must_use]
pub fn classify_url(url: &ValidatedUrl) -> WebPageType {
    let path = url.path().to_ascii_lowercase();
    let host = url.host_str().to_ascii_lowercase();

    // 1. Static file resources by extension
    let static_extensions = [
        ".pdf", ".txt", ".md", ".json", ".xml", ".csv", ".yaml", ".yml", ".toml", ".rst",
    ];
    if static_extensions.iter().any(|ext| path.ends_with(ext)) {
        return WebPageType::FileResource;
    }

    // 2. Check host subdomains and publishing platforms (e.g. docs.rs, blog.cloudflare.com, medium.com, substack.com)
    let host_segments: Vec<&str> = host.split('.').collect();
    if host_segments
        .iter()
        .any(|s| *s == "docs" || *s == "doc" || *s == "wiki" || *s == "manual")
    {
        return WebPageType::Documentation;
    }
    if host_segments
        .iter()
        .any(|s| *s == "blog" || *s == "blogs" || *s == "news")
    {
        return WebPageType::Article;
    }
    let article_platforms = ["medium.com", "substack.com", "dev.to", "hashnode.dev"];
    if article_platforms
        .iter()
        .any(|platform| host == *platform || host.ends_with(&format!(".{platform}")))
        && path != "/"
        && !path.is_empty()
    {
        return WebPageType::Article;
    }

    // 3. Check path segments
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Documentation indicators
    let doc_keywords = [
        "doc",
        "docs",
        "documentation",
        "guide",
        "guides",
        "reference",
        "manual",
        "wiki",
        "tutorial",
        "tutorials",
        "spec",
        "specification",
        "api",
        "book",
        "handbook",
    ];
    if segments.iter().any(|seg| doc_keywords.contains(seg)) {
        return WebPageType::Documentation;
    }

    // Article indicators
    let article_keywords = [
        "blog",
        "blogs",
        "article",
        "articles",
        "post",
        "posts",
        "news",
        "stories",
        "story",
        "changelog",
        "release-notes",
    ];
    if segments.iter().any(|seg| article_keywords.contains(seg)) {
        return WebPageType::Article;
    }

    // Date-based article patterns (e.g. /2024/05/20/title or /2024/05/title)
    if segments.len() >= 2 {
        let is_year = segments[0].len() == 4 && segments[0].chars().all(|c| c.is_ascii_digit());
        let is_month = (segments[1].len() == 2 || segments[1].len() == 1)
            && segments[1].chars().all(|c| c.is_ascii_digit());
        if is_year && is_month {
            return WebPageType::Article;
        }
    }

    WebPageType::GenericWeb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_documentation() {
        let cases = [
            "https://docs.rs/tokio/latest/tokio/",
            "https://kubernetes.io/docs/concepts/overview/",
            "https://react.dev/reference/react",
            "https://developer.mozilla.org/en-US/docs/Web/API",
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "https://doc.rust-lang.org/book/ch01-00-getting-started.html",
        ];

        for raw in cases {
            let target = WebTarget::from_str_default(raw).unwrap();
            assert_eq!(
                target.page_type(),
                WebPageType::Documentation,
                "Expected {raw} to be Documentation"
            );
        }
    }

    #[test]
    fn test_classify_articles() {
        let cases = [
            "https://blog.cloudflare.com/incident-report/",
            "https://medium.com/engineering/microservices-architecture",
            "https://news.ycombinator.com/news",
            "https://example.com/2024/11/announcing-v2",
            "https://company.org/posts/year-in-review",
        ];

        for raw in cases {
            let target = WebTarget::from_str_default(raw).unwrap();
            assert_eq!(
                target.page_type(),
                WebPageType::Article,
                "Expected {raw} to be Article"
            );
        }
    }

    #[test]
    fn test_classify_file_resources() {
        let cases = [
            "https://example.com/spec.pdf",
            "https://example.com/data/schema.json",
            "https://raw.githubusercontent.com/org/repo/main/README.md",
        ];

        for raw in cases {
            let target = WebTarget::from_str_default(raw).unwrap();
            assert_eq!(
                target.page_type(),
                WebPageType::FileResource,
                "Expected {raw} to be FileResource"
            );
        }
    }

    #[test]
    fn test_classify_generic_web() {
        let cases = [
            "https://example.com/",
            "https://google.com/about",
            "https://store.example.com/products",
        ];

        for raw in cases {
            let target = WebTarget::from_str_default(raw).unwrap();
            assert_eq!(
                target.page_type(),
                WebPageType::GenericWeb,
                "Expected {raw} to be GenericWeb"
            );
        }
    }

    #[test]
    fn test_target_resolve_link_and_subpath() {
        let policy = UrlSecurityPolicy::default();
        let target = WebTarget::from_str("https://example.com/docs/intro", &policy).unwrap();

        let link = target.resolve_link("quickstart", &policy).unwrap();
        assert_eq!(link.as_str(), "https://example.com/docs/quickstart");

        assert!(target.is_same_origin(&link));
        assert!(target.is_same_domain(&link));

        let subpath_link =
            ValidatedUrl::parse_default("https://example.com/docs/intro/chapter-1").unwrap();
        assert!(target.is_subpath(&subpath_link));
    }

    #[test]
    fn test_target_serde_roundtrip() {
        let target =
            WebTarget::from_str_default("https://example.com/docs/getting-started").unwrap();
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: WebTarget = serde_json::from_str(&json).unwrap();

        assert_eq!(target, deserialized);
        assert_eq!(deserialized.page_type(), WebPageType::Documentation);
    }
}
