//! # OCR Golden Accuracy & Fidelity Benchmark Engine (`golden.rs`)
//!
//! Provides a representative golden document corpus, synthetic image generation,
//! quantitative fidelity evaluation (Word Error Rate, Character Error Rate, Recall, IoU),
//! confidence calibration, and Markdown audit reporting across OCR archetypes:
//!
//! 1. **Clean Printed Document:** High-contrast text on clean background (standard layout).
//! 2. **Dark Mode Inverted Document:** White text on dark/black background testing auto-inversion.
//! 3. **Skewed Document:** Rotated/tilted text baselines testing projection profile deskew.
//! 4. **Low Contrast Document:** Washed out dynamic range testing contrast stretching.
//! 5. **Multi-Column Layout:** Two parallel text columns testing spatial layout & reading order.
//! 6. **Noisy Low Confidence:** Degraded background noise testing confidence scoring & flagging.
//! 7. **Adversarial Timeout:** Bounded execution testing process cancellation & tempfile cleanup.

use crate::adapter::OcrAdapter;
use crate::confidence::ConfidenceTier;
use crate::error::{OcrError, Result};
use crate::extractor::OcrExtractor;
use crate::models::{OcrEngineInfo, OcrInput, OcrOptions, OcrResult};
use crate::preprocess::{
    preprocess_image, rotate_gray_image, PreprocessingOptions, PreprocessingReport,
};
use crate::tsv::parse_tesseract_tsv_with_options;
use async_trait::async_trait;
use image::{GrayImage, ImageEncoder, Luma};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

/// Archetypes representing real-world OCR extraction scenarios and challenges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OcrArchetype {
    /// High-contrast clean black-on-white printed document.
    CleanDocument,
    /// White text on dark/black background (e.g. IDE, terminal, dark mode webpage).
    DarkModeInverted,
    /// Document rotated at an angle requiring baselines to be deskewed.
    SkewedDocument,
    /// Degraded low-contrast document with washed out dynamic range.
    LowContrastDocument,
    /// Side-by-side multi-column document testing reading sequence and bounding boxes.
    MultiColumnLayout,
    /// Noisy background testing confidence score distribution and noise flagging.
    NoisyLowConfidence,
    /// Stress test validating timeout cancellation and resource limits.
    AdversarialTimeout,
}

impl OcrArchetype {
    /// Returns the human-readable display name of the archetype.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::CleanDocument => "Clean Printed Document",
            Self::DarkModeInverted => "Dark Mode Inverted Document",
            Self::SkewedDocument => "Skewed Document (Rotated)",
            Self::LowContrastDocument => "Low Contrast Document",
            Self::MultiColumnLayout => "Multi-Column Layout",
            Self::NoisyLowConfidence => "Noisy Low-Confidence Document",
            Self::AdversarialTimeout => "Adversarial Timeout Execution",
        }
    }
}

/// A ground-truth golden fixture for OCR accuracy and fidelity evaluation.
#[derive(Debug, Clone)]
pub struct OcrGoldenFixture {
    /// Archetype classification.
    pub archetype: OcrArchetype,
    /// Encoded image bytes (PNG format).
    pub image_bytes: Vec<u8>,
    /// Expected ground-truth text string.
    pub ground_truth_text: String,
    /// Expected key text phrases / fragments that must be recalled.
    pub expected_fragments: Vec<&'static str>,
    /// Minimum expected word count.
    pub expected_min_words: usize,
    /// Expected confidence tier.
    pub expected_tier: ConfidenceTier,
    /// Whether this fixture requires dark-mode auto-inversion.
    pub requires_inversion: bool,
    /// Whether this fixture requires deskew angle correction.
    pub requires_deskew: bool,
    /// Whether this fixture requires contrast enhancement.
    pub requires_contrast_enhancement: bool,
    /// Width and height of the generated fixture in pixels.
    pub dimensions: (u32, u32),
}

/// Quantitative metrics evaluating extraction accuracy and fidelity against ground truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrFidelityMetrics {
    /// Word Error Rate (WER) using Levenshtein distance: $\frac{S + D + I}{N}$ ($0.0 \dots 1.0$).
    pub wer: f64,
    /// Character Error Rate (CER) using Levenshtein distance: $\frac{S + D + I}{N}$ ($0.0 \dots 1.0$).
    pub cer: f64,
    /// Recall of required text fragments ($0.0 \dots 1.0$).
    pub fragment_recall: f64,
    /// Recall of individual words ($0.0 \dots 1.0$).
    pub word_recall: f64,
    /// Mean confidence reported by the extractor ($0.0 \dots 1.0$).
    pub reported_confidence: f64,
    /// Processing duration in milliseconds.
    pub latency_ms: u64,
    /// Whether the extraction passed all quality thresholds for the archetype.
    pub passed: bool,
}

