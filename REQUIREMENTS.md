# REQUIREMENTS SPECIFICATION SUMMARY

**Project:** `web-repo-to-ai`  
**Canonical Full Specification:** [docs/REQUIREMENTS.md](./docs/REQUIREMENTS.md)

---

## 1. Product Purpose

Provide a responsive, self-hostable, open-source web application, API, CLI, and agent integration platform that converts:
1. Publicly accessible webpage URLs (static HTML and dynamic JS-rendered pages).
2. Public GitHub repositories (any branch, tag, commit, subdirectory, or file filter).
3. Authorized private GitHub repositories (via minimal read-only GitHub App authorization).
4. Batch URL sets and bounded website crawls.

Into structured, provenance-verified, token-efficient outputs (Markdown, TXT, JSON, HTML, JSONL, and ZIP packages) ready for LLMs, AI coding agents, code reviewers, security auditors, and RAG pipelines—without requiring any paid AI API keys.

---

## 2. Requirement Categories Overview

| Category ID | Domain | Summary of Mandatory Capabilities |
| :--- | :--- | :--- |
| **FR-WEB** | Webpage Extraction | Fast HTTP fetching, readability article extraction, documentation layout extraction, tables/code preservation, Playwright dynamic fallback, canonical URLs, robots/access policy awareness. |
| **FR-GH-PUB** | Public GitHub Conversion | Repo/branch/tag/commit resolution, Git Trees API with truncation recovery & git clone/archive fallback, selective file/folder filters, architecture & manifest generation, 100% file accounting. |
| **FR-GH-PRIV** | Private GitHub Integration | GitHub App OAuth & installation flow, read-only permissions (`contents:read`, `metadata:read`), encrypted short-lived tokens, cross-user isolation, zero token leakage. |
| **FR-CDM** | Canonical Document Model | Versioned intermediate schema (`CDM v1.0`) capturing source identity, revision, hierarchy, sections, line ranges, language, BLAKE3/SHA-256 hashes, citations, warnings, and exclusions. |
| **FR-OPT** | AI Context & Token Intelligence | Lossless clean mode, compact mode, 6 specialized context profiles (General AI, Code Review, Security Review, Debugging, Docs, RAG), exact/approximate tokenizers, budget prioritization, deterministic chunking. |
| **FR-EXP** | Export Engine | Markdown (default), TXT, JSON, HTML, JSONL, and streaming ZIP bundle with `manifest.json`, `checksums.sha256`, `provenance.json`, and `quality-report.json`. |
| **FR-UI** | Responsive Web Application | SvelteKit + Svelte 5 + Tailwind CSS UI with auto source detection, conversion settings, live SSE job progress, virtualized file tree & preview, token breakdown, copy/download, and local/persisted history. |
| **FR-SCALE** | Batch, Crawl & Incremental | Multi-URL batch jobs, sitemap & internal link crawling with depth/domain bounds, ETag/Last-Modified & Git blob content-addressed caching, backpressure, and resumable job leases. |
| **FR-INT** | Developer & Agent Integrations | Authenticated REST API, Rust CLI binary, MCP server (tools + resources), and browser extension foundation sharing the production extraction engine. |
| **FR-DOC** | Advanced Documents | Modular PDF extraction, Tesseract OCR capability, structured document parsing, RAG chunk exports, and optional AI provider adapters. |
| **NFR-SEC** | Security & Isolation | SSRF/DNS-rebinding protection, redirect revalidation, isolated browser worker, secret redaction, untrusted content framing, rate/size limits. |
| **NFR-PERF** | Performance & Resource Budget | CPU-only execution on 16 GB RAM host, <= 30 GB Podman disk budget, streaming archive generation, bounded worker concurrency. |
| **NFR-QUAL** | Quality, Testing & Governance | >= 95% test coverage on critical core crates, 100% pass rate on mandatory CI checks, golden dataset regression tests, SBOM & permissive license enforcement. |

For complete requirement definitions, acceptance criteria, and traceability matrices, see [docs/REQUIREMENTS.md](./docs/REQUIREMENTS.md).
