//! # PDF Table Extraction Engine
//!
//! Spatial grid detection, column alignment, row clustering,
//! cell extraction, and serialization to Markdown GFM, HTML, and CSV.

use crate::models::{BoundingBox, PdfLine};
use serde::{Deserialize, Serialize};

/// An individual cell within an extracted table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTableCell {
    /// Normalized text content of the cell.
    pub text: String,
    /// Spatial bounding box of the cell.
    pub bbox: BoundingBox,
    /// 0-indexed row index within the table.
    pub row_index: usize,
    /// 0-indexed column index within the table.
    pub col_index: usize,
    /// Number of rows spanned by this cell (default 1).
    pub row_span: usize,
    /// Number of columns spanned by this cell (default 1).
    pub col_span: usize,
    /// Whether this cell belongs to a header row.
    pub is_header: bool,
}

impl PdfTableCell {
    /// Constructs a new table cell.
    #[must_use]
    pub fn new(
        text: impl Into<String>,
        bbox: BoundingBox,
        row_index: usize,
        col_index: usize,
        is_header: bool,
    ) -> Self {
        Self {
            text: text.into(),
            bbox,
            row_index,
            col_index,
            row_span: 1,
            col_span: 1,
            is_header,
        }
    }

    /// Constructs an empty cell for sparse grids.
    #[must_use]
    pub fn empty(bbox: BoundingBox, row_index: usize, col_index: usize) -> Self {
        Self {
            text: String::new(),
            bbox,
            row_index,
            col_index,
            row_span: 1,
            col_span: 1,
            is_header: false,
        }
    }
}

/// A horizontal row of cells within an extracted table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTableRow {
    /// Cells in this row ordered from left to right.
    pub cells: Vec<PdfTableCell>,
    /// Bounding box enclosing all cells in this row.
    pub bbox: BoundingBox,
    /// Whether this row is classified as a header row.
    pub is_header: bool,
}

impl PdfTableRow {
    /// Constructs a new table row.
    #[must_use]
    pub fn new(cells: Vec<PdfTableCell>, bbox: BoundingBox, is_header: bool) -> Self {
        Self {
            cells,
            bbox,
            is_header,
        }
    }
}

/// A structured table extracted from a PDF page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTable {
    /// Unique identifier for this table (e.g. `p1-table-1`).
    pub table_id: String,
    /// 1-based page number where this table is located.
    pub page_number: u32,
    /// Bounding box enclosing the entire table.
    pub bbox: BoundingBox,
    /// Ordered rows of the table from top to bottom.
    pub rows: Vec<PdfTableRow>,
    /// Total number of columns.
    pub column_count: usize,
    /// Total number of rows.
    pub row_count: usize,
    /// Optional caption or title detected near the table.
    pub caption: Option<String>,
}

impl PdfTable {
    /// Serializes the table to GitHub Flavored Markdown (GFM).
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if self.rows.is_empty() || self.column_count == 0 {
            return String::new();
        }

        let mut md = String::new();
        if let Some(caption) = &self.caption {
            md.push_str(&format!("**{}**\n\n", caption.trim()));
        }

        // Header row
        if let Some(first_row) = self.rows.first() {
            md.push('|');
            for col_idx in 0..self.column_count {
                let cell_text = first_row
                    .cells
                    .iter()
                    .find(|c| c.col_index == col_idx)
                    .map(|c| sanitize_markdown_cell(&c.text))
                    .unwrap_or_default();
                md.push(' ');
                md.push_str(&cell_text);
                md.push_str(" |");
            }
            md.push('\n');

            // Separator row
            md.push('|');
            for _ in 0..self.column_count {
                md.push_str("---|");
            }
            md.push('\n');
        }

        // Data rows (skip first row if it was designated header)
        let data_rows = if self.rows.len() > 1 && self.rows[0].is_header {
            &self.rows[1..]
        } else {
            &self.rows[..]
        };

        for row in data_rows {
            md.push('|');
            for col_idx in 0..self.column_count {
                let cell_text = row
                    .cells
                    .iter()
                    .find(|c| c.col_index == col_idx)
                    .map(|c| sanitize_markdown_cell(&c.text))
                    .unwrap_or_default();
                md.push(' ');
                md.push_str(&cell_text);
                md.push_str(" |");
            }
            md.push('\n');
        }

