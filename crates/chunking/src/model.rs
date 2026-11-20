//! # Chunking Domain Models
//!
//! Defines chunks, configuration options, strategies, and summary reports.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{ByteRange, LineRange};
use web_repo_to_ai_code_intelligence::{SymbolKind, Visibility};
use web_repo_to_ai_core::DocumentId;

use crate::error::{ChunkResult, ChunkingError};

/// Unique identifier for a chunk within a document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChunkId(pub String);

impl ChunkId {
    /// Constructs a new `ChunkId`.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generates a deterministic chunk ID from document ID, section index, chunk index, and content hash.
    #[must_use]
    pub fn generate(
        document_id: &DocumentId,
        section_index: usize,
        chunk_index: usize,
        content_hash: &str,
    ) -> Self {
        let doc_str = document_id.to_string();
        let doc_prefix = doc_str.split('-').next().unwrap_or("doc");
        let hash_prefix = if content_hash.len() >= 8 {
            &content_hash[..8]
        } else {
            content_hash
        };
        Self(format!(
            "chk_{doc_prefix}_s{section_index}_c{chunk_index}_{hash_prefix}"
        ))
    }

    /// Returns the chunk ID string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ChunkId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ChunkId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Structural category of a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkKind {
    /// High-level AST declaration (function, method, struct, class, interface, trait, enum).
    CodeAstDeclaration,
    /// Generic code snippet or statement block.
    CodeBlock,
    /// Markdown documentation or prose section under a heading.
    MarkdownSection,
    /// Generic text paragraph or prose slice.
    Paragraph,
    /// Tabular data slice with preserved column headers.
    Table,
    /// Structured document component (OpenAPI path/operation, JSON Schema definition, etc.).
    StructuredSpec,
    /// Jupyter Notebook cell (markdown, code, or execution output).
    NotebookCell,
    /// Fallback fixed/sliding window chunk.
    SlidingWindow,
}

impl ChunkKind {
    /// Returns the snake_case label for this chunk kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CodeAstDeclaration => "code_ast_declaration",
            Self::CodeBlock => "code_block",
            Self::MarkdownSection => "markdown_section",
            Self::Paragraph => "paragraph",
            Self::Table => "table",
            Self::StructuredSpec => "structured_spec",
            Self::NotebookCell => "notebook_cell",
            Self::SlidingWindow => "sliding_window",
        }
    }
}

/// Symbol metadata associated with an AST-derived code chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkSymbolMetadata {
    /// Name of the symbol (e.g. `process_data` or `UserRecord`).
    pub name: String,
    /// Kind of declaration.
    pub kind: SymbolKind,
    /// Visibility scope.
    pub visibility: Visibility,
    /// Complete signature line.
    pub signature: String,
    /// Optional preceding doc comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<String>,
    /// Optional enclosing scope or parent class/module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_scope: Option<String>,
    /// 1-based start line of the declaration.
    pub line_number: usize,
}

/// Authoritative citation anchoring a chunk to its remote or local source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkCitation {
    /// Unique citation identifier (e.g. `cit_chk_s0_c1`).
    pub citation_id: String,
    /// Canonical source URL (e.g. `https://github.com/owner/repo` or `https://example.com/docs`).
    pub source_url: String,
    /// Precise permalink including anchor or line numbers
    /// (e.g. `https://github.com/owner/repo/blob/abc1234/src/lib.rs#L10-L45` or `https://example.com/docs#installation`).
    pub permalink: String,
    /// Compact human-readable citation label (e.g. `src/lib.rs:10-45 (Commit abc1234)` or `User Guide > Installation`).
    pub display_citation: String,
    /// Rendered Markdown citation link (e.g. `[src/lib.rs#L10-L45](https://github.com/owner/repo/blob/abc1234/src/lib.rs#L10-L45)`).
    pub markdown_citation: String,
    /// Relative repository or document file path if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// 1-indexed (start_line, end_line) if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_span: Option<(usize, usize)>,
    /// Git commit SHA or revision hash if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    /// Document or page title if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_title: Option<String>,
    /// Target anchor fragment without `#` (e.g. `L10-L45` or `installation`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}

/// Nature of the cross-reference connection between chunks or entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossReferenceKind {
    /// Immediate sequential predecessor chunk in reading/file order.
    PreviousChunk,
    /// Immediate sequential successor chunk in reading/file order.
    NextChunk,
    /// Enclosing structural parent (e.g. class enclosing method, parent heading).
    ParentScope,
    /// Contained child member or nested section.
    ChildChunk,
    /// Part of the same oversized declaration or section.
    SplitPart,
    /// Source chunk imports or depends on target module/symbol.
    Imports,
    /// Source chunk invokes a function or method defined in the target.
    Calls,
    /// Source chunk defines a symbol that is referenced elsewhere.
    DefinesSymbol,
    /// Source type inherits from or implements target type.
    Extends,
    /// Semantic reference, citation, or mention.
    References,
    /// Hyperlink navigation link.
    LinksTo,
}

