//! # Readability Candidate Scorer
//!
//! Evaluates candidate DOM containers against text density, paragraph count,
//! comma count, link density, tag weight, and class/ID heuristics.

use super::heuristics::{calculate_class_weight, calculate_tag_weight, is_unlikely_candidate};
use super::model::{ReadabilityMetrics, ReadabilityOptions};

/// Scorer for evaluating DOM container elements according to readability heuristics.
pub struct ReadabilityScorer;

impl ReadabilityScorer {
    /// Evaluates a candidate DOM node handle and returns its `ReadabilityMetrics`.
    /// Returns `None` if the node is not a valid container element or is an unlikely candidate when filtering is enabled.
    #[must_use]
    pub fn score_candidate(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        options: &ReadabilityOptions,
    ) -> Option<ReadabilityMetrics> {
        let node = handle.get(parser)?;
        let tag = match node {
            tl::Node::Tag(t) => t,
            _ => return None,
        };

        let tag_name = tag.name().as_utf8_str();
        let tag_lower = tag_name.to_ascii_lowercase();

        // Only evaluate container elements capable of wrapping articles
        if !matches!(
            tag_lower.as_str(),
            "article" | "main" | "section" | "div" | "body"
        ) {
            return None;
        }

        let attrs = tag.attributes();
        let id_opt = attrs.get("id").and_then(|v| v).map(|b| b.as_utf8_str());
        let class_opt = attrs.get("class").and_then(|v| v).map(|b| b.as_utf8_str());

        // Check unlikely candidates filter
        if options.strip_unlikely_candidates
            && is_unlikely_candidate(&tag_lower, id_opt.as_deref(), class_opt.as_deref())
        {
            return None;
        }

        let tag_weight = calculate_tag_weight(&tag_lower);
        let class_weight = calculate_class_weight(id_opt.as_deref(), class_opt.as_deref(), options);

        // Subtree traversal accumulator
        let mut text_buf = String::new();
        let mut link_text_buf = String::new();
        let mut paragraph_count = 0;
        let mut heading_count = 0;
        let mut code_block_count = 0;

        Self::traverse_subtree(
            handle,
            parser,
            false,
            &mut text_buf,
            &mut link_text_buf,
            &mut paragraph_count,
            &mut heading_count,
            &mut code_block_count,
        );

        let text_length = text_buf.len();
        let word_count = text_buf.split_whitespace().count();

        // Count commas and semicolons
        let comma_count = text_buf
            .chars()
            .filter(|&c| c == ',' || c == ';' || c == '\u{FF0C}')
            .count();

        let link_text_length = link_text_buf.len();
        let link_density = if text_length > 0 {
            link_text_length as f64 / text_length as f64
        } else {
            0.0
        };

        // Score components:
        // 1. Text length score: 1.0 point per 50 characters, capped at 60.0
        let text_score = ((text_length as f64) / 50.0).min(60.0);

        // 2. Comma score: 1.0 point per comma/semicolon, capped at 30.0
        let comma_score = (comma_count as f64).min(30.0);

        // 3. Paragraph score: 5.0 points per substantial paragraph
        let paragraph_score = (paragraph_count as f64) * 5.0;

        // 4. Heading score: 4.0 points per heading
        let heading_score = (heading_count as f64) * 4.0;

        // 5. Code block score: 10.0 points per code block
        let code_score = (code_block_count as f64) * 10.0;

        let base_score = tag_weight
            + class_weight
            + text_score
            + comma_score
            + paragraph_score
            + heading_score
            + code_score;

        // Link density multiplier
        let density_multiplier = if link_density >= 0.8 {
            0.0
        } else if link_density > options.max_link_density {
            (1.0 - link_density).powi(2)
        } else {
            1.0 - link_density
        };

        let final_score = (base_score * density_multiplier).max(0.0);

        Some(ReadabilityMetrics {
            text_length,
            word_count,
            comma_count,
            link_text_length,
            link_density,
            paragraph_count,
            heading_count,
            code_block_count,
            tag_weight,
            class_weight,
            final_score,
        })
    }

    /// Recursively traverses the candidate subtree accumulating text and structural counts.
    #[allow(clippy::too_many_arguments)]
    fn traverse_subtree(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        inside_anchor: bool,
        text_buf: &mut String,
        link_text_buf: &mut String,
        paragraph_count: &mut usize,
        heading_count: &mut usize,
        code_block_count: &mut usize,
    ) {
        let node = match handle.get(parser) {
            Some(n) => n,
            None => return,
        };

        match node {
            tl::Node::Raw(bytes) => {
                let s = bytes.as_utf8_str();
                text_buf.push_str(s.as_ref());
                if inside_anchor {
                    link_text_buf.push_str(s.as_ref());
                }
            }
            tl::Node::Tag(tag) => {
                let name = tag.name().as_utf8_str();
                let tag_lower = name.to_ascii_lowercase();

                let is_anchor = inside_anchor || tag_lower == "a";

                match tag_lower.as_str() {
                    "p" => {
                        let inner_text = tag.inner_text(parser);
                        if inner_text.trim().len() >= 25 {
                            *paragraph_count += 1;
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        *heading_count += 1;
                    }
                    "pre" => {
                        *code_block_count += 1;
                    }
                    _ => {}
                }

                for child_handle in tag.children().top().iter() {
                    Self::traverse_subtree(
                        *child_handle,
                        parser,
                        is_anchor,
                        text_buf,
                        link_text_buf,
                        paragraph_count,
                        heading_count,
                        code_block_count,
                    );
                }
            }
            tl::Node::Comment(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_article_candidate() {
        let html = r#"
        <article class="post-content">
            <h2>Article Subtitle</h2>
            <p>This is the first paragraph of an in-depth article about systems programming, containing commas, semicolons; and thoughtful prose.</p>
            <p>Here is another substantial paragraph with more details about Rust development, performance optimization, and safety guarantees.</p>
            <pre><code class="language-rust">fn main() { println!("Hello"); }</code></pre>
        </article>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = vdom.parser();
        let opts = ReadabilityOptions::default();

        let handle = vdom.query_selector("article").unwrap().next().unwrap();
        let metrics = ReadabilityScorer::score_candidate(handle, parser, &opts).unwrap();

        assert!(metrics.final_score > 50.0);
        assert_eq!(metrics.paragraph_count, 2);
        assert_eq!(metrics.heading_count, 1);
        assert_eq!(metrics.code_block_count, 1);
        assert!(metrics.comma_count >= 3);
        assert!(metrics.link_density < 0.1);
    }

    #[test]
    fn test_score_nav_link_farm_penalized() {
        let html = r#"
        <div class="sidebar-links">
            <a href="/1">Link One</a>
            <a href="/2">Link Two</a>
            <a href="/3">Link Three</a>
            <a href="/4">Link Four</a>
            <a href="/5">Link Five</a>
        </div>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = vdom.parser();
        let opts = ReadabilityOptions::default();

        let handle = vdom.query_selector("div").unwrap().next().unwrap();
        // Since class contains sidebar, is_unlikely_candidate filters it or link density is ~1.0
        let metrics = ReadabilityScorer::score_candidate(handle, parser, &opts);
        assert!(metrics.is_none() || metrics.unwrap().final_score < 5.0);
    }
}
