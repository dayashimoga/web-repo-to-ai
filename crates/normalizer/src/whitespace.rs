//! # Whitespace and Line-Ending Normalizer
//!
//! Provides deterministic canonicalization of line endings, trailing whitespace removal,
//! leading/trailing blank line trimming, and consecutive blank line collapsing.

use crate::config::WhitespaceConfig;

/// Engine for whitespace canonicalization and line-ending normalization.
pub struct WhitespaceNormalizer;

impl WhitespaceNormalizer {
    /// Normalizes whitespace and line endings across raw text.
    ///
    /// # Arguments
    /// * `text` - Input string.
    /// * `config` - Whitespace normalization configuration.
    /// * `is_code` - True if content is source code (enforces code-specific blank line limits).
    #[must_use]
    pub fn normalize_whitespace(text: &str, config: &WhitespaceConfig, is_code: bool) -> String {
        if text.is_empty() {
            return String::new();
        }

        // 1. Line ending canonicalization: convert CRLF and lone CR to standard Unix LF (\n)
        let normalized_line_endings = if config.normalize_line_endings {
            canonicalize_line_endings(text)
        } else {
            text.to_string()
        };

        // 2. Process line by line
        let lines: Vec<&str> = normalized_line_endings.split('\n').collect();
        let max_blank_lines = if is_code {
            config.max_consecutive_blank_lines_code
        } else {
            config.max_consecutive_blank_lines_prose
        };

        let mut processed_lines: Vec<String> = Vec::with_capacity(lines.len());
        let mut consecutive_blank_lines = 0;

        for line in lines {
            let processed_line = if config.trim_trailing_whitespace {
                line.trim_end_matches([' ', '\t'])
            } else {
                line
            };

            let is_blank = processed_line.trim().is_empty();

            if is_blank {
                consecutive_blank_lines += 1;
                if consecutive_blank_lines <= max_blank_lines {
                    processed_lines.push(String::new());
                }
            } else {
                consecutive_blank_lines = 0;
                let final_line = if !is_code && config.collapse_inline_prose_spaces {
                    collapse_inline_spaces(processed_line)
                } else {
                    processed_line.to_string()
                };
                processed_lines.push(final_line);
            }
        }

        // 3. Trim leading blank lines if configured
        let mut start_idx = 0;
        if config.trim_leading_blank_lines {
            while start_idx < processed_lines.len() && processed_lines[start_idx].is_empty() {
                start_idx += 1;
            }
        }

        // 4. Trim trailing blank lines if configured
        let mut end_idx = processed_lines.len();
        if config.trim_trailing_blank_lines {
            while end_idx > start_idx && processed_lines[end_idx - 1].is_empty() {
                end_idx -= 1;
            }
        }

        if start_idx >= end_idx {
            return String::new();
        }

        processed_lines[start_idx..end_idx].join("\n")
    }
}

/// Converts Windows CRLF (`\r\n`) and old Mac CR (`\r`) to standard Unix LF (`\n`).
#[must_use]
pub fn canonicalize_line_endings(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next(); // Consume paired \n in CRLF
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }

    out
}

/// Collapses multiple consecutive non-leading spaces into a single space in prose text.
///
/// Preserves leading indentation and backtick code spans.
fn collapse_inline_spaces(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_backticks = false;
    let mut prev_was_space = false;
    let mut seen_non_space = false;

    for c in line.chars() {
        if c == '`' {
            in_backticks = !in_backticks;
            seen_non_space = true;
            prev_was_space = false;
            out.push(c);
            continue;
        }

        if in_backticks {
            seen_non_space = true;
            out.push(c);
            continue;
        }

        if !seen_non_space {
            // Preserve leading indentation exactly
            if c != ' ' && c != '\t' {
                seen_non_space = true;
            }
            out.push(c);
            continue;
        }

        if c == ' ' {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            prev_was_space = false;
            out.push(c);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crlf_and_cr_canonicalization() {
        let input = "Line 1\r\nLine 2\rLine 3\nLine 4";
        let canonical = canonicalize_line_endings(input);
        assert_eq!(canonical, "Line 1\nLine 2\nLine 3\nLine 4");
    }

    #[test]
    fn test_trailing_whitespace_and_blank_lines() {
        let input = "\n\n  \nLine 1   \n\n\n\nLine 2\t\t\n\n\n";
        let config = WhitespaceConfig::default();
        let cleaned = WhitespaceNormalizer::normalize_whitespace(input, &config, false);
        // Should trim leading/trailing blank lines, trailing whitespace, and collapse to 1 blank line in prose
        assert_eq!(cleaned, "Line 1\n\nLine 2");
    }

    #[test]
    fn test_code_whitespace_preserves_indentation_and_ceiling() {
        let code =
            "\n\n    fn compute() {\n        let x = 1;   \n\n\n\n\n        let y = 2;\n    }\n\n";
        let config = WhitespaceConfig::default();
        let cleaned = WhitespaceNormalizer::normalize_whitespace(code, &config, true);
        // Code allows up to 2 blank lines and preserves indentation
        assert_eq!(
            cleaned,
            "    fn compute() {\n        let x = 1;\n\n\n        let y = 2;\n    }"
        );
    }

    #[test]
    fn test_collapse_inline_prose_spaces_preserves_code_spans() {
        let line = "This   is    prose   with  `some   code   here`   end.";
        let collapsed = collapse_inline_spaces(line);
        assert_eq!(collapsed, "This is prose with `some   code   here` end.");
    }
}