impl CrossReferenceKind {
    /// Returns the snake_case label for this cross-reference kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreviousChunk => "previous_chunk",
            Self::NextChunk => "next_chunk",
            Self::ParentScope => "parent_scope",
            Self::ChildChunk => "child_chunk",
            Self::SplitPart => "split_part",
            Self::Imports => "imports",
            Self::Calls => "calls",
            Self::DefinesSymbol => "defines_symbol",
            Self::Extends => "extends",
            Self::References => "references",
            Self::LinksTo => "links_to",
        }
    }
}

/// Semantic cross-reference or graph edge connecting a chunk to other chunks or entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkCrossReference {
    /// Target chunk identifier if resolved within the document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_chunk_id: Option<ChunkId>,
    /// Target CDM section identifier if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_section_id: Option<String>,
    /// Semantic nature of the relationship.
    pub kind: CrossReferenceKind,
    /// Target symbol name (e.g. `TokenCounter` or `chunk_document`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    /// Target file path or document route.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
    /// External destination URL if linking to an external resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    /// Human/LLM-readable description of the relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ChunkCrossReference {
    /// Constructs a basic cross-reference to a target chunk.
    #[must_use]
    pub fn new(target_chunk_id: ChunkId, kind: CrossReferenceKind) -> Self {
        Self {
            target_chunk_id: Some(target_chunk_id),
            target_section_id: None,
            kind,
            target_symbol: None,
            target_path: None,
            target_url: None,
            description: None,
        }
    }
}

/// Self-contained semantic chunk produced by the chunking engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier for this chunk.
    pub chunk_id: ChunkId,
    /// Parent document identifier.
    pub document_id: DocumentId,
    /// Parent section identifier from CDM.
    pub section_id: String,
    /// 0-indexed position of this chunk within the section or document.
    pub chunk_index: usize,
    /// Total number of chunks generated for this section or document.
    pub total_chunks: usize,
    /// Descriptive title or heading hierarchy for the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Relative file path or DOM selector of the source content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_path: Option<String>,
    /// Canonical source URL or permalink.
    pub source_url: String,
    /// 1-indexed line span of the original content covered by this chunk.
    pub line_range: LineRange,
    /// Byte range of the original content covered by this chunk.
    pub byte_range: ByteRange,
    /// Complete chunk text content (including prepended context header if enabled).
    pub content: String,
    /// Unaugmented body text without context header.
    pub raw_content: String,
    /// Optional context header prepended to content for self-contained RAG retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_header: Option<String>,
    /// BLAKE3 cryptographic hex digest of the full `content`.
    pub content_hash: String,
    /// Token count for this chunk.
    pub token_count: usize,
    /// Name of the tokenizer or estimator used (`exact` or `estimated`).
    pub token_estimator: String,
    /// Programming or markup language identifier if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Categorization of the chunk.
    pub kind: ChunkKind,
    /// Associated AST symbol declarations if this chunk is a code declaration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ast_symbols: Vec<ChunkSymbolMetadata>,
    /// Additional chunk-level metadata attributes.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
    /// Authoritative source citation with permalink, line numbers, and markdown link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<ChunkCitation>,
    /// Outgoing cross-references to sequential chunks, parent scopes, and referenced symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_references: Vec<ChunkCrossReference>,
    /// Immediate predecessor chunk ID in sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_chunk_id: Option<ChunkId>,
    /// Immediate successor chunk ID in sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_chunk_id: Option<ChunkId>,
    /// Enclosing structural parent chunk ID (e.g. class or parent heading).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_chunk_id: Option<ChunkId>,
}

impl Chunk {
    /// Computes the canonical BLAKE3 content hash for a chunk's content.
    #[must_use]
    pub fn compute_hash(text: &str) -> String {
        blake3::hash(text.as_bytes()).to_hex().to_string()
    }
}

/// Strategy for partitioning document sections into chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Automatically selects `SemanticAst` for source code, `HeadingAware` for prose/docs,
    /// and tabular preservation for tables.
    #[default]
    Adaptive,
    /// Splits strictly along AST symbol boundaries (functions, structs, classes, traits).
    SemanticAst,
    /// Splits strictly along Markdown heading hierarchies and paragraph boundaries.
    HeadingAware,
    /// Sliding window with fixed token target and configurable overlap.
    FixedWindowWithOverlap,
}

