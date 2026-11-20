//! # Token Accounting & Overlap Slicing
//!
//! Provides model-aware token counting and boundary-preserving overlap extraction.

use std::sync::Arc;
use web_repo_to_ai_tokenizer::{
    CalibratedHeuristicAdapter, TargetModel, TokenizerAdapter, TokenizerRegistry,
};

/// Token counting engine supporting model-specific BPE tokenizers and calibrated heuristics.
#[derive(Clone)]
pub struct TokenCounter {
    adapter: Arc<dyn TokenizerAdapter>,
    estimator_name: String,
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new(None)
    }
}

impl TokenCounter {
    /// Creates a new `TokenCounter` for the requested model or fallback heuristic.
    #[must_use]
    pub fn new(model_name: Option<&str>) -> Self {
        let registry = TokenizerRegistry::default();
        if let Some(name) = model_name {
            if let Some(target) = TargetModel::from_name(name) {
                if let Some(adapter) = registry.get_for_model(target) {
                    return Self {
                        estimator_name: format!("exact:{}", adapter.name()),
                        adapter,
                    };
                }
            }
            if let Some(adapter) = registry.get_by_name(name) {
                return Self {
                    estimator_name: format!("exact:{}", adapter.name()),
                    adapter,
                };
            }
        }

        let heuristic = Arc::new(CalibratedHeuristicAdapter::default());
        Self {
            estimator_name: "calibrated_heuristic".to_string(),
            adapter: heuristic,
        }
    }

    /// Counts tokens in the provided string.
    #[must_use]
    pub fn count(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        self.adapter.count_tokens(text).count
    }

    /// Returns the descriptive name of the estimator / tokenizer in use.
    #[must_use]
    pub fn estimator_name(&self) -> &str {
        &self.estimator_name
    }

    /// Slices text to fit within `max_tokens` while favoring line or sentence boundaries.
    #[must_use]
    pub fn truncate_to_tokens<'a>(&self, text: &'a str, max_tokens: usize) -> &'a str {
        if text.is_empty() || max_tokens == 0 {
            return "";
        }
        if self.count(text) <= max_tokens {
            return text;
        }

        // Binary search over line boundaries first
        let lines: Vec<&'a str> = text.lines().collect();
        if lines.is_empty() {
            return "";
        }

        let mut low = 0;
        let mut high = lines.len();
        let mut best_end = 0;

        while low < high {
            let mid = (low + high) / 2;
            let candidate_len = lines[..=mid]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>()
                .saturating_sub(1)
                .min(text.len());
            let candidate = &text[..candidate_len];

            if self.count(candidate) <= max_tokens {
                best_end = candidate_len;
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if best_end > 0 {
            &text[..best_end]
        } else {
            // Even a single line exceeds max_tokens; truncate within line using character boundary
            let chars: Vec<(usize, char)> = text.char_indices().collect();
            let mut c_low = 0;
            let mut c_high = chars.len();
            let mut c_best = 0;

            while c_low < c_high {
                let mid = (c_low + c_high) / 2;
                let byte_idx = chars[mid].0;
                let candidate = &text[..byte_idx];
                if self.count(candidate) <= max_tokens {
                    c_best = byte_idx;
                    c_low = mid + 1;
                } else {
                    c_high = mid;
                }
            }
            &text[..c_best]
        }
    }

    /// Extracts approximately `overlap_tokens` from the tail of `text` favoring line boundaries.
    #[must_use]
    pub fn extract_tail_overlap<'a>(&self, text: &'a str, overlap_tokens: usize) -> &'a str {
        if text.is_empty() || overlap_tokens == 0 {
            return "";
        }
        if self.count(text) <= overlap_tokens {
            return text;
        }

        let lines: Vec<&'a str> = text.lines().collect();
        if lines.is_empty() {
            return "";
        }

        let mut accumulated_tokens = 0;
        let mut start_line_idx = lines.len();

        for (i, line) in lines.iter().enumerate().rev() {
            let line_toks = self.count(line) + 1; // +1 for newline
            if accumulated_tokens + line_toks > overlap_tokens && accumulated_tokens > 0 {
                break;
            }
            accumulated_tokens += line_toks;
            start_line_idx = i;
            if accumulated_tokens >= overlap_tokens {
                break;
            }
        }

        if start_line_idx < lines.len() {
            // Find start byte offset of lines[start_line_idx]
            let mut byte_offset = 0;
            for line in &lines[..start_line_idx] {
                byte_offset += line.len() + 1; // including newline
            }
            let byte_offset = byte_offset.min(text.len());
            &text[byte_offset..]
        } else {
            ""
        }
    }
}
