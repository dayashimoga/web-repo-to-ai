//! Table and provenance extraction end-to-end integration test suite.

use lopdf::{Dictionary, Document, Object, Stream, StringFormat};
use web_repo_to_ai_cdm::SectionKind;
use web_repo_to_ai_pdf_extractor::extractor::PdfExtractor;

fn create_pdf_with_table_and_outline() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Page 1 content stream with prose and table
    // Running page header at (50, 765) [y >= 744.48]
    // Paragraph 1 at (50, 720) [y < 744.48]
    // Table Caption at (50, 695)
    // Table Row 0 at y=660: Col 0 at 50, Col 1 at 200, Col 2 at 350
    // Table Row 1 at y=630: Col 0 at 50, Col 1 at 200, Col 2 at 350
    // Table Row 2 at y=600: Col 0 at 50, Col 1 at 200, Col 2 at 350
    // Paragraph 2 at (50, 560)
    let content = b"\
BT /F1 9 Tf 50 765 Td (Annual Engineering Report 2026) Tj ET \
BT /F1 12 Tf 50 720 Td (Database throughput evaluation report.) Tj ET \
BT /F1 11 Tf 50 695 Td (Table 1: Engine Benchmark Comparison) Tj ET \
BT /F1 11 Tf 50 660 Td (Database) Tj ET \
BT /F1 11 Tf 200 660 Td (TPS) Tj ET \
BT /F1 11 Tf 350 660 Td (Latency) Tj ET \
BT /F1 11 Tf 50 630 Td (Postgres) Tj ET \
BT /F1 11 Tf 200 630 Td (12500) Tj ET \
BT /F1 11 Tf 350 630 Td (4.2ms) Tj ET \
BT /F1 11 Tf 50 600 Td (SQLite) Tj ET \
BT /F1 11 Tf 200 600 Td (8200) Tj ET \
BT /F1 11 Tf 350 600 Td (2.1ms) Tj ET \
BT /F1 11 Tf 50 560 Td (All benchmarks completed with 100 concurrent clients.) Tj ET \
";

    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.to_vec()));

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(page_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let outline_item_id = doc.add_object(Dictionary::from_iter(vec![(
        "Title",
        Object::String(b"Benchmark Results".to_vec(), StringFormat::Literal),
    )]));

    let outlines_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Outlines".to_vec())),
        ("First", Object::Reference(outline_item_id)),
        ("Last", Object::Reference(outline_item_id)),
        ("Count", Object::Integer(1)),
    ]));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
        ("Outlines", Object::Reference(outlines_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let info_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(b"Database Benchmarks".to_vec(), StringFormat::Literal),
        ),
        (
            "Author",
            Object::String(b"Performance Team".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf).expect("save pdf with table");
    buf
}

#[test]
fn test_end_to_end_table_and_provenance_extraction() {
    let pdf_bytes = create_pdf_with_table_and_outline();
    let extractor = PdfExtractor::new();
    let source_url = "https://example.com/reports/benchmarks.pdf";

    let cdm_doc = extractor
        .extract_to_cdm_with_url(&pdf_bytes, source_url)
        .expect("extract CDM document");

    // Must have at least 2 sections: 1 prose section and 1 table section
    assert!(cdm_doc.sections.len() >= 2);

    // Find table section
    let table_sec = cdm_doc
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Table)
        .expect("expected at least one table section");

    // Table section verification
    assert_eq!(
        table_sec.attributes.get("is_table").map(|s| s.as_str()),
        Some("true")
    );
    assert_eq!(
        table_sec.attributes.get("page_number").map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(
        table_sec
            .attributes
            .get("table_columns")
            .map(|s| s.as_str()),
        Some("3")
    );
    assert_eq!(
        table_sec.attributes.get("table_rows").map(|s| s.as_str()),
        Some("3")
    );
    assert_eq!(
        table_sec.attributes.get("content_type").map(|s| s.as_str()),
        Some("table/markdown")
    );

    // Table body contains valid GFM table
    assert!(table_sec.body.contains("| Database | TPS | Latency |"));
    assert!(table_sec.body.contains("| Postgres | 12500 | 4.2ms |"));
    assert!(table_sec.body.contains("| SQLite | 8200 | 2.1ms |"));

    // Find prose section
    let prose_sec = cdm_doc
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Prose)
        .expect("expected at least one prose section");

    assert!(prose_sec
        .body
        .contains("Database throughput evaluation report"));
    assert!(prose_sec.body.contains("All benchmarks completed"));
    assert_eq!(
        prose_sec.attributes.get("page_number").map(|s| s.as_str()),
        Some("1")
    );

    // Verify Hierarchy
    assert_eq!(cdm_doc.hierarchy.name, "Database Benchmarks");
    assert!(!cdm_doc.hierarchy.children.is_empty());
    assert_eq!(cdm_doc.hierarchy.children[0].name, "Benchmark Results");
}
