//! # OCR Domain Models and Configuration Types
//!
//! Strongly-typed representations of bounding boxes, extracted words,
//! lines, paragraphs, blocks, pages, and execution options.

use crate::limits::OcrLimits;
use serde::{Deserialize, Serialize};

/// 2D bounding box representing positional coordinates on an image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Left horizontal coordinate in pixels.
    pub left: f64,
    /// Top vertical coordinate in pixels.
    pub top: f64,
    /// Width of the bounding box in pixels.
    pub width: f64,
    /// Height of the bounding box in pixels.
    pub height: f64,
}

impl BoundingBox {
    /// Creates a new bounding box from coordinates and dimensions.
    #[must_use]
    pub const fn new(left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    /// Computes the union bounding box enclosing both boxes.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let left = self.left.min(other.left);
        let top = self.top.min(other.top);
        let right = (self.left + self.width).max(other.left + other.width);
        let bottom = (self.top + self.height).max(other.top + other.height);
        Self {
            left,
            top,
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
        }
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// An individual word token recognized by the OCR engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrWord {
    /// Recognized text content of the word.
    pub text: String,
    /// Confidence score normalized between 0.0 and 1.0.
    pub confidence: f64,
    /// Pixel bounding box.
    pub bbox: BoundingBox,
    /// 1-indexed line number containing this word.
    pub line_num: usize,
    /// 1-indexed word number within the line.
    pub word_num: usize,
}

/// A line composed of sequential word tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrLine {
    /// Words comprising this line.
    pub words: Vec<OcrWord>,
    /// Reconstructed textual content of the line.
    pub text: String,
    /// Mean confidence score across words in this line (0.0 to 1.0).
    pub confidence: f64,
    /// Enclosing bounding box of the line.
    pub bbox: BoundingBox,
    /// 1-indexed line number within the paragraph.
    pub line_num: usize,
}

/// A paragraph of text comprising one or more lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrParagraph {
    /// Lines comprising this paragraph.
    pub lines: Vec<OcrLine>,
    /// Reconstructed textual content of the paragraph.
    pub text: String,
    /// Mean confidence score across all words in this paragraph.
    pub confidence: f64,
    /// Enclosing bounding box.
    pub bbox: BoundingBox,
    /// 1-indexed paragraph number within the block.
    pub par_num: usize,
}

/// A structural block (e.g. column, caption, text block) recognized on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrBlock {
    /// 1-indexed block identifier within the page.
    pub block_num: usize,
    /// Paragraphs contained in this block.
    pub paragraphs: Vec<OcrParagraph>,
    /// Reconstructed textual content of the entire block.
    pub text: String,
    /// Mean confidence score across all words in this block.
    pub confidence: f64,
    /// Enclosing bounding box.
    pub bbox: BoundingBox,
}

/// A single processed page containing recognized blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrPage {
    /// 1-indexed page number.
    pub page_num: usize,
    /// Structural content blocks extracted from the page.
    pub blocks: Vec<OcrBlock>,
    /// Reconstructed text for the entire page.
    pub text: String,
    /// Mean confidence score across all words on the page.
    pub mean_confidence: f64,
    /// Total word count recognized on this page.
    pub word_count: usize,
}

/// Comprehensive result of an OCR extraction operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    /// Processed pages.
    pub pages: Vec<OcrPage>,
    /// Aggregate text content across all pages.
    pub text: String,
    /// Aggregate mean confidence score across all words (0.0 to 1.0).
    pub mean_confidence: f64,
    /// Total number of words recognized.
    pub total_words: usize,
    /// Duration of the OCR process execution in milliseconds.
    pub execution_duration_ms: u64,
    /// Metadata identifying the OCR engine and version used.
    pub engine_info: OcrEngineInfo,
}

/// Information describing the active OCR engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OcrEngineInfo {
    /// Engine identifier (e.g. "Tesseract OCR").
    pub name: String,
    /// Version string reported by the engine.
    pub version: String,
    /// Whether the engine binary is installed and executable.
    pub available: bool,
    /// Path to the engine binary if resolved.
    pub path: Option<String>,
    /// List of available language codes (e.g. ["eng", "fra", "deu"]).
    pub supported_languages: Vec<String>,
}

impl Default for OcrEngineInfo {
    fn default() -> Self {
        Self {
            name: "Tesseract OCR".to_string(),
            version: "unknown".to_string(),
            available: false,
            path: None,
            supported_languages: Vec::new(),
        }
    }
}

