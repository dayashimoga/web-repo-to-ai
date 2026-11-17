# CHANGELOG (`CHANGELOG.md`)

**Policy:** Append-only chronological record of verified project changes by Stage, Phase, and Subphase. Never overwrite or delete historical entries.

---

## [Unreleased] — Stage 4: Canonical Model and Export Engine

### 2026-11-17 — Phase 4.3: Export (`4.3.3`)

#### Added
- **`[4.3.3]` Implement Asynchronous Streaming Downloads in `services/api`:**
  - Added `tokio-util = { version = "0.7", default-features = false }` to root workspace dependencies and `tokio-util = { workspace = true, features = ["io"] }`, `web-repo-to-ai-exporters = { workspace = true }` to `services/api/Cargo.toml`.
  - Added `"checksums"` to `VALID_EXPORT_FORMATS` in `services/api/src/dto/jobs.rs`.
  - Integrated `DynStorage` into `AppState` in `services/api/src/state.rs` with automatic instantiation via `create_storage_backend(&config.storage)` and test/memory injection constructors `with_storage` and `with_metrics_and_storage`.
  - Implemented `get_artifact_handler` in `services/api/src/routes/jobs.rs`:
    - Handles `GET /api/v1/jobs/{job_id}/artifacts/{format}` for all supported formats (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `checksums`, `cdm_json`).
    - Storage lookup with fallback keys matching standard layout (`jobs/{job_id}/artifact.{ext}`).
    - Direct chunked streaming from storage using `ReaderStream` and Axum `Body::from_stream`, preventing large file RAM exhaustion.
    - On-demand rendering fallback: If a pre-rendered artifact does not exist in storage, looks for the stored parent CDM document (`jobs/{job_id}/artifact.cdm.json`), renders the requested format on the fly, caches it to storage, and streams it.
    - Standard RFC compliance headers: `Content-Type` matching format MIME, `Content-Disposition: attachment; filename="web-repo-to-ai-{job_id}.{ext}"`, `Content-Length`, `x-content-type-options: nosniff`, `cache-control: private, no-transform, max-age=86400`, and correlated `x-request-id`.
    - Returns standard RFC 7807 problem details (`404 Not Found`, `application/problem+json`) if neither artifact nor CDM exists.
  - Added integration test suites in `services/api/src/lib.rs` verifying streaming markdown downloads, zip streaming, manifest & checksums downloads, on-demand CDM rendering fallback, headers integrity, and 404/400 validation.
  - Verification: 41 passing API tests (up from 37), 457 total workspace Rust tests (up from 453), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.3: Export (`4.3.2`)

#### Added
- **`[4.3.2]` Implement Standalone Manifest & Checksums Generation in `crates/exporters`:**
  - Added `blake3 = { workspace = true }` to `crates/exporters/Cargo.toml`.
  - Implemented standalone cryptographic checksum engine in `crates/exporters/src/checksum.rs`:
    - `ChecksumAlgorithm`: `Sha256`, `Blake3` with `as_str()`, `Display`, and `compute_hex()`.
    - `compute_sha256_hex` and `compute_blake3_hex` for deterministic 64-character lowercase hex digest computation.
    - `ChecksumEntry`: Path, hash, algorithm, and constructor helpers `sha256`, `blake3`, `from_sha256_bytes`, and `from_blake3_bytes`.
    - BSD/GNU coreutils standard generation: `generate_checksums_manifest` and streaming `export_checksums_to_writer` formatting lines as `<hash>  <path>\n`.
    - Coreutils parser: `parse_checksums_manifest` parsing standard and binary `*` mode lines, ignoring comment lines (`#`) and whitespace, and verifying 64-hex character syntax.
    - Checksum verifier: `verify_checksums` returning a `ChecksumVerificationReport` with `matched_files`, `mismatched_files` (`ChecksumMismatch`), `missing_files`, and `is_valid()` verification status.
  - Implemented standalone package manifest and quality report generator in `crates/exporters/src/manifest.rs`:
    - `detect_mime_type`: Zero-dependency IANA MIME media type detector mapping file extensions (`.md`, `.txt`, `.json`, `.jsonl`, `.html`, `.zip`, `.rs`, `.ts`, `.js`, `.py`, `.go`, `.css`, etc.).
    - `PackageFileEntry`: Inventory item capturing relative path, uncompressed byte size, dual SHA-256 and BLAKE3 digests, format descriptor, canonical MIME type, line count, and token estimates (`div_ceil(4)`).
    - `PackageManifest`: Root manifest schema v1.0 with generator version, document ID, schema version, creation timestamp, canonical URL, source type, file count, total uncompressed bytes, and file entries.
    - `PackageProvenance`: Provenance schema capturing canonical URL, source identity, revision info, and citations list.
    - `PackageQualityReport`: Quality report schema capturing completeness ratio, formatted percentage, discovered/extracted/excluded/failed/unsupported counts, and exclusion logs.
    - Builders: `build_manifest`, `build_provenance`, `build_quality_report`.
    - Exporters: `export_manifest`, `export_provenance`, `export_quality_report` and their respective single-pass streaming writer functions.
    - `ManifestExporter`: Fluent builder API supporting in-memory and streaming generation.
  - Refactored `crates/exporters/src/zip.rs` to reuse `PackageFileEntry`, `PackageManifest`, `PackageProvenance`, and `PackageQualityReport` from `crate::manifest`, and checksum generation from `crate::checksum`.
  - Added `ManifestOptions` and `ChecksumOptions` models and presets in `crates/exporters/src/options.rs`.
  - Added `ExporterError::ChecksumValidation` error variant in `crates/exporters/src/error.rs`.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite `crates/exporters/tests/manifest_checksums.rs` (11 tests) verifying manifest generation, provenance, quality report, MIME detection, checksum formatting, parsing, corrupt file detection, missing file detection, and streaming equivalence.
  - Verification: 73 passing exporter tests (up from 62), 453 total workspace Rust tests (up from 442), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.3: Export (`4.3.1`)

