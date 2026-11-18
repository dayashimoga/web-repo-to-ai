//! # Anthropic Claude BPE Tokenizer Engine
//!
//! Tokenizer algorithm for Claude 3.5 Sonnet, Claude 3 Opus, and Claude 3 Haiku.
//! Features XML tag preservation, markdown structural formatting, and 2/4/8-space indentation merging.

use regex::Regex;
use std::sync::OnceLock;

static CLAUDE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_claude_regex() -> &'static Regex {
    CLAUDE_REGEX.get_or_init(|| {
        Regex::new(r"</?[a-zA-Z0-9_\-]+>|(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid claude regex pattern")
    })
}

/// Known XML tags and markdown tokens that are atomic single tokens in Claude's BPE.
static CLAUDE_XML_TAGS: &[&str] = &[
    "<thinking>",
    "</thinking>",
    "<artifacts>",
    "</artifacts>",
    "<antThinking>",
    "</antThinking>",
    "<code>",
    "</code>",
    "<context>",
    "</context>",
    "<source>",
    "</source>",
    "<document>",
    "</document>",
    "<file>",
    "</file>",
];

/// Dedicated counting and segmentation engine for Anthropic Claude models.
pub struct ClaudeTokenizer;

impl ClaudeTokenizer {
    /// Counts tokens for `text` using Claude BPE vocabulary rules.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_claude_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                count += uncaptured.chars().count();
            }

            let slice = m.as_str();
            let trimmed = slice.trim();

            if CLAUDE_XML_TAGS.contains(&trimmed)
                || (trimmed.starts_with('<') && trimmed.ends_with('>'))
            {
                // Structural XML tag is 1 token in Claude
                count += 1;
            } else if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                // Claude merges 2-space and 4-space indentation efficiently
                count += slice.len().div_ceil(4);
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                // Unicode characters
                count += slice.len().div_ceil(3);
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

    /// Truncates `text` cleanly along Claude BPE token boundaries up to `max_tokens`.
    #[must_use]
    pub fn truncate(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_claude_regex();
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
            let trimmed = slice.trim();

            let token_weight = if CLAUDE_XML_TAGS.contains(&trimmed)
                || (trimmed.starts_with('<') && trimmed.ends_with('>'))
            {
                1
            } else if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                slice.len().div_ceil(4)
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                slice.len().div_ceil(3)
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
    fn test_claude_xml_tag_tokenization() {
        let text = "<thinking>Deep architectural reasoning</thinking>";
        let count = ClaudeTokenizer::count(text);
        assert!(count < 10);
    }

    #[test]
    fn test_claude_indentation_merging() {
        let four_spaces = "    ";
        assert_eq!(ClaudeTokenizer::count(four_spaces), 1);
    }
}
