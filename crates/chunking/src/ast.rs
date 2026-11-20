//! # AST-Boundary Code Chunking
//!
//! Partitions source code along declaration boundaries (functions, structs, classes,
//! traits, interfaces, and methods) using syntactic tree analysis.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, CdmSection, LineRange};
use web_repo_to_ai_code_intelligence::{Language, SymbolDeclaration, SymbolExtractor};
use web_repo_to_ai_core::DocumentId;

use crate::error::ChunkResult;
use crate::model::{
    Chunk, ChunkContext, ChunkId, ChunkKind, ChunkSymbolMetadata, ChunkingConfig, ChunkingReport,
};
use crate::token::TokenCounter;

/// AST declaration slice with source spans and extracted symbols.
#[derive(Debug, Clone)]
struct AstDeclarationSpan {
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub content: String,
    pub symbols: Vec<SymbolDeclaration>,
}

/// Splits a code section along AST declaration boundaries.
pub fn chunk_code_section(
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

    // 1. Resolve Language
    let lang = resolve_language(section);

    // 2. Extract AST Symbols
    let symbols = SymbolExtractor::extract(&lang, body);

    // If no symbols found, chunk as generic code blocks
    if symbols.is_empty() {
        return chunk_generic_code_blocks(
            document_id,
            section,
            section_index,
            config,
            token_counter,
            report,
        );
    }

    // 3. Compute Declaration Spans
    let spans = compute_declaration_spans(body, &lang, &symbols);

    // 4. Pack spans into chunks respecting token budgets
    let mut chunks = Vec::new();
    let file_path = section
        .original_path
        .as_deref()
        .or(section.title.as_deref())
        .unwrap_or("source_file");

    let ctx = ChunkContext::new(*document_id, section, section_index, file_path);
    let mut current_spans: Vec<AstDeclarationSpan> = Vec::new();
    let mut current_tokens = 0;

    for span in spans {
        let span_tokens = token_counter.count(&span.content);

        // If a single declaration span exceeds max_chunk_tokens, split it cleanly
        if span_tokens > config.max_chunk_tokens {
            // First flush any pending accumulated spans
            if !current_spans.is_empty() {
                let chunk =
                    assemble_ast_chunk(&ctx, chunks.len(), &current_spans, config, token_counter);
                chunks.push(chunk);
                current_spans.clear();
                current_tokens = 0;
            }

            // Split the oversized declaration
            let oversized_chunks = split_oversized_ast_declaration(
                &ctx,
                chunks.len(),
                &span,
                config,
                token_counter,
                report,
            );
            chunks.extend(oversized_chunks);
            continue;
        }

        // Check if adding this span would exceed target_chunk_tokens
        if current_tokens + span_tokens > config.max_chunk_tokens && !current_spans.is_empty() {
            let chunk =
                assemble_ast_chunk(&ctx, chunks.len(), &current_spans, config, token_counter);
            chunks.push(chunk);
            current_spans.clear();
            current_tokens = 0;
        }

        current_tokens += span_tokens;
        current_spans.push(span);

        // If we reached or exceeded target_chunk_tokens and have sufficient content
        if current_tokens >= config.target_chunk_tokens {
            let chunk =
                assemble_ast_chunk(&ctx, chunks.len(), &current_spans, config, token_counter);
            chunks.push(chunk);
            current_spans.clear();
            current_tokens = 0;
        }
    }

    // Flush any remaining spans
    if !current_spans.is_empty() {
        let chunk = assemble_ast_chunk(&ctx, chunks.len(), &current_spans, config, token_counter);
        chunks.push(chunk);
    }

    // Update total_chunks count on each chunk
    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.ast_chunks_count += total;
    Ok(chunks)
}

/// Resolves the code intelligence `Language` for a section.
fn resolve_language(section: &CdmSection) -> Language {
    if let Some(lang_str) = section.language.as_deref() {
        let l = Language::from_identifier(lang_str);
        if !matches!(l, Language::Unknown(_)) {
            return l;
        }
    }
    Language::detect(section.original_path.as_deref(), &section.body)
}

