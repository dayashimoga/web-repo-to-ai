//! # OCR Engine Adapter Abstraction
//!
//! Defines the asynchronous `OcrAdapter` trait bridging pluggable OCR backends
//! (Tesseract CLI, containerized workers, cloud fallbacks) into the pipeline.

use crate::error::Result;
use crate::models::{OcrEngineInfo, OcrInput, OcrOptions, OcrResult};
use async_trait::async_trait;

/// Asynchronous adapter interface for Optical Character Recognition (OCR) engines.
#[async_trait]
pub trait OcrAdapter: Send + Sync {
    /// Returns whether the underlying OCR engine binary or service is reachable and functional.
    async fn is_available(&self) -> bool;

    /// Retrieves detailed metadata regarding the active OCR engine (version, supported languages).
    async fn engine_info(&self) -> Result<OcrEngineInfo>;

    /// Performs OCR extraction on the provided input image with specified options.
    async fn extract_text(&self, input: &OcrInput, options: &OcrOptions) -> Result<OcrResult>;
}
