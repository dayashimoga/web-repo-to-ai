//! # Documentation Navigation Hierarchy Extraction
//!
//! Extracts breadcrumb trails, previous/next pagination links, and version selectors.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::model::{BreadcrumbItem, DocLink, DocPagination, DocVersionInfo};
use crate::dom::link_resolver::LinkResolver;

/// Extracts hierarchical breadcrumbs, previous/next links, and version info.
pub struct DocNavigationExtractor;

impl DocNavigationExtractor {
    /// Extracts breadcrumb trail items from parsed `tl::VDom`.
    #[must_use]
    pub fn extract_breadcrumbs(
        vdom: &tl::VDom<'_>,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> Vec<BreadcrumbItem> {
        let parser = vdom.parser();
        let mut items = Vec::new();

        let breadcrumb_selectors = [
            ".breadcrumbs",
            ".breadcrumb",
            "nav[aria-label=\"breadcrumbs\"]",
            "nav[aria-label=\"breadcrumb\"]",
            "ol[itemtype*=\"BreadcrumbList\"]",
            "ul[itemtype*=\"BreadcrumbList\"]",
        ];

        for selector in breadcrumb_selectors {
            if let Some(handles) = vdom.query_selector(selector) {
                for handle in handles {
                    if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                        Self::collect_breadcrumb_items(tag, parser, base_url, policy, &mut items);
                        if !items.is_empty() {
                            return items;
                        }
                    }
                }
            }
        }

