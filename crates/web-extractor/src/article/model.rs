//! # Article Extraction Data Models & Options
//!
//! Defines the structured article model, readability metrics, and configuration options.

use serde::{Deserialize, Serialize};

/// Representation of an extracted article with readability heuristics applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedArticle {
    /// Document / article headline extracted from `<h1>`, OpenGraph, or document metadata.
    pub title: String,
    /// Article author or byline (e.g. from meta author, `rel="author"`, or `class="byline"`).
    pub byline: Option<String>,
    /// Publication timestamp if detected (e.g. from `<time datetime="...">` or OpenGraph).
    pub published_time: Option<String>,
    /// Cleaned, boilerplate-free HTML representing the primary article content.
    pub content_html: String,
    /// Clean plain text of the article with normalized paragraph breaks.
    pub content_text: String,
    /// Summary or lead excerpt (first substantial paragraph or meta description).
    pub excerpt: Option<String>,
    /// Document natural language (e.g. "en", "es").
    pub language: Option<String>,
    /// Total word count of the extracted article text.
    pub word_count: usize,
    /// Total character count of the extracted article text.
    pub char_count: usize,
    /// Estimated reading time in minutes (based on standard 200 words-per-minute baseline).
    pub reading_time_minutes: usize,
    /// Readability heuristic score of the selected candidate container.
    pub readability_score: f64,
    /// Indicates whether extraction fell back to the whole document body or `<main>`.
    pub is_fallback: bool,
    /// HTML tag name of the chosen article container (e.g. "article", "main", "div").
    pub candidate_tag: String,
    /// HTML ID of the chosen article container if present.
    pub candidate_id: Option<String>,
    /// HTML class attribute of the chosen article container if present.
    pub candidate_class: Option<String>,
}

impl ExtractedArticle {
    /// Checks whether the extracted article contains any text.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.word_count == 0 || self.content_text.trim().is_empty()
    }
}

/// Detailed metrics computed for an HTML candidate element during readability scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadabilityMetrics {
    /// Total character length of inner text.
    pub text_length: usize,
    /// Total word count.
    pub word_count: usize,
    /// Count of commas and semicolons indicating prose sentence structure.
    pub comma_count: usize,
    /// Character length of text inside anchor (`<a>`) tags.
    pub link_text_length: usize,
    /// Ratio of anchor text to total text (`link_text_length / text_length`).
    pub link_density: f64,
    /// Count of substantial paragraphs (`<p>`).
    pub paragraph_count: usize,
    /// Count of heading elements (`<h1>` through `<h6>`).
    pub heading_count: usize,
    /// Count of code blocks (`<pre>`, `<code>`).
    pub code_block_count: usize,
    /// Structural base score from tag type (e.g. `<article>` vs `<div>`).
    pub tag_weight: f64,
    /// Bonus or penalty score derived from class and ID naming patterns.
    pub class_weight: f64,
    /// Final composite readability score after link density adjustment.
    pub final_score: f64,
}

/// Configuration options controlling readability heuristics and candidate selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadabilityOptions {
    /// Minimum word count required for a candidate to qualify as a primary article (default: 20).
    pub min_word_count: usize,
    /// Minimum composite score required to accept a candidate without falling back (default: 15.0).
    pub min_score_threshold: f64,
    /// Maximum link density threshold before heavy penalties or rejection apply (default: 0.5).
    pub max_link_density: f64,
    /// Whether to apply positive and negative scoring weights based on class and ID attributes.
    pub weight_classes_and_ids: bool,
    /// Whether to filter out elements with classes like "nav", "menu", "sidebar", "widget" before scoring.
    pub strip_unlikely_candidates: bool,
    /// Custom class names that should always receive positive scoring bonuses.
    pub custom_positive_classes: Vec<String>,
    /// Custom class names that should always receive negative penalties.
    pub custom_negative_classes: Vec<String>,
}

impl Default for ReadabilityOptions {
    fn default() -> Self {
        Self {
            min_word_count: 20,
            min_score_threshold: 15.0,
            max_link_density: 0.5,
            weight_classes_and_ids: true,
            strip_unlikely_candidates: true,
            custom_positive_classes: Vec::new(),
            custom_negative_classes: Vec::new(),
        }
    }
}

impl ReadabilityOptions {
    /// Builder constructor starting from default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum word count threshold.
    #[must_use]
    pub fn with_min_word_count(mut self, count: usize) -> Self {
        self.min_word_count = count;
        self
    }

    /// Sets the minimum readability score threshold.
    #[must_use]
    pub fn with_min_score_threshold(mut self, threshold: f64) -> Self {
        self.min_score_threshold = threshold;
        self
    }

    /// Sets the maximum allowable link density.
    #[must_use]
    pub fn with_max_link_density(mut self, density: f64) -> Self {
        self.max_link_density = density;
        self
    }

    /// Toggles class and ID heuristic weighting.
    #[must_use]
    pub fn with_weight_classes_and_ids(mut self, enable: bool) -> Self {
        self.weight_classes_and_ids = enable;
        self
    }

    /// Toggles pruning of unlikely candidate elements.
    #[must_use]
    pub fn with_strip_unlikely_candidates(mut self, enable: bool) -> Self {
        self.strip_unlikely_candidates = enable;
        self
    }

    /// Adds a custom positive class pattern.
    #[must_use]
    pub fn add_positive_class(mut self, class_pattern: impl Into<String>) -> Self {
        self.custom_positive_classes.push(class_pattern.into());
        self
    }

    /// Adds a custom negative class pattern.
    #[must_use]
    pub fn add_negative_class(mut self, class_pattern: impl Into<String>) -> Self {
        self.custom_negative_classes.push(class_pattern.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readability_options_builder() {
        let opts = ReadabilityOptions::new()
            .with_min_word_count(50)
            .with_min_score_threshold(25.0)
            .with_max_link_density(0.4)
            .add_positive_class("custom-article")
            .add_negative_class("ad-box");

        assert_eq!(opts.min_word_count, 50);
        assert!((opts.min_score_threshold - 25.0).abs() < f64::EPSILON);
        assert!((opts.max_link_density - 0.4).abs() < f64::EPSILON);
        assert_eq!(opts.custom_positive_classes, vec!["custom-article"]);
        assert_eq!(opts.custom_negative_classes, vec!["ad-box"]);
    }

    #[test]
    fn test_extracted_article_is_empty() {
        let article = ExtractedArticle {
            title: "Test".to_string(),
            byline: None,
            published_time: None,
            content_html: String::new(),
            content_text: String::new(),
            excerpt: None,
            language: None,
            word_count: 0,
            char_count: 0,
            reading_time_minutes: 0,
            readability_score: 0.0,
            is_fallback: true,
            candidate_tag: "body".to_string(),
            candidate_id: None,
            candidate_class: None,
        };
        assert!(article.is_empty());
    }
}
