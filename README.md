# Web & Repo to AI (`web-repo-to-ai`)

**Universal URL & GitHub Repository → AI-Ready Content Converter**  
Open-source-first, Rust-powered, web-first, self-hostable conversion platform.

---

## Overview

`web-repo-to-ai` converts publicly accessible webpages, public GitHub repositories, authorized private GitHub repositories, complex documents (PDFs, Jupyter Notebooks, OpenAPI specs), and batch URL collections into structured, provenance-traceable, token-efficient context packages optimized for:

- **Large Language Models** (ChatGPT, Claude, Gemini, local models)
- **AI Coding Agents & IDEs** (Claude Code, Cursor, Windsurf, Antigravity)
- **Model Context Protocol (MCP) Clients** (Claude Desktop, VS Code, Goose)
- **Code Reviewers & Security Auditors**
- **RAG & Knowledge Retrieval Systems** (Qdrant, Pinecone, ChromaDB, Weaviate, Milvus)
- **Developers, Researchers, and Technical Teams**

All core extraction, structural normalization, canonical modeling, deduplication, and token optimization operate **completely offline without requiring paid AI API keys, external LLMs, or commercial scraping services**.

---

## Core Capabilities

1. **Universal Webpage Acquisition**: Fast HTTP static extraction (`html5ever`/`scraper` + readability and technical docs hierarchy parsing) with isolated Playwright + Chromium dynamic rendering fallback.
2. **Complete GitHub Repository Ingestion**: Public and authorized private repositories (via minimal read-only GitHub App), supporting branch/tag/commit selection, selective file trees, and mathematical 100% completeness accounting.
3. **Advanced Document & Spec Extraction**: Spatial PDF extraction with reading-order reconstruction and Tesseract OCR fallback; structured ingestion for OpenAPI 3.1.0 specifications, CSV/TSV matrices, and Jupyter Notebooks.
4. **Canonical Context Document Model (CDM v1.0)**: Strongly typed intermediate representation preserving exact source code, document hierarchy, tables, metadata, BLAKE3/SHA-256 hashes, line ranges, exclusions, and citations.
5. **AI Context & Token Intelligence**: Lossless clean extraction, semantic-aware deduplication ($f(f(x)) \equiv f(x)$), model-specific BPE token counting (GPT-4o, Claude 3.5 Sonnet, Gemini 1.5 Pro), AST semantic chunking, and context profiles.
6. **Multi-Format Exports**: AI-ready Markdown (default), Plain Text, JSON v1.0, HTML5, streaming JSONL, and streaming ZIP packages with embedded manifests and checksums.
7. **Developer & Agent Ecosystem**: Production REST API with OpenAPI 3.1 & SSE streaming, standalone Rust CLI (`web-repo-to-ai`), Model Context Protocol (MCP) server daemon (`web-repo-to-ai-mcp`), and Manifest V3 Chrome Extension.
8. **Defense-in-Depth Security**: Strict SSRF/DNS-rebinding protection, hop-by-hop redirect revalidation, encrypted GitHub App tokens (AES-256-GCM with AAD), recursive credential redaction, and bounded memory execution.

---

## Quickstart

### 1. Web Application

Run the web frontend locally with Node.js (>= 20 LTS):

```bash
cd apps/web
npm install
npm run dev
```

Visit `http://localhost:5173` to access the interactive conversion workspace, source detector, token budget indicators, and multi-format preview panel.

### 2. Command-Line Interface (CLI)

Build and install the unified CLI binary:

```bash
cargo install --path crates/cli
```

Convert any URL or repository directly from your terminal:

```bash
# Convert a webpage to AI-ready Markdown
web-repo-to-ai url https://docs.rs/tokio/latest/tokio/ --format markdown

# Convert a GitHub repository with token budgeting
web-repo-to-ai repo tokio-rs/tokio --max-tokens 50000 --format zip

# Convert a local project directory
web-repo-to-ai repo ./my-project --format jsonl

# Batch process multiple targets from a text file
web-repo-to-ai batch targets.txt --output-dir ./exports

# Verify package cryptographic integrity
web-repo-to-ai verify checksums.sha256
```

### 3. Model Context Protocol (MCP) Server

Launch the MCP server daemon over stdio for AI desktop clients:

```bash
cargo install --path crates/mcp
web-repo-to-ai-mcp
```

Add to your **Claude Desktop** configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "web-repo-to-ai": {
      "command": "web-repo-to-ai-mcp",
      "args": []
    }
  }
}
```

Exposes tools (`convert_url`, `convert_repo`, `detect_source`, `optimize_context`) and resources (`cdm://<doc_id>/markdown`, `cdm://<doc_id>/zip`, `file://`).

### 4. Chrome Browser Extension

Build the extension package:

```bash
cd apps/extension
npm install
npm run build
```

Open Chrome at `chrome://extensions`, enable **Developer mode**, and click **Load unpacked**, selecting the `apps/extension/dist` directory. Click the extension icon on any page to capture clean DOM content, inspect token statistics, and hand off directly to your conversion workspace.

### 5. Self-Hosted Podman / Docker Stack

Launch the full production stack (PostgreSQL, API, Worker, Browser Worker, and Web UI):

```bash
# Using Podman Compose
podman-compose -f deploy/podman-compose.yml up -d

# Using Docker Compose
docker compose -f deploy/podman-compose.yml up -d
```