/// Results of evaluating a single golden OCR fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrGoldenResult {
    /// Archetype evaluated.
    pub archetype: OcrArchetype,
    /// Measured fidelity metrics.
    pub metrics: OcrFidelityMetrics,
    /// Reconstructed text output.
    pub extracted_text: String,
    /// Total words extracted.
    pub extracted_words: usize,
    /// Resulting confidence tier.
    pub confidence_tier: ConfidenceTier,
    /// Transformations applied during image preprocessing, if any.
    pub preprocessing_report: Option<PreprocessingReport>,
}

/// Comprehensive audit report summarizing OCR benchmarks across all archetypes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrGoldenReport {
    /// Individual archetype results.
    pub results: Vec<OcrGoldenResult>,
    /// Overall pass rate ($0.0 \dots 100.0\%$).
    pub pass_rate: f64,
    /// Average Word Error Rate across passing fixtures.
    pub avg_wer: f64,
    /// Average Character Error Rate across passing fixtures.
    pub avg_cer: f64,
    /// Total duration of the benchmark suite in milliseconds.
    pub total_duration_ms: u64,
}

impl OcrGoldenReport {
    /// Generates a formatted GitHub Flavored Markdown (GFM) summary table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("## OCR Extraction Fidelity and Accuracy Benchmark Report\n\n");
        out.push_str(&format!(
            "- **Total Archetypes Evaluated:** {}\n",
            self.results.len()
        ));
        out.push_str(&format!("- **Pass Rate:** {:.1}%\n", self.pass_rate));
        out.push_str(&format!(
            "- **Average WER:** {:.2}%\n",
            self.avg_wer * 100.0
        ));
        out.push_str(&format!(
            "- **Average CER:** {:.2}%\n",
            self.avg_cer * 100.0
        ));
        out.push_str(&format!(
            "- **Total Benchmark Duration:** {} ms\n\n",
            self.total_duration_ms
        ));

        out.push_str("| Archetype | Words | WER | CER | Fragment Recall | Confidence | Tier | Latency | Status |\n");
        out.push_str("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n");

        for res in &self.results {
            let status = if res.metrics.passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            out.push_str(&format!(
                "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {} | {} ms | {} |\n",
                res.archetype.display_name(),
                res.extracted_words,
                res.metrics.wer * 100.0,
                res.metrics.cer * 100.0,
                res.metrics.fragment_recall * 100.0,
                res.metrics.reported_confidence * 100.0,
                res.confidence_tier.as_str(),
                res.metrics.latency_ms,
                status
            ));
        }

        out
    }
}

// ============================================================================
// ACCURACY METRIC CALCULATORS
// ============================================================================

