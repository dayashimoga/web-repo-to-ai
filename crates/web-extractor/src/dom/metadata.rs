//! # Document Metadata Extraction
//!
//! Extracts structured head metadata, OpenGraph tags, canonical links, and author information.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

/// Structured metadata extracted from HTML `<head>` and standard semantic tags.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title from `<title>` or OpenGraph fallback.
    pub title: Option<String>,
    /// Meta description from `<meta name="description">` or OpenGraph fallback.
    pub description: Option<String>,
    /// Document author from `<meta name="author">` or semantic rel tags.
    pub author: Option<String>,
    /// Canonical document URL from `<link rel="canonical">`.
    pub canonical_url: Option<String>,
    /// Document natural language from `<html lang="...">`.
    pub language: Option<String>,
    /// Base URL specified via `<base href="...">`.
    pub base_url: Option<String>,
    /// Comma-separated or multi-token keywords from `<meta name="keywords">`.
    pub keywords: Vec<String>,
    /// OpenGraph title (`og:title`).
    pub og_title: Option<String>,
    /// OpenGraph description (`og:description`).
    pub og_description: Option<String>,
    /// OpenGraph primary image URL (`og:image`).
    pub og_image: Option<String>,
    /// OpenGraph site name (`og:site_name`).
    pub og_site_name: Option<String>,
    /// Twitter card title (`twitter:title`).
    pub twitter_title: Option<String>,
    /// Twitter card description (`twitter:description`).
    pub twitter_description: Option<String>,
    /// Publication timestamp if present (`article:published_time`).
    pub published_time: Option<String>,
}

impl DocumentMetadata {
    /// Extracts structured document metadata from a parsed `tl::VDom`.
    #[must_use]
    pub fn extract(
        vdom: &tl::VDom<'_>,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> Self {
        let parser = vdom.parser();
        let mut meta = Self::default();

        // 1. Language from <html lang="...">
        for handle in vdom.query_selector("html").into_iter().flatten() {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                if let Some(Some(lang)) = tag.attributes().get("lang") {
                    let cow = lang.as_utf8_str();
                    let s = cow.trim();
                    if !s.is_empty() {
                        meta.language = Some(s.to_string());
                        break;
                    }
                }
            }
        }

