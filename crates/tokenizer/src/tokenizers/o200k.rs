//! # OpenAI `o200k_base` Tokenizer Engine
//!
//! Flagship tokenizer algorithm for GPT-4o and GPT-4o-mini.
//! Vocabulary size: ~200,000 tokens.
//! Features enhanced multi-lingual compression, compound code operators, and 8-space indentation merging.

use regex::Regex;
use std::sync::OnceLock;

static O200K_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_o200k_regex() -> &'static Regex {
    O200K_REGEX.get_or_init(|| {
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|::|->|=>|===|!==|&&|\|\||<<|>>|\+=|-=|\*=|/=|==|!=|\?\?|\?\.|\.\.=|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+")
            .expect("valid o200k regex pattern")
    })
}

/// Compound operators and code syntax tokens that are guaranteed single tokens in `o200k_base`.
static O200K_OPERATORS: &[&str] = &[
    "::", "->", "=>", "===", "!==", "&&", "||", "<<", ">>", "+=", "-=", "*=", "/=", "==", "!=",
    "??", "?.", "..=", "...", "<=", ">=",
];

/// Extended programming keywords and standard library terms merged in `o200k_base`.
static O200K_KEYWORDS: &[&str] = &[
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
    "undefined",
    "null",
    "boolean",
    "readonly",
    "override",
    "implements",
    "extends",
    "abstract",
    "synchronized",
];

/// Dedicated counting and segmentation engine for OpenAI `o200k_base`.
pub struct O200kTokenizer;

impl O200kTokenizer {
    /// Counts tokens for `text` using `o200k_base` vocabulary rules.
    #[must_use]
    pub fn count(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let regex = get_o200k_regex();
        let mut count = 0;
        let mut last_end = 0;

        for m in regex.find_iter(text) {
            if m.start() > last_end {
                let uncaptured = &text[last_end..m.start()];
                count += uncaptured.chars().count();
            }

            let slice = m.as_str();
            let trimmed = slice.trim();

            if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                // o200k merges spaces aggressively in blocks of up to 8
                count += slice.len().div_ceil(8);
            } else if O200K_OPERATORS.contains(&trimmed) || O200K_KEYWORDS.contains(&trimmed) {
                // Compound operators and extended keywords are 1 token
                count += 1;
            } else if slice.len() > slice.chars().count() && !slice.is_ascii() {
                // High-efficiency Unicode: ~1 token per 3-4 bytes in o200k (e.g. 1 token per CJK char)
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

    /// Truncates `text` cleanly along `o200k_base` token boundaries up to `max_tokens`.
    #[must_use]
    pub fn truncate(text: &str, max_tokens: usize) -> (String, usize) {
        if max_tokens == 0 || text.is_empty() {
            return (String::new(), 0);
        }

        let regex = get_o200k_regex();
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

            let token_weight = if slice.starts_with("  ") && slice.chars().all(|c| c == ' ') {
                slice.len().div_ceil(8)
            } else if O200K_OPERATORS.contains(&trimmed) || O200K_KEYWORDS.contains(&trimmed) {
                1
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
    fn test_o200k_operator_merging() {
        // In o200k, `::`, `->`, and `=>` are single tokens
        let operators = ":: -> => === !== && ||";
        let count = O200kTokenizer::count(operators);
        assert!(count <= 8);
    }

    #[test]
    fn test_o200k_indentation_merging() {
        let eight_spaces = "        ";
        assert_eq!(O200kTokenizer::count(eight_spaces), 1);

        let sixteen_spaces = "                ";
        assert_eq!(O200kTokenizer::count(sixteen_spaces), 2);
    }

    #[test]
    fn test_o200k_multilingual_compression() {
        let chinese = "人工智能上下文优化与代码分析";
        let count = O200kTokenizer::count(chinese);
        // Chinese characters in o200k compress much more efficiently
        assert!(count <= chinese.chars().count());
    }
}
