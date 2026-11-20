//! # OCR Provenance Attribution and Coordinate Normalization
//!
//! Provides spatial bounding box normalization ($[0.0, 1.0]$ image coordinates),
//! hierarchical provenance tracking (page, block, reading order), CDM section
//! attribute serialization, and structured JSON provenance exports for UI overlay rendering.

use crate::confidence::ConfidenceStats;
use crate::models::BoundingBox;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::CdmSectionBuilder;

/// 2D bounding box with coordinates normalized to the unit interval $[0.0, 1.0]$
/// relative to image width and height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NormalizedBoundingBox {
    /// Normalized left horizontal coordinate ($0.0 \dots 1.0$).
    pub x0: f64,
    /// Normalized top vertical coordinate ($0.0 \dots 1.0$).
    pub y0: f64,
    /// Normalized right horizontal coordinate ($0.0 \dots 1.0$).
    pub x1: f64,
    /// Normalized bottom vertical coordinate ($0.0 \dots 1.0$).
    pub y1: f64,
    /// Normalized width ($0.0 \dots 1.0$).
    pub width: f64,
    /// Normalized height ($0.0 \dots 1.0$).
    pub height: f64,
}

impl NormalizedBoundingBox {
    /// Constructs a normalized bounding box from pixel coordinates and image dimensions.
    ///
    /// Values are strictly clamped to $[0.0, 1.0]$ to enforce geometric invariants.
    #[must_use]
    pub fn from_pixels(bbox: &BoundingBox, img_width: u32, img_height: u32) -> Self {
        if img_width == 0 || img_height == 0 {
            return Self {
                x0: 0.0,
                y0: 0.0,
                x1: 0.0,
                y1: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }

        let w = img_width as f64;
        let h = img_height as f64;

        let x0 = (bbox.left / w).clamp(0.0, 1.0);
        let y0 = (bbox.top / h).clamp(0.0, 1.0);
        let x1 = ((bbox.left + bbox.width) / w).clamp(0.0, 1.0);
        let y1 = ((bbox.top + bbox.height) / h).clamp(0.0, 1.0);

        Self {
            x0,
            y0,
            x1,
            y1,
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        }
    }

    /// Converts normalized coordinates back to pixel coordinates given image dimensions.
    #[must_use]
    pub fn to_pixels(&self, img_width: u32, img_height: u32) -> BoundingBox {
        let w = img_width as f64;
        let h = img_height as f64;

        BoundingBox {
            left: self.x0 * w,
            top: self.y0 * h,
            width: self.width * w,
            height: self.height * h,
        }
    }
}

/// Detailed spatial, structural, and confidence provenance of an extracted OCR block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrBlockProvenance {
    /// 1-based page number where the block is located.
    pub page_num: usize,
    /// 1-based block index within the page.
    pub block_num: usize,
    /// 0-indexed reading sequence position on the page.
    pub reading_order: usize,
    /// Bounding box in raw pixel coordinates.
    pub pixel_bbox: BoundingBox,
    /// Bounding box normalized to $[0.0, 1.0]$ if image dimensions are known.
    pub normalized_bbox: Option<NormalizedBoundingBox>,
    /// Comprehensive statistical confidence metrics for this block.
    pub confidence_stats: ConfidenceStats,
    /// Number of words contained in the block.
    pub word_count: usize,
    /// Number of lines contained in the block.
    pub line_count: usize,
    /// Content type classification (default: `"ocr/block"`).
    pub content_type: String,
}

impl OcrBlockProvenance {
    /// Serializes block provenance into a string key-value map for CDM storage.
    #[must_use]
    pub fn to_attributes(&self) -> BTreeMap<String, String> {
        let mut attrs = BTreeMap::new();

        attrs.insert("ocr_page_number".to_string(), self.page_num.to_string());
        attrs.insert("ocr_block_number".to_string(), self.block_num.to_string());
        attrs.insert(
            "ocr_reading_order".to_string(),
            self.reading_order.to_string(),
        );
        attrs.insert("ocr_word_count".to_string(), self.word_count.to_string());
        attrs.insert("ocr_line_count".to_string(), self.line_count.to_string());
        attrs.insert("ocr_content_type".to_string(), self.content_type.clone());

        // Pixel bounding box attributes
        attrs.insert(
            "ocr_bbox_left_px".to_string(),
            format!("{:.1}", self.pixel_bbox.left),
        );
        attrs.insert(
            "ocr_bbox_top_px".to_string(),
            format!("{:.1}", self.pixel_bbox.top),
        );
        attrs.insert(
            "ocr_bbox_width_px".to_string(),
            format!("{:.1}", self.pixel_bbox.width),
        );
        attrs.insert(
            "ocr_bbox_height_px".to_string(),
            format!("{:.1}", self.pixel_bbox.height),
        );
        attrs.insert(
            "ocr_bbox_right_px".to_string(),
            format!("{:.1}", self.pixel_bbox.left + self.pixel_bbox.width),
        );
        attrs.insert(
            "ocr_bbox_bottom_px".to_string(),
            format!("{:.1}", self.pixel_bbox.top + self.pixel_bbox.height),
        );

        // Normalized bounding box attributes if available
        if let Some(ref norm) = self.normalized_bbox {
            attrs.insert("ocr_norm_x0".to_string(), format!("{:.4}", norm.x0));
            attrs.insert("ocr_norm_y0".to_string(), format!("{:.4}", norm.y0));
            attrs.insert("ocr_norm_x1".to_string(), format!("{:.4}", norm.x1));
            attrs.insert("ocr_norm_y1".to_string(), format!("{:.4}", norm.y1));
            attrs.insert("ocr_norm_w".to_string(), format!("{:.4}", norm.width));
            attrs.insert("ocr_norm_h".to_string(), format!("{:.4}", norm.height));
        }

        // Confidence attributes
        for (k, v) in self.confidence_stats.to_attributes("ocr_conf") {
            attrs.insert(k, v);
        }

        attrs
    }

