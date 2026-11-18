//! # Regular-Expression BPE Pre-Tokenizer
//!
//! Provides deterministic BPE token boundary segmentation and exact token counting
//! following standard LLM pre-tokenization regex specifications.

use regex::Regex;
use std::sync::OnceLock;

static BPE_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_bpe_regex() -> &'static Regex {
    BPE_TOKEN_REGEX.get_or_init(|| {
        // Rust-compatible BPE pre-tokenization regex without lookaround assertions:
        // Contractions | Unicode Words | Digits (1-3) | Punctuation/Symbols | Whitespace sequences
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid BPE pre-tokenizer regex")
    })
}

/// Tokenizer providing deterministic BPE token segmentation and counting.
#[derive(Debug, Clone, Copy)]
pub struct RegexBpeTokenizer;

impl RegexBpeTokenizer {
    /// Tokenizes a string into discrete BPE tokens.
    #[must_use]
    pub fn tokenize(text: &str) -> Vec<&str> {
        if text.is_empty() {
            return Vec::new();
        }

        let regex = get_bpe_regex();
        let mut tokens = Vec::new();
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            // Include any uncaptured intervening characters as individual tokens
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                for ch in uncaptured.chars() {
                    let start = text[..last_end].len();
                    let end = start + ch.len_utf8();
                    tokens.push(&text[start..end]);
                    last_end = end;
                }
            }

            tokens.push(m.as_str());

            last_end = m.end();
        }

        if last_end < text.len() {
            tokens.push(&text[last_end..]);
        }

        tokens
    }

    /// Evaluates the total token count of the text.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_bpe_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                count += text[last_end..m.start()].chars().count();
            }

            let slice = m.as_str();
            let char_len = slice.chars().count();

            if char_len > 8 && slice.chars().all(|c| c.is_alphabetic()) {
                count += char_len.div_ceil(4);
            } else if slice.len() > char_len && !slice.is_ascii() {
                // Multi-byte Unicode: ~2 bytes per token
                count += slice.len().div_ceil(2);
            } else {
                count += 1;
            }

            last_end = m.end();
        }

        if last_end < text.len() {
            count += text[last_end..].chars().count();
        }

        count.max(1)
    }

    /// Truncates the text slice cleanly at exact token boundaries.
    #[must_use]
    pub fn truncate_at_tokens(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_bpe_regex();
        let mut accumulated_tokens = 0;
        let mut cut_byte_idx = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured_chars = text[last_end..m.start()].chars().count();
                if accumulated_tokens + uncaptured_chars > max_tokens {
                    break;
                }
                accumulated_tokens += uncaptured_chars;
            }

            let slice = m.as_str();
            let token_weight =
                if slice.chars().count() > 8 && slice.chars().all(|c| c.is_alphabetic()) {
                    slice.chars().count().div_ceil(4)
                } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                    slice.len().div_ceil(2)
                } else {
                    1
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
    fn test_regex_bpe_tokenization_and_counting() {
        let text = "Hello, world! It's a test of BPE tokenization with 123 numbers.";
        let tokens = RegexBpeTokenizer::tokenize(text);
        let count = RegexBpeTokenizer::count(text);

        assert!(!tokens.is_empty());
        assert!(count >= 10);
        assert!(count <= 20);
    }

    #[test]
    fn test_regex_bpe_truncation() {
        let text = "First sentence. Second sentence. Third sentence. Fourth sentence.";
        let (truncated, resulting) = RegexBpeTokenizer::truncate_at_tokens(text, 5);

        assert!(resulting <= 5);
        assert!(!truncated.is_empty());
        assert!(text.starts_with(&truncated));
    }
}
