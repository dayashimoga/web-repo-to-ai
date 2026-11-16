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
