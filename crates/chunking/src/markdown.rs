//! # Heading-Aware Markdown & Prose Chunking
//!
//! Partitions documentation and prose along Markdown heading hierarchies (`#`..`######`)
//! and paragraph boundaries while injecting contextual breadcrumbs.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, CdmSection, LineRange};
use web_repo_to_ai_core::DocumentId;

use crate::citation::generate_chunk_citation;
use crate::error::ChunkResult;
use crate::model::{Chunk, ChunkId, ChunkKind, ChunkingConfig, ChunkingReport};
use crate::token::TokenCounter;

/// An individual Markdown heading section with text content and breadcrumb context.
#[derive(Debug, Clone)]
struct MarkdownHeadingSection {
    pub level: usize,
    pub heading: String,
    pub breadcrumbs: Vec<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub body: String,
}

/// Splits a Markdown or prose section along heading hierarchies and paragraph boundaries.
pub fn chunk_markdown_section(
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

    let ctx = crate::model::ChunkContext::new(*document_id, section, section_index, file_path);

    // 1. Parse heading sections
    let heading_sections = parse_markdown_headings(body, file_path);

    // If no headings found, split by paragraphs
    if heading_sections.is_empty() {
        return chunk_prose_paragraphs(
            document_id,
            section,
            section_index,
            config,
            token_counter,
            report,
            file_path,
        );
    }

    // 2. Pack heading sections into chunks respecting token budgets
    let mut chunks = Vec::new();
    let mut current_sections: Vec<MarkdownHeadingSection> = Vec::new();
    let mut current_tokens = 0;

    for hsec in heading_sections {
        let sec_tokens = token_counter.count(&hsec.body);

        // If a single heading section exceeds max_chunk_tokens, split by paragraphs
        if sec_tokens > config.max_chunk_tokens {
            if !current_sections.is_empty() {
                let chunk = assemble_markdown_chunk(
                    &ctx,
                    chunks.len(),
                    &current_sections,
                    config,
                    token_counter,
                );
                chunks.push(chunk);
                current_sections.clear();
                current_tokens = 0;
            }

            let split_chunks = split_oversized_markdown_section(
                &ctx,
                chunks.len(),
                &hsec,
                config,
                token_counter,
                report,
            );
            chunks.extend(split_chunks);
            continue;
        }

        if current_tokens + sec_tokens > config.max_chunk_tokens && !current_sections.is_empty() {
            let chunk = assemble_markdown_chunk(
                &ctx,
                chunks.len(),
                &current_sections,
                config,
                token_counter,
            );
            chunks.push(chunk);
            current_sections.clear();
            current_tokens = 0;
        }

        current_tokens += sec_tokens;
        current_sections.push(hsec);

        if current_tokens >= config.target_chunk_tokens {
            let chunk = assemble_markdown_chunk(
                &ctx,
                chunks.len(),
                &current_sections,
                config,
                token_counter,
            );
            chunks.push(chunk);
            current_sections.clear();
            current_tokens = 0;
        }
    }

    if !current_sections.is_empty() {
        let chunk =
            assemble_markdown_chunk(&ctx, chunks.len(), &current_sections, config, token_counter);
        chunks.push(chunk);
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.heading_chunks_count += total;
    Ok(chunks)
}

/// Parses Markdown text into heading sections with breadcrumbs.
fn parse_markdown_headings(text: &str, file_path: &str) -> Vec<MarkdownHeadingSection> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut heading_indices = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let hashes = trimmed.chars().take_while(|&c| c == '#').count();
            if hashes <= 6 && trimmed[hashes..].starts_with(' ') {
                let heading_title = trimmed[hashes..].trim().to_string();
                heading_indices.push((idx, hashes, heading_title));
            }
        }
    }

    if heading_indices.is_empty() {
        return Vec::new();
    }

    let mut sections = Vec::new();
    let mut hierarchy_stack: Vec<(usize, String)> = Vec::new();

    // Capture leading text before first heading if present
    if heading_indices[0].0 > 0 {
        let preamble_lines = &lines[0..heading_indices[0].0];
        let preamble = preamble_lines.join("\n");
        if !preamble.trim().is_empty() {
            sections.push(MarkdownHeadingSection {
                level: 0,
                heading: file_path.to_string(),
                breadcrumbs: vec![file_path.to_string()],
                start_line: 1,
                end_line: heading_indices[0].0,
                body: preamble,
            });
        }
    }

    for i in 0..heading_indices.len() {
        let (line_idx, level, ref title) = heading_indices[i];
        let start_line = line_idx + 1;
        let end_line = if i + 1 < heading_indices.len() {
            heading_indices[i + 1].0
        } else {
            lines.len()
        };

        // Update hierarchy stack
        while let Some(&(last_level, _)) = hierarchy_stack.last() {
            if last_level >= level {
                hierarchy_stack.pop();
            } else {
                break;
            }
        }
        hierarchy_stack.push((level, title.clone()));

        let mut breadcrumbs = vec![file_path.to_string()];
        breadcrumbs.extend(hierarchy_stack.iter().map(|(_, t)| t.clone()));

        let section_lines = &lines[line_idx..end_line];
        let body = section_lines.join("\n");

        sections.push(MarkdownHeadingSection {
            level,
            heading: title.clone(),
            breadcrumbs,
            start_line,
            end_line,
            body,
        });
    }

    sections
}

