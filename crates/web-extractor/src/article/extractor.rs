//! # Article Extractor
//!
//! Orchestrates DOM cleaning, candidate scoring, and structured article extraction.

use std::collections::HashSet;
use web_repo_to_ai_url_security::ValidatedUrl;

use super::model::{ExtractedArticle, ReadabilityMetrics, ReadabilityOptions};
use super::scorer::ReadabilityScorer;
use crate::dom::parser::{CleanedDocument, DomParser};
use crate::error::WebExtractorError;

/// Article extractor that scores and isolates primary editorial content from web pages.
#[derive(Debug, Clone, Default)]
pub struct ArticleExtractor {
    dom_parser: DomParser,
    options: ReadabilityOptions,
}

impl ArticleExtractor {
    /// Creates a new `ArticleExtractor` with default options and DOM parser.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `ArticleExtractor` with customized readability options.
    #[must_use]
    pub fn with_options(mut self, options: ReadabilityOptions) -> Self {
        self.options = options;
        self
    }

    /// Creates an `ArticleExtractor` with a customized `DomParser`.
    #[must_use]
    pub fn with_dom_parser(mut self, parser: DomParser) -> Self {
        self.dom_parser = parser;
        self
    }

    /// Returns a reference to the active readability options.
    #[must_use]
    pub fn options(&self) -> &ReadabilityOptions {
        &self.options
    }

    /// Extracts the primary article content from raw HTML markup.
    pub fn extract(
        &self,
        html: &str,
        base_url: Option<&ValidatedUrl>,
    ) -> Result<ExtractedArticle, WebExtractorError> {
        let cleaned = self.dom_parser.parse(html, base_url)?;
        self.extract_from_cleaned(&cleaned, base_url)
    }

    /// Extracts the primary article from an already cleaned and parsed document.
    pub fn extract_from_cleaned(
        &self,
        cleaned: &CleanedDocument,
        base_url: Option<&ValidatedUrl>,
    ) -> Result<ExtractedArticle, WebExtractorError> {
        if cleaned.is_empty() {
            return Ok(Self::empty_fallback(cleaned));
        }

        // Parse cleaned HTML to discover candidate containers
        let vdom = tl::parse(&cleaned.cleaned_html, tl::ParserOptions::default()).map_err(|e| {
            WebExtractorError::HtmlParseFailed(format!(
                "failed to parse cleaned HTML for article extraction: {e:?}"
            ))
        })?;
        let parser = vdom.parser();

        // Collect candidate tags in order of structural significance
        let candidate_selectors = ["article", "main", "section", "div"];
        let mut evaluated_handles = HashSet::new();
        let mut scored_candidates: Vec<(tl::NodeHandle, ReadabilityMetrics)> = Vec::new();

        for selector in candidate_selectors {
            if let Some(handles) = vdom.query_selector(selector) {
                for handle in handles {
                    if evaluated_handles.insert(handle) {
                        if let Some(metrics) =
                            ReadabilityScorer::score_candidate(handle, parser, &self.options)
                        {
                            scored_candidates.push((handle, metrics));
                        }
                    }
                }
            }
        }

        // Select the winning candidate with the highest composite score
        let best_candidate = scored_candidates
            .into_iter()
            .max_by(|a, b| a.1.final_score.total_cmp(&b.1.final_score));

        if let Some((handle, metrics)) = best_candidate {
            if metrics.final_score >= self.options.min_score_threshold
                && metrics.word_count >= self.options.min_word_count
            {
                // Winner qualified! Render candidate subtree
                let (candidate_tag, candidate_id, candidate_class) =
                    Self::inspect_node_attributes(handle, parser);

                let (content_html, content_text) =
                    self.dom_parser.render_subtree(handle, parser, base_url);

                let title = Self::find_candidate_title(handle, parser)
                    .or_else(|| cleaned.metadata.title.clone())
                    .or_else(|| cleaned.metadata.og_title.clone())
                    .unwrap_or_else(|| "Untitled".to_string());

                let byline = Self::find_candidate_byline(handle, parser)
                    .or_else(|| cleaned.metadata.author.clone());

                let published_time = Self::find_candidate_published_time(handle, parser)
                    .or_else(|| cleaned.metadata.published_time.clone());

                let excerpt = Self::find_candidate_excerpt(handle, parser)
                    .or_else(|| cleaned.metadata.description.clone())
                    .or_else(|| cleaned.metadata.og_description.clone());

                let word_count = content_text.split_whitespace().count();
                let char_count = content_text.len();
                let reading_time_minutes = word_count.div_ceil(200);

                return Ok(ExtractedArticle {
                    title,
                    byline,
                    published_time,
                    content_html,
                    content_text,
                    excerpt,
                    language: cleaned.metadata.language.clone(),
                    word_count,
                    char_count,
                    reading_time_minutes,
                    readability_score: metrics.final_score,
                    is_fallback: false,
                    candidate_tag,
                    candidate_id,
                    candidate_class,
                });
            }
        }

        // Fallback: whole cleaned document
        Ok(Self::whole_document_fallback(cleaned))
    }