        items
    }

    fn collect_breadcrumb_items(
        container: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
        items: &mut Vec<BreadcrumbItem>,
    ) {
        let mut position = 1;

        for child_handle in container.children().top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                let name = child_tag.name().as_utf8_str();
                let lower = name.to_ascii_lowercase();

                if lower == "li" || lower == "a" || lower == "span" {
                    let mut item_label = String::new();
                    let mut item_url = None;

                    // Check if node is or contains an <a> tag
                    if lower == "a" {
                        if let Some(Some(href)) = child_tag.attributes().get("href") {
                            let resolved = LinkResolver::resolve_href(
                                href.as_utf8_str().as_ref(),
                                base_url,
                                policy,
                            );
                            if !resolved.is_empty() {
                                item_url = Some(resolved);
                            }
                        }
                        item_label = child_tag.inner_text(parser).trim().to_string();
                    } else {
                        // Look for child anchor
                        for nested_handle in child_tag.children().top().iter() {
                            if let Some(tl::Node::Tag(nested_tag)) = nested_handle.get(parser) {
                                if nested_tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
                                    if let Some(Some(href)) = nested_tag.attributes().get("href") {
                                        let resolved = LinkResolver::resolve_href(
                                            href.as_utf8_str().as_ref(),
                                            base_url,
                                            policy,
                                        );
                                        if !resolved.is_empty() {
                                            item_url = Some(resolved);
                                        }
                                    }
                                    item_label = nested_tag.inner_text(parser).trim().to_string();
                                    break;
                                }
                            }
                        }
                        if item_label.is_empty() {
                            item_label = child_tag.inner_text(parser).trim().to_string();
                        }
                    }

                    // Clean separator characters e.g. '/', '>', '»'
                    let cleaned_label = item_label
                        .trim_matches(|c| c == '/' || c == '>' || c == '»' || c == '·' || c == ' ')
                        .trim()
                        .to_string();

                    if !cleaned_label.is_empty() {
                        items.push(BreadcrumbItem {
                            label: cleaned_label,
                            url: item_url,
                            position,
                        });
                        position += 1;
                    }
                }
            }
        }
    }

    /// Extracts previous and next pagination links from parsed `tl::VDom`.
    #[must_use]
    pub fn extract_pagination(
        vdom: &tl::VDom<'_>,
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> DocPagination {
        let parser = vdom.parser();
        let mut prev = None;
        let mut next = None;

        // 1. Check rel="prev" and rel="next"
        if let Some(handles) = vdom.query_selector("a[rel=\"prev\"], a[rel=\"next\"]") {
            for handle in handles {
                if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                    let attrs = tag.attributes();
                    let rel = attrs
                        .get("rel")
                        .and_then(|v| v)
                        .map(|b| b.as_utf8_str().to_ascii_lowercase());

                    if let Some(Some(href)) = attrs.get("href") {
                        let resolved = LinkResolver::resolve_href(
                            href.as_utf8_str().as_ref(),
                            base_url,
                            policy,
                        );
                        if !resolved.is_empty() {
                            let title = Self::extract_pagination_title(tag, parser);
                            let link = DocLink {
                                title,
                                url: resolved,
                            };

                            if rel.as_deref() == Some("prev") && prev.is_none() {
                                prev = Some(link);
                            } else if rel.as_deref() == Some("next") && next.is_none() {
                                next = Some(link);
                            }
                        }
                    }
                }
            }
        }

        // 2. Docusaurus / MkDocs class patterns if rel attributes were missing
        if prev.is_none() {
            prev = Self::find_pagination_by_class(
                vdom,
                &[
                    ".pagination-nav__link--prev",
                    "a.nav-prev",
                    "a.previous",
                    ".md-footer__link--prev",
                ],
                base_url,
                policy,
            );
        }

        if next.is_none() {
            next = Self::find_pagination_by_class(
                vdom,
                &[
                    ".pagination-nav__link--next",
                    "a.nav-next",
                    "a.next",
                    ".md-footer__link--next",
                ],
                base_url,
                policy,
            );
        }

        DocPagination { prev, next }
    }

    fn find_pagination_by_class(
        vdom: &tl::VDom<'_>,
        selectors: &[&str],
        base_url: Option<&ValidatedUrl>,
        policy: &UrlSecurityPolicy,
    ) -> Option<DocLink> {
        let parser = vdom.parser();

        for selector in selectors {
            if let Some(handles) = vdom.query_selector(selector) {
                for handle in handles {
                    if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                        if let Some(Some(href)) = tag.attributes().get("href") {
                            let resolved = LinkResolver::resolve_href(
                                href.as_utf8_str().as_ref(),
                                base_url,
                                policy,
                            );
                            if !resolved.is_empty() {
                                let title = Self::extract_pagination_title(tag, parser);
                                return Some(DocLink {
                                    title,
                                    url: resolved,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_pagination_title(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
        // In Docusaurus: title is in <div class="pagination-nav__label">Title</div>
        for child_handle in tag.children().top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                if let Some(Some(class)) = child_tag.attributes().get("class") {
                    let s = class.as_utf8_str();
                    if s.contains("pagination-nav__label") || s.contains("nav-title") {
                        let t = child_tag.inner_text(parser).trim().to_string();
                        if !t.is_empty() {
                            return t;
                        }
                    }
                }
            }
        }

        // Fallback to title attribute or inner text
        if let Some(Some(title_attr)) = tag.attributes().get("title") {
            let t = title_attr.as_utf8_str().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }

        let inner = tag.inner_text(parser);
        inner
            .replace("Previous", "")
            .replace("Next", "")
            .replace(['«', '»', '<', '>'], "")
            .trim()
            .to_string()
    }

    /// Extracts version information from version dropdowns or meta selectors.
    #[must_use]
    pub fn extract_version_info(vdom: &tl::VDom<'_>) -> Option<DocVersionInfo> {
        let parser = vdom.parser();
        let mut available = Vec::new();
        let mut current = None;

        // Check <select class*="version">
        if let Some(handles) = vdom.query_selector("select") {
            for handle in handles {
                if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                    let is_version = tag
                        .attributes()
                        .get("class")
                        .and_then(|v| v)
                        .is_some_and(|b| b.as_utf8_str().to_ascii_lowercase().contains("version"));

                    if is_version {
                        for option_handle in tag.children().top().iter() {
                            if let Some(tl::Node::Tag(opt_tag)) = option_handle.get(parser) {
                                if opt_tag.name().as_utf8_str().eq_ignore_ascii_case("option") {
                                    let text = opt_tag.inner_text(parser).trim().to_string();
                                    if !text.is_empty() {
                                        if opt_tag.attributes().get("selected").is_some() {
                                            current = Some(text.clone());
                                        }
                                        available.push(text);
                                    }
                                }
                            }
                        }
                        if !available.is_empty() {
                            return Some(DocVersionInfo {
                                current_version: current.or_else(|| available.first().cloned()),
                                available_versions: available,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_breadcrumbs() {
        let html = r#"
        <nav aria-label="breadcrumbs">
            <ul class="breadcrumbs">
                <li class="breadcrumbs__item"><a href="/">Home</a></li>
                <li class="breadcrumbs__item"><a href="/docs">Docs</a></li>
                <li class="breadcrumbs__item breadcrumbs__item--active">Getting Started</li>
            </ul>
        </nav>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse_default("https://example.com/docs/page").unwrap();

        let crumbs = DocNavigationExtractor::extract_breadcrumbs(&vdom, Some(&base), &policy);
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[0].label, "Home");
        assert_eq!(crumbs[0].url.as_deref(), Some("https://example.com/"));
        assert_eq!(crumbs[1].label, "Docs");
        assert_eq!(crumbs[1].url.as_deref(), Some("https://example.com/docs"));
        assert_eq!(crumbs[2].label, "Getting Started");
        assert_eq!(crumbs[2].url, None);
    }

    #[test]
    fn test_extract_pagination_links() {
        let html = r#"
        <div class="pagination">
            <a rel="prev" href="/docs/intro" title="Introduction">« Previous</a>
            <a rel="next" href="/docs/config" title="Configuration">Next »</a>
        </div>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse_default("https://example.com/docs/page").unwrap();

        let pag = DocNavigationExtractor::extract_pagination(&vdom, Some(&base), &policy);
        assert!(pag.prev.is_some());
        assert_eq!(pag.prev.as_ref().unwrap().title, "Introduction");
        assert_eq!(
            pag.prev.as_ref().unwrap().url,
            "https://example.com/docs/intro"
        );

        assert!(pag.next.is_some());
        assert_eq!(pag.next.as_ref().unwrap().title, "Configuration");
        assert_eq!(
            pag.next.as_ref().unwrap().url,
            "https://example.com/docs/config"
        );
    }
}
