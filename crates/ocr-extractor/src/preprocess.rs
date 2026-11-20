//! # OCR Image Preprocessing Pipeline
//!
//! Provides contrast enhancement, grayscale normalization, dark-mode auto-inversion,
//! Otsu's adaptive binarization, deskew angle detection, and dimension scaling.

use crate::error::{OcrError, Result};
use image::{imageops::FilterType, DynamicImage, GrayImage, Luma};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::debug;

/// Binarization thresholding algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BinarizationMethod {
    /// Otsu's global variance-maximizing thresholding (optimal for uniform lighting).
    #[default]
    Otsu,
    /// Fixed threshold cutoff value (0–255).
    Fixed(u8),
    /// Local adaptive mean thresholding.
    AdaptiveMean,
}

/// Configuration options for image preprocessing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessingOptions {
    /// Whether to convert input image to 8-bit grayscale.
    pub grayscale: bool,
    /// Whether to detect and invert dark background / light text images.
    pub auto_invert: bool,
    /// Whether to enhance contrast using histogram dynamic range stretching.
    pub contrast_enhancement: bool,
    /// Whether to binarize grayscale image into pure black/white text.
    pub binarize: bool,
    /// Selected binarization algorithm.
    pub binarization_method: BinarizationMethod,
    /// Whether to automatically detect text baseline skew and rotate.
    pub auto_deskew: bool,
    /// Maximum skew angle in degrees to test and correct (e.g. 15.0 degrees).
    pub max_deskew_angle_degrees: f64,
    /// Angle search resolution in degrees (e.g. 0.5 degrees).
    pub deskew_step_degrees: f64,
    /// Maximum image dimension in pixels; larger images are scaled down.
    pub max_dimension: u32,
    /// Minimum image dimension in pixels; smaller images are scaled up.
    pub min_dimension: u32,
}

impl Default for PreprocessingOptions {
    fn default() -> Self {
        Self {
            grayscale: true,
            auto_invert: true,
            contrast_enhancement: true,
            binarize: true,
            binarization_method: BinarizationMethod::Otsu,
            auto_deskew: true,
            max_deskew_angle_degrees: 15.0,
            deskew_step_degrees: 0.5,
            max_dimension: 8192,
            min_dimension: 0,
        }
    }
}

impl PreprocessingOptions {
    /// Returns whether any preprocessing transformations are enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.grayscale
            || self.auto_invert
            || self.contrast_enhancement
            || self.binarize
            || self.auto_deskew
    }

    /// Creates an options instance with all preprocessing disabled (raw passthrough).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            grayscale: false,
            auto_invert: false,
            contrast_enhancement: false,
            binarize: false,
            binarization_method: BinarizationMethod::Fixed(128),
            auto_deskew: false,
            max_deskew_angle_degrees: 0.0,
            deskew_step_degrees: 1.0,
            max_dimension: 8192,
            min_dimension: 0,
        }
    }
}

/// Telemetry and audit report detailing transformations applied during preprocessing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessingReport {
    /// Width and height of the original input image in pixels.
    pub original_dimensions: (u32, u32),
    /// Width and height of the resulting image in pixels.
    pub processed_dimensions: (u32, u32),
    /// Whether grayscale conversion was executed.
    pub grayscale_applied: bool,
    /// Whether dark-mode inversion was executed.
    pub inverted_applied: bool,
    /// Whether contrast enhancement was executed.
    pub contrast_enhanced: bool,
    /// Whether binarization was executed.
    pub binarization_applied: bool,
    /// Estimated deskew angle corrected in degrees (0.0 if not deskewed).
    pub deskew_angle_degrees: f64,
    /// Duration of preprocessing execution in milliseconds.
    pub duration_ms: u64,
}

