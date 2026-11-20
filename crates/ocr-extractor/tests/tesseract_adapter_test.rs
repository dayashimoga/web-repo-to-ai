//! # Tesseract Adapter Integration Tests
//!
//! Validates Tesseract adapter configuration, process isolation, unavailable binary
//! detection, PSM/OEM parameter mapping, and failure semantics.

use std::path::PathBuf;
use web_repo_to_ai_ocr_extractor::{
    OcrAdapter, OcrEngineMode, OcrError, OcrInput, OcrOptions, PageSegmentationMode,
    TesseractAdapter,
};

#[tokio::test]
async fn test_adapter_configuration() {
    let adapter = TesseractAdapter::new().with_binary_path("/custom/bin/tesseract");
    assert_eq!(
        adapter.binary_path(),
        PathBuf::from("/custom/bin/tesseract").as_path()
    );
}

#[tokio::test]
async fn test_unavailable_binary_detection() {
    let adapter = TesseractAdapter::new().with_binary_path("non_existent_tesseract_binary_xyz_123");

    // Availability check must return false gracefully without panicking
    assert!(!adapter.is_available().await);

    // Engine info must report available: false
    let info = adapter
        .engine_info()
        .await
        .expect("engine_info succeeds with available: false");
    assert!(!info.available);
    assert_eq!(info.name, "Tesseract OCR");

    // Extraction attempt must fail with TesseractNotFound
    let input = OcrInput::from_bytes(b"\x89PNG\r\n\x1a\nfake_image_bytes".to_vec())
        .with_filename("test.png");
    let options = OcrOptions::default();

    let err = adapter
        .extract_text(&input, &options)
        .await
        .expect_err("Must fail when binary does not exist");

    match err {
        OcrError::TesseractNotFound { path, .. } => {
            assert!(path.contains("non_existent_tesseract_binary_xyz_123"));
        }
        other => panic!("Expected TesseractNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_psm_and_oem_flag_mappings() {
    assert_eq!(PageSegmentationMode::OsdOnly.as_flag(), "0");
    assert_eq!(PageSegmentationMode::AutoOsd.as_flag(), "1");
    assert_eq!(PageSegmentationMode::AutoOnly.as_flag(), "2");
    assert_eq!(PageSegmentationMode::Auto.as_flag(), "3");
    assert_eq!(PageSegmentationMode::SingleColumn.as_flag(), "4");
    assert_eq!(PageSegmentationMode::SingleBlockVert.as_flag(), "5");
    assert_eq!(PageSegmentationMode::SingleBlock.as_flag(), "6");
    assert_eq!(PageSegmentationMode::SingleLine.as_flag(), "7");
    assert_eq!(PageSegmentationMode::SingleWord.as_flag(), "8");
    assert_eq!(PageSegmentationMode::CircleWord.as_flag(), "9");
    assert_eq!(PageSegmentationMode::SingleChar.as_flag(), "10");
    assert_eq!(PageSegmentationMode::SparseText.as_flag(), "11");
    assert_eq!(PageSegmentationMode::SparseTextOsd.as_flag(), "12");
    assert_eq!(PageSegmentationMode::RawLine.as_flag(), "13");

    assert_eq!(OcrEngineMode::LegacyOnly.as_flag(), "0");
    assert_eq!(OcrEngineMode::LstmOnly.as_flag(), "1");
    assert_eq!(OcrEngineMode::Combined.as_flag(), "2");
    assert_eq!(OcrEngineMode::Default.as_flag(), "3");
}

#[tokio::test]
async fn test_parse_tsv_direct_helper() {
    let adapter = TesseractAdapter::new();
    const TSV: &str = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t50\t50\t100\t20\t95.0\tDirect
5\t1\t1\t1\t1\t2\t160\t50\t80\t20\t92.0\tParse";

    let result = adapter
        .parse_tsv(TSV, 35)
        .await
        .expect("Direct parse succeeds");
    assert_eq!(result.total_words, 2);
    assert_eq!(result.text, "Direct Parse");
    assert_eq!(result.execution_duration_ms, 35);
}
