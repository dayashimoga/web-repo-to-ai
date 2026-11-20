//! # OCR Extractor Crate (`web-repo-to-ai-ocr-extractor`)
//!
//! Provides isolated Optical Character Recognition (OCR) execution via Tesseract CLI,
//! TSV parsing, bounding box & confidence score extraction, and Canonical Context
//! Document Model (`CdmDocument`) synthesis.

pub mod adapter;
pub mod error;
pub mod extractor;
pub mod limits;
pub mod models;
pub mod tesseract;
pub mod tsv;

pub use adapter::OcrAdapter;
pub use error::{OcrError, Result};
pub use extractor::OcrExtractor;
pub use limits::OcrLimits;
pub use models::{
    BoundingBox, OcrBlock, OcrEngineInfo, OcrEngineMode, OcrInput, OcrLine, OcrOptions, OcrPage,
    OcrParagraph, OcrResult, OcrWord, PageSegmentationMode,
};
pub use tesseract::TesseractAdapter;
pub use tsv::parse_tesseract_tsv;
