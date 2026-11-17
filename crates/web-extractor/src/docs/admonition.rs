//! # Technical Documentation Callouts & Admonition Extraction
//!
//! Detects and extracts callout boxes (Note, Tip, Info, Important, Warning, Caution, Danger)
//! across Docusaurus, Sphinx, MkDocs, mdBook, and GitHub alert formats.

use super::model::{Admonition, AdmonitionType};

/// Discovers and parses admonition callout boxes within a documentation subtree.
pub struct AdmonitionExtractor;

impl AdmonitionExtractor {
    /// Extracts all callouts / admonitions from the provided content element.
    #[must_use]
    pub fn extract_admonitions(
        content_handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
    ) -> Vec<Admonition> {
        let mut admonitions = Vec::new();
        Self::traverse_for_admonitions(content_handle, parser, &mut admonitions);
        admonitions
    }

    fn traverse_for_admonitions(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        admonitions: &mut Vec<Admonition>,
    ) {
        let node = match handle.get(parser) {
            Some(n) => n,
            None => return,
        };

        if let tl::Node::Tag(tag) = node {
            let class_opt = tag
                .attributes()
                .get("class")
                .and_then(|v| v)
                .map(|b| b.as_utf8_str().to_ascii_lowercase());

            if let Some(ref class) = class_opt {
                if let Some(kind) = Self::classify_admonition(class) {
                    let (title, content) = Self::parse_admonition_body(tag, parser);
                    if !content.is_empty() {
                        admonitions.push(Admonition {
                            kind,
                            title,
                            content,
                        });
                        // Do not recurse into nested callouts within the same box
                        return;
                    }
                }
            }

            for child in tag.children().top().iter() {
                Self::traverse_for_admonitions(*child, parser, admonitions);
            }
        }
    }

    fn classify_admonition(class: &str) -> Option<AdmonitionType> {
        let is_admonition_container = class.contains("admonition")
            || class.contains("markdown-alert")
            || class.contains("callout")
            || class.contains("alert--");

        if !is_admonition_container {
            return None;
        }

        if class.contains("note") || class.contains("secondary") {
            Some(AdmonitionType::Note)
        } else if class.contains("tip") || class.contains("success") {
            Some(AdmonitionType::Tip)
        } else if class.contains("info") {
            Some(AdmonitionType::Info)
        } else if class.contains("important") {
            Some(AdmonitionType::Important)
        } else if class.contains("warning") {
            Some(AdmonitionType::Warning)
        } else if class.contains("caution") {
            Some(AdmonitionType::Caution)
        } else if class.contains("danger") || class.contains("error") {
            Some(AdmonitionType::Danger)
        } else {
            Some(AdmonitionType::Note)
        }
    }

    fn parse_admonition_body(
        tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
    ) -> (Option<String>, String) {
        let mut title = None;
        let mut content_parts = Vec::new();

        for child_handle in tag.children().top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                let class_opt = child_tag
                    .attributes()
                    .get("class")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str().to_ascii_lowercase());

                let is_title = class_opt.as_deref().is_some_and(|c| {
                    c.contains("title")
                        || c.contains("admonition-heading")
                        || c.contains("alert-heading")
                });

                if is_title && title.is_none() {
                    let t = child_tag.inner_text(parser).trim().to_string();
                    if !t.is_empty() {
                        title = Some(t);
                    }
                } else {
                    let part = child_tag.inner_text(parser).trim().to_string();
                    if !part.is_empty() {
                        content_parts.push(part);
                    }
                }
            } else if let Some(tl::Node::Raw(bytes)) = child_handle.get(parser) {
                let s = bytes.as_utf8_str().trim().to_string();
                if !s.is_empty() {
                    content_parts.push(s);
                }
            }
        }

        let content = content_parts.join("\n\n");
        (title, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_docusaurus_admonition() {
        let html = r#"
        <div class="theme-admonition theme-admonition-warning alert alert--warning">
            <div class="admonition-heading">
                <h5>Warning</h5>
            </div>
            <div class="admonition-content">
                <p>Never bypass SSL validation in production environments.</p>
            </div>
        </div>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = vdom.parser();
        let handle = vdom
            .query_selector(".theme-admonition")
            .unwrap()
            .next()
            .unwrap();

        let adms = AdmonitionExtractor::extract_admonitions(handle, parser);
        assert_eq!(adms.len(), 1);
        assert_eq!(adms[0].kind, AdmonitionType::Warning);
        assert_eq!(adms[0].title.as_deref(), Some("Warning"));
        assert!(adms[0].content.contains("Never bypass SSL validation"));
    }

    #[test]
    fn test_extract_github_markdown_alert() {
        let html = r#"
        <div class="markdown-alert markdown-alert-tip">
            <p class="markdown-alert-title">Tip</p>
            <p>Use connection pooling to optimize throughput.</p>
        </div>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = vdom.parser();
        let handle = vdom
            .query_selector(".markdown-alert")
            .unwrap()
            .next()
            .unwrap();

        let adms = AdmonitionExtractor::extract_admonitions(handle, parser);
        assert_eq!(adms.len(), 1);
        assert_eq!(adms[0].kind, AdmonitionType::Tip);
        assert_eq!(adms[0].title.as_deref(), Some("Tip"));
        assert!(adms[0].content.contains("Use connection pooling"));
    }
}