impl ChunkingStrategy {
    /// Returns the snake_case label of the strategy.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::SemanticAst => "semantic_ast",
            Self::HeadingAware => "heading_aware",
            Self::FixedWindowWithOverlap => "fixed_window_with_overlap",
        }
    }
}

/// Configuration options for the chunking engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Primary chunking strategy.
    pub strategy: ChunkingStrategy,
    /// Desired target token count per chunk (default 512).
    pub target_chunk_tokens: usize,
    /// Hard upper token limit for a single chunk (default 1024).
    pub max_chunk_tokens: usize,
    /// Minimum token threshold below which small chunks are coalesced (default 64).
    pub min_chunk_tokens: usize,
    /// Overlap tokens between consecutive sliding chunks (default 64).
    pub overlap_tokens: usize,
    /// Whether to prepend breadcrumbs/file/symbol context headers to each chunk.
    pub preserve_headers: bool,
    /// Whether prepended context headers are counted within `max_chunk_tokens`.
    pub header_in_token_budget: bool,
    /// Whether to generate and inject authoritative source citations into chunks.
    pub include_citations: bool,
    /// Whether to discover and resolve intra-document symbol and section cross-references.
    pub include_cross_references: bool,
    /// Whether to chain sequential chunks via prev/next chunk IDs.
    pub include_sequence_links: bool,
    /// Optional target model architecture or tokenizer name (e.g. `gpt-4o`, `claude-3-5-sonnet`).
    pub tokenizer_name: Option<String>,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: ChunkingStrategy::Adaptive,
            target_chunk_tokens: 512,
            max_chunk_tokens: 1024,
            min_chunk_tokens: 64,
            overlap_tokens: 64,
            preserve_headers: true,
            header_in_token_budget: true,
            include_citations: true,
            include_cross_references: true,
            include_sequence_links: true,
            tokenizer_name: None,
        }
    }
}

impl ChunkingConfig {
    /// Validates the configuration parameters.
    pub fn validate(&self) -> ChunkResult<()> {
        if self.max_chunk_tokens == 0 {
            return Err(ChunkingError::InvalidConfig(
                "max_chunk_tokens must be greater than 0".to_string(),
            ));
        }
        if self.target_chunk_tokens == 0 {
            return Err(ChunkingError::InvalidConfig(
                "target_chunk_tokens must be greater than 0".to_string(),
            ));
        }
        if self.target_chunk_tokens > self.max_chunk_tokens {
            return Err(ChunkingError::InvalidConfig(format!(
                "target_chunk_tokens ({}) cannot exceed max_chunk_tokens ({})",
                self.target_chunk_tokens, self.max_chunk_tokens
            )));
        }
        if self.min_chunk_tokens > self.target_chunk_tokens {
            return Err(ChunkingError::InvalidConfig(format!(
                "min_chunk_tokens ({}) cannot exceed target_chunk_tokens ({})",
                self.min_chunk_tokens, self.target_chunk_tokens
            )));
        }
        if self.overlap_tokens >= self.max_chunk_tokens {
            return Err(ChunkingError::InvalidConfig(format!(
                "overlap_tokens ({}) must be strictly less than max_chunk_tokens ({})",
                self.overlap_tokens, self.max_chunk_tokens
            )));
        }
        Ok(())
    }

    /// Builder: sets target chunk tokens.
    #[must_use]
    pub fn with_target_tokens(mut self, tokens: usize) -> Self {
        self.target_chunk_tokens = tokens;
        self
    }

    /// Builder: sets max chunk tokens.
    #[must_use]
    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_chunk_tokens = tokens;
        self
    }

    /// Builder: sets min chunk tokens.
    #[must_use]
    pub fn with_min_tokens(mut self, tokens: usize) -> Self {
        self.min_chunk_tokens = tokens;
        self
    }

    /// Builder: sets overlap tokens.
    #[must_use]
    pub fn with_overlap_tokens(mut self, tokens: usize) -> Self {
        self.overlap_tokens = tokens;
        self
    }

    /// Builder: sets chunking strategy.
    #[must_use]
    pub fn with_strategy(mut self, strategy: ChunkingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Builder: sets header preservation flag.
    #[must_use]
    pub fn with_preserve_headers(mut self, preserve: bool) -> Self {
        self.preserve_headers = preserve;
        self
    }

    /// Builder: sets whether citations are generated and injected.
    #[must_use]
    pub fn with_citations(mut self, enabled: bool) -> Self {
        self.include_citations = enabled;
        self
    }

    /// Builder: sets whether cross-references are discovered and resolved.
    #[must_use]
    pub fn with_cross_references(mut self, enabled: bool) -> Self {
        self.include_cross_references = enabled;
        self
    }

    /// Builder: sets whether sequential chunks are chained.
    #[must_use]
    pub fn with_sequence_links(mut self, enabled: bool) -> Self {
        self.include_sequence_links = enabled;
        self
    }

    /// Builder: sets tokenizer name.
    #[must_use]
    pub fn with_tokenizer(mut self, tokenizer: impl Into<String>) -> Self {
        self.tokenizer_name = Some(tokenizer.into());
        self
    }
}

