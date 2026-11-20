//! # Sliding Window Chunking with Configurable Overlap
//!
//! Partitions text using a rolling window with token-budget step sizes and boundary-preserving overlap.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, CdmSection, LineRange};
use web_repo_to_ai_core::DocumentId;

use crate::citation::generate_chunk_citation;
use crate::error::ChunkResult;
use crate::model::{Chunk, ChunkId, ChunkKind, ChunkingConfig, ChunkingReport};
use crate::token::TokenCounter;

/// Splits a section using a sliding window with configurable overlap.
pub fn chunk_sliding_window(
    document_id: &DocumentId,
    section: &CdmSection,
    section_index: usize,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> ChunkResult<Vec<Chunk>> {
    let body = &section.body;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    let file_path = section
        .original_path
        .as_deref()
        .or(section.title.as_deref())
        .unwrap_or("document");

    let total_tokens = token_counter.count(body);
    if total_tokens <= config.target_chunk_tokens {
        let raw_content = body.clone();
        let context_header = if config.preserve_headers {
            Some(format!("# File: {file_path}\n\n"))
        } else {
            None
        };

        let content = if let Some(ref hdr) = context_header {
            format!("{hdr}{raw_content}")
        } else {
            raw_content.clone()
        };

        let content_hash = Chunk::compute_hash(&content);
        let token_count = token_counter.count(&content);
        let chunk_id = ChunkId::generate(document_id, section_index, 0, &content_hash);

        let line_range = section.line_range.unwrap_or_else(|| LineRange::new(1, 1));
        let citation = Some(generate_chunk_citation(
            None,
            section,
            &chunk_id,
            &line_range,
            None,
        ));

        let chunk = Chunk {
            chunk_id,
            document_id: *document_id,
            section_id: section.section_id.clone(),
            chunk_index: 0,
            total_chunks: 1,
            title: Some(file_path.to_string()),
            original_path: section.original_path.clone(),
            source_url: section.source_url.clone(),
            line_range,
            byte_range: section.byte_range.unwrap_or_else(|| ByteRange::new(0, 0)),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: section.language.clone(),
            kind: ChunkKind::SlidingWindow,
            ast_symbols: Vec::new(),
            metadata: BTreeMap::new(),
            citation,
            cross_references: Vec::new(),
            prev_chunk_id: None,
            next_chunk_id: None,
            parent_chunk_id: None,
        };

        report.window_chunks_count += 1;
        return Ok(vec![chunk]);
    }

    let lines: Vec<&str> = body.lines().collect();
    let mut chunks = Vec::new();
    let mut window_start = 0;

    while window_start < lines.len() {
        let mut window_end = window_start;
        let mut current_tokens = 0;

        while window_end < lines.len() {
            let line_toks = token_counter.count(lines[window_end]) + 1;
            if current_tokens + line_toks > config.target_chunk_tokens && window_end > window_start
            {
                break;
            }
            current_tokens += line_toks;
            window_end += 1;
        }

        let part_idx = chunks.len() + 1;
        let chunk_lines = &lines[window_start..window_end];
        let raw_content = chunk_lines.join("\n");
        let start_line = (window_start + 1) as u32;
        let end_line = window_end as u32;

        let context_header = if config.preserve_headers {
            Some(format!(
                "# File: {file_path} (Window {part_idx}, Lines {start_line}-{end_line})\n\n"
            ))
        } else {
            None
        };

        let content = if let Some(ref hdr) = context_header {
            format!("{hdr}{raw_content}")
        } else {
            raw_content.clone()
        };

        let content_hash = Chunk::compute_hash(&content);
        let token_count = token_counter.count(&content);
        let chunk_id = ChunkId::generate(document_id, section_index, chunks.len(), &content_hash);

        let mut metadata = BTreeMap::new();
        metadata.insert("window_index".to_string(), part_idx.to_string());
        if window_start > 0 && config.overlap_tokens > 0 {
            metadata.insert("has_overlap_prefix".to_string(), "true".to_string());
        }

        let line_range = LineRange::new(start_line, end_line);
        let citation = Some(generate_chunk_citation(
            None,
            section,
            &chunk_id,
            &line_range,
            None,
        ));

        chunks.push(Chunk {
            chunk_id,
            document_id: *document_id,
            section_id: section.section_id.clone(),
            chunk_index: chunks.len(),
            total_chunks: 1,
            title: Some(format!("{file_path} (Window {part_idx})")),
            original_path: section.original_path.clone(),
            source_url: section.source_url.clone(),
            line_range,
            byte_range: ByteRange::new(0, 0),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: section.language.clone(),
            kind: ChunkKind::SlidingWindow,
            ast_symbols: Vec::new(),
            metadata,
            citation,
            cross_references: Vec::new(),
            prev_chunk_id: None,
            next_chunk_id: None,
            parent_chunk_id: None,
        });

        if window_end >= lines.len() {
            break;
        }

        // Calculate overlap lines from tail
        let mut overlap_lines_count = 0;
        if config.overlap_tokens > 0 {
            let mut acc_overlap_toks = 0;
            for i in (window_start..window_end).rev() {
                let line_toks = token_counter.count(lines[i]) + 1;
                if acc_overlap_toks + line_toks > config.overlap_tokens && overlap_lines_count > 0 {
                    break;
                }
                acc_overlap_toks += line_toks;
                overlap_lines_count += 1;
                if acc_overlap_toks >= config.overlap_tokens {
                    break;
                }
            }
        }

        // Ensure we advance by at least 1 line to guarantee loop termination
        let next_start = window_end.saturating_sub(overlap_lines_count);
        window_start = if next_start > window_start {
            next_start
        } else {
            window_start + 1
        };
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.window_chunks_count += total;
    Ok(chunks)
}
