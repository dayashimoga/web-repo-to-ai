//! PDF extractor integration test suite.

use lopdf::{Dictionary, Document, Object, Stream, StringFormat};
use web_repo_to_ai_cdm::{SectionKind, SourceType};
use web_repo_to_ai_pdf_extractor::extractor::PdfExtractor;

fn create_multi_page_pdf() -> Vec<u8> {
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

    // Page 1 content
    let content1 = b"BT /F1 16 Tf 72 700 Td (Chapter 1: Overview) Tj ET BT /F1 11 Tf 72 650 Td (Content on page one.) Tj ET";
    let content1_id = doc.add_object(Stream::new(Dictionary::new(), content1.to_vec()));

    let page1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content1_id)),
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

    // Page 2 content
    let content2 = b"BT /F1 16 Tf 72 700 Td (Chapter 2: Architecture) Tj ET BT /F1 11 Tf 72 650 Td (Content on page two.) Tj ET";
    let content2_id = doc.add_object(Stream::new(Dictionary::new(), content2.to_vec()));

    let page2_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content2_id)),
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
        (
            "Kids",
            Object::Array(vec![
                Object::Reference(page1_id),
                Object::Reference(page2_id),
            ]),
        ),
        ("Count", Object::Integer(2)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let info_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(b"System Specification".to_vec(), StringFormat::Literal),
        ),
        (
            "Author",
            Object::String(b"System Team".to_vec(), StringFormat::Literal),
        ),
        (
            "Keywords",
            Object::String(b"system, spec, docs".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf).expect("save multi-page pdf");
    buf
}

#[test]
fn test_extractor_cdm_mapping() {
    let pdf_bytes = create_multi_page_pdf();
    let extractor = PdfExtractor::new();
    let source_url = "https://example.com/docs/system-spec.pdf";

    let cdm_doc = extractor
        .extract_to_cdm_with_url(&pdf_bytes, source_url)
        .expect("extract CDM document");

    // Metadata checks
    assert_eq!(cdm_doc.source_type, SourceType::DocumentPdf);
    assert_eq!(
        cdm_doc.metadata.title.as_deref(),
        Some("System Specification")
    );
    assert_eq!(cdm_doc.metadata.authors, vec!["System Team"]);
    assert_eq!(cdm_doc.metadata.tags, vec!["system", "spec", "docs"]);
    assert_eq!(
        cdm_doc
            .metadata
            .extra
            .get("pdf_version")
            .map(|s| s.as_str()),
        Some("1.5")
    );
    assert_eq!(
        cdm_doc.metadata.extra.get("page_count").map(|s| s.as_str()),
        Some("2")
    );

    // Source identity
    assert_eq!(
        cdm_doc.source_identity.raw_input,
        "https://example.com/docs/system-spec.pdf"
    );
    assert_eq!(
        cdm_doc.source_identity.canonical_url.as_str(),
        "https://example.com/docs/system-spec.pdf"
    );

    // Completeness
    assert_eq!(cdm_doc.completeness.discovered_count, 2);
    assert_eq!(cdm_doc.completeness.eligible_count, 2);
    assert_eq!(cdm_doc.completeness.extracted_count, 2);
    assert_eq!(cdm_doc.completeness.failed_count, 0);

    // Sections (1 per page)
    assert_eq!(cdm_doc.sections.len(), 2);

    let sec1 = &cdm_doc.sections[0];
    assert_eq!(sec1.title.as_deref(), Some("Page 1"));
    assert_eq!(sec1.kind, SectionKind::Prose);
    assert!(sec1.body.contains("Chapter 1: Overview"));
    assert_eq!(
        sec1.attributes.get("page_number").map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(
        sec1.attributes.get("width").map(|s| s.as_str()),
        Some("612")
    );
    assert_eq!(
        sec1.attributes.get("height").map(|s| s.as_str()),
        Some("792")
    );

    let sec2 = &cdm_doc.sections[1];
    assert_eq!(sec2.title.as_deref(), Some("Page 2"));
    assert_eq!(sec2.kind, SectionKind::Prose);
    assert!(sec2.body.contains("Chapter 2: Architecture"));
    assert_eq!(
        sec2.attributes.get("page_number").map(|s| s.as_str()),
        Some("2")
    );
}

#[test]
fn test_extractor_invalid_input() {
    let extractor = PdfExtractor::new();
    let err = extractor
        .extract_to_cdm_with_url(b"not a pdf", "https://example.com/invalid.pdf")
        .unwrap_err();
    assert!(matches!(
        err,
        web_repo_to_ai_pdf_extractor::error::PdfError::InvalidHeader(_)
    ));
}
