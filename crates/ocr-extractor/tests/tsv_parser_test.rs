//! # Tesseract TSV Parser Integration Tests
//!
//! Validates parsing of standard Tesseract TSV output streams, confidence score
//! normalization, bounding box calculations, multi-page grouping, and edge cases.

use web_repo_to_ai_ocr_extractor::{parse_tesseract_tsv, BoundingBox, OcrEngineInfo};

const REALISTIC_TSV: &str = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t1200\t800\t-1\t
2\t1\t1\t0\t0\t0\t100\t100\t1000\t150\t-1\t
3\t1\t1\t1\t0\t0\t100\t100\t1000\t150\t-1\t
4\t1\t1\t1\t1\t0\t100\t100\t600\t40\t-1\t
5\t1\t1\t1\t1\t1\t100\t100\t150\t40\t96.5\tArchitecture
5\t1\t1\t1\t1\t2\t260\t100\t120\t40\t94.0\tDecision
5\t1\t1\t1\t1\t3\t390\t100\t110\t40\t95.0\tRecord
4\t1\t1\t1\t2\t0\t100\t150\t800\t35\t-1\t
5\t1\t1\t1\t2\t1\t100\t150\t80\t35\t92.0\tStatus:
5\t1\t1\t1\t2\t2\t190\t150\t120\t35\t98.0\tAccepted
2\t1\t2\t0\t0\t0\t100\t300\t1000\t200\t-1\t
3\t1\t2\t1\t0\t0\t100\t300\t1000\t200\t-1\t
4\t1\t2\t1\t1\t0\t100\t300\t900\t35\t-1\t
5\t1\t2\t1\t1\t1\t100\t300\t80\t35\t88.0\tContext
5\t1\t2\t1\t1\t2\t190\t300\t50\t35\t91.0\tand
5\t1\t2\t1\t1\t3\t250\t300\t120\t35\t89.0\tProblem
5\t1\t2\t1\t1\t4\t380\t300\t140\t35\t90.0\tStatement
1\t2\t0\t0\t0\t0\t0\t0\t1200\t800\t-1\t
2\t2\t1\t0\t0\t0\t100\t100\t1000\t100\t-1\t
3\t2\t1\t1\t0\t0\t100\t100\t1000\t100\t-1\t
4\t2\t1\t1\t1\t0\t100\t100\t500\t35\t-1\t
5\t2\t1\t1\t1\t1\t100\t100\t150\t35\t93.0\tConsequences";

#[test]
fn test_parse_realistic_multipage_tsv() {
    let engine = OcrEngineInfo {
        name: "Tesseract OCR".to_string(),
        version: "5.3.3".to_string(),
        available: true,
        path: Some("/usr/bin/tesseract".to_string()),
        supported_languages: vec!["eng".to_string()],
    };

    let result = parse_tesseract_tsv(REALISTIC_TSV, 120, engine).expect("TSV parse must succeed");

    assert_eq!(result.total_words, 10);
    assert_eq!(result.pages.len(), 2);
    assert_eq!(result.execution_duration_ms, 120);

    // Page 1 assertions
    let page1 = &result.pages[0];
    assert_eq!(page1.page_num, 1);
    assert_eq!(page1.blocks.len(), 2);
    assert_eq!(page1.word_count, 9);

    // Block 1 assertions
    let block1 = &page1.blocks[0];
    assert_eq!(block1.block_num, 1);
    assert_eq!(block1.paragraphs.len(), 1);
    let par1 = &block1.paragraphs[0];
    assert_eq!(par1.lines.len(), 2);
    assert_eq!(par1.lines[0].text, "Architecture Decision Record");
    assert_eq!(par1.lines[1].text, "Status: Accepted");

    // Block 2 assertions
    let block2 = &page1.blocks[1];
    assert_eq!(block2.block_num, 2);
    assert_eq!(block2.text, "Context and Problem Statement");

    // Page 2 assertions
    let page2 = &result.pages[1];
    assert_eq!(page2.page_num, 2);
    assert_eq!(page2.blocks.len(), 1);
    assert_eq!(page2.text, "Consequences");

    // Confidence calculations: all scores between 88.0 and 98.0
    assert!(result.mean_confidence >= 0.88 && result.mean_confidence <= 0.98);
}

#[test]
fn test_bounding_box_union() {
    let box1 = BoundingBox::new(10.0, 20.0, 50.0, 30.0);
    let box2 = BoundingBox::new(70.0, 15.0, 40.0, 40.0);
    let union = box1.union(&box2);

    assert_eq!(union.left, 10.0);
    assert_eq!(union.top, 15.0);
    assert_eq!(union.width, 100.0); // 110 - 10
    assert_eq!(union.height, 40.0); // 55 - 15
}

#[test]
fn test_tsv_filtering_invalid_words() {
    let tsv_with_blanks = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
5\t1\t1\t1\t1\t1\t10\t10\t50\t20\t95.0\tValid
5\t1\t1\t1\t1\t2\t60\t10\t10\t20\t-1.0\t
5\t1\t1\t1\t1\t3\t80\t10\t20\t20\t80.0\t   \t
5\t1\t1\t1\t1\t4\t110\t10\t60\t20\t90.0\tToken";

    let engine = OcrEngineInfo::default();
    let result = parse_tesseract_tsv(tsv_with_blanks, 5, engine).expect("Parse succeeds");

    // Only "Valid" and "Token" should be retained
    assert_eq!(result.total_words, 2);
    assert_eq!(result.text, "Valid Token");
}
