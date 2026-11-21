# Local Development Guide (`docs/DEVELOPMENT.md`)

**Project:** `web-repo-to-ai`  
**Status:** Certified Development Guide v1.0  
**Scope:** Developer Environment Setup, Build Toolchains, Quality Gates, and Lifecycle Workflows

---

## 1. Workstation Prerequisites & System Constraints

All development workflows are engineered and calibrated for standard commodity workstations:

- **Target Hardware Constraints:**
  - **Architecture:** `x86_64` or `aarch64` CPU (CPU-only execution, zero GPU or CUDA requirements).
  - **System Memory:** Minimum 16 GB physical RAM.
  - **Storage Quota:** Strict $\le 30$ GB ceiling for container images, volumes, and intermediate build caches.
- **Required Host Toolchains:**
  - **Git:** $\ge 2.40$ (configured for LF or native line endings via `.editorconfig`).
  - **Rust:** Stable Rust toolchain ($\ge 1.80$, tested on `1.97.1`).
  - **Node.js & npm:** Node $\ge 20$ LTS (`v20.18.0+`) and npm $\ge 10$ (`10.8.2+`).
  - **Container Engine:** Podman ($\ge 5.x$) or Docker Desktop with Compose support.
  - **PowerShell / Bash:** PowerShell 5.1+ / pwsh 7+ on Windows; POSIX bash on Linux and macOS.

> [!IMPORTANT]
> **Windows Memory Bound Enforcement:** To prevent paging exhaustion and Windows virtual memory allocation errors (`os error 1455: The paging file is too small for this operation to complete`), all Cargo test commands **must** specify `--jobs 2` and clippy commands **must** specify `--jobs 4`. This is automatically enforced by `scripts/check-all.ps1`.

---

## 2. Monorepo Structure & Package Inventory

The monorepo contains 26 distinct packages:

```text
web-repo-to-ai/
├── apps/
│   ├── extension/              # Chrome Extension MV3 (TypeScript, Tailwind, Vitest)
│   └── web/                    # Frontend Web App (SvelteKit, Svelte 5 runes, TypeScript)
├── services/
│   ├── api/                    # REST & SSE API Service (Rust Axum, SQLx)
│   ├── browser-worker/         # Isolated Headless Browser (Node.js, Playwright, Chromium)
│   └── worker/                 # Conversion & Pipeline Engine (Rust Tokio)
├── crates/
│   ├── ai-adapter/             # LLM Synthesis & Local Fallback Adapters
│   ├── cdm/                    # Canonical Context Document Model v1.0
│   ├── chunking/               # AST & Heading Semantic Chunking
│   ├── cli/                    # Universal Terminal CLI (web-repo-cli)
│   ├── code-intelligence/      # Tree-sitter Multi-Language Symbol Extraction
│   ├── context-optimizer/      # Token-Budget Allocation & Context Profiles
│   ├── core/                   # Common Error Types, Config & Identifiers
│   ├── deduplication/          # BLAKE3 Exact & Fuzzy Deduplication
│   ├── exporters/              # Markdown, JSON, HTML, RAG Vectors & ZIP Generation
│   ├── github-extractor/       # GitHub API & Git Trees Traversal
│   ├── job-engine/             # PostgreSQL Distributed Leasing & State Machine
│   ├── mcp/                    # Model Context Protocol stdio/SSE Server
│   ├── normalizer/             # Lossless Unicode, Table & Codeblock Normalizer
│   ├── observability/          # Structured Tracing & Secret-Redacting Logging
│   ├── ocr-extractor/          # Optical Character Recognition & Image Ingestion
│   ├── pdf-extractor/          # PDF Document Parsing & Layout Engine
│   ├── storage/                # Content-Addressable Storage (CAS) & LocalFs/S3
│   ├── structured-extractor/   # JSON/YAML/TOML/CSV/XML Extraction
│   ├── tokenizer/              # BPE Tokenizers (cl100k, o200k, llama3) & Estimators
│   ├── url-security/           # SSRF Defense, DNS Resolution & Private IP Guard
│   └── web-extractor/          # HTML Scraping, Readability & DOM Extraction
├── deploy/                     # Podman Compose manifests & Containerfiles
├── docs/                       # Technical architecture & governance specifications
└── scripts/                    # Automation scripts for build, test, and container lifecycle
```

