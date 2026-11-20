# Chunking Crate (`web-repo-to-ai-chunking`)

Provides production-grade RAG (Retrieval-Augmented Generation) document and code chunking, semantic boundary splitting, hierarchical context injection, and token-bounded windowing for the Universal Web & Repository to AI pipeline.

## Features

- **Semantic AST-Boundary Code Chunking (`ast.rs`):**
  - Language-aware declaration span detection powered by `web-repo-to-ai-code-intelligence` for Rust, Python, TypeScript, JavaScript, Go, and generic languages.
  - Groups related symbols (functions, structs, classes, methods, enums, interfaces) within token boundaries.
  - Automatically preserves file-level preambles (imports, type definitions, package headers) across chunk units.
  - Cleanly splits oversized single declarations at internal logical block/statement boundaries with contextual signature headers (`// File: ... > Symbol: <name> (Part X)`).
  - Indentation-aware and brace-depth tracking scope resolvers.

- **Heading-Aware Prose & Markdown Chunking (`markdown.rs`):**
  - Parses GitHub Flavored Markdown heading levels (`#` through `######`).
  - Hierarchical breadcrumb stack tracking (e.g. `Architecture > Storage Engine > LSM Tree`).
  - Injects contextual Markdown breadcrumb headers (`> Context: Root > Section`) on split chunks.
  - Paragraph-boundary preserving split mechanics for oversized prose sections.

- **Structured & Tabular Data Chunking (`structured.rs`):**
  - Table-aware chunking preserving column header rows and delimiter rows (`| --- |`) across **every chunk slice**, preventing column identity loss in vector embeddings.
  - Structured section splitting for OpenAPI specs, JSON Schemas, YAML manifests, and Jupyter notebook cells.

- **Sliding Window with Overlap (`window.rs`):**
  - Line-boundary and token-calibrated rolling window chunking.
  - Configurable target chunk tokens, maximum chunk tokens, and token overlap percentage.
  - Continuity verification across chunk transitions.

- **Token Accounting & Overlap Extraction (`token.rs`):**
  - Integrates `web-repo-to-ai-tokenizer` (`TokenizerRegistry`, BPE tokenizers) with fallback to fast calibrated heuristics.
  - Token truncation and tail-overlap extraction preserving clean newline/word boundaries.

- **Adaptive Engine Orchestration (`engine.rs`):**
  - `Chunker::chunk_document`: Traverses `CdmDocument` sections and routes each section adaptively according to `SectionKind` (`Code`, `Prose`, `Markdown`, `TabularData`, `StructuredSpec`, `Unknown`).
  - `Chunker::chunk_text`: Standalone string chunking with specified strategy or language tag.
  - Mathematical integrity reporting: Aggregates total chunks, tokens, symbol spans, table chunks, and heading chunks in `ChunkingReport`.
  - Bit-for-bit hash determinism using Blake3 chunk identity hashes (`ChunkId`).

## Architecture & Data Model

- `Chunk`: Atomic RAG retrieval unit with:
  - `id`: Deterministic `ChunkId` (`Blake3` hex string derived from section coordinates and raw content).
  - `document_id`: CDM Document foreign key.
  - `section_index`: Source CDM section index.
  - `chunk_index`: 0-based sequential index across the document.
  - `kind`: `ChunkKind` (`CodeAst`, `CodeBlock`, `ProseHeading`, `ProseBlock`, `TabularData`, `StructuredSpec`, `SlidingWindow`, `Generic`).
  - `content`: Primary payload with optional breadcrumb/signature header.
  - `raw_content`: Exact unmodified raw slice from the source document.
  - `token_count`: Token count of `content`.
  - `symbol`: Optional `ChunkSymbolMetadata` (name, kind, signature, start/end line, split part).
  - `context_breadcrumbs`: Hierarchical path list (headings, file path, container symbol).
  - `byte_range` / `line_range`: Source byte and line offsets.

- `ChunkingConfig`: Configurable parameters:
  - `target_chunk_tokens` (default 512)
  - `max_chunk_tokens` (default 1024)
  - `min_chunk_tokens` (default 64)
  - `chunk_overlap_tokens` (default 64)
  - `include_breadcrumbs` (default `true`)
  - `include_context_headers` (default `true`)
  - `preserve_preambles` (default `true`)
  - `tokenizer_name` (default `cl100k_base`)
  - `strategy` (`Adaptive`, `SemanticAst`, `HeadingAware`, `FixedWindowWithOverlap`)

## Verification & Quality

- **100% Passing Test Suite:** 21 unit and integration tests across:
  - `tests/ast_chunking_test.rs`: Rust, Python, TypeScript, Go AST splitting, oversized function statement splitting.
  - `tests/heading_chunking_test.rs`: Heading hierarchy, breadcrumbs, oversized prose splitting.
  - `tests/table_chunking_test.rs`: Multi-chunk table header retention.
  - `tests/sliding_window_test.rs`: Continuity and zero overlap.
  - `tests/engine_document_test.rs`: Adaptive document chunking, bit-for-bit determinism, sub-50ms performance.
- **Strict Compliance:** Zero mocks, zero placeholders, Clippy `-D warnings` clean, `cargo deny` clean.
