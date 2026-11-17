//! # Article Extraction & Readability Engine
//!
//! Provides candidate identification, scoring heuristics, metadata and byline extraction,
//! and clean article generation.

pub mod extractor;
pub mod heuristics;
pub mod model;
pub mod scorer;

pub use extractor::ArticleExtractor;
pub use heuristics::{
    calculate_class_weight, calculate_tag_weight, is_unlikely_candidate, NEGATIVE_PATTERNS,
    POSITIVE_PATTERNS, UNLIKELY_PATTERNS,
};
pub use model::{ExtractedArticle, ReadabilityMetrics, ReadabilityOptions};
pub use scorer::ReadabilityScorer;