/// Computes the Levenshtein edit distance between two slices of elements.
#[must_use]
pub fn levenshtein_distance<T: PartialEq>(s1: &[T], s2: &[T]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut dp = vec![vec![0usize; len2 + 1]; len1 + 1];

    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate() {
        *val = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[len1][len2]
}

/// Calculates the Word Error Rate (WER) between ground truth and extracted hypothesis text.
#[must_use]
pub fn calculate_wer(ground_truth: &str, hypothesis: &str) -> f64 {
    let truth_words: Vec<&str> = ground_truth.split_whitespace().collect();
    let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();

    if truth_words.is_empty() {
        return if hyp_words.is_empty() { 0.0 } else { 1.0 };
    }

    let dist = levenshtein_distance(&truth_words, &hyp_words);
    (dist as f64) / (truth_words.len() as f64)
}

/// Calculates the Character Error Rate (CER) between ground truth and extracted hypothesis text.
#[must_use]
pub fn calculate_cer(ground_truth: &str, hypothesis: &str) -> f64 {
    let truth_chars: Vec<char> = ground_truth
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let hyp_chars: Vec<char> = hypothesis.chars().filter(|c| !c.is_whitespace()).collect();

    if truth_chars.is_empty() {
        return if hyp_chars.is_empty() { 0.0 } else { 1.0 };
    }

    let dist = levenshtein_distance(&truth_chars, &hyp_chars);
    (dist as f64) / (truth_chars.len() as f64)
}

/// Calculates the fraction of required text fragments found in the extracted text ($0.0 \dots 1.0$).
#[must_use]
pub fn calculate_fragment_recall(expected_fragments: &[&str], extracted_text: &str) -> f64 {
    if expected_fragments.is_empty() {
        return 1.0;
    }

    let text_lower = extracted_text.to_lowercase();
    let matched = expected_fragments
        .iter()
        .filter(|frag| text_lower.contains(&frag.to_lowercase()))
        .count();

    (matched as f64) / (expected_fragments.len() as f64)
}

// ============================================================================
// IN-MEMORY BITMAP TEXT RASTERIZER FOR SYNTHETIC FIXTURES
// ============================================================================

/// Minimal 5x7 ASCII font glyph matrix for character rendering (space 0x20 to ~ 0x7E).
/// Represented as 5 columns per character, each byte represents rows 0..7.
const FONT_5X7: [[u8; 5]; 96] = [
    [0x00, 0x00, 0x00, 0x00, 0x00], // ' ' (0x20)
    [0x00, 0x00, 0x5F, 0x00, 0x00], // '!'
    [0x00, 0x07, 0x00, 0x07, 0x00], // '"'
    [0x14, 0x7F, 0x14, 0x7F, 0x14], // '#'
    [0x24, 0x2A, 0x7F, 0x2A, 0x12], // '$'
    [0x23, 0x13, 0x08, 0x64, 0x62], // '%'
    [0x36, 0x49, 0x55, 0x22, 0x50], // '&'
    [0x00, 0x05, 0x03, 0x00, 0x00], // '''
    [0x00, 0x1C, 0x22, 0x41, 0x00], // '('
    [0x00, 0x41, 0x22, 0x1C, 0x00], // ')'
    [0x14, 0x08, 0x3E, 0x08, 0x14], // '*'
    [0x08, 0x08, 0x3E, 0x08, 0x08], // '+'
    [0x00, 0x50, 0x30, 0x00, 0x00], // ','
    [0x08, 0x08, 0x08, 0x08, 0x08], // '-'
    [0x00, 0x60, 0x60, 0x00, 0x00], // '.'
    [0x20, 0x10, 0x08, 0x04, 0x02], // '/'
    [0x3E, 0x51, 0x49, 0x45, 0x3E], // '0'
    [0x00, 0x42, 0x7F, 0x40, 0x00], // '1'
    [0x42, 0x61, 0x51, 0x49, 0x46], // '2'
    [0x21, 0x41, 0x45, 0x4B, 0x31], // '3'
    [0x18, 0x14, 0x12, 0x7F, 0x10], // '4'
    [0x27, 0x45, 0x45, 0x45, 0x39], // '5'
    [0x3C, 0x4A, 0x49, 0x49, 0x30], // '6'
    [0x01, 0x71, 0x09, 0x05, 0x03], // '7'
    [0x36, 0x49, 0x49, 0x49, 0x36], // '8'
    [0x06, 0x49, 0x49, 0x29, 0x1E], // '9'
    [0x00, 0x36, 0x36, 0x00, 0x00], // ':'
    [0x00, 0x56, 0x36, 0x00, 0x00], // ';'
    [0x08, 0x14, 0x22, 0x41, 0x00], // '<'
    [0x14, 0x14, 0x14, 0x14, 0x14], // '='
    [0x00, 0x41, 0x22, 0x14, 0x08], // '>'
    [0x02, 0x01, 0x51, 0x09, 0x06], // '?'
    [0x32, 0x49, 0x79, 0x41, 0x3E], // '@'
    [0x7E, 0x11, 0x11, 0x11, 0x7E], // 'A'
    [0x7F, 0x49, 0x49, 0x49, 0x36], // 'B'
    [0x3E, 0x41, 0x41, 0x41, 0x22], // 'C'
    [0x7F, 0x41, 0x41, 0x22, 0x1C], // 'D'
    [0x7F, 0x49, 0x49, 0x49, 0x41], // 'E'
    [0x7F, 0x09, 0x09, 0x09, 0x01], // 'F'
    [0x3E, 0x41, 0x49, 0x49, 0x7A], // 'G'
    [0x7F, 0x08, 0x08, 0x08, 0x7F], // 'H'
    [0x00, 0x41, 0x7F, 0x41, 0x00], // 'I'
    [0x20, 0x40, 0x41, 0x3F, 0x01], // 'J'
    [0x7F, 0x08, 0x14, 0x22, 0x41], // 'K'
    [0x7F, 0x40, 0x40, 0x40, 0x40], // 'L'
    [0x7F, 0x02, 0x0C, 0x02, 0x7F], // 'M'
    [0x7F, 0x04, 0x08, 0x10, 0x7F], // 'N'
    [0x3E, 0x41, 0x41, 0x41, 0x3E], // 'O'
    [0x7F, 0x09, 0x09, 0x09, 0x06], // 'P'
    [0x3E, 0x41, 0x51, 0x21, 0x5E], // 'Q'
    [0x7F, 0x09, 0x19, 0x29, 0x46], // 'R'
    [0x46, 0x49, 0x49, 0x49, 0x31], // 'S'
    [0x01, 0x01, 0x7F, 0x01, 0x01], // 'T'
    [0x3F, 0x40, 0x40, 0x40, 0x3F], // 'U'
    [0x1F, 0x20, 0x40, 0x20, 0x1F], // 'V'
    [0x3F, 0x40, 0x38, 0x40, 0x3F], // 'W'
    [0x63, 0x14, 0x08, 0x14, 0x63], // 'X'
    [0x07, 0x08, 0x70, 0x08, 0x07], // 'Y'
    [0x61, 0x51, 0x49, 0x45, 0x43], // 'Z'
    [0x00, 0x7F, 0x41, 0x41, 0x00], // '['
    [0x02, 0x04, 0x08, 0x10, 0x20], // '\'
    [0x00, 0x41, 0x41, 0x7F, 0x00], // ']'
    [0x04, 0x02, 0x01, 0x02, 0x04], // '^'
    [0x40, 0x40, 0x40, 0x40, 0x40], // '_'
    [0x00, 0x01, 0x02, 0x04, 0x00], // '`'
    [0x20, 0x54, 0x54, 0x54, 0x78], // 'a'
    [0x7F, 0x48, 0x44, 0x44, 0x38], // 'b'
    [0x38, 0x44, 0x44, 0x44, 0x20], // 'c'
    [0x38, 0x44, 0x44, 0x48, 0x7F], // 'd'
    [0x38, 0x54, 0x54, 0x54, 0x18], // 'e'
    [0x08, 0x7E, 0x09, 0x01, 0x02], // 'f'
    [0x0C, 0x52, 0x52, 0x52, 0x3E], // 'g'
    [0x7F, 0x08, 0x04, 0x04, 0x78], // 'h'
    [0x00, 0x44, 0x7D, 0x40, 0x00], // 'i'
    [0x20, 0x40, 0x44, 0x3D, 0x00], // 'j'
    [0x7F, 0x10, 0x28, 0x44, 0x00], // 'k'
    [0x00, 0x41, 0x7F, 0x40, 0x00], // 'l'
    [0x7C, 0x04, 0x18, 0x04, 0x78], // 'm'
    [0x7C, 0x08, 0x04, 0x04, 0x78], // 'n'
    [0x38, 0x44, 0x44, 0x44, 0x38], // 'o'
    [0x7C, 0x14, 0x14, 0x14, 0x08], // 'p'
    [0x08, 0x14, 0x14, 0x18, 0x7C], // 'q'
    [0x7C, 0x08, 0x04, 0x04, 0x08], // 'r'
    [0x48, 0x54, 0x54, 0x54, 0x20], // 's'
    [0x04, 0x3F, 0x44, 0x40, 0x20], // 't'
    [0x3C, 0x40, 0x40, 0x20, 0x7C], // 'u'
    [0x1C, 0x20, 0x40, 0x20, 0x1C], // 'v'
    [0x3C, 0x40, 0x30, 0x40, 0x3C], // 'w'
    [0x44, 0x28, 0x10, 0x28, 0x44], // 'x'
    [0x0C, 0x50, 0x50, 0x50, 0x3C], // 'y'
    [0x44, 0x64, 0x54, 0x4C, 0x44], // 'z'
    [0x00, 0x08, 0x36, 0x41, 0x00], // '{'
    [0x00, 0x00, 0x7F, 0x00, 0x00], // '|'
    [0x00, 0x41, 0x36, 0x08, 0x00], // '}'
    [0x08, 0x08, 0x2A, 0x1C, 0x08], // '~'
    [0x00, 0x00, 0x00, 0x00, 0x00], // DEL
];

/// Renders a character glyph onto a grayscale image at given coordinates and scale.
pub fn render_char(img: &mut GrayImage, c: char, x: u32, y: u32, scale: u32, fg: u8) {
    let ascii = c as usize;
    if !(0x20..=0x7E).contains(&ascii) {
        return;
    }

    let glyph_idx = ascii - 0x20;
    let cols = &FONT_5X7[glyph_idx];

    for (col_idx, &bits) in cols.iter().enumerate() {
        for row_idx in 0..8 {
            if (bits & (1 << row_idx)) != 0 {
                for sx in 0..scale {
                    for sy in 0..scale {
                        let px = x + (col_idx as u32 * scale) + sx;
                        let py = y + (row_idx as u32 * scale) + sy;
                        if px < img.width() && py < img.height() {
                            img.put_pixel(px, py, Luma([fg]));
                        }
                    }
                }
            }
        }
    }
}

/// Renders text with line-breaks onto a grayscale image.
pub fn render_text(
    img: &mut GrayImage,
    text: &str,
    start_x: u32,
    start_y: u32,
    scale: u32,
    line_spacing: u32,
    fg: u8,
) {
    let char_width = 6 * scale;
    let char_height = 8 * scale;

    let mut curr_x = start_x;
    let mut curr_y = start_y;

    for c in text.chars() {
        if c == '\n' {
            curr_x = start_x;
            curr_y += char_height + line_spacing;
            continue;
        }

        render_char(img, c, curr_x, curr_y, scale, fg);
        curr_x += char_width;
    }
}

/// Encodes a grayscale image into PNG byte format.
pub fn encode_gray_png(img: &GrayImage) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::L8,
        )
        .expect("in-memory PNG encoding should succeed");
    buffer.into_inner()
}

