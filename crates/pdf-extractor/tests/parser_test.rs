//! PDF parser integration test suite.

use lopdf::{Dictionary, Document, Object, Stream, StringFormat};
use web_repo_to_ai_pdf_extractor::error::PdfError;
use web_repo_to_ai_pdf_extractor::parser::PdfParser;

fn create_valid_in_memory_pdf() -> Vec<u8> {
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

    let content = b"BT /F1 16 Tf 72 700 Td (Document Title) Tj ET BT /F1 11 Tf 72 650 Td (First paragraph content here.) Tj ET";
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
        Object::String(b"Overview Section".to_vec(), StringFormat::Literal),
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
            Object::String(
                b"Engineering Architecture Report".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "Author",
            Object::String(b"Lead Architect".to_vec(), StringFormat::Literal),
        ),
        (
            "Subject",
            Object::String(
                b"PDF Extraction Specification".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "Creator",
            Object::String(b"DocGen".to_vec(), StringFormat::Literal),
        ),
        (
            "Producer",
            Object::String(b"Rustlopdf".to_vec(), StringFormat::Literal),
        ),
        (
            "Keywords",
            Object::String(
                b"rust, pdf, extraction, layout".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "CreationDate",
            Object::String(b"D:20261120120000".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf).expect("save valid pdf");
    buf
}

#[test]
fn test_empty_document_error() {
    let parser = PdfParser::new();
    let err = parser.parse(&[]).unwrap_err();
    match err {
        PdfError::EmptyDocument => {}
        other => panic!("Expected EmptyDocument, got {:?}", other),
    }
}

#[test]
fn test_invalid_header_errors() {
    let parser = PdfParser::new();

    // Too short
    let err = parser.parse(b"%PD").unwrap_err();
    assert!(matches!(err, PdfError::InvalidHeader(_)));

    // Not a PDF
    let err = parser
        .parse(b"<!DOCTYPE html><html><body>Not a PDF</body></html>")
        .unwrap_err();
    assert!(matches!(err, PdfError::InvalidHeader(_)));
}

#[test]
fn test_malformed_structure_error() {
    let parser = PdfParser::new();
    // Valid header but garbage content
    let garbage = b"%PDF-1.4\nCorrupted binary garbage without xref or catalog\n%%EOF";
    let err = parser.parse(garbage).unwrap_err();
    assert!(matches!(err, PdfError::MalformedStructure(_)));
}

#[test]
fn test_valid_pdf_parsing_end_to_end() {
    let pdf_bytes = create_valid_in_memory_pdf();
    let parser = PdfParser::new();

    let doc = parser.parse(&pdf_bytes).expect("parse valid pdf");

    // Version
    assert_eq!(doc.metadata.pdf_version, "1.5");

    // Metadata
    assert_eq!(
        doc.metadata.title.as_deref(),
        Some("Engineering Architecture Report")
    );
    assert_eq!(doc.metadata.author.as_deref(), Some("Lead Architect"));
    assert_eq!(
        doc.metadata.subject.as_deref(),
        Some("PDF Extraction Specification")
    );
    assert_eq!(doc.metadata.creator.as_deref(), Some("DocGen"));
    assert_eq!(doc.metadata.producer.as_deref(), Some("Rustlopdf"));
    assert_eq!(
        doc.metadata.keywords,
        vec!["rust", "pdf", "extraction", "layout"]
    );
    assert!(doc.metadata.creation_date.is_some());

    // Pages
    assert_eq!(doc.total_pages, 1);
    assert_eq!(doc.pages.len(), 1);

    let page = &doc.pages[0];
    assert_eq!(page.page_number, 1);
    assert_eq!(page.dimensions.width, 612.0);
    assert_eq!(page.dimensions.height, 792.0);
    assert_eq!(page.dimensions.rotation, 0);

    // Text content
    assert!(page.raw_text.contains("Document Title"));
    assert!(page.raw_text.contains("First paragraph content here."));
    assert!(doc.total_characters > 0);
    assert!(doc.total_estimated_tokens > 0);

    // Outlines
    assert_eq!(doc.outline.len(), 1);
    assert_eq!(doc.outline[0].title, "Overview Section");
}
