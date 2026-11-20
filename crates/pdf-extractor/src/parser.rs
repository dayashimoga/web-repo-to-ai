//! # PDF Document Parser
//!
//! Low-level PDF structure traversal, header validation, object tree resolution,
//! content stream decoding, font state tracking, and layout integration.

use crate::error::{PdfError, Result};
use crate::layout::PdfLayoutEngine;
use crate::models::{
    BoundingBox, OutlineItem, PageDimensions, PdfDocument, PdfMetadata, PdfPage, PdfSpan,
};
use chrono::{DateTime, Utc};
use lopdf::{Document, Object};

/// High-performance pure Rust parser for PDF document structures.
#[derive(Debug, Default)]
pub struct PdfParser {
    layout_engine: PdfLayoutEngine,
}

impl PdfParser {
    /// Creates a new `PdfParser` with default layout engine settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            layout_engine: PdfLayoutEngine::default(),
        }
    }

    /// Creates a `PdfParser` with a custom `PdfLayoutEngine`.
    #[must_use]
    pub const fn with_layout_engine(layout_engine: PdfLayoutEngine) -> Self {
        Self { layout_engine }
    }

    /// Parses a complete PDF document from raw in-memory bytes.
    pub fn parse(&self, bytes: &[u8]) -> Result<PdfDocument> {
        if bytes.is_empty() {
            return Err(PdfError::EmptyDocument);
        }

        // Validate PDF magic header
        let version = self.extract_pdf_version(bytes)?;

        // Load document through lopdf
        let doc = Document::load_mem(bytes).map_err(|err| {
            let msg = err.to_string();
            if msg.to_lowercase().contains("encrypt") {
                PdfError::EncryptedDocument
            } else {
                PdfError::MalformedStructure(format!("Failed to parse PDF xref or trailer: {msg}"))
            }
        })?;

        // Guard against encrypted PDFs
        if doc.is_encrypted() {
            return Err(PdfError::EncryptedDocument);
        }

        // Extract metadata dictionary
        let mut metadata = self.extract_metadata(&doc);
        metadata.pdf_version = version;

        // Discover and order pages
        let page_map = doc.get_pages();
        let total_pages = page_map.len() as u32;
        metadata.page_count = total_pages;

        if total_pages == 0 {
            return Ok(PdfDocument {
                metadata,
                pages: Vec::new(),
                outline: Vec::new(),
                total_pages: 0,
                total_characters: 0,
                total_estimated_tokens: 0,
            });
        }

        // Parse each page
        let mut pages = Vec::with_capacity(page_map.len());
        let mut total_characters = 0;
        let mut total_estimated_tokens = 0;

        for (page_num, page_obj_id) in page_map {
            let page = self.parse_page(&doc, page_num, page_obj_id)?;
            total_characters += page.char_count;
            total_estimated_tokens += page.estimated_tokens;
            pages.push(page);
        }

        // Extract outline bookmarks
        let outline = self.extract_outlines(&doc);

        Ok(PdfDocument {
            metadata,
            pages,
            outline,
            total_pages,
            total_characters,
            total_estimated_tokens,
        })
    }

    /// Validates the `%PDF-` header and returns the version string.
    fn extract_pdf_version(&self, bytes: &[u8]) -> Result<String> {
        // PDF specification allows up to 1024 bytes of leading junk before %PDF-
        let limit = bytes.len().min(1024);
        let prefix = &bytes[..limit];

        if let Some(pos) = prefix.windows(5).position(|w| w == b"%PDF-") {
            let version_slice = &bytes[pos + 5..limit.min(pos + 12)];
            let version_str = String::from_utf8_lossy(version_slice);
            let version = version_str
                .split(|c: char| c.is_whitespace() || c == '%' || c == '\r' || c == '\n')
                .next()
                .unwrap_or("1.4")
                .trim();
            Ok(version.to_string())
        } else {
            let sample = String::from_utf8_lossy(&bytes[..limit.min(16)]);
            Err(PdfError::InvalidHeader(sample.to_string()))
        }
    }

    /// Extracts document information dictionary (`/Info`) metadata.
    fn extract_metadata(&self, doc: &Document) -> PdfMetadata {
        let mut meta = PdfMetadata::default();

        let info_dict = if let Ok(info_ref) = doc.trailer.get(b"Info") {
            match info_ref {
                Object::Reference(id) => doc.get_object(*id).and_then(Object::as_dict).ok(),
                Object::Dictionary(dict) => Some(dict),
                _ => None,
            }
        } else {
            None
        };

        if let Some(dict) = info_dict {
            if let Ok(obj) = dict.get(b"Title") {
                meta.title = get_string_or_name(obj);
            }
            if let Ok(obj) = dict.get(b"Author") {
                meta.author = get_string_or_name(obj);
            }
            if let Ok(obj) = dict.get(b"Subject") {
                meta.subject = get_string_or_name(obj);
            }
            if let Ok(obj) = dict.get(b"Creator") {
                meta.creator = get_string_or_name(obj);
            }
            if let Ok(obj) = dict.get(b"Producer") {
                meta.producer = get_string_or_name(obj);
            }
            if let Ok(obj) = dict.get(b"Keywords") {
                if let Some(kw) = get_string_or_name(obj) {
                    meta.keywords = kw
                        .split([',', ';'])
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
            if let Ok(obj) = dict.get(b"CreationDate") {
                if let Some(creation) = get_string_or_name(obj) {
                    meta.creation_date = parse_pdf_date(&creation);
                }
            }
            if let Ok(obj) = dict.get(b"ModDate") {
                if let Some(mod_date) = get_string_or_name(obj) {
                    meta.mod_date = parse_pdf_date(&mod_date);
                }
            }
        }

        meta
    }

    /// Parses a single page, resolving its dimensions, extracting text spans, and clustering layout.
    fn parse_page(
        &self,
        doc: &Document,
        page_num: u32,
        page_obj_id: (u32, u16),
    ) -> Result<PdfPage> {
        let dimensions = self.extract_page_dimensions(doc, page_obj_id);

        // Attempt fine-grained operator stream text extraction
        let mut spans = self.extract_spans_from_page(doc, page_num, page_obj_id);

        // If operator stream produced no spans, fall back to lopdf's extract_text
        if spans.is_empty() {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                if !text.trim().is_empty() {
                    spans = synthesize_spans_from_text(&text, &dimensions);
                }
            }
        }

        // Build spatial layout blocks
        let blocks = self.layout_engine.build_page_blocks(spans, &dimensions);

        // Combine plain text across blocks
        let mut raw_text = String::new();
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                raw_text.push_str("\n\n");
            }
            raw_text.push_str(&block.text);
        }

        // If blocks were empty but lopdf extract_text had content, preserve it
        if raw_text.is_empty() {
            if let Ok(t) = doc.extract_text(&[page_num]) {
                raw_text = t.trim().to_string();
            }
        }

        let char_count = raw_text.chars().count();
        let estimated_tokens = char_count.div_ceil(4);

        Ok(PdfPage {
            page_number: page_num,
            dimensions,
            blocks,
            raw_text,
            char_count,
            estimated_tokens,
        })
    }

    /// Resolves the page MediaBox or CropBox dimensions and rotation.
    fn extract_page_dimensions(&self, doc: &Document, page_obj_id: (u32, u16)) -> PageDimensions {
        let mut width = 612.0;
        let mut height = 792.0;
        let mut rotation = 0;

        if let Ok(page_dict) = doc.get_object(page_obj_id).and_then(Object::as_dict) {
            // Check rotation
            if let Ok(rot) = page_dict.get(b"Rotate").and_then(Object::as_i64) {
                rotation = (rot.rem_euclid(360)) as u32;
            }

            // Check MediaBox, falling back to CropBox
            let box_obj = page_dict
                .get(b"MediaBox")
                .or_else(|_| page_dict.get(b"CropBox"));

            if let Ok(Object::Array(arr)) = box_obj {
                if arr.len() >= 4 {
                    let x0 = to_f64(&arr[0]).unwrap_or(0.0);
                    let y0 = to_f64(&arr[1]).unwrap_or(0.0);
                    let x1 = to_f64(&arr[2]).unwrap_or(612.0);
                    let y1 = to_f64(&arr[3]).unwrap_or(792.0);

                    width = (x1 - x0).abs();
                    height = (y1 - y0).abs();
                }
            }
        }

        PageDimensions {
            width,
            height,
            rotation,
        }
    }

    /// Extracts text spans with spatial coordinates from the page's content stream operations.
    fn extract_spans_from_page(
        &self,
        doc: &Document,
        _page_num: u32,
        page_obj_id: (u32, u16),
    ) -> Vec<PdfSpan> {
        let content_data = doc.get_page_content(page_obj_id);
        if content_data.is_empty() {
            return Vec::new();
        }

        let content = match lopdf::content::Content::decode(&content_data) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut spans = Vec::new();
        let mut cur_font_name: Option<String> = None;
        let mut cur_font_size: f64 = 12.0;
        let mut cur_x: f64 = 0.0;
        let mut cur_y: f64 = 0.0;
        let mut in_text_object = false;

        for op in content.operations {
            match op.operator.as_str() {
                "BT" => {
                    in_text_object = true;
                    cur_x = 0.0;
                    cur_y = 0.0;
                }
                "ET" => {
                    in_text_object = false;
                }
                "Tf" => {
                    // Set font and size: /FontName fontSize Tf
                    if op.operands.len() >= 2 {
                        if let Object::Name(bytes) = &op.operands[0] {
                            cur_font_name = Some(String::from_utf8_lossy(bytes).to_string());
                        }
                        if let Some(size) = to_f64(&op.operands[1]) {
                            cur_font_size = size.abs();
                        }
                    }
                }
                "Td" | "TD" => {
                    // Move text position: tx ty Td
                    if op.operands.len() >= 2 {
                        let tx = to_f64(&op.operands[0]).unwrap_or(0.0);
                        let ty = to_f64(&op.operands[1]).unwrap_or(0.0);
                        cur_x += tx;
                        cur_y += ty;
                    }
                }
                "Tm" => {
                    // Set text matrix: a b c d e f Tm (e=x, f=y)
                    if op.operands.len() >= 6 {
                        let e = to_f64(&op.operands[4]).unwrap_or(0.0);
                        let f = to_f64(&op.operands[5]).unwrap_or(0.0);
                        cur_x = e;
                        cur_y = f;
                    }
                }
                "T*" => {
                    // Move to start of next line
                    cur_y -= cur_font_size * 1.2;
                }
                "Tj" => {
                    // Show text: (string) Tj
                    if in_text_object && !op.operands.is_empty() {
                        if let Some(text) = decode_text_object(&op.operands[0]) {
                            if !text.trim().is_empty() {
                                let span_w = (text.len() as f64) * (cur_font_size * 0.5);
                                let span_h = cur_font_size;
                                spans.push(PdfSpan {
                                    text: text.clone(),
                                    font_name: cur_font_name.clone(),
                                    font_size: cur_font_size,
                                    bbox: BoundingBox::new(
                                        cur_x,
                                        cur_y,
                                        cur_x + span_w,
                                        cur_y + span_h,
                                    ),
                                });
                                cur_x += span_w;
                            }
                        }
                    }
                }
                "TJ" if in_text_object && !op.operands.is_empty() => {
                    // Show text with individual glyph positioning: [array] TJ
                    if let Object::Array(arr) = &op.operands[0] {
                        let mut combined = String::new();
                        let start_x = cur_x;
                        for item in arr {
                            match item {
                                Object::String(bytes, _) => {
                                    let s = String::from_utf8_lossy(bytes);
                                    combined.push_str(&s);
                                    cur_x += (s.len() as f64) * (cur_font_size * 0.5);
                                }
                                Object::Integer(adj) => {
                                    // Glyph spacing adjustment in thousandths of a unit of text space
                                    cur_x -= (*adj as f64) * cur_font_size / 1000.0;
                                }
                                Object::Real(adj) => {
                                    cur_x -= (*adj as f64) * cur_font_size / 1000.0;
                                }
                                _ => {}
                            }
                        }
                        if !combined.trim().is_empty() {
                            spans.push(PdfSpan {
                                text: combined,
                                font_name: cur_font_name.clone(),
                                font_size: cur_font_size,
                                bbox: BoundingBox::new(
                                    start_x,
                                    cur_y,
                                    cur_x,
                                    cur_y + cur_font_size,
                                ),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        spans
    }

    /// Extracts document outline hierarchy (bookmarks).
    fn extract_outlines(&self, doc: &Document) -> Vec<OutlineItem> {
        let mut outline = Vec::new();

        let root_dict = match doc.trailer.get(b"Root") {
            Ok(Object::Reference(id)) => doc.get_object(*id).and_then(Object::as_dict).ok(),
            Ok(Object::Dictionary(dict)) => Some(dict),
            _ => None,
        };

        if let Some(root) = root_dict {
            if let Ok(outlines_ref) = root.get(b"Outlines") {
                if let Ok(outlines_dict) = match outlines_ref {
                    Object::Reference(id) => doc.get_object(*id).and_then(Object::as_dict),
                    Object::Dictionary(dict) => Ok(dict),
                    _ => return outline,
                } {
                    if let Ok(Object::Reference(first_id)) = outlines_dict.get(b"First") {
                        self.traverse_outline_items(doc, *first_id, &mut outline);
                    }
                }
            }
        }

        outline
    }

    /// Recursively traverses linked outline items (`/Next`, `/First`).
    fn traverse_outline_items(
        &self,
        doc: &Document,
        mut current_id: (u32, u16),
        items: &mut Vec<OutlineItem>,
    ) {
        let mut visited = std::collections::HashSet::new();

        while visited.insert(current_id) {
            if let Ok(item_dict) = doc.get_object(current_id).and_then(Object::as_dict) {
                let title = item_dict
                    .get(b"Title")
                    .ok()
                    .and_then(get_string_or_name)
                    .unwrap_or_else(|| "Untitled Section".to_string());

                let mut children = Vec::new();
                if let Ok(Object::Reference(child_id)) = item_dict.get(b"First") {
                    self.traverse_outline_items(doc, *child_id, &mut children);
                }

                items.push(OutlineItem {
                    title,
                    page_number: None, // Simplified page destination
                    children,
                });

                // Follow /Next sibling
                if let Ok(Object::Reference(next_id)) = item_dict.get(b"Next") {
                    current_id = *next_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

/// Helper function to convert an Object to f64.
fn to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(r) => Some(*r as f64),
        Object::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

/// Helper function to extract a string or name value from an Object.
fn get_string_or_name(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        _ => None,
    }
}

/// Decodes text operand from string or name.
fn decode_text_object(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        _ => None,
    }
}

/// Synthesizes spans with synthetic bounding boxes from plain text lines when operator stream is unavailable.
fn synthesize_spans_from_text(text: &str, dimensions: &PageDimensions) -> Vec<PdfSpan> {
    let mut spans = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return spans;
    }

    let line_height = 14.0;
    let mut cur_y = dimensions.height - 40.0;

    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let width = (trimmed.len() as f64) * 7.0;
            spans.push(PdfSpan {
                text: trimmed.to_string(),
                font_name: None,
                font_size: 11.0,
                bbox: BoundingBox::new(40.0, cur_y - line_height, 40.0 + width, cur_y),
            });
        }
        cur_y -= line_height * 1.2;
    }

    spans
}

/// Parses standard PDF dates (e.g. `D:YYYYMMDDHHmmSSOHH'mm'`).
fn parse_pdf_date(date_str: &str) -> Option<DateTime<Utc>> {
    let s = date_str.strip_prefix("D:").unwrap_or(date_str);
    if s.len() < 8 {
        return None;
    }

    let year: i32 = s.get(0..4)?.parse().ok()?;
    let month: u32 = s.get(4..6)?.parse().ok()?;
    let day: u32 = s.get(6..8)?.parse().ok()?;

    let (hour, min, sec) = if s.len() >= 14 {
        let h: u32 = s.get(8..10)?.parse().ok()?;
        let m: u32 = s.get(10..12)?.parse().ok()?;
        let sec: u32 = s.get(12..14)?.parse().ok()?;
        (h, m, sec)
    } else {
        (0, 0, 0)
    };

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, min, sec))
        .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
}
