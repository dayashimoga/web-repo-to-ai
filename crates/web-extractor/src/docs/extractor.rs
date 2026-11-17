//! # Technical Documentation Extractor
//!
//! Orchestrates documentation framework detection, breadcrumb extraction, table of contents
//! generation, callout/admonition discovery, pagination extraction, and clean content rendering.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::admonition::AdmonitionExtractor;
use super::framework::detect_framework;
use super::model::{DocFramework, ExtractedDocumentation};
use super::navigation::DocNavigationExtractor;
use super::toc::extract_table_of_contents;
use crate::dom::parser::DomParser;
use crate::error::WebExtractorError;

/// Documentation extractor isolating technical content and navigation hierarchies.
#[derive(Debug, Clone, Default)]
pub struct DocExtractor {
    dom_parser: DomParser,
    url_security_policy: UrlSecurityPolicy,
}

impl DocExtractor {
    /// Creates a new `DocExtractor` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom `DomParser`.
    #[must_use]
    pub fn with_dom_parser(mut self, parser: DomParser) -> Self {
        self.dom_parser = parser;
        self
    }

    /// Sets a custom `UrlSecurityPolicy`.
    #[must_use]
    pub fn with_policy(mut self, policy: UrlSecurityPolicy) -> Self {
        self.url_security_policy = policy;
        self
    }

    /// Extracts structured technical documentation from HTML markup.
    pub fn extract(
        &self,
        html: &str,
        base_url: Option<&ValidatedUrl>,
    ) -> Result<ExtractedDocumentation, WebExtractorError> {
        // 1. Parse raw HTML into VDom for navigation & framework detection
        let raw_vdom = tl::parse(html, tl::ParserOptions::default()).map_err(|e| {
            WebExtractorError::HtmlParseFailed(format!(
                "failed to parse raw HTML for doc extraction: {e:?}"
            ))
        })?;

        let framework = detect_framework(&raw_vdom);
        let breadcrumbs = DocNavigationExtractor::extract_breadcrumbs(
            &raw_vdom,
            base_url,
            &self.url_security_policy,
        );
        let pagination = DocNavigationExtractor::extract_pagination(
            &raw_vdom,
            base_url,
            &self.url_security_policy,
        );
        let version_info = DocNavigationExtractor::extract_version_info(&raw_vdom);

        // 2. Clean HTML with DomParser to remove boilerplate and ads
        let cleaned = self.dom_parser.parse(html, base_url)?;
        if cleaned.is_empty() {
            return Ok(ExtractedDocumentation {
                title: cleaned
                    .metadata
                    .title
                    .clone()
                    .unwrap_or_else(|| "Documentation".to_string()),
                framework,
                breadcrumbs,
                table_of_contents: Vec::new(),
                pagination,
                version_info,
                admonitions: Vec::new(),
                sections: Vec::new(),
                content_html: String::new(),
                content_text: String::new(),
                word_count: 0,
                reading_time_minutes: 0,
                language: cleaned.metadata.language.clone(),
            });
        }

        // 3. Parse cleaned HTML to discover content container and extract ToC & callouts
        let cleaned_vdom =
            tl::parse(&cleaned.cleaned_html, tl::ParserOptions::default()).map_err(|e| {
                WebExtractorError::HtmlParseFailed(format!(
                    "failed to parse cleaned HTML for doc extraction: {e:?}"
                ))
            })?;
        let parser = cleaned_vdom.parser();

        let content_selectors = Self::get_selectors_for_framework(framework);

        let mut candidate_handle = None;
        for selector in content_selectors {
            if let Some(mut handles) = cleaned_vdom.query_selector(selector) {
                if let Some(h) = handles.next() {
                    candidate_handle = Some(h);
                    break;
                }
            }
        }

        let (table_of_contents, sections, admonitions, content_html, content_text) =
            if let Some(handle) = candidate_handle {
                let (toc, secs) = extract_table_of_contents(handle, parser);
                let adms = AdmonitionExtractor::extract_admonitions(handle, parser);
                let (html, text) = self.dom_parser.render_subtree(handle, parser, base_url);
                (toc, secs, adms, html, text)
            } else {
                // If no specific container selector matched, extract ToC from root handles
                let mut all_toc = Vec::new();
                let mut all_secs = Vec::new();
                let mut all_adms = Vec::new();

                for root_h in cleaned_vdom.children() {
                    let (toc, secs) = extract_table_of_contents(*root_h, parser);
                    all_toc.extend(toc);
                    all_secs.extend(secs);
                    all_adms.extend(AdmonitionExtractor::extract_admonitions(*root_h, parser));
                }

                (
                    all_toc,
                    all_secs,
                    all_adms,
                    cleaned.cleaned_html.clone(),
                    cleaned.cleaned_text.clone(),
                )
            };

        // Determine title: prefer first top-level ToC heading, falling back to metadata
        let title = table_of_contents
            .first()
            .map(|t| t.title.clone())
            .or_else(|| cleaned.metadata.title.clone())
            .unwrap_or_else(|| "Documentation".to_string());

        let word_count = content_text.split_whitespace().count();
        let reading_time_minutes = word_count.div_ceil(200);

        Ok(ExtractedDocumentation {
            title,
            framework,
            breadcrumbs,
            table_of_contents,
            pagination,
            version_info,
            admonitions,
            sections,
            content_html,
            content_text,
            word_count,
            reading_time_minutes,
            language: cleaned.metadata.language.clone(),
        })
    }

