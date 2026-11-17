//! # Content Completeness Scoring & Extraction Quality
//!
//! Provides multidimensional quality assessment, quantitative metrics extraction,
//! heading hierarchy validation, diagnostic warning classification, and completeness
//! grading for extracted webpages.

pub mod model;
pub mod scorer;

pub use model::{
    CompletenessLevel, ContentMetrics, DimensionScores, ExtractionQualityReport, ExtractionWarning,
    ExtractionWarningCategory,
};
pub use scorer::{QualityScorer, QualityScorerOptions};
