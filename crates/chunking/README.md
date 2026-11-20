# Chunking Crate (`web-repo-to-ai-chunking`)

Provides production-grade RAG (Retrieval-Augmented Generation) document and code chunking, semantic boundary splitting, hierarchical context injection, citation anchoring, and token-bounded windowing for the Universal Web & Repository to AI pipeline.

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

- **Authoritative Citation & Provenance Anchoring (`citation.rs`):**
  - Generates exact remote permalinks with commit hashes and line spans for GitHub repositories:
    `https://github.com/{owner}/{repo}/blob/{commit_sha}/{path}#L{start}-L{end}`
  - Generates heading slug anchors for web documentation:
    `https://docs.rs/tokio/1.38.0/tokio/index.html#asynchronous-runtime`
  - Generates PDF page-level citations:
    `https://arxiv.org/pdf/1706.03762.pdf#page=4` (`(p. 4)`)
  - Generates structured spec pointers (e.g. `docs/openapi.yaml > GET /api/v1/jobs` with anchor `#GET--api-v1-jobs`) and Jupyter notebook cell pointers.
  - Formats ready-to-render Markdown citation links (`markdown_citation`) and compact human-readable labels (`display_citation`).

- **Cross-Reference Resolution & Injection Engine (`cross_ref.rs`):**
  - Sequential chunk chaining: Automatically connects predecessor and successor chunks via `prev_chunk_id` and `next_chunk_id` with bidirectional graph edges.
  - Split-part parent-child hierarchy: Connects split parts of oversized declarations with `parent_chunk_id`, `CrossReferenceKind::SplitPart`, and `CrossReferenceKind::ChildChunk`.
  - Intra-document symbol resolution: Analyzes symbol definitions and references across code chunks, resolving function calls (`CrossReferenceKind::Calls`) and module imports (`CrossReferenceKind::Imports`) directly to their declaring chunk IDs.
  - Canonical Context Document Model (`CdmDocument`) relationship propagation: Maps explicit `Relationship` graph edges (`Imports`, `Calls`, `Extends`, `LinksTo`, `References`) directly onto retrieved chunks.
  - Injected context header formatting: Enriches chunk content headers with authoritative citations, sequence positions (`Chunk 1 of 5 (Prev: None, Next: chk_...)`), and summarized references.

- **JSONL Streaming & Vector Ingest Schemas (`export/`):**
  - **`ChunkExporter`**: High-performance streaming serializer supporting writing directly to any `std::io::Write` destination without full-buffer overhead.
  - **`VectorTarget` Adapters**:
    - `GenericJsonl`: Canonical newline-delimited RAG records with complete chunk coordinates, citations, and graph edges.
    - `Qdrant`: Point structure with `id` (deterministic RFC 4122 UUID or string), optional dense vector, and rich metadata `payload`. Supports both JSONL points and `{"points": [...]}` batch envelopes.
    - `Pinecone`: Upsert vector structure with strict metadata validation (flattening to scalar strings, numbers, booleans, and string arrays; zero nulls or nested objects). Supports both JSONL and `{"vectors": [...]}` batch envelopes.
    - `Chroma`: Document record structure with `document`, `metadata`, and collection batch representations (`ids`, `documents`, `metadatas`).
    - `Milvus`: Relational row entities with coordinate fields, hashes, and source URLs.
    - `Weaviate`: Data object format with mandatory RFC 4122 UUIDs, class names (default `"DocumentChunk"`), and properties payload. Supports `{"objects": [...]}` batch envelopes.
    - `OpenAiBatch`: Format for OpenAI Batch Embeddings API (`POST /v1/embeddings`) with `custom_id`, model specification, and input text.
    - `CohereBatch`: Record format for Cohere Embed endpoint.
  - **Deterministic RFC 4122 UUID Generator**: `to_deterministic_uuid` converts any chunk ID or Blake3 hash into a valid Version 4, Variant 1 RFC 4122 UUID with 100% platform reproducibility.
  - **`VectorSchemaValidator`**: Proactively verifies chunk payloads against database constraints (empty ID/content rejection, Weaviate uppercase class check, Pinecone metadata type compliance, OpenAI context limits).

