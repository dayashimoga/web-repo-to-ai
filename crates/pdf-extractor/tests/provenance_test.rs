//! Page and section provenance test suite.

use web_repo_to_ai_cdm::{CdmSectionBuilder, SectionKind};
use web_repo_to_ai_pdf_extractor::models::{BoundingBox, PageDimensions};
use web_repo_to_ai_pdf_extractor::provenance::{PageProvenance, SectionProvenance};

#[test]
fn test_section_provenance_attributes() {
    let bbox = BoundingBox::new(72.0, 500.0, 540.0, 680.0);
    let prov = SectionProvenance {
        page_number: 3,
        bbox,
        reading_order: 2,
        column_index: Some(1),
        parent_heading: Some("2. System Architecture".to_string()),
        content_type: "text/prose".to_string(),
        is_table: false,
        table_id: None,
    };

    let attrs = prov.to_attributes();
    assert_eq!(attrs.get("page_number").map(|s| s.as_str()), Some("3"));
    assert_eq!(attrs.get("x0").map(|s| s.as_str()), Some("72.00"));
    assert_eq!(attrs.get("y0").map(|s| s.as_str()), Some("500.00"));
    assert_eq!(attrs.get("x1").map(|s| s.as_str()), Some("540.00"));
    assert_eq!(attrs.get("y1").map(|s| s.as_str()), Some("680.00"));
    assert_eq!(attrs.get("width").map(|s| s.as_str()), Some("468.00"));
    assert_eq!(attrs.get("height").map(|s| s.as_str()), Some("180.00"));
    assert_eq!(attrs.get("reading_order").map(|s| s.as_str()), Some("2"));
    assert_eq!(attrs.get("column_index").map(|s| s.as_str()), Some("1"));
    assert_eq!(
        attrs.get("parent_heading").map(|s| s.as_str()),
        Some("2. System Architecture")
    );
    assert_eq!(
        attrs.get("content_type").map(|s| s.as_str()),
        Some("text/prose")
    );
    assert_eq!(attrs.get("is_table").map(|s| s.as_str()), Some("false"));

    // Verify attaching to CdmSectionBuilder
    let sec_builder = CdmSectionBuilder::new(
        "sec_3",
        "https://example.com/doc.pdf",
        SectionKind::Prose,
        "Section content here.",
    );
    let section = prov.attach_to_builder(sec_builder).build();

    assert_eq!(
        section.attributes.get("page_number").map(|s| s.as_str()),
        Some("3")
    );
    assert_eq!(
        section.attributes.get("x0").map(|s| s.as_str()),
        Some("72.00")
    );
    assert_eq!(
        section.attributes.get("column_index").map(|s| s.as_str()),
        Some("1")
    );
}

#[test]
fn test_table_section_provenance_attributes() {
    let bbox = BoundingBox::new(50.0, 300.0, 450.0, 420.0);
    let prov = SectionProvenance {
        page_number: 1,
        bbox,
        reading_order: 1,
        column_index: None,
        parent_heading: Some("Performance Metrics".to_string()),
        content_type: "table/markdown".to_string(),
        is_table: true,
        table_id: Some("page-1-table-1".to_string()),
    };

    let attrs = prov.to_attributes();
    assert_eq!(attrs.get("page_number").map(|s| s.as_str()), Some("1"));
    assert_eq!(attrs.get("is_table").map(|s| s.as_str()), Some("true"));
    assert_eq!(
        attrs.get("table_id").map(|s| s.as_str()),
        Some("page-1-table-1")
    );
    assert_eq!(
        attrs.get("content_type").map(|s| s.as_str()),
        Some("table/markdown")
    );
}

#[test]
fn test_page_provenance_attributes() {
    let dims = PageDimensions {
        width: 612.0,
        height: 792.0,
        rotation: 0,
    };
    let page_prov = PageProvenance {
        page_number: 1,
        dimensions: dims,
        block_count: 5,
        table_count: 1,
        char_count: 1200,
        estimated_tokens: 300,
    };

    let attrs = page_prov.to_attributes();
    assert_eq!(attrs.get("page_number").map(|s| s.as_str()), Some("1"));
    assert_eq!(attrs.get("page_width").map(|s| s.as_str()), Some("612.00"));
    assert_eq!(attrs.get("page_height").map(|s| s.as_str()), Some("792.00"));
    assert_eq!(attrs.get("block_count").map(|s| s.as_str()), Some("5"));
    assert_eq!(attrs.get("table_count").map(|s| s.as_str()), Some("1"));
    assert_eq!(attrs.get("char_count").map(|s| s.as_str()), Some("1200"));
    assert_eq!(
        attrs.get("estimated_tokens").map(|s| s.as_str()),
        Some("300")
    );
}
