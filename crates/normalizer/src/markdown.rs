//! # Markdown Structural Normalizer
//!
//! Provides deterministic standardization of Markdown code block fences,
//! heading level spacing, bullet list markers, and GFM rectangular tables.

use crate::config::MarkdownConfig;
use regex::Regex;
use std::sync::OnceLock;

static HEADING_RE: OnceLock<Regex> = OnceLock::new();
static FENCE_OPEN_RE: OnceLock<Regex> = OnceLock::new();
static LIST_MARKER_RE: OnceLock<Regex> = OnceLock::new();

fn heading_regex() -> &'static Regex {
    HEADING_RE.get_or_init(|| Regex::new(r"^\s*(#{1,6})\s*(\S.*)$").expect("valid regex"))
}

fn fence_open_regex() -> &'static Regex {
    FENCE_OPEN_RE.get_or_init(|| {
        Regex::new(r"^(`{3,}|~{3,})\s*([a-zA-Z0-9_+#.-]*)\s*$").expect("valid regex")
    })
}

fn list_marker_regex() -> &'static Regex {
    LIST_MARKER_RE.get_or_init(|| Regex::new(r"^(\s*)\*\s+(.*)$").expect("valid regex"))
}

/// Engine for Markdown syntax and structural table canonicalization.
pub struct MarkdownNormalizer;

impl MarkdownNormalizer {
    /// Normalizes Markdown structure and syntax across text lines.
    #[must_use]
    pub fn normalize_markdown(text: &str, config: &MarkdownConfig) -> String {
        if text.is_empty() {
            return String::new();
        }

        let lines: Vec<&str> = text.split('\n').collect();
        let mut output_lines: Vec<String> = Vec::with_capacity(lines.len());
        let mut in_code_block = false;
        let mut table_buffer: Vec<&str> = Vec::new();

        for line in lines {
            let trimmed_line = line.trim();

            // 1. Code block fence detection
            if let Some(caps) = fence_open_regex().captures(trimmed_line) {
                // If we were buffering a table, flush it first
                if !table_buffer.is_empty() {
                    output_lines.extend(normalize_table_block(&table_buffer, config));
                    table_buffer.clear();
                }

                let lang = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if in_code_block {
                    // Closing fence
                    in_code_block = false;
                    output_lines.push("```".to_string());
                } else {
                    // Opening fence
                    in_code_block = true;
                    if config.normalize_code_fences {
                        let clean_lang = lang.trim().to_ascii_lowercase();
                        if clean_lang.is_empty() {
                            output_lines.push("```".to_string());
                        } else {
                            output_lines.push(format!("```{clean_lang}"));
                        }
                    } else {
                        output_lines.push(line.to_string());
                    }
                }
                continue;
            }

            // Inside code block -> preserve line contents exactly
            if in_code_block {
                output_lines.push(line.to_string());
                continue;
            }

            // 2. Table detection (lines starting with | or containing markdown table structure)
            if config.rectangularize_tables && is_table_row(trimmed_line) {
                table_buffer.push(trimmed_line);
                continue;
            } else if !table_buffer.is_empty() {
                // Flush buffered table lines
                output_lines.extend(normalize_table_block(&table_buffer, config));
                table_buffer.clear();
            }

            // 3. Heading level spacing normalization
            if config.normalize_heading_spacing {
                if let Some(caps) = heading_regex().captures(line) {
                    let hashes = &caps[1];
                    let title = &caps[2];
                    output_lines.push(format!("{hashes} {title}"));
                    continue;
                }
            }

            // 4. Bullet list marker standardization (* -> -)
            if config.standardize_list_markers {
                if let Some(caps) = list_marker_regex().captures(line) {
                    let indent = &caps[1];
                    let item_text = &caps[2];
                    output_lines.push(format!("{indent}- {item_text}"));
                    continue;
                }
            }

            output_lines.push(line.to_string());
        }

        // Flush any trailing table lines
        if !table_buffer.is_empty() {
            output_lines.extend(normalize_table_block(&table_buffer, config));
        }

        output_lines.join("\n")
    }
}

