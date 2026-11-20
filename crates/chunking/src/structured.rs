//! # Structured & Tabular Document Chunking
//!
//! Preserves table column headers across chunks and partitions structured components.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, CdmSection, LineRange, SectionKind};
use web_repo_to_ai_core::DocumentId;

use crate::error::ChunkResult;
use crate::model::{Chunk, ChunkId, ChunkKind, ChunkingConfig, ChunkingReport};
use crate::token::TokenCounter;

/// Splits a tabular or structured section.
pub fn chunk_structured_section(
    document_id: &DocumentId,
    section: &CdmSection,
    section_index: usize,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> ChunkResult<Vec<Chunk>> {
    if section.kind == SectionKind::Table || is_gfm_table(&section.body) {
        chunk_table_section(
            document_id,
            section,
            section_index,
            config,
            token_counter,
            report,
        )
    } else {
        chunk_generic_structured_section(
            document_id,
            section,
            section_index,
            config,
            token_counter,
            report,
        )
    }
}

/// Checks if a string begins with or consists of a GitHub Flavored Markdown table.
fn is_gfm_table(text: &str) -> bool {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    if let (Some(header), Some(sep)) = (lines.next(), lines.next()) {
        header.starts_with('|')
            && header.ends_with('|')
            && sep.starts_with('|')
            && sep.contains('-')
    } else {
        false
    }
}

/// Splits a table section while preserving header and separator rows in every chunk.
fn chunk_table_section(
    document_id: &DocumentId,
    section: &CdmSection,
    section_index: usize,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> ChunkResult<Vec<Chunk>> {
    let lines: Vec<&str> = section.body.lines().collect();
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let file_path = section
        .original_path
        .as_deref()
        .or(section.title.as_deref())
        .unwrap_or("table");

    // Identify header and separator lines
    let mut header_lines = Vec::new();
    let mut data_start_idx = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if header_lines.is_empty() {
            header_lines.push(*line);
        } else if header_lines.len() == 1 && trimmed.starts_with('|') && trimmed.contains('-') {
            header_lines.push(*line);
            data_start_idx = idx + 1;
            break;
        } else {
            data_start_idx = idx;
            break;
        }
    }

    let header_prefix = if !header_lines.is_empty() {
        format!("{}\n", header_lines.join("\n"))
    } else {
        String::new()
    };

    let header_tokens = token_counter.count(&header_prefix);
    let data_lines = &lines[data_start_idx..];

    let mut chunks = Vec::new();
    let mut current_rows = Vec::new();
    let mut current_tokens = header_tokens;
    let mut chunk_data_start_line = (data_start_idx + 1) as u32;

    for (row_idx, line) in data_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let line_tokens = token_counter.count(trimmed);

        if current_tokens + line_tokens > config.target_chunk_tokens && !current_rows.is_empty() {
            let part_idx = chunks.len() + 1;
            let rows_body = current_rows.join("\n");
            let raw_content = if header_prefix.is_empty() {
                rows_body
            } else {
                format!("{header_prefix}{rows_body}")
            };

            let chunk_end_line =
                chunk_data_start_line + current_rows.len().saturating_sub(1) as u32;

            let context_header = if config.preserve_headers {
                Some(format!(
                    "# Table: {file_path} (Part {part_idx}, Rows {chunk_data_start_line}-{chunk_end_line})\n\n"
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
            let chunk_id =
                ChunkId::generate(document_id, section_index, chunks.len(), &content_hash);

            let mut metadata = BTreeMap::new();
            metadata.insert("table_part".to_string(), part_idx.to_string());
            metadata.insert("rows_in_chunk".to_string(), current_rows.len().to_string());
            metadata.insert(
                "header_retained".to_string(),
                (!header_prefix.is_empty()).to_string(),
            );

            chunks.push(Chunk {
                chunk_id,
                document_id: *document_id,
                section_id: section.section_id.clone(),
                chunk_index: chunks.len(),
                total_chunks: 1,
                title: Some(format!("{file_path} (Table Part {part_idx})")),
                original_path: section.original_path.clone(),
                source_url: section.source_url.clone(),
                line_range: LineRange::new(1, chunk_end_line),
                byte_range: ByteRange::new(0, 0),
                content,
                raw_content,
                context_header,
                content_hash,
                token_count,
                token_estimator: token_counter.estimator_name().to_string(),
                language: Some("markdown".to_string()),
                kind: ChunkKind::Table,
                ast_symbols: Vec::new(),
                metadata,
            });

            current_rows.clear();
            current_tokens = header_tokens;
            chunk_data_start_line = (data_start_idx + row_idx + 1) as u32;
        }

        current_tokens += line_tokens;
        current_rows.push(*line);
    }

    if !current_rows.is_empty() {
        let part_idx = chunks.len() + 1;
        let rows_body = current_rows.join("\n");
        let raw_content = if header_prefix.is_empty() {
            rows_body
        } else {
            format!("{header_prefix}{rows_body}")
        };

        let chunk_end_line = chunk_data_start_line + current_rows.len().saturating_sub(1) as u32;

        let context_header = if config.preserve_headers {
            Some(format!(
                "# Table: {file_path} (Part {part_idx}, Rows {chunk_data_start_line}-{chunk_end_line})\n\n"
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
        metadata.insert("table_part".to_string(), part_idx.to_string());
        metadata.insert("rows_in_chunk".to_string(), current_rows.len().to_string());
        metadata.insert(
            "header_retained".to_string(),
            (!header_prefix.is_empty()).to_string(),
        );

        chunks.push(Chunk {
            chunk_id,
            document_id: *document_id,
            section_id: section.section_id.clone(),
            chunk_index: chunks.len(),
            total_chunks: 1,
            title: Some(format!("{file_path} (Table Part {part_idx})")),
            original_path: section.original_path.clone(),
            source_url: section.source_url.clone(),
            line_range: LineRange::new(1, chunk_end_line),
            byte_range: ByteRange::new(0, 0),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: Some("markdown".to_string()),
            kind: ChunkKind::Table,
            ast_symbols: Vec::new(),
            metadata,
        });
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.table_chunks_count += total;
    Ok(chunks)
}

/// Chunks generic structured specs (OpenAPI, JSON Schema, YAML, etc.).
fn chunk_generic_structured_section(
    document_id: &DocumentId,
    section: &CdmSection,
    section_index: usize,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> ChunkResult<Vec<Chunk>> {
    let file_path = section
        .original_path
        .as_deref()
        .or(section.title.as_deref())
        .unwrap_or("spec");

    let is_notebook =
        section.attributes.contains_key("nbformat") || section.attributes.contains_key("cell_type");

    let chunk_kind = if is_notebook {
        ChunkKind::NotebookCell
    } else {
        ChunkKind::StructuredSpec
    };

    let total_tokens = token_counter.count(&section.body);
    if total_tokens <= config.target_chunk_tokens {
        let raw_content = section.body.clone();
        let context_header = if config.preserve_headers {
            Some(format!("# Spec: {file_path}\n\n"))
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

        let mut metadata = BTreeMap::new();
        for (k, v) in &section.attributes {
            metadata.insert(k.clone(), v.clone());
        }

        let chunk = Chunk {
            chunk_id,
            document_id: *document_id,
            section_id: section.section_id.clone(),
            chunk_index: 0,
            total_chunks: 1,
            title: Some(file_path.to_string()),
            original_path: section.original_path.clone(),
            source_url: section.source_url.clone(),
            line_range: section.line_range.unwrap_or_else(|| LineRange::new(1, 1)),
            byte_range: section.byte_range.unwrap_or_else(|| ByteRange::new(0, 0)),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: section.language.clone(),
            kind: chunk_kind,
            ast_symbols: Vec::new(),
            metadata,
        };

        report.ast_chunks_count += 1;
        return Ok(vec![chunk]);
    }

    // Split oversized structured specs by line boundaries
    let lines: Vec<&str> = section.body.lines().collect();
    let mut chunks = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_tokens = 0;
    let mut start_line = 1;

    for line in &lines {
        let line_toks = token_counter.count(line);
        if current_tokens + line_toks > config.target_chunk_tokens && !current_lines.is_empty() {
            let part_idx = chunks.len() + 1;
            let raw_content = current_lines.join("\n");
            let line_count = current_lines.len();
            let end_line = start_line + line_count.saturating_sub(1);

            let context_header = if config.preserve_headers {
                Some(format!("# Spec: {file_path} (Part {part_idx})\n\n"))
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
            let chunk_id =
                ChunkId::generate(document_id, section_index, chunks.len(), &content_hash);

            let mut metadata = BTreeMap::new();
            metadata.insert("part".to_string(), part_idx.to_string());

            chunks.push(Chunk {
                chunk_id,
                document_id: *document_id,
                section_id: section.section_id.clone(),
                chunk_index: chunks.len(),
                total_chunks: 1,
                title: Some(format!("{file_path} (Part {part_idx})")),
                original_path: section.original_path.clone(),
                source_url: section.source_url.clone(),
                line_range: LineRange::new(start_line as u32, end_line as u32),
                byte_range: ByteRange::new(0, 0),
                content,
                raw_content,
                context_header,
                content_hash,
                token_count,
                token_estimator: token_counter.estimator_name().to_string(),
                language: section.language.clone(),
                kind: chunk_kind,
                ast_symbols: Vec::new(),
                metadata,
            });

            current_lines.clear();
            current_tokens = 0;
            start_line = end_line + 1;
        }

        current_tokens += line_toks;
        current_lines.push(*line);
    }

    if !current_lines.is_empty() {
        let part_idx = chunks.len() + 1;
        let raw_content = current_lines.join("\n");
        let line_count = current_lines.len();
        let end_line = start_line + line_count.saturating_sub(1);

        let context_header = if config.preserve_headers {
            Some(format!("# Spec: {file_path} (Part {part_idx})\n\n"))
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
        metadata.insert("part".to_string(), part_idx.to_string());

        chunks.push(Chunk {
            chunk_id,
            document_id: *document_id,
            section_id: section.section_id.clone(),
            chunk_index: chunks.len(),
            total_chunks: 1,
            title: Some(format!("{file_path} (Part {part_idx})")),
            original_path: section.original_path.clone(),
            source_url: section.source_url.clone(),
            line_range: LineRange::new(start_line as u32, end_line as u32),
            byte_range: ByteRange::new(0, 0),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: section.language.clone(),
            kind: chunk_kind,
            ast_symbols: Vec::new(),
            metadata,
        });
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.ast_chunks_count += total;
    Ok(chunks)
}
