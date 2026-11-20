# Performance Engineering & Resource Budgets (`docs/PERFORMANCE.md`)

**Project:** `web-repo-to-ai`

---

## 1. Hardware & Resource Envelope

| Resource | Hard Limit / Target | Enforcement Mechanism |
| :--- | :--- | :--- |
| **Execution Hardware** | CPU-only, 16 GB System RAM | Zero GPU dependencies; bounded Tokio worker semaphores. |
| **Podman Disk Usage** | <= 30 GB total | Multi-stage slim images, shared layer caching, `scripts/check-resources.ps1`. |
| **Single File Size** | Default 10 MB (configurable) | Streaming byte-counter abort in `web-extractor` and `github-extractor`. |
| **Repository Archive Size** | Default 250 MB (configurable) | Streaming reader cap + explicit `Oversized` exclusion classification. |
| **Job Timeout** | Default 300s (configurable) | Tokio timeout + lease expiration in `job-engine`. |
| **Browser Navigation Timeout** | Default 30s (configurable) | Playwright page navigation timeout & resource interceptor. |

---

## 2. Key Performance Optimizations

1. **Static-First Web Acquisition:** Avoids Chromium startup for >85% of documentation and article URLs.
2. **Streaming Exports:** ZIP and JSONL generators stream directly to storage buffers without holding full archives in memory.
3. **Content-Addressed Blob Caching (`BLAKE3`):** Prevents re-parsing, re-normalizing, and re-tokenizing unchanged Git blobs across commits or branches.

---

## 3. Extraction Performance Benchmarks & SLAs

Empirically validated via automated regression tests in `services/worker/tests/extraction_benchmarks_test.rs` under unoptimized debug and release baselines on standard x86_64 host hardware.

### 3.1 Web HTML Extraction Performance

| Pipeline Stage | Sample Size | $P_{50}$ Latency | $P_{90}$ Latency | $P_{95}$ Latency | $P_{99}$ Latency | Throughput | Target SLA ($P_{95}$) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **WHATWG DOM Parsing (`tl`)** | ~40 KB HTML | 1.01 ms | 1.43 ms | 1.55 ms | 1.86 ms | **36.67 MB/s** | <= 30.0 ms |
| **Article Readability & Scoring** | ~40 KB HTML | 4.26 ms | 4.85 ms | 5.22 ms | 5.55 ms | **8.94 MB/s** | <= 50.0 ms |
| **Technical Docs & Hierarchy** | ~40 KB HTML | 3.14 ms | 4.55 ms | 4.58 ms | 5.25 ms | **11.35 MB/s** | <= 40.0 ms |

*Invariants Enforced:*
- Zero-copy HTML token streaming where applicable.
- HTML tag stripping and boilerplate pruning execute in linear time $O(N)$ with respect to DOM node count.
- Minimum throughput floor: >= 3.0 MB/s for DOM parsing, >= 2.0 MB/s for readability heuristics.

---

### 3.2 Structured Format Ingestion

| Archetype / Format | Payload Dimensions | $P_{50}$ Latency | $P_{90}$ Latency | $P_{95}$ Latency | $P_{99}$ Latency | Throughput | Target SLA ($P_{95}$) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Tabular Delimited (CSV/TSV)** | 2,500 rows x 8 cols (~250 KB) | 30.99 ms | 36.01 ms | 38.13 ms | 41.56 ms | **7.83 MB/s** | <= 80.0 ms |
| **OpenAPI 3.1.0 Specification** | 20 endpoints (~21 KB JSON) | 1.57 ms | 2.06 ms | 2.34 ms | 2.53 ms | **12.13 MB/s** | <= 20.0 ms |
| **Jupyter Notebook (`ipynb`)** | 20 cells (~11 KB JSON) | 0.98 ms | 1.09 ms | 1.11 ms | 1.14 ms | **12.01 MB/s** | <= 15.0 ms |

*Invariants Enforced:*
- Stream-based tabular row parsing with strict chunk allocation limits.
- OpenAPI hierarchical traversal resolves schema references and endpoint summaries without memory bloat.
- Jupyter Notebook cell extraction partitions markdown narration, code blocks, and execution outputs deterministically.

