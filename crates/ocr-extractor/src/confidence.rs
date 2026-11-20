//! # OCR Confidence Scoring and Statistical Analysis
//!
//! Provides granular confidence score aggregation, distribution statistics
//! (mean, median, standard deviation, min/max), quality tier classification,
//! and low-confidence token detection for quality gating and provenance.

use crate::models::{BoundingBox, OcrWord};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Qualitative confidence rating for OCR recognition quality.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceTier {
    /// High confidence ($\ge 0.80$): reliable for direct AI reasoning and zero-shot ingestion.
    High,
    /// Medium confidence ($0.50 \le c < 0.80$): generally readable, may contain minor character ambiguities.
    #[default]
    Medium,
    /// Low confidence ($< 0.50$): degraded or noisy region; high likelihood of misrecognition.
    Low,
}

impl ConfidenceTier {
    /// Maps a normalized confidence score ($0.0 \dots 1.0$) into a `ConfidenceTier`.
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        if score >= 0.80 {
            Self::High
        } else if score >= 0.50 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    /// Returns a string representation of the tier.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Comprehensive statistical breakdown of OCR confidence scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceStats {
    /// Mean confidence score across words ($0.0 \dots 1.0$).
    pub mean: f64,
    /// Minimum confidence score ($0.0 \dots 1.0$).
    pub min: f64,
    /// Maximum confidence score ($0.0 \dots 1.0$).
    pub max: f64,
    /// Median confidence score ($0.0 \dots 1.0$).
    pub median: f64,
    /// Standard deviation across confidence scores.
    pub std_dev: f64,
    /// Number of words with confidence $\ge 0.80$.
    pub high_count: usize,
    /// Number of words with confidence $0.50 \le c < 0.80$.
    pub medium_count: usize,
    /// Number of words with confidence $< 0.50$.
    pub low_count: usize,
    /// Total number of words evaluated.
    pub total_words: usize,
    /// Qualitative tier derived from the mean score.
    pub tier: ConfidenceTier,
}

impl Default for ConfidenceStats {
    fn default() -> Self {
        Self::empty()
    }
}

impl ConfidenceStats {
    /// Constructs an empty `ConfidenceStats` object representing zero recognized words.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            mean: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            std_dev: 0.0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            total_words: 0,
            tier: ConfidenceTier::Low,
        }
    }

    /// Computes confidence statistics from a slice of normalized scores ($0.0 \dots 1.0$).
    #[must_use]
    pub fn compute_from_scores(scores: &[f64]) -> Self {
        if scores.is_empty() {
            return Self::empty();
        }

        let total_words = scores.len();
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted[0];
        let max = sorted[total_words - 1];

        let sum: f64 = sorted.iter().sum();
        let mean = sum / (total_words as f64);

        let median = if total_words % 2 == 1 {
            sorted[total_words / 2]
        } else {
            let mid = total_words / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        };

        let variance = if total_words > 1 {
            let sum_sq_diff: f64 = sorted.iter().map(|s| (s - mean).powi(2)).sum();
            sum_sq_diff / (total_words as f64)
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for &score in &sorted {
            if score >= 0.80 {
                high_count += 1;
            } else if score >= 0.50 {
                medium_count += 1;
            } else {
                low_count += 1;
            }
        }

        let tier = ConfidenceTier::from_score(mean);

        Self {
            mean,
            min,
            max,
            median,
            std_dev,
            high_count,
            medium_count,
            low_count,
            total_words,
            tier,
        }
    }

    /// Computes confidence statistics from a slice of `OcrWord` tokens.
    #[must_use]
    pub fn compute_from_words(words: &[OcrWord]) -> Self {
        let scores: Vec<f64> = words.iter().map(|w| w.confidence).collect();
        Self::compute_from_scores(&scores)
    }

    /// Percentage of recognized words with high confidence ($\ge 0.80$).
    #[must_use]
    pub fn high_percentage(&self) -> f64 {
        if self.total_words == 0 {
            0.0
        } else {
            (self.high_count as f64 / self.total_words as f64) * 100.0
        }
    }

    /// Percentage of recognized words with low confidence ($< 0.50$).
    #[must_use]
    pub fn low_percentage(&self) -> f64 {
        if self.total_words == 0 {
            0.0
        } else {
            (self.low_count as f64 / self.total_words as f64) * 100.0
        }
    }

    /// Serializes confidence statistics to string key-value attributes for CDM storage.
    #[must_use]
    pub fn to_attributes(&self, prefix: &str) -> BTreeMap<String, String> {
        let mut attrs = BTreeMap::new();
        let p = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}_")
        };

        attrs.insert(format!("{p}mean"), format!("{:.4}", self.mean));
        attrs.insert(format!("{p}min"), format!("{:.4}", self.min));
        attrs.insert(format!("{p}max"), format!("{:.4}", self.max));
        attrs.insert(format!("{p}median"), format!("{:.4}", self.median));
        attrs.insert(format!("{p}std_dev"), format!("{:.4}", self.std_dev));
        attrs.insert(format!("{p}tier"), self.tier.as_str().to_string());
        attrs.insert(format!("{p}high_count"), self.high_count.to_string());
        attrs.insert(format!("{p}medium_count"), self.medium_count.to_string());
        attrs.insert(format!("{p}low_count"), self.low_count.to_string());
        attrs.insert(format!("{p}total_words"), self.total_words.to_string());
        attrs.insert(
            format!("{p}high_pct"),
            format!("{:.1}", self.high_percentage()),
        );
        attrs.insert(
            format!("{p}low_pct"),
            format!("{:.1}", self.low_percentage()),
        );

        attrs
    }
}

