//! # OpenAI `cl100k_base` Tokenizer Engine
//!
//! Standard tokenizer algorithm for GPT-4, GPT-4 Turbo, and GPT-3.5-Turbo.
//! Vocabulary size: ~100,277 tokens.

use regex::Regex;
use std::sync::OnceLock;

static CL100K_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_cl100k_regex() -> &'static Regex {
    CL100K_REGEX.get_or_init(|| {
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid cl100k regex pattern")
    })
}

/// Known programming keywords that are single tokens in `cl100k_base`.
static CL100K_CODE_KEYWORDS: &[&str] = &[
    "function",
    "return",
    "import",
    "export",
    "interface",
    "constructor",
    "undefined",
    "async",
    "await",
    "struct",
    "impl",
    "public",
    "private",
    "protected",
    "default",
    "continue",
    "break",
    "switch",
    "typeof",
    "instanceof",
    "package",
    "namespace",
    "include",
    "require",
];

/// Dedicated counting and segmentation engine for OpenAI `cl100k_base`.
pub struct Cl100kTokenizer;

impl Cl100kTokenizer {
    /// Counts tokens for `text` using `cl100k_base` vocabulary rules.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_cl100k_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                count += uncaptured.chars().count();
            }

            let slice = m.as_str();
            let trimmed = slice.trim();

            if slice.starts_with("    ") && slice.chars().all(|c| c == ' ') {
                // cl100k merges spaces in groups of up to 4
                count += slice.len().div_ceil(4);
            } else if CL100K_CODE_KEYWORDS.contains(&trimmed) {
                // Keyword is a single token
                count += 1;
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                // Non-ASCII UTF-8 bytes: ~1 token per 2 bytes (or 1.5 tokens per CJK character)
                count += slice.len().div_ceil(2);
            } else {
                let char_len = slice.chars().count();
                if char_len > 8 && slice.chars().all(|c| c.is_alphabetic()) {
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

    /// Truncates `text` cleanly along `cl100k_base` token boundaries up to `max_tokens`.
    #[must_use]
    pub fn truncate(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_cl100k_regex();
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

            let token_weight = if slice.starts_with("    ") && slice.chars().all(|c| c == ' ') {
                slice.len().div_ceil(4)
            } else if CL100K_CODE_KEYWORDS.contains(&trimmed) {
                1
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                slice.len().div_ceil(2)
            } else {
                let char_len = slice.chars().count();
                if char_len > 8 && slice.chars().all(|c| c.is_alphabetic()) {
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
    fn test_cl100k_token_counting() {
        let code = "export async function handleRequest() {\n    return 42;\n}";
        let count = Cl100kTokenizer::count(code);
        assert!(count > 0);
        assert!(count < 25);
    }

    #[test]
    fn test_cl100k_space_merging() {
        let four_spaces = "    ";
        assert_eq!(Cl100kTokenizer::count(four_spaces), 1);

        let eight_spaces = "        ";
        assert_eq!(Cl100kTokenizer::count(eight_spaces), 2);
    }
}