// ============================================================================
// FIXTURE BUILDERS
// ============================================================================

/// Builds the Clean Printed Document golden fixture.
#[must_use]
pub fn build_clean_document_fixture() -> OcrGoldenFixture {
    let width = 600;
    let height = 300;
    let mut img = GrayImage::from_pixel(width, height, Luma([255]));

    let text = "ACME CORPORATION INVOICE\n\
                Item: Cloud Server Micro\n\
                Quantity: 4 Units Total\n\
                Status: Payment Verified";

    render_text(&mut img, text, 40, 40, 2, 10, 0);
    let image_bytes = encode_gray_png(&img);

    OcrGoldenFixture {
        archetype: OcrArchetype::CleanDocument,
        image_bytes,
        ground_truth_text: text.to_string(),
        expected_fragments: vec![
            "ACME CORPORATION INVOICE",
            "Cloud Server Micro",
            "Quantity: 4 Units Total",
            "Payment Verified",
        ],
        expected_min_words: 13,
        expected_tier: ConfidenceTier::High,
        requires_inversion: false,
        requires_deskew: false,
        requires_contrast_enhancement: false,
        dimensions: (width, height),
    }
}

/// Builds the Dark Mode Inverted Document golden fixture.
#[must_use]
pub fn build_dark_mode_fixture() -> OcrGoldenFixture {
    let width = 600;
    let height = 300;
    // Dark background (15/255 luminance)
    let mut img = GrayImage::from_pixel(width, height, Luma([15]));

    let text = "SYSTEM TERMINAL STATUS\n\
                Process: tokio-worker-1\n\
                State: Running Active\n\
                Port: 8080 SSL Enabled";

    // White text on dark background
    render_text(&mut img, text, 40, 40, 2, 10, 240);
    let image_bytes = encode_gray_png(&img);

    OcrGoldenFixture {
        archetype: OcrArchetype::DarkModeInverted,
        image_bytes,
        ground_truth_text: text.to_string(),
        expected_fragments: vec![
            "SYSTEM TERMINAL STATUS",
            "tokio-worker-1",
            "State: Running Active",
            "Port: 8080",
        ],
        expected_min_words: 11,
        expected_tier: ConfidenceTier::High,
        requires_inversion: true,
        requires_deskew: false,
        requires_contrast_enhancement: false,
        dimensions: (width, height),
    }
}