---

### 3.3 PDF 2D Spatial Layout & Reading-Order Sorting

| Archetype Suite | Sample Document Count | $P_{50}$ Latency | $P_{90}$ Latency | $P_{95}$ Latency | $P_{99}$ Latency | Throughput | Target SLA ($P_{95}$) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Golden PDF Archetype Suite** *(Academic Multi-Column, Financial Report, API Spec, Hybrid Scanned, Spatial Noise)* | 5 diverse documents (~11.4 KB total) | 6.25 ms | 7.34 ms | 8.47 ms | 8.47 ms | **1.93 MB/s** | <= 100.0 ms |

*Invariants Enforced:*
- Multi-column topological sorting runs $O(N \log N)$ 2D spatial coordinate projection.
- Encrypted documents enforce ISO 32000-1 permission bitmasks (`can_copy = false`) and reject unauthorized extraction gracefully without CPU spin.
- Scanned document detector computes text-to-image raster heuristics in < 2 ms per page.

---

### 3.4 Polyglot AST Code Intelligence Parsing

| Language Target | Analyzed Artifacts | $P_{50}$ Latency | $P_{90}$ Latency | $P_{95}$ Latency | $P_{99}$ Latency | Throughput | Target SLA ($P_{95}$) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Multi-Language Suite (`rs`, `py`, `ts`, `go`)** | 4 full modules (~3.3 KB combined) | 0.28 ms | 0.37 ms | 0.40 ms | 0.43 ms | **11.14 MB/s** | <= 20.0 ms |

*Invariants Enforced:*
- Complete symbol tree extraction (functions, methods, classes, structs, traits, interfaces) across all 4 primary languages finishes in < 1 ms per batch.
- Line-of-code metrics (total lines, code lines, comment lines, blank lines, doc comments) calculated in a single linear pass.

---

### 3.5 Normalization & Token Estimation Throughput

| Pipeline Step | Corpus Size | $P_{50}$ Latency | $P_{90}$ Latency | $P_{95}$ Latency | $P_{99}$ Latency | Throughput | Target SLA ($P_{95}$) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Structural Text Normalization** | 250 KB polyglot text | 11.15 ms | 12.13 ms | 12.14 ms | 12.97 ms | **22.11 MB/s** | <= 25.0 ms |
| **Dual BPE Token Counting (GPT-4o + Claude 3.5)** | 500 KB equivalent text | 84.85 ms | 93.15 ms | 93.30 ms | 100.79 ms | **5.63 MB/s** | <= 150.0 ms |

*Invariants Enforced:*
- Normalization guarantees idempotent UTF-8 NFKC canonicalization, smart quote normalization, and whitespace pruning at > 20 MB/s.
- Dual-model BPE tokenization counts exact or calibrated token counts with honest uncertainty intervals ($\pm 7.5\%$ bound for heuristic fast paths).

---

### 3.6 Multi-Threaded Concurrency Scaling

Empirically verified with 64 interleaved extraction tasks (HTML DOM parsing, Code Intelligence, and CSV Tabular ingestion) dispatched across bounded Tokio worker threads:

| Concurrency Tier | Total Execution Time | Task Completion Rate | Scaling Speedup | Efficiency Ratio |
| :--- | :--- | :--- | :--- | :--- |
| **1 Worker Thread** | 191.91 ms | 333.5 tasks/s | **1.00x** (Baseline) | 100.0% |
| **2 Worker Threads** | 87.02 ms | 735.5 tasks/s | **2.21x** | 110.5% (Cache warm) |
| **4 Worker Threads** | 49.11 ms | 1303.1 tasks/s | **3.91x** | 97.8% (Near-linear) |
| **8 Worker Threads** | 33.23 ms | 1925.7 tasks/s | **5.77x** | 72.1% (Workstation bounded) |

*Concurrency Invariants:*
- Linear or near-linear scaling up to available physical cores.
- Thread-safe, lock-free parsers prevent cross-worker lock contention.
- Bounded semaphore permits strictly cap memory overhead during concurrent extraction spikes.