- **Golden Benchmark Evaluation Corpus & Determinism Engine (`golden.rs`):**
  - **8 Real-World Archetype Suites**:
    1. `MultiLanguageAstCode`: Polyglot codebase (Rust, Python, TypeScript, Go) with language-specific AST symbols, preambles, and signatures.
    2. `HierarchicalMarkdownDocumentation`: Deep technical documentation with `#` through `######` headings, nested lists, and contextual breadcrumbs.
    3. `TabularDataMatrix`: CSV/TSV table slices validating persistent column header row retention (`| --- |`) across split chunk slices.
    4. `StructuredSpecOpenApi`: OpenAPI 3.1.0 REST API specification with path operations, schemas, and endpoints.
    5. `JupyterNotebookPipeline`: Interactive nbformat v4 data science pipeline with code and markdown cells.
    6. `OversizedDeclarationRoutines`: Giant functions exceeding token budgets split at statement/block boundaries with contextual signature headers (`// File: ... > Symbol: ... (Part X)`).
    7. `CrossReferenceGraphCorpus`: Interconnected code sections verifying symbol call and import graph edge resolution.
    8. `AdversarialChunkBounds`: Extreme token limits, multi-byte Unicode emojis, mathematical symbols, and box drawing characters.
  - **`ChunkingFidelityMetrics`**: Computes total chunks, total tokens, average tokens, min/max tokens, bit-for-bit repeatability ($H_1 == H_2 == H_3$), boundary line/byte integrity, citation validity ($\ge 99\%$), cross-references count, simulated Top-1 retrieval recall ($\ge 99\%$), and sub-50ms execution latency.
  - **Simulated Top-1 Retrieval Engine**: Deterministic keyword, exact phrase, symbol match, and heading weight scorer (`score_chunk_relevance`, `retrieve_top_k`) validating query relevancy and recall across all archetypes.
  - **Bit-for-Bit Hash Determinism**: Verifies that 10 sequential iterations and 8 concurrent parallel threads generate 100% identical Blake3 chunk hashes, token counts, and coordinate boundaries.
  - **Graph Continuity Invariants**: Enforces bidirectional chaining symmetry (`chunks[i].next_chunk_id == chunks[i+1].id && chunks[i+1].prev_chunk_id == chunks[i].id`) and strict byte/line coordinate monotonicity.

- **Sliding Window with Overlap (`window.rs`):**
  - Line-boundary and token-calibrated rolling window chunking.
  - Configurable target chunk tokens, maximum chunk tokens, and token overlap percentage.
  - Continuity verification across chunk transitions.

- **Token Accounting & Overlap Extraction (`token.rs`):**
  - Integrates `web-repo-to-ai-tokenizer` (`TokenizerRegistry`, BPE tokenizers) with fallback to fast calibrated heuristics.
  - Token truncation and tail-overlap extraction preserving clean newline/word boundaries.

- **Adaptive Engine Orchestration (`engine.rs`):**
  - `Chunker::chunk_document`: Traverses `CdmDocument` sections, routes each section adaptively, enriches citations, and resolves cross-references across all document chunks.
  - `Chunker::chunk_section`: Chunks a single CDM section with sequential chaining and citation injection.
  - `Chunker::chunk_text`: Standalone string chunking with specified strategy or language tag.
  - Mathematical integrity reporting: Aggregates total chunks, tokens, symbol spans, table chunks, heading chunks, citations count, and cross-references count in `ChunkingReport`.
  - Bit-for-bit hash determinism using Blake3 chunk identity hashes (`ChunkId`).

## Architecture & Data Model

- `Chunk`: Atomic RAG retrieval unit with:
  - `chunk_id`: Deterministic `ChunkId` (`Blake3` hex string derived from section coordinates and content hash).
  - `document_id`: CDM Document foreign key.
  - `section_id`: Source CDM section index.
  - `chunk_index`: 0-based sequential index across the document.
  - `total_chunks`: Total chunks in the document or section.
  - `kind`: `ChunkKind` (`CodeAstDeclaration`, `CodeBlock`, `MarkdownSection`, `Paragraph`, `Table`, `StructuredSpec`, `NotebookCell`, `SlidingWindow`).
  - `content`: Enriched chunk payload with prepended context header (if enabled).
  - `raw_content`: Exact unmodified raw slice from the source document.
  - `context_header`: Optional prepended header with citation, sequence, and references.
  - `citation`: Optional `ChunkCitation` with permalink, line span, commit SHA, and markdown link.
  - `cross_references`: Array of `ChunkCrossReference` connecting chunks to symbols, sequential chunks, and related sections.
  - `prev_chunk_id` / `next_chunk_id`: Direct sequential chaining pointers.
  - `parent_chunk_id`: Parent container or primary split declaration pointer.
  - `token_count`: Token count of `content`.
  - `ast_symbols`: Array of `ChunkSymbolMetadata` (name, kind, visibility, signature, doc comment, line number).
  - `line_range` / `byte_range`: Source coordinate spans.

