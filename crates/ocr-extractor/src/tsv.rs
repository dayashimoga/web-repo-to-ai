//! # Tesseract TSV Stream Parser
//!
//! Parses Tesseract TSV (Tab-Separated Values) output streams into structured
//! hierarchical models (`OcrResult`, `OcrPage`, `OcrBlock`, `OcrParagraph`, `OcrLine`, `OcrWord`).

use crate::error::Result;
use crate::models::{
    BoundingBox, OcrBlock, OcrEngineInfo, OcrLine, OcrPage, OcrParagraph, OcrResult, OcrWord,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct RawTsvRow {
    level: usize,
    page_num: usize,
    block_num: usize,
    par_num: usize,
    line_num: usize,
    word_num: usize,
    bbox: BoundingBox,
    conf: f64,
    text: String,
}

/// Parses raw Tesseract TSV text into an aggregated `OcrResult`.
pub fn parse_tesseract_tsv(
    tsv: &str,
    execution_duration_ms: u64,
    engine_info: OcrEngineInfo,
) -> Result<OcrResult> {
    let mut words: Vec<RawTsvRow> = Vec::new();
    let mut lines = tsv.lines();

    // Verify or skip header
    if let Some(first_line) = lines.next() {
        let first_lower = first_line.to_ascii_lowercase();
        if !first_lower.starts_with("level") && !first_lower.contains("page_num") {
            // First line might already be data
            if let Some(row) = parse_row(first_line) {
                if row.level == 5 && !row.text.trim().is_empty() {
                    words.push(row);
                }
            }
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(row) = parse_row(trimmed) {
            // Level 5 represents word tokens in Tesseract TSV format
            if row.level == 5 && !row.text.trim().is_empty() {
                words.push(row);
            }
        }
    }

    if words.is_empty() {
        return Ok(OcrResult {
            pages: Vec::new(),
            text: String::new(),
            mean_confidence: 0.0,
            total_words: 0,
            execution_duration_ms,
            engine_info,
            preprocessing_report: None,
        });
    }

    // Hierarchical grouping: Page -> Block -> Paragraph -> Line -> Words
    // Key: (page_num, block_num, par_num, line_num)
    type GroupKey = (usize, usize, usize, usize);
    let mut grouped: BTreeMap<GroupKey, Vec<OcrWord>> = BTreeMap::new();

    let mut sum_confidence = 0.0;
    let total_words = words.len();

    for raw in words {
        let norm_conf = (raw.conf.max(0.0) / 100.0).clamp(0.0, 1.0);
        sum_confidence += norm_conf;

        let word = OcrWord {
            text: raw.text,
            confidence: norm_conf,
            bbox: raw.bbox,
            line_num: raw.line_num,
            word_num: raw.word_num,
        };

        let key = (raw.page_num, raw.block_num, raw.par_num, raw.line_num);
        grouped.entry(key).or_default().push(word);
    }

    let mean_confidence = if total_words > 0 {
        sum_confidence / (total_words as f64)
    } else {
        0.0
    };

    // Construct Lines
    let mut lines_map: BTreeMap<(usize, usize, usize), Vec<OcrLine>> = BTreeMap::new();
    for ((page_num, block_num, par_num, line_num), mut line_words) in grouped {
        line_words.sort_by_key(|w| w.word_num);

        let line_text = line_words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let line_conf = if !line_words.is_empty() {
            line_words.iter().map(|w| w.confidence).sum::<f64>() / (line_words.len() as f64)
        } else {
            0.0
        };

        let line_bbox = line_words
            .iter()
            .map(|w| w.bbox)
            .reduce(|a, b| a.union(&b))
            .unwrap_or_default();

        let ocr_line = OcrLine {
            words: line_words,
            text: line_text,
            confidence: line_conf,
            bbox: line_bbox,
            line_num,
        };

        lines_map
            .entry((page_num, block_num, par_num))
            .or_default()
            .push(ocr_line);
    }

    // Construct Paragraphs
    let mut par_map: BTreeMap<(usize, usize), Vec<OcrParagraph>> = BTreeMap::new();
    for ((page_num, block_num, par_num), mut par_lines) in lines_map {
        par_lines.sort_by_key(|l| l.line_num);

        let par_text = par_lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let par_words_count: usize = par_lines.iter().map(|l| l.words.len()).sum();
        let par_conf = if par_words_count > 0 {
            par_lines
                .iter()
                .map(|l| l.confidence * (l.words.len() as f64))
                .sum::<f64>()
                / (par_words_count as f64)
        } else {
            0.0
        };

        let par_bbox = par_lines
            .iter()
            .map(|l| l.bbox)
            .reduce(|a, b| a.union(&b))
            .unwrap_or_default();

        let ocr_par = OcrParagraph {
            lines: par_lines,
            text: par_text,
            confidence: par_conf,
            bbox: par_bbox,
            par_num,
        };

        par_map
            .entry((page_num, block_num))
            .or_default()
            .push(ocr_par);
    }

    // Construct Blocks
    let mut block_map: BTreeMap<usize, Vec<OcrBlock>> = BTreeMap::new();
    for ((page_num, block_num), mut block_pars) in par_map {
        block_pars.sort_by_key(|p| p.par_num);

        let block_text = block_pars
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let total_block_words: usize = block_pars
            .iter()
            .flat_map(|p| p.lines.iter())
            .map(|l| l.words.len())
            .sum();

        let block_conf = if total_block_words > 0 {
            block_pars
                .iter()
                .flat_map(|p| p.lines.iter())
                .map(|l| l.confidence * (l.words.len() as f64))
                .sum::<f64>()
                / (total_block_words as f64)
        } else {
            0.0
        };

        let block_bbox = block_pars
            .iter()
            .map(|p| p.bbox)
            .reduce(|a, b| a.union(&b))
            .unwrap_or_default();

        let ocr_block = OcrBlock {
            block_num,
            paragraphs: block_pars,
            text: block_text,
            confidence: block_conf,
            bbox: block_bbox,
        };

        block_map.entry(page_num).or_default().push(ocr_block);
    }

    // Construct Pages
    let mut pages: Vec<OcrPage> = Vec::new();
    for (page_num, mut page_blocks) in block_map {
        page_blocks.sort_by_key(|b| b.block_num);

        let page_text = page_blocks
            .iter()
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let page_words: usize = page_blocks
            .iter()
            .flat_map(|b| b.paragraphs.iter())
            .flat_map(|p| p.lines.iter())
            .map(|l| l.words.len())
            .sum();

        let page_conf = if page_words > 0 {
            page_blocks
                .iter()
                .flat_map(|b| b.paragraphs.iter())
                .flat_map(|p| p.lines.iter())
                .map(|l| l.confidence * (l.words.len() as f64))
                .sum::<f64>()
                / (page_words as f64)
        } else {
            0.0
        };

        pages.push(OcrPage {
            page_num,
            blocks: page_blocks,
            text: page_text,
            mean_confidence: page_conf,
            word_count: page_words,
        });
    }

    pages.sort_by_key(|p| p.page_num);

    let doc_text = pages
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(OcrResult {
        pages,
        text: doc_text,
        mean_confidence,
        total_words,
        execution_duration_ms,
        engine_info,
        preprocessing_report: None,
    })
}

fn parse_row(line: &str) -> Option<RawTsvRow> {
    let mut cols = line.split('\t');

    let level_str = cols.next()?;
    let level: usize = level_str.parse().ok()?;

    let page_num: usize = cols.next()?.parse().unwrap_or(1);
    let block_num: usize = cols.next()?.parse().unwrap_or(1);
    let par_num: usize = cols.next()?.parse().unwrap_or(1);
    let line_num: usize = cols.next()?.parse().unwrap_or(1);
    let word_num: usize = cols.next()?.parse().unwrap_or(1);

    let left: f64 = cols.next()?.parse().unwrap_or(0.0);
    let top: f64 = cols.next()?.parse().unwrap_or(0.0);
    let width: f64 = cols.next()?.parse().unwrap_or(0.0);
    let height: f64 = cols.next()?.parse().unwrap_or(0.0);

    let conf_str = cols.next()?;
    let conf: f64 = conf_str.parse().unwrap_or(-1.0);

    // Text may contain tabs or be the remainder
    let text = cols.collect::<Vec<_>>().join("\t");

    Some(RawTsvRow {
        level,
        page_num,
        block_num,
        par_num,
        line_num,
        word_num,
        bbox: BoundingBox::new(left, top, width, height),
        conf,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TSV: &str = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t800\t600\t-1\t
2\t1\t1\t0\t0\t0\t50\t50\t700\t200\t-1\t
3\t1\t1\t1\t0\t0\t50\t50\t700\t200\t-1\t
4\t1\t1\t1\t1\t0\t50\t50\t400\t30\t-1\t
5\t1\t1\t1\t1\t1\t50\t50\t100\t30\t95.0\tHello
5\t1\t1\t1\t1\t2\t160\t50\t120\t30\t90.0\tWorld
4\t1\t1\t1\t2\t0\t50\t90\t500\t30\t-1\t
5\t1\t1\t1\t2\t1\t50\t90\t150\t30\t85.0\tTesseract
5\t1\t1\t1\t2\t2\t210\t90\t80\t30\t80.0\tOCR";

    #[test]
    fn test_parse_sample_tsv() {
        let engine = OcrEngineInfo::default();
        let result = parse_tesseract_tsv(SAMPLE_TSV, 42, engine).expect("parse should succeed");

        assert_eq!(result.total_words, 4);
        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];
        assert_eq!(page.page_num, 1);
        assert_eq!(page.blocks.len(), 1);
        let block = &page.blocks[0];
        assert_eq!(block.paragraphs.len(), 1);
        let par = &block.paragraphs[0];
        assert_eq!(par.lines.len(), 2);

        assert_eq!(par.lines[0].text, "Hello World");
        assert_eq!(par.lines[1].text, "Tesseract OCR");
        assert_eq!(par.text, "Hello World\nTesseract OCR");

        // Mean confidence: (0.95 + 0.90 + 0.85 + 0.80) / 4 = 0.875
        assert!((result.mean_confidence - 0.875).abs() < 1e-4);
        assert_eq!(result.execution_duration_ms, 42);
    }

    #[test]
    fn test_parse_empty_tsv() {
        let engine = OcrEngineInfo::default();
        let result = parse_tesseract_tsv("", 10, engine).expect("empty parse succeeds");
        assert_eq!(result.total_words, 0);
        assert!(result.text.is_empty());
        assert_eq!(result.mean_confidence, 0.0);
    }
}
