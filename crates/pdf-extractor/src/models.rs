//! # Document and Layout Models for PDF Extraction
//!
//! Spatial geometry, text blocks, page structures, document metadata, and outlines.

use crate::table::PdfTable;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 2D rectangular bounding box defined by lower-left `(x0, y0)` and upper-right `(x1, y1)`
/// in PDF user space coordinates (points, 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum horizontal coordinate (left).
    pub x0: f64,
    /// Minimum vertical coordinate (bottom in PDF default space).
    pub y0: f64,
    /// Maximum horizontal coordinate (right).
    pub x1: f64,
    /// Maximum vertical coordinate (top in PDF default space).
    pub y1: f64,
}

impl BoundingBox {
    /// Creates a new bounding box from coordinates, normalizing so `x0 <= x1` and `y0 <= y1`.
    #[must_use]
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    /// Zero-sized bounding box at origin.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }

    /// Horizontal width in points.
    #[must_use]
    pub fn width(&self) -> f64 {
        (self.x1 - self.x0).abs()
    }

    /// Vertical height in points.
    #[must_use]
    pub fn height(&self) -> f64 {
        (self.y1 - self.y0).abs()
    }

    /// Area in square points.
    #[must_use]
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Checks if this bounding box intersects with another.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
    }

    /// Checks if this bounding box completely encloses another.
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        self.x0 <= other.x0 && self.y0 <= other.y0 && self.x1 >= other.x1 && self.y1 >= other.y1
    }

    /// Returns the smallest bounding box containing both `self` and `other`.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }
}

/// Physical and visual dimensions of a PDF page.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageDimensions {
    /// Page width in points (standard US Letter is 612.0, A4 is 595.28).
    pub width: f64,
    /// Page height in points (standard US Letter is 792.0, A4 is 841.89).
    pub height: f64,
    /// Page display rotation in degrees (0, 90, 180, 270).
    pub rotation: u32,
}

impl Default for PageDimensions {
    fn default() -> Self {
        Self {
            width: 612.0,  // Default US Letter width
            height: 792.0, // Default US Letter height
            rotation: 0,
        }
    }
}

/// Extracted PDF Document Information Dictionary metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfMetadata {
    /// Title of the document.
    pub title: Option<String>,
    /// Author or primary contributor.
    pub author: Option<String>,
    /// Subject or description.
    pub subject: Option<String>,
    /// List of keyword tags.
    pub keywords: Vec<String>,
    /// Software program that created the original content.
    pub creator: Option<String>,
    /// Software program that produced the PDF file.
    pub producer: Option<String>,
    /// Date and time the document was created.
    pub creation_date: Option<DateTime<Utc>>,
    /// Date and time the document was last modified.
    pub mod_date: Option<DateTime<Utc>>,
    /// PDF specification version (e.g. "1.4", "1.7", "2.0").
    pub pdf_version: String,
    /// Total number of pages declared in the document catalog.
    pub page_count: u32,
}

/// An individual text span with uniform font and styling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfSpan {
    /// Extracted text content.
    pub text: String,
    /// PostScript or base font name, if identified.
    pub font_name: Option<String>,
    /// Point size of the font.
    pub font_size: f64,
    /// Bounding box enclosing this span.
    pub bbox: BoundingBox,
}

/// A horizontally aligned sequence of text spans forming a single visual line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfLine {
    /// Complete text of the line.
    pub text: String,
    /// Component text spans.
    pub spans: Vec<PdfSpan>,
    /// Bounding box enclosing the line.
    pub bbox: BoundingBox,
    /// Dominant font size of the line.
    pub font_size: f64,
}

/// Semantic classification of a text block based on spatial and typographic analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextBlockKind {
    /// Document or section heading (level 1-6).
    Header,
    /// Body paragraph.
    Paragraph,
    /// Bulleted or numbered list item.
    ListItem,
    /// Running header at the top of the page.
    PageHeader,
    /// Running footer or page number at the bottom of the page.
    PageFooter,
    /// Tabular data or cell content.
    TableContent,
    /// Fixed-width or code snippet.
    CodeBlock,
}

/// A coherent block of text (paragraph, header, list item) clustered spatially.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTextBlock {
    /// Combined plain text of all lines in the block.
    pub text: String,
    /// Constituent lines in reading order.
    pub lines: Vec<PdfLine>,
    /// Bounding box enclosing the entire block.
    pub bbox: BoundingBox,
    /// Estimated dominant font size.
    pub font_size: f64,
    /// Inferred semantic kind.
    pub kind: TextBlockKind,
    /// Reading order sequence index on the page (0-based).
    pub reading_order: u32,
}

/// A parsed and laid-out page from the PDF document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPage {
    /// 1-based page index.
    pub page_number: u32,
    /// Page dimensions and rotation.
    pub dimensions: PageDimensions,
    /// Spatially clustered and ordered text blocks (excluding separated headers/footers).
    pub blocks: Vec<PdfTextBlock>,
    /// Extracted structured tables on this page.
    pub tables: Vec<PdfTable>,
    /// Running page headers detected on the page.
    pub headers: Vec<PdfTextBlock>,
    /// Running page footers detected on the page.
    pub footers: Vec<PdfTextBlock>,
    /// Aggregated plain text content of the page in reading order.
    pub raw_text: String,
    /// Total character count on the page.
    pub char_count: usize,
    /// Estimated token count (using standard 4 chars/token heuristic).
    pub estimated_tokens: usize,
}

/// Hierarchical bookmark or outline item from the PDF document catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineItem {
    /// Display title of the outline bookmark.
    pub title: String,
    /// Target 1-based page number, if resolved.
    pub page_number: Option<u32>,
    /// Nested child outline items.
    pub children: Vec<OutlineItem>,
}

/// A fully parsed and structured PDF document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDocument {
    /// Document metadata and info dictionary properties.
    pub metadata: PdfMetadata,
    /// All parsed pages with layout information.
    pub pages: Vec<PdfPage>,
    /// Document outline hierarchy (table of contents / bookmarks).
    pub outline: Vec<OutlineItem>,
    /// Total number of pages.
    pub total_pages: u32,
    /// Total character count across all pages.
    pub total_characters: usize,
    /// Total estimated tokens across all pages.
    pub total_estimated_tokens: usize,
}