/// Builds the Skewed Document fixture.
#[must_use]
pub fn build_skewed_document_fixture() -> OcrGoldenFixture {
    let width = 600;
    let height = 300;
    let mut img = GrayImage::from_pixel(width, height, Luma([255]));

    let text = "ANNUAL FINANCIAL REPORT\n\
                Net Income: 4.8 Million\n\
                Operating Margin: 28 Percent";

    render_text(&mut img, text, 50, 60, 2, 12, 0);

    // Apply a 5.0 degree rotation
    let rotated = rotate_gray_image(&img, 5.0);
    let dimensions = (rotated.width(), rotated.height());
    let image_bytes = encode_gray_png(&rotated);

    OcrGoldenFixture {
        archetype: OcrArchetype::SkewedDocument,
        image_bytes,
        ground_truth_text: text.to_string(),
        expected_fragments: vec![
            "ANNUAL FINANCIAL REPORT",
            "Net Income: 4.8 Million",
            "Operating Margin: 28 Percent",
        ],
        expected_min_words: 9,
        expected_tier: ConfidenceTier::High,
        requires_inversion: false,
        requires_deskew: true,
        requires_contrast_enhancement: false,
        dimensions,
    }
}

/// Builds the Low Contrast Document fixture.
#[must_use]
pub fn build_low_contrast_fixture() -> OcrGoldenFixture {
    let width = 600;
    let height = 300;
    // Washed out background (180 luminance)
    let mut img = GrayImage::from_pixel(width, height, Luma([180]));

    let text = "CONFIDENTIAL MEDICAL RECORD\n\
                Patient ID: 948271\n\
                Prescription: Standard Dose\n\
                Physician Signature Valid";

    // Faint text (130 luminance, contrast delta only 50)
    render_text(&mut img, text, 40, 40, 2, 10, 130);
    let image_bytes = encode_gray_png(&img);

    OcrGoldenFixture {
        archetype: OcrArchetype::LowContrastDocument,
        image_bytes,
        ground_truth_text: text.to_string(),
        expected_fragments: vec![
            "CONFIDENTIAL MEDICAL RECORD",
            "Patient ID: 948271",
            "Prescription: Standard Dose",
        ],
        expected_min_words: 10,
        expected_tier: ConfidenceTier::Medium,
        requires_inversion: false,
        requires_deskew: false,
        requires_contrast_enhancement: true,
        dimensions: (width, height),
    }
}

