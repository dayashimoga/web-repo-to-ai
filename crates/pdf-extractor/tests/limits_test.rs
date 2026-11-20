//! PDF resource limits and safety controls test suite.

use lopdf::{Dictionary, Document, Object, Stream};
use web_repo_to_ai_cdm::ExtractionStatus;
use web_repo_to_ai_pdf_extractor::error::PdfError;
use web_repo_to_ai_pdf_extractor::extractor::{PdfExtractionOptions, PdfExtractor};
use web_repo_to_ai_pdf_extractor::limits::PdfLimits;
use web_repo_to_ai_pdf_extractor::parser::{PdfParser, PdfParserOptions};

/// Creates a synthetic multi-page document with N text pages.
fn create_n_page_pdf(n: usize) -> Vec<u8> {
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

    let mut page_ids = Vec::with_capacity(n);
    for i in 1..=n {
        let content = format!("BT /F1 12 Tf 72 700 Td (Content for Page {i}) Tj ET");
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));

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
        page_ids.push(Object::Reference(page_id));
    }

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(page_ids)),
        ("Count", Object::Integer(n as i64)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).expect("Failed to serialize PDF");
    bytes
}

#[test]
fn test_file_size_limit_rejection() {
    let pdf_bytes = create_n_page_pdf(2);
    let bytes_len = pdf_bytes.len();

    // Set limit smaller than the document
    let limits = PdfLimits::default().with_max_file_size_bytes(bytes_len - 10);
    let parser_opts = PdfParserOptions {
        limits,
        ..Default::default()
    };
    let parser = PdfParser::new().with_options(parser_opts);

    let err = parser
        .parse(&pdf_bytes)
        .expect_err("Should reject document exceeding max_file_size_bytes");

    match err {
        PdfError::LimitExceeded {
            limit_name,
            max_allowed,
            actual,
        } => {
            assert_eq!(limit_name, "file_size_bytes");
            assert_eq!(max_allowed, bytes_len - 10);
            assert_eq!(actual, bytes_len);
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_page_count_limit_strict_rejection() {
    let pdf_bytes = create_n_page_pdf(5);

    // Strict page limit: fail if page count > 3
    let limits = PdfLimits::default()
        .with_max_page_count(3)
        .with_truncate_on_page_limit(false);
    let parser_opts = PdfParserOptions {
        limits,
        ..Default::default()
    };
    let parser = PdfParser::new().with_options(parser_opts);

    let err = parser
        .parse(&pdf_bytes)
        .expect_err("Should reject 5-page document with limit of 3");

    match err {
        PdfError::LimitExceeded {
            limit_name,
            max_allowed,
            actual,
        } => {
            assert_eq!(limit_name, "page_count");
            assert_eq!(max_allowed, 3);
            assert_eq!(actual, 5);
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_page_count_limit_graceful_truncation_and_cdm() {
    let pdf_bytes = create_n_page_pdf(5);

    // Graceful page truncation limit: process only first 2 of 5 pages
    let limits = PdfLimits::default()
        .with_max_page_count(2)
        .with_truncate_on_page_limit(true);

    let opts = PdfExtractionOptions {
        limits,
        ..Default::default()
    };
    let extractor = PdfExtractor::with_options(opts);

    let cdm = extractor
        .extract_to_cdm(&pdf_bytes)
        .expect("Truncation mode should succeed gracefully");

    // Total discovered was 5, but only 2 were extracted
    assert_eq!(cdm.completeness.discovered_count, 5);
    assert_eq!(cdm.completeness.extracted_count, 2);
    assert_eq!(cdm.completeness.excluded_count, 3);

    // Invariant: discovered = extracted + excluded
    assert_eq!(
        cdm.completeness.discovered_count,
        cdm.completeness.extracted_count + cdm.completeness.excluded_count
    );

    // Status is Partial because limit truncated pages
    assert_eq!(cdm.extraction_status, ExtractionStatus::Partial);

    // Exactly 2 sections emitted for pages 1 and 2
    assert_eq!(cdm.sections.len(), 2);
    assert_eq!(cdm.sections[0].title, Some("Page 1".to_string()));
    assert_eq!(cdm.sections[1].title, Some("Page 2".to_string()));

    // Truncation flag in metadata
    assert_eq!(
        cdm.metadata
            .extra
            .get("truncated_by_limit")
            .map(String::as_str),
        Some("true")
    );
}