/// Computes declaration spans pairing symbol declarations with their body boundaries.
fn compute_declaration_spans(
    body: &str,
    lang: &Language,
    symbols: &[SymbolDeclaration],
) -> Vec<AstDeclarationSpan> {
    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    // Precalculate line byte offsets
    let mut line_byte_offsets = Vec::with_capacity(lines.len() + 1);
    let mut acc_byte = 0;
    for l in &lines {
        line_byte_offsets.push(acc_byte);
        acc_byte += l.len() + 1; // newline
    }
    line_byte_offsets.push(acc_byte.min(body.len()));

    let mut spans = Vec::new();

    // 1. Capture preamble before the first symbol if present
    if let Some(first_sym) = symbols.first() {
        if first_sym.line_number > 1 {
            let preamble_end_line = first_sym.line_number - 1;
            let start_byte = 0;
            let end_byte = line_byte_offsets[preamble_end_line].min(body.len());
            let preamble_text = &body[start_byte..end_byte];
            if !preamble_text.trim().is_empty() {
                spans.push(AstDeclarationSpan {
                    start_line: 1,
                    end_line: preamble_end_line,
                    start_byte,
                    end_byte,
                    content: preamble_text.to_string(),
                    symbols: Vec::new(),
                });
            }
        }
    }

    // 2. Compute spans for each symbol
    let mut sym_idx = 0;
    while sym_idx < symbols.len() {
        let sym = &symbols[sym_idx];
        let start_line = sym.line_number;

        // Determine end line for this symbol
        let end_line = match lang {
            Language::Python => find_python_scope_end(&lines, start_line),
            Language::Rust
            | Language::TypeScript
            | Language::JavaScript
            | Language::Go
            | Language::Java
            | Language::CSharp
            | Language::Cpp
            | Language::C => find_brace_scope_end(&lines, start_line),
            _ => {
                // Fallback: end is line before next symbol or EOF
                if sym_idx + 1 < symbols.len() {
                    symbols[sym_idx + 1].line_number.saturating_sub(1)
                } else {
                    lines.len()
                }
            }
        };

        let end_line = end_line.max(start_line).min(lines.len());

        // Collect all symbols that fall within [start_line, end_line]
        let mut span_symbols = vec![sym.clone()];
        let mut next_sym_idx = sym_idx + 1;
        while next_sym_idx < symbols.len() && symbols[next_sym_idx].line_number <= end_line {
            span_symbols.push(symbols[next_sym_idx].clone());
            next_sym_idx += 1;
        }

        let start_byte = line_byte_offsets[start_line - 1];
        let end_byte = line_byte_offsets[end_line].min(body.len());
        let content = body[start_byte..end_byte].to_string();

        spans.push(AstDeclarationSpan {
            start_line,
            end_line,
            start_byte,
            end_byte,
            content,
            symbols: span_symbols,
        });

        sym_idx = next_sym_idx;
    }

    spans
}

/// Finds the closing brace of a block starting at `start_line` (1-indexed).
fn find_brace_scope_end(lines: &[&str], start_line: usize) -> usize {
    let mut brace_depth: i32 = 0;
    let mut opened = false;

    for (idx, line) in lines.iter().enumerate().skip(start_line - 1) {
        let trimmed = line.trim();
        // Skip comment lines
        if trimmed.starts_with("//") {
            continue;
        }

        let mut in_quote = false;
        let mut quote_char = ' ';
        let mut prev_char = ' ';

        for ch in line.chars() {
            if in_quote {
                if ch == quote_char && prev_char != '\\' {
                    in_quote = false;
                }
            } else if ch == '"' || ch == '\'' || ch == '`' {
                in_quote = true;
                quote_char = ch;
            } else if ch == '{' {
                brace_depth += 1;
                opened = true;
            } else if ch == '}' {
                brace_depth -= 1;
                if opened && brace_depth <= 0 {
                    return idx + 1;
                }
            }
            prev_char = ch;
        }

        // If braces opened and now depth is 0 or less, scope ended
        if opened && brace_depth <= 0 {
            return idx + 1;
        }
    }

    lines.len()
}