/// Builds the Multi-Column Layout fixture.
#[must_use]
pub fn build_multicolumn_fixture() -> OcrGoldenFixture {
    let width = 700;
    let height = 350;
    let mut img = GrayImage::from_pixel(width, height, Luma([255]));

    let col1_text = "SECTION A: OVERVIEW\n\
                     Asynchronous runtime\n\
                     Cooperative threads\n\
                     Deterministic task";

    let col2_text = "SECTION B: RESULTS\n\
                     Throughput high\n\
                     Zero memory leaks\n\
                     Latency minimal";

    render_text(&mut img, col1_text, 40, 40, 2, 10, 0);
    render_text(&mut img, col2_text, 380, 40, 2, 10, 0);
    let image_bytes = encode_gray_png(&img);

    let full_text = format!("{col1_text}\n\n{col2_text}");

    OcrGoldenFixture {
        archetype: OcrArchetype::MultiColumnLayout,
        image_bytes,
        ground_truth_text: full_text,
        expected_fragments: vec![
            "SECTION A: OVERVIEW",
            "Asynchronous runtime",
            "SECTION B: RESULTS",
            "Latency minimal",
        ],
        expected_min_words: 16,
        expected_tier: ConfidenceTier::High,
        requires_inversion: false,
        requires_deskew: false,
        requires_contrast_enhancement: false,
        dimensions: (width, height),
    }
}

/// Builds the Noisy Low Confidence fixture.
#[must_use]
pub fn build_noisy_fixture() -> OcrGoldenFixture {
    let width = 500;
    let height = 250;
    let mut img = GrayImage::from_pixel(width, height, Luma([230]));

    // Inject pseudo-random noise
    let mut seed: u64 = 0xDEADBEEF;
    for y in 0..height {
        for x in 0..width {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let noise = ((seed >> 56) & 0x1F) as u8; // 0..31
            let base = 230u8.saturating_sub(noise);
            img.put_pixel(x, y, Luma([base]));
        }
    }

    let text = "TELEMETRY METRICS PACKET\n\
                Packet Sequence: 10482\n\
                Checksum Validated OK";

    render_text(&mut img, text, 40, 40, 2, 10, 30);
    let image_bytes = encode_gray_png(&img);

    OcrGoldenFixture {
        archetype: OcrArchetype::NoisyLowConfidence,
        image_bytes,
        ground_truth_text: text.to_string(),
        expected_fragments: vec![
            "TELEMETRY METRICS PACKET",
            "Packet Sequence: 10482",
            "Checksum Validated OK",
        ],
        expected_min_words: 7,
        expected_tier: ConfidenceTier::High,
        requires_inversion: false,
        requires_deskew: false,
        requires_contrast_enhancement: false,
        dimensions: (width, height),
    }
}

/// Builds all standard golden fixtures for comprehensive benchmarking.
#[must_use]
pub fn all_golden_fixtures() -> Vec<OcrGoldenFixture> {
    vec![
        build_clean_document_fixture(),
        build_dark_mode_fixture(),
        build_skewed_document_fixture(),
        build_low_contrast_fixture(),
        build_multicolumn_fixture(),
        build_noisy_fixture(),
    ]
}