        md.trim_end().to_string()
    }

    /// Serializes the table to clean semantic HTML5 (`<table>`).
    #[must_use]
    pub fn to_html(&self) -> String {
        if self.rows.is_empty() || self.column_count == 0 {
            return String::new();
        }

        let mut html = String::from("<table>\n");
        if let Some(caption) = &self.caption {
            html.push_str(&format!("  <caption>{}</caption>\n", escape_html(caption)));
        }

        let (header_row, data_rows) = if self.rows[0].is_header {
            (Some(&self.rows[0]), &self.rows[1..])
        } else {
            (None, &self.rows[..])
        };

        if let Some(header) = header_row {
            html.push_str("  <thead>\n    <tr>\n");
            for col_idx in 0..self.column_count {
                let text = header
                    .cells
                    .iter()
                    .find(|c| c.col_index == col_idx)
                    .map(|c| escape_html(&c.text))
                    .unwrap_or_default();
                html.push_str(&format!("      <th>{}</th>\n", text));
            }
            html.push_str("    </tr>\n  </thead>\n");
        }

        html.push_str("  <tbody>\n");
        for row in data_rows {
            html.push_str("    <tr>\n");
            for col_idx in 0..self.column_count {
                let text = row
                    .cells
                    .iter()
                    .find(|c| c.col_index == col_idx)
                    .map(|c| escape_html(&c.text))
                    .unwrap_or_default();
                html.push_str(&format!("      <td>{}</td>\n", text));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n</table>");

        html
    }

    /// Serializes the table to RFC 4180 compliant CSV.
    #[must_use]
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        for row in &self.rows {
            for col_idx in 0..self.column_count {
                if col_idx > 0 {
                    csv.push(',');
                }
                let cell_text = row
                    .cells
                    .iter()
                    .find(|c| c.col_index == col_idx)
                    .map(|c| c.text.as_str())
                    .unwrap_or("");
                csv.push_str(&escape_csv(cell_text));
            }
            csv.push('\n');
        }
        csv
    }

    /// Returns cell reference at specific row and column index.
    #[must_use]
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&PdfTableCell> {
        self.rows
            .get(row)
            .and_then(|r| r.cells.iter().find(|c| c.col_index == col))
    }

    /// Returns list of header names.
    #[must_use]
    pub fn get_headers(&self) -> Vec<String> {
        if let Some(first_row) = self.rows.first() {
            if first_row.is_header {
                return (0..self.column_count)
                    .map(|col_idx| {
                        first_row
                            .cells
                            .iter()
                            .find(|c| c.col_index == col_idx)
                            .map(|c| c.text.clone())
                            .unwrap_or_default()
                    })
                    .collect();
            }
        }
        Vec::new()
    }
}

/// Configuration options for table detection.
#[derive(Debug, Clone)]
pub struct TableDetectorOptions {
    /// Minimum number of columns required to classify as a table.
    pub min_columns: usize,
    /// Minimum number of rows required to classify as a table.
    pub min_rows: usize,
    /// Vertical tolerance for grouping segments into the same baseline row.
    pub row_vertical_tolerance: f64,
    /// Horizontal alignment tolerance for clustering column boundaries across rows.
    pub column_alignment_tolerance: f64,
}

impl Default for TableDetectorOptions {
    fn default() -> Self {
        Self {
            min_columns: 2,
            min_rows: 2,
            row_vertical_tolerance: 3.5,
            column_alignment_tolerance: 20.0,
        }
    }
}

/// Automated detector for tabular structures from spatial line segments.
#[derive(Debug, Clone, Default)]
pub struct PdfTableDetector {
    options: TableDetectorOptions,
}

impl PdfTableDetector {
    /// Creates a new table detector with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new table detector with custom options.
    #[must_use]
    pub const fn with_options(options: TableDetectorOptions) -> Self {
        Self { options }
    }

