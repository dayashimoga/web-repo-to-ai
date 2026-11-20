//! # PDF Layout Analysis Engine
//!
//! Spatial text reconstruction, reading order determination, line clustering,
//! multi-column detection, and semantic block classification.

use crate::models::{BoundingBox, PageDimensions, PdfLine, PdfSpan, PdfTextBlock, TextBlockKind};
use crate::table::{PdfTable, PdfTableDetector};

/// Result of layout and table analysis for a page.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayoutResult {
    /// Content text blocks (headings, paragraphs, list items, and table representations) in reading order.
    pub content_blocks: Vec<PdfTextBlock>,
    /// Extracted structured tables.
    pub tables: Vec<PdfTable>,
    /// Running page headers detected near top margin.
    pub page_headers: Vec<PdfTextBlock>,
    /// Running page footers detected near bottom margin.
    pub page_footers: Vec<PdfTextBlock>,
}

/// Configuration parameters for the spatial layout engine.
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// Minimum font size threshold (in points) to consider a block a heading.
    pub header_font_size_threshold: f64,
    /// Vertical tolerance multiplier for grouping text spans into the same line.
    pub line_vertical_tolerance: f64,
    /// Line spacing threshold factor for splitting lines into distinct paragraphs.
    pub paragraph_split_threshold: f64,
    /// Maximum horizontal gap factor (multiplier of font size) between spans in the same line.
    pub max_span_horizontal_gap_factor: f64,
    /// Margin percentage (from top/bottom) considered running header/footer.
    pub header_footer_margin_ratio: f64,
    /// Whether to detect and order multi-column text layouts.
    pub detect_columns: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            header_font_size_threshold: 13.0,
            line_vertical_tolerance: 3.0,
            paragraph_split_threshold: 1.5,
            max_span_horizontal_gap_factor: 2.5,
            header_footer_margin_ratio: 0.06, // Top/bottom 6% of page
            detect_columns: true,
        }
    }
}

/// The layout engine responsible for turning disjoint text fragments into structured blocks.
#[derive(Debug, Clone)]
pub struct PdfLayoutEngine {
    options: LayoutOptions,
}

impl Default for PdfLayoutEngine {
    fn default() -> Self {
        Self::new(LayoutOptions::default())
    }
}

impl PdfLayoutEngine {
    /// Creates a new layout engine with custom options.
    #[must_use]
    pub const fn new(options: LayoutOptions) -> Self {
        Self { options }
    }

    /// Processes a collection of text spans for a given page, clustering them into
    /// lines, ordering them by reading sequence, and grouping them into semantic text blocks.
    #[must_use]
    pub fn build_page_blocks(
        &self,
        mut spans: Vec<PdfSpan>,
        dimensions: &PageDimensions,
    ) -> Vec<PdfTextBlock> {
        if spans.is_empty() {
            return Vec::new();
        }

        // 1. Cluster spans into lines based on vertical alignment
        let lines = self.cluster_spans_into_lines(&mut spans);

        // 2. Check for multi-column layout and sort lines into reading order
        let ordered_lines = if self.options.detect_columns {
            self.order_lines_by_columns(lines, dimensions)
        } else {
            self.order_lines_top_to_bottom(lines)
        };

        // 3. Cluster lines into semantic blocks (paragraphs, headers, lists, footers)
        self.cluster_lines_into_blocks(ordered_lines, dimensions)
    }