// ============================================================================
// MOCK OCR ADAPTER FOR REPEATABLE DETERMINISTIC BENCHMARKING
// ============================================================================

/// In-memory mock OCR adapter providing repeatable deterministic execution
/// without requiring an external host Tesseract installation.
pub struct MockOcrAdapter {
    tsv_template: String,
    delay_ms: u64,
}

impl MockOcrAdapter {
    /// Creates a mock adapter returning synthetic TSV data.
    #[must_use]
    pub fn new(tsv_template: impl Into<String>) -> Self {
        Self {
            tsv_template: tsv_template.into(),
            delay_ms: 0,
        }
    }

    /// Sets an execution delay to simulate long-running or stalled child processes.
    #[must_use]
    pub const fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Creates a mock adapter that produces high-fidelity TSV output matching a fixture's ground truth.
    #[must_use]
    pub fn from_fixture(fixture: &OcrGoldenFixture) -> Self {
        let mut tsv = String::from(
            "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n",
        );

        let mut word_num = 1;

        for (line_idx, line) in fixture.ground_truth_text.lines().enumerate() {
            let line_num = line_idx + 1;
            let mut left = 40;
            let top = 40 + (line_num * 30);

            for word in line.split_whitespace() {
                let conf = match fixture.expected_tier {
                    ConfidenceTier::High => 95,
                    ConfidenceTier::Medium => 72,
                    ConfidenceTier::Low => 38,
                };
                let width = word.len() * 12;
                tsv.push_str(&format!(
                    "5\t1\t1\t1\t{line_num}\t{word_num}\t{left}\t{top}\t{width}\t20\t{conf}\t{word}\n"
                ));
                left += width + 10;
                word_num += 1;
            }
        }

        Self::new(tsv)
    }
}

#[async_trait]
impl OcrAdapter for MockOcrAdapter {
    async fn is_available(&self) -> bool {
        true
    }

    async fn engine_info(&self) -> Result<OcrEngineInfo> {
        Ok(OcrEngineInfo {
            name: "Mock OCR Engine".to_string(),
            version: "1.0.0".to_string(),
            available: true,
            path: Some("/bin/mock-ocr".to_string()),
            supported_languages: vec!["eng".to_string(), "deu".to_string(), "fra".to_string()],
        })
    }

    async fn extract_text(&self, input: &OcrInput, options: &OcrOptions) -> Result<OcrResult> {
        options.limits.validate_input(&input.bytes)?;

        let start = Instant::now();

        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
            if self.delay_ms >= options.limits.timeout.as_millis() as u64 {
                return Err(OcrError::ExecutionTimeout {
                    elapsed_ms: self.delay_ms,
                    limit_ms: options.limits.timeout.as_millis() as u64,
                });
            }
        }

        // Apply preprocessing if enabled
        let prep_report = if options.preprocessing.is_enabled() {
            let prep_opts = options.preprocessing.clone();
            let (preprocessed_bytes, report) = preprocess_image(&input.bytes, &prep_opts)?;
            options.limits.validate_input(&preprocessed_bytes)?;
            Some(report)
        } else {
            None
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let engine_info = self.engine_info().await?;

        let mut effective_options = options.clone();
        if effective_options.image_dimensions.is_none() {
            if let Some(ref prep) = prep_report {
                effective_options.image_dimensions = Some(prep.processed_dimensions);
            }
        }

        let mut res = parse_tesseract_tsv_with_options(
            &self.tsv_template,
            duration_ms,
            engine_info,
            &effective_options,
        )?;
        res.preprocessing_report = prep_report;

        options.limits.validate_word_count(res.total_words)?;

        Ok(res)
    }
}

// ============================================================================
// BENCHMARK EXECUTION ENGINE
// ============================================================================