    /// Attaches all provenance attributes to a `CdmSectionBuilder`.
    #[must_use]
    pub fn attach_to_builder(&self, mut builder: CdmSectionBuilder) -> CdmSectionBuilder {
        for (k, v) in self.to_attributes() {
            builder = builder.attribute(k, v);
        }
        builder
    }
}

/// Provenance metadata representing an entire processed OCR page or image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrPageProvenance {
    /// 1-based page number.
    pub page_num: usize,
    /// Image width in pixels if known.
    pub image_width: Option<u32>,
    /// Image height in pixels if known.
    pub image_height: Option<u32>,
    /// Total text blocks identified.
    pub block_count: usize,
    /// Total lines identified.
    pub line_count: usize,
    /// Total words identified.
    pub word_count: usize,
    /// Statistical confidence metrics for the page.
    pub confidence_stats: ConfidenceStats,
}

impl OcrPageProvenance {
    /// Serializes page provenance to string key-value attributes for CDM storage.
    #[must_use]
    pub fn to_attributes(&self) -> BTreeMap<String, String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("page_number".to_string(), self.page_num.to_string());
        attrs.insert("block_count".to_string(), self.block_count.to_string());
        attrs.insert("line_count".to_string(), self.line_count.to_string());
        attrs.insert("word_count".to_string(), self.word_count.to_string());

        if let Some(w) = self.image_width {
            attrs.insert("image_width".to_string(), w.to_string());
        }
        if let Some(h) = self.image_height {
            attrs.insert("image_height".to_string(), h.to_string());
        }

        for (k, v) in self.confidence_stats.to_attributes("page_conf") {
            attrs.insert(k, v);
        }

        attrs
    }
}

/// Complete document-level provenance report aggregating all pages and blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrProvenanceReport {
    /// List of page-level provenance items.
    pub pages: Vec<OcrPageProvenance>,
    /// List of all extracted block provenance items in reading order.
    pub blocks: Vec<OcrBlockProvenance>,
    /// Total word count.
    pub total_words: usize,
    /// Aggregate confidence statistics.
    pub overall_confidence: ConfidenceStats,
    /// Source image dimensions if known.
    pub image_dimensions: Option<(u32, u32)>,
}

impl OcrProvenanceReport {
    /// Serializes the provenance report to a compact JSON string.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serializes the provenance report to a formatted JSON string.
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_bounding_box() {
        let bbox = BoundingBox::new(100.0, 200.0, 300.0, 400.0);
        let norm = NormalizedBoundingBox::from_pixels(&bbox, 1000, 2000);

        assert!((norm.x0 - 0.1).abs() < 1e-4);
        assert!((norm.y0 - 0.1).abs() < 1e-4);
        assert!((norm.x1 - 0.4).abs() < 1e-4);
        assert!((norm.y1 - 0.3).abs() < 1e-4);
        assert!((norm.width - 0.3).abs() < 1e-4);
        assert!((norm.height - 0.2).abs() < 1e-4);

        let roundtrip = norm.to_pixels(1000, 2000);
        assert!((roundtrip.left - 100.0).abs() < 1e-4);
        assert!((roundtrip.top - 200.0).abs() < 1e-4);
        assert!((roundtrip.width - 300.0).abs() < 1e-4);
        assert!((roundtrip.height - 400.0).abs() < 1e-4);
    }

    #[test]
    fn test_block_provenance_to_attributes() {
        let pixel_bbox = BoundingBox::new(50.0, 60.0, 200.0, 100.0);
        let norm_bbox = NormalizedBoundingBox::from_pixels(&pixel_bbox, 1000, 1000);
        let conf_stats = ConfidenceStats::compute_from_scores(&[0.92, 0.88]);

        let prov = OcrBlockProvenance {
            page_num: 1,
            block_num: 2,
            reading_order: 1,
            pixel_bbox,
            normalized_bbox: Some(norm_bbox),
            confidence_stats: conf_stats,
            word_count: 2,
            line_count: 1,
            content_type: "ocr/block".to_string(),
        };

        let attrs = prov.to_attributes();
        assert_eq!(attrs.get("ocr_page_number").unwrap(), "1");
        assert_eq!(attrs.get("ocr_block_number").unwrap(), "2");
        assert_eq!(attrs.get("ocr_reading_order").unwrap(), "1");
        assert_eq!(attrs.get("ocr_bbox_left_px").unwrap(), "50.0");
        assert_eq!(attrs.get("ocr_bbox_width_px").unwrap(), "200.0");
        assert_eq!(attrs.get("ocr_norm_x0").unwrap(), "0.0500");
        assert_eq!(attrs.get("ocr_norm_w").unwrap(), "0.2000");
        assert_eq!(attrs.get("ocr_conf_tier").unwrap(), "high");
    }
}
