//! # OCR Extractor Crate (`web-repo-to-ai-ocr-extractor`)
//!
//! Provides isolated Optical Character Recognition (OCR) execution via Tesseract CLI,
//! TSV parsing, bounding box & confidence score extraction, and Canonical Context
//! Document Model (`CdmDocument`) synthesis.

pub mod adapter;
pub mod error;
pub mod extractor;
pub mod language;
pub mod limits;
pub mod models;
pub mod preprocess;
pub mod tesseract;
pub mod tsv;

pub use adapter::OcrAdapter;
pub use error::{OcrError, Result};
pub use extractor::OcrExtractor;
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
pub use tesseract::TesseractAdapter;
pub use tsv::parse_tesseract_tsv;
