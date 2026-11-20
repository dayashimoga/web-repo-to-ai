//! # Page and Section Provenance Attribution
//!
//! Spatial coordinates, reading order sequences, column indices,
//! parent outline headings, and CDM section metadata attribution.

use crate::models::{BoundingBox, PageDimensions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::CdmSectionBuilder;

/// Spatial and structural provenance of an entire PDF page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageProvenance {
    /// 1-based page number.
    pub page_number: u32,
    /// Dimensions and rotation of the page.
    pub dimensions: PageDimensions,
    /// Number of text blocks identified on the page.
    pub block_count: usize,
    /// Number of structured tables identified on the page.
    pub table_count: usize,
    /// Total characters extracted from the page.
    pub char_count: usize,
    /// Estimated tokens for the page.
    pub estimated_tokens: usize,
}

impl PageProvenance {
    /// Serializes page provenance to string-based attribute map.
    #[must_use]
    pub fn to_attributes(&self) -> BTreeMap<String, String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("page_number".to_string(), self.page_number.to_string());
        attrs.insert(
            "page_width".to_string(),
            format!("{:.2}", self.dimensions.width),
        );
        attrs.insert(
            "page_height".to_string(),
            format!("{:.2}", self.dimensions.height),
        );
        attrs.insert(
            "page_rotation".to_string(),
            self.dimensions.rotation.to_string(),
        );
        attrs.insert("block_count".to_string(), self.block_count.to_string());
        attrs.insert("table_count".to_string(), self.table_count.to_string());
        attrs.insert("char_count".to_string(), self.char_count.to_string());
        attrs.insert(
            "estimated_tokens".to_string(),
            self.estimated_tokens.to_string(),
        );
        attrs
    }
}

/// Detailed spatial and document provenance for an extracted section or table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionProvenance {
    /// 1-based page number where the section is located.
    pub page_number: u32,
    /// Bounding box enclosing the section in PDF points (72 dpi).
    pub bbox: BoundingBox,
    /// 0-indexed reading sequence position on the page.
    pub reading_order: u32,
    /// Column index if within a multi-column layout (0 = left, 1 = right, etc.).
    pub column_index: Option<usize>,
    /// Hierarchical path or nearest parent heading.
    pub parent_heading: Option<String>,
    /// Content type classification (e.g. `text/prose`, `table/markdown`).
    pub content_type: String,
    /// Whether this section represents an extracted table.
    pub is_table: bool,
    /// Unique table ID if this section represents a table.
    pub table_id: Option<String>,
}

impl SectionProvenance {
    /// Serializes section provenance to string-based attribute map for CDM storage.
    #[must_use]
    pub fn to_attributes(&self) -> BTreeMap<String, String> {
        let mut attrs = BTreeMap::new();
        attrs.insert("page_number".to_string(), self.page_number.to_string());
        attrs.insert("x0".to_string(), format!("{:.2}", self.bbox.x0));
        attrs.insert("y0".to_string(), format!("{:.2}", self.bbox.y0));
        attrs.insert("x1".to_string(), format!("{:.2}", self.bbox.x1));
        attrs.insert("y1".to_string(), format!("{:.2}", self.bbox.y1));
        attrs.insert("width".to_string(), format!("{:.2}", self.bbox.width()));
        attrs.insert("height".to_string(), format!("{:.2}", self.bbox.height()));
        attrs.insert("reading_order".to_string(), self.reading_order.to_string());
        attrs.insert("content_type".to_string(), self.content_type.clone());
        attrs.insert("is_table".to_string(), self.is_table.to_string());

        if let Some(col) = self.column_index {
            attrs.insert("column_index".to_string(), col.to_string());
        }
        if let Some(heading) = &self.parent_heading {
            attrs.insert("parent_heading".to_string(), heading.clone());
        }
        if let Some(tid) = &self.table_id {
            attrs.insert("table_id".to_string(), tid.clone());
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