        // 2. Base URL from <base href="...">
        for handle in vdom.query_selector("base").into_iter().flatten() {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                if let Some(Some(href)) = tag.attributes().get("href") {
                    let cow = href.as_utf8_str();
                    let s = cow.trim();
                    if !s.is_empty() {
                        meta.base_url = Some(s.to_string());
                        break;
                    }
                }
            }
        }

        // 3. Document Title from <title>
        for handle in vdom.query_selector("title").into_iter().flatten() {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                let text = tag.inner_text(parser);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    meta.title = Some(trimmed.to_string());
                    break;
                }
            }
        }

        // 4. Meta tags: description, author, keywords, OpenGraph, Twitter
        for handle in vdom.query_selector("meta").into_iter().flatten() {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                let attrs = tag.attributes();

                let name = attrs
                    .get("name")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str().to_ascii_lowercase());

                let property = attrs
                    .get("property")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str().to_ascii_lowercase());

                let content = attrs
                    .get("content")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str().trim().to_string())
                    .filter(|s| !s.is_empty());

                if let Some(val) = content {
                    match name.as_deref() {
                        Some("description") if meta.description.is_none() => {
                            meta.description = Some(val.clone());
                        }
                        Some("author") if meta.author.is_none() => {
                            meta.author = Some(val.clone());
                        }
                        Some("keywords") if meta.keywords.is_empty() => {
                            meta.keywords = val
                                .split(',')
                                .map(|k| k.trim().to_string())
                                .filter(|k| !k.is_empty())
                                .collect();
                        }
                        Some("twitter:title") if meta.twitter_title.is_none() => {
                            meta.twitter_title = Some(val.clone());
                        }
                        Some("twitter:description") if meta.twitter_description.is_none() => {
                            meta.twitter_description = Some(val.clone());
                        }
                        _ => {}
                    }

                    match property.as_deref() {
                        Some("og:title") if meta.og_title.is_none() => {
                            meta.og_title = Some(val.clone());
                        }
                        Some("og:description") if meta.og_description.is_none() => {
                            meta.og_description = Some(val.clone());
                        }
                        Some("og:image") if meta.og_image.is_none() => {
                            meta.og_image = Some(val.clone());
                        }
                        Some("og:site_name") if meta.og_site_name.is_none() => {
                            meta.og_site_name = Some(val.clone());
                        }
                        Some("article:published_time") if meta.published_time.is_none() => {
                            meta.published_time = Some(val.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        // 5. Canonical URL from <link rel="canonical" href="...">
        for handle in vdom.query_selector("link").into_iter().flatten() {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                let attrs = tag.attributes();
                let is_canonical = attrs
                    .get("rel")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str().eq_ignore_ascii_case("canonical"))
                    .unwrap_or(false);

                if is_canonical {
                    if let Some(Some(href)) = attrs.get("href") {
                        let cow = href.as_utf8_str();
                        let raw_href = cow.trim();
                        if !raw_href.is_empty() {
                            if let Some(base) = base_url {
                                if let Ok(resolved) = base.join(raw_href, policy) {
                                    meta.canonical_url = Some(resolved.to_string());
                                } else {
                                    meta.canonical_url = Some(raw_href.to_string());
                                }
                            } else {
                                meta.canonical_url = Some(raw_href.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Fallbacks: If title is empty, fallback to og:title or twitter:title
        if meta.title.is_none() {
            meta.title = meta.og_title.clone().or_else(|| meta.twitter_title.clone());
        }

        // Fallbacks: If description is empty, fallback to og:description or twitter:description
        if meta.description.is_none() {
            meta.description = meta
                .og_description
                .clone()
                .or_else(|| meta.twitter_description.clone());
        }

        meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_full_metadata() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en-US">
            <head>
                <title>Rust Systems Programming Guide</title>
                <base href="https://example.com/docs/">
                <meta name="description" content="In-depth guide to safe Rust systems programming.">
                <meta name="author" content="Jane Doe">
                <meta name="keywords" content="rust, systems, async, tokio">
                <meta property="og:title" content="Rust Systems Guide">
                <meta property="og:description" content="Safe systems programming.">
                <meta property="og:image" content="https://example.com/banner.png">
                <meta property="og:site_name" content="Tech Docs">
                <meta property="article:published_time" content="2026-11-17T00:00:00Z">
                <link rel="canonical" href="https://example.com/docs/rust-systems">
            </head>
            <body><h1>Content</h1></body>
            </html>
        "#;

        let policy = UrlSecurityPolicy::default();
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let meta = DocumentMetadata::extract(&vdom, None, &policy);

        assert_eq!(
            meta.title.as_deref(),
            Some("Rust Systems Programming Guide")
        );
        assert_eq!(meta.language.as_deref(), Some("en-US"));
        assert_eq!(meta.base_url.as_deref(), Some("https://example.com/docs/"));
        assert_eq!(
            meta.description.as_deref(),
            Some("In-depth guide to safe Rust systems programming.")
        );
        assert_eq!(meta.author.as_deref(), Some("Jane Doe"));
        assert_eq!(meta.keywords, vec!["rust", "systems", "async", "tokio"]);
        assert_eq!(meta.og_title.as_deref(), Some("Rust Systems Guide"));
        assert_eq!(
            meta.og_image.as_deref(),
            Some("https://example.com/banner.png")
        );
        assert_eq!(meta.og_site_name.as_deref(), Some("Tech Docs"));
        assert_eq!(
            meta.canonical_url.as_deref(),
            Some("https://example.com/docs/rust-systems")
        );
        assert_eq!(meta.published_time.as_deref(), Some("2026-11-17T00:00:00Z"));
    }

    #[test]
    fn test_fallback_title_and_description() {
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Fallback Title From OG">
                <meta property="og:description" content="Fallback Description From OG">
            </head>
            <body><p>Text</p></body>
            </html>
        "#;

        let policy = UrlSecurityPolicy::default();
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let meta = DocumentMetadata::extract(&vdom, None, &policy);

        assert_eq!(meta.title.as_deref(), Some("Fallback Title From OG"));
        assert_eq!(
            meta.description.as_deref(),
            Some("Fallback Description From OG")
        );
    }
}
