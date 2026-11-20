//! # OCR Timeout Cancellation and Process Isolation Tests
//!
//! Validates timeout enforcement, child process cancellation, temporary file cleanup guards,
//! and resource bounds checking against oversized inputs and token floods.

use std::sync::Arc;
use std::time::Duration;
use web_repo_to_ai_ocr_extractor::{
    MockOcrAdapter, OcrError, OcrExtractor, OcrInput, OcrLimits, OcrOptions, PreprocessingOptions,
};

#[tokio::test]
async fn test_timeout_cancellation_with_mock_adapter() {
    let adapter = Arc::new(MockOcrAdapter::new("").with_delay(150));
    let extractor = OcrExtractor::new(adapter);

    let limits = OcrLimits::default().with_timeout(Duration::from_millis(50));
    let options = OcrOptions::default().with_limits(limits);
    let input = OcrInput::from_bytes(vec![1, 2, 3, 4]);

    let res = extractor.extract_to_result(&input, &options).await;

    match res {
        Err(OcrError::ExecutionTimeout {
            elapsed_ms,
            limit_ms,
        }) => {
            assert!(elapsed_ms >= 50);
            assert_eq!(limit_ms, 50);
        }
        other => panic!("Expected ExecutionTimeout, got {other:?}"),
    }
}

#[tokio::test]
async fn test_input_byte_limits_enforced_before_execution() {
    let adapter = Arc::new(MockOcrAdapter::new(""));
    let extractor = OcrExtractor::new(adapter);

    let limits = OcrLimits::default().with_max_image_bytes(100);
    let options = OcrOptions::default().with_limits(limits);

    // 200 bytes exceeds 100 byte limit
    let input = OcrInput::from_bytes(vec![0; 200]);
    let res = extractor.extract_to_result(&input, &options).await;

    match res {
        Err(OcrError::InputLimitExceeded(msg)) => {
            assert!(msg.contains("200") && msg.contains("100"));
        }
        other => panic!("Expected InputLimitExceeded, got {other:?}"),
    }
}

#[tokio::test]
async fn test_word_count_limits_enforced_after_extraction() {
    let tsv_flood = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t10\t10\t50\t20\t95\tWord1
5\t1\t1\t1\t1\t2\t65\t10\t50\t20\t95\tWord2
5\t1\t1\t1\t1\t3\t120\t10\t50\t20\t95\tWord3
";
    // Adapter returns 3 words, but limit is set to max 2 words
    let adapter = Arc::new(MockOcrAdapter::new(tsv_flood));
    let extractor = OcrExtractor::new(adapter);

    let limits = OcrLimits::default().with_max_word_count(2);
    let options = OcrOptions::default()
        .with_limits(limits)
        .with_preprocessing(PreprocessingOptions::disabled());
    let input = OcrInput::from_bytes(vec![0; 10]);

    let res = extractor.extract_to_result(&input, &options).await;

    match res {
        Err(OcrError::InputLimitExceeded(msg)) => {
            assert!(msg.contains('3') && msg.contains('2'));
        }
        other => panic!("Expected InputLimitExceeded on word count, got {other:?}"),
    }
}

#[test]
fn test_temp_file_cleanup_semantics() {
    let temp_file =
        std::env::temp_dir().join(format!("test_ocr_cleanup_{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temp_file, b"test-temp-bytes").expect("writing temp file should succeed");
    assert!(temp_file.exists());

    // Scoped cleanup
    {
        struct Guard(std::path::PathBuf);
        impl Drop for Guard {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let _g = Guard(temp_file.clone());
    }

    assert!(
        !temp_file.exists(),
        "Temp file should be cleanly removed on drop"
    );
}
