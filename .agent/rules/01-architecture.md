# Rule 01: Architecture & Dependency Boundaries

1. **Strict Layer Separation:** Keep `apps/web` (UI), `services/api` (HTTP/SSE), `services/worker` (background pipeline), `services/browser-worker` (isolated Playwright), and `crates/*` strictly separated.
2. **CDM Single Source of Truth:** All extractors (`web-extractor`, `github-extractor`) MUST emit a versioned `CdmDocument` (`crates/cdm`). All optimizers and exporters (`crates/exporters`) MUST consume `CdmDocument`.
3. **Acyclic Crate Graph:** Shared crates in `crates/*` must never depend on binary service crates (`services/*`).
4. **Externalized Large Artifacts:** Never store full repository archives or large extracted documents inside PostgreSQL rows; store metadata in PostgreSQL and payloads in `crates/storage`.
5. **Provider-Independent Traits:** Use trait abstractions for artifact storage (`StorageBackend`) and job orchestration (`JobEngine`).