    /// Performs comprehensive layout analysis on page spans:
    /// clusters lines, extracts structured tables, orders columns, separates running headers/footers,
    /// and assembles semantic content blocks.
    #[must_use]
    pub fn build_page_layout(
        &self,
        mut spans: Vec<PdfSpan>,
        dimensions: &PageDimensions,
        page_number: u32,
    ) -> PageLayoutResult {
        if spans.is_empty() {
            return PageLayoutResult {
                content_blocks: Vec::new(),
                tables: Vec::new(),
                page_headers: Vec::new(),
                page_footers: Vec::new(),
            };
        }

        // 1. Cluster spans into lines
        let lines = self.cluster_spans_into_lines(&mut spans);

        // 2. Detect tables from clustered lines
        let table_detector = PdfTableDetector::new();
        let (tables, non_table_lines) = table_detector.detect_tables(&lines, page_number);

        // 3. Order remaining lines (multi-column or top-to-bottom)
        let ordered_lines = if self.options.detect_columns {
            self.order_lines_by_columns(non_table_lines, dimensions)
        } else {
            self.order_lines_top_to_bottom(non_table_lines)
        };

        // 4. Cluster lines into blocks
        let all_blocks = self.cluster_lines_into_blocks(ordered_lines, dimensions);

        // 5. Separate page headers, footers, and content
        let mut content_blocks = Vec::new();
        let mut page_headers = Vec::new();
        let mut page_footers = Vec::new();

        for block in all_blocks {
            match block.kind {
                TextBlockKind::PageHeader => page_headers.push(block),
                TextBlockKind::PageFooter => page_footers.push(block),
                _ => content_blocks.push(block),
            }
        }

        // 6. Integrate table blocks into content_blocks for linear plain text representation
        for table in &tables {
            let table_block = PdfTextBlock {
                text: table.to_markdown(),
                lines: Vec::new(),
                bbox: table.bbox,
                font_size: 10.0,
                kind: TextBlockKind::TableContent,
                reading_order: content_blocks.len() as u32,
            };
            content_blocks.push(table_block);
        }

        // Re-sort content blocks by vertical sequence (descending y1)
        content_blocks.sort_by(|a, b| {
            b.bbox
                .y1
                .partial_cmp(&a.bbox.y1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Re-index reading order
        for (idx, block) in content_blocks.iter_mut().enumerate() {
            block.reading_order = idx as u32;
        }

        PageLayoutResult {
            content_blocks,
            tables,
            page_headers,
            page_footers,
        }
    }

    /// Clusters loose text spans into horizontal lines.
    fn cluster_spans_into_lines(&self, spans: &mut [PdfSpan]) -> Vec<PdfLine> {
        if spans.is_empty() {
            return Vec::new();
        }

        // Sort initially by descending Y (top of page in PDF coordinates has highest Y),
        // then ascending X (left to right).
        spans.sort_by(|a, b| {
            b.bbox
                .y0
                .partial_cmp(&a.bbox.y0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.bbox
                        .x0
                        .partial_cmp(&b.bbox.x0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut lines: Vec<PdfLine> = Vec::new();

        for span in spans.iter() {
            // Find existing line that matches this span's baseline within tolerance
            let tol = (span.font_size * 0.3).max(self.options.line_vertical_tolerance);
            let mut matched = false;

            for line in &mut lines {
                let y_diff = (line.bbox.y0 - span.bbox.y0).abs();
                if y_diff <= tol {
                    // Check horizontal gap: if spans are horizontally separated by more than
                    // max_span_horizontal_gap_factor * font_size, treat them as separate lines/columns.
                    let max_gap = span.font_size * self.options.max_span_horizontal_gap_factor;
                    let is_adjacent = span.bbox.x0 <= line.bbox.x1 + max_gap
                        && line.bbox.x0 <= span.bbox.x1 + max_gap;

                    if is_adjacent {
                        // Match found: add span to this line
                        line.spans.push(span.clone());
                        line.bbox = line.bbox.union(&span.bbox);
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                // Start a new line
                lines.push(PdfLine {
                    text: String::new(),
                    bbox: span.bbox,
                    font_size: span.font_size,
                    spans: vec![span.clone()],
                });
            }
        }

        // Sort spans within each line left-to-right and assemble combined line text
        for line in &mut lines {
            line.spans.sort_by(|a, b| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut line_text = String::new();
            let mut dominant_font_size = 0.0;
            let mut max_span_len = 0;

            for (i, span) in line.spans.iter().enumerate() {
                if i > 0 {
                    // If there's a significant gap between spans, insert a space
                    let prev_span = &line.spans[i - 1];
                    let gap = span.bbox.x0 - prev_span.bbox.x1;
                    if gap > 1.5 && !line_text.ends_with(' ') && !span.text.starts_with(' ') {
                        line_text.push(' ');
                    }
                }
                line_text.push_str(&span.text);

                if span.text.len() >= max_span_len {
                    max_span_len = span.text.len();
                    dominant_font_size = span.font_size;
                }
            }

            line.text = line_text.trim().to_string();
            line.font_size = if dominant_font_size > 0.0 {
                dominant_font_size
            } else {
                10.0
            };
        }

        // Filter out empty lines
        lines.retain(|l| !l.text.is_empty());
        lines
    }

    /// Orders lines strictly from top to bottom (descending Y).
    fn order_lines_top_to_bottom(&self, mut lines: Vec<PdfLine>) -> Vec<PdfLine> {
        lines.sort_by(|a, b| {
            b.bbox
                .y1
                .partial_cmp(&a.bbox.y1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.bbox
                        .x0
                        .partial_cmp(&b.bbox.x0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        lines
    }

    /// Detects two-column layouts and orders text down the left column first, then the right column.
    fn order_lines_by_columns(
        &self,
        lines: Vec<PdfLine>,
        dimensions: &PageDimensions,
    ) -> Vec<PdfLine> {
        if lines.len() < 4 {
            return self.order_lines_top_to_bottom(lines);
        }

        let mid_x = dimensions.width / 2.0;
        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut full_width_lines = Vec::new();

        // Check if page exhibits strong two-column characteristics
        for line in lines {
            let width = line.bbox.width();
            let is_full_width = width > dimensions.width * 0.65;

            if is_full_width {
                full_width_lines.push(line);
            } else if line.bbox.x1 <= mid_x + 20.0 {
                left_lines.push(line);
            } else if line.bbox.x0 >= mid_x - 20.0 {
                right_lines.push(line);
            } else {
                full_width_lines.push(line);
            }
        }

        // Only use column ordering if there is a substantial number of lines in both columns
        let is_multi_column = left_lines.len() >= 3 && right_lines.len() >= 3;

        if !is_multi_column {
            // Recombine and use standard top-to-bottom
            let mut all = full_width_lines;
            all.extend(left_lines);
            all.extend(right_lines);
            return self.order_lines_top_to_bottom(all);
        }

        // Sort each column top-to-bottom
        left_lines = self.order_lines_top_to_bottom(left_lines);
        right_lines = self.order_lines_top_to_bottom(right_lines);
        full_width_lines = self.order_lines_top_to_bottom(full_width_lines);

        // Interleave top full-width banners (headers), left column, right column, then footer
        let header_y = dimensions.height * (1.0 - self.options.header_footer_margin_ratio);
        let footer_y = dimensions.height * self.options.header_footer_margin_ratio;

        let mut top_banners = Vec::new();
        let mut bottom_banners = Vec::new();
        let mut other_banners = Vec::new();

        for l in full_width_lines {
            if l.bbox.y0 >= header_y {
                top_banners.push(l);
            } else if l.bbox.y1 <= footer_y {
                bottom_banners.push(l);
            } else {
                other_banners.push(l);
            }
        }

        let mut ordered = top_banners;
        ordered.extend(other_banners);
        ordered.extend(left_lines);
        ordered.extend(right_lines);
        ordered.extend(bottom_banners);

        ordered
    }

    /// Clusters lines into coherent blocks (paragraphs, headers, list items).
    fn cluster_lines_into_blocks(
        &self,
        lines: Vec<PdfLine>,
        dimensions: &PageDimensions,
    ) -> Vec<PdfTextBlock> {
        if lines.is_empty() {
            return Vec::new();
        }

        let mut blocks: Vec<PdfTextBlock> = Vec::new();
        let mut current_lines: Vec<PdfLine> = Vec::new();
        let header_margin_y = dimensions.height * (1.0 - self.options.header_footer_margin_ratio);
        let footer_margin_y = dimensions.height * self.options.header_footer_margin_ratio;

        for line in lines {
            let should_split = if let Some(prev) = current_lines.last() {
                let vertical_gap = prev.bbox.y0 - line.bbox.y1;
                let expected_gap = prev.font_size * self.options.paragraph_split_threshold;
                let font_size_change =
                    (prev.font_size - line.font_size).abs() / prev.font_size.max(1.0);

                let is_prev_header = prev.font_size >= self.options.header_font_size_threshold;
                let is_curr_header = line.font_size >= self.options.header_font_size_threshold;
                let is_list_item = is_list_marker(&line.text);

                // Split conditions
                vertical_gap > expected_gap
                    || vertical_gap < -2.0 // Text jumped upwards (e.g. new column or section)
                    || font_size_change > 0.25
                    || (is_prev_header != is_curr_header)
                    || is_curr_header
                    || is_list_item
            } else {
                false
            };

            if should_split && !current_lines.is_empty() {
                let block = self.build_block_from_lines(
                    std::mem::take(&mut current_lines),
                    header_margin_y,
                    footer_margin_y,
                    blocks.len() as u32,
                );
                blocks.push(block);
            }

            current_lines.push(line);
        }

        if !current_lines.is_empty() {
            let block = self.build_block_from_lines(
                current_lines,
                header_margin_y,
                footer_margin_y,
                blocks.len() as u32,
            );
            blocks.push(block);
        }

        blocks
    }

    /// Converts a cluster of lines into a `PdfTextBlock` with classified semantic type.
    fn build_block_from_lines(
        &self,
        lines: Vec<PdfLine>,
        header_margin_y: f64,
        footer_margin_y: f64,
        reading_order: u32,
    ) -> PdfTextBlock {
        let mut combined_text = String::new();
        let mut bbox = BoundingBox::zero();
        let mut max_font_size = 0.0;

        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                bbox = line.bbox;
            } else {
                bbox = bbox.union(&line.bbox);
                combined_text.push(' ');
            }
            combined_text.push_str(&line.text);
            if line.font_size > max_font_size {
                max_font_size = line.font_size;
            }
        }

        let trimmed_text = combined_text.trim().to_string();

        // Classify block kind
        let kind = if bbox.y0 >= header_margin_y {
            TextBlockKind::PageHeader
        } else if bbox.y1 <= footer_margin_y {
            TextBlockKind::PageFooter
        } else if max_font_size >= self.options.header_font_size_threshold {
            TextBlockKind::Header
        } else if is_list_marker(&trimmed_text) {
            TextBlockKind::ListItem
        } else {
            TextBlockKind::Paragraph
        };

        PdfTextBlock {
            text: trimmed_text,
            lines,
            bbox,
            font_size: if max_font_size > 0.0 {
                max_font_size
            } else {
                10.0
            },
            kind,
            reading_order,
        }
    }
}

/// Helper to detect common list bullet markers or numbered list items.
fn is_list_marker(text: &str) -> bool {
    let t = text.trim_start();
    t.starts_with('•')
        || t.starts_with('-')
        || t.starts_with('*')
        || t.starts_with('–')
        || (t.len() >= 3
            && t.chars().next().is_some_and(|c| c.is_ascii_digit())
            && (t.chars().nth(1) == Some('.') || t.chars().nth(1) == Some(')')))
}
