# Canonical Context Document Model & Database Schema (`docs/DATA_MODEL.md`)

**Project:** `web-repo-to-ai`  
**CDM Schema Version:** `1.0.0`

---

## 1. Canonical Context Document Model (`crates/cdm`)

Every extraction source produces a `CdmDocument` (`schema_version = "1.0.0"`), which serves as the single source of truth for optimization, token counting, chunking, and export generation.

### 1.1 Core Rust Domain Structure (Conceptual Schema)

```rust
pub struct CdmDocument {
    pub schema_version: String,              // e.g., "1.0.0"
    pub document_id: Uuid,
    pub source_identity: SourceIdentity,     // URL, repo owner/name, subpath
    pub source_type: SourceType,             // WebpageStatic, WebpageDynamic, GitHubPublic, GitHubPrivate, DocumentPdf, DocumentOcr
    pub canonical_url: String,
    pub retrieved_at: DateTime<Utc>,
    pub revision: RevisionInfo,              // Git commit SHA, branch, tag, or HTTP ETag/Last-Modified
    pub content_type: String,
    pub metadata: DocumentMetadata,          // Title, description, license, authors, language breakdown
    pub hierarchy: HierarchyNode,            // Root of document outline or repository file tree
    pub sections: Vec<CdmSection>,           // Ordered content units (files, article sections, code blocks)
    pub relationships: Vec<Relationship>,    // Imports, internal links, symbol references
    pub citations: Vec<Citation>,            // Provenance anchors for sections/chunks
    pub extraction_status: ExtractionStatus, // Completed, Partial, Failed, Unsupported
    pub warnings: Vec<ExtractionWarning>,
    pub exclusions: Vec<ExclusionRecord>,    // Every skipped/excluded file or block with reason
    pub completeness: CompletenessReport,    // Discovered vs. eligible vs. extracted vs. failed accounting
}

pub struct CdmSection {
    pub section_id: String,
    pub parent_id: Option<String>,
    pub title: Option<String>,
    pub kind: SectionKind,                   // Prose, CodeFile, CodeSnippet, Table, Frontmatter, Manifest
    pub original_path: Option<String>,       // Repo-relative file path or DOM selector path
    pub source_url: String,
    pub line_range: Option<LineRange>,       // 1-indexed start_line..=end_line
    pub byte_range: Option<ByteRange>,
    pub language: Option<String>,            // e.g., "rust", "typescript", "markdown"
    pub content_hash: String,                // BLAKE3 hex digest of exact section body
    pub body: String,
    pub token_Metrics: Option<SectionTokenMetrics>,
}
```

### 1.2 Completeness Invariant

For every repository conversion:
$$\text{discovered\_count} = \text{extracted\_count} + \text{excluded\_count} + \text{failed\_count} + \text{unsupported\_count}$$
If any eligible file is missing from `extracted_count` without an explicit entry in `exclusions` or `failed_count`, `CompletenessReport::is_complete()` evaluates to `false` and the job cannot be marked `COMPLETED`.

---

## 2. PostgreSQL Database Schema (`services/api` & `crates/job-engine`)

Large content artifacts are stored in `crates/storage` (filesystem or S3-compatible object storage). PostgreSQL stores relational state, job leases, manifests, and encrypted credentials.

### 2.1 Core Tables

1. **`jobs`**
   - `id` (`UUID PRIMARY KEY`)
   - `kind` (`TEXT NOT NULL` — `web_single`, `web_batch`, `web_crawl`, `github_public`, `github_private`)
   - `status` (`TEXT NOT NULL` — `queued`, `leased`, `running`, `completed`, `partial`, `failed`, `cancelled`)
   - `source_url` (`TEXT NOT NULL`)
   - `config_json` (`JSONB NOT NULL` — profile, format list, token budget, filters)
   - `progress_phase` (`TEXT NOT NULL`)
   - `progress_pct` (`SMALLINT NOT NULL DEFAULT 0`)
   - `leased_by` (`TEXT NULL` — worker instance ID)
   - `lease_expires_at` (`TIMESTAMPTZ NULL`)
   - `attempt_count` (`SMALLINT NOT NULL DEFAULT 0`)
   - `error_code` (`TEXT NULL`)
   - `error_message` (`TEXT NULL`)
   - `owner_session_id` (`UUID NULL`)
   - `created_at`, `updated_at`, `completed_at` (`TIMESTAMPTZ`)

2. **`job_artifacts`**
   - `id` (`UUID PRIMARY KEY`)
   - `job_id` (`UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE`)
   - `format` (`TEXT NOT NULL` — `cdm_json`, `markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`, `quality_report`)
   - `storage_key` (`TEXT NOT NULL`)
   - `byte_size` (`BIGINT NOT NULL`)
   - `sha256_hex` (`TEXT NOT NULL`)
   - `blake3_hex` (`TEXT NOT NULL`)
   - `expires_at` (`TIMESTAMPTZ NOT NULL`)
   - `created_at` (`TIMESTAMPTZ NOT NULL`)

3. **`github_installations`**
   - `id` (`UUID PRIMARY KEY`)
   - `owner_session_id` (`UUID NOT NULL`)
   - `github_installation_id` (`BIGINT NOT NULL`)
   - `account_login` (`TEXT NOT NULL`)
   - `encrypted_token` (`BYTEA NULL`)
   - `token_nonce` (`BYTEA NULL`)
   - `token_expires_at` (`TIMESTAMPTZ NULL`)
   - `revoked_at` (`TIMESTAMPTZ NULL`)
   - `created_at`, `updated_at` (`TIMESTAMPTZ NOT NULL`)

4. **`content_cache_index`**
   - `cache_key` (`TEXT PRIMARY KEY` — e.g., git blob SHA or URL + validator hash)
   - `blake3_hex` (`TEXT NOT NULL`)
   - `storage_key` (`TEXT NOT NULL`)
   - `byte_size` (`BIGINT NOT NULL`)
   - `last_accessed_at` (`TIMESTAMPTZ NOT NULL`)
