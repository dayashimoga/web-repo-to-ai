//! # DOM Parser and Sanitization Engine
//!
//! Parses HTML documents into clean DOM structures with boilerplate, advertising,
//! and script removal while strictly preserving content hierarchies, tables, and code blocks.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::cleaner::BoilerplateCleaner;
use super::link_resolver::LinkResolver;
use super::metadata::DocumentMetadata;
use super::policy::BoilerplatePolicy;
use crate::error::WebExtractorError;

/// HTML void element tags that do not have closing tags in HTML5.
const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// HTML block-level elements that produce paragraph or section breaks in plain text output.
const BLOCK_TAGS: &[&str] = &[
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "div",
    "section",
    "article",
    "main",
    "header",
    "blockquote",
    "li",
    "tr",
    "pre",
    "hr",
];

/// Representation of a parsed and cleaned HTML document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanedDocument {
    /// Document metadata extracted from head/meta tags.
    pub metadata: DocumentMetadata,
    /// Raw HTML byte length before cleaning.
    pub raw_html_bytes: usize,
    /// Cleaned HTML markup with boilerplate and noise tags pruned.
    pub cleaned_html: String,
    /// Plain text representation extracted from the cleaned DOM.
    pub cleaned_text: String,
    /// Number of boilerplate nodes stripped during sanitization.
    pub removed_nodes_count: usize,
    /// Whether `<main>` or `<article>` semantic boundaries were detected.
    pub has_semantic_main: bool,
}

impl CleanedDocument {
    /// Checks whether any non-empty content was extracted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cleaned_text.trim().is_empty()
    }

    /// Returns the length of the extracted clean text in characters.
    #[must_use]
    pub fn text_length(&self) -> usize {
        self.cleaned_text.chars().count()
    }
}

/// Static HTML DOM parser and boilerplate removal coordinator.
#[derive(Debug, Clone)]
pub struct DomParser {
    boilerplate_policy: BoilerplatePolicy,
    url_security_policy: UrlSecurityPolicy,
}

impl Default for DomParser {
    fn default() -> Self {
        Self::new(BoilerplatePolicy::default())
    }
}

impl DomParser {
    /// Constructs a `DomParser` with the specified `BoilerplatePolicy` and default URL policy.
    #[must_use]
    pub fn new(boilerplate_policy: BoilerplatePolicy) -> Self {
        Self {
            boilerplate_policy,
            url_security_policy: UrlSecurityPolicy::default(),
        }
    }

    /// Sets the URL security policy for link and asset canonicalization.
    #[must_use]
    pub fn with_url_security_policy(mut self, policy: UrlSecurityPolicy) -> Self {
        self.url_security_policy = policy;
        self
    }

    /// Returns a reference to the active `BoilerplatePolicy`.
    #[must_use]
    pub fn boilerplate_policy(&self) -> &BoilerplatePolicy {
        &self.boilerplate_policy
    }

    /// Parses an HTML string, extracts head metadata, prunes boilerplate, resolves links,
    /// and formats clean HTML and structured text representations.
    pub fn parse(
        &self,
        html: &str,
        base_url: Option<&ValidatedUrl>,
    ) -> Result<CleanedDocument, WebExtractorError> {
        let raw_html_bytes = html.len();

        if html.trim().is_empty() {
            return Ok(CleanedDocument {
                metadata: DocumentMetadata::default(),
                raw_html_bytes: 0,
                cleaned_html: String::new(),
                cleaned_text: String::new(),
                removed_nodes_count: 0,
                has_semantic_main: false,
            });
        }

        let vdom = tl::parse(html, tl::ParserOptions::default())
            .map_err(|e| WebExtractorError::Http(format!("HTML DOM parsing error: {e:?}")))?;

        // 1. Extract metadata and compute effective base URL
        let metadata = DocumentMetadata::extract(&vdom, base_url, &self.url_security_policy);
        let effective_base = if let Some(base_str) = &metadata.base_url {
            if let Some(parent_base) = base_url {
                parent_base
                    .join(base_str, &self.url_security_policy)
                    .ok()
                    .or_else(|| base_url.cloned())
            } else {
                ValidatedUrl::parse_default(base_str).ok()
            }
        } else {
            base_url.cloned()
        };

        // 2. Detect semantic boundaries (<main> or <article>)
        let has_semantic_main = vdom
            .query_selector("main, article, [role='main']")
            .into_iter()
            .flatten()
            .next()
            .is_some();

        // 3. Traverse DOM and build clean HTML and clean text
        let parser = vdom.parser();
        let mut cleaned_html = String::with_capacity(raw_html_bytes);
        let mut cleaned_text = String::with_capacity(raw_html_bytes / 2);
        let mut removed_nodes_count = 0;

        for root_handle in vdom.children() {
            self.render_node(
                *root_handle,
                parser,
                effective_base.as_ref(),
                false,
                false,
                &mut cleaned_html,
                &mut cleaned_text,
                &mut removed_nodes_count,
            );
        }

        // Post-process cleaned text: collapse excessive empty lines (more than 2 consecutive newlines)
        let normalized_text = self.normalize_text_newlines(&cleaned_text);

        Ok(CleanedDocument {
            metadata,
            raw_html_bytes,
            cleaned_html: cleaned_html.trim().to_string(),
            cleaned_text: normalized_text.trim().to_string(),
            removed_nodes_count,
            has_semantic_main,
        })
    }

