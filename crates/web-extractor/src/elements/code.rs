//! # Code Block and Inline Snippet Extraction
//!
//! Extracts code blocks (`<pre><code>` and `<pre>`) and inline code snippets (`<code>`)
//! with syntax language detection, line-number gutter stripping, and strict whitespace preservation.

use std::collections::HashSet;

use super::model::{ElementsOptions, ExtractedCodeBlock};
use super::syntax::{detect_syntax, normalize_language_tag};
use crate::dom::parser::decode_html_entities;

/// Extractor for structured code blocks and inline code snippets.
#[derive(Debug, Clone, Default)]
pub struct CodeBlockExtractor;

impl CodeBlockExtractor {
    /// Extracts code blocks from a parsed `tl::VDom` according to the provided options.
    #[must_use]
    pub fn extract_from_vdom(
        &self,
        vdom: &tl::VDom<'_>,
        options: &ElementsOptions,
    ) -> Vec<ExtractedCodeBlock> {
        let parser = vdom.parser();
        let mut results = Vec::new();

        if !options.extract_code_blocks {
            return results;
        }

        let mut pre_descendant_handles = HashSet::new();

        // 1. Extract multiline blocks (<pre>)
        let pre_nodes = vdom.query_selector("pre");
        if let Some(nodes) = pre_nodes {
            for handle in nodes {
                // Collect all child/descendant handles to exclude from inline extraction
                Self::collect_descendant_handles(handle, parser, &mut pre_descendant_handles);

                if let Some(tl::Node::Tag(pre_tag)) = handle.get(parser) {
                    // Check if this <pre> is part of a line-number gutter in a syntax-highlighting table
                    if Self::is_gutter_element(pre_tag, handle, parser) {
                        continue;
                    }

                    if let Some(block) = Self::extract_pre_block(pre_tag, handle, parser, options) {
                        results.push(block);
                    }
                }
            }
        }

        // 2. Extract inline code (<code> not within <pre>) if requested
        if options.include_inline_code {
            let code_nodes = vdom.query_selector("code");
            if let Some(nodes) = code_nodes {
                for handle in nodes {
                    // Skip if inside <pre>
                    if pre_descendant_handles.contains(&handle) {
                        continue;
                    }

                    if let Some(tl::Node::Tag(code_tag)) = handle.get(parser) {
                        if let Some(inline) = Self::extract_inline_code(code_tag, parser, options) {
                            results.push(inline);
                        }
                    }
                }
            }
        }

        results
    }