    /// Detects tabular structures from page lines, separating tables from non-table lines.
    #[must_use]
    pub fn detect_tables(
        &self,
        lines: &[PdfLine],
        page_number: u32,
    ) -> (Vec<PdfTable>, Vec<PdfLine>) {
        if lines.len() < self.options.min_rows * self.options.min_columns {
            return (Vec::new(), lines.to_vec());
        }

        // 1. Group lines by baseline Y
        let mut baseline_groups: Vec<Vec<PdfLine>> = Vec::new();
        for line in lines {
            let mut matched = false;
            for group in &mut baseline_groups {
                if let Some(first) = group.first() {
                    if (first.bbox.y0 - line.bbox.y0).abs() <= self.options.row_vertical_tolerance {
                        group.push(line.clone());
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                baseline_groups.push(vec![line.clone()]);
            }
        }

        // Sort baseline groups descending by Y (top to bottom)
        baseline_groups.sort_by(|a, b| {
            let y_a = a.first().map_or(0.0, |l| l.bbox.y0);
            let y_b = b.first().map_or(0.0, |l| l.bbox.y0);
            y_b.partial_cmp(&y_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sort segments in each baseline group ascending by X (left to right)
        for group in &mut baseline_groups {
            group.sort_by(|a, b| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // 2. Identify candidate rows with >= min_columns segments
        let mut candidate_rows: Vec<(usize, &Vec<PdfLine>)> = Vec::new();
        for (idx, group) in baseline_groups.iter().enumerate() {
            if group.len() >= self.options.min_columns {
                candidate_rows.push((idx, group));
            }
        }

        if candidate_rows.len() < self.options.min_rows {
            return (Vec::new(), lines.to_vec());
        }

        // 3. Cluster consecutive candidate rows that share aligned column structures
        let mut table_clusters: Vec<Vec<&Vec<PdfLine>>> = Vec::new();
        let mut current_cluster: Vec<&Vec<PdfLine>> = Vec::new();
        let mut current_col_starts: Vec<f64> = Vec::new();

        for (_idx, group) in candidate_rows {
            let row_col_starts: Vec<f64> = group.iter().map(|l| l.bbox.x0).collect();

            let aligns = if current_cluster.is_empty() {
                true
            } else {
                columns_align(
                    &current_col_starts,
                    &row_col_starts,
                    self.options.column_alignment_tolerance,
                )
            };

            if aligns {
                if current_cluster.is_empty() {
                    current_col_starts = row_col_starts;
                } else {
                    // Update running column bounds with average/median
                    current_col_starts = merge_column_starts(&current_col_starts, &row_col_starts);
                }
                current_cluster.push(group);
            } else {
                if current_cluster.len() >= self.options.min_rows {
                    table_clusters.push(std::mem::take(&mut current_cluster));
                } else {
                    current_cluster.clear();
                }
                current_col_starts = row_col_starts;
                current_cluster.push(group);
            }
        }

        if current_cluster.len() >= self.options.min_rows {
            table_clusters.push(current_cluster);
        }

        if table_clusters.is_empty() {
            return (Vec::new(), lines.to_vec());
        }

        // 4. Construct PdfTable objects and mark consumed lines
        let mut tables = Vec::new();
        let mut consumed_line_bboxes: Vec<BoundingBox> = Vec::new();

        for (table_idx, cluster) in table_clusters.into_iter().enumerate() {
            // Establish unified column anchors for this cluster
            let column_anchors =
                compute_unified_column_anchors(&cluster, self.options.column_alignment_tolerance);
            let column_count = column_anchors.len();

            if column_count < self.options.min_columns {
                continue;
            }

            let mut rows = Vec::new();
            let mut table_bbox = BoundingBox::zero();

            for (row_idx, group) in cluster.iter().enumerate() {
                let mut row_bbox = BoundingBox::zero();
                let mut cells = Vec::with_capacity(column_count);

                // Assign each segment in group to closest column anchor
                for (col_idx, &anchor_x) in column_anchors.iter().enumerate() {
                    let matched_segment = group.iter().find(|l| {
                        (l.bbox.x0 - anchor_x).abs() <= self.options.column_alignment_tolerance
                    });

                    if let Some(seg) = matched_segment {
                        if row_bbox == BoundingBox::zero() {
                            row_bbox = seg.bbox;
                        } else {
                            row_bbox = row_bbox.union(&seg.bbox);
                        }

                        cells.push(PdfTableCell::new(
                            seg.text.clone(),
                            seg.bbox,
                            row_idx,
                            col_idx,
                            row_idx == 0,
                        ));
                    } else {
                        // Empty cell at anchor position
                        let empty_bbox = BoundingBox::new(anchor_x, 0.0, anchor_x + 10.0, 0.0);
                        cells.push(PdfTableCell::empty(empty_bbox, row_idx, col_idx));
                    }
                }

                if table_bbox == BoundingBox::zero() {
                    table_bbox = row_bbox;
                } else {
                    table_bbox = table_bbox.union(&row_bbox);
                }

                for seg in *group {
                    consumed_line_bboxes.push(seg.bbox);
                }

                rows.push(PdfTableRow::new(cells, row_bbox, row_idx == 0));
            }

            let row_count = rows.len();
            let table_id = format!("page-{page_number}-table-{}", table_idx + 1);

            tables.push(PdfTable {
                table_id,
                page_number,
                bbox: table_bbox,
                rows,
                column_count,
                row_count,
                caption: None,
            });
        }

        // 5. Detect possible captions immediately preceding detected tables
        for table in &mut tables {
            for line in lines {
                if !consumed_line_bboxes.contains(&line.bbox) {
                    let is_above =
                        line.bbox.y0 >= table.bbox.y1 && (line.bbox.y0 - table.bbox.y1) <= 35.0;
                    if is_above && is_table_caption(&line.text) {
                        table.caption = Some(line.text.clone());
                        consumed_line_bboxes.push(line.bbox);
                        break;
                    }
                }
            }
        }

        // 6. Filter out consumed lines
        let remaining_lines: Vec<PdfLine> = lines
            .iter()
            .filter(|l| !consumed_line_bboxes.contains(&l.bbox))
            .cloned()
            .collect();

        (tables, remaining_lines)
    }
}

/// Checks whether two sequences of column start coordinates align within tolerance.
fn columns_align(cols_a: &[f64], cols_b: &[f64], tolerance: f64) -> bool {
    if cols_a.is_empty() || cols_b.is_empty() {
        return false;
    }

    // Number of columns should match closely
    let count_diff = (cols_a.len() as isize - cols_b.len() as isize).unsigned_abs();
    if count_diff > 1 {
        return false;
    }

    // Check count of aligning columns
    let mut matched_count = 0;
    for &xa in cols_a {
        if cols_b.iter().any(|&xb| (xa - xb).abs() <= tolerance) {
            matched_count += 1;
        }
    }

    let min_match = cols_a.len().min(cols_b.len());
    matched_count >= min_match.saturating_sub(1).max(2)
}

/// Merges column start coordinates from previous rows with a new row.
fn merge_column_starts(existing: &[f64], new_starts: &[f64]) -> Vec<f64> {
    let mut merged = existing.to_vec();
    for &ns in new_starts {
        let mut matched = false;
        for ex in &mut merged {
            if (*ex - ns).abs() <= 20.0 {
                *ex = (*ex + ns) / 2.0; // Running average
                matched = true;
                break;
            }
        }
        if !matched {
            merged.push(ns);
        }
    }
    merged.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    merged
}

/// Computes unified column start anchors across all rows in a table cluster.
fn compute_unified_column_anchors(cluster: &[&Vec<PdfLine>], tolerance: f64) -> Vec<f64> {
    let mut all_starts = Vec::new();
    for group in cluster {
        for l in *group {
            all_starts.push(l.bbox.x0);
        }
    }
    all_starts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Cluster coordinates within tolerance
    let mut clusters: Vec<(f64, usize)> = Vec::new();
    for &x in &all_starts {
        let mut matched = false;
        for (center, count) in &mut clusters {
            if (*center - x).abs() <= tolerance {
                *center = (*center * (*count as f64) + x) / ((*count + 1) as f64);
                *count += 1;
                matched = true;
                break;
            }
        }
        if !matched {
            clusters.push((x, 1));
        }
    }

    clusters.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    clusters.into_iter().map(|(c, _)| c).collect()
}

/// Checks if a line resembles a table caption or title.
fn is_table_caption(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    lower.starts_with("table ")
        || lower.starts_with("table:")
        || lower.starts_with("tab. ")
        || lower.starts_with("table -")
}

/// Sanitizes cell text for Markdown table emission (escapes `|` and replaces newlines).
fn sanitize_markdown_cell(text: &str) -> String {
    text.replace('\n', " ")
        .replace('\r', "")
        .replace('|', "\\|")
        .trim()
        .to_string()
}

/// Escapes HTML text.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escapes CSV values per RFC 4180.
fn escape_csv(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        val.to_string()
    }
}
