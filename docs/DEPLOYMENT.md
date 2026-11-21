# Deployment & Self-Hosting Guide (`docs/DEPLOYMENT.md`)

**Project:** `web-repo-to-ai`  
**Status:** Certified Deployment Guide v1.0  
**Scope:** Production Deployment, Container Topology, Binary Distribution, Environment Configuration & Disaster Recovery

---

## 1. Supported Deployment Models

`web-repo-to-ai` supports four distinct deployment topologies to accommodate environments ranging from single-developer workstations to air-gapped enterprise clusters:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        DEPLOYMENT TOPOLOGIES                           │
├───────────────────────┬───────────────────────┬────────────────────────┤
│ 1. Self-Hosted Stack  │ 2. Binary / Standalone│ 3. Chrome Extension    │
│    (Podman / Compose) │    (Linux/Win/macOS)  │    (Developer / Store) │
│ - 5 isolated containers│ - Standalone CLI      │ - MV3 Client Extension │
│ - Internal networks   │ - Standalone MCP      │ - Local DOM Extraction │
│ - Persistent volumes  │ - Native systemd/tasks│ - Direct API / MCP con │
├───────────────────────┴───────────────────────┴────────────────────────┤
│ 4. Hybrid Cloud / Edge Architecture                                    │
│ - Edge UI (Cloudflare Pages) + S3 Storage + Dedicated Ingestion Worker │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Model 1: Self-Hosted Container Stack (Primary)

The standard production deployment is fully containerized using **Podman** and **Podman Compose** (or Docker Compose), defined in `deploy/podman-compose.yml`.

### 2.1 Service Topology

| Container Service | Base Image | Ports | Purpose |
| :--- | :--- | :--- | :--- |
| `postgres` | `postgres:16-alpine` | `5432` | Relational store for jobs, leases, credentials, and state machine transitions. |
| `api` | `debian:bookworm-slim` | `8080` | Rust Axum REST API and Server-Sent Events (SSE) server. |
| `worker` | `debian:bookworm-slim` | *(None)* | Multi-stage extraction, AST analysis, deduplication, and optimization worker. |
| `browser-worker` | `node:20-slim` | `3000` | Isolated Playwright + Chromium headless browser for dynamic JS rendering. |
| `web` | `node:20-slim` | `5173` | SvelteKit + Svelte 5 production web interface. |
| `caddy` *(optional profile)* | `caddy:2-alpine` | `80`, `443` | Reverse proxy with automatic TLS certificate issuance. |

### 2.2 Network Architecture & Sandbox Isolation

To guarantee defense-in-depth isolation against Server-Side Request Forgery (SSRF) and container escapes:
- `internal_net`: Private bridge connecting `api`, `worker`, and `postgres`.
- `browser_net`: Segregated bridge connecting `worker` and `browser-worker`.
- **Strict Isolation Rule:** `browser-worker` is strictly forbidden from attaching to `internal_net`. It cannot communicate with PostgreSQL, and all its outbound subresource fetches are validated by URL security policies.

---

## 3. Model 2: Standalone Binary Distribution

For minimal overhead or air-gapped systems without container engines, standalone executable binaries are compiled across supported targets:

| Architecture | Target Triple | Binaries Produced |
| :--- | :--- | :--- |
| **Linux (x86_64)** | `x86_64-unknown-linux-gnu` / `musl` | `web-repo-api`, `web-repo-worker`, `web-repo-cli`, `web-repo-mcp` |
| **Linux (aarch64)** | `aarch64-unknown-linux-gnu` / `musl` | `web-repo-api`, `web-repo-worker`, `web-repo-cli`, `web-repo-mcp` |
| **Windows (x86_64)**| `x86_64-pc-windows-msvc` | `web-repo-api.exe`, `web-repo-worker.exe`, `web-repo-cli.exe`, `web-repo-mcp.exe` |
| **macOS (Apple Silicon)** | `aarch64-apple-darwin` | `web-repo-api`, `web-repo-worker`, `web-repo-cli`, `web-repo-mcp` |
| **macOS (Intel)** | `x86_64-apple-darwin` | `web-repo-api`, `web-repo-worker`, `web-repo-cli`, `web-repo-mcp` |

