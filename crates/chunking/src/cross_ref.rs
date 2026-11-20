//! # Cross-Reference Resolution & Metadata Injection
//!
//! Resolves inter-chunk and intra-document cross-references:
//! - Sequence chaining (`prev_chunk_id` and `next_chunk_id`)
//! - Multi-part split declaration parent-child hierarchy
//! - AST symbol call-graph and import dependency edges
//! - Canonical Context Document Model (`CdmDocument`) relationship propagation
//! - Context header injection with citations and sequence indicators

use std::collections::HashMap;

use web_repo_to_ai_cdm::{CdmDocument, RelationshipKind};

use crate::model::{Chunk, ChunkCrossReference, ChunkKind, ChunkingConfig, CrossReferenceKind};
use crate::token::TokenCounter;

/// Engine for resolving and injecting citations and cross-references across chunks.
pub struct CrossReferenceResolver;

impl CrossReferenceResolver {
    /// Resolves all cross-references across document chunks and enriches context headers.
    pub fn resolve_and_inject(
        chunks: &mut [Chunk],
        doc: Option<&CdmDocument>,
        config: &ChunkingConfig,
        token_counter: &TokenCounter,
    ) {
        if chunks.is_empty() {
            return;
        }

        let total_chunks = chunks.len();

        // 1. Sequence chaining (prev_chunk_id & next_chunk_id)
        if config.include_sequence_links {
            Self::chain_sequence_links(chunks, config);
        }

        // 2. Multi-part declaration split hierarchy (parent_chunk_id & SplitPart)
        Self::chain_split_parts(chunks);

        // 3. Intra-document symbol call and import resolution
        if config.include_cross_references {
            Self::resolve_symbol_cross_references(chunks);
        }

        // 4. Propagate explicit CDM document relationships
        if let Some(d) = doc {
            if config.include_cross_references {
                Self::propagate_cdm_relationships(chunks, d);
            }
        }

        // 5. Context header injection & re-tokenization
        if config.preserve_headers {
            Self::inject_context_headers(chunks, total_chunks, config, token_counter);
        }
    }

    /// Chains sequential predecessor and successor links across chunks.
    fn chain_sequence_links(chunks: &mut [Chunk], config: &ChunkingConfig) {
        let total = chunks.len();
        for i in 0..total {
            let prev_id = if i > 0 {
                Some(chunks[i - 1].chunk_id.clone())
            } else {
                None
            };

            let next_id = if i + 1 < total {
                Some(chunks[i + 1].chunk_id.clone())
            } else {
                None
            };

            chunks[i].prev_chunk_id = prev_id.clone();
            chunks[i].next_chunk_id = next_id.clone();

            if config.include_cross_references {
                if let Some(p_id) = prev_id {
                    let prev_chunk = &chunks[i - 1];
                    chunks[i].cross_references.push(ChunkCrossReference {
                        target_chunk_id: Some(p_id),
                        target_section_id: Some(prev_chunk.section_id.clone()),
                        kind: CrossReferenceKind::PreviousChunk,
                        target_symbol: None,
                        target_path: prev_chunk.original_path.clone(),
                        target_url: None,
                        description: Some(format!(
                            "Preceding chunk in document sequence (chunk {})",
                            i - 1
                        )),
                    });
                }

                if let Some(n_id) = next_id {
                    let next_chunk = &chunks[i + 1];
                    chunks[i].cross_references.push(ChunkCrossReference {
                        target_chunk_id: Some(n_id),
                        target_section_id: Some(next_chunk.section_id.clone()),
                        kind: CrossReferenceKind::NextChunk,
                        target_symbol: None,
                        target_path: next_chunk.original_path.clone(),
                        target_url: None,
                        description: Some(format!(
                            "Succeeding chunk in document sequence (chunk {})",
                            i + 1
                        )),
                    });
                }
            }
        }
    }

