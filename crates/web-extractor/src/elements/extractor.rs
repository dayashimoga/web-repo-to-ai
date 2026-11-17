//! # Structured Elements Orchestration Engine
//!
//! Coordinates the extraction of tables, code blocks, syntax detection, and hyperlinks
//! from raw HTML strings or parsed DOM trees into `ExtractedElements`.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::code::CodeBlockExtractor;
use super::link::LinkExtractor;
use super::model::{ElementsOptions, ExtractedElements};
use super::table::TableExtractor;
use crate::error::WebExtractorError;

/// Coordinator for structured document element extraction.
#[derive(Debug, Clone)]
pub struct ElementsExtractor {
    options: ElementsOptions,
    url_security_policy: UrlSecurityPolicy,
    table_extractor: TableExtractor,
    code_extractor: CodeBlockExtractor,
    link_extractor: LinkExtractor,
}

impl Default for ElementsExtractor {
    fn default() -> Self {
        Self::new(ElementsOptions::default())
    }
}

impl ElementsExtractor {
    /// Constructs an `ElementsExtractor` with the specified options and default security policy.
    #[must_use]
    pub fn new(options: ElementsOptions) -> Self {
        let policy = UrlSecurityPolicy::default();
        Self {
            options,
            url_security_policy: policy.clone(),
            table_extractor: TableExtractor,
            code_extractor: CodeBlockExtractor,
            link_extractor: LinkExtractor::new().with_url_security_policy(policy),
        }
    }

    /// Sets the URL security policy for link resolution and classification.
    #[must_use]
    pub fn with_url_security_policy(mut self, policy: UrlSecurityPolicy) -> Self {
        self.url_security_policy = policy.clone();
        self.link_extractor = self.link_extractor.with_url_security_policy(policy);
        self
    }

    /// Returns a reference to the active `ElementsOptions`.
    #[must_use]
    pub fn options(&self) -> &ElementsOptions {
        &self.options
    }

    /// Parses an HTML string and extracts tables, code blocks, and hyperlinks.
    pub fn extract(
        &self,
        html: &str,
        base_url: Option<&ValidatedUrl>,
    ) -> Result<ExtractedElements, WebExtractorError> {
        if html.trim().is_empty() {
            return Ok(ExtractedElements::default());
        }

        let vdom = tl::parse(html, tl::ParserOptions::default()).map_err(|e| {
            WebExtractorError::HtmlParseFailed(format!("HTML DOM parse error: {e:?}"))
        })?;

        Ok(self.extract_from_vdom(&vdom, base_url))
    }

    /// Extracts tables, code blocks, and hyperlinks from an existing `tl::VDom`.
    #[must_use]
    pub fn extract_from_vdom(
        &self,
        vdom: &tl::VDom<'_>,
        base_url: Option<&ValidatedUrl>,
    ) -> ExtractedElements {
        let tables = self.table_extractor.extract_from_vdom(vdom, &self.options);
        let code_blocks = self.code_extractor.extract_from_vdom(vdom, &self.options);
        let links = self
            .link_extractor
            .extract_from_vdom(vdom, base_url, &self.options);

        ExtractedElements {
            tables,
            code_blocks,
            links,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_combined_document() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Full Feature Page</title></head>
            <body>
                <h1>Overview</h1>
                <p>Welcome to our library! Check out the <a href="/getting-started">Getting Started</a> guide.</p>
                
                <h2>Code Example</h2>
                <pre><code class="language-rust">
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
                </code></pre>

                <h2>Performance Benchmark</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Engine</th>
                            <th>Ops/sec</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Rust</td>
                            <td>1,200,000</td>
                        </tr>
                    </tbody>
                </table>

                <p>Download the <a href="/reports/benchmark-q3.pdf">Full PDF Report</a>.</p>
            </body>
            </html>
        "#;

        let base_url = ValidatedUrl::parse_default("https://docs.rs/my-crate/1.0.0/").unwrap();
        let extractor = ElementsExtractor::default();
        let elements = extractor.extract(html, Some(&base_url)).unwrap();

        assert_eq!(elements.tables.len(), 1);
        assert_eq!(elements.tables[0].headers, vec!["Engine", "Ops/sec"]);
        assert_eq!(elements.tables[0].rows[0], vec!["Rust", "1,200,000"]);

        assert_eq!(elements.code_blocks.len(), 1);
        assert_eq!(elements.code_blocks[0].language.as_deref(), Some("rust"));
        assert!(elements.code_blocks[0].code.contains("pub fn add"));

        assert_eq!(elements.links.len(), 2);
        assert_eq!(elements.links[0].url, "https://docs.rs/getting-started");
        assert_eq!(
            elements.links[0].link_type,
            super::super::model::LinkType::Internal
        );

        assert_eq!(
            elements.links[1].url,
            "https://docs.rs/reports/benchmark-q3.pdf"
        );
        assert_eq!(
            elements.links[1].link_type,
            super::super::model::LinkType::Asset
        );
    }
}
