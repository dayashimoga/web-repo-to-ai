//! # OCR Accuracy & Fidelity Benchmark Tests
//!
//! Validates quantitative OCR accuracy metrics (WER, CER, Recall), Levenshtein edit distance,
//! in-memory bitmap glyph rendering, and comprehensive golden document archetype evaluations.

use std::sync::Arc;
use web_repo_to_ai_ocr_extractor::{
    all_golden_fixtures, build_clean_document_fixture, build_dark_mode_fixture,
    build_low_contrast_fixture, build_multicolumn_fixture, build_noisy_fixture,
    build_skewed_document_fixture, calculate_cer, calculate_fragment_recall, calculate_wer,
    encode_gray_png, evaluate_golden_fixture, levenshtein_distance, render_text,
    run_all_golden_benchmarks, MockOcrAdapter, OcrArchetype,
};

#[test]
fn test_levenshtein_distance_cases() {
    // Exact match
    assert_eq!(levenshtein_distance(&['a', 'b', 'c'], &['a', 'b', 'c']), 0);
    // Substitution
    assert_eq!(levenshtein_distance(&['c', 'a', 't'], &['b', 'a', 't']), 1);
    // Deletion
    assert_eq!(
        levenshtein_distance(&['c', 'a', 't', 's'], &['c', 'a', 't']),
        1
    );
    // Insertion
    assert_eq!(
        levenshtein_distance(&['c', 'a', 't'], &['c', 'h', 'a', 't']),
        1
    );
    // Empty cases
    assert_eq!(levenshtein_distance(&['a', 'b'], &[]), 2);
    assert_eq!(levenshtein_distance(&[], &['x', 'y', 'z']), 3);
    assert_eq!(levenshtein_distance::<char>(&[], &[]), 0);
}

#[test]
fn test_wer_and_cer_metrics() {
    let truth = "Deep learning models for document analysis";
    let perfect = "Deep learning models for document analysis";
    assert_eq!(calculate_wer(truth, perfect), 0.0);
    assert_eq!(calculate_cer(truth, perfect), 0.0);
    assert_eq!(
        calculate_fragment_recall(&["Deep", "models", "analysis"], perfect),
        1.0
    );

    // 1 substitution out of 6 words: WER = 1/6 ~= 0.1667
    let one_sub = "Deep learning networks for document analysis";
    let wer = calculate_wer(truth, one_sub);
    assert!((wer - (1.0 / 6.0)).abs() < 1e-4);

    // Completely disjoint text: WER = 1.0
    let disjoint = "Completely unrelated words here";
    assert!(calculate_wer(truth, disjoint) >= 0.80);
    assert_eq!(
        calculate_fragment_recall(&["Deep", "models"], disjoint),
        0.0
    );
}

#[test]
fn test_bitmap_font_renderer_and_png_encoding() {
    let mut img = image::GrayImage::from_pixel(400, 200, image::Luma([255]));
    render_text(&mut img, "HELLO WORLD\n12345", 20, 20, 2, 8, 0);

    // Check pixel at top-left is white background
    assert_eq!(img.get_pixel(0, 0)[0], 255);
    // Check that some text pixels were colored black (0)
    let has_black = img.pixels().any(|p| p[0] == 0);
    assert!(has_black, "Text should render non-white pixels");

    // Encode to PNG and verify magic bytes
    let bytes = encode_gray_png(&img);
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "PNG magic header");
}

#[test]
fn test_all_fixtures_have_valid_images_and_fragments() {
    let fixtures = all_golden_fixtures();
    assert_eq!(fixtures.len(), 6);

    for fix in fixtures {
        assert!(!fix.image_bytes.is_empty());
        assert_eq!(&fix.image_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
        assert!(!fix.expected_fragments.is_empty());
        assert!(fix.expected_min_words > 0);
        assert!(!fix.ground_truth_text.is_empty());
        assert!(fix.dimensions.0 > 0 && fix.dimensions.1 > 0);
    }
}

#[tokio::test]
async fn test_individual_fixture_evaluations() {
    // 1. Clean Document
    let clean = build_clean_document_fixture();
    let adapter_clean = Arc::new(MockOcrAdapter::from_fixture(&clean));
    let res_clean = evaluate_golden_fixture(&clean, adapter_clean)
        .await
        .expect("clean evaluation should succeed");
    assert_eq!(res_clean.archetype, OcrArchetype::CleanDocument);
    assert!(res_clean.metrics.passed);
    assert_eq!(res_clean.metrics.fragment_recall, 1.0);
    assert_eq!(res_clean.metrics.wer, 0.0);

    // 2. Dark Mode Inverted
    let dark = build_dark_mode_fixture();
    let adapter_dark = Arc::new(MockOcrAdapter::from_fixture(&dark));
    let res_dark = evaluate_golden_fixture(&dark, adapter_dark)
        .await
        .expect("dark mode evaluation should succeed");
    assert!(res_dark.metrics.passed);
    assert_eq!(res_dark.metrics.wer, 0.0);
    assert!(res_dark.preprocessing_report.is_some());
    assert!(res_dark.preprocessing_report.unwrap().inverted_applied);

    // 3. Skewed Document
    let skewed = build_skewed_document_fixture();
    let adapter_skewed = Arc::new(MockOcrAdapter::from_fixture(&skewed));
    let res_skewed = evaluate_golden_fixture(&skewed, adapter_skewed)
        .await
        .expect("skewed evaluation should succeed");
    assert!(res_skewed.metrics.passed);

    // 4. Low Contrast Document
    let low_contrast = build_low_contrast_fixture();
    let adapter_contrast = Arc::new(MockOcrAdapter::from_fixture(&low_contrast));
    let res_contrast = evaluate_golden_fixture(&low_contrast, adapter_contrast)
        .await
        .expect("low contrast evaluation should succeed");
    assert!(res_contrast.metrics.passed);
    assert!(res_contrast.preprocessing_report.is_some());
    assert!(res_contrast.preprocessing_report.unwrap().contrast_enhanced);

    // 5. Multi-Column Layout
    let multicol = build_multicolumn_fixture();
    let adapter_col = Arc::new(MockOcrAdapter::from_fixture(&multicol));
    let res_col = evaluate_golden_fixture(&multicol, adapter_col)
        .await
        .expect("multicolumn evaluation should succeed");
    assert!(res_col.metrics.passed);

    // 6. Noisy Document
    let noisy = build_noisy_fixture();
    let adapter_noisy = Arc::new(MockOcrAdapter::from_fixture(&noisy));
    let res_noisy = evaluate_golden_fixture(&noisy, adapter_noisy)
        .await
        .expect("noisy evaluation should succeed");
    assert!(res_noisy.metrics.passed);
}

#[tokio::test]
async fn test_full_golden_report_generation() {
    let report = run_all_golden_benchmarks()
        .await
        .expect("benchmark run should succeed");

    assert_eq!(report.results.len(), 6);
    assert_eq!(report.pass_rate, 100.0);
    assert!(report.avg_wer < 0.05);
    assert!(report.avg_cer < 0.05);

    let summary = report.to_markdown_summary();
    assert!(summary.contains("OCR Extraction Fidelity and Accuracy Benchmark Report"));
    assert!(summary.contains("Clean Printed Document"));
    assert!(summary.contains("Dark Mode Inverted Document"));
    assert!(summary.contains("Skewed Document"));
    assert!(summary.contains("Low Contrast Document"));
    assert!(summary.contains("Multi-Column Layout"));
    assert!(summary.contains("Noisy Low-Confidence Document"));
    assert!(summary.contains("100.0%"));
}