/// Preprocesses raw input image bytes, returning preprocessed PNG bytes and an audit report.
pub fn preprocess_image(
    bytes: &[u8],
    options: &PreprocessingOptions,
) -> Result<(Vec<u8>, PreprocessingReport)> {
    let start = Instant::now();

    if !options.is_enabled() {
        return Ok((
            bytes.to_vec(),
            PreprocessingReport {
                original_dimensions: (0, 0),
                processed_dimensions: (0, 0),
                grayscale_applied: false,
                inverted_applied: false,
                contrast_enhanced: false,
                binarization_applied: false,
                deskew_angle_degrees: 0.0,
                duration_ms: 0,
            },
        ));
    }

    let dyn_img = image::load_from_memory(bytes)
        .map_err(|e| OcrError::ImageFormat(format!("Failed to decode image: {e}")))?;

    let orig_w = dyn_img.width();
    let orig_h = dyn_img.height();

    // Scale image if it exceeds max dimension or is below min dimension
    let mut current_img = dyn_img;
    let max_dim = orig_w.max(orig_h);
    let min_dim = orig_w.min(orig_h);

    if max_dim > options.max_dimension && options.max_dimension > 0 {
        let ratio = options.max_dimension as f64 / max_dim as f64;
        let new_w = (orig_w as f64 * ratio).round().max(1.0) as u32;
        let new_h = (orig_h as f64 * ratio).round().max(1.0) as u32;
        current_img = current_img.resize_exact(new_w, new_h, FilterType::Triangle);
    } else if min_dim < options.min_dimension
        && options.min_dimension > 0
        && max_dim * 2 <= options.max_dimension
    {
        let ratio = options.min_dimension as f64 / min_dim as f64;
        let new_w = (orig_w as f64 * ratio).round() as u32;
        let new_h = (orig_h as f64 * ratio).round() as u32;
        current_img = current_img.resize_exact(new_w, new_h, FilterType::Lanczos3);
    }

    let mut gray: GrayImage = current_img.to_luma8();
    let grayscale_applied = options.grayscale;

    // 1. Dark-mode auto-inversion
    let mut inverted_applied = false;
    if options.auto_invert {
        let total_pixels = (gray.width() as u64) * (gray.height() as u64);
        let sum_lum: u64 = gray.pixels().map(|p| p[0] as u64).sum();
        if let Some(mean_lum) = sum_lum.checked_div(total_pixels) {
            // If average pixel luminance is below 115, image is predominantly dark (white on black)
            if mean_lum < 115 {
                for p in gray.pixels_mut() {
                    p[0] = 255 - p[0];
                }
                inverted_applied = true;
                debug!(mean_lum, "Auto-inverted dark-mode image for OCR");
            }
        }
    }

    // 2. Contrast enhancement
    let mut contrast_enhanced = false;
    if options.contrast_enhancement {
        enhance_contrast(&mut gray);
        contrast_enhanced = true;
    }

    // 3. Deskew detection & rotation
    let mut deskew_angle = 0.0;
    if options.auto_deskew && options.max_deskew_angle_degrees > 0.0 {
        let detected = detect_skew_angle(
            &gray,
            options.max_deskew_angle_degrees,
            options.deskew_step_degrees,
        );
        if detected.abs() >= 0.25 {
            gray = rotate_gray_image(&gray, -detected);
            deskew_angle = detected;
            debug!(detected, "Deskewed image for OCR alignment");
        }
    }

    // 4. Binarization
    let mut binarization_applied = false;
    if options.binarize {
        match options.binarization_method {
            BinarizationMethod::Otsu => {
                let threshold = compute_otsu_threshold(&gray);
                apply_threshold(&mut gray, threshold);
                binarization_applied = true;
            }
            BinarizationMethod::Fixed(val) => {
                apply_threshold(&mut gray, val);
                binarization_applied = true;
            }
            BinarizationMethod::AdaptiveMean => {
                apply_adaptive_threshold(&mut gray);
                binarization_applied = true;
            }
        }
    }

    let processed_dims = (gray.width(), gray.height());

    // Encode preprocessed image to PNG bytes
    let mut out_bytes = Vec::new();
    let dyn_out = DynamicImage::ImageLuma8(gray);
    dyn_out
        .write_to(
            &mut std::io::Cursor::new(&mut out_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| OcrError::PreprocessingError(format!("Failed to encode PNG: {e}")))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok((
        out_bytes,
        PreprocessingReport {
            original_dimensions: (orig_w, orig_h),
            processed_dimensions: processed_dims,
            grayscale_applied,
            inverted_applied,
            contrast_enhanced,
            binarization_applied,
            deskew_angle_degrees: deskew_angle,
            duration_ms,
        },
    ))
}

/// Computes Otsu's optimal global binarization threshold.
#[must_use]
pub fn compute_otsu_threshold(img: &GrayImage) -> u8 {
    let mut hist = [0u64; 256];
    for p in img.pixels() {
        hist[p[0] as usize] += 1;
    }

    let total: u64 = hist.iter().sum();
    if total == 0 {
        return 128;
    }

    let mut sum: f64 = 0.0;
    for (i, &count) in hist.iter().enumerate() {
        sum += (i as f64) * (count as f64);
    }

    let mut sum_b = 0.0;
    let mut w_b: u64 = 0;
    let mut max_variance = 0.0;
    let mut t_min = 128usize;
    let mut t_max = 128usize;

    for (t, &count) in hist.iter().enumerate() {
        w_b += count;
        if w_b == 0 {
            continue;
        }

        let w_f = total - w_b;
        if w_f == 0 {
            break;
        }

        sum_b += (t as f64) * (count as f64);

        let m_b = sum_b / (w_b as f64);
        let m_f = (sum - sum_b) / (w_f as f64);

        let variance = (w_b as f64) * (w_f as f64) * (m_b - m_f) * (m_b - m_f);

        if variance > max_variance + 1e-5 {
            max_variance = variance;
            t_min = t;
            t_max = t;
        } else if (variance - max_variance).abs() <= 1e-5 && max_variance > 0.0 {
            t_max = t;
        }
    }

    ((t_min + t_max) / 2) as u8
}

/// Applies a fixed threshold cutoff to a grayscale image.
pub fn apply_threshold(img: &mut GrayImage, threshold: u8) {
    for p in img.pixels_mut() {
        p[0] = if p[0] <= threshold { 0 } else { 255 };
    }
}

/// Enhances image contrast by linear stretching between 1st and 99th percentiles.
pub fn enhance_contrast(img: &mut GrayImage) {
    let mut hist = [0u64; 256];
    for p in img.pixels() {
        hist[p[0] as usize] += 1;
    }

    let total: u64 = hist.iter().sum();
    if total == 0 {
        return;
    }

    let p1_target = (total as f64 * 0.01).round() as u64;
    let p99_target = (total as f64 * 0.99).round() as u64;

    let mut count = 0u64;
    let mut min_val = 0u8;
    for (val, &c) in hist.iter().enumerate() {
        count += c;
        if count >= p1_target {
            min_val = val as u8;
            break;
        }
    }

    let mut max_val = 255u8;
    count = 0;
    for (val, &c) in hist.iter().enumerate() {
        count += c;
        if count >= p99_target {
            max_val = val as u8;
            break;
        }
    }

    if max_val <= min_val {
        return;
    }

    let scale = 255.0 / (max_val as f64 - min_val as f64);
    for p in img.pixels_mut() {
        let v = p[0];
        let stretched = if v <= min_val {
            0
        } else if v >= max_val {
            255
        } else {
            ((v as f64 - min_val as f64) * scale).round() as u8
        };
        p[0] = stretched;
    }
}

/// Applies local adaptive mean thresholding.
pub fn apply_adaptive_threshold(img: &mut GrayImage) {
    let w = img.width() as usize;
    let h = img.height() as usize;
    let radius = 15;
    let c = 7; // subtracted constant

    let copy = img.clone();

    for y in 0..h {
        let y_min = y.saturating_sub(radius);
        let y_max = (y + radius).min(h - 1);
        for x in 0..w {
            let x_min = x.saturating_sub(radius);
            let x_max = (x + radius).min(w - 1);

            let mut sum: u32 = 0;
            let mut count: u32 = 0;
            for iy in y_min..=y_max {
                for ix in x_min..=x_max {
                    sum += copy.get_pixel(ix as u32, iy as u32)[0] as u32;
                    count += 1;
                }
            }

            let mean = (sum / count) as i32;
            let thresh = (mean - c).clamp(0, 255) as u8;

            let orig = copy.get_pixel(x as u32, y as u32)[0];
            img.put_pixel(
                x as u32,
                y as u32,
                Luma([if orig <= thresh { 0 } else { 255 }]),
            );
        }
    }
}

/// Detects skew angle by finding the projection angle that maximizes horizontal row variance.
#[must_use]
pub fn detect_skew_angle(img: &GrayImage, max_angle: f64, step: f64) -> f64 {
    let w = img.width();
    let h = img.height();

    // Fast path: downsample for projection profile calculation if image is large
    let (scaled_img, sw, sh) = if w > 500 {
        let factor = 500.0 / w as f64;
        let new_w = 500;
        let new_h = (h as f64 * factor).round().max(10.0) as u32;
        let resized = image::imageops::resize(img, new_w, new_h, FilterType::Nearest);
        (resized, new_w, new_h)
    } else {
        (img.clone(), w, h)
    };

    let mut best_angle = 0.0;
    let mut max_variance = 0.0;

    let steps = ((max_angle * 2.0) / step).round() as i32;
    for i in 0..=steps {
        let angle = -max_angle + (i as f64 * step);
        let rad = angle.to_radians();
        let tan_a = rad.tan();

        // Compute row projection profile at this angle
        let mut row_sums = vec![0u64; sh as usize];

        for y in 0..sh {
            let mut sum = 0u64;
            for x in 0..sw {
                let offset_y = y as f64 + (x as f64 - sw as f64 * 0.5) * tan_a;
                let target_y = offset_y.round() as i32;
                if target_y >= 0 && target_y < sh as i32 {
                    let val = scaled_img.get_pixel(x, target_y as u32)[0];
                    // Invert: dark pixels (ink) contribute higher weight
                    sum += (255 - val) as u64;
                }
            }
            row_sums[y as usize] = sum;
        }

        let mean: f64 = row_sums.iter().sum::<u64>() as f64 / (sh as f64);
        let variance: f64 = row_sums
            .iter()
            .map(|&s| {
                let diff = s as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (sh as f64);

        if variance > max_variance {
            max_variance = variance;
            best_angle = angle;
        }
    }

    best_angle
}

/// Rotates a grayscale image around its center by a specified angle in degrees using bilinear interpolation.
#[must_use]
pub fn rotate_gray_image(img: &GrayImage, angle_degrees: f64) -> GrayImage {
    let w = img.width();
    let h = img.height();
    let rad = -angle_degrees.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();

    let cx = w as f64 * 0.5;
    let cy = h as f64 * 0.5;

    let mut out = GrayImage::from_pixel(w, h, Luma([255])); // Fill background with white

    for y in 0..h {
        let dy = y as f64 - cy;
        for x in 0..w {
            let dx = x as f64 - cx;
            let src_x = cx + dx * cos_a - dy * sin_a;
            let src_y = cy + dx * sin_a + dy * cos_a;

            if src_x >= 0.0 && src_x < (w - 1) as f64 && src_y >= 0.0 && src_y < (h - 1) as f64 {
                let x0 = src_x.floor() as u32;
                let y0 = src_y.floor() as u32;
                let x1 = x0 + 1;
                let y1 = y0 + 1;

                let fx = src_x - x0 as f64;
                let fy = src_y - y0 as f64;

                let p00 = img.get_pixel(x0, y0)[0] as f64;
                let p10 = img.get_pixel(x1, y0)[0] as f64;
                let p01 = img.get_pixel(x0, y1)[0] as f64;
                let p11 = img.get_pixel(x1, y1)[0] as f64;

                let interpolated = (p00 * (1.0 - fx) * (1.0 - fy)
                    + p10 * fx * (1.0 - fy)
                    + p01 * (1.0 - fx) * fy
                    + p11 * fx * fy)
                    .round() as u8;

                out.put_pixel(x, y, Luma([interpolated]));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_otsu_threshold() {
        // Bimodal distribution: 50 pixels with 20, 50 pixels with 220
        let mut img = GrayImage::new(10, 10);
        for (i, p) in img.pixels_mut().enumerate() {
            p[0] = if i < 50 { 20 } else { 220 };
        }

        let thresh = compute_otsu_threshold(&img);
        assert!((20..=220).contains(&thresh));
        // Otsu threshold should cleanly sit between the two clusters
        assert!((100..=140).contains(&thresh));
    }

    #[test]
    fn test_auto_inversion_dark_background() {
        // Image with dark background (luminance 10) and bright text (luminance 240)
        let mut img = GrayImage::from_pixel(100, 100, Luma([10]));
        // Add some bright text lines
        for x in 10..90 {
            img.put_pixel(x, 50, Luma([240]));
        }

        let mut png_bytes = Vec::new();
        DynamicImage::ImageLuma8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        let opts = PreprocessingOptions {
            auto_invert: true,
            binarize: false,
            contrast_enhancement: false,
            auto_deskew: false,
            ..Default::default()
        };

        let (processed, report) =
            preprocess_image(&png_bytes, &opts).expect("Preprocessing succeeds");
        assert!(report.inverted_applied);

        let decoded = image::load_from_memory(&processed).unwrap().to_luma8();
        // Background should now be light (~245)
        assert!(decoded.get_pixel(0, 0)[0] > 200);
    }

    #[test]
    fn test_contrast_enhancement() {
        // Low contrast washed out image (values between 100 and 150)
        let mut img = GrayImage::new(20, 20);
        for (i, p) in img.pixels_mut().enumerate() {
            p[0] = 100 + ((i % 51) as u8);
        }

        enhance_contrast(&mut img);
        let min_val = img.pixels().map(|p| p[0]).min().unwrap();
        let max_val = img.pixels().map(|p| p[0]).max().unwrap();

        // Stretched dynamic range
        assert!(min_val < 20);
        assert!(max_val > 230);
    }
}