/// An individual word identified as having low recognition confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowConfidenceWord {
    /// Recognized word text.
    pub text: String,
    /// Confidence score ($0.0 \dots 1.0$).
    pub confidence: f64,
    /// Bounding box in pixels.
    pub bbox: BoundingBox,
    /// 1-indexed line number.
    pub line_num: usize,
    /// 1-indexed word number in line.
    pub word_num: usize,
}

/// Identifies all words in a collection with confidence strictly below `threshold`.
#[must_use]
pub fn find_low_confidence_words(words: &[OcrWord], threshold: f64) -> Vec<LowConfidenceWord> {
    words
        .iter()
        .filter(|w| w.confidence < threshold)
        .map(|w| LowConfidenceWord {
            text: w.text.clone(),
            confidence: w.confidence,
            bbox: w.bbox,
            line_num: w.line_num,
            word_num: w.word_num,
        })
        .collect()
}

/// Configuration options for filtering and flagging low-confidence OCR tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceFilterOptions {
    /// Words with confidence strictly below this threshold are omitted if `filter_enabled` is true.
    pub min_confidence_cutoff: f64,
    /// Whether to drop words below `min_confidence_cutoff`.
    pub filter_enabled: bool,
    /// Threshold below which words are flagged with warning markers in output text.
    pub low_confidence_threshold: f64,
    /// Prefix or wrapping marker for flagged low-confidence words (e.g. `"[?]"`).
    pub marker: String,
}

impl Default for ConfidenceFilterOptions {
    fn default() -> Self {
        Self {
            min_confidence_cutoff: 0.0,
            filter_enabled: false,
            low_confidence_threshold: 0.50,
            marker: "[?]".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_stats_empty() {
        let stats = ConfidenceStats::empty();
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.tier, ConfidenceTier::Low);
        assert_eq!(stats.high_percentage(), 0.0);
    }

    #[test]
    fn test_confidence_stats_calculation() {
        let scores = vec![0.95, 0.90, 0.85, 0.70, 0.40];
        let stats = ConfidenceStats::compute_from_scores(&scores);

        assert_eq!(stats.total_words, 5);
        assert_eq!(stats.min, 0.40);
        assert_eq!(stats.max, 0.95);
        assert!((stats.mean - 0.76).abs() < 1e-4);
        assert_eq!(stats.median, 0.85);
        assert_eq!(stats.high_count, 3); // 0.95, 0.90, 0.85
        assert_eq!(stats.medium_count, 1); // 0.70
        assert_eq!(stats.low_count, 1); // 0.40
        assert_eq!(stats.tier, ConfidenceTier::Medium);
        assert!((stats.high_percentage() - 60.0).abs() < 1e-4);
        assert!((stats.low_percentage() - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_find_low_confidence_words() {
        let words = vec![
            OcrWord {
                text: "Good".to_string(),
                confidence: 0.92,
                bbox: BoundingBox::new(0.0, 0.0, 50.0, 20.0),
                line_num: 1,
                word_num: 1,
            },
            OcrWord {
                text: "Blurry".to_string(),
                confidence: 0.35,
                bbox: BoundingBox::new(55.0, 0.0, 60.0, 20.0),
                line_num: 1,
                word_num: 2,
            },
        ];

        let low = find_low_confidence_words(&words, 0.50);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].text, "Blurry");
        assert_eq!(low[0].confidence, 0.35);
    }

    #[test]
    fn test_attributes_serialization() {
        let scores = vec![0.95, 0.85];
        let stats = ConfidenceStats::compute_from_scores(&scores);
        let attrs = stats.to_attributes("conf");

        assert_eq!(attrs.get("conf_tier").unwrap(), "high");
        assert_eq!(attrs.get("conf_total_words").unwrap(), "2");
        assert_eq!(attrs.get("conf_high_count").unwrap(), "2");
        assert_eq!(attrs.get("conf_low_count").unwrap(), "0");
    }
}
