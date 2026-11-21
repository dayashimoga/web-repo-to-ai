# Canonical Context Document Model & Database Schema (`docs/DATA_MODEL.md`)

**Project:** `web-repo-to-ai`  
**Status:** Certified Specification v1.0  
**CDM Schema Version:** `1.0.0`

---

## 1. Canonical Context Document Model (`crates/cdm`)

The **Canonical Context Document Model (CDM)** is the unified, versioned, strongly typed representation produced by all ingest extractors (`web-extractor`, `github-extractor`, `pdf-extractor`, `ocr-extractor`, `structured-extractor`). It guarantees lossless semantic fidelity, cryptographic tamper-evidence, and uniform token accounting across all downstream optimizers and multi-format exporters.

### 1.1 Core Domain Structs

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Root Canonical Context Document representing an extracted webpage, repository, or file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdmDocument {
    /// Semantic schema version (strictly "1.0.0").
    pub schema_version: String,
    /// Strongly-typed unique document UUID.
    pub document_id: Uuid,
    /// Source identification (URL, repository slug, file path).
    pub source_identity: SourceIdentity,
    /// Ingestion source type.
    pub source_type: SourceType,
    /// Normalized canonical URL or path.
    pub canonical_url: String,
    /// UTC timestamp of extraction.
    pub retrieved_at: DateTime<Utc>,
    /// Target revision (Git commit SHA, branch, tag, or HTTP ETag).
    pub revision: RevisionInfo,
    /// Content MIME type (e.g. "text/html", "application/vnd.github.v3+json").
    pub content_type: String,
    /// Metadata attributes (title, description, authors, license, language breakdown).
    pub metadata: DocumentMetadata,
    /// Structural hierarchy outline (document headings or repository file tree).
    pub hierarchy: HierarchyNode,
    /// Ordered sequence of content sections.
    pub sections: Vec<CdmSection>,
    /// Structural and semantic relationships (imports, symbols, citations).
    pub relationships: Vec<Relationship>,
    /// Provenance anchors mapping content back to line/byte ranges.
    pub citations: Vec<Citation>,
    /// Status of the extraction process.
    pub extraction_status: ExtractionStatus,
    /// Non-fatal warnings encountered during extraction.
    pub warnings: Vec<ExtractionWarning>,
    /// Explicit accounting of excluded or skipped elements.
    pub exclusions: Vec<ExclusionRecord>,
    /// Completeness audit report.
    pub completeness: CompletenessReport,
}

/// An individual content section (e.g., Markdown section, code file, table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdmSection {
    /// Unique section ID within the document.
    pub section_id: String,
    /// Parent section ID in the hierarchy (if nested).
    pub parent_id: Option<String>,
    /// Section heading or file path.
    pub title: Option<String>,
    /// Classification of section content.
    pub kind: SectionKind,
    /// Repository-relative or DOM selector path.
    pub original_path: Option<String>,
    /// Source provenance URL.
    pub source_url: String,
    /// 1-indexed start and end line bounds.
    pub line_range: Option<LineRange>,
    /// 0-indexed byte bounds within source artifact.
    pub byte_range: Option<ByteRange>,
    /// Programming or markup language identifier (e.g. "rust", "typescript", "markdown").
    pub language: Option<String>,
    /// Cryptographic BLAKE3 hex digest of the raw section body.
    pub content_hash: String,
    /// Clean, normalized text content.
    pub body: String,
    /// Token counts across supported tokenizers with uncertainty intervals.
    pub token_metrics: Option<SectionTokenMetrics>,
}

/// Content section classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionKind {
    Prose,
    CodeFile,
    CodeSnippet,
    Table,
    Frontmatter,
    Manifest,
    StructuredData,
    PdfPage,
    OcrText,
}