/// Quantitative telemetry and summary metrics for a chunking execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkingReport {
    /// Number of CDM sections processed.
    pub sections_processed: usize,
    /// Total number of chunks generated.
    pub chunks_generated: usize,
    /// Total token count across all chunks.
    pub total_tokens: usize,
    /// Minimum token count observed in a chunk.
    pub min_tokens: usize,
    /// Maximum token count observed in a chunk.
    pub max_tokens: usize,
    /// Mean average token count per chunk.
    pub avg_tokens: f64,
    /// Number of chunks created via AST declaration splitting.
    pub ast_chunks_count: usize,
    /// Number of chunks created via heading hierarchy splitting.
    pub heading_chunks_count: usize,
    /// Number of chunks created via tabular splitting.
    pub table_chunks_count: usize,
    /// Number of chunks created via sliding window fallback.
    pub window_chunks_count: usize,
    /// Number of oversized elements that required sub-splitting.
    pub oversized_splits_count: usize,
    /// Total number of citations generated.
    pub citations_count: usize,
    /// Total number of cross-references resolved.
    pub cross_references_count: usize,
}

impl Default for ChunkingReport {
    fn default() -> Self {
        Self {
            sections_processed: 0,
            chunks_generated: 0,
            total_tokens: 0,
            min_tokens: 0,
            max_tokens: 0,
            avg_tokens: 0.0,
            ast_chunks_count: 0,
            heading_chunks_count: 0,
            table_chunks_count: 0,
            window_chunks_count: 0,
            oversized_splits_count: 0,
            citations_count: 0,
            cross_references_count: 0,
        }
    }
}

impl ChunkingReport {
    /// Recomputes summary aggregates (min, max, avg, citations, cross-refs) from a list of chunks.
    pub fn recalculate_aggregates(&mut self, chunks: &[Chunk]) {
        self.chunks_generated = chunks.len();
        self.total_tokens = chunks.iter().map(|c| c.token_count).sum();
        self.min_tokens = chunks.iter().map(|c| c.token_count).min().unwrap_or(0);
        self.max_tokens = chunks.iter().map(|c| c.token_count).max().unwrap_or(0);
        self.avg_tokens = if self.chunks_generated > 0 {
            self.total_tokens as f64 / self.chunks_generated as f64
        } else {
            0.0
        };
        self.citations_count = chunks.iter().filter(|c| c.citation.is_some()).count();
        self.cross_references_count = chunks.iter().map(|c| c.cross_references.len()).sum();
    }
}

/// Contextual coordinate information for the section currently being chunked.
#[derive(Debug, Clone, Copy)]
pub struct ChunkContext<'a> {
    /// Identifier of the parent document.
    pub document_id: DocumentId,
    /// Reference to the source CDM section.
    pub section: &'a web_repo_to_ai_cdm::CdmSection,
    /// Index of the section within the document.
    pub section_index: usize,
    /// Canonical file path or title for breadcrumb headers.
    pub file_path: &'a str,
}

impl<'a> ChunkContext<'a> {
    /// Creates a new `ChunkContext`.
    #[must_use]
    pub fn new(
        document_id: DocumentId,
        section: &'a web_repo_to_ai_cdm::CdmSection,
        section_index: usize,
        file_path: &'a str,
    ) -> Self {
        Self {
            document_id,
            section,
            section_index,
            file_path,
        }
    }
}

/// The final outcome of a document chunking execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkingResult {
    /// Identifier of the parent document.
    pub document_id: DocumentId,
    /// The generated chunks in sequential reading order.
    pub chunks: Vec<Chunk>,
    /// Total number of chunks.
    pub total_chunks: usize,
    /// Total tokens across all chunks.
    pub total_tokens: usize,
    /// Strategy applied during chunking.
    pub strategy_used: ChunkingStrategy,
    /// Wall-clock execution time in milliseconds.
    pub elapsed_ms: u64,
    /// Quantitative telemetry report.
    pub report: ChunkingReport,
}
