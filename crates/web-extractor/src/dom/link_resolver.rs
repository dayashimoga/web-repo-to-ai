//! # Link and Subresource Canonicalization
//!
//! Resolves relative links, navigation URLs, and asset paths against document base URLs.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

/// Canonicalizes relative links and image sources against the document base URL.
pub struct LinkResolver;

impl LinkResolver {
    /// Resolves an `href` link attribute against the document base URL and security policy.
    ///
    /// Preserves fragment anchors (`#section`), mailto links, and tel links as-is.
    /// Blocks `javascript:` execution schemes.
    pub fn resolve_href(
        raw_href: &str,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> String {
        let trimmed = raw_href.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Anchor fragment links
        if trimmed.starts_with('#') {
            return trimmed.to_string();
        }

        // Scheme checks
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("javascript:") {
            return String::new(); // Strip malicious javascript: schemes
        }

        if lower.starts_with("mailto:") || lower.starts_with("tel:") {
            return trimmed.to_string();
        }

        if let Some(base) = base_url {
            if let Ok(resolved) = base.join(trimmed, policy) {
                return resolved.to_string();
            }
        }

        trimmed.to_string()
    }

    /// Resolves an `img` or asset `src` attribute against the document base URL and security policy.
    pub fn resolve_src(
        raw_src: &str,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> String {
        let trimmed = raw_src.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("data:") {
            return trimmed.to_string();
        }

        if lower.starts_with("javascript:") {
            return String::new();
        }

        if let Some(base) = base_url {
            if let Ok(resolved) = base.join(trimmed, policy) {
                return resolved.to_string();
            }
        }

        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_relative_href() {
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse_default("https://example.com/docs/intro").unwrap();
        assert_eq!(
            LinkResolver::resolve_href("getting-started", Some(&base), &policy),
            "https://example.com/docs/getting-started"
        );
        assert_eq!(
            LinkResolver::resolve_href("/api/v1", Some(&base), &policy),
            "https://example.com/api/v1"
        );
        assert_eq!(
            LinkResolver::resolve_href("../index.html", Some(&base), &policy),
            "https://example.com/index.html"
        );
        assert_eq!(
            LinkResolver::resolve_href("https://other.com/page", Some(&base), &policy),
            "https://other.com/page"
        );
    }

    #[test]
    fn test_resolve_anchor_and_schemes() {
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse_default("https://example.com/").unwrap();
        assert_eq!(
            LinkResolver::resolve_href("#section-2", Some(&base), &policy),
            "#section-2"
        );
        assert_eq!(
            LinkResolver::resolve_href("mailto:dev@example.com", Some(&base), &policy),
            "mailto:dev@example.com"
        );
        assert_eq!(
            LinkResolver::resolve_href("javascript:alert(1)", Some(&base), &policy),
            ""
        );
    }

    #[test]
    fn test_resolve_src() {
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse_default("https://example.com/blog/post-1").unwrap();
        assert_eq!(
            LinkResolver::resolve_src("images/photo.png", Some(&base), &policy),
            "https://example.com/blog/images/photo.png"
        );
        assert_eq!(
            LinkResolver::resolve_src("data:image/png;base64,iVBORw0KGgo...", Some(&base), &policy),
            "data:image/png;base64,iVBORw0KGgo..."
        );
    }
}
