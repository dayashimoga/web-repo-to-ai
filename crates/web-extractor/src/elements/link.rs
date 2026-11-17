//! # Hyperlink Extraction, Canonicalization, and Classification
//!
//! Extracts HTML `<a>` links, resolves relative URLs against document base URLs,
//! and classifies links into internal, external, anchor, asset, and special schemes.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::model::{ElementsOptions, ExtractedLink, LinkType};
use crate::dom::parser::decode_html_entities;

/// Known file extensions representing static downloadable assets, data, or media files.
pub const ASSET_EXTENSIONS: &[&str] = &[
    // Documents & Archives
    ".pdf", ".zip", ".tar", ".gz", ".tgz", ".tar.gz", ".xz", ".bz2", ".7z", ".rar", ".exe", ".dmg",
    ".pkg", ".deb", ".rpm", ".iso", // Data formats
    ".csv", ".tsv", ".xlsx", ".xls", ".docx", ".doc", ".pptx", ".ppt", ".parquet", ".json", ".xml",
    ".avro", // Media & Graphics
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".ico", ".bmp", ".tif", ".tiff", ".mp4",
    ".webm", ".avi", ".mov", ".mkv", ".mp3", ".wav", ".ogg", ".flac",
];

/// Extractor for document hyperlinks.
#[derive(Debug, Clone, Default)]
pub struct LinkExtractor {
    url_security_policy: UrlSecurityPolicy,
}

