//! # Scanned Document Detection & Image Analysis
//!
//! Evaluates page text density and raster image coverage heuristics to identify scanned pages,
//! calculate scanned document confidence scores, and recommend OCR processing when text is absent or minimal.

use crate::models::PageDimensions;
use lopdf::{Document, Object};
use serde::{Deserialize, Serialize};

/// Metadata for a raster image discovered in a PDF page's `/Resources` dictionary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfImageInfo {
    /// The indirect object ID of the image stream.
    pub object_id: (u32, u16),

    /// The name key in the page's `/XObject` dictionary (e.g. `Im1`).
    pub name: String,

    /// Pixel width of the image.
    pub pixel_width: u32,

    /// Pixel height of the image.
    pub pixel_height: u32,

    /// Color space if specified (e.g. `DeviceRGB`, `DeviceGray`).
    pub color_space: Option<String>,

    /// Color depth per color component (e.g. 8).
    pub bits_per_component: Option<u32>,

    /// Compression filter applied to the image stream (e.g. `DCTDecode` for JPEG).
    pub filter: Option<String>,
}

/// Heuristic evaluation options for identifying scanned pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannedDetectorOptions {
    /// Minimum extractable character count below which a page containing images is deemed scanned (default: 50).
    pub min_chars_per_page: usize,

    /// Minimum character density (characters per point squared) threshold (default: 0.0002).
    pub min_text_density: f64,

    /// Ratio of scanned pages required to flag the document as predominantly scanned (default: 0.50).
    pub predominantly_scanned_threshold: f64,
}

impl Default for ScannedDetectorOptions {
    fn default() -> Self {
        Self {
            min_chars_per_page: 50,
            min_text_density: 0.0002,
            predominantly_scanned_threshold: 0.50,
        }
    }
}

/// Analysis outcome for an individual PDF page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannedPageAnalysis {
    /// 1-based page index.
    pub page_number: u32,

    /// Total characters of extracted digital text.
    pub char_count: usize,

    /// Text density in characters per point squared (`char_count / (width * height)`).
    pub text_density: f64,

    /// Number of raster images discovered on this page.
    pub image_count: usize,

    /// Detailed image metadata for each raster image on the page.
    pub images: Vec<PdfImageInfo>,

    /// Whether this page is classified as scanned.
    pub is_scanned: bool,

    /// Confidence score from 0.0 (definitely digital text) to 1.0 (pure scanned bitmap).
    pub confidence: f64,

    /// Explanatory diagnostic message.
    pub reason: String,
}

/// Summary analysis across an entire PDF document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannedDocumentAnalysis {
    /// Total number of pages evaluated.
    pub total_pages: u32,

    /// Number of pages identified as scanned.
    pub scanned_page_count: u32,

    /// Ratio of scanned pages to total pages (`0.0 ..= 1.0`).
    pub scanned_ratio: f64,

    /// Whether the document as a whole is predominantly scanned (>= threshold).
    pub is_predominantly_scanned: bool,

    /// Whether OCR processing is recommended to capture full content.
    pub ocr_recommended: bool,

    /// Per-page analyses.
    pub page_analyses: Vec<ScannedPageAnalysis>,
}

impl Default for ScannedDocumentAnalysis {
    fn default() -> Self {
        Self {
            total_pages: 0,
            scanned_page_count: 0,
            scanned_ratio: 0.0,
            is_predominantly_scanned: false,
            ocr_recommended: false,
            page_analyses: Vec::new(),
        }
    }
}