    /// Recursively evaluates and renders a node and its children into output buffers.
    #[allow(clippy::too_many_arguments)]
    fn render_node(
        &self,
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        base_url: Option<&ValidatedUrl>,
        inside_main_or_article: bool,
        inside_pre: bool,
        cleaned_html: &mut String,
        cleaned_text: &mut String,
        removed_nodes_count: &mut usize,
    ) {
        let node = match handle.get(parser) {
            Some(n) => n,
            None => return,
        };

        match node {
            tl::Node::Comment(_) => {
                if self.boilerplate_policy.strip_comments {
                    *removed_nodes_count += 1;
                }
            }
            tl::Node::Raw(bytes) => {
                let text = bytes.as_utf8_str();
                cleaned_html.push_str(text.as_ref());
                let decoded_text = decode_html_entities(text.as_ref());
                cleaned_text.push_str(&decoded_text);
            }
            tl::Node::Tag(tag) => {
                let name = tag.name().as_utf8_str();
                let tag_lower = name.to_ascii_lowercase();

                // Skip entire <head> section (metadata already extracted)
                if tag_lower == "head" {
                    return;
                }

                let id_opt = tag
                    .attributes()
                    .get("id")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str());

                let class_opt = tag
                    .attributes()
                    .get("class")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str());

                let role_opt = tag
                    .attributes()
                    .get("role")
                    .and_then(|v| v)
                    .map(|b| b.as_utf8_str());

                let is_now_main = inside_main_or_article
                    || tag_lower == "main"
                    || tag_lower == "article"
                    || role_opt.as_deref() == Some("main")
                    || role_opt.as_deref() == Some("article");

                // Evaluate node against boilerplate policy
                if BoilerplateCleaner::is_boilerplate(
                    &tag_lower,
                    id_opt.as_deref(),
                    class_opt.as_deref(),
                    role_opt.as_deref(),
                    is_now_main,
                    &self.boilerplate_policy,
                ) {
                    *removed_nodes_count += 1;
                    return; // Prune boilerplate element and all its descendants!
                }

                let is_now_pre = inside_pre || tag_lower == "pre";
                let is_block = BLOCK_TAGS.contains(&tag_lower.as_str());

                // Build opening tag with resolved URLs for links and images
                self.write_opening_tag(tag, &tag_lower, base_url, cleaned_html);

                // Recurse into child nodes
                for child_handle in tag.children().top().iter() {
                    self.render_node(
                        *child_handle,
                        parser,
                        base_url,
                        is_now_main,
                        is_now_pre,
                        cleaned_html,
                        cleaned_text,
                        removed_nodes_count,
                    );
                }

                // Write closing tag if not void element
                if !VOID_TAGS.contains(&tag_lower.as_str()) {
                    cleaned_html.push_str("</");
                    cleaned_html.push_str(&tag_lower);
                    cleaned_html.push('>');
                }

                // Append natural paragraph break to plain text for block tags
                if is_block && !inside_pre {
                    cleaned_text.push('\n');
                }
            }
        }
    }

    /// Renders opening HTML tag with canonicalized URLs for `a[href]` and `img[src]`.
    fn write_opening_tag(
        &self,
        tag: &tl::HTMLTag<'_>,
        tag_lower: &str,
        base_url: Option<&ValidatedUrl>,
        out: &mut String,
    ) {
        out.push('<');
        out.push_str(tag_lower);

        let attrs = tag.attributes();

        match tag_lower {
            "a" => {
                if let Some(Some(href)) = attrs.get("href") {
                    let href_str = href.as_utf8_str();
                    let resolved = LinkResolver::resolve_href(
                        href_str.as_ref(),
                        base_url,
                        &self.url_security_policy,
                    );
                    if !resolved.is_empty() {
                        out.push_str(" href=\"");
                        out.push_str(&resolved);
                        out.push('"');
                    }
                }
                if let Some(Some(title)) = attrs.get("title") {
                    out.push_str(" title=\"");
                    out.push_str(title.as_utf8_str().as_ref());
                    out.push('"');
                }
            }
            "img" => {
                if let Some(Some(src)) = attrs.get("src") {
                    let src_str = src.as_utf8_str();
                    let resolved = LinkResolver::resolve_src(
                        src_str.as_ref(),
                        base_url,
                        &self.url_security_policy,
                    );
                    if !resolved.is_empty() {
                        out.push_str(" src=\"");
                        out.push_str(&resolved);
                        out.push('"');
                    }
                }
                if let Some(Some(alt)) = attrs.get("alt") {
                    out.push_str(" alt=\"");
                    out.push_str(alt.as_utf8_str().as_ref());
                    out.push('"');
                }
            }
            "code" | "pre" => {
                // Preserve language class e.g. class="language-rust"
                if let Some(Some(class)) = attrs.get("class") {
                    out.push_str(" class=\"");
                    out.push_str(class.as_utf8_str().as_ref());
                    out.push('"');
                }
            }
            _ => {
                // For other content tags, preserve id and class if present
                if let Some(Some(id)) = attrs.get("id") {
                    out.push_str(" id=\"");
                    out.push_str(id.as_utf8_str().as_ref());
                    out.push('"');
                }
                if let Some(Some(class)) = attrs.get("class") {
                    out.push_str(" class=\"");
                    out.push_str(class.as_utf8_str().as_ref());
                    out.push('"');
                }
            }
        }

        out.push('>');
    }

    /// Collapses consecutive empty lines down to a clean paragraph break (`\n\n`).
    fn normalize_text_newlines(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut newline_count = 0;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                newline_count += 1;
                if newline_count <= 2 && !result.is_empty() {
                    result.push('\n');
                }
            } else {
                if newline_count > 0 && !result.is_empty() {
                    result.push('\n');
                }
                newline_count = 0;
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }
}

