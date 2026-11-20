//! # Resource Bounds and DoS Limits Tests
//!
//! Validates enforcement of maximum input byte sizes, empty input rejection,
//! malformed document handling, and unrecognized format defense.

use web_repo_to_ai_structured_extractor::{
    StructuredError, StructuredExtractor, StructuredInput, StructuredLimits, StructuredOptions,
};

#[test]
fn test_empty_input_rejection() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str("");
    let options = StructuredOptions::default();

    let res = extractor.extract_to_cdm(&input, &options);
    match res {
        Err(StructuredError::LimitExceeded(msg)) => {
            assert!(msg.contains("empty"));
        }
        other => panic!("Expected LimitExceeded on empty input, got {other:?}"),
    }
}

#[test]
fn test_max_input_bytes_limit_exceeded() {
    let extractor = StructuredExtractor::new();
    let limits = StructuredLimits::default().with_max_input_bytes(50);
    let options = StructuredOptions::default().with_limits(limits);

    let large_yaml =
        "openapi: 3.0.0\ninfo:\n  title: This string is way longer than fifty bytes limit\n";
    let input = StructuredInput::from_str(large_yaml);

    let res = extractor.extract_to_cdm(&input, &options);
    match res {
        Err(StructuredError::LimitExceeded(msg)) => {
            assert!(msg.contains("exceeds maximum allowed limit"));
        }
        other => panic!("Expected LimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_unrecognized_format_rejection() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str("Just some random unformatted text without structure");
    let options = StructuredOptions::default();

    let res = extractor.extract_to_cdm(&input, &options);
    match res {
        Err(StructuredError::UnrecognizedFormat(_)) => {}
        other => panic!("Expected UnrecognizedFormat, got {other:?}"),
    }
}
