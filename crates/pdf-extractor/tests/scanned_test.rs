//! Scanned PDF detection and image analysis integration test suite.

use lopdf::{Dictionary, Document, Object, Stream};
use web_repo_to_ai_cdm::ExtractionStatus;
use web_repo_to_ai_pdf_extractor::extractor::{PdfExtractionOptions, PdfExtractor};
use web_repo_to_ai_pdf_extractor::parser::PdfParser;
use web_repo_to_ai_pdf_extractor::scanned::ScannedDetectorOptions;

/// Creates an in-memory PDF where Page 1 has pure digital text,
/// Page 2 is a pure scanned page with a raster image and no text,
/// and Page 3 is a blank page.
fn create_mixed_digital_and_scanned_pdf() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    // Font dictionary
    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    // Image XObject stream representing a scanned bitmap
    let mut image_dict = Dictionary::new();
    image_dict.set("Type", Object::Name(b"XObject".to_vec()));
    image_dict.set("Subtype", Object::Name(b"Image".to_vec()));
    image_dict.set("Width", Object::Integer(1600));
    image_dict.set("Height", Object::Integer(2400));
    image_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    image_dict.set("BitsPerComponent", Object::Integer(8));
    image_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
    let image_stream = Stream::new(image_dict, vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00]);
    let image_id = doc.add_object(image_stream);

    // Resources for digital page (Font only)
    let digital_res_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Resources for scanned page (Image XObject only)
    let scanned_res_id = doc.add_object(Dictionary::from_iter(vec![(
        "XObject",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "Im1",
            Object::Reference(image_id),
        )])),
    )]));

    // Empty resources for blank page
    let blank_res_id = doc.add_object(Dictionary::new());

    // Content for Page 1 (normal digital text)
    let text_content = b"BT /F1 14 Tf 72 700 Td (Overview of Distributed Database Architecture) Tj ET BT /F1 11 Tf 72 650 Td (This document explains horizontal sharding and consensus protocols in modern distributed databases. High availability requires replication across three or more availability zones.) Tj ET";
    let text_content_id = doc.add_object(Stream::new(Dictionary::new(), text_content.to_vec()));

    // Content for Page 2 (empty text stream for scanned page)
    let empty_content_id = doc.add_object(Stream::new(Dictionary::new(), Vec::new()));

    // Page 1: Digital text
    let page1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(text_content_id)),
        ("Resources", Object::Reference(digital_res_id)),
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

    // Page 2: Scanned image with no text
    let page2_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(empty_content_id)),
        ("Resources", Object::Reference(scanned_res_id)),
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

    // Page 3: Blank page (no text, no image)
    let page3_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(empty_content_id)),
        ("Resources", Object::Reference(blank_res_id)),
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
                Object::Reference(page3_id),
            ]),
        ),
        ("Count", Object::Integer(3)),
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
fn test_scanned_page_detection_and_heuristics() {
    let pdf_bytes = create_mixed_digital_and_scanned_pdf();
    let parser = PdfParser::new();
    let doc = parser
        .parse(&pdf_bytes)
        .expect("Should parse mixed PDF successfully");

    assert_eq!(doc.total_pages, 3);
    assert_eq!(doc.pages.len(), 3);

    // Page 1: Digital text
    let p1 = &doc.pages[0];
    assert_eq!(p1.page_number, 1);
    assert!(
        !p1.is_scanned,
        "Page 1 is digital text, should not be marked scanned"
    );
    assert!(p1.char_count > 100);
    assert!(p1.text_density > 0.0002);
    assert_eq!(p1.images.len(), 0);
    assert_eq!(p1.scanned_confidence, 0.0);

    // Page 2: Scanned bitmap
    let p2 = &doc.pages[1];
    assert_eq!(p2.page_number, 2);
    assert!(
        p2.is_scanned,
        "Page 2 has an image and 0 text, must be marked scanned"
    );
    assert_eq!(p2.char_count, 0);
    assert_eq!(p2.text_density, 0.0);
    assert_eq!(p2.images.len(), 1);
    assert_eq!(p2.scanned_confidence, 1.0);
    assert_eq!(p2.images[0].name, "Im1");
    assert_eq!(p2.images[0].pixel_width, 1600);
    assert_eq!(p2.images[0].pixel_height, 2400);
    assert_eq!(p2.images[0].color_space.as_deref(), Some("DeviceRGB"));
    assert_eq!(p2.images[0].filter.as_deref(), Some("DCTDecode"));

    // Page 3: Blank page
    let p3 = &doc.pages[2];
    assert_eq!(p3.page_number, 3);
    assert!(
        !p3.is_scanned,
        "Page 3 is blank, should not be marked scanned"
    );
    assert_eq!(p3.char_count, 0);
    assert_eq!(p3.images.len(), 0);
    assert_eq!(p3.scanned_confidence, 0.0);

    // Document-level summary
    let summary = &doc.scanned_analysis;
    assert_eq!(summary.total_pages, 3);
    assert_eq!(summary.scanned_page_count, 1);
    assert!((summary.scanned_ratio - 1.0 / 3.0).abs() < 0.001);
    assert!(!summary.is_predominantly_scanned, "1 of 3 is not >= 50%");
    assert!(
        summary.ocr_recommended,
        "OCR recommended when scanned pages exist"
    );
}

#[test]
fn test_predominantly_scanned_pdf_and_cdm_mapping() {
    let pdf_bytes = create_mixed_digital_and_scanned_pdf();
    let opts = PdfExtractionOptions {
        source_url: Some("https://example.com/mixed_report.pdf".to_string()),
        // Adjust threshold to 0.30 so 1 of 3 pages (33.3%) triggers predominantly_scanned
        scanned_options: ScannedDetectorOptions {
            min_chars_per_page: 50,
            min_text_density: 0.0002,
            predominantly_scanned_threshold: 0.30,
        },
        ..Default::default()
    };

    let extractor = PdfExtractor::with_options(opts);
    let cdm = extractor
        .extract_to_cdm(&pdf_bytes)
        .expect("Extraction to CDM should succeed");

    // Check document-level metadata
    let extra = &cdm.metadata.extra;
    assert_eq!(extra.get("is_scanned").map(String::as_str), Some("true"));
    assert_eq!(
        extra.get("scanned_page_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        extra.get("ocr_recommended").map(String::as_str),
        Some("true")
    );

    // Check extraction status: predominantly scanned marks status Partial
    assert_eq!(cdm.extraction_status, ExtractionStatus::Partial);

    // Check completeness report
    assert_eq!(cdm.completeness.discovered_count, 3);
    assert_eq!(cdm.completeness.extracted_count, 3);
    assert_eq!(cdm.completeness.excluded_count, 0);

    // Check section provenance attributes on Page 2 (scanned)
    let page2_sec = cdm
        .sections
        .iter()
        .find(|s| s.attributes.get("page_number").map(String::as_str) == Some("2"))
        .expect("Page 2 section should exist");
    assert_eq!(
        page2_sec.attributes.get("is_scanned").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        page2_sec.attributes.get("image_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        page2_sec
            .attributes
            .get("scanned_confidence")
            .map(String::as_str),
        Some("1.0000")
    );
}