/// Assembles a single chunk from multiple markdown heading sections.
fn assemble_markdown_chunk(
    ctx: &crate::model::ChunkContext<'_>,
    chunk_index: usize,
    sections: &[MarkdownHeadingSection],
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
) -> Chunk {
    let first = sections.first().expect("non-empty sections");
    let last = sections.last().expect("non-empty sections");

    let start_line = first.start_line as u32;
    let end_line = last.end_line as u32;

    let raw_content = sections
        .iter()
        .map(|s| s.body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    let breadcrumbs = &first.breadcrumbs;
    let breadcrumb_str = breadcrumbs.join(" > ");

    let context_header = if config.preserve_headers {
        Some(format!("# Context: {breadcrumb_str}\n\n"))
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
    let chunk_id = ChunkId::generate(
        &ctx.document_id,
        ctx.section_index,
        chunk_index,
        &content_hash,
    );

    let title = Some(breadcrumb_str.clone());

    let mut metadata = BTreeMap::new();
    metadata.insert("breadcrumbs".to_string(), breadcrumb_str);
    metadata.insert("heading".to_string(), first.heading.clone());
    metadata.insert("heading_level".to_string(), first.level.to_string());
    metadata.insert("source_file".to_string(), ctx.file_path.to_string());

    let line_range = LineRange::new(start_line, end_line);
    let citation = Some(generate_chunk_citation(
        None,
        ctx.section,
        &chunk_id,
        &line_range,
        None,
    ));

    Chunk {
        chunk_id,
        document_id: ctx.document_id,
        section_id: ctx.section.section_id.clone(),
        chunk_index,
        total_chunks: 1,
        title,
        original_path: ctx.section.original_path.clone(),
        source_url: ctx.section.source_url.clone(),
        line_range,
        byte_range: ByteRange::new(0, 0),
        content,
        raw_content,
        context_header,
        content_hash,
        token_count,
        token_estimator: token_counter.estimator_name().to_string(),
        language: Some("markdown".to_string()),
        kind: ChunkKind::MarkdownSection,
        ast_symbols: Vec::new(),
        metadata,
        citation,
        cross_references: Vec::new(),
        prev_chunk_id: None,
        next_chunk_id: None,
        parent_chunk_id: None,
    }
}

/// Splits an oversized markdown section across paragraph or sentence boundaries.
fn split_oversized_markdown_section(
    ctx: &crate::model::ChunkContext<'_>,
    starting_chunk_idx: usize,
    hsec: &MarkdownHeadingSection,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> Vec<Chunk> {
    report.oversized_splits_count += 1;

    let breadcrumb_str = hsec.breadcrumbs.join(" > ");
    let paragraphs: Vec<&str> = hsec.body.split("\n\n").collect();

    let mut chunks = Vec::new();
    let mut current_paras = Vec::new();
    let mut current_tokens = 0;
    let mut current_start_line = hsec.start_line;

    for para in paragraphs {
        let para_trimmed = para.trim();
        if para_trimmed.is_empty() {
            continue;
        }

        let para_tokens = token_counter.count(para_trimmed);

        if current_tokens + para_tokens > config.target_chunk_tokens && !current_paras.is_empty() {
            let part_idx = chunks.len() + 1;
            let raw_content = current_paras.join("\n\n");
            let line_count = raw_content.lines().count();
            let chunk_end_line = current_start_line + line_count.saturating_sub(1);

            let context_header = if config.preserve_headers {
                Some(format!("# Context: {breadcrumb_str} (Part {part_idx})\n\n"))
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
            let chunk_id = ChunkId::generate(
                &ctx.document_id,
                ctx.section_index,
                starting_chunk_idx + chunks.len(),
                &content_hash,
            );

            let mut metadata = BTreeMap::new();
            metadata.insert(
                "breadcrumbs".to_string(),
                format!("{breadcrumb_str} (Part {part_idx})"),
            );
            metadata.insert("is_oversized_split".to_string(), "true".to_string());
            metadata.insert("split_part".to_string(), part_idx.to_string());

            let line_range = LineRange::new(current_start_line as u32, chunk_end_line as u32);
            let citation = Some(generate_chunk_citation(
                None,
                ctx.section,
                &chunk_id,
                &line_range,
                None,
            ));

            chunks.push(Chunk {
                chunk_id,
                document_id: ctx.document_id,
                section_id: ctx.section.section_id.clone(),
                chunk_index: starting_chunk_idx + chunks.len(),
                total_chunks: 1,
                title: Some(format!("{breadcrumb_str} (Part {part_idx})")),
                original_path: ctx.section.original_path.clone(),
                source_url: ctx.section.source_url.clone(),
                line_range,
                byte_range: ByteRange::new(0, 0),
                content,
                raw_content,
                context_header,
                content_hash,
                token_count,
                token_estimator: token_counter.estimator_name().to_string(),
                language: Some("markdown".to_string()),
                kind: ChunkKind::MarkdownSection,
                ast_symbols: Vec::new(),
                metadata,
                citation,
                cross_references: Vec::new(),
                prev_chunk_id: None,
                next_chunk_id: None,
                parent_chunk_id: None,
            });

            current_paras.clear();
            current_tokens = 0;
            current_start_line = chunk_end_line + 1;
        }

        current_tokens += para_tokens;
        current_paras.push(para_trimmed);
    }

    if !current_paras.is_empty() {
        let part_idx = chunks.len() + 1;
        let raw_content = current_paras.join("\n\n");
        let line_count = raw_content.lines().count();
        let chunk_end_line = current_start_line + line_count.saturating_sub(1);

        let context_header = if config.preserve_headers {
            Some(format!("# Context: {breadcrumb_str} (Part {part_idx})\n\n"))
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
        let chunk_id = ChunkId::generate(
            &ctx.document_id,
            ctx.section_index,
            starting_chunk_idx + chunks.len(),
            &content_hash,
        );

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "breadcrumbs".to_string(),
            format!("{breadcrumb_str} (Part {part_idx})"),
        );
        metadata.insert("source_file".to_string(), ctx.file_path.to_string());

        let line_range = LineRange::new(current_start_line as u32, chunk_end_line as u32);
        let citation = Some(generate_chunk_citation(
            None,
            ctx.section,
            &chunk_id,
            &line_range,
            None,
        ));

        chunks.push(Chunk {
            chunk_id,
            document_id: ctx.document_id,
            section_id: ctx.section.section_id.clone(),
            chunk_index: starting_chunk_idx + chunks.len(),
            total_chunks: 1,
            title: Some(format!("{breadcrumb_str} (Part {part_idx})")),
            original_path: ctx.section.original_path.clone(),
            source_url: ctx.section.source_url.clone(),
            line_range,
            byte_range: ByteRange::new(0, 0),
            content,
            raw_content,
            context_header,
            content_hash,
            token_count,
            token_estimator: token_counter.estimator_name().to_string(),
            language: Some("markdown".to_string()),
            kind: ChunkKind::MarkdownSection,
            ast_symbols: Vec::new(),
            metadata,
            citation,
            cross_references: Vec::new(),
            prev_chunk_id: None,
            next_chunk_id: None,
            parent_chunk_id: None,
        });
    }

    chunks
}

/// Fallback for prose sections with no explicit Markdown headings.
fn chunk_prose_paragraphs(
    document_id: &DocumentId,
    section: &CdmSection,
    section_index: usize,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
    file_path: &str,
) -> ChunkResult<Vec<Chunk>> {
    let paragraphs: Vec<&str> = section.body.split("\n\n").collect();
    if paragraphs.is_empty() {
        return Ok(Vec::new());
    }

    let mut chunks = Vec::new();
    let mut current_paras = Vec::new();
    let mut current_tokens = 0;
    let mut start_line = 1;

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }

        let p_toks = token_counter.count(trimmed);

        if current_tokens + p_toks > config.target_chunk_tokens && !current_paras.is_empty() {
            let chunk_index = chunks.len();
            let raw_content = current_paras.join("\n\n");
            let line_count = raw_content.lines().count();
            let end_line = start_line + line_count.saturating_sub(1);

            let context_header = if config.preserve_headers {
                Some(format!("# Document: {file_path}\n\n"))
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
                ChunkId::generate(document_id, section_index, chunk_index, &content_hash);

            let line_range = LineRange::new(start_line as u32, end_line as u32);
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
                chunk_index,
                total_chunks: 1,
                title: Some(format!("{file_path} (Section {chunk_index})")),
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
                kind: ChunkKind::Paragraph,
                ast_symbols: Vec::new(),
                metadata: BTreeMap::new(),
                citation,
                cross_references: Vec::new(),
                prev_chunk_id: None,
                next_chunk_id: None,
                parent_chunk_id: None,
            });

            current_paras.clear();
            current_tokens = 0;
            start_line = end_line + 1;
        }

        current_tokens += p_toks;
        current_paras.push(trimmed);
    }

    if !current_paras.is_empty() {
        let chunk_index = chunks.len();
        let raw_content = current_paras.join("\n\n");
        let line_count = raw_content.lines().count();
        let end_line = start_line + line_count.saturating_sub(1);

        let context_header = if config.preserve_headers {
            Some(format!("# Document: {file_path}\n\n"))
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
        let chunk_id = ChunkId::generate(document_id, section_index, chunk_index, &content_hash);

        let line_range = LineRange::new(start_line as u32, end_line as u32);
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
            chunk_index,
            total_chunks: 1,
            title: Some(format!("{file_path} (Section {chunk_index})")),
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
            kind: ChunkKind::Paragraph,
            ast_symbols: Vec::new(),
            metadata: BTreeMap::new(),
            citation,
            cross_references: Vec::new(),
            prev_chunk_id: None,
            next_chunk_id: None,
            parent_chunk_id: None,
        });
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.heading_chunks_count += total;
    Ok(chunks)
}