---

## 3. End-to-End Developer Workflow Runbook

### Scenario A: Hybrid Native Development (Recommended)
This workflow offers sub-second compile and live-reload cycles. Only PostgreSQL runs in a container; all Rust crates and Node frontends run natively on the host:

```powershell
# 1. Start solely the PostgreSQL container
pwsh scripts/dev-up.ps1 -Services postgres

# 2. Terminal 1: Run the Rust REST API
cargo run --bin api

# 3. Terminal 2: Run the SvelteKit Web UI
cd apps/web
npm run dev

# 4. Terminal 3: Execute CLI conversion against the live API or standalone
cargo run --bin web-repo-cli -- url https://example.com --format markdown
```

### Scenario B: Full-Stack Containerized Staging
Test the complete 5-container architecture (PostgreSQL, API, Worker, Browser-Worker, Web UI) in isolated container networks:

```powershell
# Boot entire stack detached with automated resource budget auditing
pwsh scripts/dev-up.ps1

# Inspect runtime container status and healthchecks
pwsh scripts/dev-status.ps1

# Follow consolidated logs across all containers
pwsh scripts/dev-logs.ps1 -Follow

# Shut down the stack safely (preserving persistent database volumes)
pwsh scripts/dev-down.ps1
```

---

## 4. Comprehensive Monorepo Quality Gates (`scripts/check-all.*`)

Before submitting code or integrating subphases, all developers must execute the unified verification gate:

```powershell
# Windows PowerShell
powershell -ExecutionPolicy Bypass -File scripts/check-all.ps1
```
```bash
# Linux / macOS / CI Runner
bash scripts/check-all.sh
```

### The 5 Automated Verification Gates:

1. **Gate 1: Bootstrap & Governance Integrity:**
   - Validates that all 14 root governance contracts exist and are non-empty.
   - Audits all 10 agent skills (`.agent/skills/*/SKILL.md`) for YAML frontmatter and 7 mandatory sections.
   - Enforces linear Git history, default branch `main`, and absence of untracked temporary files.
2. **Gate 2: GitHub Actions Workflow Validation:**
   - Executes `python scripts/validate-workflows.py` to ensure all `.github/workflows/*.yml` files have valid YAML syntax, valid step names, correct runner labels (`ubuntu-latest`, `windows-latest`), and pin secure actions.
3. **Gate 3: Rust Workspace Quality Gate:**
   - Code formatting: `cargo fmt --all -- --check`
   - Strict static analysis: `cargo clippy --workspace --all-targets --jobs 4 -- -D warnings`
   - Complete unit and integration tests: `cargo test --workspace --jobs 2` (over 700+ tests passing)
   - Dependency security and license audit: `cargo deny check`
4. **Gate 4: Web Frontend Quality Gate (`apps/web`):**
   - Linter: `npm run lint` (ESLint)
   - Typechecking: `npm run check` (SvelteKit `svelte-check` with TypeScript)
   - Component and unit tests: `npm test` (Vitest with 210+ test assertions)
5. **Gate 5: Chrome Extension Quality Gate (`apps/extension`):**
   - TypeScript compilation and bundling: `npm run build`
   - Extension test suite: `npm test` (Vitest with 100+ test assertions)

---

## 5. Component Build & Test Guide

### 5.1 Rust Services & Crates
```powershell
# Compile all crates in release mode
cargo build --workspace --release

# Run tests for a specific crate with bounded jobs
cargo test -p web-repo-cdm --jobs 2
cargo test -p web-repo-worker --jobs 2

# Run integration and stress tests
cargo test -p web-repo-worker --test load_and_failure_testing_test --jobs 2
```

### 5.2 Frontend Web App (`apps/web`)
```powershell
cd apps/web

# Install dependencies
npm ci

# Run development server with Hot Module Replacement (HMR)
npm run dev

# Run Vitest unit tests in watch mode
npm run test:watch

# Produce optimized production build
npm run build
```

### 5.3 Chrome Extension (`apps/extension`)
```powershell
cd apps/extension

# Install dependencies
npm ci

# Build extension distribution package (outputs to apps/extension/dist/)
npm run build

# Run extension automated tests
npm test
```
*To load the extension in Google Chrome:* Navigate to `chrome://extensions`, enable **Developer mode**, click **Load unpacked**, and select the `apps/extension/dist` directory.