Compilation:
```bash
cargo build --release --bin web-repo-api --bin web-repo-worker --bin web-repo-cli --bin web-repo-mcp
```

---

## 4. Model 3: Chrome Extension Distribution

The Chrome Extension MV3 (`apps/extension`) runs client-side in Google Chrome, Brave, Edge, and other Chromium-based browsers:

1. **Production Build:**
   ```powershell
   cd apps/extension
   npm ci
   npm run build
   ```
2. **Distribution Package:**
   Outputs to `apps/extension/dist/`. Includes `manifest.json`, bundled background service workers, content scripts, and options UI.
3. **Enterprise Sideloading / Developer Installation:**
   - Open `chrome://extensions/`.
   - Enable **Developer mode**.
   - Click **Load unpacked** and select `apps/extension/dist`.

---

## 5. Resource & Storage Management (The 30 GB Ceiling)

In accordance with `GLOBAL_ENGINEERING_CONTRACT.md` and Rule 08 (`08-deployment.md`), the entire containerized deployment environment is strictly bounded by a **30 GB storage ceiling**:

- **Multi-Stage Container Builds:** Intermediate Cargo build dependencies and npm caches are discarded; production runtime containers contain only compiled static binaries and minimal dynamic libraries.
- **Threshold Alerts:**
  - **80% (24 GB):** Warning threshold triggering automated cache pruning alerts.
  - **100% (30 GB):** Critical threshold aborting builds to protect workstation host stability.
- **Automated Lifecycle Scripts:**
  ```powershell
  # Windows PowerShell
  pwsh scripts/check-resources.ps1      # Audit CPU, RAM, disk, and Podman quota
  pwsh scripts/clean-caches.ps1 -DryRun # Check reclaimable disk space
  pwsh scripts/clean-caches.ps1         # Purge intermediate layers and build caches safely
  ```
  ```bash
  # POSIX Equivalent
  bash scripts/check-resources.sh
  bash scripts/clean-caches.sh
  ```

---

## 6. Environment Variables Reference

All runtime configuration is managed through environment variables specified in `deploy/.env` (templated from `deploy/.env.example`):

| Variable | Default Value | Service | Sensitivity | Description |
| :--- | :--- | :--- | :---: | :--- |
| `POSTGRES_DB` | `webrepotodev` | `postgres`, `api`, `worker` | Low | PostgreSQL database name. |
| `POSTGRES_USER` | `webrepouser` | `postgres`, `api`, `worker` | Medium | Administrative database username. |
| `POSTGRES_PASSWORD` | `devpassword123` | `postgres`, `api`, `worker` | **High** | Database password. Must be rotated in production. |
| `POSTGRES_HOST` | `postgres` | `api`, `worker` | Low | Database hostname inside the container network. |
| `POSTGRES_PORT` | `5432` | `postgres`, `api`, `worker` | Low | Database port. |
| `DATABASE_URL` | `postgresql://webrepouser:...@postgres:5432/webrepotodev` | `api`, `worker` | **High** | Fully qualified connection string for SQLx. |
| `API_PORT` | `8080` | `api` | Low | Port bound by the Axum REST API server. |
| `API_HOST` | `0.0.0.0` | `api` | Low | Network interface bound by the Axum API server. |
| `RUST_LOG` | `info,web_repo_to_ai=debug` | `api`, `worker` | Low | Tracing filter directive controlling log verbosity. |
| `STORAGE_BACKEND` | `local` | `api`, `worker` | Low | Active artifact storage engine (`local` or `s3`). |
| `STORAGE_LOCAL_PATH` | `/data/artifacts` | `api`, `worker` | Low | Storage mount path for exported files and CAS blobs. |
| `BROWSER_WORKER_URL` | `http://browser-worker:3000` | `worker` | Low | Internal endpoint for the isolated Playwright service. |
| `PUBLIC_API_URL` | `http://127.0.0.1:8080/api/v1` | `web` | Low | Base URL for REST API endpoints consumed by the web UI. |
| `WEB_PORT` | `5173` | `web` | Low | Port bound by the SvelteKit frontend server. |
| `WEB_HOST` | `0.0.0.0` | `web` | Low | Host interface bound by the SvelteKit server. |
| `GITHUB_APP_ID` | *(unset / empty)* | `api`, `worker` | Medium | Optional GitHub App ID for authenticated repository conversion. |
| `GITHUB_APP_PRIVATE_KEY_PATH`| *(unset / empty)* | `api`, `worker` | **High** | Path to the RSA private key PEM file for GitHub App auth. |
| `GITHUB_APP_CLIENT_ID` | *(unset / empty)* | `api`, `web` | Medium | OAuth client ID for user authorization flow. |
| `GITHUB_APP_CLIENT_SECRET` | *(unset / empty)* | `api` | **High** | OAuth client secret for exchanging temporary auth codes. |
| `TOKEN_ENCRYPTION_KEY` | *(unset / empty)* | `api`, `worker` | **High** | 256-bit hex key used for `AES-256-GCM` secret encryption. |