- `ChunkCitation`:
  - `citation_id`: Unique identifier.
  - `source_url`: Canonical remote URL or file path.
  - `permalink`: Exact permalink with commit SHA and line range (`#Lstart-Lend`) or anchor.
  - `display_citation`: Compact citation label (`path:start-end (short_sha)`).
  - `markdown_citation`: Formatted Markdown link (`[path#Lstart-Lend](permalink)`).
  - `line_span`: `(start_line, end_line)`.
  - `commit_sha`: Git commit hash if from a repository.

- `ChunkCrossReference`:
  - `target_chunk_id`: ID of target chunk within document (if resolved).
  - `target_section_id`: ID of target section.
  - `kind`: `CrossReferenceKind` (`PreviousChunk`, `NextChunk`, `ParentScope`, `ChildChunk`, `SplitPart`, `Imports`, `Calls`, `DefinesSymbol`, `Extends`, `References`, `LinksTo`).
  - `target_symbol`: Referenced symbol name.
  - `target_path`: File path of the target.
  - `description`: Human/LLM-readable summary of the edge.

- `ChunkingConfig`: Configurable parameters:
  - `strategy`: `ChunkingStrategy` (`Adaptive`, `SemanticAst`, `HeadingAware`, `FixedWindowWithOverlap`).
  - `target_chunk_tokens` (default 512).
  - `max_chunk_tokens` (default 1024).
  - `min_chunk_tokens` (default 64).
  - `overlap_tokens` (default 64).
  - `preserve_headers` (default `true`).
  - `include_citations` (default `true`).
  - `include_cross_references` (default `true`).
  - `include_sequence_links` (default `true`).
  - `tokenizer_name` (default `cl100k_base`).

## Verification & Quality

- **100% Passing Test Suite:** 55 unit and integration tests across 11 test suites:
  - `tests/golden_benchmark_test.rs`: 8 golden benchmark archetypes, simulated Top-1 retrieval recall ($\ge 99\%$), latency verification ($< 50\,\text{ms}$), Markdown report generator.
  - `tests/chunk_determinism_test.rs`: 10-iteration sequential determinism, 8-thread concurrent multi-threaded determinism, bidirectional sequential chaining symmetry, and coordinate monotonicity/non-empty body invariants.
  - `tests/export_jsonl_test.rs`: Streaming JSONL export, line/byte metrics, token budget truncation, empty collection handling.
  - `tests/vector_schema_test.rs`: Qdrant points and batch envelopes, Pinecone strict metadata typing, Chroma batch collections, Milvus entities, Weaviate RFC 4122 UUID objects, OpenAI Batch API formatting, Cohere embed records, deterministic UUID generation, schema validator rejections.
  - `tests/citation_test.rs`: GitHub permalinks with commit hashes, web documentation anchors, PDF page numbers, OpenAPI operation paths, local file citations.
  - `tests/cross_reference_test.rs`: Sequential chunk chaining, split-part parent-child hierarchy, intra-document symbol calls/imports, CDM relationship propagation, context header injection, Serde JSON roundtrip.
  - `tests/ast_chunking_test.rs`: Rust, Python, TypeScript, Go AST splitting, oversized function statement splitting.
  - `tests/heading_chunking_test.rs`: Heading hierarchy, breadcrumbs, oversized prose splitting.
  - `tests/table_chunking_test.rs`: Multi-chunk table header retention.
  - `tests/sliding_window_test.rs`: Continuity and zero overlap.
  - `tests/engine_document_test.rs`: Adaptive document chunking, bit-for-bit determinism, sub-50ms performance.
- **Strict Compliance:** Zero mocks, zero placeholders, Clippy `-D warnings` clean, `cargo deny` clean.
- **Formal Certification:** Phase 10.4 (RAG chunk generation) 100% complete and certified across all 4 subphases (10.4.1, 10.4.2, 10.4.3, 10.4.4).