impl LinkExtractor {
    /// Constructs a `LinkExtractor` with default URL security policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            url_security_policy: UrlSecurityPolicy::default(),
        }
    }

    /// Configures the extractor with a custom URL security policy.
    #[must_use]
    pub fn with_url_security_policy(mut self, policy: UrlSecurityPolicy) -> Self {
        self.url_security_policy = policy;
        self
    }

    /// Extracts and classifies all hyperlinks from a parsed `tl::VDom`.
    #[must_use]
    pub fn extract_from_vdom(
        &self,
        vdom: &tl::VDom<'_>,
        base_url: Option<&ValidatedUrl>,
        options: &ElementsOptions,
    ) -> Vec<ExtractedLink> {
        let parser = vdom.parser();
        let mut results = Vec::new();

        if !options.extract_links {
            return results;
        }

        let a_nodes = vdom.query_selector("a[href]");
        if let Some(nodes) = a_nodes {
            for handle in nodes {
                if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                    if let Some(link) = self.extract_single_link(tag, parser, base_url) {
                        results.push(link);
                    }
                }
            }
        }

        results
    }

    /// Extracts and classifies a single `<a>` tag.
    fn extract_single_link(
        &self,
        tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
        base_url: Option<&ValidatedUrl>,
    ) -> Option<ExtractedLink> {
        let attrs = tag.attributes();

        let raw_href_bytes = attrs.get("href").flatten()?;
        let raw_href = raw_href_bytes.as_utf8_str();
        let href_trimmed = raw_href.trim();

        if href_trimmed.is_empty() {
            return None;
        }

        // Neutralize dangerous javascript: schemes for security
        let href_lower = href_trimmed.to_ascii_lowercase();
        if href_lower.starts_with("javascript:") || href_lower.starts_with("vbscript:") {
            return None;
        }

        // Extract title
        let title = attrs
            .get("title")
            .flatten()
            .map(|b| decode_html_entities(b.as_utf8_str().trim()))
            .filter(|s| !s.is_empty());

        // Extract rel tokens
        let rel: Vec<String> = attrs
            .get("rel")
            .flatten()
            .map(|b| b.as_utf8_str())
            .map(|s| s.split_whitespace().map(str::to_ascii_lowercase).collect())
            .unwrap_or_default();

        // Extract anchor text
        let mut text_buf = String::new();
        Self::collect_text(tag, parser, &mut text_buf);
        let text = text_buf.split_whitespace().collect::<Vec<_>>().join(" ");

        // Classify and resolve URL
        let (resolved_url, link_type) = self.resolve_and_classify(href_trimmed, base_url);

        Some(ExtractedLink {
            url: resolved_url,
            original_href: href_trimmed.to_string(),
            text,
            title,
            rel,
            link_type,
        })
    }

    /// Resolves an href and determines its `LinkType`.
    fn resolve_and_classify(
        &self,
        href: &str,
        base_url: Option<&ValidatedUrl>,
    ) -> (String, LinkType) {
        let href_lower = href.to_ascii_lowercase();

        // 1. In-page anchor
        if href.starts_with('#') {
            return (href.to_string(), LinkType::Anchor);
        }

        // 2. Special schemes
        if href_lower.starts_with("mailto:")
            || href_lower.starts_with("tel:")
            || href_lower.starts_with("sms:")
            || href_lower.starts_with("urn:")
        {
            return (href.to_string(), LinkType::Special);
        }

        // 3. Resolve URL
        let resolved = if let Some(base) = base_url {
            base.join(href, &self.url_security_policy)
                .map(|u| u.as_str().to_string())
                .unwrap_or_else(|_| href.to_string())
        } else {
            href.to_string()
        };

        // 4. Check for asset extension
        let clean_path = href
            .split('?')
            .next()
            .unwrap_or(href)
            .split('#')
            .next()
            .unwrap_or(href);
        let path_lower = clean_path.to_ascii_lowercase();

        if ASSET_EXTENSIONS.iter().any(|ext| path_lower.ends_with(ext)) {
            return (resolved, LinkType::Asset);
        }

        // 5. Internal vs External
        if let Some(base) = base_url {
            if let Ok(target_val) = ValidatedUrl::parse(&resolved, &self.url_security_policy) {
                if target_val.host() == base.host() {
                    return (resolved, LinkType::Internal);
                }
                return (resolved, LinkType::External);
            }
        } else if href.starts_with("http://") || href.starts_with("https://") {
            return (resolved, LinkType::External);
        } else {
            return (resolved, LinkType::Internal);
        }

        (resolved, LinkType::Internal)
    }

    /// Recursively collects visible anchor text from tag children.
    fn collect_text(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>, buf: &mut String) {
        for child_handle in tag.children().top().iter() {
            if let Some(node) = child_handle.get(parser) {
                match node {
                    tl::Node::Raw(bytes) => {
                        let decoded = decode_html_entities(bytes.as_utf8_str().as_ref());
                        buf.push_str(&decoded);
                    }
                    tl::Node::Tag(child_tag) => {
                        Self::collect_text(child_tag, parser, buf);
                    }
                    tl::Node::Comment(_) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_and_classify_links() {
        let base_url = ValidatedUrl::parse_default("https://example.com/docs/intro").unwrap();
        let html = r##"
            <div>
                <a href="#quick-start">Quick Start</a>
                <a href="/guide/setup" title="Setup Guide">Setup</a>
                <a href="https://external.org/spec" rel="nofollow noopener">Specification</a>
                <a href="/downloads/manual.pdf">User Manual</a>
                <a href="mailto:support@example.com">Contact Us</a>
                <a href="javascript:alert(1)">Exploit</a>
            </div>
        "##;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = LinkExtractor::new();
        let links =
            extractor.extract_from_vdom(&vdom, Some(&base_url), &ElementsOptions::default());

        assert_eq!(links.len(), 5); // javascript: link is neutralized and stripped!

        // 1. Anchor
        assert_eq!(links[0].url, "#quick-start");
        assert_eq!(links[0].text, "Quick Start");
        assert_eq!(links[0].link_type, LinkType::Anchor);

        // 2. Internal
        assert_eq!(links[1].url, "https://example.com/guide/setup");
        assert_eq!(links[1].text, "Setup");
        assert_eq!(links[1].title.as_deref(), Some("Setup Guide"));
        assert_eq!(links[1].link_type, LinkType::Internal);

        // 3. External
        assert_eq!(links[2].url, "https://external.org/spec");
        assert_eq!(links[2].text, "Specification");
        assert_eq!(links[2].rel, vec!["nofollow", "noopener"]);
        assert_eq!(links[2].link_type, LinkType::External);

        // 4. Asset
        assert_eq!(links[3].url, "https://example.com/downloads/manual.pdf");
        assert_eq!(links[3].link_type, LinkType::Asset);

        // 5. Special
        assert_eq!(links[4].url, "mailto:support@example.com");
        assert_eq!(links[4].link_type, LinkType::Special);
    }
}