#### Added
- **`[4.3.1]` Implement Deterministic ZIP Package Generator in `crates/exporters`:**
  - Added dependency `zip = { version = "9.6", default-features = false, features = ["deflate"] }` with `flate2/zlib-rs` pure safe Rust backend (zero C dependencies, zero cargo deny advisories) and `sha2 = { workspace = true }`.
  - Implemented `ZipPackageOptions` and `ZipPackageMetrics` in `crates/exporters/src/options.rs`:
    - Flags for bundled formats: `include_markdown`, `include_plain_text`, `include_json_cdm`, `include_json_structured`, `include_html`, `include_jsonl`.
    - Flags for source files & audit metadata: `include_sources`, `include_manifest`, `include_provenance`, `include_quality_report`, `include_checksums`.
    - Nested sub-options: `markdown_options`, `plain_text_options`, `json_options`, `html_options`, `jsonl_options`.
    - Compression options: `compression_method` (`ZipCompressionMethod::Deflate` or `Stored`), and `compression_level` (0–9).
    - Presets: `complete()` (default), `minimal()` (markdown + plain_text + manifest), `formats_only()` (all 6 formats + manifest/provenance/quality/checksums without sources), and `sources_only()` (hierarchical sources + manifest/checksums).
  - Implemented core ZIP exporter engine in `crates/exporters/src/zip.rs`:
    - `ZipPackageExporter`: Fluent builder API supporting in-memory `export_to_bytes` and streaming `export_to_writer`.
    - `export_zip_package` and `export_zip_package_to_writer`: High-performance deterministic ZIP package generator writing to any `std::io::Write + std::io::Seek` sink.
    - Deterministic reproducible builds: Sets uniform fixed timestamps (`1980-01-01 00:00:00`) and standard permissions (`0o644`) on all ZIP entries.
    - Path traversal defense: Sanitizes all archive paths using `sanitize_zip_path`, stripping drive prefixes, leading slashes, backslashes, and `../` directory traversal attempts to prevent zip-slip vulnerabilities.
    - Metadata manifests: Emits `manifest.json` (`PackageManifest`), `provenance.json` (`PackageProvenance`), and `quality-report.json` (`PackageQualityReport`).
    - Standard checksums: Generates `checksums.sha256` matching GNU `sha256sum` format (`<hex_digest>  <relative_archive_path>`) for all embedded archive files.
    - Telemetry metrics: Captures total files bundled, uncompressed byte size, compressed byte size, and overall package SHA-256 digest.
  - Re-exported public ZIP package types in `crates/exporters/src/lib.rs`.
  - Created integration test suite `crates/exporters/tests/zip_export.rs` (9 tests) verifying complete preset, minimal preset, formats-only preset, sources-only preset, streaming writer equivalence, path sanitization security defense, empty documents, serde roundtrips, and builder API.
  - Verification: 62 passing exporter tests (up from 53), 442 total workspace Rust tests (up from 433), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.4` — **PHASE 4.2 CERTIFIED**)

#### Added
- **`[4.2.4]` Implement HTML & JSONL Generators in `crates/exporters`:**
  - Implemented `HtmlOptions` configuration model in `crates/exporters/src/options.rs`:
    - Standalone HTML5 document toggle (`standalone`), embedded responsive CSS (`include_css`), custom CSS injection (`custom_css`), header banner (`include_header`), hierarchy tree style (`hierarchy_style`), table of contents (`include_table_of_contents`), collapsible `<details><summary>` sections (`collapsible_sections`, `initially_expanded`), code line numbers (`include_line_numbers`), sources & citations table (`include_citations`), excluded files audit table (`include_exclusions`), completeness report (`include_completeness_report`), and token budget ceiling (`max_token_budget`).
    - Added preset constructors: `standalone()`, `fragment()`, and `minimal()`.
  - Implemented core HTML exporter engine in `crates/exporters/src/html.rs`:
    - `HtmlExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - `export_html` and `export_html_to_writer`: Single-pass streaming renderer writing directly to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Embedded zero-CDN responsive stylesheet (`DEFAULT_HTML_CSS`) with automatic `@media (prefers-color-scheme: dark)` dark/light theme switching, system font stacks, badge badges, card layouts, and code block styling.
    - Security-hardened HTML character entity escaping (`escape_html`) preventing XSS vulnerabilities.
    - Anchor slug generator (`slugify`) for table of contents navigation.
    - Collapsible section rendering with line numbering support.
  - Implemented `JsonlFlavor` and `JsonlOptions` configuration models in `crates/exporters/src/options.rs`:
    - Flavors: `RagChunks` (embedding/retrieval records with document/line coordinates), `FineTuningChat` (conversational instruction dataset records with `messages`), `FineTuningPromptCompletion` (`prompt`/`completion` pairs), and `CanonicalSections` (lossless serialized CDM sections per line).
    - Presets: `rag()`, `chat_fine_tuning()`, `prompt_completion_fine_tuning()`, and `canonical()`.
  - Implemented core JSON Lines exporter in `crates/exporters/src/jsonl.rs`:
    - `JsonlExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - Models: `RagChunkRecord`, `ChatMessage`, `FineTuningChatRecord`, `FineTuningPromptCompletionRecord`.
    - High-throughput streaming serialization writing strictly one line per section terminated by `\n`.
  - Added `ExporterError::Json` variant with `#[from] serde_json::Error` in `crates/exporters/src/error.rs`.
  - Re-exported all HTML and JSONL types in `crates/exporters/src/lib.rs`.
  - Created integration test suites:
    - `crates/exporters/tests/html_export.rs` (10 tests) verifying standalone layout, fragments, minimal preset, line numbers, token budget truncation, streaming byte equivalence, empty documents, serde roundtrips, XSS defense, and builder API.
    - `crates/exporters/tests/jsonl_export.rs` (9 tests) verifying RAG chunks, chat fine-tuning, prompt/completion pairs, canonical CDM sections, token budget truncation, streaming byte equivalence, empty documents, serde roundtrips, and builder API.
  - Verification: 53 passing exporter tests (up from 34), 433 total workspace Rust tests (up from 414), 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.3`)

#### Added
- **`[4.2.3]` Implement Multi-Flavor JSON Generator in `crates/exporters`:**
  - Implemented `JsonFormatFlavor` and `JsonExportOptions` configuration models in `crates/exporters/src/options.rs`:
    - `JsonFormatFlavor`: `CanonicalCdm` (full schema v1.0), `StructuredContext` (token-optimized flat envelope tailored for LLM system prompts and RAG embeddings), and `Minified` (compact single-line JSON without whitespace).
    - `JsonExportOptions`: Options controlling pretty-printing, field filtering (`include_metadata`, `include_hierarchy`, `include_sections`, `include_citations`, `include_exclusions`, `include_completeness`), and token budgeting (`max_token_budget`). Includes preset constructors `canonical()`, `structured()`, and `compact()`.
  - Implemented core JSON exporter engine in `crates/exporters/src/json.rs`:
    - `JsonExporter`: Fluent builder API supporting in-memory `export_to_string` and streaming `export_to_writer`.
    - `export_json` and `export_json_to_writer`: High-performance deterministic serializing functions writing directly to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Token-optimized Structured Context envelope models: `StructuredContextEnvelope`, `StructuredMetrics`, `StructuredCompleteness`, `StructuredHierarchyNode`, `StructuredSection`, `StructuredCitation`, and `StructuredExclusion` with `#[serde(default)]` and `#[derive(Default)]` for seamless deserialization.
    - Dynamic section filtering and token budget caps that prune overflow sections while marking output with `truncated: true` telemetry.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite in `crates/exporters/tests/json_export.rs` (8 tests) verifying Canonical CDM serialization, Structured Context envelopes, Minified single-line output, token budget truncation, field filtering, streaming writer byte equivalence, empty document handling, and serde roundtrips.
  - Verification: 34 passing exporter tests, 414 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.2`)

#### Added
- **`[4.2.2]` Implement Plain-Text Generator in `crates/exporters`:**
  - Implemented `PlainTextOptions` configuration model in `crates/exporters/src/options.rs`:
    - Configurable top-level document metadata banner (`include_header_banner`), hierarchy tree style (`HierarchyStyle::AsciiTree` default), plain-text table of contents, customizable section divider lines (`divider_char`, `divider_width`), line numbering, exclusions and citations audit sections, completeness report, and token budget ceiling.
    - Added preset constructors `minimal()` and `compact()`.
  - Implemented core Plain-Text generator in `crates/exporters/src/plain_text.rs`:
    - `PlainTextExporter`: Builder API supporting `export_to_string` and streaming `export_to_writer`.
    - `export_plain_text`: In-memory deterministic plain-text generation.
    - `export_plain_text_to_writer`: Single-pass streaming renderer writing directly to any `std::io::Write` sink with `CountingWriter` for exact byte and newline telemetry.
    - Clean ASCII header banner detailing Document Title, Source URL, Type, Retrieved timestamp, License, Language, Authors, Commit SHA, volume/token metrics, and Completeness yield.
    - Plain-text directory / outline tree rendering (ASCII tree, Unicode tree, or bulleted list) without Markdown code block fences.
    - Plain-text Table of Contents with numbered section index, titles, language, line count, byte size, and token estimates.
    - Clean polyglot section formatting with structured divider banners (`----------------`), section kind labels, metadata headers, and clean bodies without markdown fence noise.
    - Plain-text Sources & Citations section.
    - Plain-text Excluded Files & Content audit section detailing skipped paths, reasons, patterns, sizes, and rationale.
    - Completeness Accounting breakdown with discovered, extracted, excluded, failed, and unsupported counts.
    - Token budget enforcement halting section emission with an ASCII warning banner when budget is exceeded.
  - Factored shared utilities into `crates/exporters/src/util.rs` (`CountingWriter`, `format_bytes`).
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Created integration test suite in `crates/exporters/tests/plain_text_export.rs` (10 tests) verifying default export, minimal/compact presets, line numbering, token budget truncation, streaming writer byte equivalence, hierarchy styles, custom dividers, empty document handling, and serde roundtrips.
  - Verification: 26 passing exporter tests, 406 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.2: Format Generators (`4.2.1`)

#### Added
- **`[4.2.1]` Implement Markdown Generator in `crates/exporters`:**
  - Added `chrono` dependency to `crates/exporters/Cargo.toml` with serde support.
  - Implemented typed domain errors in `crates/exporters/src/error.rs`:
    - `ExporterError`: Enumerating `Io`, `Formatting`, `BudgetExceeded`, and `InvalidDocument`.
    - `Result<T>` type alias.
  - Implemented configuration models and telemetry metrics in `crates/exporters/src/options.rs`:
    - `CodeFenceStyle`: `Backticks` and `Tildes` with automatic fence escalation to 4+ markers when code bodies contain backticks or tildes.
    - `HierarchyStyle`: `UnicodeTree` (`├──`, `└──`), `AsciiTree` (`|--`, `\--`), `BulletedList` (`- **dir/**`), and `None`.
    - `ExportMetrics`: Tracking `bytes_written`, `lines_written`, `sections_exported`, `exclusions_exported`, `citations_exported`, `estimated_tokens`, and `truncated`.
    - `MarkdownOptions`: Comprehensive configuration controlling frontmatter, metadata block, hierarchy tree, table of contents with GFM slugs, per-section metadata, exclusions, citations, completeness accounting, heading levels, and token budget limits. Includes preset constructors `minimal()` and `compact()`.
  - Implemented core Markdown exporter engine in `crates/exporters/src/markdown.rs`:
    - `MarkdownExporter`: Fluent engine with `export_to_string` and streaming `export_to_writer`.
    - `export_markdown`: In-memory deterministic Markdown generation.
    - `export_markdown_to_writer`: Single-pass streaming renderer writing to any `std::io::Write` sink with `CountingWriter` for exact telemetry.
    - Structured YAML frontmatter with string escaping for special characters (`:`, `"`, `#`, `\n`).
    - Human- and AI-readable metadata summary block with source URL, type, license, and language tags.
    - Formatted directory / outline tree in code blocks preserving file sizes and line counts.
    - Table of contents with anchor links derived via GitHub GFM slugification rules and token/line telemetry.
    - Polyglot section formatting covering all `SectionKind` variants (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Admonition`, `Navigation`, `Manifest`, `Frontmatter`).
    - Escalating code fence delimiters preventing markdown fence collisions.
    - GFM blockquote alerts (`> [!NOTE]`, `> [!WARNING]`, etc.) for admonition callouts.
    - Auditable exclusions table documenting skipped files, patterns, sizes, and rationale.
    - Sources & citations section for full provenance tracking.
    - Completeness accounting summary block.
    - Token budget truncation defense with advisory notices when limits are exceeded.
  - Re-exported public types in `crates/exporters/src/lib.rs`.
  - Implemented 15 integration tests in `crates/exporters/tests/markdown_export.rs` verifying default formatting, minimal/compact presets, fence escalation, GFM alerts, line numbering, token budget truncation, streaming writer byte equivalence, hierarchy styles, YAML escaping, and empty document handling.
  - Verification: 16 passing exporter tests, 396 total workspace Rust tests, 184 frontend tests, 29 browser-worker tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.4`) — PHASE 4.1 CERTIFIED

#### Added
- **`[4.1.4]` Implement Schema Versioning and Migration Framework in `crates/cdm` (Phase 4.1 Certified):**
  - Implemented semantic versioning representation in `crates/cdm/src/version.rs`:
    - `SchemaVersion`: Strongly-typed SemVer struct (`major`, `minor`, `patch`, `pre_release`) implementing parsing via `FromStr`, `Display`, ordering via `Ord`/`PartialOrd`, string serde roundtrips, and compatibility negotiation via `is_compatible_with` (`major == target.major`).
    - Standard version constants: `current()`, `v0_8_0()`, `v0_9_0()`, `v1_0_0()`.
  - Implemented migration step architecture in `crates/cdm/src/migration.rs`:
    - `MigrationStep` trait with `from_version()`, `to_version()`, `description()`, and `migrate(&Value) -> Result<Value, CdmError>`.
    - `Migration0_8To0_9`: Migrates legacy `content_blocks` to normalized `sections`, extracts `source_url`, sets initial `source_identity` coordinates, and upgrades schema version to `0.9.0`.
    - `Migration0_9To1_0`: Establishes root `hierarchy` tree, derives mathematical `CompletenessReport` preserving invariants, populates missing BLAKE3 content hashes and baseline token metrics, initializes auxiliary collections (`exclusions`, `warnings`, `citations`, `relationships`), and bumps schema version to `1.0.0`.
    - `MigrationRegistry`: Thread-safe registry finding shortest migration chains using Breadth-First Search (BFS), executing sequential transformations on `serde_json::Value` payloads, and providing `migrate_and_deserialize` into [`CdmDocument`].
    - Added helper migration functions `migrate_document_json` and `migrate_document_value`.
  - Implemented schema and document diffing in `crates/cdm/src/diff.rs`:
    - `SchemaDiff`: Computes differences between two document instances, detecting schema version shifts, added sections, removed sections, modified sections (`body`, `content_hash`, `line_range`, `byte_range`, `title`), hierarchy modifications, and completeness shifts.
    - `SectionDiff`: Detailed attribute-level section comparison.
  - Extended `JsonOptions` and JSON deserialization in `crates/cdm/src/json.rs`:
    - Added `auto_migrate: bool` to `JsonOptions` and fluent builder `.with_auto_migrate(bool)`.
    - Updated `from_json_with_options`, `from_slice_with_options`, and `from_reader_with_options` to seamlessly up-migrate legacy payloads when `auto_migrate` is enabled.
  - Added convenience methods to `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.diff(&other) -> SchemaDiff`
    - `CdmDocument::from_json_migrated(json_str) -> Result<CdmDocument>`
    - `CdmDocument::from_value_migrated(value) -> Result<CdmDocument>`
  - Added `#[serde(default)]` to `DocumentMetadata` and `RevisionInfo` for robust forward/backward compatibility when deserializing partial schema payloads.
  - Re-exported versioning, migration, and diffing primitives in `crates/cdm/src/lib.rs`.
  - Added comprehensive integration tests in `crates/cdm/tests/schema_migration.rs` verifying multi-step migration (`v0.8.0` $\to$ `v0.9.0` $\to$ `v1.0.0`), direct preview migration, streaming reader/slice auto-migration, document diffing, and error handling for unregistered version jumps and downgrade attempts.
  - Verified 78/78 tests in `crates/cdm` (40 unit + 7 JSON serde + 7 schema migration + 8 schema validation + 16 validation invariants), 381/381 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.
  - Formally completed and certified **Phase 4.1 (Canonical Context Document Model)** across all 4 subphases (`4.1.1` through `4.1.4`).

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.3`)

#### Added
- **`[4.1.3]` Implement CDM Schema Validation and Invariant Checks in `crates/cdm`:**
  - Added dependency to `crates/cdm/Cargo.toml`: `url = { workspace = true }` for standard URL validation.
  - Implemented comprehensive validation engine in `crates/cdm/src/validator.rs`:
    - `ValidationSeverity`: `Error` (fatal violation) and `Warning` (advisory finding).
    - `ValidationIssue`: Strongly-typed validation issue tracking invariant code, human-readable message, structural path coordinate, and severity level.
    - `ValidationConfig`: Configurable validation rules (`strict_mode`, `verify_hashes`, `enforce_completeness`, `check_referential_integrity`, `check_tree_acyclicity`, `max_tree_depth`), with fluent constructors `strict()`, `permissive()`, and builder methods.
    - `ValidationReport`: Aggregate report collecting issues across sections, hierarchy nodes, relationships, and completeness, with methods `is_valid()`, `has_errors()`, `has_warnings()`, `error_count()`, `warning_count()`, `filter_by_code()`, `filter_by_path_prefix()`, and human-readable `summary()`.
    - `CdmValidator`: Stateful validator running 8 modular validation passes:
      1. Major schema version compatibility against `SCHEMA_VERSION` (`1.0.0`).
      2. Document identity & metadata invariants (non-nil `document_id`, HTTP/HTTPS `canonical_url` validation via `url::Url::parse`, `raw_input` non-empty, GitHub/web coordinate checks, 64-char hex format checks on BLAKE3/SHA-256 hashes, descriptive title warning).
      3. Section-level invariants (non-empty unique `section_id`s, valid `source_url`s, 1-indexed monotonic `line_range` `1 <= start <= end`, `byte_range` `start <= end`, BLAKE3 content hash recomputation against `body`, and token metrics consistency).
      4. Hierarchy tree acyclicity & integrity (depth bounding to `max_tree_depth`, duplicate node ID cycle detection, and referential integrity verifying every node's `section_id` exists in document sections).
      5. Relationships referential integrity (`source_id` and internal `target_id` existence in document sections, self-referential cycle warnings).
      6. Authoritative citations validity (URL parse verification, section ID referential integrity).
      7. Completeness accounting invariants (strict formula $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$, verifying complete extraction status requires `is_complete() == true`).
      8. Warning records and exclusions integrity (non-empty warning codes, messages, and exclusion paths).
  - Extended `CdmError` in `crates/cdm/src/error.rs` with `ValidationFailed { error_count, summary }` and helper `CdmError::from_report(&ValidationReport)`, mapped cleanly to workspace `CoreError::Validation`.
  - Added convenience validation methods to `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.validation_report() -> ValidationReport`
    - `doc.validate_with_config(&ValidationConfig) -> ValidationReport`
    - `doc.validate() -> Result<(), CdmError>`
    - `doc.validate_strict() -> Result<(), CdmError>`
  - Re-exported validation types in `crates/cdm/src/lib.rs`.
  - Added 12 unit tests in `src/validator.rs` and 16 integration tests in `crates/cdm/tests/validation_invariants.rs` covering all invariant checks, cycle detection, referential integrity violations, duplicate IDs, invalid line/byte ranges, tampered hashes, completeness matrix violations, and diagnostic reports.
  - Verified 60/60 tests in `crates/cdm`, 363/363 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.2`)

#### Added
- **`[4.1.2]` Implement CDM JSON Serialization and Deserialization in `crates/cdm`:**
  - Added dependency to `crates/cdm/Cargo.toml`: `thiserror = { workspace = true }`.
  - Implemented strongly-typed domain error taxonomy `CdmError` in `crates/cdm/src/error.rs`:
    - `Serialization(serde_json::Error)`: JSON string/stream encoding failures.
    - `Deserialization(serde_json::Error)`: JSON syntax and structural decoding failures.
    - `PayloadTooLarge { size_bytes, max_bytes }`: Defense against unbounded payload bombs and excessive allocation.
    - `UnsupportedSchemaVersion { version, supported }`: Major schema version mismatch protection.
    - `HashMismatch { expected, actual }`: Document/section cryptographic integrity verification failures.
    - `Validation(String)`: Semantic invariant and model validation errors.
    - `Io(std::io::Error)`: Low-level I/O reader/writer stream errors.
    - Implemented `From<CdmError> for CoreError` mapping CDM domain errors into standard workspace `CoreError` codes (`INVALID_INPUT`, `SERIALIZATION_ERROR`, `IO_ERROR`).
  - Implemented production-grade JSON serialization engine in `crates/cdm/src/json.rs`:
    - `JsonOptions`: Configurable serialization/deserialization behavior (`pretty`, `max_bytes` default 64 MiB, `verify_hashes`, `validate_version`).
    - `to_json_pretty(&CdmDocument) -> Result<String, CdmError>`: Deterministic multi-line formatted JSON.
    - `to_json_compact(&CdmDocument) -> Result<String, CdmError>`: Space-efficient single-line JSON.
    - `to_writer<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`: Streaming compact writer.
    - `to_writer_pretty<W: std::io::Write>(&CdmDocument, writer: W) -> Result<(), CdmError>`: Streaming formatted writer.
    - `from_json(&str) -> Result<CdmDocument, CdmError>`: Memory-safe deserializer from string slice.
    - `from_slice(&[u8]) -> Result<CdmDocument, CdmError>`: Byte-slice deserializer.
    - `from_reader<R: std::io::Read>(reader: R) -> Result<CdmDocument, CdmError>`: Streaming deserializer bounded by default 64 MiB ceiling defense.
    - `from_reader_with_options<R: std::io::Read>(reader: R, options: &JsonOptions) -> Result<CdmDocument, CdmError>`: Fully customizable stream reader with `std::io::Read::take(max_bytes + 1)` defense preventing memory exhaustion.
  - Implemented convenience methods directly on `CdmDocument` in `crates/cdm/src/model.rs`:
    - `doc.to_json_pretty()`, `doc.to_json_compact()`, `CdmDocument::from_json(&str)`, `CdmDocument::from_slice(&[u8])`, `CdmDocument::from_reader(reader)`.
  - Re-exported `CdmError`, `JsonOptions`, and JSON utilities in `crates/cdm/src/lib.rs`.
  - Added comprehensive integration test suite in `crates/cdm/tests/json_serde.rs`:
    - `test_json_lossless_roundtrip_complex_document`: Verifies exact property equality between original document and deserialized document.
    - `test_json_deterministic_serialization`: Verifies byte-for-byte identical output across repeated runs with BTreeMap sorting.
    - `test_json_streaming_writer_and_reader`: Verifies streaming I/O roundtrip fidelity.
    - `test_payload_size_ceiling_defense`: Verifies rejection with `PayloadTooLarge` when stream exceeds configured byte limit.
    - `test_corrupted_json_syntax_rejected`: Verifies `Deserialization` error on truncated/invalid JSON.
    - `test_schema_version_validation_logic`: Verifies rejection of incompatible major schema versions while accepting minor updates.
    - `test_hash_integrity_verification_on_deserialization`: Verifies detection of tampered section content hashes via optional validation.
  - Verified 32/32 tests in `crates/cdm` (17 unit + 7 JSON serde integration + 8 schema validation integration), 335/335 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 4.1: Canonical Context Document Model (`4.1.1`)

#### Added
- **`[4.1.1]` Implement Canonical Context Document Model (CDM) Schema in `crates/cdm`:**
  - Added dependencies to `crates/cdm/Cargo.toml`: `chrono` (with `serde`), `uuid` (with `v4`, `v7`, `serde`), and `blake3`.
  - Implemented core document models in `crates/cdm/src/model.rs`: `CdmDocument`, `SourceIdentity`, `SourceType` (`WebpageStatic`, `WebpageDynamic`, `GitHubPublic`, `GitHubPrivate`, `DocumentPdf`, `DocumentOcr`), `RevisionInfo`, `ExtractionStatus` (`Completed`, `Partial`, `Failed`, `Unsupported`), `ExtractionWarning`, and `WarningSeverity` (`Critical`, `Warning`, `Info`).
  - Implemented section models in `crates/cdm/src/section.rs`: `CdmSection`, `SectionKind` (`Prose`, `CodeFile`, `CodeSnippet`, `Table`, `Frontmatter`, `Manifest`, `Navigation`, `Admonition`), `LineRange` with line count arithmetic, `ByteRange` with saturating byte length, and `SectionTokenMetrics` with whitespace/character counting and baseline token estimation.
  - Implemented metadata and provenance models in `crates/cdm/src/metadata.rs`: `DocumentMetadata`, `Citation` (generating concise, markdown callout, and plaintext formats), and `ExclusionRecord` with typed `ExclusionReason` (`RulePattern`, `SizeLimit`, `BinaryFile`, `ExcludedExtension`, `Duplicate`, `UserConfig`, `SecurityRestricted`, `UnsupportedType`).
  - Implemented structural tree hierarchy in `crates/cdm/src/hierarchy.rs`: `HierarchyNode`, `HierarchyKind` (`Root`, `Directory`, `File`, `Section`, `Heading`), child management, recursive ID search (`find_node`), path search (`find_node_by_path`), pre-order flattening (`flatten`), and node/file counters.
  - Implemented relationship models in `crates/cdm/src/relationships.rs`: `Relationship` and `RelationshipKind` (`Imports`, `ImportedBy`, `Calls`, `Extends`, `LinksTo`, `References`, `ParentOf`, `ChildOf`, `DefinesSymbol`).
  - Implemented completeness accounting in `crates/cdm/src/completeness.rs`: `CompletenessReport` verifying the strict mathematical invariant formula $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed} + \text{unsupported}$ and `is_complete()`.
  - Implemented ergonomic fluent builders in `crates/cdm/src/builder.rs`: `CdmDocumentBuilder` and `CdmSectionBuilder` with automatic BLAKE3 hash computation and baseline token metric derivation.
  - Added 19 comprehensive unit and integration tests across `crates/cdm` (11 unit + 8 integration), validating schema consistency, tamper detection, hierarchy queries, completeness matrix, and token metrics, bringing workspace tests to 322 passing tests.
  - Verified 19/19 tests in `crates/cdm`, 322/322 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.4`) — PHASE 3.4 & STAGE 3 CERTIFIED

#### Added
- **`[3.4.4]` Implement Golden Dataset Test Suite for Universal Webpage Extraction in `crates/web-extractor` (Phase 3.4 & Stage 3 Certified):**
  - Created comprehensive golden dataset integration test suite in `crates/web-extractor/tests/golden_dataset.rs` covering 8 real-world HTML archetypes:
    1. `test_golden_archetype_1_technical_blog_article`: Technical blog article validating article title, author byline, publication ISO timestamp, word count, reading time estimation, Rust syntax highlighting, OpenGraph and Schema.org JSON-LD parsing, and quality completeness score ($\ge 0.85$, `Complete`).
    2. `test_golden_archetype_2_docusaurus_documentation`: Multi-version technical documentation site validating Docusaurus framework detection, 3-level breadcrumbs navigation, typed admonitions (`Tip`, `Warning`), and next/previous pagination links.
    3. `test_golden_archetype_3_api_specification_tables`: REST API specification validating complex HTML tables with alignment indicators (`:---:`, `---:`), GFM rectangular table normalization, backtick formatting, and HTTP reference extraction.
    4. `test_golden_archetype_4_polyglot_code_with_line_gutters`: Multi-language code repository page validating monotonic line number gutter stripping from Rust code, Python shebang detection, and SQL DDL syntax parsing without indentation corruption.
    5. `test_golden_archetype_5_noisy_malformed_html`: Noisy real-world news layout validating unclosed markup recovery, `BoilerplateCleaner` pruning of cookie banners, inline advertisements, social share widgets, newsletter popups, and related articles lists.
    6. `test_golden_archetype_6_spa_hydration_lifecycle`: Single-page application lifecycle validating `SpaDetector` heuristic identification of low-density skeleton shells and noscript fallbacks, followed by post-render hydration verifying extraction of complete client-side telemetry markup.
    7. `test_golden_archetype_7_canonical_and_simhash_dedup`: Search engine deduplication pipeline validating relative canonical URL resolution with UTM query stripping, exact URL duplicate registration, 64-bit SimHash near-duplicate clustering within Hamming distance threshold ($\le 8$), and distinct content isolation.
    8. `test_golden_archetype_8_sparse_stub_diagnostics`: Malformed/sparse stub testing asserting heavy diagnostic penalty (< 0.40 score), issuance of `CONTENT_SPARSE` and missing metadata warnings, and exclusion from CDM conversion (`is_acceptable_for_cdm = false`).
  - Verified 143/143 tests in `crates/web-extractor` (135 unit + 8 golden integration tests), 304/304 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.
  - Formally completed and certified **Phase 3.4 (Extraction Quality)** and **Stage 3 (Universal Webpage Extraction)** across all 16 subphases (`3.1.1` through `3.4.4`).

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.3`)

#### Added
- **`[3.4.3]` Implement Content Completeness Scoring and Extraction Warnings in `crates/web-extractor`:**
  - Implemented multidimensional completeness and quality models in `crates/web-extractor/src/quality/model.rs`: `CompletenessLevel` (`Complete`, `High`, `Moderate`, `Partial`, `Degraded`, `Empty`), `ExtractionWarningCategory`, `ExtractionWarning`, `DimensionScores`, `ContentMetrics`, and `ExtractionQualityReport`.
  - Implemented `QualityScorer` in `crates/web-extractor/src/quality/scorer.rs` evaluating metadata completeness (title, description, author, dates, language, canonical link), content volume and density, structural fidelity (heading hierarchy, code syntax detection, table header normalization, link categorization), and acquisition provenance (hashes, HTTP status, cache validators, canonical verification, SSRF blocked subresources, unhandled JS page errors, dynamic degradation).
  - Implemented DOM heading analysis using `tl` parser scanning heading sequences and detecting hierarchy gaps (e.g. `<h1>` jumped directly to `<h3>` or `<h4>`), emitting `HEADING_HIERARCHY_GAP` warnings.
  - Implemented empirical content metrics extraction: word count, character count, estimated reading time, paragraph count, heading counts by level, code block and table counts, internal/external link breakdown, content density ratio, and boilerplate ratio.
  - Integrated `ExtractionQualityReport` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` across both `hydrate_static` and `render_and_hydrate` execution paths.
  - Re-exported `quality` types in `crates/web-extractor/src/lib.rs`.
  - Added 8 unit and integration tests covering rich article scoring, empty content critical warnings, sparse text degradation, missing metadata penalties, heading hierarchy gaps, code syntax and table header detection, subresource/page error diagnostics, and browser degradation penalties, bringing `crates/web-extractor` to 135/135 passing tests and workspace to 296/296 passing tests.
  - Verified 135/135 tests in `crates/web-extractor`, 296/296 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.2`)

#### Added
- **`[3.4.2]` Implement Duplicate Detection and Canonical URL Resolution in `crates/web-extractor`:**
  - Implemented deduplication models in `crates/web-extractor/src/dedup/model.rs`: `CanonicalSource`, `CanonicalResolution`, `DuplicateClassification`, `DuplicateRecord`, `DedupOptions`.
  - Implemented `CanonicalUrlResolver` in `crates/web-extractor/src/dedup/canonical.rs` resolving `<link rel="canonical">` tags with relative URL resolution, cross-domain canonical detection, and SSRF security validation preventing private IP or loopback targets from polluting canonical resolution.
  - Implemented 64-bit `SimHasher` in `crates/web-extractor/src/dedup/simhash.rs` utilizing unigram tokenization, term-frequency weighting, 64-bit FNV-1a hashing, Hamming distance computation, and Jaccard token similarity for robust near-duplicate detection.
  - Implemented thread-safe `DuplicateRegistry` in `crates/web-extractor/src/dedup/registry.rs` indexing canonical URLs, BLAKE3 content digests, and 64-bit SimHash fingerprints, classifying incoming documents as `Unique`, `ExactCanonical`, `ExactContent`, or `NearDuplicate`.
  - Integrated `CanonicalUrlResolver` into `ProvenanceBuilder::build_static` and `ProvenanceBuilder::build_dynamic` in `crates/web-extractor/src/provenance/extractor.rs`, automatically recording canonical resolutions on `SourceAttribution`.
  - Re-exported `dedup` types in `crates/web-extractor/src/lib.rs`.
  - Added 10 unit and integration tests covering canonical resolution, cross-domain and SSRF security handling, SimHash Hamming distance similarity, and concurrent registry duplicate classification, bringing `crates/web-extractor` to 127/127 passing tests and the workspace to 288/288 passing tests.
  - Verified 127/127 tests in `crates/web-extractor`, 288/288 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.4: Extraction Quality (`3.4.1`)

#### Added
- **`[3.4.1]` Implement Source Attribution, HTTP Provenance, and Metadata Capture in `crates/web-extractor`:**
  - Added `blake3` and `sha2` workspace dependencies to `crates/web-extractor/Cargo.toml`.
  - Implemented strongly-typed source attribution models in `crates/web-extractor/src/provenance/model.rs`: `SourceAttribution`, `HttpProvenance`, `RedirectHopRecord`, `ContentHashes`, `CapturedMetadata`, `BrowserDiagnostics`, `ProvenanceCitations`, `ProvenanceWarning`, and `WarningSeverity`.
  - Implemented dual cryptographic content hash computation (`ContentHashes::compute`) generating SHA-256 and BLAKE3 lowercase hex digests over raw response markup bytes and extracted plain text.
  - Implemented `MetadataExtractor` in `crates/web-extractor/src/provenance/extractor.rs` extracting and parsing Schema.org JSON-LD structured data blocks (`<script type="application/ld+json">`) and cascading metadata precedence (OpenGraph -> Twitter -> HTML `<head>` -> Schema.org).
  - Implemented `ProvenanceBuilder` in `crates/web-extractor/src/provenance/extractor.rs` synthesizing `SourceAttribution` for both static acquisition (`build_static`) and dynamic browser rendering (`build_dynamic`), capturing HTTP headers, ETag, Last-Modified, cache-control, server, pinned IP, redirect history, and generating formatted citations (concise, markdown callout, plaintext).
  - Implemented diagnostic warning classification detecting canonical URL mismatches (`CANONICAL_URL_MISMATCH`), missing cache validators (`MISSING_HTTP_CACHE_VALIDATORS`), missing author metadata (`MISSING_AUTHOR`), blocked subresources, and dynamic-to-static degradation (`BROWSER_DEGRADED_TO_STATIC`).
  - Integrated `SourceAttribution` directly into `ExtractedWebpage` in `crates/web-extractor/src/browser/extractor.rs` for all static and dynamic execution paths.
  - Re-exported provenance types in `crates/web-extractor/src/lib.rs`.
  - Added unit and integration tests covering content hash computation, Schema.org JSON-LD extraction, metadata precedence cascading, and provenance builder warning generation, bringing `crates/web-extractor` to 117/117 passing tests and workspace to 278/278 passing tests.
  - Verified 117/117 tests in `crates/web-extractor`, 278/278 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.4`) — PHASE 3.3 CERTIFIED

#### Added
- **`[3.3.4]` Implement Dynamic DOM Extraction and Post-Render Hydration in `crates/web-extractor` (Phase 3.3 Milestone Certified):**
  - Implemented `DynamicWebExtractor` orchestrating full web extraction: fast streaming static acquisition, content density & SPA shell heuristic analysis, dynamic browser worker invocation for JS-rendered pages, and post-render hydration into DOM, article, documentation, and structured element models.
  - Implemented `SpaDetector` heuristic analyzer detecting empty DOMs, framework hydration roots (`#root`, `#app`, `#__next`, `#svelte`, `<app-root>`), `<noscript>` JavaScript warnings, and low-density markup shells (< 5% content density).
  - Implemented post-render hydration pipeline feeding rendered HTML into `DomParser`, `ArticleExtractor`, `DocExtractor`, and `ElementsExtractor`, returning unified `ExtractedWebpage` instances.
  - Implemented `DynamicProvenance` capturing extraction method (`Static` vs `DynamicBrowser`), HTTP status, redirect chains, console logs, unhandled page errors, blocked subresources, and diagnostic warnings.
  - Implemented graceful degradation in `DynamicWebExtractor::extract`: if dynamic browser rendering fails or times out on an SPA page with fallback static content, preserve static extraction and attach diagnostic provenance warnings rather than failing the job.
  - Re-exported `DynamicWebExtractor`, `SpaDetector`, `ExtractionStrategy`, `ExtractionMethod`, `DynamicExtractorOptions`, `DynamicProvenance`, and `ExtractedWebpage` in `crates/web-extractor/src/lib.rs`.
  - Added 4 unit and integration tests in `crates/web-extractor/src/browser/extractor.rs` testing SPA detector heuristics, rich article static path, SPA fallback with mock browser worker, and graceful degradation on browser worker failure, bringing `crates/web-extractor` to 114/114 tests and workspace to 275/275 passing tests.
  - Formally certified **Phase 3.3 ("Dynamic extraction")** with all 4 subphases (`3.3.1` – `3.3.4`) fully implemented, verified, and accepted.
  - Verified 114/114 tests in `crates/web-extractor`, 275/275 workspace tests, 29/29 Node.js tests in `services/browser-worker`, 184/184 frontend tests in `apps/web`, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.3`)

#### Added
- **`[3.3.3]` Implement Resource Limits, Navigation Timeouts, and Sandbox Isolation in `services/browser-worker` and `crates/web-extractor`:**
  - Implemented `ConcurrencyLimiter` in `services/browser-worker/src/limiter.js` enforcing maximum concurrent page sessions (default 4) and a bounded FIFO queue (capacity 20) with queue timeout (15s), rejecting capacity overflows with HTTP 429 `ConcurrencyLimitExceeded` and `Retry-After: 5`.
  - Implemented automated Chromium lifecycle recycling in `PlaywrightEngine` (`recycleBrowser`), automatically relaunching Chromium when active sessions reach 0 and either `rendersCount >= maxRendersPerBrowser` (200) or process memory exceeds RSS threshold (768 MB).
  - Implemented hard rendering session deadline timer (`hardTimeoutBufferMs = 5000` added to client timeout) via `Promise.race`, terminating hung pages with 504 `SessionTimeout`.
  - Hardened context isolation with `serviceWorkers: 'block'`, `acceptDownloads: false`, and `permissions: []`.
  - Implemented automatic dialog dismissal on `page.on('dialog')` so `alert()`, `confirm()`, and `prompt()` modals never block worker execution.
  - Implemented subresource request ceiling (`maxSubresources = 200`), aborting excess subresources with `blockedbyclient` and recording in `blocked_subresources`.
  - Implemented maximum rendered HTML payload ceiling (`maxHtmlBytes = 10 MB`), rejecting oversized DOMs with 413 `PayloadTooLarge`.
  - Hardened `isPrivateOrLoopbackHost` to comprehensively classify IPv4 private ranges (RFC 1918), loopback (127.0.0.0/8), link-local/cloud metadata (169.254.0.0/16, 169.254.169.254, 100.100.100.200, metadata.google.internal), CGNAT (100.64.0.0/10), current network (0.0.0.0/8), broadcast, IPv6 ULA (fc00::/7), IPv6 link-local (fe80::/10), and IPv4-mapped IPv6.
  - Updated `crates/web-extractor/src/browser/error.rs` and `client.rs` to map HTTP 429 to `BrowserWorkerError::TooManyRequests` and HTTP 413 to `BrowserWorkerError::PayloadTooLarge`.
  - Configured `deny.toml` advisory ignore for `RUSTSEC-2026-0543` (transitive sqlx chrono advisory).
  - Added 9 unit and integration tests in `services/browser-worker/test/limits.test.js` validating limiter queueing, 429 rejection with `Retry-After`, address classification, dialog auto-dismissal, subresource ceilings, and HTML payload bounds, bringing `services/browser-worker` tests to 29/29 passing across 7 suites.
  - Verified 110/110 tests in `crates/web-extractor`, 271/271 workspace tests, 184/184 frontend tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.2`)

#### Added
- **`[3.3.2]` Implement Playwright/Chromium Dynamic Fallback Integration in `services/browser-worker` and `crates/web-extractor`:**
  - Integrated `playwright-core` into `services/browser-worker` with pinned `package.json` and reproducible `package-lock.json`.
  - Implemented `PlaywrightEngine` in `services/browser-worker/src/engine.js` with launch fallback resolution across bundled Playwright binaries, `CHROME_BIN`, `BROWSER_PATH`, Google Chrome channel, and Microsoft Edge channel, enforcing sandboxed container arguments (`--no-sandbox`, `--disable-dev-shm-usage`, `--disable-gpu`, `--disable-extensions`).
  - Implemented isolated per-request `BrowserContext` lifecycles, ensuring complete state, cookie, and cache segregation with unconditional cleanup in a `finally` block to prevent RAM, socket, or process leaks.
  - Implemented subresource request interception (`isPrivateOrLoopbackHost`) blocking non-HTTP(S) schemes and private/loopback/cloud-metadata IP destinations (`127.0.0.1`, `localhost`, `169.254.169.254`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`).
  - Implemented bounded console log (max 100) and page error (max 50) buffers with string truncation to prevent memory bloat from noisy scripts.
  - Implemented error classification and mapping into RFC 7807 problem details (`NavigationTimeout`, `SelectorTimeout`, `DnsResolutionFailed`, `ConnectionRefused`, `TlsCertificateError`).
  - Integrated `engine.render()` and live readiness probes into `services/browser-worker/src/routes.js` and graceful shutdown hooks in `services/browser-worker/server.js`.
  - Updated `crates/web-extractor/src/browser/client.rs` to map `SelectorTimeout` errors to `BrowserWorkerError::NavigationTimeout`.
  - Added 7 unit and integration tests in `services/browser-worker/test/engine.test.js` validating hostname security classification, error mapping, live Chromium SPA hydration with selector wait, and HTTP redirect chains, bringing `services/browser-worker` tests to 20/20 passing across 4 suites.
  - Verified 110/110 tests in `crates/web-extractor`, 271/271 workspace tests, 184/184 frontend tests, and 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.3: Dynamic Extraction (`3.3.1`)

#### Added
- **`[3.3.1]` Implement Browser Worker Architecture and IPC Protocol in `services/browser-worker` and `crates/web-extractor`:**
  - Initialized isolated Node.js microservice in `services/browser-worker` with `package.json`, Node >=20 engine, and standard library HTTP server.
  - Implemented strongly typed IPC/RPC protocol in `services/browser-worker/src/protocol.js` validating render requests (`url`, `wait_until`, `timeout_ms`, `wait_for_selector`, `viewport`, `user_agent`, `headers`, `javascript_enabled`, `block_subresource_ips`) and generating RFC 7807 problem details error envelopes and standard response payloads.
  - Implemented HTTP dispatcher in `services/browser-worker/src/routes.js` handling `GET /health` (liveness), `GET /ready` (readiness), and `POST /render` (dynamic rendering RPC).
  - Implemented standalone daemon in `services/browser-worker/server.js` with graceful shutdown signal handling on `SIGINT`/`SIGTERM`.
  - Added 13 unit and integration tests in `services/browser-worker/test/protocol.test.js` using Node.js built-in `node:test`.
  - Defined strongly typed Rust IPC models in `crates/web-extractor/src/browser/model.rs`: `BrowserRenderRequest`, `BrowserRenderResponse`, `BrowserRedirect`, `ConsoleMessage`, `PageError`, `BlockedSubresource`, `WaitUntil`, `Viewport`, `BrowserTiming`, and `BrowserWorkerHealth`.
  - Implemented `BrowserWorkerError` in `crates/web-extractor/src/browser/error.rs` and mapped into `WebExtractorError::BrowserWorker` and `CoreError::Extraction`.
  - Implemented asynchronous `BrowserWorkerClient` in `crates/web-extractor/src/browser/client.rs` executing `check_health`, `check_readiness`, and `render` RPC calls over HTTP with connection timeouts and RFC 7807 error parsing.
  - Added 3 unit and integration tests in `crates/web-extractor` (110 total in crate), expanding workspace tests to 271 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.4`) & Phase 3.2 Milestone Certification

#### Added
- **`[3.2.4]` Implement Tables, Code Blocks, Syntax Detection, and Link Extraction in `crates/web-extractor`:**
  - Defined `ExtractedTable`, `TableAlignment`, `ExtractedCodeBlock`, `ExtractedLink`, `LinkType`, `ExtractedElements`, and `ElementsOptions` in `crates/web-extractor/src/elements/model.rs`.
  - Implemented `TableExtractor` in `crates/web-extractor/src/elements/table.rs` extracting table headers, body rows, captions, and alignments (`align` and CSS `text-align`), normalizing `colspan` and `rowspan` into rectangular matrices, and generating GitHub Flavored Markdown (GFM) tables with character escaping.
  - Implemented `detect_syntax` and `normalize_language_tag` in `crates/web-extractor/src/elements/syntax.rs` identifying programming languages via shebangs and keyword patterns across Rust, Python, TypeScript, JavaScript, Go, C/C++, Java, SQL, HTML, CSS, JSON, YAML, TOML, Bash, and Dockerfile.
  - Implemented `CodeBlockExtractor` in `crates/web-extractor/src/elements/code.rs` handling `<pre><code>` and `<pre>` blocks, identifying language classes and data attributes, stripping Rouge/Pygments line-number gutters and monotonic numeric prefixes, and preserving exact indentation, whitespace, and line breaks without loss.
  - Implemented `LinkExtractor` in `crates/web-extractor/src/elements/link.rs` resolving hyperlinks against base URLs, classifying links into `Internal`, `External`, `Anchor`, `Asset` (`.pdf`, `.zip`, `.parquet`, `.tar.gz`, etc.), and `Special` (`mailto:`, `tel:`), and neutralizing dangerous `javascript:` schemes.
  - Implemented `ElementsExtractor` in `crates/web-extractor/src/elements/extractor.rs` coordinating unified extraction of tables, code, and links from raw HTML strings or parsed DOM trees.
  - Added 20 unit and integration tests across elements modules and `lib.rs` (107 total in crate), expanding workspace tests to 268 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.
  - Formally certified completion of **Phase 3.2: Static Extraction** (`3.2.1` – `3.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.3`)

#### Added
- **`[3.2.3]` Implement Technical Documentation and Navigation Hierarchy Extraction in `crates/web-extractor`:**
  - Defined `ExtractedDocumentation`, `DocFramework`, `Admonition`, `AdmonitionType`, `BreadcrumbItem`, `TocItem`, `DocLink`, `DocPagination`, `DocVersionInfo`, and `DocSection` in `crates/web-extractor/src/docs/model.rs`.
  - Implemented `detect_framework` in `crates/web-extractor/src/docs/framework.rs` identifying generator frameworks (Docusaurus, mdBook, Sphinx, MkDocs, Rustdoc, VitePress, Nextra, GitBook, Generic).
  - Implemented `extract_table_of_contents` in `crates/web-extractor/src/docs/toc.rs` extracting heading levels (`h1`–`h6`), stripping anchor symbols (`#`, `¶`, `§`), mapping anchor IDs, and building nested ToC trees.
  - Implemented `DocNavigationExtractor` in `crates/web-extractor/src/docs/navigation.rs` resolving breadcrumb lists, previous/next pagination links, and version dropdown selectors.
  - Implemented `AdmonitionExtractor` in `crates/web-extractor/src/docs/admonition.rs` classifying callouts across standard and framework-specific alert styles.
  - Implemented `DocExtractor` in `crates/web-extractor/src/docs/extractor.rs` orchestrating raw HTML navigation discovery, framework-targeted container selection, cleaned content rendering, and section decomposition.
  - Updated `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` protecting admonitions in `CONTENT_INDICATORS` and refining ad token matching with boundary checks.
  - Isolated health check probe file naming in `services/api/src/routes/health.rs` with `RequestId::new()` to eliminate Windows sharing violations during concurrent test execution.
  - Added 14 unit and integration tests across doc frameworks (87 total in crate, 248 across workspace).

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.2`)

#### Added
- **`[3.2.2]` Implement Article Extraction and Readability Heuristics in `crates/web-extractor`:**
  - Defined `ExtractedArticle`, `ReadabilityMetrics`, and `ReadabilityOptions` builder in `crates/web-extractor/src/article/model.rs`.
  - Implemented heuristic pattern scoring in `crates/web-extractor/src/article/heuristics.rs` featuring positive/negative class matchers, unlikely candidate filters, and structural tag weights.
  - Implemented `ReadabilityScorer` in `crates/web-extractor/src/article/scorer.rs` computing text density, comma and prose frequency, paragraph density, heading counts, code block bonuses, and quadratic link-density penalties.
  - Implemented `ArticleExtractor` in `crates/web-extractor/src/article/extractor.rs` identifying candidate article containers (`article`, `main`, `section`, `div`), rendering selected subtrees via `DomParser::render_subtree`, extracting `<h1>` headlines, tiered bylines (`rel="author"`, `class="author"`), publication timestamps (`<time datetime="...">`), and lead excerpts, calculating reading times via `div_ceil`, and supporting whole-document fallback.
  - Extended `DomParser` in `crates/web-extractor/src/dom/parser.rs` to render isolated subtrees and preserve `rel`, `id`, `class` on `<a>` tags and `datetime` on `<time>` tags.
  - Added `HtmlParseFailed` and `ExtractionFailed` error variants to `WebExtractorError` and mapped into `CoreError`.
  - Added 15 unit and integration tests across article extraction modules (73 total in crate), expanding workspace tests to 234 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.2: Static Extraction (`3.2.1`)

#### Added
- **`[3.2.1]` Implement DOM Parsing and Boilerplate Removal in `crates/web-extractor`:**
  - Integrated `tl` (v0.7.8, safe pure Rust, MIT licensed, zero dependencies) into root `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `DocumentMetadata` in `crates/web-extractor/src/dom/metadata.rs` extracting `<title>`, meta description/author/keywords, OpenGraph (`og:*`), Twitter card (`twitter:*`), `<base href="...">`, document language, and canonical link attributes with robust fallback cascades.
  - Implemented `BoilerplatePolicy` and builder in `crates/web-extractor/src/dom/policy.rs` providing granular controls for scripts, styles, comments, nav, footers, sidebars, forms, ads, cookies, social sharing widgets, and custom class block/allow lists.
  - Implemented `BoilerplateCleaner` in `crates/web-extractor/src/dom/cleaner.rs` distinguishing boilerplate elements from semantic content, explicitly protecting `<pre><code>` blocks, markdown body containers, and `<table>` content.
  - Implemented `LinkResolver` in `crates/web-extractor/src/dom/link_resolver.rs` resolving and canonicalizing relative `<a>` href and `<img>` src attributes against base URLs while neutralizing malicious `javascript:` schemes.
  - Implemented `DomParser` and `CleanedDocument` in `crates/web-extractor/src/dom/parser.rs` executing single-pass recursive DOM traversal, boilerplate pruning, link rewriting, standard entity decoding, and text newline normalization.
  - Added 20 unit and integration tests across DOM modules and `lib.rs` (58 total in crate), expanding workspace tests to 219 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.4`) & Phase 3.1 Milestone Certification

#### Added
- **`[3.1.4]` Implement Compression Bombs, Timeouts, and Body Size Limits in `crates/web-extractor`:**
  - Added `flate2` (with pure-Rust `rust_backend`), `brotli`, and `zstd` to root `Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `ContentEncoding` enum in `crates/web-extractor/src/compression/encoding.rs` supporting `Identity`, `Gzip`, `Deflate`, `Brotli`, and `Zstd` with case-insensitive token and comma-separated header parsing.
  - Implemented `BodyLimits` in `crates/web-extractor/src/compression/limits.rs` encapsulating `max_raw_body_bytes` (10 MB), `max_decompressed_bytes` (25 MB), `max_expansion_ratio` (20.0), `min_ratio_check_bytes` (4 KB), and `enforce_content_length`.
  - Implemented `BoundedDecompressor` in `crates/web-extractor/src/compression/decompressor.rs` utilizing `CountingReader` with atomic byte tracking to stream decompressed data and detect ratio explosions and size breaches in real time across single and chained encodings.
  - Added `DecompressionBomb`, `DecompressionFailed`, `UnsupportedContentEncoding`, and `ContentLengthMismatch` variants to `WebExtractorError` and mapped into `CoreError`.
  - Upgraded `WebFetcher` in `crates/web-extractor/src/http/fetcher.rs` with total operation timeout bounding, chunk read inactivity timeouts, declared Content-Length early exit, Content-Length match enforcement, and transparent multi-format decompression.
  - Extended `FetchedResponse` with `raw_body_size` and `content_encoding` for full acquisition provenance.
  - Extended `MockHttpTransport` with `set_delay` and `set_chunk_delay` to enable deterministic timeout and slow-drip testing.
  - Added 22 unit and integration tests across compression modules and `lib.rs` (38 total in crate), expanding workspace tests to 199 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.
  - Formally certified completion of **Phase 3.1: Secure Acquisition** (`3.1.1` – `3.1.4` all `ACCEPTED`).

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.3`)

#### Added
- **`[3.1.3]` Implement HTTP Fetching, Streaming Bodies, and Redirect Revalidation in `crates/web-extractor`:**
  - Added `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }` to root `Cargo.toml`.
  - Added `reqwest`, `async-trait`, `bytes`, `futures`, `chrono`, and `tokio` to `crates/web-extractor/Cargo.toml`.
  - Added `"CDLA-Permissive-2.0"` to permitted licenses in `deny.toml` to support `webpki-roots` (Mozilla root CA store for pure-Rust TLS without C OpenSSL).
  - Implemented `WebExtractorError` in `crates/web-extractor/src/error.rs` covering `UrlSecurity`, `Http`, `RedirectLoop`, `TooManyRedirects`, `MissingLocationHeader`, `InvalidRedirectUrl`, `PayloadTooLarge`, `Timeout`, and `Network`, with seamless conversion into `CoreError`.
  - Implemented `RedirectHop` and `RedirectTracker` in `crates/web-extractor/src/http/redirect.rs` tracking hop history, detecting URL cycles, and enforcing max redirect limits.
  - Implemented `FetchedResponse` in `crates/web-extractor/src/http/response.rs` capturing initial URL, terminal canonical URL, status code, response headers, body bytes, pinned IP address, redirect history, ETag, and Last-Modified headers.
  - Defined asynchronous `HttpTransport` trait with production `ReqwestTransport` (configured with `Policy::none()` to disable automatic redirects and enforce connection to pinned socket addresses) and programmable `MockHttpTransport` in `crates/web-extractor/src/http/transport.rs`.
  - Implemented `WebFetcher` and `FetchOptions` in `crates/web-extractor/src/http/fetcher.rs` coordinating hop-by-hop DNS resolution & IP pinning via `SecureDnsResolver`, custom User-Agent and Accept header injection, redirect revalidation, and streaming chunked body consumption bounded by `max_body_bytes`.
  - Fixed target URL propagation in `SecureDnsResolver::resolve_and_pin` when reusing cached DNS resolutions.
  - Added 8 unit and integration tests in `crates/web-extractor/src/lib.rs` (16 total in crate), expanding workspace tests to 177 passing across all 17 crates with 100% pass rate across all 5 monorepo verification gates.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.2`)

#### Added
- **`[3.1.2]` Implement DNS Resolution, IP Pinning, and DNS-Rebinding Protection in `crates/url-security`:**
  - Added `tokio = { workspace = true, features = ["net", "rt", "macros", "time", "sync"] }`, `async-trait = { workspace = true }`, and `chrono = { workspace = true, features = ["serde"] }` to `crates/url-security/Cargo.toml`.
  - Added `DnsResolutionFailed`, `DnsRebindingDetected`, and `IpPinningMismatch` variants to `UrlSecurityError` in `crates/url-security/src/error.rs`, mapping rebinding and pinning mismatches to `CoreError::Security`.
  - Defined asynchronous `DnsResolver` trait in `crates/url-security/src/dns.rs` along with production `SystemDnsResolver` utilizing `tokio::net::lookup_host` and thread-safe `MockDnsResolver` supporting static host mappings and ordered multi-response rebinding attack sequences.
  - Implemented immutable `PinnedTarget` in `crates/url-security/src/pinning.rs` containing `url`, `host`, `port`, `pinned_ip`, `all_resolved_ips`, `resolved_at`, and `ttl_secs`, with `socket_addr()`, TTL expiration checking, and socket destination verification via `verify_socket_addr`.
  - Implemented `SecureDnsResolver` in `crates/url-security/src/resolver.rs` providing thread-safe `RwLock` target caching, IP literal pre-checks, hostname blocklist validation, and **strict all-record SSRF validation** (rejecting entire DNS responses if even one resolved IP belongs to loopback, private, or cloud metadata CIDRs).
  - Integrated `resolve_and_pin` directly on `WebTarget` in `crates/web-extractor/src/url.rs`.
  - Added 5 new unit tests in `crates/url-security` and 1 integration test in `crates/web-extractor`, expanding workspace tests to 169 passing across all 17 crates, with all 5 monorepo verification gates passing cleanly.

### 2026-11-17 — Phase 3.1: Secure Acquisition (`3.1.1`)

#### Added
- **`[3.1.1]` Implement URL Parsing, Normalization, and Validation in `crates/url-security` and `crates/web-extractor`:**
  - Added `url = { workspace = true }`, `serde = { workspace = true, features = ["derive"] }`, and `serde_json = { workspace = true }` to `crates/url-security/Cargo.toml` and `crates/web-extractor/Cargo.toml`.
  - Implemented `UrlSecurityError` domain error enum in `crates/url-security/src/error.rs` covering `TooLong`, `ForbiddenScheme`, `ParseError`, `UserinfoForbidden`, `BlockedIp`, `BlockedHost`, `BlockedPort`, `MissingHost`, and `InvalidCharacters`, with standard conversion into `CoreError`.
  - Implemented `UrlSecurityPolicy` and `UrlSecurityPolicyBuilder` in `crates/url-security/src/policy.rs` supporting configurable length limits, scheme allowlists, marketing/analytics tracking query parameter lists, port blocklists against protocol confusion attacks, and test overrides.
  - Implemented comprehensive SSRF filters in `crates/url-security/src/ip_filter.rs`:
    - `is_blocked_ip` covering IPv4 loopback (`127.0.0.0/8`), RFC 1918 private (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), link-local / cloud metadata (`169.254.0.0/16` including AWS/GCP/Azure `169.254.169.254`), CGNAT (`100.64.0.0/10`), multicast (`224.0.0.0/4`), reserved (`240.0.0.0/4`), IPv6 loopback (`::1`), unspecified (`::`), unique-local (`fc00::/7`), link-local (`fe80::/10`), multicast (`ff00::/8`), documentation (`2001:db8::/32`), IPv4-mapped IPv6 (`::ffff:0:0/96`), IPv4-compatible IPv6, 6to4 encapsulation (`2002::/16`), and Teredo tunneling (`2001:0000::/32`).
    - `is_blocked_host` covering domain blocklists, local/internal domain patterns (`localhost`, `*.localhost`, `*.local`, `*.internal`, `*.lan`), cloud metadata domain names (`metadata.google.internal`), and alternative integer IP notations (POSIX `inet_aton` decimal integers e.g. `2130706433`, hex `0x7f000001`, octal `017700000001`, and dotted abbreviations `127.1`).
  - Implemented `UrlNormalizer` in `crates/url-security/src/normalizer.rs` performing length verification, null byte and control character detection, WHATWG parsing, scheme validation, default-port stripping (`:80` for http, `:443` for https), duplicate path slash collapse, tracking parameter stripping (`utm_*`, `fbclid`, `gclid`), and deterministic query parameter sorting.
  - Implemented domain wrapper `ValidatedUrl` in `crates/url-security/src/validated_url.rs` with `as_str`, `origin`, `is_same_origin`, `is_same_domain`, `is_subpath_of`, `join` for relative link traversal with full security re-evaluation, and secure serde serialization/deserialization.
  - Implemented `WebTarget` and `WebPageType` in `crates/web-extractor/src/url.rs` with heuristic classifier `classify_url` recognizing documentation (by keywords and subdomains e.g. `docs.rs`), articles (by keywords, date paths, and platforms e.g. `medium.com`, `substack.com`), file resources (by file extensions), and generic web pages.
  - Added 16 unit tests in `crates/url-security` and 7 unit tests in `crates/web-extractor`, expanding workspace unit tests to 163 passing across all 17 crates with 0 warnings and 0 errors across all 5 monorepo verification gates.

### 2026-11-17 — Phase 2.4: Storage (`2.4.4`), Phase 2.4 Milestone Certification & Stage 2 Completion Certification

#### Added
- **`[2.4.4]` Implement Artifact Checksum Validation and Verification in `crates/storage`:**
  - Added `sha2` (v0.10) and `blake3` (v1.5) to root `Cargo.toml` and `crates/storage/Cargo.toml`.
  - Implemented `ChecksumAlgorithm` enum supporting `Sha256` and `Blake3` with string representations, parse traits, and digest length constants.
  - Implemented `ArtifactDigests` computing SHA-256 and BLAKE3 concurrently over in-memory slices (`compute`) and over asynchronous streaming readers (`compute_streaming`) in 64 KB chunks to bound RAM usage.
  - Implemented `ChecksumVerificationStatus` (`Verified`, `Mismatch`, `Missing`, `Corrupted`) and structured `ArtifactIntegrityReport` tracking expected vs actual hashes, byte sizes, and error details.
  - Implemented `StorageIntegrityAuditSummary` with health checks and success rate calculations.
  - Implemented `ArtifactVerifier` coordinating storage backends and PostgreSQL: `verify_bytes`, `verify_storage_file`, `verify_artifact_record`, `verify_job_artifact`, `verify_all_job_artifacts`, and `audit_storage_integrity`.
  - Implemented BSD/GNU-compatible `checksums.sha256` manifest generator `generate_sha256_manifest` and robust parser `parse_sha256_manifest`.
  - Added 7 unit tests in `crates/storage/src/checksum.rs` verifying empty vectors, hash matching/validation, streaming vs slice parity, verifier outcomes, relational artifact verification, manifest round-trips, and audit summary health, expanding storage crate tests to 55 and automated Rust workspace tests to 142/142 passing. Verified all 5 monorepo gates pass cleanly.
  - Formally certified completion of **Phase 2.4: Storage** (`2.4.1` – `2.4.4` all `ACCEPTED`).
  - Formally certified completion of **Stage 2: Backend and Processing Foundation** (15 subphases across `Phase 2.1`, `Phase 2.2`, `Phase 2.3`, `Phase 2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.4: Storage (`2.4.3`)

#### Added
- **`[2.4.3]` Implement Artifact Retention and Ephemeral Cleanup Policies in `crates/storage`:**
  - Implemented `RetentionPolicy` in `crates/storage/src/retention.rs` with configurable TTLs (`default_ttl`, `max_ttl`, `ephemeral_ttl`), high/low capacity watermarks (`high_watermark_bytes`, `low_watermark_bytes`), batch sizes, invariant validation, and bounded `calculate_expires_at` deadline computation.
  - Implemented `CleanupReport` tracking evaluated artifacts, deleted artifacts, bytes freed, deleted ephemeral staging files, freed ephemeral bytes, and non-fatal error accumulators with merge and inspection helpers.
  - Implemented relational repository `ArtifactRepository` providing SQLx CRUD operations on `job_artifacts` with non-blocking transactional batch deletion using `FOR UPDATE SKIP LOCKED`.
  - Extended `StorageBackend` trait, `LocalFsStorage`, and `InMemoryStorage` with `cleanup_stale_staging_files` scanning for and purging orphaned temporary staging files (`.*.tmp.*`).
  - Implemented central lifecycle coordinator `RetentionManager` executing expired database artifact sweeps, orphaned staging file cleanup, job directory deletion, and capacity watermark enforcement (pruning oldest artifacts down to low watermark threshold).
  - Added 7 unit tests in `crates/storage/src/retention.rs` covering policy validations, TTL calculations, report merging/metrics, in-memory stale staging cleanup, watermark capacity enforcement, filesystem staging cleanup, and job directory purging, expanding storage crate tests to 48 and automated Rust workspace tests to 135/135 passing. Verified all 5 monorepo gates pass cleanly.

### 2026-11-17 — Phase 2.4: Storage (`2.4.2`)

#### Added
- **`[2.4.2]` Implement Pluggable Storage Backend Trait in `crates/storage`:**
  - Defined object-safe asynchronous storage stream alias `pub type AsyncReadStream = Pin<Box<dyn tokio::io::AsyncRead + Send>>;` and dispatch handle `pub type DynStorage = Arc<dyn StorageBackend>;`.
  - Implemented async trait `#[async_trait] pub trait StorageBackend: Send + Sync + std::fmt::Debug` covering `backend_type`, `put`, `get`, `open_read`, `exists`, `delete`, `delete_directory`, `metadata`, `list`, and `storage_capacity`.
  - Implemented `StorageBackend` for `LocalFsStorage` mapping directly to atomic filesystem operations.
  - Implemented `InMemoryStorage` using `Arc<RwLock<HashMap<String, Vec<u8>>>>` with strict path traversal checking, prefix-based listing, byte count metrics in `storage_capacity()`, and `Cursor<Vec<u8>>` for `open_read()`, providing zero-placeholder in-memory operations.
  - Implemented runtime backend factory `create_storage_backend(config: &StorageConfig) -> Result<DynStorage, CoreError>` supporting `StorageBackendConfig::LocalFs` and returning configuration errors for unconfigured backends.
  - Added 5 unit tests in `crates/storage/src/backend.rs` verifying LocalFsStorage via `DynStorage`, `InMemoryStorage` CRUD, recursive directory deletion, traversal rejections, and factory configuration errors, expanding storage crate tests to 41 and automated Rust workspace tests to 128/128 passing. Verified all 5 monorepo gates pass cleanly.

### 2026-11-17 — Phase 2.4: Storage (`2.4.1`)

#### Added
- **`[2.4.1]` Implement Local Filesystem Artifact Storage in `crates/storage`:**
  - Added `tokio` features `["fs", "io-util", "macros", "rt"]` to `crates/storage/Cargo.toml`.
  - Implemented `LocalFsStorage` in `crates/storage/src/local_fs.rs` providing durable atomic file writes, reading, streaming handles (`open_read`), existence probing, single file deletion, recursive directory deletion, and metadata retrieval.
  - Implemented strict defense-in-depth path traversal defenses in `resolve_path(&self, key)`: rejects null bytes, absolute paths, root/drive prefixes (`C:`), parent traversal components (`..`), Windows reserved device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`), and trailing period/space characters.
  - Implemented atomic staging writes: writes data to `.<name>.tmp.<uuid>`, flushes to physical storage with `sync_all()`, and renames atomically with Windows fallback.
  - Implemented standard artifact key layout helpers in `layout` module (`job_artifact_key`, `job_dir_key`, `blob_key`).
  - Implemented recursive directory capacity accounting in `storage_capacity()` returning `StorageUsage` (`total_artifact_bytes`, `artifact_count`).
  - Added 8 unit tests in `crates/storage/src/local_fs.rs` covering CRUD lifecycle, atomic overwrite, traversal rejections, recursive directory deletion, listing, storage capacity metrics, not found errors, and payload limits, expanding storage crate tests to 36 and automated Rust workspace tests to 123/123 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.4`) & Phase 2.3 Milestone Certification

#### Added
- **`[2.3.4]` Implement Durable Job Leasing, Heartbeat, and Recovery in `crates/storage`:**
  - Implemented `LeaseConfig` in `crates/storage/src/leasing.rs` defining worker identity (`worker_id`), lease duration (`lease_duration_seconds`), heartbeat interval (`heartbeat_interval_seconds`), and max retry attempts (`max_attempts`) with invariant validation and Serde support.
  - Implemented `HeartbeatResult` and `LeaseRecoveryReport` structs with accumulator and inspection helpers (`total_processed`, `is_empty`) and Serde support.
  - Implemented `JobRepository::acquire_next_job` using PostgreSQL `SELECT ... FOR UPDATE SKIP LOCKED` inside an atomic transaction, dynamically leasing either `queued` jobs or stale `leased`/`running` jobs whose lease expired, updating attempt count, calculating lease expiration relative to `NOW()`, and appending an audit record to `job_state_transitions`.
  - Implemented `JobRepository::heartbeat_job` and `JobRepository::is_lease_valid`, enforcing worker lease ownership, preventing heartbeat on terminal jobs, verifying expiration boundaries, updating timestamps, and optionally logging progress transitions.
  - Implemented `JobRepository::release_lease` enabling graceful voluntary lease relinquishment back to `queued` on worker shutdown, and `JobRepository::recover_stale_jobs` sweeping expired worker leases to either `queued` (for retry) or `failed` with code `LEASE_TIMEOUT_EXCEEDED` (when max attempts reached).
  - Implemented coordinator `LeaseManager` wrapping `PgPool` and `LeaseConfig` with typed async leasing operations, updated `JobStatus::can_transition_to` to permit `Running -> Leased` reacquisition, and added 6 unit tests covering leasing configurations, validation rejections, constants invariants, and Serde round-trips, expanding storage crate tests to 28 and automated Rust workspace tests to 115/115 passing. Verified all 5 quality gates pass cleanly.
  - Formally certified completion of **Phase 2.3: Persistence** (`2.3.1` – `2.3.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.3: Persistence (`2.3.3`)

#### Added
- **`[2.3.3]` Implement Job State Machine, Transitions, and History in `crates/storage`:**
  - Added migration file `crates/storage/migrations/20261117000002_job_state_transitions.sql` defining `job_state_transitions` audit trail table with `job_id REFERENCES jobs(id) ON DELETE CASCADE`, `from_status`, `to_status`, `phase`, `progress_pct`, `message`, `error_code`, `worker_id`, `transitioned_at`, and index `idx_job_state_transitions_job_id`.
  - Updated schema DDL script `crates/storage/src/schema/ddl.sql`, expected catalog tables/indexes in `crates/storage/src/schema/mod.rs`, and expected columns in `crates/storage/src/validation/mod.rs`.
  - Documented `job_state_transitions` table in `docs/DATA_MODEL.md` Section 2.1.
  - Implemented `JobStateMachine` and `JobTransitionRequest` builder in `crates/storage/src/state_machine.rs`:
    - Enforces legal status transitions and terminal state immutability (`Completed`, `Failed`, `Cancelled`).
    - Enforces progress bounds (0..=100), completion invariants (100% progress and Completed phase), and failure invariants.
    - Added typed entity struct `JobStateTransitionRow` deriving `sqlx::FromRow`, `Serialize`, and `Deserialize`.
  - Implemented transactional `JobRepository` in `crates/storage/src/repository.rs`:
    - `create_job(&pool, &JobRow)` and `get_job(&pool, id)`.
    - `transition_job(&pool, id, req)` using transaction with row-level locking (`SELECT ... FOR UPDATE`), state machine validation, updating status/lease/completion timestamps, and inserting audit records into `job_state_transitions`.
    - `get_transition_history(&pool, id)` retrieving chronological transition history.
    - Convenience transition helpers (`update_progress`, `complete_job`, `fail_job`, `cancel_job`).
  - Added 5 unit tests in `crates/storage/src/state_machine.rs` verifying legal/illegal transitions, request builders, invariant validations, and transition row construction, expanding storage crate tests to 22 and workspace automated tests to 109/109 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.2`)

#### Added
- **`[2.3.2]` Implement Database Migrations and Schema Validation in `crates/storage`:**
  - Added initial SQLx migration file `crates/storage/migrations/20261117000001_initial_schema.sql` defining relational tables, indexes, constraints, and triggers.
  - Implemented compile-time embedded SQLx migration runner `crates/storage/src/migrations/mod.rs` with `MIGRATOR` (`sqlx::migrate!("./migrations")`), `run_migrations(&PgPool)`, and structured `MigrationReport` with elapsed execution timing and migration version tracking.
  - Implemented runtime schema introspection and catalog validation in `crates/storage/src/validation/mod.rs`:
    - `EXPECTED_TRIGGERS` and `expected_table_columns()` column invariant maps.
    - `validate_catalog_metadata()` verifying presence of all required tables, columns, indexes, and triggers.
    - `validate_schema(&PgPool)` querying `information_schema.tables`, `information_schema.columns`, `pg_indexes`, and `information_schema.triggers` against live PostgreSQL instances.
    - `SchemaValidationReport` formatting machine-readable validation summaries.
  - Implemented central connection pool and lifecycle manager `DatabaseManager` in `crates/storage/src/database.rs` wrapping `sqlx::PgPool` with `connect`, `connect_with_config`, `migrate`, `validate_schema`, `is_healthy` (`SELECT 1`), and `close()`.
  - Added 7 unit tests covering embedded migration verification, report generation, and catalog metadata validation across table, column, index, and trigger omissions, expanding storage crate tests to 17 and total automated Rust workspace tests to 104/104 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.3: Persistence (`2.3.1`)

#### Added
- **`[2.3.1]` Design and Implement PostgreSQL Relational Schema in `crates/storage`:**
  - Added `sqlx` (v0.8, runtime-tokio, postgres, chrono, uuid, json, macros, migrate) and `serde`/`chrono`/`uuid` dependencies to workspace and `crates/storage/Cargo.toml`.
  - Authored standalone production-grade PostgreSQL 16 DDL script (`crates/storage/src/schema/ddl.sql`):
    - Table `jobs`: supports `kind` (web & GitHub variants), `status` (queued, leased, running, completed, partial, failed, cancelled), `progress_phase`, `progress_pct` (0..=100), `attempt_count`, `leased_by`, `lease_expires_at`, and `owner_session_id`.
    - Table `job_artifacts`: foreign key referencing `jobs(id)` with `ON DELETE CASCADE`, unique `(job_id, format)`, strict 64-char hex SHA256 and BLAKE3 digests, byte size, and expiration timestamps.
    - Table `github_installations`: unique `(owner_session_id, github_installation_id)`, encrypted token and 12-byte nonce storage, and token revocation timestamps.
    - Table `content_cache_index`: content-addressed cache mapping blob signatures to storage keys with LRU `last_accessed_at` timestamps.
    - Performance indexes: `idx_jobs_status_lease`, `idx_jobs_owner_session`, `idx_jobs_created_at`, `idx_job_artifacts_job_id`, `idx_job_artifacts_expires_at`, `idx_github_installations_owner`, and `idx_content_cache_last_accessed`.
    - Triggers: `fn_set_updated_at()` auto-updating `updated_at` timestamps on row modifications.
  - Implemented strongly-typed Rust entity models deriving `sqlx::FromRow` and Serde (`crates/storage/src/schema/`):
    - `JobKind`, `JobStatus` (with `can_transition_to` transition validation), `JobProgressPhase`, and `JobRow`.
    - `ArtifactFormat` (with MIME types and file extensions) and `JobArtifactRow`.
    - `GitHubInstallationRow` (with active and revocation status helpers).
    - `ContentCacheIndexRow` (with `touch()` method).
  - Implemented `split_ddl_statements()` parser in `crates/storage/src/schema/mod.rs` and added 10 unit tests in `crates/storage`, expanding total automated Rust workspace tests to 97/97 passing. Verified all 5 quality gates pass cleanly.

### 2026-11-17 — Phase 2.2: API (`2.2.4`) & Phase 2.2 Milestone Certification

#### Added
- **`[2.2.4]` Implement Health, Readiness, and Metrics Endpoints in `services/api`:**
  - Implemented container orchestrator liveness probe `GET /api/v1/health` returning structured JSON (`status: "pass"`, `service`, `version`, `uptime_seconds`, ISO 8601 `timestamp`).
  - Implemented deep subsystem readiness probe `GET /api/v1/ready` auditing storage volume writeability (creates and removes `.healthcheck_probe` file in configured `storage.base_dir`), database configuration (`database.url`), and worker queue readiness, returning 200 OK (`ready`) or 503 Service Unavailable (`not_ready`).
  - Expanded `AppMetrics` in `crates/observability/src/metrics.rs` with `http_requests_total` (`CounterVec`) and `http_request_duration_ms` (`Histogram`).
  - Added request duration timing and `http_requests_total` recording into `trace_id_middleware` in `services/api/src/middleware.rs`.
  - Added `start_time` and `uptime_seconds()` to `AppState` in `services/api/src/state.rs`.
  - Implemented Prometheus exposition endpoint `GET /api/v1/metrics` serving dynamic `web_repo_to_ai_uptime_seconds`, `web_repo_to_ai_build_info`, and application counters/gauges/histograms.
  - Added 4 integration tests in `services/api/src/lib.rs` verifying liveness fields, subsystem readiness degradation, unconfigured database failures, and Prometheus telemetry format, expanding workspace tests to 88/88 passing (37 in `services/api`). Verified all 5 quality gates pass cleanly.
  - Formally certified completion of **Phase 2.2: API** (`2.2.1` – `2.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 2.2: API (`2.2.3`)

#### Added
- **`[2.2.3]` Implement API Error Contracts and RFC 7807 Problem Details in `services/api`:**
  - Implemented RFC 7807 problem details response wrapper `ApiProblem` and helper constructors (`services/api/src/error.rs`):
    - Sets `Content-Type: application/problem+json` on all HTTP error responses.
    - Accurately maps domain errors to standard HTTP status codes (`400`, `401`, `403`, `404`, `405`, `413`, `429`, `500`, `504`).
    - Guarantees exact correlation between `x-request-id` response header and `body.error.request_id` in problem JSON envelopes.
    - Provided helpers: `to_problem_response()`, `not_found_problem()`, `validation_problem()`, `schema_problem()`, `payload_too_large_problem()`, and `method_not_allowed_problem()`.
  - Added `MethodNotAllowed` (405) variant to `ErrorCode` and `CoreError` in `crates/core/src/error.rs`.
  - Configured `RequestBodyLimitLayer` (10 MB request body limit) in `services/api/src/server.rs` with automatic RFC 7807 `PAYLOAD_TOO_LARGE` conversion in `ValidatedJson`.
  - Refactored `fallback_handler` in `services/api/src/routes/mod.rs` and handlers in `services/api/src/routes/jobs.rs` to format 404 errors as RFC 7807 problem details with correlated request ID.
  - Added 4 integration tests in `services/api/src/lib.rs` verifying problem JSON media types, request ID correlation integrity, 10 MB payload rejections, and `traceparent` propagation on errors, expanding workspace tests to 84/84 passing. Verified all 5 quality gates pass.

### 2026-11-17 — Phase 2.2: API (`2.2.2`)

#### Added
- **`[2.2.2]` Implement Request Validation and Parameter Parsing in `services/api`:**
  - Implemented typed request and response DTOs (`services/api/src/dto/`):
    - `DetectSourcesRequest` / `DetectSourcesResponse` with URL length ceilings (<= 2048 chars), supported schemes (`http`, `https`), host presence checks, batch limit (<= 50 URLs), and `classify_source_url()` helper.
    - `CreateJobRequest` / `JobResponse` validating 1..=50 sources, git reference sanitization (no `..`, control characters, or shell metacharacters), token budget bounds (1,000..=2,000,000), target model identifier syntax, and allowed export formats (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`).
    - `ListJobsQuery` / `PaginatedResponse` enforcing page numbers (>= 1), page limits (1..=100), permitted statuses (`queued`, `processing`, `completed`, `failed`, `cancelled`), and sort orders (`asc`, `desc`).
    - `JobPathParams` / `ArtifactPathParams` validating strongly-typed UUID v7 identifiers and artifact format strings.
  - Implemented custom Axum extractors with automatic domain validation (`services/api/src/extractors/`):
    - `ValidatedJson<T>`: deserializes JSON body and invokes `Validatable::validate()`, translating syntax and validation errors into machine-readable RFC 7807 problem details payloads (`INVALID_REQUEST_SCHEMA`, `VALIDATION_ERROR`, `UNSUPPORTED_URL_SCHEME`) with correlated `request_id`.
    - `ValidatedQuery<T>`: parses query strings and validates domain invariants.
    - `ValidatedPath<T>`: extracts and validates URI path parameters.
  - Implemented route endpoints (`services/api/src/routes/`):
    - `POST /api/v1/sources/detect`: classifies input URLs into `WebPage`, `PublicGitHub`, `PrivateGitHub`, or `BatchList`.
    - `POST /api/v1/jobs`: accepts validated job specifications, creates UUID v7 `JobId`, records Prometheus metric `jobs_total`, and returns `202 Accepted`.
    - `GET /api/v1/jobs`: returns validated paginated job collection.
    - `GET /api/v1/jobs/:job_id` and `GET /api/v1/jobs/:job_id/artifacts/:format`: path parameter validation endpoints.
  - Added `CoreError::InvalidRequestSchema` to `crates/core/src/error.rs` and implemented `Validatable` for all domain IDs in `crates/core/src/id.rs`.
  - Added 14 integration tests in `services/api/src/lib.rs` and 5 unit tests in `services/api/src/dto/mod.rs`, expanding workspace tests to 78/78 passing. Verified all 5 quality gates pass.

### 2026-11-17 — Phase 2.2: API (`2.2.1`)

#### Added
- **`[2.2.1]` Implement Axum Service and Router Scaffolding in `services/api`:**
  - Implemented application state container `AppState` (`services/api/src/state.rs`) holding shared references to `Arc<AppConfig>` and `Arc<AppMetrics>`.
  - Implemented HTTP middleware pipeline (`services/api/src/middleware.rs`):
    - Configurable `CorsLayer` parsing `cors_allowed_origins` from configuration, with support for standard methods (`GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`, `HEAD`) and headers.
    - Global request `TimeoutLayer` bounding execution time according to `server.request_timeout_secs`.
    - Distributed tracing and correlation middleware (`trace_id_middleware`) preserving or generating `x-request-id` (UUID v7) and W3C `traceparent` headers across request and response lifecycles.
  - Implemented HTTP router and handlers (`services/api/src/routes/`):
    - Root landing handler `/` returning JSON metadata, service name, and API v1 endpoints descriptor.
    - Container orchestrator liveness probe `/api/v1/health` and readiness probe `/api/v1/ready`.
    - Prometheus metrics exposition handler `/api/v1/metrics` serving `text/plain` Prometheus telemetry.
    - Fallback handler returning RFC 7807 compliant `ApiErrorEnvelope` with `RESOURCE_NOT_FOUND` on unmapped routes.
  - Implemented server lifecycle (`services/api/src/server.rs`):
    - `build_app()` assembling middleware layers and route tree.
    - `run_server()` binding TCP listener to configured host and port with graceful termination on `Ctrl+C` / `SIGINT` (and Unix `SIGTERM`).
  - Updated binary entry point `services/api/src/main.rs` initializing `init_tracing()` and `AppConfig::from_env()`.
  - Added 8 integration tests and 1 binary metadata test in `services/api`, expanding workspace test coverage to 59/59 passing tests. Verified all 5 gates pass cleanly.

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.3`) & Phase 2.1 Milestone Certification

#### Added
- **`[2.1.3]` Set Up Structured Logging, OpenTelemetry Tracing, and Prometheus Metrics in `crates/observability`:**
  - Implemented automated secret redaction engine in `crates/observability/src/redaction.rs`:
    - Sanitizes GitHub Personal Access Tokens (`ghp_*`, `gho_*`, `github_pat_*`), OAuth Bearer tokens, embedded basic auth credentials in URLs (`user:pass@host`), provider API keys (OpenAI `sk-*`, Anthropic `sk-ant-*`, AWS `AKIA*`), and private key blocks.
    - Provided `redact_sensitive_text()`, `is_sensitive_key()`, and recursive `redact_json_value()`.
  - Implemented OpenTelemetry and W3C TraceContext distributed trace propagation in `crates/observability/src/trace.rs`:
    - Defined `TraceContext` representation supporting 128-bit `trace_id`, 64-bit `span_id`, and `trace_flags`.
    - Implemented W3C `traceparent` parsing (`from_traceparent`) with all-zero rejection, formatting (`to_traceparent`), and child span context derivation (`new_child()`).
  - Implemented Prometheus-compatible metrics registry in `crates/observability/src/metrics.rs`:
    - Lock-free thread-safe atomic `Counter`, `CounterVec`, `Gauge`, and `Histogram` (with standard latency buckets).
    - Defined `AppMetrics` tracking `jobs_total`, `job_duration_ms`, `extracted_files_total`, `tokens_processed_total`, `storage_bytes_current`, `active_workers`, and `ssrf_blocks_total`.
    - Implemented `render_prometheus_text()` outputting valid Prometheus text exposition format.
  - Implemented structured logging subscriber in `crates/observability/src/logging.rs`:
    - Defined `LoggingConfig` and `LogFormat` (`Json`, `Pretty`, `Compact`).
    - Implemented idempotent `init_tracing()` with dynamic `EnvFilter` from `RUST_LOG` or default level.
  - Added 16 unit tests in `crates/observability`, expanding total automated Rust tests across the workspace to 50/50 passing.
  - Formally certified completion of **Phase 2.1: Rust architecture** (`2.1.1` – `2.1.3` all `ACCEPTED`).

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.2`)

#### Added
- **`[2.1.2]` Implement Typed Domain Errors, Results, and Configuration Loader in `crates/core`:**
  - Implemented typed domain error taxonomy in `crates/core/src/error.rs`:
    - Defined machine-readable `ErrorCode` enum and HTTP status code mappings (`INVALID_REQUEST_SCHEMA`, `UNSUPPORTED_URL_SCHEME`, `SSRF_ADDRESS_BLOCKED`, `UNAUTHORIZED_GITHUB_REPO`, `RESOURCE_NOT_FOUND`, `RATE_LIMIT_EXCEEDED`, `PAYLOAD_TOO_LARGE`, `EXTRACTION_TIMEOUT`, `CONFIG_ERROR`, `VALIDATION_ERROR`, `STORAGE_ERROR`, `JOB_ENGINE_ERROR`, `INTERNAL_SERVER_ERROR`).
    - Implemented `CoreError` with `thiserror`, standard `Result<T, E = CoreError>` type alias, and conversion into RFC 7807 compliant `ApiErrorEnvelope` / `ApiError` payloads.
  - Implemented strongly-typed domain identifiers in `crates/core/src/id.rs`:
    - Defined time-ordered UUID v7 newtypes: `JobId`, `DocumentId`, `TenantId`, `RequestId`, `ArtifactId`.
    - Implemented `Display`, `FromStr`, `From<Uuid>`, `Into<Uuid>`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, and transparent Serde serialization.
  - Implemented hierarchical configuration loader in `crates/core/src/config.rs`:
    - Defined `AppConfig` and modular sub-configurations: `ServerConfig`, `DatabaseConfig`, `StorageConfig`, `SecurityConfig`, `ExtractionConfig`, `OptimizationConfig`.
    - Provided default development values requiring zero mandatory environment variables.
    - Implemented `from_env()` parsing environment variable overrides (`PORT`, `HOST`, `DATABASE_URL`, `STORAGE_DIR`, etc.).
    - Implemented self-validation enforcing ports, non-zero payloads, connection bounds, budget ceilings, and strict production SSRF invariants (`allow_private_ips` prohibited in production).
  - Implemented core shared traits in `crates/core/src/traits.rs`: `Identifiable`, `Clock`, `SystemClock`, and `Validatable`.
  - Added 18 unit tests in `crates/core`, increasing total workspace tests to 34/34 passing. Verified all 5 gates pass cleanly.

### 2026-11-17 — Phase 2.1: Rust Architecture (`2.1.1`)

#### Added
- **`[2.1.1]` Create Rust Workspace and 17 Crate Structures:**
  - Initialized monorepo root `Cargo.toml` with `[workspace]` configuration, unified lints, and workspace dependency management:
    - Declared 2 backend services (`services/api`, `services/worker`) and 15 domain/processing crates (`crates/core`, `crates/cdm`, `crates/url-security`, `crates/web-extractor`, `crates/github-extractor`, `crates/code-intelligence`, `crates/normalizer`, `crates/deduplication`, `crates/tokenizer`, `crates/context-optimizer`, `crates/chunking`, `crates/exporters`, `crates/storage`, `crates/job-engine`, `crates/observability`).
    - Configured package metadata defaults (`edition = "2021"`, `version = "0.1.0"`, `license = "MIT OR Apache-2.0"`).
    - Enforced strict workspace compiler warnings (`rust.unsafe_code = "deny"`, `rust.unused_must_use = "deny"`, `clippy.all = "warn"`, `clippy.pedantic = "warn"`, `clippy.correctness = "deny"`).
    - Pinned internal and third-party dependencies (`tokio`, `serde`, `tracing`, `thiserror`, `anyhow`, `chrono`) with exact versions and path mappings.
  - Initialized all 17 member crates with compilable `Cargo.toml` and entry points (`src/lib.rs` / `src/main.rs`), each providing crate metadata functions and passing unit tests.
  - Configured `deny.toml` compatible with cargo-deny 0.20+ (license validation, bans, security advisories, and path dependency wildcard allowances).
  - Configured `rustfmt.toml` with stable formatting options compatible with Rust 1.97.1.
  - Updated `scripts/check-all.sh` and `scripts/check-all.ps1` with robust cross-platform `cargo` binary detection and `cargo deny check` quality gating.
  - Verified 17/17 Rust unit tests pass, `cargo check --workspace` compiles cleanly in 0.64s, `cargo fmt` and `cargo clippy` report 0 warnings/errors, and all 5 monorepo verification gates pass.

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.3`), Phase 1.4 Certification & Stage 1 Milestone Certification

#### Added
- **`[1.4.3]` UI Performance Benchmarks, Asset Budget Audits, Design Token Regression & Stage 1 Certification:**
  - Authored comprehensive performance and visual regression test suite `apps/web/tests/ui-performance.test.ts` (11 tests across 3 categories):
    - **Production Bundle Size & Dependency Budget:** Asserted `package.json` contains zero external runtime dependencies (`dependencies` is empty); verified absence of bloated third-party libraries (no lodash, moment, prismjs, highlight.js, rxjs, lucide-react, MUI, Chakra); verified client build asset footprints strictly satisfy budgets (largest JS chunk $57.4\text{ kB} \le 100\text{ kB}$ raw, largest CSS chunk $44.1\text{ kB} \le 60\text{ kB}$ raw, total client bundle $\sim 260\text{ kB} \le 350\text{ kB}$ raw).
    - **Algorithmic Throughput & Micro-Benchmarks:** Verified high-throughput execution for syntax highlighting ($> 20,000\text{ lines/sec}$ on 1,000 JSON lines; $< 40\text{ms}$ on 500+ XML lines), token budget utilization evaluation (10,000 calls in $< 50\text{ms}$, $> 200,000\text{ ops/sec}$), URL classification and SSRF pre-checking (5,000 calls in $< 50\text{ms}$), and Web Crypto SHA-256 integrity digests (500 KB payload in $< 50\text{ms}$).
    - **Visual Design System & Styling Token Regression:** Verified full CSS custom properties export in `tokens.css` (`--color-brand-*`, `--color-accent-*`, `--color-surface-*`, `--color-status-*`, `--elevation-*`, `--spacing-*`, `--radius-*`, `.dark` theme overrides); verified Tailwind semantic utility classes (`.bg-theme-canvas`, `.bg-theme-panel`, `.bg-theme-subtle`, `.text-theme-primary`, `.text-theme-secondary`, `.text-theme-muted`, `.border-theme-subtle`, `.border-theme-strong`); verified WCAG AAA contrast compliance across both Dark mode ($\ge 14:1$ canvas, $\ge 12:1$ panel) and Light mode ($\ge 14:1$), plus AA for brand buttons ($\ge 4.5:1$); audited all 12 frontend components confirming zero inline arbitrary hex color pollution.
  - Expanded total automated test suite to 184/184 tests passing across 15 test suites with 0 failures.
  - Formally certified completion of **Phase 1.4: UX validation** across all 3 subphases (`1.4.1` – `1.4.3` all `ACCEPTED`).
  - Formally certified completion of **Stage 1: Frontend and Product Experience** across all 15 subphases (`1.1.1` – `1.4.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.2`)

#### Added
- **`[1.4.2]` Mobile, Tablet, and Desktop Viewport Tests & Responsive Invariant Suite:**
  - Expanded `apps/web/tests/responsive-layout.test.ts` into a multi-viewport certification test suite (28 tests across 6 sections):
    - **Tailwind Breakpoint Spectrum:** Verified resolution mapping across representative devices for Mobile (`xs`: 320px–480px, including iPhone SE, iPhone 14/15, Pixel 7), Tablet (`sm`: 640px, `md`: 768px–834px, `lg`: 1024px, including iPad Mini, iPad 10th Gen, iPad Air), and Desktop (`xl`: 1280px–1440px, `2xl`: 1536px–2560px, including standard laptops, QHD, and 1080p FHD displays).
    - **Mobile Viewport Invariants (320px–480px):** Verified horizontal overflow prevention (`overflow-x-hidden`, `max-w-full`, `min-h-screen`), standard responsive container padding (`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8`), Hero typography scaling (`text-3xl sm:text-5xl lg:text-6xl`), full-width action button stacking (`flex-col`, `w-full min-h-[48px]`), format pills 2-column collapse, value metric 1-column collapse, `UrlInput` vertical stacking with iOS auto-zoom prevention (`text-sm sm:text-base`), content grids single-column collapse, and Header mobile hamburger menu button (`md:hidden`).
    - **Tablet Viewport Invariants (640px–1024px):** Verified horizontal button flow at `sm` (`sm:flex-row`), 3-column format pills (`sm:grid-cols-3`), 3-column metric cards (`sm:grid-cols-3`), 3-column workflow cards (`md:grid-cols-3`), 2-column sources and feature grids (`md:grid-cols-2`), and Header desktop navigation visibility (`hidden md:flex`).
    - **Desktop Viewport Invariants (1280px–1920px+):** Verified 6-column format matrix (`lg:grid-cols-6`), 3-column feature highlights (`lg:grid-cols-3`), 5-column footer matrix (`lg:grid-cols-5`), maximum headline typography (`lg:text-6xl`), and container constraints preventing ultra-wide stretching (`max-w-7xl mx-auto`, `max-w-6xl mx-auto`).
    - **Conversion Workspace Viewport Invariants:** Verified workspace header responsive alignment (`flex-col sm:flex-row items-start sm:items-center`), 4-stage pipeline execution cards collapsing (1-col mobile, 2-col tablet, 4-col desktop), completeness accounting matrix (2-col mobile, 4-col tablet+), `TokenBudgetBar` metric grid, `PreviewPanel` 12-column responsive layout with sidebar stacking on mobile and side-by-side splitting on tablet/desktop (`grid-cols-1 md:grid-cols-12`), `DownloadControls` artifact cards (`flex-col md:flex-row md:items-center`), and bounded execution telemetry log scroll (`max-h-64 overflow-y-auto`).
    - **Touch Target Accessibility (WCAG 2.1 AA/AAA):** Verified $\ge 44 \times 44\text{px}$ touch targets for Header hamburger toggle and theme buttons, $\ge 44\text{px}$ touch heights for mobile dropdown links, and $\ge 48\text{px}$ touch heights for Hero CTA and `UrlInput` controls.
  - Expanded total automated test suite to 173/173 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.4: UX Validation (`1.4.1`)

#### Added
- **`[1.4.1]` Keyboard Navigation and Accessibility Audits (WCAG 2.1 AA & AAA):**
  - Authored comprehensive accessibility audit test suite `apps/web/tests/a11y-audit.test.ts` (19 tests) asserting compliance across 8 distinct accessibility dimensions:
    - **Semantic HTML Landmarks:** Validated `<header>`, `<nav aria-label="Main Navigation">`, `<main id="main-content">`, and `<footer>` on both landing page and workspace routes; verified explicit `<section>` regions with `aria-labelledby` across `Hero.svelte`, `WorkflowSteps.svelte`, `SupportedSources.svelte`, and `FeatureHighlights.svelte`.
    - **WAI-ARIA Progressbar Pattern:** Validated compliance in `JobWorkspace.svelte` (overall pipeline progress) and `TokenBudgetBar.svelte` (context utilization progressbar) asserting `role="progressbar"`, dynamic `aria-valuenow`, `aria-valuemin={0}`, `aria-valuemax={100}`, and descriptive `aria-label`.
    - **WAI-ARIA Tabs Design Pattern:** Validated bidirectional linkage in `PreviewPanel.svelte` format tabs asserting `role="tablist"`, interactive tab controls (`role="tab"`, `id="tab-{id}"`, dynamic `aria-selected`, `aria-controls`), and panel containers (`role="tabpanel"`, `id="panel-{id}"`, `aria-labelledby`).
    - **Polite Live Regions & Assertive Alerts:** Verified non-disruptive feedback live regions (`role="status"`, `role="log"`, `aria-live="polite"`) across `UrlInput`, `JobWorkspace`, `PreviewPanel`, and `DownloadControls`, alongside assertive error and budget overflow banners (`role="alert"`).
    - **Screen Reader Exclusion of Decorative Content:** Verified `aria-hidden="true"` on line number gutter cells (`line.lineNumber`) and decorative background SVGs and visual effects.
    - **Universal Visible Focus Indicators:** Verified WCAG 2.4.7 visible focus styles (`focus-visible:ring` and `focus-visible:outline`) across interactive primitives (`Button`, `Input`, `Toggle`, `UrlInput`, `ConversionSettings`, `PreviewPanel`, `TokenBudgetBar`, `DownloadControls`).
    - **Color Contrast Certification:** Mathematically verified WCAG AAA ($\ge 7.0:1$) compliance for dark canvas ($14.0:1$) and panels ($12.0:1$) with white text, light canvas with dark slate text ($14.0:1$), and WCAG AA ($\ge 4.5:1$) for brand primary buttons.
    - **Keyboard Navigation Shortcuts:** Validated Escape key listeners in `UrlInput` (clearing input) and `Modal` (dialog dismissal), and Enter/Space keyboard toggling on `Toggle` switches (`role="switch"`, `aria-checked`).
  - Total automated test suite expanded to 152/152 tests passing across 14 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.4`) & Phase 1.3 Certification

#### Added
- **`[1.3.4]` Export Artifact Download Controls, SHA-256 Verification & Phase 1.3 Certification:**
  - Defined export artifact contracts and cryptographic SHA-256 generators in `apps/web/src/lib/types/download.ts`:
    - `computeSha256Hex`: Generates standard 64-character hexadecimal digests via Web Crypto API (`crypto.subtle.digest`).
    - `deriveArtifactBaseName`: Sanitizes filesystem-safe artifact base names from GitHub repositories and web documentation URLs.
    - `buildExportArtifacts`: Asynchronously generates concrete `ExportArtifact` records for all 5 formats with real content, byte sizes, and SHA-256 digests.
    - `generateChecksumsTxt`: Produces standard coreutils `sha256sum` formatted checksum lists (`<hash>  <filename>`).
    - `generateChecksumManifestJson`: Produces schema v1.0.0 JSON checksum manifests matching CDM specifications.
    - `triggerBlobDownload`: Client-side Blob download trigger with automatic Object URL cleanup.
  - Authored accessible, interactive `DownloadControls.svelte` in `apps/web/src/lib/components/workspace/DownloadControls.svelte`:
    - Region landmark (`role="region"`, `aria-label="Format Selection and Download Controls"`).
    - Batch download controls: "Download All Formats", "Checksums (.txt)", and "manifest.json".
    - Individual artifact cards with format icons, extension badges, byte sizes, truncated SHA-256 pills with one-click copy feedback, and download buttons.
    - Collapsible checksum manifest drawer rendering coreutils `checksums.txt` content.
    - Screen reader announcement live region (`role="status"`, `aria-live="polite"`).
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `DownloadControls` into `JobWorkspace.svelte` directly below the preview panel.
  - Authored comprehensive automated test suite `apps/web/tests/download-controls.test.ts` (11 tests) validating SHA-256 hashing, base name derivation, artifact generation, checksums.txt formatting, manifest JSON schema, and component ARIA landmarks.
  - Expanded total automated test suite to 133/133 passing tests across 13 test suites.
  - Formally certified completion of **Phase 1.3: Conversion workspace** across all 4 subphases (`1.3.1` – `1.3.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.3`)

#### Added
- **`[1.3.3]` Token Statistics Interface, Context Window Budget Bar & Headroom Metrics:**
  - Defined token budget evaluation domain model and calculations in `apps/web/src/lib/types/token-budget.ts`:
    - Mathematical helpers: `calculateSavingsPct` (token reduction ratio), `calculateUtilizationPct` (budget consumption), and `calculateHeadroom` (available conversational reasoning buffer).
    - Evaluation engine `evaluateBudgetUtilization` supporting 3 status levels: `optimal` (utilization $\le 85\%$), `warning` ($> 85\%$ and $\le 100\%$), and `exceeded` ($> 100\%$).
    - Calibration across AI model context windows (Gemini 2M, Gemini 1M, Claude 200k, GPT-4o 128k, Local 32k).
  - Authored accessible, interactive `TokenBudgetBar.svelte` in `apps/web/src/lib/components/workspace/TokenBudgetBar.svelte`:
    - Region landmark (`role="region"`, `aria-label="Token Statistics and Context Budget Bar"`).
    - 4-card metric grid: Raw Ingested Tokens, Deduplicated Tokens, Final Context Output, and Token Optimization Savings with reduction badge.
    - Dual-layer progress bar (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`) showing Final Tokens, ghost Deduplication Savings segment, and 80% safety margin threshold line.
    - Dynamic model preset switcher allowing instant re-evaluation against all major model context windows.
    - Headroom callout displaying remaining tokens for conversation prompts and completions.
    - Accessible context overflow alert banner (`role="alert"`) with one-click model upgrade switch to Gemini 1.5 Pro.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `TokenBudgetBar` into `JobWorkspace.svelte` positioned between the 4-stage pipeline execution cards and completeness accounting matrix.
  - Authored comprehensive automated test suite `apps/web/tests/token-budget.test.ts` (15 tests) validating savings math, headroom math, utilization bounds, ARIA landmarks, preset switching, and workspace integration.
  - Expanded total automated test suite to 122/122 passing tests across 12 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.2`)

#### Added
- **`[1.3.2]` Syntax-Highlighted Content Preview Panel & Multi-Format Exporters:**
  - Defined preview domain contracts and multi-format exporters in `apps/web/src/lib/types/preview.ts`:
    - 5 standard export formats: Markdown (`.md`), JSON CDM schema v1.0.0 (`.json`), Claude XML (`.xml`), Plaintext (`.txt`), and JSONL (`.jsonl`).
    - Canonical Context Document Model (CDM) serializer matching `docs/DATA_MODEL.md` with completeness accounting.
    - Claude context XML hierarchy serializer with `<documents>`, `<document index="...">`, and `<document_content>`.
    - Deterministic token estimation heuristic `estimateTokens` and sample extracted repository/article generator `generateSampleExtractedFiles`.
  - Authored zero-dependency, high-performance syntax highlighter in `apps/web/src/lib/utils/syntax-highlighter.ts`:
    - Strict HTML escaping (`escapeHtml`) preventing cross-site scripting (XSS).
    - Syntax token classification and styling for JSON (keys, strings, numbers, booleans, null), XML (tags, attributes, values, comments), Markdown (headings, blockquotes, code blocks, lists), and general code (keywords, comments, strings, numbers).
    - Line splitter `highlightLines` with 1-based line numbers.
  - Authored accessible, interactive `PreviewPanel.svelte` in `apps/web/src/lib/components/workspace/PreviewPanel.svelte`:
    - Accessible tablist (`role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`).
    - Extracted files sidebar with instant path filtering, file icons, byte size, token counts, and full concatenated context mode.
    - Copy to clipboard button with animated visual confirmation (`✓ Copied!`) and polite ARIA announcement (`role="status"`, `aria-live="polite"`).
    - Line numbers gutter with `aria-hidden="true"`, word wrap toggle, search within preview text, and direct format download.
    - Context metrics banner displaying estimated tokens, line counts, and byte sizes.
    - Re-exported via `apps/web/src/lib/components/workspace/index.ts`.
  - Integrated `PreviewPanel` into `JobWorkspace.svelte` with toggleable visibility and automatic target format synchronization.
  - Authored comprehensive automated test suite `apps/web/tests/preview-panel.test.ts` (17 tests) validating domain models, exporters, syntax highlighters, ARIA accessibility, and workspace integration.
  - Expanded total automated test suite to 107/107 passing tests across 11 test suites.

### 2026-11-17 — Phase 1.3: Conversion Workspace (`1.3.1`)

#### Added
- **`[1.3.1]` Conversion Job Workspace, Pipeline Tracker & File Completeness Accounting:**
  - Defined conversion job domain model, state transitions, and file completeness invariant in `apps/web/src/lib/types/job.ts`:
    - Strict lifecycle states: `queued`, `leased`, `running`, `completed`, `partial`, `failed`, `cancelled`.
    - Standard 4-stage pipeline definitions: Acquisition & SSRF Defense, CDM Canonicalization, MinHash Deduplication, Export & Token Fit.
    - Verified invariant helper `verifyCompletenessInvariant` enforcing $\text{discovered} = \text{extracted} + \text{excluded} + \text{failed}$ per `docs/DATA_MODEL.md` to prevent silent file drops.
    - Weighted progress calculator `calculateOverallProgress` and deterministic mock-free factory `createInitialJob`.
  - Authored accessible, responsive `JobWorkspace.svelte` component in `apps/web/src/lib/components/workspace/JobWorkspace.svelte`:
    - Overall progress bar landmark (`role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`).
    - 4-stage execution cards highlighting active, completed, failed, and pending stages with step numbers and status badges.
    - Repository completeness accounting matrix displaying discovered, extracted, excluded, and failed counts with verification badge.
    - Live execution telemetry log terminal with windowed DOM slice (most recent 200 items per Rule 03 bounded memory) and `role="log"`, `aria-live="polite"`.
    - Failure recovery alert banner (`role="alert"`) with error diagnostics and interactive retry trigger.
    - Action buttons for interactive job cancellation and pipeline retry.
  - Exported component via `apps/web/src/lib/components/workspace/index.ts`.
  - Authored dedicated workspace route in `apps/web/src/routes/workspace/+page.svelte`:
    - Reads query parameters (`?url=...&format=...&preset=...&budget=...`) on client load with fallback to initial verified target.
    - Integrated with site `Header`, `Footer`, breadcrumbs navigation, and SEO meta tags.
  - Updated `apps/web/src/routes/+page.svelte` to provide direct "Open Workspace →" action in conversion feedback banner.
  - Authored comprehensive automated test suite `apps/web/tests/job-workspace.test.ts` (14 tests) validating pipeline stages, job factories, completeness invariants, weighted progress math, ARIA attributes, and route integration.
  - Expanded total automated unit and invariant test suite to 90/90 passing tests across 10 test suites.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.4`) & Phase 1.2 Certification

#### Added
- **`[1.2.4]` Responsive Layout Auditing, Mobile Touch Targets & Phase 1.2 Certification:**
  - Audited and hardened responsive layout rules across all Phase 1.2 components:
    - Root layout (`apps/web/src/routes/+layout.svelte`) configured with `w-full max-w-full overflow-x-hidden` ensuring zero horizontal page scrolling across mobile viewports down to `320px`.
    - Enforced WCAG 2.1 AA recommended touch target sizing ($\ge 44 \times 44\text{px}$) across mobile hamburger button, theme switch toggle, mobile navigation drawer links, clear buttons, and submit controls.
    - Updated `Hero.svelte` action buttons with mobile full-width vertical stacking and $\ge 48\text{px}$ touch targets.
    - Updated `UrlInput.svelte` submit button with mobile full-width stacking (`w-full sm:w-auto min-h-[48px]`).
  - Authored automated responsive layout test suite `apps/web/tests/responsive-layout.test.ts` (7 tests) validating overflow containment, container padding, responsive grid collapsing, and touch target minimums.
  - Total automated test suite count expanded to 76/76 passing tests with zero failures across 9 test suites.
  - Formally certified completion of **Phase 1.2: Main website** across all 4 subphases (`1.2.1` – `1.2.4` all `ACCEPTED`).

### 2026-11-17 — Phase 1.2: Main Website (`1.2.3`)

#### Added
- **`[1.2.3]` Conversion Settings Panel, AI Model Presets & Content Normalization Toggles:**
  - Defined conversion settings contracts and calibrated AI model presets in `apps/web/src/lib/types/settings.ts`:
    - Model presets for Gemini 1.5 Pro (2M window), Gemini 1.5 Flash (1M window), Claude 3.5 Sonnet (200k window), GPT-4o (128k window), Local LLMs (32k/128k windows), and Custom budget.
    - CDM normalization and filtering options: `stripBoilerplate`, `stripCodeComments`, `excludeTests`, `enableDynamicRendering`, `semanticDeduplication`, `maxFileSizeKb`.
    - Validation utility enforcing budget bounds (1,000 to 2,000,000 tokens) and file size bounds (64 to 10,240 KB).
  - Authored accessible conversion settings component in `apps/web/src/lib/components/settings/ConversionSettings.svelte`:
    - Collapsible disclosure panel with `aria-expanded`, `aria-controls`, and `role="region"`.
    - Preset selection grid updating calibrated token ceilings.
    - Context window token budget range slider with live percentage progress bar and tabular number formatting.
    - Accessible switches (`Toggle`) for boilerplate stripping, code comment stripping, test exclusion, and MinHash semantic deduplication.
    - Dynamic Chromium rendering fallback toggle and file size threshold number input.
    - Exported component via `apps/web/src/lib/components/settings/index.ts`.
  - Integrated `ConversionSettings` into `apps/web/src/routes/+page.svelte` in `#convert`, binding settings state directly to conversion job feedback.
  - Authored automated unit and component invariant test suite `apps/web/tests/conversion-settings.test.ts` (6 tests), expanding total test suite count to 69/69 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.2`)

#### Added
- **`[1.2.2]` URL Input Component, Real-Time Source Detection & SSRF Pre-validation Engine:**
  - Authored URL detection and classification utility `apps/web/src/lib/utils/url-detector.ts`:
    - Full classification supporting standard public GitHub repositories, branch/tree subpaths, file blobs, XML sitemaps, sitemap indexes, and generic web documentation.
    - Implemented client-side zero-trust SSRF pre-check (`isBlockedHost`) preventing submission to loopback addresses (`127.0.0.0/8`, `::1`), RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata endpoints (`169.254.169.254`, `metadata.google.internal`), and non-HTTP protocols (`file:`, `javascript:`, `data:`, `ftp:`).
    - Extracted GitHub metadata including owner, repo, ref, and subpath.
  - Authored dedicated accessible URL input component in `apps/web/src/lib/components/input/UrlInput.svelte`:
    - Accessible search form landmark (`role="search"`, `aria-label="URL Ingestion Form"`).
    - Dynamic source badge with ARIA live region (`role="status"`, `aria-live="polite"`).
    - Contextual leading icons matching detected source modalities (GitHub Octocat, Sitemap XML, Security Warning Shield, Globe).
    - Clear button control (`aria-label="Clear URL input"`), `Escape` key handling, and submit button with SVG loading spinner.
    - Accessible error alert banner (`role="alert"`) rendering explicit SSRF and syntax error diagnostics.
    - Exported component via `apps/web/src/lib/components/input/index.ts`.
  - Integrated `UrlInput` into `apps/web/src/routes/+page.svelte` replacing the prototype input in `#convert` with real-time classification, dynamic target format suggestion, and verified submission feedback.
  - Authored automated unit test suites `apps/web/tests/url-detector.test.ts` (14 tests) and `apps/web/tests/url-input-component.test.ts` (3 tests), expanding total test suite count to 63/63 passing tests.

### 2026-11-17 — Phase 1.2: Main Website (`1.2.1`)

#### Added
- **`[1.2.1]` Production Landing Page, Semantic Navigation & Deterministic Pipeline Showcase:**
  - Authored navigation components in `apps/web/src/lib/components/navigation/`:
    - `Header.svelte`: Sticky semantic navigation header with brand AI icon, stage badge, desktop anchor navigation (`#convert`, `#workflow`, `#sources`, `#features`), external documentation and GitHub repository badges, accessible dark/light mode toggle with dynamic ARIA attributes, and responsive mobile dropdown drawer.
    - `Footer.svelte`: Semantic footer (`role="contentinfo"`) with copyright, MIT license, compliance badges (CPU-only 16 GB, <= 30 GB Podman, $0 / zero API costs), and external links to system architecture, SSRF defense, CDM data model, and performance targets.
    - `index.ts`: Unified navigation exports.
  - Authored landing feature components in `apps/web/src/lib/components/landing/`:
    - `Hero.svelte`: Production hero headline, value proposition subtitle, primary "Start Conversion" and "Explore How It Works" call-to-action buttons, 6 multi-format exporter pills (`.md`, `.json`, `.xml`, `.txt`, `.jsonl`, `.zip`), and hardware/operational metric badges.
    - `WorkflowSteps.svelte`: 3-step structured cards detailing Acquisition & SSRF Defense, CDM Canonicalization & Optimization, and Multi-Format Export & Token Fit.
    - `SupportedSources.svelte`: 4-source responsive grid matrix highlighting Static Documentation (HTML5 parser), Dynamic JavaScript SPAs (isolated Chromium fallback), Public GitHub Repositories (Git Trees API), and Private Repositories (GitHub App ephemeral tokens).
    - `FeatureHighlights.svelte`: 6 architectural pillar cards covering zero-trust SSRF defense, AST code intelligence, zero mandatory paid APIs, 30 GB Podman storage ceiling, deterministic token budgeting, and privacy-first self-hostability.
    - `index.ts`: Unified landing exports.
  - Assembled root `apps/web/src/routes/+page.svelte` integrating semantic landmarks (`header`, `main`, `footer`), quick conversion entry with URL input, reactive source detection, format selector pills, and live verification feedback banner.
  - Enhanced `apps/web/src/lib/components/ui/Button.svelte` with optional `href`, `target`, and `rel` attributes for accessible link buttons.
  - Enhanced `apps/web/src/lib/components/ui/Input.svelte` with optional `ariaLabel` attribute for headless input labeling.
  - Authored comprehensive test suite `apps/web/tests/landing-page.test.ts` validating semantic landmarks, navigation links, export formats, 3-step pipeline, 4 sources, and 6 feature pillars (expanding total passing tests to 46/46).

### 2026-11-17 — Phase 1.1: Design System (`1.1.3`)

#### Added
- **`[1.1.3]` Reusable Accessible UI Component Primitives & Phase 1.1 Completion:**
  - Authored accessible UI components in `apps/web/src/lib/components/ui/`:
    - `Button.svelte`: 5 variants (`primary`, `secondary`, `outline`, `ghost`, `danger`), 3 sizes (`sm`, `md`, `lg`), SVG loading spinner (`aria-busy`), disabled state, keyboard focus rings.
    - `Input.svelte`: accessible text/url input with label, helper text, error message, clear button, and ARIA described-by linkage.
    - `Toggle.svelte`: accessible switch primitive with `role="switch"`, `aria-checked`, label, description, and keyboard toggle (`Space`, `Enter`).
    - `Badge.svelte`: status badge with 6 semantic variants and optional status dot.
    - `Modal.svelte`: accessible dialog with backdrop overlay, `Escape` key dismissal, click-outside dismissal, `role="dialog"`, `aria-modal="true"`, and header/footer action slots.
    - `Tooltip.svelte`: contextual popover appearing on hover and keyboard focus (`role="tooltip"`).
  - Exported all UI primitives via `apps/web/src/lib/components/ui/index.ts`.
  - Updated showcase page `apps/web/src/routes/+page.svelte` demonstrating live interactive UI components.
  - Authored unit test suite `apps/web/tests/ui-components.test.ts`, expanding total test suite count to 37/37 passed tests.
  - Formally certified completion of **Phase 1.1: Design System** (Subphases `1.1.1`, `1.1.2`, `1.1.3` all `ACCEPTED`).

### 2026-11-17 — Phase 1.1: Design System (`1.1.2`)

#### Added
- **`[1.1.2]` Typography Hierarchy, Spacing Primitives & WCAG Contrast Systems:**
  - Authored accessible typography scale components in `apps/web/src/lib/components/typography/`: `Heading.svelte` supporting levels 1–6 (`h1`–`h6`) with proportional sizes `xs` to `5xl`, `Text.svelte` supporting body sizes, weights, and semantic tone variants, `Code.svelte` for inline and block code, and `Mono.svelte` for tabular numbers and token metrics.
  - Authored layout primitives in `apps/web/src/lib/components/layout/`: `Container.svelte` with responsive breakpoint max-widths, `Stack.svelte` supporting vertical/horizontal direction and 4px baseline gap spacing (`gap-0` to `gap-12`), and `Grid.svelte` supporting responsive 1–12 columns.
  - Implemented color contrast and accessibility utilities in `apps/web/src/lib/tokens/color-utils.ts` verifying WCAG 2.1 AA/AAA compliance thresholds, automatic text contrast selection, token budget status badge styling, and universal visible focus rings.
  - Published comprehensive design system specification in `docs/DESIGN_SYSTEM.md` covering typography hierarchy, 4px baseline rhythm, container breakpoints, surface layers, token budget bands, and WCAG AA/AAA certification tables.
  - Updated showcase page `apps/web/src/routes/+page.svelte` integrating typography and layout components with interactive budget controls.
  - Authored unit test suites `apps/web/tests/typography.test.ts` and `apps/web/tests/color-utils.test.ts`, expanding total test count to 31/31 passed tests.

### 2026-11-17 — Phase 1.1: Design System (`1.1.1`)

#### Added
- **`[1.1.1]` SvelteKit 2 + Svelte 5 Foundation & Visual Design Tokens:**
  - Initialized `apps/web` with SvelteKit 2, Svelte 5 runes (`$state`, `$derived`, `$props`), strict TypeScript, Tailwind CSS, PostCSS, Autoprefixer, `@sveltejs/adapter-node`, Vitest, and Prettier.
  - Implemented atomic design token module `apps/web/src/lib/tokens/index.ts` defining brand primary AI violet/indigo (`#6366f1`), accent telemetry cyan (`#06b6d4`), neutral slate surfaces (`#090d16` canvas to `#f8fafc`), semantic status indicators (`success`, `warning`, `error`, `info`), and context token budget indicators (`safe` <=80%, `warning` >80%, `critical` >100%).
  - Implemented CSS custom properties stylesheet `apps/web/src/lib/styles/tokens.css` with `:root` and `.dark` variables, alongside base styling `apps/web/src/lib/styles/app.css`.
  - Configured `apps/web/tailwind.config.ts` cleanly integrating design tokens into the Tailwind theme engine.
  - Authored root layout `+layout.svelte` and interactive token budget test showcase `+page.svelte`.
  - Authored automated Vitest suite `apps/web/tests/tokens.test.ts` verifying token completeness, hex regex validation, budget threshold logic, WCAG AAA dark background contrast (>= 7:1), WCAG AA brand contrast (>= 4.5:1), and CSS variable flattening.
  - Verified all 5 monorepo verification gates with 0 errors across PowerShell and POSIX runners.

---

## Stage 0: Project Bootstrap and Engineering Foundation (`COMPLETED` / `ACCEPTED`)

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.5`)

#### Added
- **`[0.3.5]` Setup Documentation, Disaster Recovery Runbooks & Stage 0 Certification:**
  - Authored comprehensive Environment Variables Reference in `docs/DEPLOYMENT.md` detailing all 19 configuration variables from `deploy/.env.example` (PostgreSQL, Axum API, Worker, Playwright, SvelteKit, and GitHub App credentials).
  - Authored PostgreSQL Logical Backup & Restore Runbook (`pg_dump`/`pg_restore`) for both Windows PowerShell and POSIX bash with clean table recreation and volume copying.
  - Authored Artifact Storage Volume Backup & Restore Runbook for `deploy_artifact_data`.
  - Authored Emergency Cold Disaster Recovery Runbook guaranteeing complete system recovery from cold workstation restart or storage corruption in under 5 minutes.
  - Authored Production Hardening Guidelines covering Caddy automatic HTTPS reverse proxy configuration with TLS headers, secret manager injection, block-level encryption at rest, AES-256-GCM token encryption, read-only container filesystems, dropped Linux capabilities, and container resource limits.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 223/223 automated checks.
  - Formally certified completion of **Stage 0: Project Bootstrap and Engineering Foundation** (Phases `0.1`, `0.2`, `0.3`, 15 subphases `0.1.1` – `0.3.5` all `ACCEPTED`).

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.4`)

#### Added
- **`[0.3.4]` Reproducible Container Builds & Architecture Validation:**
  - Authored `scripts/verify-container-builds.ps1` and `scripts/verify-container-builds.sh` auditing `deploy/podman-compose.yml` and all 4 Containerfiles.
  - Validated multi-stage build patterns, minimal slim/Alpine runtime base images, unprivileged runtime users (`appuser` UID 1000, `pwuser`, `node` UID 1000), network isolation boundaries (`browser_net` vs `internal_net`), and security sandbox flags (`no-new-privileges:true`).
  - Executed live container healthcheck verification on PostgreSQL 16 Alpine and validated persistent volume retention (`deploy_postgres_data`) across teardown.
  - Validated total Podman storage usage is strictly within the 30 GB ceiling (5.27 GB / 30 GB, 17.6%).
  - Documented container verification and reproducibility standards in `docs/DEPLOYMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 216/216 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.3`)

#### Added
- **`[0.3.3]` Development Environment Lifecycle Scripts:**
  - Authored `scripts/dev-up.ps1` and `scripts/dev-up.sh` for starting the compose stack, ensuring `deploy/.env` initialization, verifying the 30 GB storage ceiling prior to launch, supporting service subsets (`-Services`) and image rebuilding (`-Build`), and awaiting database healthcheck readiness.
  - Authored `scripts/dev-down.ps1` and `scripts/dev-down.sh` for service teardown with interactive protection for persistent database volumes (`postgres_data`, `artifact_data`) and post-teardown cache pruning (`-PruneCaches`).
  - Authored `scripts/dev-logs.ps1` and `scripts/dev-logs.sh` supporting service filtering, line tailing, and real-time log streaming (`-Follow`).
  - Authored `scripts/dev-restart.ps1` and `scripts/dev-restart.sh` for safe service restarts and healthcheck validation.
  - Authored `scripts/dev-status.ps1` and `scripts/dev-status.sh` reporting runtime container state, healthcheck status, exposed ports, and storage budget compliance in table and structured JSON formats.
  - Documented complete developer lifecycle workflow in `docs/DEVELOPMENT.md`.
  - Expanded bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 213/213 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.2`)

#### Added
- **`[0.3.2]` 30 GB Storage Ceiling Enforcement & Safe Cache Pruning:**
  - Authored `scripts/check-resources.ps1` and `scripts/check-resources.sh` to monitor host CPU cores, RAM (against 16 GB baseline), available disk space, and Podman storage breakdown (`Images`, `Containers`, `Local Volumes`) via `podman system df --format json`.
  - Enforced the 30 GB container storage ceiling (`NFR-PERF-02`, Rule 08) with automated 80% warning alerts and non-zero exit code on ceiling breach.
  - Authored `scripts/clean-caches.ps1` and `scripts/clean-caches.sh` for safe cache pruning (build caches, stopped build containers, dangling untagged layers) with dry-run inspection, before-and-after reclaimed byte calculation, and guaranteed retention of persistent database volumes (`postgres_data`).
  - Integrated Gate 5 (Resource Budget & 30 GB Storage Ceiling) into monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `docs/DEPLOYMENT.md` with complete resource monitoring commands, safety guarantees, and pruning workflows.
  - Extended bootstrap verification suite (`scripts/verify-bootstrap.ps1`) to 199/199 automated checks.

### 2026-11-17 — Phase 0.3: Containerized Development Foundation (`0.3.1`)

#### Added
- **`[0.3.1]` Podman Development Configuration:**
  - Authored `deploy/podman-compose.yml` declaring PostgreSQL 16 Alpine, Rust API service, Rust Worker, isolated Playwright Browser Worker, and SvelteKit Web frontend across two bridge networks (`internal_net`, `browser_net`) with persistent volumes and healthchecks.
  - Authored multi-stage `deploy/Containerfile.api` and `deploy/Containerfile.worker` with minimal Debian Bookworm slim runtimes and unprivileged `appuser`.
  - Authored `deploy/Containerfile.browser-worker` with unprivileged `pwuser`.
  - Authored multi-stage `deploy/Containerfile.web` with Node 20 slim runtime and unprivileged `node` user.
  - Authored `deploy/.env.example` documenting service environment variables.
  - Verified configuration via `podman-compose config` CLI and expanded test assertions to 194 checks.

### 2026-11-17 — Phase 0.2: GitHub and CI Foundation (`0.2.1` – `0.2.5`)

#### Added
- **`[0.2.5]` Security & Dependency Scanning:**
  - Configured `deny.toml` with strict RustSec advisory checks, permissive license allowlists (MIT, Apache-2.0, BSD, ISC, Unicode, PostgreSQL, CC0, Zlib, Unlicense), copyleft rejection, and registry restrictions.
  - Authored `.github/workflows/security.yml` executing secret leakage scans, `cargo-deny` checks, and `npm audit`.
  - Authored `.github/workflows/dependency-review.yml` gating PRs on dependency vulnerabilities and license compliance.
  - Authored `.github/workflows/codeql.yml` running automated CodeQL static security analysis.
  - Extended verification suite (`scripts/verify-bootstrap.ps1` & `scripts/check-all.ps1`) to 187 automated checks.
- **`[0.2.4]` Linting, Formatting & Quality Check Runners:**
  - Configured `rustfmt.toml` enforcing Rust 2021 edition, 100 character width, Unix LF line endings, import reordering, and comment formatting.
  - Configured `.prettierrc.json` and `.prettierignore` for web, JSON, Markdown, and TypeScript styling.
  - Authored cross-platform monorepo quality audit runners `scripts/check-all.ps1` and `scripts/check-all.sh`.
  - Updated `scripts/verify-bootstrap.ps1` with lint/format configuration assertions.
- **`[0.2.3]` GitHub Actions CI/CD Workflows:**
  - Authored `.github/workflows/ci.yml` with concurrency controls, bootstrap integrity checks, and conditional Rust and SvelteKit test gates.
  - Authored `.github/workflows/build.yml` with separate artifact uploads for frontend build bundle, backend binaries, and SHA-256 checksums.
  - Authored `.github/workflows/container.yml` auditing Containerfile specifications and enforcing the 30 GB storage ceiling.
  - Authored `.github/workflows/release.yml` with automated release gate verification, CycloneDX SBOM generation, SHA-256 checksums, and GitHub Release asset publishing.
  - Created `scripts/validate-workflows.py` for automated PyYAML validation and integrated it into `scripts/verify-bootstrap.ps1`.
- **`[0.2.2]` Branch Protection & Repository Governance:**
  - Configured repository merge strategy via GitHub REST API: disabled merge commits (`allow_merge_commit: false`), enabled squash merge (`allow_squash_merge: true`), and enabled rebase merge (`allow_rebase_merge: true`) to enforce linear history.
  - Enabled automatic head branch pruning on PR merge (`delete_branch_on_merge: true`).
  - Enabled Dependabot vulnerability alerts and automated security fixes.
  - Enforced branch protection on `main`: blocked force pushes (`allow_force_pushes: false`), blocked branch deletions (`allow_deletions: false`), and required PR conversation resolution (`required_conversation_resolution: true`).
  - Documented repository governance and protection rules in `docs/DEVELOPMENT.md`.
- **`[0.2.1]` GitHub Remote Repository & Push:**
  - Verified Git Credential Manager OAuth scopes (`gist, repo, workflow`) for authenticated user `dayashimoga`.
  - Created public GitHub repository `https://github.com/dayashimoga/web-repo-to-ai` via GitHub REST API with issues enabled.
  - Configured Git remote `origin` pointing to `https://github.com/dayashimoga/web-repo-to-ai.git`.
  - Pushed `main` branch with upstream tracking (`git push -u origin main`).
  - Verified remote tracking and commit SHA alignment (`98cc1decf23f270aef970877eb3a4b574b86d822 refs/heads/main`).

### 2026-11-16 — Phase 0.1: Repository and Governance Initialization (`0.1.1` – `0.1.5`)

#### Added
- **`[0.1.1]` Environment & Tooling Inspection:**
  - Verified Windows host environment (14 logical processors, 16.64 GB physical RAM, 33.51 GB free disk on `C:`), `git 2.55.0.windows.5`, `rustc/cargo 1.97.1`, `node v20.18.0`, `npm 10.8.2`, `podman 5.8.3` (`podman-machine-default` WSL VM with 30 GiB disk ceiling), `podman-compose`, and Git Credential Manager authentication status (`dayashimoga`).
- **`[0.1.2]` Monorepo & Git Foundation (`58e0f80`):**
  - Initialized local Git repository on `main` branch with LF line-ending normalization (`.gitattributes`, `core.autocrlf=false`).
  - Added `.gitignore`, `.editorconfig`, MIT `LICENSE`, and `README.md`.
  - Created monorepo structure across `apps/web`, `services/{api,worker,browser-worker}`, 15 shared Rust crate directories (`crates/{core,cdm,url-security,web-extractor,github-extractor,code-intelligence,normalizer,deduplication,tokenizer,context-optimizer,chunking,exporters,storage,job-engine,observability}`), `docs/`, `deploy/`, `scripts/`, `.agent/`, and `.github/workflows/`.
- **`[0.1.3]` Engineering Contract, Requirements & Baseline Documentation (`6372d8a`):**
  - Created `GLOBAL_ENGINEERING_CONTRACT.md` enforcing product integrity, CDM-centered Rust architecture, tiered performance, completeness accounting, SSRF/OAuth security, and one-subphase-at-a-time execution.
  - Created `REQUIREMENTS.md`, `docs/REQUIREMENTS.md`, and `DECISIONS.md` (`ADR-001` through `ADR-007`).
  - Created `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/DATA_MODEL.md`, `docs/API.md`, `docs/DEVELOPMENT.md`, `docs/DEPLOYMENT.md`, `docs/USER_GUIDE.md`, `docs/TESTING.md`, `docs/PERFORMANCE.md`, and `docs/LICENSING.md`.
- **`[0.1.4]` Agent System, Skills, Rules & Workflows (`f2aa8e9`):**
  - Created `AGENTS.md` and registered 9 specialized agent roles (`architecture-lead`, `frontend-engineer`, `rust-backend-engineer`, `extraction-engineer`, `github-integration-engineer`, `security-reviewer`, `qa-engineer`, `devops-engineer`, `documentation-reviewer`) in the Antigravity runtime and `.agent/agents/*.md`.
  - Created 10 domain skills under `.agent/skills/*/SKILL.md` (`architecture`, `rust-engineering`, `frontend-engineering`, `web-extraction`, `github-integration`, `context-optimization`, `security-engineering`, `testing-quality`, `devops-cicd`, `documentation`).
  - Created 10 engineering rules under `.agent/rules/00-global-engineering.md` through `09-no-mock-no-placeholder.md`.
  - Created 10 reusable workflows under `.agent/workflows/*.md`.
- **`[0.1.5]` Staged Roadmap, Ledger & Bootstrap Verification:**
  - Created `ROADMAP.md` and `docs/ROADMAP.md` defining all 12 stages (`Stage 0` – `Stage 11`), 48 phases, and 197 subphases.
  - Created `IMPLEMENTATION_LEDGER.md`, `PROJECT_STATUS.md`, `TODO.md`, `CHANGELOG.md`, and `scripts/verify-bootstrap.ps1`.