    fn get_selectors_for_framework(framework: DocFramework) -> &'static [&'static str] {
        match framework {
            DocFramework::Docusaurus => &[
                "article",
                ".theme-doc-markdown",
                "main",
                ".markdown",
                "div.theme-doc-item-container",
            ],
            DocFramework::MdBook => &["#content", "main#content", "div#content", "main", "article"],
            DocFramework::Sphinx => &[
                "div.document",
                "div.documentwrapper",
                "div.body",
                "main",
                "article",
            ],
            DocFramework::MkDocs => &[".md-content__inner", ".md-content", "article", "main"],
            DocFramework::Rustdoc => &["section#main-content", "main", "article"],
            DocFramework::VitePress => &[".vp-doc", "main", "article"],
            DocFramework::Nextra => &[".nextra-content", "main", "article"],
            DocFramework::GitBook | DocFramework::Generic => &[
                "main",
                "article",
                ".markdown-body",
                ".content",
                ".documentation",
                "#content",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_docusaurus_page() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Installation | Docusaurus</title>
            <meta name="generator" content="Docusaurus v3.1">
        </head>
        <body>
            <nav aria-label="breadcrumbs">
                <ul class="breadcrumbs">
                    <li class="breadcrumbs__item"><a href="/">Docs</a></li>
                    <li class="breadcrumbs__item"><a href="/guides">Guides</a></li>
                    <li class="breadcrumbs__item breadcrumbs__item--active">Installation</li>
                </ul>
            </nav>
            <div class="pagination">
                <a rel="prev" href="/docs/intro" title="Introduction">« Intro</a>
                <a rel="next" href="/docs/config" title="Configuration">Config »</a>
            </div>
            <main>
                <article class="theme-doc-markdown markdown">
                    <h1 id="installation">Installation</h1>
                    <p>To install the tool, run the following command in your terminal.</p>
                    <pre><code class="language-bash">cargo install web-repo-to-ai</code></pre>
                    <h2 id="requirements">System Requirements</h2>
                    <p>Ensure that you have Rust 1.85 or later installed on your system.</p>
                    <div class="theme-admonition theme-admonition-tip alert alert--success">
                        <div class="admonition-heading"><h5>Tip</h5></div>
                        <div class="admonition-content"><p>Use rustup for simple toolchain management.</p></div>
                    </div>
                </article>
            </main>
        </body>
        </html>
        "#;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::Docusaurus);
        assert_eq!(doc.title, "Installation");
        assert_eq!(doc.breadcrumbs.len(), 3);
        assert_eq!(doc.breadcrumbs[0].label, "Docs");
        assert_eq!(doc.breadcrumbs[2].label, "Installation");

        assert!(doc.pagination.prev.is_some());
        assert_eq!(doc.pagination.prev.as_ref().unwrap().title, "Introduction");
        assert!(doc.pagination.next.is_some());

        assert_eq!(doc.table_of_contents.len(), 1);
        assert_eq!(doc.table_of_contents[0].title, "Installation");
        assert_eq!(doc.table_of_contents[0].children.len(), 1);
        assert_eq!(
            doc.table_of_contents[0].children[0].title,
            "System Requirements"
        );

        assert_eq!(doc.admonitions.len(), 1);
        assert_eq!(
            doc.admonitions[0].kind,
            super::super::model::AdmonitionType::Tip
        );
        assert!(doc.admonitions[0].content.contains("Use rustup"));

        assert!(doc.content_text.contains("cargo install"));
        assert!(doc.content_html.contains("<code class=\"language-bash\">"));
    }
}