    /// Constructs an empty fallback representation.
    fn empty_fallback(cleaned: &CleanedDocument) -> ExtractedArticle {
        ExtractedArticle {
            title: cleaned
                .metadata
                .title
                .clone()
                .or_else(|| cleaned.metadata.og_title.clone())
                .unwrap_or_else(|| "Untitled".to_string()),
            byline: cleaned.metadata.author.clone(),
            published_time: cleaned.metadata.published_time.clone(),
            content_html: String::new(),
            content_text: String::new(),
            excerpt: cleaned
                .metadata
                .description
                .clone()
                .or_else(|| cleaned.metadata.og_description.clone()),
            language: cleaned.metadata.language.clone(),
            word_count: 0,
            char_count: 0,
            reading_time_minutes: 0,
            readability_score: 0.0,
            is_fallback: true,
            candidate_tag: "body".to_string(),
            candidate_id: None,
            candidate_class: None,
        }
    }

    /// Constructs a fallback representation utilizing the entire cleaned document body.
    fn whole_document_fallback(cleaned: &CleanedDocument) -> ExtractedArticle {
        let word_count = cleaned.cleaned_text.split_whitespace().count();
        let char_count = cleaned.cleaned_text.len();
        let reading_time_minutes = word_count.div_ceil(200);

        let excerpt = cleaned
            .metadata
            .description
            .clone()
            .or_else(|| cleaned.metadata.og_description.clone())
            .or_else(|| {
                cleaned
                    .cleaned_text
                    .lines()
                    .find(|l| l.trim().len() >= 40)
                    .map(|s| s.trim().to_string())
            });

        ExtractedArticle {
            title: cleaned
                .metadata
                .title
                .clone()
                .or_else(|| cleaned.metadata.og_title.clone())
                .unwrap_or_else(|| "Untitled".to_string()),
            byline: cleaned.metadata.author.clone(),
            published_time: cleaned.metadata.published_time.clone(),
            content_html: cleaned.cleaned_html.clone(),
            content_text: cleaned.cleaned_text.clone(),
            excerpt,
            language: cleaned.metadata.language.clone(),
            word_count,
            char_count,
            reading_time_minutes,
            readability_score: 0.0,
            is_fallback: true,
            candidate_tag: if cleaned.has_semantic_main {
                "main".to_string()
            } else {
                "body".to_string()
            },
            candidate_id: None,
            candidate_class: None,
        }
    }