### 5.4 Universal CLI (`crates/cli`)
```powershell
# Display help and available subcommands
cargo run --bin web-repo-cli -- --help

# Test local URL extraction
cargo run --bin web-repo-cli -- url https://en.wikipedia.org/wiki/Rust_(programming_language) --profile summary

# Test GitHub repository extraction
cargo run --bin web-repo-cli -- repo https://github.com/octocat/Hello-World --token-budget 50000 --format markdown
```

### 5.5 MCP Server (`crates/mcp`)
```powershell
# Run the MCP server over stdio for local agent integration
cargo run --bin web-repo-mcp

# Test MCP server configuration via Claude Desktop or Cursor
# Add to claude_desktop_config.json:
# "web-repo-to-ai": {
#   "command": "cargo",
#   "args": ["run", "--bin", "web-repo-mcp", "--release"]
# }
```

---

## 6. Container Lifecycle & Storage Management

All container operations are managed via non-intrusive PowerShell and bash scripts located in `scripts/`:

| Command | Description |
| :--- | :--- |
| `pwsh scripts/dev-up.ps1` | Launches all containers detached after verifying the 30 GB storage ceiling. |
| `pwsh scripts/dev-up.ps1 -Services postgres` | Starts only PostgreSQL for local hybrid development. |
| `pwsh scripts/dev-status.ps1` | Displays runtime status, open ports, healthcheck states, and storage footprint. |
| `pwsh scripts/dev-logs.ps1 -Follow` | Streams consolidated logs across all containers. |
| `pwsh scripts/dev-restart.ps1 -Services api` | Restarts the API container and waits for database health verification. |
| `pwsh scripts/dev-down.ps1` | Stops all services while preserving database and artifact storage volumes. |
| `pwsh scripts/dev-down.ps1 -Volumes` | Prompts for confirmation and deletes persistent volumes for a fresh state. |
| `pwsh scripts/check-resources.ps1` | Audits CPU, RAM, disk space, and evaluates Podman storage against the 30 GB ceiling. |
| `pwsh scripts/clean-caches.ps1` | Safely prunes build caches, intermediate layers, and stopped build containers. |

---

## 7. Troubleshooting & Disaster Recovery Runbook

### 7.1 Windows Virtual Memory / Paging Exhaustion (`os error 1455`)
- **Symptom:** `error: could not compile ... os error 1455: The paging file is too small for this operation to complete`.
- **Cause:** High concurrent compiler threads under multi-threaded linkers on 14+ core Windows workstations exceeding virtual page allocations.
- **Remedy:** Always pass `--jobs 2` to `cargo test` and `--jobs 4` to `cargo clippy`. Ensure `scripts/check-all.ps1` is used for full audits.

### 7.2 Directory Junction Paths on Windows
- **Symptom:** Git or Node scripts fail to resolve directory symlinks when `web-repo-to-ai` resides across drive junctions (e.g., `C:\Users\...\web-repo-to-ai` linked to `H:\web-repo-to-ai`).
- **Remedy:** `scripts/check-all.ps1` automatically detects directory junctions via `Get-Item . | Select-Object -ExpandProperty LinkType` and dynamically targets canonical volume roots before invoking `npm` or `cargo`.

### 7.3 Port Conflicts (`5432`, `8080`, `5173`, `3000`)
- **Symptom:** `Error: address already in use` upon booting containers or running native services.
- **Remedy:**
  1. Inspect listening ports:
     ```powershell
     Get-NetTCPConnection -LocalPort 5432,8080,5173,3000 -ErrorAction SilentlyContinue | Format-Table LocalAddress,LocalPort,OwningProcess
     ```
  2. If a local PostgreSQL instance is running on port 5432, adjust `POSTGRES_PORT=5433` in `deploy/.env` without modifying container source manifests.

### 7.4 Storage Ceiling Warnings (> 24 GB)
- **Symptom:** `scripts/check-resources.ps1` outputs `WARNING_STORAGE_HIGH` or startup is aborted.
- **Remedy:** Execute `pwsh scripts/clean-caches.ps1` to purge untagged build layers and intermediate cache mounts safely. Persistent database data is never touched.
