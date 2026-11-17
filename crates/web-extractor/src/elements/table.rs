//! # HTML Table Extraction and Matrix Normalization
//!
//! Extracts HTML `<table>` elements into rectangular data matrices, resolving `colspan`,
//! `rowspan`, captions, and column alignments with conversion to GitHub Flavored Markdown.

use super::model::{ElementsOptions, ExtractedTable, TableAlignment};
use crate::dom::parser::decode_html_entities;

/// Extractor for structured HTML tables.
#[derive(Debug, Clone, Default)]
pub struct TableExtractor;

impl TableExtractor {
    /// Extracts all eligible tables from a parsed `tl::VDom`.
    #[must_use]
    pub fn extract_from_vdom(
        &self,
        vdom: &tl::VDom<'_>,
        options: &ElementsOptions,
    ) -> Vec<ExtractedTable> {
        let parser = vdom.parser();
        let mut results = Vec::new();

        if !options.extract_tables {
            return results;
        }

        let table_nodes = vdom.query_selector("table");
        if let Some(nodes) = table_nodes {
            for handle in nodes {
                if let Some(tl::Node::Tag(table_tag)) = handle.get(parser) {
                    // Skip syntax highlighter gutter tables (e.g., Pygments, Rouge)
                    if Self::is_syntax_highlighter_table(table_tag) {
                        continue;
                    }

                    if let Some(table) = Self::extract_single_table(table_tag, parser, options) {
                        if table.num_rows >= options.min_table_rows {
                            results.push(table);
                        }
                    }
                }
            }
        }

        results
    }

    /// Checks if a `<table>` is a line-number gutter container from a code highlighter.
    fn is_syntax_highlighter_table(tag: &tl::HTMLTag<'_>) -> bool {
        let class = tag
            .attributes()
            .get("class")
            .flatten()
            .map(|b| b.as_utf8_str())
            .unwrap_or_default();
        let class_lower = class.to_ascii_lowercase();

        class_lower.contains("highlighttable")
            || class_lower.contains("rouge-table")
            || class_lower.contains("syntax-table")
    }

    /// Extracts a single HTML table into an `ExtractedTable` representation.
    fn extract_single_table(
        table_tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
        _options: &ElementsOptions,
    ) -> Option<ExtractedTable> {
        let caption = Self::extract_caption(table_tag, parser);

        // Find column alignments from <colgroup> if present
        let mut col_alignments = Self::extract_colgroup_alignments(table_tag, parser);

        // Extract sections: thead rows, tbody rows, tfoot rows, direct tr rows
        let mut thead_rows = Vec::new();
        let mut body_rows = Vec::new();

        for child_handle in table_tag.children().top().iter() {
            if let Some(tl::Node::Tag(tag)) = child_handle.get(parser) {
                let name = tag.name().as_utf8_str();
                let lower = name.to_ascii_lowercase();

                match lower.as_str() {
                    "thead" => {
                        Self::collect_tr_handles(*child_handle, parser, &mut thead_rows);
                    }
                    "tbody" | "tfoot" => {
                        Self::collect_tr_handles(*child_handle, parser, &mut body_rows);
                    }
                    "tr" => {
                        body_rows.push(*child_handle);
                    }
                    _ => {}
                }
            }
        }

        if thead_rows.is_empty() && body_rows.is_empty() {
            return None;
        }

        // Build grid considering colspan and rowspan
        let all_input_rows: Vec<(tl::NodeHandle, bool)> = thead_rows
            .into_iter()
            .map(|h| (h, true))
            .chain(body_rows.into_iter().map(|h| (h, false)))
            .collect();

        let (mut grid, header_row_count, alignments) =
            Self::build_normalized_grid(&all_input_rows, parser);

        if grid.is_empty() {
            return None;
        }

        // Merge column alignments
        if col_alignments.len() < alignments.len() {
            col_alignments.resize(alignments.len(), TableAlignment::None);
        }
        for (idx, align) in alignments.iter().enumerate() {
            if *align != TableAlignment::None && idx < col_alignments.len() {
                col_alignments[idx] = *align;
            }
        }

        // Determine headers vs data rows
        let (headers, rows, has_headers) = if header_row_count > 0 {
            // First header row as headers
            let headers = grid.remove(0);
            (headers, grid, true)
        } else {
            (Vec::new(), grid, false)
        };

        let num_cols = if !headers.is_empty() {
            headers.len()
        } else if let Some(first) = rows.first() {
            first.len()
        } else {
            0
        };

        if num_cols == 0 {
            return None;
        }

        let num_rows = if has_headers {
            rows.len() + 1
        } else {
            rows.len()
        };

        // Ensure alignments match num_cols
        col_alignments.resize(num_cols, TableAlignment::None);

        Some(ExtractedTable {
            caption,
            headers,
            rows,
            column_alignments: col_alignments,
            has_headers,
            num_rows,
            num_cols,
        })
    }