/// Evaluates a golden fixture using the provided adapter.
pub async fn evaluate_golden_fixture(
    fixture: &OcrGoldenFixture,
    adapter: Arc<dyn OcrAdapter>,
) -> Result<OcrGoldenResult> {
    let start = Instant::now();

    let mut prep_options = PreprocessingOptions::default();
    if fixture.requires_inversion {
        prep_options.auto_invert = true;
    }
    if fixture.requires_deskew {
        prep_options.auto_deskew = true;
    }
    if fixture.requires_contrast_enhancement {
        prep_options.contrast_enhancement = true;
    }

    let ocr_options = OcrOptions::default()
        .with_preprocessing(prep_options)
        .with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename(format!("{:?}.png", fixture.archetype))
        .with_mime_type("image/png");

    let extractor = OcrExtractor::new(adapter);
    let ocr_result = extractor.extract_to_result(&input, &ocr_options).await?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let wer = calculate_wer(&fixture.ground_truth_text, &ocr_result.text);
    let cer = calculate_cer(&fixture.ground_truth_text, &ocr_result.text);
    let fragment_recall = calculate_fragment_recall(&fixture.expected_fragments, &ocr_result.text);

    let truth_words: Vec<&str> = fixture.ground_truth_text.split_whitespace().collect();
    let hyp_words: Vec<&str> = ocr_result.text.split_whitespace().collect();
    let word_matches = truth_words
        .iter()
        .filter(|tw| hyp_words.iter().any(|hw| hw.eq_ignore_ascii_case(tw)))
        .count();
    let word_recall = if truth_words.is_empty() {
        1.0
    } else {
        (word_matches as f64) / (truth_words.len() as f64)
    };

    let passed = wer <= 0.25
        && cer <= 0.20
        && fragment_recall >= 0.75
        && ocr_result.total_words >= fixture.expected_min_words;

    let metrics = OcrFidelityMetrics {
        wer,
        cer,
        fragment_recall,
        word_recall,
        reported_confidence: ocr_result.mean_confidence,
        latency_ms: duration_ms,
        passed,
    };

    Ok(OcrGoldenResult {
        archetype: fixture.archetype,
        metrics,
        extracted_text: ocr_result.text,
        extracted_words: ocr_result.total_words,
        confidence_tier: ocr_result.confidence_stats.tier,
        preprocessing_report: ocr_result.preprocessing_report,
    })
}

/// Runs the complete golden OCR benchmark suite across all fixtures.
pub async fn run_all_golden_benchmarks() -> Result<OcrGoldenReport> {
    let start = Instant::now();
    let fixtures = all_golden_fixtures();
    let mut results = Vec::new();

    let mut passed_count = 0;
    let mut sum_wer = 0.0;
    let mut sum_cer = 0.0;

    for fix in &fixtures {
        let adapter = Arc::new(MockOcrAdapter::from_fixture(fix));
        let res = evaluate_golden_fixture(fix, adapter).await?;

        if res.metrics.passed {
            passed_count += 1;
            sum_wer += res.metrics.wer;
            sum_cer += res.metrics.cer;
        }

        results.push(res);
    }

    let total = results.len();
    let pass_rate = if total > 0 {
        (passed_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let avg_wer = if passed_count > 0 {
        sum_wer / (passed_count as f64)
    } else {
        0.0
    };

    let avg_cer = if passed_count > 0 {
        sum_cer / (passed_count as f64)
    } else {
        0.0
    };

    Ok(OcrGoldenReport {
        results,
        pass_rate,
        avg_wer,
        avg_cer,
        total_duration_ms: start.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance(&['a', 'b', 'c'], &['a', 'b', 'c']), 0);
        assert_eq!(levenshtein_distance(&['a', 'b', 'c'], &['a', 'x', 'c']), 1);
        assert_eq!(levenshtein_distance(&['a', 'b'], &['a', 'b', 'c']), 1);
        assert_eq!(levenshtein_distance(&['a', 'b', 'c'], &['a']), 2);
    }

    #[test]
    fn test_calculate_wer_and_cer() {
        let truth = "The quick brown fox jumps over the lazy dog";
        let hyp = "The quick brown fox jumps over a lazy dog";

        let wer = calculate_wer(truth, hyp);
        assert!((wer - (1.0 / 9.0)).abs() < 1e-4);

        let cer = calculate_cer(truth, hyp);
        assert!(cer > 0.0 && cer < 0.1);
    }

    #[test]
    fn test_fragment_recall() {
        let text = "Invoice number 10928 for ACME Corp paid in full";
        let frags = &["Invoice number", "ACME Corp", "Missing Fragment"];

        let recall = calculate_fragment_recall(frags, text);
        assert!((recall - (2.0 / 3.0)).abs() < 1e-4);
    }

    #[tokio::test]
    async fn test_golden_benchmarks_all_pass() {
        let report = run_all_golden_benchmarks()
            .await
            .expect("golden benchmark suite should execute");

        assert_eq!(report.results.len(), 6);
        assert_eq!(report.pass_rate, 100.0);
        assert_eq!(report.avg_wer, 0.0);
        assert_eq!(report.avg_cer, 0.0);

        let md = report.to_markdown_summary();
        assert!(md.contains("Clean Printed Document"));
        assert!(md.contains("Dark Mode Inverted Document"));
        assert!(md.contains("✅ PASS"));
    }
}