/// Decodes common standard HTML entities for clean plain-text output.
fn decode_html_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&copy;", "©")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let parser = DomParser::default();
        let doc = parser.parse("", None).unwrap();
        assert!(doc.is_empty());
        assert_eq!(doc.raw_html_bytes, 0);
        assert_eq!(doc.removed_nodes_count, 0);
    }

    #[test]
    fn test_boilerplate_removal_simple_page() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Blog Post Title</title>
                <script>console.log("analytics");</script>
                <style>body { color: red; }</style>
            </head>
            <body>
                <nav class="site-nav">
                    <a href="/home">Home</a>
                    <a href="/about">About</a>
                </nav>
                <div class="sidebar-ad banner">
                    <p>Buy this product now!</p>
                </div>
                <main class="entry-content">
                    <h1>Main Article Heading</h1>
                    <p>This is the core body paragraph of the article.</p>
                    <pre><code class="language-rust">fn main() {
    println!("Hello, World!");
}</code></pre>
                </main>
                <div id="cookie-banner" class="cookie-notice">
                    <p>We use cookies to improve experience.</p>
                </div>
                <footer class="site-footer">
                    <p>&copy; 2026 Company Inc.</p>
                </footer>
            </body>
            </html>
        "#;

        let parser = DomParser::default();
        let base_url = ValidatedUrl::parse_default("https://example.com/post-1").unwrap();
        let doc = parser.parse(html, Some(&base_url)).unwrap();

        // 1. Metadata verified
        assert_eq!(doc.metadata.title.as_deref(), Some("Blog Post Title"));

        // 2. Semantic main detected
        assert!(doc.has_semantic_main);

        // 3. Boilerplate pruned
        assert!(!doc.cleaned_html.contains("site-nav"));
        assert!(!doc.cleaned_html.contains("sidebar-ad"));
        assert!(!doc.cleaned_html.contains("Buy this product now!"));
        assert!(!doc.cleaned_html.contains("cookie-banner"));
        assert!(!doc.cleaned_html.contains("site-footer"));
        assert!(!doc.cleaned_html.contains("console.log"));

        // 4. Core content preserved
        assert!(doc.cleaned_html.contains("<h1>Main Article Heading</h1>"));
        assert!(doc
            .cleaned_html
            .contains("<p>This is the core body paragraph of the article.</p>"));

        // 5. Code block indentation preserved exactly (Rule 04)
        assert!(doc
            .cleaned_html
            .contains("fn main() {\n    println!(\"Hello, World!\");\n}"));
        assert!(doc
            .cleaned_text
            .contains("fn main() {\n    println!(\"Hello, World!\");\n}"));

        // 6. Removed nodes count recorded
        assert!(doc.removed_nodes_count > 0);
    }

    #[test]
    fn test_relative_link_and_img_resolution() {
        let html = r##"
            <article>
                <h1>Guide</h1>
                <p>Read the <a href="/docs/guide-2">next chapter</a> or see <a href="#anchor">jump link</a>.</p>
                <img src="images/chart.png" alt="Architecture Diagram">
            </article>
        "##;

        let parser = DomParser::default();
        let base_url = ValidatedUrl::parse_default("https://example.com/docs/chapter-1").unwrap();
        let doc = parser.parse(html, Some(&base_url)).unwrap();

        assert!(doc
            .cleaned_html
            .contains("href=\"https://example.com/docs/guide-2\""));
        assert!(doc.cleaned_html.contains("href=\"#anchor\""));
        assert!(doc
            .cleaned_html
            .contains("src=\"https://example.com/docs/images/chart.png\""));
        assert!(doc.cleaned_html.contains("alt=\"Architecture Diagram\""));
    }
}
