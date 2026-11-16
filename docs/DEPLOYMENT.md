# Deployment & Self-Hosting Guide (`docs/DEPLOYMENT.md`)

**Project:** `web-repo-to-ai`

---

## 1. Deployment Models

### 1.1 Self-Hosted Container Stack (Primary)
Runs via **Podman** and **Podman Compose** (`deploy/podman-compose.yml`):
- `postgres`: PostgreSQL 16 Alpine container with persistent volume for job metadata and leases.
- `api`: Rust Axum HTTP API service container.
- `worker`: Rust extraction & token optimization worker container.
- `browser-worker`: Isolated Playwright + Chromium rendering service container on a restricted network.
- `web`: SvelteKit web frontend container.
- `caddy` (optional profile): Automatic HTTPS reverse proxy.

### 1.2 Hybrid Cloud / Edge Deployment (Optional)
- **Frontend:** Cloudflare Pages (`apps/web`).
- **Storage:** S3-compatible object storage or Cloudflare R2 (`crates/storage`).
- **Processing:** Self-hosted Rust `api`, `worker`, and `browser-worker` instances with PostgreSQL.

---

## 2. Resource & Storage Constraints

- **30 GB Podman Ceiling:** All container images use multi-stage builds with minimal runtime base images (`debian:bookworm-slim` / `alpine`) so the combined image + volume footprint remains well within the 30 GB Podman VM disk ceiling.
- **Artifact Retention:** Automatic background TTL cleanup purges expired artifacts from local/S3 storage and `job_artifacts`.