/// Finds the end of a Python indented block starting at `start_line` (1-indexed).
fn find_python_scope_end(lines: &[&str], start_line: usize) -> usize {
    if start_line > lines.len() {
        return lines.len();
    }

    let header_line = lines[start_line - 1];
    let base_indent = header_line.len() - header_line.trim_start().len();

    let mut last_non_empty = start_line;

    for (idx, line) in lines.iter().enumerate().skip(start_line) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent <= base_indent {
            return last_non_empty;
        }
        last_non_empty = idx + 1;
    }

    last_non_empty
}

/// Assembles a single `Chunk` from a collection of AST declaration spans.
fn assemble_ast_chunk(
    ctx: &crate::model::ChunkContext<'_>,
    chunk_index: usize,
    spans: &[AstDeclarationSpan],
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
) -> Chunk {
    let first = spans.first().expect("non-empty spans");
    let last = spans.last().expect("non-empty spans");

    let start_line = first.start_line as u32;
    let end_line = last.end_line as u32;
    let start_byte = first.start_byte as u64;
    let end_byte = last.end_byte as u64;

    let raw_content = spans
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut all_symbols = Vec::new();
    for span in spans {
        for sym in &span.symbols {
            all_symbols.push(ChunkSymbolMetadata {
                name: sym.name.clone(),
                kind: sym.kind,
                visibility: sym.visibility,
                signature: sym.signature.clone(),
                doc_comment: sym.doc_comment.clone(),
                parent_scope: None,
                line_number: sym.line_number,
            });
        }
    }

    let primary_symbol = all_symbols.first().map(|s| s.name.as_str());

    // Construct context header if enabled
    let context_header = if config.preserve_headers {
        let mut header = format!("// File: {}", ctx.file_path);
        if let Some(sym_name) = primary_symbol {
            header.push_str(&format!(" > Symbol: {sym_name}"));
        }
        header.push('\n');
        Some(header)
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

    let title = primary_symbol
        .map(|s| format!("{} > {s}", ctx.file_path))
        .or_else(|| Some(ctx.file_path.to_string()));

    let chunk_id = ChunkId::generate(
        &ctx.document_id,
        ctx.section_index,
        chunk_index,
        &content_hash,
    );

    let mut metadata = BTreeMap::new();
    if let Some(lang) = ctx.section.language.as_deref() {
        metadata.insert("language".to_string(), lang.to_string());
    }
    if !all_symbols.is_empty() {
        metadata.insert("symbol_count".to_string(), all_symbols.len().to_string());
        let names = all_symbols
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        metadata.insert("symbols".to_string(), names);
    }

    Chunk {
        chunk_id,
        document_id: ctx.document_id,
        section_id: ctx.section.section_id.clone(),
        chunk_index,
        total_chunks: 1, // updated subsequently
        title,
        original_path: ctx.section.original_path.clone(),
        source_url: ctx.section.source_url.clone(),
        line_range: LineRange::new(start_line, end_line),
        byte_range: ByteRange::new(start_byte, end_byte),
        content,
        raw_content,
        context_header,
        content_hash,
        token_count,
        token_estimator: token_counter.estimator_name().to_string(),
        language: ctx.section.language.clone(),
        kind: ChunkKind::CodeAstDeclaration,
        ast_symbols: all_symbols,
        metadata,
    }
}

/// Splits an oversized AST declaration along statement or block lines.
fn split_oversized_ast_declaration(
    ctx: &crate::model::ChunkContext<'_>,
    starting_chunk_idx: usize,
    span: &AstDeclarationSpan,
    config: &ChunkingConfig,
    token_counter: &TokenCounter,
    report: &mut ChunkingReport,
) -> Vec<Chunk> {
    report.oversized_splits_count += 1;

    let primary_symbol = span.symbols.first();
    let sym_name = primary_symbol
        .map(|s| s.name.as_str())
        .unwrap_or("declaration");
    let sym_sig = primary_symbol.map(|s| s.signature.as_str()).unwrap_or("fn");

    let lines: Vec<&str> = span.content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_start_line = span.start_line;

    for (idx, line) in lines.iter().enumerate() {
        current_lines.push(*line);
        let candidate_text = current_lines.join("\n");
        let candidate_tokens = token_counter.count(&candidate_text);

        if candidate_tokens >= config.target_chunk_tokens || idx == lines.len() - 1 {
            let part_idx = chunks.len() + 1;
            let raw_content = current_lines.join("\n");
            let line_count = current_lines.len();
            let chunk_end_line = current_start_line + line_count.saturating_sub(1);

            let context_header = if config.preserve_headers {
                Some(format!(
                    "// File: {} > Symbol: {sym_name} (Part {part_idx})\n// Signature: {sym_sig}\n",
                    ctx.file_path
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
            let chunk_id = ChunkId::generate(
                &ctx.document_id,
                ctx.section_index,
                starting_chunk_idx + chunks.len(),
                &content_hash,
            );

            let mut ast_symbols = Vec::new();
            if let Some(s) = primary_symbol {
                ast_symbols.push(ChunkSymbolMetadata {
                    name: s.name.clone(),
                    kind: s.kind,
                    visibility: s.visibility,
                    signature: s.signature.clone(),
                    doc_comment: s.doc_comment.clone(),
                    parent_scope: None,
                    line_number: current_start_line,
                });
            }

            let mut metadata = BTreeMap::new();
            metadata.insert("is_oversized_split".to_string(), "true".to_string());
            metadata.insert("split_part".to_string(), part_idx.to_string());

            chunks.push(Chunk {
                chunk_id,
                document_id: ctx.document_id,
                section_id: ctx.section.section_id.clone(),
                chunk_index: starting_chunk_idx + chunks.len(),
                total_chunks: 1,
                title: Some(format!("{} > {sym_name} (Part {part_idx})", ctx.file_path)),
                original_path: ctx.section.original_path.clone(),
                source_url: ctx.section.source_url.clone(),
                line_range: LineRange::new(current_start_line as u32, chunk_end_line as u32),
                byte_range: ByteRange::new(0, 0),
                content,
                raw_content,
                context_header,
                content_hash,
                token_count,
                token_estimator: token_counter.estimator_name().to_string(),
                language: ctx.section.language.clone(),
                kind: ChunkKind::CodeAstDeclaration,
                ast_symbols,
                metadata,
            });

            current_lines.clear();
            current_start_line = chunk_end_line + 1;
        }
    }

    chunks
}

/// Fallback chunker for code sections without identified AST symbols.
fn chunk_generic_code_blocks(
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
        .unwrap_or("code_file");

    let mut chunks = Vec::new();
    let mut current_lines = Vec::new();
    let mut start_line = 1;

    for (idx, line) in lines.iter().enumerate() {
        current_lines.push(*line);
        let candidate = current_lines.join("\n");
        let tokens = token_counter.count(&candidate);

        if tokens >= config.target_chunk_tokens || idx == lines.len() - 1 {
            let chunk_index = chunks.len();
            let raw_content = current_lines.join("\n");
            let end_line = start_line + current_lines.len().saturating_sub(1);

            let context_header = if config.preserve_headers {
                Some(format!(
                    "// File: {file_path} (Lines {start_line}-{end_line})\n"
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
                ChunkId::generate(document_id, section_index, chunk_index, &content_hash);

            chunks.push(Chunk {
                chunk_id,
                document_id: *document_id,
                section_id: section.section_id.clone(),
                chunk_index,
                total_chunks: 1,
                title: Some(format!("{file_path} (Lines {start_line}-{end_line})")),
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
                kind: ChunkKind::CodeBlock,
                ast_symbols: Vec::new(),
                metadata: BTreeMap::new(),
            });

            current_lines.clear();
            start_line = end_line + 1;
        }
    }

    let total = chunks.len();
    for c in &mut chunks {
        c.total_chunks = total;
    }

    report.ast_chunks_count += total;
    Ok(chunks)
}
