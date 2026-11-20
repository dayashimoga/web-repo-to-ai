//! # OCR Timeout, Resource Limits and DoS Safety Tests
//!
//! Validates memory ceilings, oversized image rejection, word count thresholds,
//! and timeout enforcement.

use std::time::Duration;
use web_repo_to_ai_ocr_extractor::{
    OcrAdapter, OcrError, OcrInput, OcrLimits, OcrOptions, TesseractAdapter,
};

#[test]
fn test_input_byte_limits() {
    let limits = OcrLimits::new().with_max_image_bytes(1024); // 1 KiB limit

    // Empty input must be rejected
    let empty_err = limits
        .validate_input(&[])
        .expect_err("Empty input must fail");
    assert!(matches!(empty_err, OcrError::InputLimitExceeded(_)));

    // Within limit
    assert!(limits.validate_input(&[0u8; 512]).is_ok());

    // Exceeding limit
    let oversized = vec![0u8; 2048];
    let over_err = limits
        .validate_input(&oversized)
        .expect_err("Oversized input must fail");
    assert!(matches!(over_err, OcrError::InputLimitExceeded(_)));
}

#[test]
fn test_word_count_limits() {
    let limits = OcrLimits::new().with_max_word_count(500);

    assert!(limits.validate_word_count(499).is_ok());
    assert!(limits.validate_word_count(500).is_ok());

    let err = limits
        .validate_word_count(501)
        .expect_err("501 words must fail");
    assert!(matches!(err, OcrError::InputLimitExceeded(_)));
}

#[tokio::test]
async fn test_adapter_enforces_input_limits() {
    let adapter = TesseractAdapter::new();
    let limits = OcrLimits::new().with_max_image_bytes(100);
    let options = OcrOptions::default().with_limits(limits);

    let big_input = OcrInput::from_bytes(vec![0u8; 500]).with_filename("huge.png");

    let err = adapter
        .extract_text(&big_input, &options)
        .await
        .expect_err("Adapter must enforce input limits upfront");

    assert!(matches!(err, OcrError::InputLimitExceeded(_)));
}

#[test]
fn test_custom_limits_builder() {
    let limits = OcrLimits::default()
        .with_max_image_bytes(25 * 1024 * 1024)
        .with_timeout(Duration::from_secs(15))
        .with_max_word_count(50_000)
        .with_max_pages(50);

    assert_eq!(limits.max_image_bytes, 25 * 1024 * 1024);
    assert_eq!(limits.timeout, Duration::from_secs(15));
    assert_eq!(limits.max_word_count, 50_000);
    assert_eq!(limits.max_pages, 50);
}
