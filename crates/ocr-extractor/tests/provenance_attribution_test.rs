//! # Provenance Attribution and Coordinate Normalization Integration Tests
//!
//! Validates 2D bounding box geometry, coordinate normalization ($[0.0, 1.0]$),
//! reading order attribution, section builder integration, JSON provenance exports,
//! and end-to-end CDM document attribute mapping.

use web_repo_to_ai_cdm::CdmSectionBuilder;
use web_repo_to_ai_ocr_extractor::{
    parse_tesseract_tsv_with_options, BoundingBox, ConfidenceStats, NormalizedBoundingBox,
    OcrBlockProvenance, OcrEngineInfo, OcrExtractor, OcrInput, OcrOptions, OcrPageProvenance,
    OcrProvenanceReport,
};

#[test]
fn test_bounding_box_geometry_methods() {
    let b1 = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(b1.left, 10.0);
    assert_eq!(b1.top, 20.0);
    assert_eq!(b1.width, 100.0);
    assert_eq!(b1.height, 50.0);
    assert_eq!(b1.right(), 110.0);
    assert_eq!(b1.bottom(), 70.0);
    assert_eq!(b1.area(), 5000.0);

    // Points inside and outside
    assert!(b1.contains_point(10.0, 20.0));
    assert!(b1.contains_point(60.0, 45.0));
    assert!(b1.contains_point(110.0, 70.0));
    assert!(!b1.contains_point(9.9, 20.0));
    assert!(!b1.contains_point(60.0, 70.1));

    // Intersection
    let b2 = BoundingBox::new(50.0, 30.0, 100.0, 100.0);
    assert!(b1.intersects(&b2));
    assert!(b2.intersects(&b1));

    let b3 = BoundingBox::new(200.0, 200.0, 50.0, 50.0);
    assert!(!b1.intersects(&b3));
    assert!(!b3.intersects(&b1));

    // Union
    let u = b1.union(&b3);
    assert_eq!(u.left, 10.0);
    assert_eq!(u.top, 20.0);
    assert_eq!(u.right(), 250.0);
    assert_eq!(u.bottom(), 250.0);
}

#[test]
fn test_normalized_coordinates_clamping_and_roundtrip() {
    let bbox = BoundingBox::new(200.0, 400.0, 600.0, 800.0);
    let norm = NormalizedBoundingBox::from_pixels(&bbox, 1000, 2000);

    assert!((norm.x0 - 0.20).abs() < 1e-4);
    assert!((norm.y0 - 0.20).abs() < 1e-4);
    assert!((norm.x1 - 0.80).abs() < 1e-4);
    assert!((norm.y1 - 0.60).abs() < 1e-4);
    assert!((norm.width - 0.60).abs() < 1e-4);
    assert!((norm.height - 0.40).abs() < 1e-4);

    let pixel_back = norm.to_pixels(1000, 2000);
    assert!((pixel_back.left - 200.0).abs() < 1e-4);
    assert!((pixel_back.top - 400.0).abs() < 1e-4);
    assert!((pixel_back.width - 600.0).abs() < 1e-4);
    assert!((pixel_back.height - 800.0).abs() < 1e-4);

    // Test zero dimensions safety
    let zero_norm = NormalizedBoundingBox::from_pixels(&bbox, 0, 0);
    assert_eq!(zero_norm.x0, 0.0);
    assert_eq!(zero_norm.width, 0.0);

    // Test out of bounds clamping
    let overflow_bbox = BoundingBox::new(-50.0, 950.0, 2000.0, 500.0);
    let clamped_norm = NormalizedBoundingBox::from_pixels(&overflow_bbox, 1000, 1000);
    assert_eq!(clamped_norm.x0, 0.0);
    assert_eq!(clamped_norm.x1, 1.0);
    assert_eq!(clamped_norm.y0, 0.95);
    assert_eq!(clamped_norm.y1, 1.0);
}

#[test]
fn test_block_provenance_to_section_builder() {
    let pixel_bbox = BoundingBox::new(100.0, 150.0, 400.0, 200.0);
    let norm_bbox = NormalizedBoundingBox::from_pixels(&pixel_bbox, 1000, 1000);
    let conf_stats = ConfidenceStats::compute_from_scores(&[0.95, 0.90, 0.85]);

    let prov = OcrBlockProvenance {
        page_num: 1,
        block_num: 3,
        reading_order: 2,
        pixel_bbox,
        normalized_bbox: Some(norm_bbox),
        confidence_stats: conf_stats,
        word_count: 3,
        line_count: 1,
        content_type: "ocr/block".to_string(),
    };

    let builder = CdmSectionBuilder::new(
        "test_sec".to_string(),
        "file://test.png".to_string(),
        web_repo_to_ai_cdm::SectionKind::Prose,
        "Sample block text".to_string(),
    );

    let built = prov.attach_to_builder(builder).build();
    let attrs = built.attributes;

    assert_eq!(attrs.get("ocr_page_number").unwrap(), "1");
    assert_eq!(attrs.get("ocr_block_number").unwrap(), "3");
    assert_eq!(attrs.get("ocr_reading_order").unwrap(), "2");
    assert_eq!(attrs.get("ocr_word_count").unwrap(), "3");
    assert_eq!(attrs.get("ocr_line_count").unwrap(), "1");
    assert_eq!(attrs.get("ocr_content_type").unwrap(), "ocr/block");

    assert_eq!(attrs.get("ocr_bbox_left_px").unwrap(), "100.0");
    assert_eq!(attrs.get("ocr_bbox_top_px").unwrap(), "150.0");
    assert_eq!(attrs.get("ocr_bbox_width_px").unwrap(), "400.0");
    assert_eq!(attrs.get("ocr_bbox_height_px").unwrap(), "200.0");
    assert_eq!(attrs.get("ocr_bbox_right_px").unwrap(), "500.0");
    assert_eq!(attrs.get("ocr_bbox_bottom_px").unwrap(), "350.0");

    assert_eq!(attrs.get("ocr_norm_x0").unwrap(), "0.1000");
    assert_eq!(attrs.get("ocr_norm_y0").unwrap(), "0.1500");
    assert_eq!(attrs.get("ocr_norm_x1").unwrap(), "0.5000");
    assert_eq!(attrs.get("ocr_norm_y1").unwrap(), "0.3500");
    assert_eq!(attrs.get("ocr_norm_w").unwrap(), "0.4000");
    assert_eq!(attrs.get("ocr_norm_h").unwrap(), "0.2000");

    assert_eq!(attrs.get("ocr_conf_tier").unwrap(), "high");
    assert_eq!(attrs.get("ocr_conf_mean").unwrap(), "0.9000");
}