/// Ingestion source origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    WebpageStatic,
    WebpageDynamic,
    GitHubPublic,
    GitHubPrivate,
    DocumentPdf,
    DocumentOcr,
    StructuredData,
}
```

---

## 2. Completeness & Verification Invariants

### 2.1 The Completeness Equation
To prevent silent omissions, every repository and multi-page crawl conversion must satisfy the strict **Completeness Invariant**:

$$\text{discovered\_count} = \text{extracted\_count} + \text{excluded\_count} + \text{failed\_count} + \text{unsupported\_count}$$

```rust
impl CompletenessReport {
    /// Evaluates whether all discovered files/pages have been accounted for.
    pub fn is_complete(&self) -> bool {
        self.discovered_count == 
            (self.extracted_count + self.excluded_count + self.failed_count + self.unsupported_count)
            && self.failed_count == 0
    }
}
```
If `is_complete()` evaluates to `false`, the job state transitions to `Partial` or `Failed`, and a comprehensive audit manifest is emitted documenting all unaccounted files.

---

## 3. RAG Semantic Chunk & Vector Adapter Models (`crates/chunking`)

When exporting for Retrieval-Augmented Generation (RAG) and vector databases, CDM sections are deterministically chunked with sliding-window overlap:

```rust
/// Individual semantic chunk generated for vector database ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdmChunk {
    /// Deterministic UUID computed from document_id + section_id + chunk_index.
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub section_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub token_count: usize,
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub title: Option<String>,
    pub source_url: String,
    pub original_path: Option<String>,
    pub language: Option<String>,
    pub line_range: Option<LineRange>,
    pub section_kind: SectionKind,
    pub tags: Vec<String>,
}
```

### 3.1 Vector Database Schema Mappings

The multi-format exporter (`crates/exporters`) supports direct schema adaptation for:
- **Qdrant:** `{"points": [{"id": "<uuid>", "vector": null, "payload": {...}}]}`
- **Pinecone:** `{"vectors": [{"id": "<uuid>", "values": [], "metadata": {...}}]}`
- **ChromaDB:** `{"ids": ["<uuid>"], "documents": ["..."], "metadatas": [{...}]}`
- **Weaviate:** `{"class": "DocumentChunk", "properties": {...}}`
- **Milvus:** Direct entity collection schema matching `CdmChunk` fields.

---

## 4. PostgreSQL Relational Database Schema (`services/api` & `crates/job-engine`)

Relational state, asynchronous job leases, credential vaults, and progress audit trails are persisted in PostgreSQL 16:

```sql
-- Core Jobs Table
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind TEXT NOT NULL,                  -- 'web_single', 'web_batch', 'web_crawl', 'github_public', 'github_private', 'pdf_document'
    status TEXT NOT NULL,                -- 'queued', 'leased', 'running', 'completed', 'partial', 'failed', 'cancelled'
    source_url TEXT NOT NULL,
    config_json JSONB NOT NULL,          -- Profile, token budget, format selections, exclusions
    progress_phase TEXT NOT NULL,        -- 'queued', 'acquiring', 'parsing', 'normalizing', 'tokenizing', 'exporting', 'completed'
    progress_pct SMALLINT NOT NULL DEFAULT 0 CHECK (progress_pct >= 0 AND progress_pct <= 100),
    leased_by TEXT NULL,                 -- Worker instance ID holding active lease
    lease_expires_at TIMESTAMPTZ NULL,   -- Expiration timestamp for pessimistic lease
    attempt_count SMALLINT NOT NULL DEFAULT 0,
    max_attempts SMALLINT NOT NULL DEFAULT 3,
    error_code TEXT NULL,
    error_message TEXT NULL,
    owner_session_id UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NULL
);

CREATE INDEX idx_jobs_status_lease ON jobs (status, lease_expires_at) WHERE status IN ('queued', 'leased', 'running');
CREATE INDEX idx_jobs_created_at ON jobs (created_at DESC);

-- Job Output Artifacts Table
CREATE TABLE job_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    format TEXT NOT NULL,                -- 'cdm_json', 'markdown', 'txt', 'json', 'html', 'jsonl', 'zip', 'manifest'
    storage_key TEXT NOT NULL,           -- Key in LocalFs or S3 object store
    byte_size BIGINT NOT NULL,
    sha256_hex TEXT NOT NULL,
    blake3_hex TEXT NOT NULL,
    token_count INT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_artifacts_job_id ON job_artifacts (job_id);

-- GitHub App Installations & Encrypted OAuth Credentials
CREATE TABLE github_installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_session_id UUID NOT NULL,
    github_installation_id BIGINT NOT NULL UNIQUE,
    account_login TEXT NOT NULL,
    encrypted_token BYTEA NULL,          -- AES-256-GCM encrypted token payload
    token_nonce BYTEA NULL,              -- 12-byte initialization vector / nonce
    token_expires_at TIMESTAMPTZ NULL,
    revoked_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Content-Addressable Storage Blob Index
CREATE TABLE content_cache_index (
    cache_key TEXT PRIMARY KEY,          -- Source URL hash or Git Blob SHA
    blake3_hex TEXT NOT NULL,            -- Content-addressed cryptographic hash
    storage_key TEXT NOT NULL,
    byte_size BIGINT NOT NULL,
    hit_count BIGINT NOT NULL DEFAULT 1,
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_content_cache_lru ON content_cache_index (last_accessed_at ASC);

-- Audit Trail for Job State Machine Transitions
CREATE TABLE job_state_transitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    from_status TEXT NOT NULL,
    to_status TEXT NOT NULL,
    phase TEXT NOT NULL,
    progress_pct SMALLINT NOT NULL CHECK (progress_pct >= 0 AND progress_pct <= 100),
    message TEXT NULL,
    error_code TEXT NULL,
    worker_id TEXT NULL,
    transitioned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_state_transitions_job ON job_state_transitions (job_id, transitioned_at ASC);
```

---

## 5. Content-Addressable Storage (CAS) Schema

Blob storage in `crates/storage` follows a hierarchical content-addressed layout:

```text
storage/
├── cas/
│   ├── 0a/
│   │   └── 3f/
│   │       └── 0a3f89b1c2d3e4f5...blob    # Raw immutable blob named by 64-char BLAKE3 hex
│   └── ...
├── artifacts/
│   ├── 6c1f4d1f-b26b-4de5-9428-b834cbdbd307/
│   │   ├── document.cdm.json
│   │   ├── output.md
│   │   └── export_bundle.zip
│   └── ...
└── staging/
    └── .artifact.tmp.8372648172             # Atomic staging scratch file before rename
```

- **Atomic Commits:** Writers stage files to `.artifact.tmp.*` and rename atomically once the full payload is flushed to disk.
- **Deduplication:** Repeated files across repositories or web pages share the identical CAS blob, providing optimal disk utilization.
- **Audit Verification:** `CasStorage::audit()` recomputes BLAKE3 hashes across all indexed blobs to guarantee zero bit-rot.
