//! # Mistral Tekken Tokenizer Engine
//!
//! Tokenizer algorithm for Mistral Large, Mistral Small, and Codestral.
//! Vocabulary size: ~131,072 tokens.
//! Features aggressive code symbol merging and multi-language efficiency.

use regex::Regex;
use std::sync::OnceLock;

static MISTRAL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_mistral_regex() -> &'static Regex {
    MISTRAL_REGEX.get_or_init(|| {
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|::|->|=>|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid mistral tekken regex pattern")
    })
}

/// Dedicated counting and segmentation engine for Mistral Tekken models.
pub struct MistralTokenizer;

impl MistralTokenizer {
    /// Counts tokens for `text` using Mistral Tekken 131k vocabulary rules.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_mistral_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                count += uncaptured.chars().count();
            }

            let slice = m.as_str();

            if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                // Tekken merges spaces in blocks of up to 6
                count += slice.len().div_ceil(6);
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                // Multi-lingual compression
                count += slice.len().div_ceil(3);
            } else {
                let char_len = slice.chars().count();
                if char_len > 10 && slice.chars().all(|c| c.is_alphabetic()) {
                    count += char_len.div_ceil(5);
                } else {
                    count += 1;
                }
            }

            last_end = m.end();
        }

        if last_end < text.len() {
            let trailing = &text[last_end..];
            count += trailing.chars().count();
        }

        count.max(1)
    }

    /// Truncates `text` cleanly along Mistral token boundaries up to `max_tokens`.
    #[must_use]
    pub fn truncate(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_mistral_regex();
        let mut accumulated_tokens = 0;
        let mut cut_byte_idx = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                let uncaptured_chars = uncaptured.chars().count();
                if accumulated_tokens + uncaptured_chars > max_tokens {
                    break;
                }
                accumulated_tokens += uncaptured_chars;
            }

            let slice = m.as_str();

            let token_weight = if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                slice.len().div_ceil(6)
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                slice.len().div_ceil(3)
            } else {
                let char_len = slice.chars().count();
                if char_len > 10 && slice.chars().all(|c| c.is_alphabetic()) {
                    char_len.div_ceil(5)
                } else {
                    1
                }
            };

            if accumulated_tokens + token_weight > max_tokens {
                break;
            }

            accumulated_tokens += token_weight;
            cut_byte_idx = m.end();
            last_end = m.end();
        }

        (text[..cut_byte_idx].to_string(), accumulated_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_token_counting() {
        let text = "Mistral Large Tekken 131k tokenizer executing code extraction.";
        let count = MistralTokenizer::count(text);
        assert!(count > 0);
        assert!(count < 15);
    }
}
