//! # Token Counting Precision & Metrics
//!
//! Provides strongly-typed distinctions between exact BPE token counts and calibrated estimates,
//! guaranteeing transparency and preventing dishonest token representations.

use serde::{Deserialize, Serialize};

/// Level of precision for a computed token count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenPrecision {
    /// Exact count computed by genuine BPE or SentencePiece tokenization.
    Exact,
    /// Calibrated heuristic estimation backed by empirical language ratios with explicit uncertainty.
    CalibratedEstimate {
        /// Uncertainty percentage bound (e.g. 7.5 represents $\pm 7.5\%$).
        uncertainty_pct: f64,
    },
    /// Generic uncalibrated rule-of-thumb estimate (e.g. character count divided by 4).
    HeuristicEstimate,
}

impl TokenPrecision {
    /// Returns true if the token count is mathematically exact.
    #[must_use]
    pub const fn is_exact(&self) -> bool {
        matches!(self, Self::Exact)
    }

    /// Returns the uncertainty percentage (0.0 for exact counts).
    #[must_use]
    pub fn uncertainty_percentage(&self) -> f64 {
        match self {
            Self::Exact => 0.0,
            Self::CalibratedEstimate { uncertainty_pct } => *uncertainty_pct,
            Self::HeuristicEstimate => 25.0, // Standard 25% uncertainty on rough chars / 4
        }
    }
}

/// Detailed result of token counting for a text segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenCountResult {
    /// Computed or estimated token count.
    pub count: usize,
    /// Precision tier of the count.
    pub precision: TokenPrecision,
    /// Name of the tokenizer adapter used.
    pub tokenizer_name: &'static str,
    /// Total character count.
    pub character_count: usize,
    /// Total word count.
    pub word_count: usize,
    /// Total line count.
    pub line_count: usize,
    /// Empirical character-to-token ratio (characters / tokens).
    pub chars_per_token: f64,
    /// Expected confidence interval $[min, max]$ for estimated counts.
    pub confidence_range: Option<(usize, usize)>,
}

impl TokenCountResult {
    /// Constructs an exact token count result.
    #[must_use]
    pub fn exact(
        count: usize,
        tokenizer_name: &'static str,
        character_count: usize,
        word_count: usize,
        line_count: usize,
    ) -> Self {
        let chars_per_token = if count > 0 {
            (character_count as f64 / count as f64 * 100.0).round() / 100.0
        } else {
            0.0
        };

        Self {
            count,
            precision: TokenPrecision::Exact,
            tokenizer_name,
            character_count,
            word_count,
            line_count,
            chars_per_token,
            confidence_range: None,
        }
    }

    /// Constructs a calibrated estimate token count result.
    #[must_use]
    pub fn calibrated(
        count: usize,
        uncertainty_pct: f64,
        tokenizer_name: &'static str,
        character_count: usize,
        word_count: usize,
        line_count: usize,
    ) -> Self {
        let chars_per_token = if count > 0 {
            (character_count as f64 / count as f64 * 100.0).round() / 100.0
        } else {
            0.0
        };

        let delta = ((count as f64) * (uncertainty_pct / 100.0)).ceil() as usize;
        let min_bound = count.saturating_sub(delta);
        let max_bound = count.saturating_add(delta);

        Self {
            count,
            precision: TokenPrecision::CalibratedEstimate { uncertainty_pct },
            tokenizer_name,
            character_count,
            word_count,
            line_count,
            chars_per_token,
            confidence_range: Some((min_bound, max_bound)),
        }
    }
}

/// Result of truncating a text segment to fit within a token ceiling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruncationResult {
    /// Resulting truncated text content.
    pub truncated_text: String,
    /// Token count prior to truncation.
    pub original_tokens: usize,
    /// Token count after truncation.
    pub resulting_tokens: usize,
    /// True if content had to be truncated to satisfy the ceiling.
    pub was_truncated: bool,
    /// Number of tokens omitted during truncation.
    pub omitted_tokens: usize,
}
