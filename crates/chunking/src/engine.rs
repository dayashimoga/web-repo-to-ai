//! # Chunking Engine
//!
//! Orchestrates semantic AST, heading-aware, tabular, and sliding-window chunking
//! across Canonical Context Document Models (`CdmDocument`).

use std::time::Instant;
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};
use web_repo_to_ai_core::DocumentId;

use crate::ast::chunk_code_section;
use crate::error::{ChunkResult, ChunkingError};
use crate::markdown::chunk_markdown_section;
use crate::model::{Chunk, ChunkingConfig, ChunkingReport, ChunkingResult, ChunkingStrategy};
use crate::structured::chunk_structured_section;
use crate::token::TokenCounter;
use crate::window::chunk_sliding_window;

/// Primary orchestrator for document and section chunking.
#[derive(Clone)]
pub struct Chunker {
    config: ChunkingConfig,
    token_counter: TokenCounter,
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(ChunkingConfig::default())
    }
}

impl Chunker {
    /// Creates a new `Chunker` with the given configuration.
    pub fn new(config: ChunkingConfig) -> Self {
        let token_counter = TokenCounter::new(config.tokenizer_name.as_deref());
        Self {
            config,
            token_counter,
        }
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &ChunkingConfig {
        &self.config
    }

    /// Chunks a complete `CdmDocument` into sequential self-contained chunks.
    pub fn chunk_document(&self, doc: &CdmDocument) -> ChunkResult<ChunkingResult> {
        self.config.validate()?;

        if doc.sections.is_empty() {
            return Err(ChunkingError::EmptyDocument(doc.document_id.to_string()));
        }

        let start_time = Instant::now();
        let mut report = ChunkingReport::default();
        let mut all_chunks = Vec::new();

        for (sec_idx, section) in doc.sections.iter().enumerate() {
            let section_chunks =
                self.chunk_section_internal(&doc.document_id, section, sec_idx, &mut report)?;
            all_chunks.extend(section_chunks);
            report.sections_processed += 1;
        }

        // Reassign global sequential chunk indices across the document
        let total_chunks = all_chunks.len();
        for (idx, c) in all_chunks.iter_mut().enumerate() {
            c.chunk_index = idx;
            c.total_chunks = total_chunks;
        }

        report.recalculate_aggregates(&all_chunks);
        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(ChunkingResult {
            document_id: doc.document_id,
            total_chunks,
            total_tokens: report.total_tokens,
            strategy_used: self.config.strategy,
            elapsed_ms,
            chunks: all_chunks,
            report,
        })
    }

    /// Chunks a single `CdmSection`.
    pub fn chunk_section(
        &self,
        document_id: &DocumentId,
        section: &CdmSection,
        section_index: usize,
    ) -> ChunkResult<Vec<Chunk>> {
        self.config.validate()?;
        let mut report = ChunkingReport::default();
        let mut chunks =
            self.chunk_section_internal(document_id, section, section_index, &mut report)?;
        let total = chunks.len();
        for c in &mut chunks {
            c.total_chunks = total;
        }
        Ok(chunks)
    }

    /// Chunks raw text content into a `ChunkingResult`.
    pub fn chunk_text(
        &self,
        text: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> ChunkResult<ChunkingResult> {
        self.config.validate()?;

        if text.trim().is_empty() {
            return Err(ChunkingError::EmptyDocument("raw_text".to_string()));
        }

        let doc_id = DocumentId::new();
        let section = CdmSection {
            section_id: "sec_0".to_string(),
            parent_id: None,
            title: title.map(str::to_string),
            kind: if language.is_some() {
                SectionKind::CodeFile
            } else {
                SectionKind::Prose
            },
            original_path: title.map(str::to_string),
            source_url: "raw://inline".to_string(),
            line_range: None,
            byte_range: None,
            language: language.map(str::to_string),
            content_hash: CdmSection::compute_hash(text),
            body: text.to_string(),
            token_metrics: None,
            attributes: std::collections::BTreeMap::new(),
        };

        let start_time = Instant::now();
        let mut report = ChunkingReport::default();
        let chunks = self.chunk_section_internal(&doc_id, &section, 0, &mut report)?;
        report.sections_processed = 1;
        report.recalculate_aggregates(&chunks);
        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(ChunkingResult {
            document_id: doc_id,
            total_chunks: chunks.len(),
            total_tokens: report.total_tokens,
            strategy_used: self.config.strategy,
            elapsed_ms,
            chunks,
            report,
        })
    }

    /// Internal routing logic selecting the appropriate chunker based on strategy and content kind.
    fn chunk_section_internal(
        &self,
        document_id: &DocumentId,
        section: &CdmSection,
        section_index: usize,
        report: &mut ChunkingReport,
    ) -> ChunkResult<Vec<Chunk>> {
        match self.config.strategy {
            ChunkingStrategy::FixedWindowWithOverlap => chunk_sliding_window(
                document_id,
                section,
                section_index,
                &self.config,
                &self.token_counter,
                report,
            ),
            ChunkingStrategy::SemanticAst => chunk_code_section(
                document_id,
                section,
                section_index,
                &self.config,
                &self.token_counter,
                report,
            ),
            ChunkingStrategy::HeadingAware => chunk_markdown_section(
                document_id,
                section,
                section_index,
                &self.config,
                &self.token_counter,
                report,
            ),
            ChunkingStrategy::Adaptive => match section.kind {
                SectionKind::CodeFile | SectionKind::CodeSnippet => chunk_code_section(
                    document_id,
                    section,
                    section_index,
                    &self.config,
                    &self.token_counter,
                    report,
                ),
                SectionKind::Table => chunk_structured_section(
                    document_id,
                    section,
                    section_index,
                    &self.config,
                    &self.token_counter,
                    report,
                ),
                SectionKind::Prose | SectionKind::Admonition | SectionKind::Navigation => {
                    chunk_markdown_section(
                        document_id,
                        section,
                        section_index,
                        &self.config,
                        &self.token_counter,
                        report,
                    )
                }
                SectionKind::Frontmatter | SectionKind::Manifest => chunk_structured_section(
                    document_id,
                    section,
                    section_index,
                    &self.config,
                    &self.token_counter,
                    report,
                ),
            },
        }
    }
}

/// Convenience function: Chunks a `CdmDocument` using default or custom settings.
pub fn chunk_document(doc: &CdmDocument, config: ChunkingConfig) -> ChunkResult<ChunkingResult> {
    let chunker = Chunker::new(config);
    chunker.chunk_document(doc)
}

/// Convenience function: Chunks raw text using default or custom settings.
pub fn chunk_text(
    text: &str,
    title: Option<&str>,
    language: Option<&str>,
    config: ChunkingConfig,
) -> ChunkResult<ChunkingResult> {
    let chunker = Chunker::new(config);
    chunker.chunk_text(text, title, language)
}