/// Discovers and extracts raster image XObjects for a specific page.
#[must_use]
pub fn extract_page_images(doc: &Document, page_obj_id: (u32, u16)) -> Vec<PdfImageInfo> {
    let mut images = Vec::new();

    let page_dict = match doc.get_object(page_obj_id).and_then(Object::as_dict) {
        Ok(d) => d,
        Err(_) => return images,
    };

    // Page may define Resources directly or via an indirect reference
    let resources_obj = match page_dict.get(b"Resources") {
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => obj,
            Err(_) => return images,
        },
        Ok(obj) => obj,
        Err(_) => return images,
    };

    let resources_dict = match resources_obj.as_dict().ok() {
        Some(d) => d,
        None => return images,
    };

    // In Resources, check XObject dictionary
    let xobject_obj = match resources_dict.get(b"XObject") {
        Ok(Object::Reference(id)) => match doc.get_object(*id) {
            Ok(obj) => obj,
            Err(_) => return images,
        },
        Ok(obj) => obj,
        Err(_) => return images,
    };

    let xobject_dict = match xobject_obj.as_dict().ok() {
        Some(d) => d,
        None => return images,
    };

    for (name_bytes, obj_val) in xobject_dict.iter() {
        let obj_id = match obj_val {
            Object::Reference(id) => *id,
            _ => continue,
        };

        let stream_obj = match doc.get_object(obj_id) {
            Ok(obj) => obj,
            Err(_) => continue,
        };

        let stream_dict = match stream_obj.as_stream().ok() {
            Some(s) => &s.dict,
            None => continue,
        };

        // Check if Subtype is Image
        let is_image = stream_dict
            .get(b"Subtype")
            .ok()
            .and_then(|obj| obj.as_name().ok())
            .map(|n| n == b"Image")
            .unwrap_or(false);

        if !is_image {
            continue;
        }

        let name = String::from_utf8_lossy(name_bytes).to_string();
        let pixel_width = stream_dict
            .get(b"Width")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .unwrap_or(0) as u32;
        let pixel_height = stream_dict
            .get(b"Height")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .unwrap_or(0) as u32;
        let color_space = stream_dict
            .get(b"ColorSpace")
            .ok()
            .and_then(|obj| obj.as_name().ok())
            .map(|n| String::from_utf8_lossy(n).to_string());
        let bits_per_component = stream_dict
            .get(b"BitsPerComponent")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .map(|b| b as u32);
        let filter = stream_dict
            .get(b"Filter")
            .ok()
            .and_then(|obj| obj.as_name().ok())
            .map(|n| String::from_utf8_lossy(n).to_string());

        images.push(PdfImageInfo {
            object_id: obj_id,
            name,
            pixel_width,
            pixel_height,
            color_space,
            bits_per_component,
            filter,
        });
    }

    images
}

/// Evaluates page metrics to classify whether a page is scanned.
#[must_use]
pub fn analyze_page_scanned(
    page_number: u32,
    dimensions: &PageDimensions,
    char_count: usize,
    images: &[PdfImageInfo],
    options: &ScannedDetectorOptions,
) -> ScannedPageAnalysis {
    let page_area = (dimensions.width * dimensions.height).max(1.0);
    let text_density = char_count as f64 / page_area;
    let image_count = images.len();

    let (is_scanned, confidence, reason) = if image_count > 0 && char_count == 0 {
        (
            true,
            1.0,
            format!(
                "Page has {image_count} image(s) and zero extractable text (pure scanned page)"
            ),
        )
    } else if image_count > 0 && char_count < options.min_chars_per_page {
        (
            true,
            0.85,
            format!(
                "Page has {image_count} image(s) with minimal text ({char_count} chars < threshold {})",
                options.min_chars_per_page
            ),
        )
    } else if image_count > 0 && text_density < options.min_text_density {
        (
            true,
            0.75,
            format!(
                "Low text density ({text_density:.6} chars/pt² < threshold {:.6}) with {image_count} image(s)",
                options.min_text_density
            ),
        )
    } else if image_count == 0 && char_count == 0 {
        (false, 0.0, "Blank page (no text and no images)".to_string())
    } else {
        (
            false,
            0.0,
            format!("Normal digital text ({char_count} chars, density: {text_density:.6})"),
        )
    };

    ScannedPageAnalysis {
        page_number,
        char_count,
        text_density,
        image_count,
        images: images.to_vec(),
        is_scanned,
        confidence,
        reason,
    }
}

/// Aggregates per-page analyses into a document-level summary report.
#[must_use]
pub fn analyze_document_scanned(
    page_analyses: &[ScannedPageAnalysis],
    options: &ScannedDetectorOptions,
) -> ScannedDocumentAnalysis {
    let total_pages = page_analyses.len() as u32;
    let scanned_page_count = page_analyses.iter().filter(|p| p.is_scanned).count() as u32;
    let scanned_ratio = if total_pages > 0 {
        scanned_page_count as f64 / total_pages as f64
    } else {
        0.0
    };
    let is_predominantly_scanned = scanned_ratio >= options.predominantly_scanned_threshold;
    let ocr_recommended = scanned_page_count > 0;

    ScannedDocumentAnalysis {
        total_pages,
        scanned_page_count,
        scanned_ratio,
        is_predominantly_scanned,
        ocr_recommended,
        page_analyses: page_analyses.to_vec(),
    }
}