---

## 7. Backup, Restore & Disaster Recovery Runbook

Guarantees recovery from catastrophic outages in **under 5 minutes**.

### 7.1 Database Backup & Restore

#### Create Backup:
```powershell
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupFile = "backups/postgres_$timestamp.dump"
podman exec web-repo-postgres pg_dump -U webrepouser -d webrepotodev -F c -b -v -f /tmp/backup.dump
podman cp web-repo-postgres:/tmp/backup.dump $backupFile
podman exec web-repo-postgres rm -f /tmp/backup.dump
```

#### Execute Restore:
```powershell
podman cp backups/latest.dump web-repo-postgres:/tmp/restore.dump
podman exec web-repo-postgres pg_restore -U webrepouser -d webrepotodev --clean --if-exists -v /tmp/restore.dump
podman exec web-repo-postgres rm -f /tmp/restore.dump
```

### 7.2 Artifact & CAS Volume Backup & Restore

```bash
# Backup: Archive storage volume to compressed tarball
podman run --rm -v deploy_artifact_data:/data:ro -v $(pwd)/backups:/backup alpine tar czf /backup/artifacts_backup.tar.gz -C /data .

# Restore: Extract tarball into fresh storage volume
podman run --rm -v deploy_artifact_data:/data -v $(pwd)/backups:/backup alpine sh -c "rm -rf /data/* && tar xzf /backup/artifacts_backup.tar.gz -C /data"
```

### 7.3 Cold Recovery Procedure (< 5 Minutes)

```text
1. pwsh scripts/dev-down.ps1
2. pwsh scripts/clean-caches.ps1
3. pwsh scripts/dev-up.ps1 -Services postgres
4. podman cp backups/latest.dump web-repo-postgres:/tmp/restore.dump && pg_restore
5. podman run --rm -v deploy_artifact_data:/data ... tar xzf ...
6. pwsh scripts/dev-up.ps1
7. pwsh scripts/dev-status.ps1 && pwsh scripts/check-resources.ps1
```

---

## 8. Production Hardening Guidelines

### 8.1 Caddy Reverse Proxy & Security Headers

In production, route all external ingress through Caddy to enforce TLS 1.3 and defense-in-depth HTTP headers:

```caddyfile
converter.example.com {
    encode gzip zstd

    handle /api/* {
        reverse_proxy api:8080
    }

    handle /* {
        reverse_proxy web:5173
    }

    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
        Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:;"
    }
}
```

### 8.2 Health Probes & Telemetry Endpoints

The API and Worker services expose standardized probes for Kubernetes and Podman health monitors:

- **Liveness Probe:** `GET /health/live` (returns HTTP 200 `{"status":"alive"}`).
- **Readiness Probe:** `GET /health/ready` (evaluates PostgreSQL connection pool health, returns HTTP 200 or 503).
- **Prometheus Metrics:** `GET /metrics` (returns open-standard Prometheus text metrics for worker job throughput, failure counts, chaos recovery, and lease duration).