/// Tesseract Page Segmentation Modes (PSM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageSegmentationMode {
    /// Orientation and script detection (OSD) only.
    OsdOnly = 0,
    /// Automatic page segmentation with OSD.
    AutoOsd = 1,
    /// Automatic page segmentation, but no OSD, or OCR.
    AutoOnly = 2,
    /// Fully automatic page segmentation, but no OSD (Default).
    #[default]
    Auto = 3,
    /// Assume a single column of text of variable sizes.
    SingleColumn = 4,
    /// Assume a single uniform block of vertically aligned text.
    SingleBlockVert = 5,
    /// Assume a single uniform block of text.
    SingleBlock = 6,
    /// Treat the image as a single text line.
    SingleLine = 7,
    /// Treat the image as a single word.
    SingleWord = 8,
    /// Treat the image as a single word in a circle.
    CircleWord = 9,
    /// Treat the image as a single character.
    SingleChar = 10,
    /// Sparse text. Find as much text as possible in no particular order.
    SparseText = 11,
    /// Sparse text with OSD.
    SparseTextOsd = 12,
    /// Raw line. Treat the image as a single text line.
    RawLine = 13,
}

impl PageSegmentationMode {
    /// Returns the integer CLI argument value for Tesseract `--psm <value>`.
    #[must_use]
    pub const fn as_flag(&self) -> &'static str {
        match self {
            Self::OsdOnly => "0",
            Self::AutoOsd => "1",
            Self::AutoOnly => "2",
            Self::Auto => "3",
            Self::SingleColumn => "4",
            Self::SingleBlockVert => "5",
            Self::SingleBlock => "6",
            Self::SingleLine => "7",
            Self::SingleWord => "8",
            Self::CircleWord => "9",
            Self::SingleChar => "10",
            Self::SparseText => "11",
            Self::SparseTextOsd => "12",
            Self::RawLine => "13",
        }
    }
}

/// Tesseract OCR Engine Modes (OEM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OcrEngineMode {
    /// Legacy engine only.
    LegacyOnly = 0,
    /// Neural nets LSTM engine only.
    LstmOnly = 1,
    /// Legacy + LSTM engines.
    Combined = 2,
    /// Default based on what is available.
    #[default]
    Default = 3,
}

impl OcrEngineMode {
    /// Returns the integer CLI argument value for Tesseract `--oem <value>`.
    #[must_use]
    pub const fn as_flag(&self) -> &'static str {
        match self {
            Self::LegacyOnly => "0",
            Self::LstmOnly => "1",
            Self::Combined => "2",
            Self::Default => "3",
        }
    }
}

/// Input payload submitted for OCR recognition.
#[derive(Debug, Clone)]
pub struct OcrInput {
    /// Raw binary image data.
    pub bytes: Vec<u8>,
    /// Optional filename or identifier.
    pub filename: Option<String>,
    /// Optional MIME type (e.g. "image/png", "image/jpeg", "image/tiff").
    pub mime_type: Option<String>,
    /// Optional origin URL.
    pub source_url: Option<String>,
}

impl OcrInput {
    /// Creates a new `OcrInput` from raw byte payload.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            filename: None,
            mime_type: None,
            source_url: None,
        }
    }

    /// Attaches an origin filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Attaches a MIME content type.
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Attaches an origin source URL.
    #[must_use]
    pub fn with_source_url(mut self, source_url: impl Into<String>) -> Self {
        self.source_url = Some(source_url.into());
        self
    }
}

/// Configuration options passed to OCR engine adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrOptions {
    /// Target language code or combinations (e.g. "eng", "eng+fra").
    pub language: String,
    /// Page segmentation mode.
    pub psm: PageSegmentationMode,
    /// OCR engine mode.
    pub oem: OcrEngineMode,
    /// Optional path to custom `tessdata` language models directory.
    pub tessdata_dir: Option<String>,
    /// Resource and safety limits.
    pub limits: OcrLimits,
    /// Additional arguments passed to the underlying engine.
    pub extra_args: Vec<String>,
}

impl Default for OcrOptions {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegmentationMode::Auto,
            oem: OcrEngineMode::Default,
            tessdata_dir: None,
            limits: OcrLimits::default(),
            extra_args: Vec::new(),
        }
    }
}

impl OcrOptions {
    /// Sets the target recognition language.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Sets the page segmentation mode (PSM).
    #[must_use]
    pub const fn with_psm(mut self, psm: PageSegmentationMode) -> Self {
        self.psm = psm;
        self
    }

    /// Sets the OCR engine mode (OEM).
    #[must_use]
    pub const fn with_oem(mut self, oem: OcrEngineMode) -> Self {
        self.oem = oem;
        self
    }

    /// Sets the path to custom tessdata directory.
    #[must_use]
    pub fn with_tessdata_dir(mut self, dir: impl Into<String>) -> Self {
        self.tessdata_dir = Some(dir.into());
        self
    }

    /// Sets execution safety limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: OcrLimits) -> Self {
        self.limits = limits;
        self
    }
}
