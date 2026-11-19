# REST & Streaming API Specification (`docs/API.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Base Path:** `/api/v1`  
**Protocol Version:** OpenAPI 3.1.0  
**Status:** Certified OpenAPI 3.1 Specification & Interactive Documentation (`Subphase 9.1.1`)

---

## Table of Contents
1. [Overview & Machine-Readable Specifications](#1-overview--machine-readable-specifications)
2. [Global Headers & Authentication Conventions](#2-global-headers--authentication-conventions)
3. [RFC 7807 Problem Details Error Contract](#3-rfc-7807-problem-details-error-contract)
4. [System & Telemetry Endpoints](#4-system--telemetry-endpoints)
5. [Source Detection & SSRF Pre-Validation](#5-source-detection--ssrf-pre-validation)
6. [Single Conversion Jobs Lifecycle](#6-single-conversion-jobs-lifecycle)
7. [Real-Time SSE Streaming Contract](#7-real-time-sse-streaming-contract)
8. [Multi-URL Parallel Batch Processing](#8-multi-url-parallel-batch-processing)
9. [GitHub App & Webhook Integration](#9-github-app--webhook-integration)
10. [Export Formats & Artifact Specifications](#10-export-formats--artifact-specifications)

---

## 1. Overview & Machine-Readable Specifications

The `web-repo-to-ai-api` service exposes a production-grade, asynchronous, streaming-capable REST API built on Tokio and Axum. It serves both human operators and autonomous AI agents with predictable contracts, rigorous schema validation, and real-time execution feedback.

### Interactive Documentation & Specifications
- **Interactive API Documentation (Scalar / Swagger):**
  - Path: `GET /api/v1/docs` (or top-level alias `GET /docs`)
  - Content-Type: `text/html; charset=utf-8`
  - Features: Modern interactive API explorer with theme switching (Deep Space Dark / Light), schema visualization, and live curl requests.
- **OpenAPI 3.1.0 JSON Document:**
  - Path: `GET /api/v1/openapi.json`
  - Content-Type: `application/json; charset=utf-8`
  - Specification: Fully compliant OpenAPI 3.1.0 JSON Schema.
- **OpenAPI 3.1.0 YAML Document:**
  - Path: `GET /api/v1/openapi.yaml`
  - Content-Type: `application/yaml; charset=utf-8`
- **Root Service Descriptor:**
  - Path: `GET /`
  - Content-Type: `application/json`
  - Returns service metadata, version, and links to all primary subsystem routes.

---

## 2. Global Headers & Authentication Conventions

| Header Name | Type | Direction | Description |
| :--- | :--- | :--- | :--- |
| `X-Request-ID` | UUIDv4 / UUIDv7 | In / Out | Distributed request correlation identifier. Injected into response and all RFC 7807 error envelopes. |
| `traceparent` | W3C TraceContext | In / Out | W3C distributed tracing context header (`00-{trace_id}-{span_id}-{flags}`). |
| `X-Session-ID` | String | In | Tenant isolation session identifier for web and private GitHub installation access. |
| `Authorization` | String | In | Optional Bearer API key for developer and AI agent invocations (`Bearer <token>`). |
| `X-Hub-Signature-256` | String | In | HMAC-SHA256 signature for GitHub App webhook event verification (`sha256=<hex_digest>`). |
| `X-GitHub-Event` | String | In | GitHub webhook event name (e.g. `installation`, `installation_repositories`). |
| `X-GitHub-Delivery` | UUID | In | GitHub webhook delivery identifier. |
| `Content-Type` | MIME Type | In / Out | Standard request and response encoding (`application/json`, `application/problem+json`, `text/event-stream`, `application/zip`). |

---

## 3. RFC 7807 Problem Details Error Contract

All error responses strictly adhere to the RFC 7807 problem details specification and return HTTP content type `application/problem+json`.

### Error Envelope Schema
```json
{
  "error": {
    "code": "SSRF_ADDRESS_BLOCKED",
    "message": "The requested URL resolved to a restricted private or loopback network address.",
    "details": {
      "url": "http://169.254.169.254/latest/meta-data/"
    },
    "request_id": "019335c4-7c20-7000-8000-000000000001"
  }
}
```

### Standard Error Catalog

| Error Code | HTTP Status | Description & Mitigation |
| :--- | :---: | :--- |
| `INVALID_REQUEST_SCHEMA` | `400` | Malformed JSON body or missing required attributes. Check field types and JSON syntax. |
| `VALIDATION_ERROR` | `400` | Semantic validation failure (e.g., token budget outside 100..2,000,000 or empty URLs list). |
| `UNSUPPORTED_URL_SCHEME` | `400` | URL scheme is not `http://` or `https://` (e.g., `file://`, `ftp://`). |
| `SSRF_ADDRESS_BLOCKED` | `403` | Target domain resolved to private IPv4/IPv6, CGNAT, loopback, or cloud metadata address. |
| `UNAUTHORIZED_GITHUB_REPO` | `401` / `403` | User does not have an active GitHub App installation grant for the requested private repo. |
| `RESOURCE_NOT_FOUND` | `404` | Requested job, batch, artifact format, or installation does not exist. |
| `PAYLOAD_TOO_LARGE` | `413` | Uploaded request body exceeded the 2 MB buffer limit. |
| `RATE_LIMIT_EXCEEDED` | `429` | Upstream or local concurrency / rate ceiling reached. Inspect `Retry-After` header. |
| `EXTRACTION_TIMEOUT` | `504` | External HTTP fetch, Playwright render, or tree traversal exceeded timeout ceiling. |
| `INTERNAL_SERVER_ERROR` | `500` | Unexpected engine fault. Error is logged and correlated with `request_id`. |

---

## 4. System & Telemetry Endpoints

### 4.1 Liveness Probe
- **`GET /api/v1/health`**
- **Response `200 OK`:**
```json
{
  "status": "pass",
  "service": "web-repo-to-ai-api",
  "version": "0.1.0"
}
```

### 4.2 Readiness Probe
- **`GET /api/v1/ready`**
- **Response `200 OK`:**
```json
{
  "status": "ready",
  "checks": {
    "database": "connected",
    "storage": "writable"
  }
}
```
- **Response `503 Service Unavailable`:** Returned if PostgreSQL or disk storage fails health verification.

### 4.3 Prometheus Metrics Exposition
- **`GET /api/v1/metrics`**
- **Content-Type:** `text/plain; version=0.0.4`
- Exposes standard metrics including:
  - `http_requests_total{method,path,status}`
  - `http_request_duration_seconds{method,path}`
  - `active_workers_gauge`
  - `batch_jobs_total{status}`
  - `streaming_bytes_exported_total`

---

## 5. Source Detection & SSRF Pre-Validation

### 5.1 Detect Sources
- **`POST /api/v1/sources/detect`**
- **Description:** Pre-validates up to 50 URLs, applies SSRF zero-trust filtering, and extracts GitHub repository coordinates or webpage metadata.
- **Request Body:**
```json
{
  "urls": [
    "https://github.com/tokio-rs/tokio/tree/master/tokio",
    "https://docs.rs/axum/latest/axum/"
  ]
}
```
- **Response `200 OK`:**
```json
{
  "sources": [
    {
      "url": "https://github.com/tokio-rs/tokio/tree/master/tokio",
      "classification": "public_github",
      "repo_owner": "tokio-rs",
      "repo_name": "tokio",
      "git_ref": "master",
      "subpath": "tokio",
      "valid": true,
      "reason": null
    },
    {
      "url": "https://docs.rs/axum/latest/axum/",
      "classification": "webpage",
      "repo_owner": null,
      "repo_name": null,
      "git_ref": null,
      "subpath": null,
      "valid": true,
      "reason": null
    }
  ]
}
```

---

## 6. Single Conversion Jobs Lifecycle

### 6.1 Create Conversion Job
- **`POST /api/v1/jobs`**
- **Request Body:**
```json
{
  "sources": ["https://github.com/tokio-rs/tokio"],
  "git_ref": "master",
  "path_patterns": ["tokio/src/**/*.rs"],
  "token_budget": 128000,
  "target_model": "claude-3-5-sonnet",
  "export_formats": ["markdown", "json", "zip"],
  "deduplicate_exact": true,
  "deduplicate_semantic": true,
  "remove_comments": false
}
```
- **Response `201 Created`:**
```json
{
  "id": "019335c4-7c20-7000-8000-000000000002",
  "status": "queued",
  "source_type": "github",
  "created_at": "2026-11-19T10:00:00Z",
  "finished_at": null,
  "token_stats": null,
  "artifacts": [],
  "error": null
}
```

### 6.2 List Conversion Jobs
- **`GET /api/v1/jobs?page=1&limit=20&status=completed&sort=created_at:desc`**
- **Response `200 OK`:**
```json
{
  "items": [
    {
      "id": "019335c4-7c20-7000-8000-000000000002",
      "status": "completed",
      "source_type": "github",
      "created_at": "2026-11-19T10:00:00Z",
      "finished_at": "2026-11-19T10:00:04Z",
      "token_stats": {
        "raw_tokens": 154200,
        "optimized_tokens": 89400,
        "budget_percentage": 69.84,
        "savings_percentage": 42.02
      },
      "artifacts": ["markdown", "json", "zip"],
      "error": null
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 20,
  "total_pages": 1
}
```

### 6.3 Get Job Details
- **`GET /api/v1/jobs/{job_id}`**
- **Response `200 OK`:** Returns [`JobResponse`](#componentsschemasjobresponse).

### 6.4 Preview Converted Content
- **`GET /api/v1/jobs/{job_id}/preview?format=markdown&max_lines=200`**
- **Response `200 OK`:**
```json
{
  "job_id": "019335c4-7c20-7000-8000-000000000002",
  "format": "markdown",
  "content": "# Repository: tokio-rs/tokio\n\n## File: tokio/src/lib.rs\n...",
  "truncated": false,
  "total_lines": 184,
  "rendered_lines": 184,
  "repo_tree": [
    {
      "path": "tokio/src/lib.rs",
      "size_bytes": 14200,
      "classification": "source_code"
    }
  ]
}
```

### 6.5 Download Artifact Stream
- **`GET /api/v1/jobs/{job_id}/artifacts/{format}`**
- **Supported Formats:** `markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`.
- **Headers Returned:**
  - `Content-Disposition: attachment; filename="tokio-rs_tokio_markdown.md"`
  - `X-Checksum-Sha256: <hex_sha256_digest>`
- **Response `200 OK`:** Raw binary or chunked stream.

---

## 7. Real-Time SSE Streaming Contract

Clients connect to `GET /api/v1/jobs/{job_id}/events` (or alias `GET /api/v1/jobs/{job_id}/stream`) with header `Accept: text/event-stream`.

### Event Lifecycle & Payload Examples

#### Phase Change Event (`phase`)
```http
event: phase
data: {"job_id":"019335c4-7c20-7000-8000-000000000002","phase":"extracting","progress_pct":25.0}
```

#### File Accounting Event (`file`)
```http
event: file
data: {"job_id":"019335c4-7c20-7000-8000-000000000002","path":"tokio/src/runtime.rs","size_bytes":18450,"classification":"code"}
```

#### Token Optimization Event (`tokens`)
```http
event: tokens
data: {"job_id":"019335c4-7c20-7000-8000-000000000002","raw_tokens":154200,"optimized_tokens":89400,"savings_pct":42.02}
```

#### Terminal Completion Event (`complete`)
```http
event: complete
data: {"job_id":"019335c4-7c20-7000-8000-000000000002","status":"completed","artifacts":["markdown","json","zip"]}
```

---

## 8. Multi-URL Parallel Batch Processing

Batch processing allows submitting up to 100 URLs for bounded concurrent execution with automatic AIMD concurrency, backpressure flow control, and BLAKE3-backed state checkpoint resumption.

### 8.1 Submit Batch Job
- **`POST /api/v1/jobs/batch`**
- **Request Body:**
```json
{
  "urls": [
    "https://docs.rs/tokio/latest/tokio/",
    "https://docs.rs/axum/latest/axum/",
    "https://docs.rs/tower/latest/tower/"
  ],
  "max_parallelism": 4,
  "context_profile": "documentation",
  "token_budget": 200000,
  "target_model": "gpt-4o",
  "export_formats": ["zip", "markdown", "jsonl"]
}
```
- **Response `202 Accepted`:**
```json
{
  "batch_id": "019335c4-7c20-7000-8000-000000000003",
  "total_items": 3,
  "max_parallelism": 4,
  "status": "queued",
  "created_at": "2026-11-19T10:05:00Z"
}
```

### 8.2 Get Batch Job Status & Progress
- **`GET /api/v1/jobs/batch/{batch_id}`**
- **Response `200 OK`:**
```json
{
  "batch_id": "019335c4-7c20-7000-8000-000000000003",
  "total_items": 3,
  "succeeded_count": 2,
  "failed_count": 0,
  "timed_out_count": 0,
  "cancelled_count": 0,
  "status": "running",
  "progress": {
    "percentage": 66.67,
    "completed_items": 2,
    "total_items": 3,
    "current_stage": "extracting"
  },
  "created_at": "2026-11-19T10:05:00Z",
  "finished_at": null
}
```

### 8.3 Cancel Batch Job
- **`DELETE /api/v1/jobs/batch/{batch_id}`** (or `POST /api/v1/jobs/batch/{batch_id}/cancel`)
- **Response `200 OK`:** Returns updated [`BatchJobResponse`](#componentsschemasbatchjobresponse) with status `cancelled`.

### 8.4 List Batch Items & Outcomes
- **`GET /api/v1/jobs/batch/{batch_id}/items`**
- **Response `200 OK`:**
```json
{
  "batch_id": "019335c4-7c20-7000-8000-000000000003",
  "total_items": 3,
  "items": [
    {
      "item_index": 0,
      "job_id": "019335c4-7c20-7000-8000-000000000004",
      "url": "https://docs.rs/tokio/latest/tokio/",
      "status": "completed",
      "duration_ms": 124,
      "error": null
    },
    {
      "item_index": 1,
      "job_id": "019335c4-7c20-7000-8000-000000000005",
      "url": "https://docs.rs/axum/latest/axum/",
      "status": "completed",
      "duration_ms": 98,
      "error": null
    }
  ]
}
```

### 8.5 Export Consolidated Batch Output
- **`GET /api/v1/jobs/batch/{batch_id}/export/{format}`**
- **Supported Formats:** `zip`, `markdown`, `jsonl`, `txt`, `json`.
- Streams the consolidated multi-item output or structured multi-document archive with `checksums.sha256` and `batch-manifest.json`.

---

## 9. GitHub App & Webhook Integration

### 9.1 List Authorized Installations
- **`GET /api/v1/auth/github/installations`**
- **Headers:** `X-Session-ID: <session_uuid>`
- **Response `200 OK`:**
```json
{
  "installations": [
    {
      "installation_id": 5892341,
      "account_login": "my-org",
      "account_type": "Organization",
      "repositories_count": 12
    }
  ]
}
```

### 9.2 Revoke Installation
- **`DELETE /api/v1/auth/github/installations/{installation_id}`**
- **Headers:** `X-Session-ID: <session_uuid>`
- **Response `200 OK`:**
```json
{
  "revoked": true,
  "installation_id": 5892341
}
```

### 9.3 Receive GitHub App Webhook
- **`POST /api/v1/auth/github/webhook`**
- **Headers Required:**
  - `X-GitHub-Event: installation`
  - `X-Hub-Signature-256: sha256=<hmac_sha256_hex>`
  - `X-GitHub-Delivery: <delivery_uuid>`
- **Response `200 OK`:**
```json
{
  "status": "acknowledged",
  "event": "installation",
  "delivery_id": "019335c4-7c20-7000-8000-000000000006"
}
```

---

## 10. Export Formats & Artifact Specifications

| Format Code | MIME Type | Target LLM Use Case | Internal Invariants |
| :--- | :--- | :--- | :--- |
| `markdown` | `text/markdown` | Direct Prompting, Claude Projects, NotebookLM | GitHub-Flavored Markdown, table normalization, line-numbered code fences |
| `txt` | `text/plain` | Legacy LLM context injection, raw text buffers | Clean Unicode headings, stripped tags, preserved structural indents |
| `json` | `application/json` | Structured tooling, LangChain, LlamaIndex | Full Canonical Document Model (CDM v1.0) JSON schema |
| `html` | `text/html` | Browser rendering, sandboxed iframe viewing | Standalone clean HTML5, inline sanitized styles, zero tracking scripts |
| `jsonl` | `application/x-ndjson` | Fine-tuning, vector database ingestion | One line-delimited JSON document per CDM section |
| `zip` | `application/zip` | Multi-file archive with checksums & manifest | Safe path traversal (anti-zip-slip), `checksums.sha256`, `manifest.json` |
| `manifest` | `application/json` | Verification & provenance tracking | 100% file completeness audit, SHA-256 & BLAKE3 digests |