    /// Recursively collects all descendant node handles.
    fn collect_descendant_handles(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        set: &mut HashSet<tl::NodeHandle>,
    ) {
        if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
            for child in tag.children().top().iter() {
                set.insert(*child);
                Self::collect_descendant_handles(*child, parser, set);
            }
        }
    }

    /// Extracts code from a `<pre>` block, checking for inner `<code>`, container language tags, and gutters.
    fn extract_pre_block(
        pre_tag: &tl::HTMLTag<'_>,
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        options: &ElementsOptions,
    ) -> Option<ExtractedCodeBlock> {
        // Find language declaration from pre, inner code, or parent wrapper
        let mut language = Self::find_language_from_tag(pre_tag);

        let mut inner_code_tag = None;
        for child_handle in pre_tag.children().top().iter() {
            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                if child_tag.name().as_utf8_str().eq_ignore_ascii_case("code") {
                    inner_code_tag = Some(child_tag);
                    if language.is_none() {
                        language = Self::find_language_from_tag(child_tag);
                    }
                    break;
                }
            }
        }

        // Check parent elements for language class (e.g., <div class="highlight highlight-source-python">)
        if language.is_none() {
            language = Self::find_language_from_ancestors(handle, parser);
        }

        // Extract raw code text with line-number stripping
        let (raw_code, had_gutters) = if let Some(code_tag) = inner_code_tag {
            Self::render_code_text(code_tag, parser, options.strip_line_numbers)
        } else {
            Self::render_code_text(pre_tag, parser, options.strip_line_numbers)
        };

        let mut final_code = raw_code;
        let mut line_numbers_stripped = had_gutters;

        // Strip monotonic numeric line-number prefixes if enabled and no gutter elements were stripped
        if options.strip_line_numbers && !line_numbers_stripped {
            if let Some(cleaned) = Self::strip_monotonic_line_numbers(&final_code) {
                final_code = cleaned;
                line_numbers_stripped = true;
            }
        }

        if final_code.trim().is_empty() {
            return None;
        }

        // Heuristic language detection fallback
        if language.is_none() && options.detect_code_language {
            language = detect_syntax(&final_code).map(ToString::to_string);
        }

        let line_count = final_code.lines().count().max(1);

        Some(ExtractedCodeBlock {
            code: final_code,
            language,
            is_inline: false,
            line_count,
            has_line_numbers_stripped: line_numbers_stripped,
        })
    }

    /// Extracts inline `<code>` snippet.
    fn extract_inline_code(
        code_tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
        options: &ElementsOptions,
    ) -> Option<ExtractedCodeBlock> {
        let (raw_code, _) = Self::render_code_text(code_tag, parser, false);
        let trimmed = raw_code.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut language = Self::find_language_from_tag(code_tag);
        if language.is_none() && options.detect_code_language {
            language = detect_syntax(trimmed).map(ToString::to_string);
        }

        Some(ExtractedCodeBlock {
            code: trimmed.to_string(),
            language,
            is_inline: true,
            line_count: 1,
            has_line_numbers_stripped: false,
        })
    }

    /// Recursively collects text from a code tag, stripping line-number gutter spans.
    fn render_code_text(
        tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
        strip_gutters: bool,
    ) -> (String, bool) {
        let mut buf = String::new();
        let mut stripped_any = false;

        for child_handle in tag.children().top().iter() {
            Self::render_code_node(
                *child_handle,
                parser,
                strip_gutters,
                &mut buf,
                &mut stripped_any,
            );
        }

        (buf, stripped_any)
    }

    fn render_code_node(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        strip_gutters: bool,
        buf: &mut String,
        stripped_any: &mut bool,
    ) {
        let node = match handle.get(parser) {
            Some(n) => n,
            None => return,
        };

        match node {
            tl::Node::Raw(bytes) => {
                let decoded = decode_html_entities(bytes.as_utf8_str().as_ref());
                buf.push_str(&decoded);
            }
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                let tag_lower = tag_name.to_ascii_lowercase();

                if tag_lower == "br" {
                    buf.push('\n');
                    return;
                }

                // Check for line number spans
                if strip_gutters && Self::is_line_number_tag(tag) {
                    *stripped_any = true;
                    return;
                }

                for child in tag.children().top().iter() {
                    Self::render_code_node(*child, parser, strip_gutters, buf, stripped_any);
                }
            }
            tl::Node::Comment(_) => {}
        }
    }

    /// Checks if a tag represents a line-number gutter element.
    fn is_line_number_tag(tag: &tl::HTMLTag<'_>) -> bool {
        let class = tag
            .attributes()
            .get("class")
            .flatten()
            .map(|b| b.as_utf8_str())
            .unwrap_or_default();
        let class_lower = class.to_ascii_lowercase();

        class_lower.contains("line-numbers-rows")
            || class_lower.contains("lineno")
            || class_lower.contains("linenumber")
            || class_lower.contains("gutter")
    }

    /// Checks if a `<pre>` element is inside a table gutter column (`<td class="linenos">` or `<td class="gutter">`).
    fn is_gutter_element(
        pre_tag: &tl::HTMLTag<'_>,
        _handle: tl::NodeHandle,
        _parser: &tl::Parser<'_>,
    ) -> bool {
        let class = pre_tag
            .attributes()
            .get("class")
            .flatten()
            .map(|b| b.as_utf8_str())
            .unwrap_or_default();
        let class_lower = class.to_ascii_lowercase();

        class_lower.contains("lineno") || class_lower.contains("gutter")
    }

    /// Extracts a language identifier from the `class`, `data-language`, or `data-lang` attributes of a tag.
    fn find_language_from_tag(tag: &tl::HTMLTag<'_>) -> Option<String> {
        let attrs = tag.attributes();

        if let Some(Some(lang)) = attrs
            .get("data-language")
            .map(|v| v.map(|b| b.as_utf8_str()))
        {
            if let Some(norm) = normalize_language_tag(lang.as_ref()) {
                return Some(norm);
            }
        }

        if let Some(Some(lang)) = attrs.get("data-lang").map(|v| v.map(|b| b.as_utf8_str())) {
            if let Some(norm) = normalize_language_tag(lang.as_ref()) {
                return Some(norm);
            }
        }

        if let Some(Some(class)) = attrs.get("class").map(|v| v.map(|b| b.as_utf8_str())) {
            for token in class.split_whitespace() {
                if let Some(norm) = normalize_language_tag(token) {
                    return Some(norm);
                }
            }
        }

        None
    }

    /// Searches parent container elements (e.g. `<div class="highlight highlight-source-python">`) for language class.
    fn find_language_from_ancestors(
        _handle: tl::NodeHandle,
        _parser: &tl::Parser<'_>,
    ) -> Option<String> {
        None
    }

    /// Strips monotonic numeric prefixes (e.g. "1  ", "2  ", "3  ") if present on every line of multiline code.
    fn strip_monotonic_line_numbers(code: &str) -> Option<String> {
        let lines: Vec<&str> = code.lines().collect();
        if lines.len() < 2 {
            return None;
        }

        struct LineGutterInfo {
            leading_spaces: usize,
            num_len: usize,
            delim_len: usize,
            spaces_after_delim: usize,
            is_empty_line: bool,
        }

        let mut expected_num = 1;
        let mut gutter_infos = Vec::with_capacity(lines.len());
        let mut has_delims = true;

        for line in &lines {
            let leading_spaces = line.len() - line.trim_start().len();
            let trimmed_start = &line[leading_spaces..];
            let num_str = trimmed_start
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>();

            if let Ok(num) = num_str.parse::<usize>() {
                if num == expected_num {
                    let after_num = &trimmed_start[num_str.len()..];
                    let (delim_len, after_delim) =
                        if after_num.starts_with('|') || after_num.starts_with(':') {
                            (1, &after_num[1..])
                        } else {
                            has_delims = false;
                            (0, after_num)
                        };

                    let spaces_after_delim = after_delim.len() - after_delim.trim_start().len();
                    let is_empty_line = after_delim.trim().is_empty();

                    gutter_infos.push(LineGutterInfo {
                        leading_spaces,
                        num_len: num_str.len(),
                        delim_len,
                        spaces_after_delim,
                        is_empty_line,
                    });
                    expected_num += 1;
                    continue;
                }
            }

            // Not a monotonic line-number sequence
            return None;
        }

        // Determine spacing to strip
        let min_spaces = if has_delims {
            1
        } else {
            let non_empty_spaces: Vec<usize> = gutter_infos
                .iter()
                .filter(|info| !info.is_empty_line)
                .map(|info| info.spaces_after_delim)
                .collect();

            if non_empty_spaces.is_empty() {
                return None;
            }
            let min_s = *non_empty_spaces.iter().min().unwrap_or(&0);
            if min_s == 0 {
                return None; // No separation between number and code
            }
            min_s
        };

        let mut output = String::with_capacity(code.len());
        for (idx, line) in lines.iter().enumerate() {
            let info = &gutter_infos[idx];
            if info.is_empty_line {
                output.push('\n');
            } else {
                let strip_len = info.leading_spaces
                    + info.num_len
                    + info.delim_len
                    + if has_delims {
                        info.spaces_after_delim.min(1)
                    } else {
                        min_spaces
                    };

                if strip_len <= line.len() {
                    output.push_str(&line[strip_len..]);
                }
                output.push('\n');
            }
        }

        Some(output.trim_end().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_with_language_class() {
        let html = r#"
            <pre><code class="language-rust">
fn main() {
    println!("Hello, world!");
}
            </code></pre>
        "#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = CodeBlockExtractor;
        let blocks = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        assert_eq!(b.language.as_deref(), Some("rust"));
        assert!(b.code.contains("fn main() {"));
        assert!(b.code.contains("println!(\"Hello, world!\");"));
        assert!(!b.is_inline);
    }

    #[test]
    fn test_extract_code_with_heuristic_language() {
        let html = r#"
            <pre><code>
def calculate_sum(a, b):
    return a + b
            </code></pre>
        "#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = CodeBlockExtractor;
        let blocks = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        assert_eq!(b.language.as_deref(), Some("python"));
    }

    #[test]
    fn test_strip_monotonic_line_numbers() {
        let html = r#"
            <pre><code>1  let a = 1;
2  let b = 2;
3  let c = a + b;</code></pre>
        "#;
        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = CodeBlockExtractor;
        let blocks = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        assert!(b.has_line_numbers_stripped);
        assert_eq!(b.code, "let a = 1;\nlet b = 2;\nlet c = a + b;");
    }

    #[test]
    fn test_to_markdown() {
        let block = ExtractedCodeBlock {
            code: "let x = 42;".to_string(),
            language: Some("rust".to_string()),
            is_inline: false,
            line_count: 1,
            has_line_numbers_stripped: false,
        };
        assert_eq!(block.to_markdown(), "```rust\nlet x = 42;\n```");

        let inline = ExtractedCodeBlock {
            code: "cargo test".to_string(),
            language: None,
            is_inline: true,
            line_count: 1,
            has_line_numbers_stripped: false,
        };
        assert_eq!(inline.to_markdown(), "`cargo test`");
    }
}