    /// Inspects and extracts the tag name, ID, and class attributes of a node.
    fn inspect_node_attributes(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
    ) -> (String, Option<String>, Option<String>) {
        if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
            let name = tag.name().as_utf8_str().to_string();
            let attrs = tag.attributes();
            let id = attrs
                .get("id")
                .and_then(|v| v)
                .map(|b| b.as_utf8_str().to_string());
            let class = attrs
                .get("class")
                .and_then(|v| v)
                .map(|b| b.as_utf8_str().to_string());
            (name, id, class)
        } else {
            ("div".to_string(), None, None)
        }
    }

    /// Recursively discovers an `<h1>` or `<h2>` heading within the candidate container.
    fn find_candidate_title(handle: tl::NodeHandle, parser: &tl::Parser<'_>) -> Option<String> {
        let node = handle.get(parser)?;
        if let tl::Node::Tag(tag) = node {
            let name = tag.name().as_utf8_str();
            let lower = name.to_ascii_lowercase();
            if lower == "h1" || lower == "h2" {
                let text = tag.inner_text(parser);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }

            for child in tag.children().top().iter() {
                if let Some(t) = Self::find_candidate_title(*child, parser) {
                    return Some(t);
                }
            }
        }
        None
    }

    /// Recursively discovers byline or author information within the candidate container.
    fn find_candidate_byline(handle: tl::NodeHandle, parser: &tl::Parser<'_>) -> Option<String> {
        // Priority 1: Specific author indicators (rel="author", class="author", class="author-name")
        if let Some(author) = Self::find_specific_author(handle, parser) {
            return Some(author);
        }

        // Priority 2: General byline container with prefix cleaning
        Self::find_general_byline(handle, parser)
    }

    /// Searches for specific author tags or attributes (`rel="author"` or class containing `"author"`).
    fn find_specific_author(handle: tl::NodeHandle, parser: &tl::Parser<'_>) -> Option<String> {
        let node = handle.get(parser)?;
        if let tl::Node::Tag(tag) = node {
            let attrs = tag.attributes();

            // 1. rel="author"
            if let Some(Some(rel)) = attrs.get("rel") {
                if rel.as_utf8_str().eq_ignore_ascii_case("author") {
                    let text = tag.inner_text(parser);
                    let trimmed = text.trim();
                    let cleaned = trimmed
                        .strip_prefix("By ")
                        .or_else(|| trimmed.strip_prefix("by "))
                        .unwrap_or(trimmed)
                        .trim();
                    if !cleaned.is_empty() {
                        return Some(cleaned.to_string());
                    }
                }
            }

            // 2. class or id matching author
            let class_opt = attrs.get("class").and_then(|v| v).map(|b| b.as_utf8_str());
            let id_opt = attrs.get("id").and_then(|v| v).map(|b| b.as_utf8_str());

            let is_author_attr = |s: &str| {
                let l = s.to_ascii_lowercase();
                l.split_whitespace().any(|c| {
                    c == "author"
                        || c == "author-name"
                        || c == "author_name"
                        || c.contains("author")
                })
            };

            if class_opt.as_deref().is_some_and(is_author_attr)
                || id_opt.as_deref().is_some_and(is_author_attr)
            {
                let text = tag.inner_text(parser);
                let trimmed = text.trim();
                let cleaned = trimmed
                    .strip_prefix("By ")
                    .or_else(|| trimmed.strip_prefix("by "))
                    .unwrap_or(trimmed)
                    .trim();
                if !cleaned.is_empty() && cleaned.len() <= 80 {
                    return Some(cleaned.to_string());
                }
            }

            for child in tag.children().top().iter() {
                if let Some(author) = Self::find_specific_author(*child, parser) {
                    return Some(author);
                }
            }
        }
        None
    }

    /// Searches for general byline containers (`class="byline"` or `"written-by"`).
    fn find_general_byline(handle: tl::NodeHandle, parser: &tl::Parser<'_>) -> Option<String> {
        let node = handle.get(parser)?;
        if let tl::Node::Tag(tag) = node {
            let attrs = tag.attributes();
            let class_opt = attrs.get("class").and_then(|v| v).map(|b| b.as_utf8_str());
            let id_opt = attrs.get("id").and_then(|v| v).map(|b| b.as_utf8_str());

            let is_byline_attr = |s: &str| {
                let l = s.to_ascii_lowercase();
                l.contains("byline") || l.contains("written-by")
            };

            if class_opt.as_deref().is_some_and(is_byline_attr)
                || id_opt.as_deref().is_some_and(is_byline_attr)
            {
                let text = tag.inner_text(parser);
                let trimmed = text.trim();
                let cleaned = trimmed
                    .strip_prefix("Published by ")
                    .or_else(|| trimmed.strip_prefix("published by "))
                    .or_else(|| trimmed.strip_prefix("By "))
                    .or_else(|| trimmed.strip_prefix("by "))
                    .unwrap_or(trimmed)
                    .trim();
                if !cleaned.is_empty() && cleaned.len() <= 100 {
                    return Some(cleaned.to_string());
                }
            }

            for child in tag.children().top().iter() {
                if let Some(author) = Self::find_general_byline(*child, parser) {
                    return Some(author);
                }
            }
        }
        None
    }

    /// Recursively discovers publication time within `<time>` or date tags.
    fn find_candidate_published_time(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
    ) -> Option<String> {
        let node = handle.get(parser)?;
        if let tl::Node::Tag(tag) = node {
            let name = tag.name().as_utf8_str();
            let lower = name.to_ascii_lowercase();

            if lower == "time" {
                let attrs = tag.attributes();
                if let Some(Some(dt)) = attrs.get("datetime") {
                    let s = dt.as_utf8_str().trim().to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
                let text = tag.inner_text(parser).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }

            for child in tag.children().top().iter() {
                if let Some(time) = Self::find_candidate_published_time(*child, parser) {
                    return Some(time);
                }
            }
        }
        None
    }

    /// Recursively discovers an excerpt from the first substantial paragraph.
    fn find_candidate_excerpt(handle: tl::NodeHandle, parser: &tl::Parser<'_>) -> Option<String> {
        let node = handle.get(parser)?;
        if let tl::Node::Tag(tag) = node {
            let name = tag.name().as_utf8_str();
            let lower = name.to_ascii_lowercase();

            if lower == "p" {
                let text = tag.inner_text(parser);
                let trimmed = text.trim();
                if trimmed.len() >= 40 {
                    return Some(trimmed.to_string());
                }
            }

            for child in tag.children().top().iter() {
                if let Some(exc) = Self::find_candidate_excerpt(*child, parser) {
                    return Some(exc);
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
    fn test_extract_article_from_blog_post() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Building High-Performance Parsers in Rust</title>
            <meta name="author" content="Ferris the Crab">
            <meta name="description" content="A deep dive into writing safe and zero-copy parsers in Rust.">
        </head>
        <body>
            <nav class="site-navigation">
                <a href="/">Home</a>
                <a href="/archive">Archive</a>
            </nav>
            <div class="sidebar">
                <h3>Related Posts</h3>
                <ul>
                    <li><a href="/post-1">Post 1</a></li>
                    <li><a href="/post-2">Post 2</a></li>
                </ul>
            </div>
            <article class="post-content entry-body" id="article-123">
                <h1>Building High-Performance Parsers in Rust</h1>
                <div class="byline">By <span class="author">Ferris</span> on <time datetime="2026-11-17">Nov 17, 2026</time></div>
                <p>Writing parsers in Rust offers a unique combination of extreme performance, memory safety, and expressive type modeling that few languages can match.</p>
                <p>When designing an extraction engine for web content, one must balance WHATWG compliance, execution latency, and defense against adversarial compression or recursive bombs.</p>
                <pre><code class="language-rust">fn parse_input(data: &[u8]) -> Result<(), Error> {
    Ok(())
}</code></pre>
                <p>In conclusion, choosing pure safe Rust libraries guarantees that our processing pipelines will never suffer from memory corruption vulnerabilities.</p>
            </article>
            <footer class="site-footer">
                <p>&copy; 2026 Rust Tech Blog. All rights reserved.</p>
            </footer>
        </body>
        </html>
        "#;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(!article.is_fallback);
        assert_eq!(article.candidate_tag, "article");
        assert_eq!(article.candidate_id.as_deref(), Some("article-123"));
        assert_eq!(article.title, "Building High-Performance Parsers in Rust");
        assert_eq!(article.byline.as_deref(), Some("Ferris"));
        assert_eq!(article.published_time.as_deref(), Some("2026-11-17"));
        assert!(article.word_count >= 50);
        assert!(article.readability_score > 30.0);
        assert!(article.content_text.contains("Writing parsers in Rust"));
        assert!(article.content_text.contains("fn parse_input"));
        assert!(!article.content_text.contains("Related Posts"));
        assert!(!article.content_text.contains("All rights reserved"));
    }

    #[test]
    fn test_extract_article_fallback_on_plain_document() {
        let html = r#"
        <html>
        <head><title>Simple Note</title></head>
        <body>
            <p>This is a short note without complex article wrappers or editorial markup.</p>
        </body>
        </html>
        "#;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(article.is_fallback);
        assert_eq!(article.title, "Simple Note");
        assert!(article.content_text.contains("This is a short note"));
    }
}
