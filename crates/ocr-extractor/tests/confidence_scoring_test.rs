//! # Confidence Scoring and Statistical Analysis Integration Tests
//!
//! Validates confidence score aggregation, distribution metrics (mean, median, std dev, min, max),
//! quality tiers, low-confidence token filtering, and low-confidence warning annotations.

use web_repo_to_ai_ocr_extractor::{
    find_low_confidence_words, parse_tesseract_tsv_with_options, BoundingBox, ConfidenceStats,
    ConfidenceTier, OcrEngineInfo, OcrOptions, OcrWord,
};

#[test]
fn test_confidence_distribution_and_tiers() {
    // Highly accurate scan
    let high_scores = vec![0.98, 0.95, 0.92, 0.89, 0.88, 0.85];
    let high_stats = ConfidenceStats::compute_from_scores(&high_scores);
    assert_eq!(high_stats.total_words, 6);
    assert_eq!(high_stats.tier, ConfidenceTier::High);
    assert_eq!(high_stats.tier.as_str(), "high");
    assert_eq!(high_stats.high_count, 6);
    assert_eq!(high_stats.medium_count, 0);
    assert_eq!(high_stats.low_count, 0);
    assert_eq!(high_stats.high_percentage(), 100.0);
    assert_eq!(high_stats.low_percentage(), 0.0);
    assert!(high_stats.min >= 0.85);
    assert!(high_stats.max <= 0.98);
    assert!(high_stats.std_dev > 0.0);

    // Medium quality scan
    let med_scores = vec![0.85, 0.75, 0.70, 0.65, 0.55];
    let med_stats = ConfidenceStats::compute_from_scores(&med_scores);
    assert_eq!(med_stats.tier, ConfidenceTier::Medium);
    assert_eq!(med_stats.tier.as_str(), "medium");
    assert_eq!(med_stats.high_count, 1);
    assert_eq!(med_stats.medium_count, 4);
    assert_eq!(med_stats.low_count, 0);
    assert_eq!(med_stats.median, 0.70);

    // Low quality / noisy scan
    let low_scores = vec![0.55, 0.45, 0.35, 0.20, 0.10];
    let low_stats = ConfidenceStats::compute_from_scores(&low_scores);
    assert_eq!(low_stats.tier, ConfidenceTier::Low);
    assert_eq!(low_stats.tier.as_str(), "low");
    assert_eq!(low_stats.low_count, 4);
    assert!((low_stats.low_percentage() - 80.0).abs() < 1e-4);
}

#[test]
fn test_median_even_and_odd_counts() {
    // Odd count: exact middle element
    let odd = vec![0.10, 0.50, 0.90];
    let odd_stats = ConfidenceStats::compute_from_scores(&odd);
    assert_eq!(odd_stats.median, 0.50);

    // Even count: mean of two middle elements
    let even = vec![0.40, 0.60, 0.80, 1.00];
    let even_stats = ConfidenceStats::compute_from_scores(&even);
    assert!((even_stats.median - 0.70).abs() < 1e-4);
}

#[test]
fn test_low_confidence_word_detection() {
    let words = vec![
        OcrWord {
            text: "Reliable".to_string(),
            confidence: 0.95,
            bbox: BoundingBox::new(10.0, 10.0, 80.0, 20.0),
            line_num: 1,
            word_num: 1,
        },
        OcrWord {
            text: "Uncertain".to_string(),
            confidence: 0.42,
            bbox: BoundingBox::new(95.0, 10.0, 75.0, 20.0),
            line_num: 1,
            word_num: 2,
        },
        OcrWord {
            text: "Noise".to_string(),
            confidence: 0.18,
            bbox: BoundingBox::new(175.0, 10.0, 50.0, 20.0),
            line_num: 1,
            word_num: 3,
        },
    ];

    let low_words = find_low_confidence_words(&words, 0.50);
    assert_eq!(low_words.len(), 2);
    assert_eq!(low_words[0].text, "Uncertain");
    assert_eq!(low_words[0].confidence, 0.42);
    assert_eq!(low_words[0].word_num, 2);

    assert_eq!(low_words[1].text, "Noise");
    assert_eq!(low_words[1].confidence, 0.18);
    assert_eq!(low_words[1].word_num, 3);
}

#[test]
fn test_tsv_parsing_with_min_confidence_cutoff() {
    let tsv_data = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t10\t10\t50\t20\t92\tClear
5\t1\t1\t1\t1\t2\t65\t10\t50\t20\t35\tSmudged
5\t1\t1\t1\t1\t3\t120\t10\t50\t20\t88\tText
";
    let engine = OcrEngineInfo::default();

    // 1. Without cutoff: all 3 words extracted
    let normal_opts = OcrOptions::default();
    let res_normal = parse_tesseract_tsv_with_options(tsv_data, 10, engine.clone(), &normal_opts)
        .expect("normal parsing should succeed");
    assert_eq!(res_normal.total_words, 3);
    assert_eq!(res_normal.text, "Clear Smudged Text");

    // 2. With cutoff 0.50: "Smudged" (conf 0.35) is omitted
    let filtered_opts = OcrOptions::default().with_min_word_confidence(0.50);
    let res_filtered = parse_tesseract_tsv_with_options(tsv_data, 10, engine, &filtered_opts)
        .expect("filtered parsing should succeed");
    assert_eq!(res_filtered.total_words, 2);
    assert_eq!(res_filtered.text, "Clear Text");
    assert!(res_filtered.mean_confidence >= 0.88);
}

#[test]
fn test_tsv_parsing_with_low_confidence_flagging() {
    let tsv_data = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t10\t10\t50\t20\t95\tInvoice
5\t1\t1\t1\t1\t2\t65\t10\t50\t20\t40\t109283
";
    let engine = OcrEngineInfo::default();
    let opts = OcrOptions::default().with_low_confidence_flagging(0.50, "[?]");

    let res = parse_tesseract_tsv_with_options(tsv_data, 10, engine, &opts)
        .expect("parsing with flagging should succeed");

    assert_eq!(res.total_words, 2);
    assert_eq!(res.text, "Invoice [?]109283");
}

#[test]
fn test_confidence_stats_attributes() {
    let scores = vec![0.90, 0.80, 0.40];
    let stats = ConfidenceStats::compute_from_scores(&scores);
    let attrs = stats.to_attributes("doc");

    assert_eq!(attrs.get("doc_mean").unwrap(), "0.7000");
    assert_eq!(attrs.get("doc_min").unwrap(), "0.4000");
    assert_eq!(attrs.get("doc_max").unwrap(), "0.9000");
    assert_eq!(attrs.get("doc_median").unwrap(), "0.8000");
    assert_eq!(attrs.get("doc_tier").unwrap(), "medium");
    assert_eq!(attrs.get("doc_high_count").unwrap(), "2");
    assert_eq!(attrs.get("doc_low_count").unwrap(), "1");
}