    /// Detects and links split parts of oversized declarations.
    fn chain_split_parts(chunks: &mut [Chunk]) {
        // Collect indices of chunks that belong to multi-part splits
        // Identified by metadata "split_part" or title ending in "(Part X)"
        let mut split_groups: HashMap<(String, String), Vec<usize>> = HashMap::new();

        for (idx, chunk) in chunks.iter().enumerate() {
            let sym_name = chunk
                .metadata
                .get("symbol_name")
                .cloned()
                .or_else(|| chunk.ast_symbols.first().map(|s| s.name.clone()))
                .or_else(|| {
                    chunk.title.as_ref().and_then(|t| {
                        let pos = t.find(" > ")?;
                        let rest = &t[pos + 3..];
                        rest.find(" (Part ")
                            .map(|part_pos| rest[..part_pos].to_string())
                    })
                });

            let is_split = chunk.metadata.contains_key("split_part")
                || chunk.title.as_deref().unwrap_or("").contains(" (Part ");

            if is_split {
                if let Some(name) = sym_name {
                    let key = (chunk.section_id.clone(), name);
                    split_groups.entry(key).or_default().push(idx);
                }
            }
        }

        // Link split parts: Part 1 is the primary/parent chunk
        for ((_sec_id, sym_name), indices) in split_groups {
            if indices.len() <= 1 {
                continue;
            }

            let primary_idx = indices[0];
            let primary_id = chunks[primary_idx].chunk_id.clone();
            let primary_path = chunks[primary_idx].original_path.clone();
            let primary_sec = chunks[primary_idx].section_id.clone();

            for &child_idx in &indices[1..] {
                chunks[child_idx].parent_chunk_id = Some(primary_id.clone());
                chunks[child_idx]
                    .cross_references
                    .push(ChunkCrossReference {
                        target_chunk_id: Some(primary_id.clone()),
                        target_section_id: Some(primary_sec.clone()),
                        kind: CrossReferenceKind::SplitPart,
                        target_symbol: Some(sym_name.clone()),
                        target_path: primary_path.clone(),
                        target_url: None,
                        description: Some(format!(
                            "Primary declaration signature for split symbol `{sym_name}` (Part 1)"
                        )),
                    });

                // Also link parent to child
                let child_id = chunks[child_idx].chunk_id.clone();
                let child_part = chunks[child_idx]
                    .metadata
                    .get("split_part")
                    .cloned()
                    .unwrap_or_else(|| "N".to_string());
                chunks[primary_idx]
                    .cross_references
                    .push(ChunkCrossReference {
                        target_chunk_id: Some(child_id),
                        target_section_id: Some(primary_sec.clone()),
                        kind: CrossReferenceKind::ChildChunk,
                        target_symbol: Some(sym_name.clone()),
                        target_path: primary_path.clone(),
                        target_url: None,
                        description: Some(format!(
                        "Continuation part of split declaration `{sym_name}` (Part {child_part})"
                    )),
                    });
            }
        }
    }

    /// Resolves intra-document symbol calls and imports between code chunks.
    fn resolve_symbol_cross_references(chunks: &mut [Chunk]) {
        // Build definition map: symbol_name -> (chunk_id, section_id, file_path)
        let mut def_map: HashMap<String, (crate::model::ChunkId, String, Option<String>)> =
            HashMap::new();

        for chunk in chunks.iter() {
            for sym in &chunk.ast_symbols {
                if sym.name.len() >= 2 && !is_common_keyword(&sym.name) {
                    def_map.entry(sym.name.clone()).or_insert_with(|| {
                        (
                            chunk.chunk_id.clone(),
                            chunk.section_id.clone(),
                            chunk.original_path.clone(),
                        )
                    });
                }
            }
        }

        if def_map.is_empty() {
            return;
        }

        // Scan chunks for calls or imports of declared symbols
        for chunk in chunks.iter_mut() {
            let raw = &chunk.raw_content;
            let mut matched_refs = Vec::new();

            for (sym_name, (target_chunk_id, target_sec_id, target_path)) in &def_map {
                if target_chunk_id == &chunk.chunk_id {
                    continue;
                }

                if contains_word(raw, sym_name) {
                    let kind = classify_symbol_relationship(raw, sym_name);
                    matched_refs.push(ChunkCrossReference {
                        target_chunk_id: Some(target_chunk_id.clone()),
                        target_section_id: Some(target_sec_id.clone()),
                        kind,
                        target_symbol: Some(sym_name.clone()),
                        target_path: target_path.clone(),
                        target_url: None,
                        description: Some(format!(
                            "{} symbol `{sym_name}` defined in {}",
                            kind.as_str(),
                            target_path.as_deref().unwrap_or("document")
                        )),
                    });

                    // Cap maximum auto-discovered symbol cross references per chunk to 8
                    if matched_refs.len() >= 8 {
                        break;
                    }
                }
            }

            chunk.cross_references.extend(matched_refs);
        }
    }