    /// Extracts caption text from `<caption>` child tag if present.
    fn extract_caption(table_tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Option<String> {
        for child_handle in table_tag.children().top().iter() {
            if let Some(tl::Node::Tag(tag)) = child_handle.get(parser) {
                if tag.name().as_utf8_str().eq_ignore_ascii_case("caption") {
                    let mut text = String::new();
                    Self::collect_text(tag, parser, &mut text);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extracts column alignments from `<colgroup><col>` elements.
    fn extract_colgroup_alignments(
        table_tag: &tl::HTMLTag<'_>,
        parser: &tl::Parser<'_>,
    ) -> Vec<TableAlignment> {
        let mut alignments = Vec::new();

        for child_handle in table_tag.children().top().iter() {
            if let Some(tl::Node::Tag(tag)) = child_handle.get(parser) {
                if tag.name().as_utf8_str().eq_ignore_ascii_case("colgroup") {
                    for col_handle in tag.children().top().iter() {
                        if let Some(tl::Node::Tag(col_tag)) = col_handle.get(parser) {
                            if col_tag.name().as_utf8_str().eq_ignore_ascii_case("col") {
                                alignments.push(Self::extract_cell_alignment(col_tag));
                            }
                        }
                    }
                }
            }
        }

        alignments
    }

    /// Recursively collects all `<tr>` tag handles within a container (like `<thead>` or `<tbody>`).
    fn collect_tr_handles(
        handle: tl::NodeHandle,
        parser: &tl::Parser<'_>,
        out: &mut Vec<tl::NodeHandle>,
    ) {
        if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
            for child_handle in tag.children().top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    if child_tag.name().as_utf8_str().eq_ignore_ascii_case("tr") {
                        out.push(*child_handle);
                    }
                }
            }
        }
    }

    /// Builds a 2D normalized grid handling `colspan` and `rowspan`.
    fn build_normalized_grid(
        rows: &[(tl::NodeHandle, bool)],
        parser: &tl::Parser<'_>,
    ) -> (Vec<Vec<String>>, usize, Vec<TableAlignment>) {
        let mut grid: Vec<Vec<String>> = Vec::new();
        let mut occupied: Vec<Vec<bool>> = Vec::new();
        let mut header_row_count = 0;
        let mut col_alignments: Vec<TableAlignment> = Vec::new();

        for (row_idx, (tr_handle, is_in_thead)) in rows.iter().enumerate() {
            let tr_tag = match tr_handle.get(parser) {
                Some(tl::Node::Tag(t)) => t,
                _ => continue,
            };

            // Collect cells: <th> and <td>
            let mut cells = Vec::new();
            let mut all_th = true;

            for cell_handle in tr_tag.children().top().iter() {
                if let Some(tl::Node::Tag(tag)) = cell_handle.get(parser) {
                    let name = tag.name().as_utf8_str();
                    let lower = name.to_ascii_lowercase();
                    if lower == "th" || lower == "td" {
                        if lower != "th" {
                            all_th = false;
                        }
                        cells.push(tag);
                    }
                }
            }

            if cells.is_empty() {
                continue;
            }

            if *is_in_thead || (row_idx == 0 && all_th) {
                header_row_count += 1;
            }

            // Ensure grid has a slot for this row
            if grid.len() <= row_idx {
                grid.resize(row_idx + 1, Vec::new());
                occupied.resize(row_idx + 1, Vec::new());
            }

            let mut col_idx = 0;

            for cell_tag in cells {
                // Find next unoccupied column slot in this row
                while col_idx < occupied[row_idx].len() && occupied[row_idx][col_idx] {
                    col_idx += 1;
                }

                let colspan = Self::extract_span(cell_tag, "colspan");
                let rowspan = Self::extract_span(cell_tag, "rowspan");
                let alignment = Self::extract_cell_alignment(cell_tag);

                let mut cell_text = String::new();
                Self::collect_text(cell_tag, parser, &mut cell_text);
                let cleaned_text = cell_text.split_whitespace().collect::<Vec<_>>().join(" ");

                // Track alignment
                while col_alignments.len() <= col_idx {
                    col_alignments.push(TableAlignment::None);
                }
                if alignment != TableAlignment::None
                    && col_alignments[col_idx] == TableAlignment::None
                {
                    col_alignments[col_idx] = alignment;
                }

                // Fill current and spanned slots
                for r in 0..rowspan {
                    let target_r = row_idx + r;
                    while grid.len() <= target_r {
                        grid.push(Vec::new());
                        occupied.push(Vec::new());
                    }

                    for c in 0..colspan {
                        let target_c = col_idx + c;
                        while grid[target_r].len() <= target_c {
                            grid[target_r].push(String::new());
                            occupied[target_r].push(false);
                        }

                        if r == 0 && c == 0 {
                            grid[target_r][target_c] = cleaned_text.clone();
                        } else {
                            // Subordinate spanned cell: empty string to maintain column offsets
                            grid[target_r][target_c] = String::new();
                        }
                        occupied[target_r][target_c] = true;
                    }
                }

                col_idx += colspan;
            }
        }

        // Calculate max column width and pad all rows to equal length
        let max_cols = grid.iter().map(Vec::len).max().unwrap_or(0);
        for row in &mut grid {
            row.resize(max_cols, String::new());
        }

        (grid, header_row_count, col_alignments)
    }

    /// Extracts numeric span attribute value (`colspan` or `rowspan`), clamping between 1 and 50.
    fn extract_span(tag: &tl::HTMLTag<'_>, attr_name: &str) -> usize {
        tag.attributes()
            .get(attr_name)
            .flatten()
            .and_then(|bytes| bytes.as_utf8_str().parse::<usize>().ok())
            .unwrap_or(1)
            .clamp(1, 50)
    }

    /// Extracts cell alignment from `align` attribute or inline style.
    fn extract_cell_alignment(tag: &tl::HTMLTag<'_>) -> TableAlignment {
        let attrs = tag.attributes();

        if let Some(Some(align_val)) = attrs.get("align").map(|v| v.map(|b| b.as_utf8_str())) {
            let align = TableAlignment::parse(align_val.as_ref());
            if align != TableAlignment::None {
                return align;
            }
        }

        if let Some(Some(style_val)) = attrs.get("style").map(|v| v.map(|b| b.as_utf8_str())) {
            let lower_style = style_val.to_ascii_lowercase();
            if let Some(pos) = lower_style.find("text-align:") {
                let rest = &lower_style[pos + 11..];
                let val = rest.split(';').next().unwrap_or("").trim();
                let align = TableAlignment::parse(val);
                if align != TableAlignment::None {
                    return align;
                }
            }
        }

        TableAlignment::None
    }

    /// Recursively collects text from a node, decoding entities and replacing `<br>` with space.
    fn collect_text(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>, buf: &mut String) {
        for child_handle in tag.children().top().iter() {
            if let Some(node) = child_handle.get(parser) {
                match node {
                    tl::Node::Raw(bytes) => {
                        let decoded = decode_html_entities(bytes.as_utf8_str().as_ref());
                        buf.push_str(&decoded);
                    }
                    tl::Node::Tag(child_tag) => {
                        let lower = child_tag.name().as_utf8_str().to_ascii_lowercase();
                        if lower == "br" {
                            buf.push(' ');
                        } else {
                            Self::collect_text(child_tag, parser, buf);
                        }
                    }
                    tl::Node::Comment(_) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_standard_table() {
        let html = r#"
            <table>
                <caption>Server Metrics</caption>
                <thead>
                    <tr>
                        <th align="left">Host</th>
                        <th align="center">CPU (%)</th>
                        <th align="right">Memory (GB)</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>node-1</td>
                        <td>45</td>
                        <td>8.2</td>
                    </tr>
                    <tr>
                        <td>node-2</td>
                        <td>78</td>
                        <td>14.1</td>
                    </tr>
                </tbody>
            </table>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = TableExtractor;
        let tables = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert_eq!(t.caption.as_deref(), Some("Server Metrics"));
        assert_eq!(t.headers, vec!["Host", "CPU (%)", "Memory (GB)"]);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0], vec!["node-1", "45", "8.2"]);
        assert_eq!(t.rows[1], vec!["node-2", "78", "14.1"]);
        assert_eq!(
            t.column_alignments,
            vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right
            ]
        );

        let md = t.to_markdown();
        assert!(md.contains("**Server Metrics**"));
        assert!(md.contains("| Host | CPU (%) | Memory (GB) |"));
        assert!(md.contains("| :--- | :---: | ---: |"));
        assert!(md.contains("| node-1 | 45 | 8.2 |"));
    }

    #[test]
    fn test_extract_table_with_colspan() {
        let html = r#"
            <table>
                <tr>
                    <th colspan="2">Name</th>
                    <th>Age</th>
                </tr>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                </tr>
            </table>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = TableExtractor;
        let tables = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert_eq!(t.num_cols, 3);
        assert_eq!(t.headers, vec!["Name", "", "Age"]);
        assert_eq!(t.rows[0], vec!["John", "Doe", "30"]);
    }

    #[test]
    fn test_extract_table_without_headers() {
        let html = r#"
            <table>
                <tr>
                    <td>Alpha</td>
                    <td>Beta</td>
                </tr>
                <tr>
                    <td>Gamma</td>
                    <td>Delta</td>
                </tr>
            </table>
        "#;

        let vdom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let extractor = TableExtractor;
        let tables = extractor.extract_from_vdom(&vdom, &ElementsOptions::default());

        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert!(!t.has_headers);
        assert_eq!(t.rows.len(), 2);
        let md = t.to_markdown();
        assert!(md.contains("| Column 1 | Column 2 |"));
        assert!(md.contains("| Alpha | Beta |"));
    }
}