---

## Monorepo Architecture

```text
web-repo-to-ai/
├── apps/
│   ├── web/                    # SvelteKit + Svelte 5 + TypeScript + Tailwind CSS UI
│   └── extension/              # Chrome Extension (Manifest V3, TypeScript, Vite)
├── services/
│   ├── api/                    # Rust Axum REST & SSE streaming API service
│   ├── worker/                 # Rust asynchronous extraction & optimization daemon
│   └── browser-worker/         # Isolated Playwright + Chromium rendering service
├── crates/
│   ├── core/                   # Shared domain types, errors, UUID v7 IDs, and config
│   ├── cdm/                    # Canonical Context Document Model (CDM v1.0)
│   ├── url-security/           # SSRF, DNS-rebinding, and IP-filtering defenses
│   ├── web-extractor/          # Static HTML, article, and documentation extraction
│   ├── github-extractor/       # Public & private GitHub API and Git Trees acquisition
│   ├── code-intelligence/      # Tree-sitter AST parsing, language & symbol analysis
│   ├── normalizer/             # Structural text and document normalization
│   ├── deduplication/          # Content-hash & semantic-aware deduplication
│   ├── tokenizer/              # Model-specific BPE tokenizers and estimators
│   ├── context-optimizer/      # Budget prioritization and context profile shaping
│   ├── chunking/               # AST semantic chunking and vector schema adapters
│   ├── exporters/              # Markdown, TXT, JSON, HTML, JSONL, and streaming ZIP
│   ├── storage/                # Local filesystem, CAS deduplication, and retention
│   ├── job-engine/             # PostgreSQL durable job state, leasing, and flow control
│   ├── observability/          # Tracing, Prometheus metrics, and secret redaction
│   ├── cli/                    # Unified command-line interface binary
│   ├── mcp/                    # Model Context Protocol (MCP) JSON-RPC 2.0 server
│   ├── pdf-extractor/          # Spatial PDF extraction and reading-order reconstruction
│   ├── ocr-extractor/          # Tesseract OCR engine and image text acquisition
│   ├── structured-extractor/   # OpenAPI 3.1, CSV/TSV, and Jupyter Notebook parsers
│   └── ai-adapter/             # Local Ollama & OpenAI-compatible provider adapters
├── deploy/                     # Podman Compose, containerfiles, and Caddy configs
├── scripts/                    # Resource budget, quality gates, and dev lifecycle scripts
├── docs/                       # Technical architecture, security, API, and user guides
└── .agent/                     # Antigravity agent skills, rules, and workflows
```

---

## Documentation Index

### Core Governance & Tracking
- [Global Engineering Contract](./GLOBAL_ENGINEERING_CONTRACT.md)
- [Project Status](./PROJECT_STATUS.md)
- [Implementation Ledger](./IMPLEMENTATION_LEDGER.md)
- [Roadmap](./ROADMAP.md)
- [Requirements Specification](./REQUIREMENTS.md)
- [Architecture Decisions](./DECISIONS.md)
- [Agent Organization & Ownership](./AGENTS.md)
- [Todo Tracker](./TODO.md)
- [Changelog](./CHANGELOG.md)

### Detailed Technical Guides (`docs/`)
- [User Guide & Onboarding (`docs/USER_GUIDE.md`)](./docs/USER_GUIDE.md)
- [System Architecture Specification (`docs/ARCHITECTURE.md`)](./docs/ARCHITECTURE.md)
- [REST & Streaming API Reference (`docs/API.md`)](./docs/API.md)
- [Security Architecture & Threat Model (`docs/SECURITY.md`)](./docs/SECURITY.md)
- [Data Model & Canonical CDM (`docs/DATA_MODEL.md`)](./docs/DATA_MODEL.md)
- [Local Development Guide (`docs/DEVELOPMENT.md`)](./docs/DEVELOPMENT.md)
- [Production Deployment Guide (`docs/DEPLOYMENT.md`)](./docs/DEPLOYMENT.md)
- [Testing & Quality Verification (`docs/TESTING.md`)](./docs/TESTING.md)
- [Performance & Benchmark Baselines (`docs/PERFORMANCE.md`)](./docs/PERFORMANCE.md)
- [Third-Party License Inventory (`docs/LICENSING.md`)](./docs/LICENSING.md)
- [GitHub App Integration Guide (`docs/GITHUB_APP.md`)](./docs/GITHUB_APP.md)
- [Frontend Design System (`docs/DESIGN_SYSTEM.md`)](./docs/DESIGN_SYSTEM.md)
- [Requirements Traceability (`docs/REQUIREMENTS.md`)](./docs/REQUIREMENTS.md)
- [Staged Execution Roadmap (`docs/ROADMAP.md`)](./docs/ROADMAP.md)
- [Production Release Certification Report (`docs/PRODUCTION_CERTIFICATION.md`)](./docs/PRODUCTION_CERTIFICATION.md)
- [Production Build & Release Guide (`docs/PRODUCTION_BUILD.md`)](./docs/PRODUCTION_BUILD.md)

---

## License

Licensed under the [MIT License](./LICENSE). See [docs/LICENSING.md](./docs/LICENSING.md) for the third-party dependency license inventory and governance policy.
