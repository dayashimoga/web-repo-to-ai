# Web & Repo to AI (`web-repo-to-ai`)

**Universal URL & GitHub Repository → AI-Ready Content Converter**  
Open-source-first, Rust-powered, web-first, self-hostable conversion platform.

---

## Overview

`web-repo-to-ai` converts publicly accessible webpages, public GitHub repositories, authorized private GitHub repositories, and batch URL sets into accurate, structured, provenance-traceable, token-efficient context packages for:

- **Large Language Models** (ChatGPT, Claude, Gemini, local models)
- **AI Coding Agents & IDEs**
- **Code Reviewers & Security Auditors**
- **RAG & Knowledge Retrieval Systems**
- **Developers, Researchers, and Technical Teams**

All core extraction, structural normalization, canonical modeling, deduplication, and token optimization operate **without requiring paid AI API keys, external LLMs, or commercial scraping services**.

---

## Core Capabilities

1. **Universal Webpage Extraction**: Fast HTTP static extraction (`html5ever`/`scraper` + readability/documentation extraction) with isolated Playwright + Chromium dynamic rendering fallback.
2. **Complete GitHub Repository Conversion**: Public and authorized private repositories (via minimal read-only GitHub App), supporting branch/tag/commit selection, selective file trees, and strict completeness accounting.
3. **Canonical Context Document Model (CDM)**: Versioned intermediate representation preserving exact source code, document hierarchy, tables, metadata, hashes, line ranges, exclusions, and provenance.
4. **AI Context & Token Intelligence**: Lossless clean extraction, semantic-aware deduplication, context profiles (General AI, Code Review, Security Review, Debugging, Documentation, RAG), model-specific token counting, and deterministic budget chunking.
5. **Multi-Format Exports**: AI-ready Markdown (default), TXT, JSON, HTML, JSONL, and streaming ZIP archives with manifests, checksums, and quality reports.
6. **Defense-in-Depth Security**: Mandatory SSRF/DNS-rebinding protection, redirect revalidation, isolated browser rendering, encrypted GitHub credentials, secret redaction, and untrusted content boundaries.

---

## Monorepo Architecture

```text
web-repo-to-ai/
├── apps/
│   └── web/                    # SvelteKit + Svelte 5 + TypeScript + Tailwind CSS frontend
├── services/
│   ├── api/                    # Rust Axum HTTP API service
│   ├── worker/                 # Rust asynchronous extraction & optimization worker
│   └── browser-worker/         # Isolated Playwright + Chromium rendering service
├── crates/
│   ├── core/                   # Shared domain types, config, and error taxonomy
│   ├── cdm/                    # Versioned Canonical Context Document Model
│   ├── url-security/           # URL validation, DNS resolution, and SSRF protection
│   ├── web-extractor/          # Static & dynamic HTML/article/docs extraction
│   ├── github-extractor/       # GitHub REST/GraphQL/Git tree & source acquisition
│   ├── code-intelligence/      # Tree-sitter parsing, language & symbol analysis
│   ├── normalizer/             # Structural normalization & clean serialization
│   ├── deduplication/          # Content-hash & semantic-aware deduplication
│   ├── tokenizer/              # Model-specific tokenizers & explicit estimators
│   ├── context-optimizer/      # Context budget prioritization & profile shaping
│   ├── chunking/               # Deterministic, boundary-aware chunking
│   ├── exporters/              # Markdown, TXT, JSON, HTML, JSONL, and ZIP exporters
│   ├── storage/                # Local filesystem & S3-compatible artifact storage
│   ├── job-engine/             # PostgreSQL-backed durable job state & leasing
│   └── observability/          # Structured tracing, metrics, and audit logging
├── deploy/                     # Podman Compose, containerfiles, and Caddy configs
├── scripts/                    # Environment inspection, resource budget, and dev scripts
├── docs/                       # Comprehensive architecture, security, API, and user docs
└── .agent/                     # Antigravity agent skills, rules, and workflows
```

---

## Governance & Documentation

- [Global Engineering Contract](./GLOBAL_ENGINEERING_CONTRACT.md)
- [Project Status](./PROJECT_STATUS.md)
- [Implementation Ledger](./IMPLEMENTATION_LEDGER.md)
- [Roadmap](./ROADMAP.md)
- [Requirements Specification](./REQUIREMENTS.md)
- [Architecture Decisions](./DECISIONS.md)
- [Agent Organization & Ownership](./AGENTS.md)
- [Todo Tracker](./TODO.md)
- [Changelog](./CHANGELOG.md)

### Detailed Documentation (`docs/`)
- [Requirements (`docs/REQUIREMENTS.md`)](./docs/REQUIREMENTS.md)
- [System Architecture (`docs/ARCHITECTURE.md`)](./docs/ARCHITECTURE.md)
- [Security & Threat Model (`docs/SECURITY.md`)](./docs/SECURITY.md)
- [Data Model & CDM (`docs/DATA_MODEL.md`)](./docs/DATA_MODEL.md)
- [API Reference (`docs/API.md`)](./docs/API.md)
- [Development Guide (`docs/DEVELOPMENT.md`)](./docs/DEVELOPMENT.md)
- [Deployment Guide (`docs/DEPLOYMENT.md`)](./docs/DEPLOYMENT.md)
- [User Guide (`docs/USER_GUIDE.md`)](./docs/USER_GUIDE.md)
- [Testing & Validation (`docs/TESTING.md`)](./docs/TESTING.md)
- [Performance & Benchmarks (`docs/PERFORMANCE.md`)](./docs/PERFORMANCE.md)
- [Third-Party Licensing (`docs/LICENSING.md`)](./docs/LICENSING.md)
- [Staged Roadmap (`docs/ROADMAP.md`)](./docs/ROADMAP.md)

---

## License

Licensed under the [MIT License](./LICENSE). See [docs/LICENSING.md](./docs/LICENSING.md) for third-party dependency license policy and inventory.
