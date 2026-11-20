//! # OCR Extractor Crate (`web-repo-to-ai-ocr-extractor`)
//!
//! Provides isolated Optical Character Recognition (OCR) execution via Tesseract CLI,
//! TSV parsing, bounding box & confidence score extraction, and Canonical Context
//! Document Model (`CdmDocument`) synthesis.

pub mod adapter;
pub mod confidence;
pub mod error;
pub mod extractor;
pub mod golden;
pub mod language;
pub mod limits;
pub mod models;
pub mod preprocess;
pub mod provenance;
pub mod tesseract;
pub mod tsv;

pub use adapter::OcrAdapter;
pub use confidence::{
    find_low_confidence_words, ConfidenceFilterOptions, ConfidenceStats, ConfidenceTier,
    LowConfidenceWord,
};
pub use error::{OcrError, Result};
pub use extractor::OcrExtractor;
pub use golden::{
    all_golden_fixtures, build_clean_document_fixture, build_dark_mode_fixture,
    build_low_contrast_fixture, build_multicolumn_fixture, build_noisy_fixture,
    build_skewed_document_fixture, calculate_cer, calculate_fragment_recall, calculate_wer,
    encode_gray_png, evaluate_golden_fixture, levenshtein_distance, render_text,
    run_all_golden_benchmarks, MockOcrAdapter, OcrArchetype, OcrFidelityMetrics, OcrGoldenFixture,
    OcrGoldenReport, OcrGoldenResult,
};
pub use language::{
    canonicalize_languages, is_cjk, is_rtl, normalize_language_code, parse_language_string,
    resolve_languages, LanguageFallbackPolicy,
};
pub use limits::OcrLimits;
pub use models::{
    BoundingBox, OcrBlock, OcrEngineInfo, OcrEngineMode, OcrInput, OcrLine, OcrOptions, OcrPage,
    OcrParagraph, OcrResult, OcrWord, PageSegmentationMode,
};
pub use preprocess::{
    apply_adaptive_threshold, apply_threshold, compute_otsu_threshold, detect_skew_angle,
    enhance_contrast, preprocess_image, rotate_gray_image, BinarizationMethod,
    PreprocessingOptions, PreprocessingReport,
};
pub use provenance::{
    NormalizedBoundingBox, OcrBlockProvenance, OcrPageProvenance, OcrProvenanceReport,
};
pub use tesseract::TesseractAdapter;
pub use tsv::{parse_tesseract_tsv, parse_tesseract_tsv_with_options};