#[test]
fn test_provenance_report_json_serialization() {
    let conf = ConfidenceStats::compute_from_scores(&[0.92, 0.88]);
    let page_prov = OcrPageProvenance {
        page_num: 1,
        image_width: Some(1920),
        image_height: Some(1080),
        block_count: 1,
        line_count: 1,
        word_count: 2,
        confidence_stats: conf.clone(),
    };

    let block_prov = OcrBlockProvenance {
        page_num: 1,
        block_num: 1,
        reading_order: 0,
        pixel_bbox: BoundingBox::new(100.0, 100.0, 300.0, 50.0),
        normalized_bbox: Some(NormalizedBoundingBox::from_pixels(
            &BoundingBox::new(100.0, 100.0, 300.0, 50.0),
            1920,
            1080,
        )),
        confidence_stats: conf.clone(),
        word_count: 2,
        line_count: 1,
        content_type: "ocr/block".to_string(),
    };

    let report = OcrProvenanceReport {
        pages: vec![page_prov],
        blocks: vec![block_prov],
        total_words: 2,
        overall_confidence: conf,
        image_dimensions: Some((1920, 1080)),
    };

    let json = report.to_json();
    assert!(json.contains("\"total_words\":2"));
    assert!(json.contains("\"image_dimensions\":[1920,1080]"));

    let pretty_json = report.to_json_pretty();
    assert!(pretty_json.contains("  \"total_words\": 2"));

    // Deserialize roundtrip check
    let parsed: OcrProvenanceReport =
        serde_json::from_str(&json).expect("provenance report JSON should parse");
    assert_eq!(parsed.total_words, 2);
    assert_eq!(parsed.blocks.len(), 1);
    assert_eq!(parsed.pages[0].image_width, Some(1920));
}

#[test]
fn test_end_to_end_extractor_provenance_cdm_mapping() {
    let tsv_data = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t60\t150\t40\t96\tHeading
5\t1\t2\t1\t1\t1\t50\t120\t200\t30\t92\tBody
5\t1\t2\t1\t1\t2\t260\t120\t100\t30\t88\tParagraph
";
    let engine = OcrEngineInfo::default();
    let opts = OcrOptions::default().with_image_dimensions(1000, 800);

    let ocr_result = parse_tesseract_tsv_with_options(tsv_data, 25, engine, &opts)
        .expect("TSV parse with options should succeed");

    assert!(ocr_result.provenance_report.is_some());
    let rep = ocr_result.provenance_report.as_ref().unwrap();
    assert_eq!(rep.blocks.len(), 2);
    assert_eq!(rep.total_words, 3);
    assert_eq!(rep.image_dimensions, Some((1000, 800)));

    let extractor = OcrExtractor::with_tesseract();
    let input = OcrInput::from_bytes(vec![0; 10])
        .with_filename("document_scan.png")
        .with_source_url("file:///scans/document_scan.png");

    let cdm = extractor
        .convert_to_cdm(&ocr_result, &input)
        .expect("CDM conversion should succeed");

    // Document-level provenance metadata
    let doc_meta = &cdm.metadata.extra;
    assert_eq!(doc_meta.get("ocr_image_width").unwrap(), "1000");
    assert_eq!(doc_meta.get("ocr_image_height").unwrap(), "800");
    assert_eq!(doc_meta.get("ocr_conf_tier").unwrap(), "high");
    assert_eq!(doc_meta.get("ocr_conf_total_words").unwrap(), "3");

    // Section-level provenance
    assert_eq!(cdm.sections.len(), 2);

    let sec1 = &cdm.sections[0];
    assert_eq!(sec1.attributes.get("ocr_block_number").unwrap(), "1");
    assert_eq!(sec1.attributes.get("ocr_reading_order").unwrap(), "0");
    assert_eq!(sec1.attributes.get("ocr_bbox_left_px").unwrap(), "50.0");
    assert_eq!(sec1.attributes.get("ocr_norm_x0").unwrap(), "0.0500");
    assert_eq!(sec1.attributes.get("ocr_norm_w").unwrap(), "0.1500");
    assert_eq!(sec1.attributes.get("ocr_norm_y0").unwrap(), "0.0750");

    let sec2 = &cdm.sections[1];
    assert_eq!(sec2.attributes.get("ocr_block_number").unwrap(), "2");
    assert_eq!(sec2.attributes.get("ocr_reading_order").unwrap(), "1");
    assert_eq!(sec2.attributes.get("ocr_bbox_left_px").unwrap(), "50.0");
    assert_eq!(sec2.attributes.get("ocr_bbox_width_px").unwrap(), "310.0"); // union of 50..250 and 260..360
    assert_eq!(sec2.attributes.get("ocr_conf_total_words").unwrap(), "2");
}