/// Returns true if a trimmed line has markdown table row format.
#[inline]
fn is_table_row(trimmed: &str) -> bool {
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|')
}

/// Normalizes a contiguous block of Markdown table rows into a rectangular table.
fn normalize_table_block(table_lines: &[&str], config: &MarkdownConfig) -> Vec<String> {
    if table_lines.len() < 2 {
        // Need at least header + delimiter row
        return table_lines.iter().map(|s| s.to_string()).collect();
    }

    // Parse header cells
    let header_cells = parse_table_cells(table_lines[0]);
    let col_count = header_cells.len();
    if col_count == 0 {
        return table_lines.iter().map(|s| s.to_string()).collect();
    }

    // Check delimiter row
    let delimiter_cells = parse_table_cells(table_lines[1]);
    let is_valid_delimiter = delimiter_cells.iter().all(|c| c.contains('-'));
    if !is_valid_delimiter {
        return table_lines.iter().map(|s| s.to_string()).collect();
    }

    let mut result = Vec::with_capacity(table_lines.len());

    // 1. Format header row
    result.push(format_table_row(
        &header_cells,
        col_count,
        config.compact_table_cells,
    ));

    // 2. Format delimiter row
    let mut normalized_delimiters = Vec::with_capacity(col_count);
    for i in 0..col_count {
        if let Some(existing) = delimiter_cells.get(i) {
            let left_align = existing.starts_with(':');
            let right_align = existing.ends_with(':');
            if left_align && right_align {
                normalized_delimiters.push(":---:");
            } else if right_align {
                normalized_delimiters.push("---:");
            } else {
                normalized_delimiters.push(":---");
            }
        } else {
            normalized_delimiters.push(":---");
        }
    }
    result.push(format_table_row(
        &normalized_delimiters,
        col_count,
        config.compact_table_cells,
    ));

    // 3. Format data rows
    for row_line in &table_lines[2..] {
        let cells = parse_table_cells(row_line);
        result.push(format_table_row(
            &cells,
            col_count,
            config.compact_table_cells,
        ));
    }

    result
}

/// Splits a table row into trimmed cell strings.
fn parse_table_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);

    inner.split('|').map(|c| c.trim()).collect()
}

/// Formats a list of cell strings into a standard markdown table row.
fn format_table_row(cells: &[&str], target_col_count: usize, _compact: bool) -> String {
    let mut out = String::from("|");
    for i in 0..target_col_count {
        let cell_text = cells.get(i).copied().unwrap_or("");
        out.push(' ');
        out.push_str(cell_text);
        out.push_str(" |");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_fence_normalization() {
        let input = "```  RUST\nfn main() {}\n````";
        let config = MarkdownConfig::default();
        let cleaned = MarkdownNormalizer::normalize_markdown(input, &config);
        assert_eq!(cleaned, "```rust\nfn main() {}\n```");
    }

    #[test]
    fn test_heading_spacing_normalization() {
        let input = "##Title Without Space\n###   Title With Many Spaces";
        let config = MarkdownConfig::default();
        let cleaned = MarkdownNormalizer::normalize_markdown(input, &config);
        assert_eq!(
            cleaned,
            "## Title Without Space\n### Title With Many Spaces"
        );
    }

    #[test]
    fn test_list_marker_standardization() {
        let input = "* Item 1\n  * Nested Item\n- Already Dash";
        let config = MarkdownConfig::default();
        let cleaned = MarkdownNormalizer::normalize_markdown(input, &config);
        assert_eq!(cleaned, "- Item 1\n  - Nested Item\n- Already Dash");
    }

    #[test]
    fn test_table_rectangularization() {
        let input =
            "| Name | Age | City |\n|---|---|---|\n| Alice | 30 |\n| Bob | 25 | London | Extra |";
        let config = MarkdownConfig::default();
        let cleaned = MarkdownNormalizer::normalize_markdown(input, &config);
        let expected = "| Name | Age | City |\n| :--- | :--- | :--- |\n| Alice | 30 |  |\n| Bob | 25 | London |";
        assert_eq!(cleaned, expected);
    }
}
