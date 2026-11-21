# User Guide & Platform Operations (`docs/USER_GUIDE.md`)

**Project:** `web-repo-to-ai` (Universal URL & GitHub Repository → AI-Ready Content Converter)  
**Target Audience:** End Users, Developers, AI Engineers, Security Auditors, and Agentic Integrators

---

## Table of Contents

1. [Overview & Core Philosophy](#1-overview--core-philosophy)
2. [Web Conversion Workspace](#2-web-conversion-workspace)
3. [Public GitHub Repository Conversion](#3-public-github-repository-conversion)
4. [Private GitHub Repository Integration](#4-private-github-repository-integration)
5. [Multi-Target Batch Processing](#5-multi-target-batch-processing)
6. [Command-Line Interface (CLI)](#6-command-line-interface-cli)
7. [Model Context Protocol (MCP) Server](#7-model-context-protocol-mcp-server)
8. [Chrome Browser Extension (Manifest V3)](#8-chrome-browser-extension-manifest-v3)
9. [Advanced Documents (PDF, OCR & Structured Specs)](#9-advanced-documents-pdf-ocr--structured-specs)
10. [AI Context Profiles & Token Optimization](#10-ai-context-profiles--token-optimization)
11. [RAG Chunks & Vector Database Ingestion](#11-rag-chunks--vector-database-ingestion)
12. [Local AI Adapters & Zero-Key Fallback](#12-local-ai-adapters--zero-key-fallback)

---

## 1. Overview & Core Philosophy

`web-repo-to-ai` converts publicly accessible webpages, GitHub repositories, complex documents, and batch collections into clean, structured, token-optimized context packages tailored for AI consumption.

### Key Operational Invariants
- **Zero Required AI Keys:** All core extraction, normalization, deduplication, token estimation, and chunking execute locally on CPU without requiring external paid LLM services or API keys.
- **100% Completeness Guarantee:** Every repository conversion accounts for every discovered file in a cryptographic balance sheet ($\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed}$).
- **Mathematical Idempotency:** Normalization, deduplication, and export generation satisfy $f(f(x)) \equiv f(x)$, ensuring bit-for-bit repeatability across repeated passes.
- **Zero-Trust Security:** Mandatory SSRF defenses, IPv4/IPv6 private IP filtering, cloud metadata service isolation, hop-by-hop redirect verification, and automated secret redaction.

---

## 2. Web Conversion Workspace

The primary web application (`apps/web`) provides an intuitive, responsive interface built with SvelteKit, Svelte 5 runes, and Tailwind CSS.

### 2.1 Converting a Webpage
1. Paste any valid `http://` or `https://` URL into the main input bar.
2. The instant **Source Detector** classifies the target (e.g. `WebpageStatic`, `WebpageDynamic`, or `StructuredDocument`).
3. Select your target **AI Model Preset** (e.g., GPT-4o, Claude 3.5 Sonnet, or Gemini 1.5 Pro) and configure token budget limits using the interactive slider.
4. If the page relies heavily on client-side JavaScript rendering, toggle **Dynamic Browser Rendering** (automatic Playwright headless fallback is enabled by default).
5. Click **Convert** to initiate the 4-stage pipeline tracker:
   - **Acquisition:** SSRF validation and HTTP retrieval.
   - **Extraction:** Article readability and DOM hierarchy extraction.
   - **Optimization:** Text normalization, deduplication, and token budgeting.
   - **Packaging:** Multi-format rendering and cryptographic checksum derivation.
6. Review the output in the syntax-highlighted **Preview Panel**, inspect token savings in the **Token Budget Bar**, and copy or download the resulting artifact.

---

## 3. Public GitHub Repository Conversion

`web-repo-to-ai` ingests public repositories directly using GitHub's Git Trees API or raw tree traversal.

### 3.1 Step-by-Step Conversion
1. Enter a GitHub repository address:
   - Root URL: `https://github.com/owner/repo`
   - Specific branch or tag: `https://github.com/owner/repo/tree/v2.0`
   - Subdirectory: `https://github.com/owner/repo/tree/main/crates/core`
   - Shorthand: `owner/repo`
2. Select your desired Git reference (**Branch**, **Tag**, or specific **Commit SHA**).
3. Optionally configure **Subpath Filters** and **File Category Exclusions** (e.g., skip vendor files, test fixtures, or minified binaries).
4. Review the **Completeness Accounting Balance Sheet**:
   - Total files discovered in the Git tree.
   - Files extracted into the Canonical Context Document Model (CDM).
   - Files excluded with explicit reason codes (`BinarySkipped`, `VendorPath`, `OversizedFile`, `PatternExcluded`).
5. Copy the unified Markdown directly into your clipboard or download a complete `.zip` archive containing the formatted document, `manifest.json`, `quality-report.json`, and `checksums.sha256`.

---

## 4. Private GitHub Repository Integration

Private repository conversion operates through an authorized, read-only GitHub App.

### 4.1 Authorization & Security Boundaries
1. Click **Connect GitHub** in the conversion workspace.
2. Authorize the `web-repo-to-ai` GitHub App for selected repositories.
3. The App requests strictly **Read-Only** permissions:
   - `Contents: Read-only` (read repository trees and blobs)
   - `Metadata: Read-only` (read basic repository metadata)
   - **Zero Write Permissions:** The App cannot modify repositories, issues, or pull requests.
4. All OAuth and installation access tokens are encrypted server-side using **AES-256-GCM** with Session-Bound Additional Authenticated Data (AAD).
5. Tokens are never exposed to client-side scripts, never logged in telemetry, and can be revoked instantly at any time from your GitHub settings.

---

## 5. Multi-Target Batch Processing

For bulk conversions across multiple documentation sites, repositories, or URLs, the platform supports multi-target batch execution:

- **Web UI Batch Input:** Paste multiple URLs separated by newlines or upload a `.txt` target manifest.
- **Dynamic Flow Control:** Batches are processed concurrently under adaptive concurrency ceilings (AIMD) and watermark backpressure, preventing rate limiting or system overload.
- **Fault Isolation:** Individual failures in a batch do not abort sibling tasks (`--continue-on-error`).
- **Unified Summary:** The batch summary report categorizes successful, failed, and skipped items with detailed per-item durations and token counts.

---

## 6. Command-Line Interface (CLI)

The `web-repo-to-ai` CLI provides a unified binary for local conversions, automated scripts, CI/CD pipelines, and terminal workflows.

### 6.1 Installation
```bash
cargo install --path crates/cli
```

### 6.2 Primary Commands

#### `web-repo-to-ai url`
Converts a single webpage to AI context:
```bash
# Convert to Markdown (default)
web-repo-to-ai url https://example.com/docs

# Convert with specific format and token ceiling
web-repo-to-ai url https://example.com/docs --format json --max-tokens 8000

# Remote execution via running API server
web-repo-to-ai url https://example.com/docs --api-url http://localhost:3000 --api-key wr_live_...

# Output structured JSON envelope for script consumption
web-repo-to-ai url https://example.com/docs --json
```

#### `web-repo-to-ai repo`
Converts a GitHub repository or local directory:
```bash
# Ingest remote GitHub repository
web-repo-to-ai repo tokio-rs/tokio --format markdown --max-tokens 100000

# Target specific branch and subpath
web-repo-to-ai repo tokio-rs/tokio --ref master --subpath tokio/src

# Ingest local project directory
web-repo-to-ai repo /path/to/my-project --format zip --output-dir ./dist
```

#### `web-repo-to-ai batch`
Processes a list of targets from a file or stdin:
```bash
# Convert targets from file with bounded concurrency
web-repo-to-ai batch targets.txt --concurrency 4 --output-dir ./exports

# Convert targets from stdin
cat urls.txt | web-repo-to-ai batch - --continue-on-error
```

#### `web-repo-to-ai config`
Manages CLI configurations:
```bash
web-repo-to-ai config show         # Display merged configuration
web-repo-to-ai config init         # Generate template config file
web-repo-to-ai config validate     # Verify config syntax and file paths
web-repo-to-ai config path         # Print active config file location
```

#### `web-repo-to-ai verify`
Verifies artifact package cryptographic checksums and schema integrity:
```bash
# Verify checksums manifest
web-repo-to-ai verify checksums.sha256

# Verify ZIP package internal consistency
web-repo-to-ai verify export-package.zip

# Strict CDM JSON schema validation
web-repo-to-ai verify document.json --strict
```

#### `web-repo-to-ai completions`
Generates shell autocompletions:
```bash
web-repo-to-ai completions bash > /etc/bash_completion.d/web-repo-to-ai
web-repo-to-ai completions zsh > "${fpath[1]}/_web-repo-to-ai"
web-repo-to-ai completions fish > ~/.config/fish/completions/web-repo-to-ai.fish
web-repo-to-ai completions powershell >> $PROFILE
```

---

## 7. Model Context Protocol (MCP) Server

`web-repo-to-ai-mcp` implements the official Model Context Protocol (specification 2024-11-05), exposing conversion tools and resources over standard I/O (`stdio`) for AI agents.

### 7.1 Desktop Client Configuration

#### Claude Desktop (`claude_desktop_config.json`)
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

#### Cursor / Windsurf / Antigravity MCP Settings
Configure a stdio MCP server pointing to the `web-repo-to-ai-mcp` executable (or `web-repo-to-ai mcp`).

### 7.2 Available MCP Tools

| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `convert_url` | `url`, `format`, `max_tokens`, `profile` | Converts any public webpage into AI context. |
| `convert_repo` | `target`, `git_ref`, `subpath`, `format`, `max_tokens` | Converts a GitHub repository or local directory. |
| `detect_source` | `input` | Inspects a string and categorizes source type and metadata. |
| `optimize_context` | `text`, `model`, `max_tokens` | Calculates exact token counts and prunes budget. |

### 7.3 Available MCP Resources

| URI Template | MIME Type | Description |
| :--- | :--- | :--- |
| `cdm://{doc_id}/markdown` | `text/markdown` | AI-ready Markdown document representation. |
| `cdm://{doc_id}/json` | `application/json` | Full Canonical Context Document Model JSON. |
| `cdm://{doc_id}/zip` | `application/zip` | Base64-encoded streaming ZIP export archive. |
| `cdm://{doc_id}/section/{section_id}` | `text/plain` | Granular section body extraction by section ID. |
| `file://{path}` | Auto-detected | Local workspace file retrieval within allowed root boundaries. |

---

## 8. Chrome Browser Extension (Manifest V3)

The Chrome Extension (`apps/extension`) enables instant one-click webpage extraction directly from your browser.

### 8.1 Installation
1. Navigate to `apps/extension` and build the distribution:
   ```bash
   npm install && npm run build
   ```
2. In Google Chrome or Chromium, open `chrome://extensions/`.
3. Enable **Developer mode** (top-right toggle).
4. Click **Load unpacked** and select `apps/extension/dist`.

### 8.2 Usage & Features
- **One-Click Page Capture:** Click the extension toolbar icon to extract clean readability content and DOM structure from the active tab.
- **Local Token Meter:** Instant token estimate using built-in BPE approximation.
- **Copy to Clipboard:** Copy formatted Markdown directly into AI prompts with one click.
- **Open in Web App:** Deep-link the active tab directly into your self-hosted `web-repo-to-ai` workspace for advanced token tuning and multi-format exports.
- **Options Configuration:** Configure default API server URL, Bearer API keys, and context profile presets.

---

## 9. Advanced Documents (PDF, OCR & Structured Specs)

Beyond HTML and Git repositories, `web-repo-to-ai` natively processes complex technical documentation:

### 9.1 PDF Documents
- **Spatial Layout Reconstruction:** Extracts text blocks, coordinates, and bounding boxes, sorting content in human reading order ($y$-axis primary, $x$-axis secondary).
- **Table Extraction:** Detects tabular structures and serializes them to Markdown tables.
- **OCR Fallback:** Scanned PDF pages automatically route through the embedded **Tesseract OCR engine** for image text recovery.

### 9.2 Structured Specifications & Notebooks
- **OpenAPI 3.1.0 / Swagger:** Parses API schemas, endpoints, headers, and request/response models into organized markdown documentation.
- **Tabular Data (CSV/TSV):** Parses rows into structured markdown tables with token truncation guards.
- **Jupyter Notebooks (`.ipynb`):** Serializes markdown narrative cells and code execution cells while stripping noisy base64 binary image outputs.

---

## 10. AI Context Profiles & Token Optimization

`web-repo-to-ai` includes fine-tuned context profiles designed for specific AI workflow archetypes:

| Context Profile | Optimization Strategy | Best For |
| :--- | :--- | :--- |
| `General AI` | Balanced prose, code files, and essential documentation. | General questions and summaries. |
| `Code Review` | Prioritizes modified files, public APIs, and interface definitions. | PR reviews and architectural analysis. |
| `Security Review` | Prioritizes auth, routing, crypto, config, and dependency manifests. | Security audits and vulnerability scans. |
| `Debugging` | Retains full implementations, error handlers, and stack traces. | Bug investigation and troubleshooting. |
| `Documentation` | Prioritizes READMEs, docstrings, OpenAPI specs, and guides. | Knowledge base creation and technical docs. |
| `RAG` | Optimizes for boundary-aware chunking with rich citation anchors. | Vector database ingestion and semantic search. |

---

## 11. RAG Chunks & Vector Database Ingestion

When generating context for Retrieval-Augmented Generation (RAG) systems, `web-repo-to-ai` provides semantic, AST-boundary-aware chunking and direct vector database export schemas:

### 11.1 Supported Vector Adapters (`crates/chunking`)
- **Generic JSONL:** Newline-delimited JSON objects with chunk IDs, coordinates, and metadata.
- **Qdrant:** Points and batch upload payloads with deterministic UUIDs.
- **Pinecone:** Upsert vectors with flattened metadata dictionaries and symbol arrays.
- **ChromaDB:** Document collections with parallel-array batch payloads.
- **Milvus:** Entity rows with partition keys, source hashes, and line coordinates.
- **Weaviate:** Class objects conforming to RFC 4122 deterministic UUID rules.
- **OpenAI Batch Embeddings:** `POST /v1/embeddings` batch envelopes ready for embedding pipelines.
- **Cohere Embed:** Input arrays optimized for Cohere representation models.

---

## 12. Local AI Adapters & Zero-Key Fallback

`web-repo-to-ai` supports optional local and external AI adapters (`crates/ai-adapter`) governed by strict opt-in policies:

### 12.1 Execution Modes
- `Disabled` (Default): All AI operations use deterministic CPU-only algorithms (extractive summarization, heuristic symbol ranking, BLAKE3 pseudo-embeddings). Zero network calls are made to any AI endpoint.
- `LocalOnly`: Permits connections only to local daemons (e.g., Ollama at `http://localhost:11434` or local vLLM/LM Studio). Strictly blocks remote internet hosts and cloud metadata endpoints (`169.254.169.254`).
- `AllowRemote`: Enables connections to external OpenAI-compatible endpoints with Bearer authentication.

### 12.2 Local Ollama Configuration
```bash
# Start local Ollama daemon
ollama serve

# Run extraction with local AI enrichment
web-repo-to-ai url https://example.com/article --profile general
```
If the local model daemon is offline, the pipeline seamlessly falls back to the deterministic zero-key engine without failing the conversion.