    /// Maps explicit relationships from `CdmDocument::relationships` to corresponding chunks.
    fn propagate_cdm_relationships(chunks: &mut [Chunk], doc: &CdmDocument) {
        if doc.relationships.is_empty() {
            return;
        }

        let mut section_map: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, c) in chunks.iter().enumerate() {
            section_map
                .entry(c.section_id.clone())
                .or_default()
                .push(idx);
        }

        for rel in &doc.relationships {
            let src_indices = section_map.get(&rel.source_id).cloned();
            let tgt_indices = section_map.get(&rel.target_id).cloned();

            let kind = map_cdm_relationship_kind(rel.kind);

            let target_info = tgt_indices
                .and_then(|list| list.first().copied())
                .map(|idx| {
                    (
                        chunks[idx].chunk_id.clone(),
                        chunks[idx].section_id.clone(),
                        chunks[idx].original_path.clone(),
                    )
                });

            if let Some(src_list) = src_indices {
                for src_idx in src_list {
                    if let Some((ref tgt_chunk_id, ref tgt_sec_id, ref tgt_path)) = target_info {
                        chunks[src_idx].cross_references.push(ChunkCrossReference {
                            target_chunk_id: Some(tgt_chunk_id.clone()),
                            target_section_id: Some(tgt_sec_id.clone()),
                            kind,
                            target_symbol: None,
                            target_path: tgt_path.clone(),
                            target_url: None,
                            description: rel.description.clone(),
                        });
                    } else {
                        // External relationship target URL
                        chunks[src_idx].cross_references.push(ChunkCrossReference {
                            target_chunk_id: None,
                            target_section_id: None,
                            kind,
                            target_symbol: None,
                            target_path: None,
                            target_url: Some(rel.target_id.clone()),
                            description: rel.description.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Injects enriched context headers containing citations, sequence markers, and cross-references.
    fn inject_context_headers(
        chunks: &mut [Chunk],
        total_chunks: usize,
        config: &ChunkingConfig,
        token_counter: &TokenCounter,
    ) {
        for chunk in chunks.iter_mut() {
            let is_code = matches!(
                chunk.kind,
                ChunkKind::CodeAstDeclaration | ChunkKind::CodeBlock
            );

            let mut header_lines = Vec::new();

            // 1. Existing context header or title
            if let Some(ref existing) = chunk.context_header {
                header_lines.push(existing.clone());
            } else if let Some(ref title) = chunk.title {
                if is_code {
                    header_lines.push(format!("// File: {title}"));
                } else {
                    header_lines.push(format!("> Context: {title}"));
                }
            }

            // 2. Citation line
            if config.include_citations {
                if let Some(ref cit) = chunk.citation {
                    if is_code {
                        header_lines.push(format!("// Source: {}", cit.markdown_citation));
                    } else {
                        header_lines.push(format!("> Source: {}", cit.markdown_citation));
                    }
                }
            }

            // 3. Sequence indicators
            if config.include_sequence_links {
                let prev_label = chunk.prev_chunk_id.as_ref().map_or("None", |p| p.as_str());
                let next_label = chunk.next_chunk_id.as_ref().map_or("None", |n| n.as_str());

                let seq_text = format!(
                    "Sequence: Chunk {} of {} (Prev: {}, Next: {})",
                    chunk.chunk_index + 1,
                    total_chunks,
                    prev_label,
                    next_label
                );

                if is_code {
                    header_lines.push(format!("// {seq_text}"));
                } else {
                    header_lines.push(format!("> {seq_text}"));
                }
            }

            // 4. Cross-references summary
            if config.include_cross_references {
                let relevant_refs: Vec<_> = chunk
                    .cross_references
                    .iter()
                    .filter(|r| {
                        !matches!(
                            r.kind,
                            CrossReferenceKind::PreviousChunk | CrossReferenceKind::NextChunk
                        )
                    })
                    .filter_map(|r| {
                        if let Some(ref sym) = r.target_symbol {
                            Some(format!("{} `{sym}`", r.kind.as_str()))
                        } else if let Some(ref p) = r.target_path {
                            Some(format!("{} in {p}", r.kind.as_str()))
                        } else {
                            r.description.clone()
                        }
                    })
                    .collect();

                if !relevant_refs.is_empty() {
                    let summary = relevant_refs.join(", ");
                    if is_code {
                        header_lines.push(format!("// References: {summary}"));
                    } else {
                        header_lines.push(format!("> References: {summary}"));
                    }
                }
            }

            if !header_lines.is_empty() {
                let full_header = header_lines.join("\n");
                chunk.content = format!("{full_header}\n\n{}", chunk.raw_content);
                chunk.context_header = Some(full_header);
                chunk.content_hash = Chunk::compute_hash(&chunk.content);
                chunk.token_count = token_counter.count(&chunk.content);
            }
        }
    }
}

/// Helper to classify whether a symbol usage in source text represents an import or call.
fn classify_symbol_relationship(text: &str, symbol: &str) -> CrossReferenceKind {
    for line in text.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("require("))
            && trimmed.contains(symbol)
        {
            return CrossReferenceKind::Imports;
        }
    }

    let call_pattern_1 = format!("{symbol}(");
    let call_pattern_2 = format!("{symbol}::");
    if text.contains(&call_pattern_1) || text.contains(&call_pattern_2) {
        CrossReferenceKind::Calls
    } else {
        CrossReferenceKind::References
    }
}

/// Helper checking if `text` contains `word` with boundary delimiters.
fn contains_word(text: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(idx) = text[start..].find(word) {
        let actual_idx = start + idx;
        let end_idx = actual_idx + word.len();

        let is_left_boundary = if actual_idx == 0 {
            true
        } else {
            let prev = text[..actual_idx].chars().next_back().unwrap();
            !prev.is_alphanumeric() && prev != '_'
        };

        let is_right_boundary = if end_idx >= text.len() {
            true
        } else {
            let next = text[end_idx..].chars().next().unwrap();
            !next.is_alphanumeric() && next != '_'
        };

        if is_left_boundary && is_right_boundary {
            return true;
        }

        start = end_idx;
    }

    false
}

/// Helper filtering out common programming language keywords.
fn is_common_keyword(name: &str) -> bool {
    matches!(
        name,
        "if" | "else"
            | "for"
            | "while"
            | "loop"
            | "match"
            | "let"
            | "mut"
            | "const"
            | "static"
            | "pub"
            | "fn"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "type"
            | "mod"
            | "use"
            | "self"
            | "Self"
            | "def"
            | "class"
            | "return"
            | "yield"
            | "async"
            | "await"
            | "function"
            | "var"
            | "package"
            | "interface"
            | "new"
            | "true"
            | "false"
            | "nil"
            | "null"
            | "None"
    )
}

/// Maps CDM relationship kinds to chunk cross-reference kinds.
fn map_cdm_relationship_kind(cdm_kind: RelationshipKind) -> CrossReferenceKind {
    match cdm_kind {
        RelationshipKind::Imports => CrossReferenceKind::Imports,
        RelationshipKind::ImportedBy => CrossReferenceKind::References,
        RelationshipKind::Calls => CrossReferenceKind::Calls,
        RelationshipKind::Extends => CrossReferenceKind::Extends,
        RelationshipKind::LinksTo => CrossReferenceKind::LinksTo,
        RelationshipKind::References => CrossReferenceKind::References,
        RelationshipKind::ParentOf => CrossReferenceKind::ChildChunk,
        RelationshipKind::ChildOf => CrossReferenceKind::ParentScope,
        RelationshipKind::DefinesSymbol => CrossReferenceKind::DefinesSymbol,
        RelationshipKind::DuplicateOf => CrossReferenceKind::References,
    }
}
