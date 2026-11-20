//! # OCR Image Preprocessing Integration Tests
//!
//! Validates grayscale conversion, dark-mode auto-inversion, contrast enhancement,
//! Otsu's binarization, skew angle detection, and dimension scaling.

use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage};
use web_repo_to_ai_ocr_extractor::{
    apply_adaptive_threshold, compute_otsu_threshold, detect_skew_angle, enhance_contrast,
    preprocess_image, rotate_gray_image, BinarizationMethod, PreprocessingOptions,
};

fn encode_rgb_png(img: RgbImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("PNG encoding succeeds");
    bytes
}

fn encode_gray_png(img: GrayImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    DynamicImage::ImageLuma8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("PNG encoding succeeds");
    bytes
}

#[test]
fn test_otsu_threshold_calculation() {
    let mut img = GrayImage::new(100, 100);
    for (i, p) in img.pixels_mut().enumerate() {
        p[0] = if i < 5000 { 30 } else { 210 };
    }
    let threshold = compute_otsu_threshold(&img);
    assert!((100..=140).contains(&threshold));
}

#[test]
fn test_grayscale_and_binarization_pipeline() {
    let mut rgb = RgbImage::new(120, 60);
    for y in 0..60 {
        for x in 0..120 {
            // Background is pale yellow (high luminance)
            let color = if (20..40).contains(&y) && (10..110).contains(&x) {
                // Horizontal blue bar (dark text line)
                Rgb([20, 40, 180])
            } else {
                Rgb([250, 248, 230])
            };
            rgb.put_pixel(x, y, color);
        }
    }

    let input_bytes = encode_rgb_png(rgb);
    let options = PreprocessingOptions {
        grayscale: true,
        auto_invert: false,
        contrast_enhancement: true,
        binarize: true,
        binarization_method: BinarizationMethod::Otsu,
        auto_deskew: false,
        ..Default::default()
    };

    let (out_bytes, report) =
        preprocess_image(&input_bytes, &options).expect("Preprocessing succeeds");

    assert!(report.grayscale_applied);
    assert!(report.binarization_applied);
    assert!(!report.inverted_applied);

    let decoded = image::load_from_memory(&out_bytes)
        .expect("Decode output")
        .to_luma8();

    // Verify output is purely binary (0 or 255)
    for p in decoded.pixels() {
        assert!(p[0] == 0 || p[0] == 255);
    }

    // Text bar region should be black (0)
    assert_eq!(decoded.get_pixel(50, 30)[0], 0);
    // Background should be white (255)
    assert_eq!(decoded.get_pixel(5, 5)[0], 255);
}

#[test]
fn test_auto_invert_dark_mode() {
    // Dark mode image: black background with white text lines
    let mut dark_img = GrayImage::from_pixel(100, 100, Luma([15]));
    for y in 45..55 {
        for x in 10..90 {
            dark_img.put_pixel(x, y, Luma([245]));
        }
    }

    let input_bytes = encode_gray_png(dark_img);
    let options = PreprocessingOptions {
        auto_invert: true,
        binarize: false,
        contrast_enhancement: false,
        auto_deskew: false,
        ..Default::default()
    };

    let (out_bytes, report) =
        preprocess_image(&input_bytes, &options).expect("Preprocessing succeeds");

    assert!(report.inverted_applied);

    let decoded = image::load_from_memory(&out_bytes)
        .expect("Decode output")
        .to_luma8();

    // Background should now be inverted to near white (> 220)
    assert!(decoded.get_pixel(5, 5)[0] > 220);
    // Text should now be dark (< 30)
    assert!(decoded.get_pixel(50, 50)[0] < 30);
}

#[test]
fn test_standard_light_mode_not_inverted() {
    // Standard document: white background with black text
    let mut light_img = GrayImage::from_pixel(100, 100, Luma([250]));
    for y in 45..55 {
        for x in 10..90 {
            light_img.put_pixel(x, y, Luma([10]));
        }
    }

    let input_bytes = encode_gray_png(light_img);
    let options = PreprocessingOptions::default();

    let (_, report) = preprocess_image(&input_bytes, &options).expect("Preprocessing succeeds");
    assert!(!report.inverted_applied);
}

#[test]
fn test_contrast_stretching() {
    // Image compressed into 80..130 range
    let mut gray = GrayImage::new(50, 50);
    for (i, p) in gray.pixels_mut().enumerate() {
        p[0] = 80 + ((i % 51) as u8);
    }

    enhance_contrast(&mut gray);

    let min_p = gray.pixels().map(|p| p[0]).min().unwrap();
    let max_p = gray.pixels().map(|p| p[0]).max().unwrap();

    assert!(min_p <= 10);
    assert!(max_p >= 245);
}

#[test]
fn test_adaptive_mean_binarization() {
    let mut gray = GrayImage::new(60, 60);
    for y in 0..60 {
        for x in 0..60 {
            // Lighting gradient from top to bottom
            let bg = 100 + (y as u8 * 2);
            let val = if (25..35).contains(&y) && (15..45).contains(&x) {
                bg.saturating_sub(40) // Text darker than local background
            } else {
                bg
            };
            gray.put_pixel(x, y, Luma([val]));
        }
    }

    apply_adaptive_threshold(&mut gray);

    // Verify binary output
    for p in gray.pixels() {
        assert!(p[0] == 0 || p[0] == 255);
    }
}

#[test]
fn test_deskew_angle_detection() {
    let w = 200;
    let h = 200;
    let mut img = GrayImage::from_pixel(w, h, Luma([255]));

    // Draw horizontal black stripes (simulating text lines)
    for y in (20..180).step_by(20) {
        for dy in 0..6 {
            for x in 20..180 {
                img.put_pixel(x, y + dy, Luma([0]));
            }
        }
    }

    // Perfectly horizontal image should have deskew angle ~0.0
    let detected_0 = detect_skew_angle(&img, 10.0, 0.5);
    assert!(detected_0.abs() < 0.51);

    // Rotate image by +4.0 degrees
    let rotated = rotate_gray_image(&img, 4.0);
    let detected_4 = detect_skew_angle(&rotated, 10.0, 0.5);

    // Skew detection should detect approximately +4.0 (+-1.0 deg)
    assert!(
        (detected_4 - 4.0).abs() <= 1.0,
        "Expected ~4.0 degrees, detected: {detected_4}"
    );
}

#[test]
fn test_disabled_preprocessing_passthrough() {
    let raw_bytes = b"non_image_or_raw_passthrough_bytes".to_vec();
    let options = PreprocessingOptions::disabled();

    let (out, report) = preprocess_image(&raw_bytes, &options).expect("Passthrough succeeds");
    assert_eq!(out, raw_bytes);
    assert!(!report.grayscale_applied);
    assert!(!report.binarization_applied);
    assert_eq!(report.duration_ms, 0);
}

#[test]
fn test_dimension_scaling() {
    let img = GrayImage::from_pixel(100, 50, Luma([200]));
    let bytes = encode_gray_png(img);

    // Upscale small image to minimum dimension 300
    let opts = PreprocessingOptions {
        min_dimension: 300,
        ..Default::default()
    };

    let (_, report) = preprocess_image(&bytes, &opts).expect("Preprocessing succeeds");
    assert_eq!(report.original_dimensions, (100, 50));
    assert!(report.processed_dimensions.1 >= 300);
}
