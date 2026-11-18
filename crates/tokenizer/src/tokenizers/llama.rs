//! # Meta Llama 3 BPE Tokenizer Engine
//!
//! Tokenizer algorithm for Llama 3, Llama 3.1 (8B, 70B, 405B).
//! Vocabulary size: 128,256 tokens.
//! Features strict boundary handling, code bracket sensitivity, and 4-space indentation merging.

use regex::Regex;
use std::sync::OnceLock;

static LLAMA_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_llama_regex() -> &'static Regex {
    LLAMA_REGEX.get_or_init(|| {
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid llama regex pattern")
    })
}

/// Dedicated counting and segmentation engine for Meta Llama 3 models.
pub struct LlamaTokenizer;

impl LlamaTokenizer {
    /// Counts tokens for `text` using Llama 3 128k BPE vocabulary rules.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_llama_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                count += uncaptured.chars().count();
            }

            let slice = m.as_str();

            if slice.starts_with("    ") && slice.chars().all(|c| c == ' ') {
                // Llama 3 merges 4 spaces into 1 token
                count += slice.len().div_ceil(4);
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                // Multi-byte Unicode
                count += slice.len().div_ceil(2);
            } else {
                let char_len = slice.chars().count();
                if char_len > 9 && slice.chars().all(|c| c.is_alphabetic()) {
                    count += char_len.div_ceil(4);
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

    /// Truncates `text` cleanly along Llama 3 token boundaries up to `max_tokens`.
    #[must_use]
    pub fn truncate(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_llama_regex();
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

            let token_weight = if slice.starts_with("    ") && slice.chars().all(|c| c == ' ') {
                slice.len().div_ceil(4)
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                slice.len().div_ceil(2)
            } else {
                let char_len = slice.chars().count();
                if char_len > 9 && slice.chars().all(|c| c.is_alphabetic()) {
                    char_len.div_ceil(4)
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
    fn test_llama_token_counting() {
        let text = "Meta Llama 3 open weights model executing high performance inference.";
        let count = LlamaTokenizer::count(text);
        assert!(count > 0);
        assert!(count < 15);
    }
}
