//! # Character Encoding Sanitizer
//!
//! Provides deterministic UTF-8 sanitization, Byte Order Mark (BOM) stripping,
//! null byte elimination, unprintable ASCII control character removal,
//! zero-width space stripping, and typographic punctuation canonicalization.

use crate::config::EncodingConfig;

/// Engine for character encoding repair and Unicode sanitization.
pub struct EncodingSanitizer;

impl EncodingSanitizer {
    /// Sanitizes an input string according to the provided [`EncodingConfig`].
    #[must_use]
    pub fn sanitize_text(text: &str, config: &EncodingConfig) -> String {
        if text.is_empty() {
            return String::new();
        }

        // 1. Strip leading BOM if requested
        let text_without_bom = if config.strip_bom && text.starts_with('\u{FEFF}') {
            &text['\u{FEFF}'.len_utf8()..]
        } else {
            text
        };

        let mut output = String::with_capacity(text_without_bom.len());

        // 2. Iterate characters and apply filters
        for c in text_without_bom.chars() {
            // Null byte check
            if config.strip_null_bytes && c == '\0' {
                continue;
            }

            // Zero-width characters check
            if config.strip_zero_width_chars && is_zero_width_char(c) {
                continue;
            }

            // Unprintable ASCII control characters check (preserve \t, \n, \r)
            if config.strip_control_chars && is_unprintable_ascii_control(c) {
                continue;
            }

            // Typographic punctuation normalization
            if config.normalize_typographic_punctuation {
                match c {
                    // Smart single quotes & apostrophes
                    '‘' | '’' | '‚' | '‛' => output.push('\''),
                    // Smart double quotes
                    '“' | '”' | '„' | '‟' => output.push('"'),
                    // Em-dash & En-dash
                    '—' => output.push_str("--"),
                    '–' => output.push('-'),
                    // Non-breaking space
                    '\u{00A0}' => output.push(' '),
                    // Horizontal ellipsis
                    '…' => output.push_str("..."),
                    _ => output.push(c),
                }
            } else {
                output.push(c);
            }
        }

        output
    }
}

/// Returns true if character is an invisible zero-width or formatting codepoint.
#[inline]
const fn is_zero_width_char(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // Zero Width Space
        | '\u{200C}' // Zero Width Non-Joiner
        | '\u{200D}' // Zero Width Joiner
        | '\u{2060}' // Word Joiner
        | '\u{FEFF}' // Zero Width No-Break Space (BOM)
    )
}

/// Returns true if character is an unprintable ASCII control character.
///
/// Strictly preserves standard whitespace: `\t` (0x09), `\n` (0x0A), and `\r` (0x0D).
#[inline]
const fn is_unprintable_ascii_control(c: char) -> bool {
    let u = c as u32;
    (u < 0x20 && u != 0x09 && u != 0x0A && u != 0x0D) || u == 0x7F
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom_and_null_byte_stripping() {
        let input = "\u{FEFF}Hello\0, World!\0";
        let config = EncodingConfig::default();
        let cleaned = EncodingSanitizer::sanitize_text(input, &config);
        assert_eq!(cleaned, "Hello, World!");
    }

    #[test]
    fn test_zero_width_characters_stripping() {
        let input = "Sec\u{200B}ret\u{200C}Code\u{200D}Block\u{2060}!";
        let config = EncodingConfig::default();
        let cleaned = EncodingSanitizer::sanitize_text(input, &config);
        assert_eq!(cleaned, "SecretCodeBlock!");
    }

    #[test]
    fn test_control_character_stripping_preserves_tabs_and_newlines() {
        let input = "Line 1\x01\x02\tIndented\x1F\nLine 2\r\n";
        let config = EncodingConfig::default();
        let cleaned = EncodingSanitizer::sanitize_text(input, &config);
        assert_eq!(cleaned, "Line 1\tIndented\nLine 2\r\n");
    }

    #[test]
    fn test_typographic_punctuation_normalization() {
        let input = "“Hello,” she said, ‘it’s great—really great…’\u{00A0}done";
        let config = EncodingConfig {
            normalize_typographic_punctuation: true,
            ..Default::default()
        };
        let cleaned = EncodingSanitizer::sanitize_text(input, &config);
        assert_eq!(
            cleaned,
            "\"Hello,\" she said, 'it's great--really great...' done"
        );
    }
}
