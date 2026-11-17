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

### 2.1 The 30 GB Podman Ceiling
In accordance with `GLOBAL_ENGINEERING_CONTRACT.md`, `NFR-PERF-02`, and Rule 08 (`08-deployment.md`), the entire containerized development and deployment environment is bounded by a **30 GB storage ceiling**:
- Container images use multi-stage builds (`rust` builder -> `debian:bookworm-slim` or `alpine` runtime) to minimize image footprint.
- Total combined storage (images, containers, local volumes, build caches) is audited continuously.
- Default warning threshold is set at **80% (24 GB)**, triggering proactive cache pruning alerts before disk exhaustion.
- Critical threshold is set at **100% (30 GB)**, causing verification scripts to fail and alert the developer immediately.

### 2.2 Storage Inspection (`scripts/check-resources.*`)
The repository provides automated resource inspection scripts for Windows PowerShell and POSIX bash:
```powershell
# Windows PowerShell
pwsh scripts/check-resources.ps1
pwsh scripts/check-resources.ps1 -CeilingGB 30 -Json
pwsh scripts/check-resources.ps1 -Quiet
```
```bash
# Linux / macOS / CI
bash scripts/check-resources.sh
bash scripts/check-resources.sh --ceiling-gb 30 --json
bash scripts/check-resources.sh --quiet
```
Outputs report:
1. **Host CPU & Memory:** Logical cores, total RAM, and available RAM against the 16 GB baseline requirement.
2. **Host Drive Capacity:** Free and total host disk space.
3. **Podman Storage Footprint:** Exact byte counts and reclaimable space for Images, Containers, and Local Volumes via `podman system df --format json`.
4. **Budget Status:** `HEALTHY_WITHIN_BUDGET`, `WARNING_STORAGE_HIGH`, or `CRITICAL_CEILING_BREACH`.

### 2.3 Safe Cache Pruning (`scripts/clean-caches.*`)
When build caches or dangling image layers accumulate, developers can safely reclaim disk space without risk to active work:
```powershell
# Inspect reclaimable space without modifying storage (Dry Run)
pwsh scripts/clean-caches.ps1 -DryRun

# Execute safe cache pruning (prunes build caches, stopped build containers, dangling untagged layers)
pwsh scripts/clean-caches.ps1
```
```bash
# POSIX equivalent
bash scripts/clean-caches.sh --dry-run
bash scripts/clean-caches.sh
```

**Guardrails & Safety Guarantees:**
- **Persistent Volume Protection:** Persistent database volumes (e.g., `postgres_data`, `artifact_data`) are never pruned by default.
- **Source Safety:** No tracked repository files, source code, or uncommitted work are ever modified or removed.
- **Tagged Image Retention:** Named production and base images are preserved; only untagged intermediate layers and persistent build mounts are pruned.

### 2.4 Artifact Retention
- **Artifact Retention:** Automatic background TTL cleanup purges expired artifacts from local/S3 storage and `job_artifacts`.

### 2.5 Reproducible Container Builds & Architectural Audit
All container configurations are audited for reproducibility, least-privilege security, and storage compliance via dedicated verification scripts:
```powershell
# Windows PowerShell
pwsh scripts/verify-container-builds.ps1          # Full audit (specs, budget, live container healthcheck, volume persistence)
pwsh scripts/verify-container-builds.ps1 -Quick  # Static architecture and storage ceiling audit
```
```bash
# POSIX bash
bash scripts/verify-container-builds.sh
bash scripts/verify-container-builds.sh --quick
```

**Architectural Standards Enforced:**
1. **Multi-Stage Builds:** All service Containerfiles (`api`, `worker`, `web`) isolate build dependencies (Cargo, rustc, npm) from production runtime images, preventing build bloat and keeping total storage strictly below 30 GB.
2. **Minimal Slim Base Images:** Runtimes use `debian:bookworm-slim` and `node:20-slim`; database uses `postgres:16-alpine`.
3. **Unprivileged Users:** Runtime processes run as non-root users (`appuser` UID 1000, `pwuser`, `node` UID 1000). Root execution is strictly prohibited.
4. **Network & Sandbox Isolation:** Browser worker is isolated on `browser_net` with `security_opt: ["no-new-privileges:true"]`, blocked from internal PostgreSQL database communication.

---

## 3. Environment Variables Reference

All runtime configuration for `web-repo-to-ai` services is driven through environment variables defined in `deploy/.env` (templated from `deploy/.env.example`). The table below documents each variable, default values, sensitivity, and usage.

| Variable | Default Value | Service(s) | Sensitivity | Description |
| :--- | :--- | :--- | :---: | :--- |
| `POSTGRES_DB` | `webrepotodev` | `postgres`, `api`, `worker` | Low | Name of the primary PostgreSQL database instance. |
| `POSTGRES_USER` | `webrepouser` | `postgres`, `api`, `worker` | Medium | Administrative database user name. |
| `POSTGRES_PASSWORD` | `devpassword123` | `postgres`, `api`, `worker` | **High** | Database password. Must be rotated in production. |
| `POSTGRES_HOST` | `127.0.0.1` / `postgres` | `api`, `worker` | Low | Hostname or IP for PostgreSQL (`postgres` inside compose network). |
| `POSTGRES_PORT` | `5432` | `postgres`, `api`, `worker` | Low | Port exposed and connected for PostgreSQL. |
| `DATABASE_URL` | `postgresql://webrepouser:...@127.0.0.1:5432/webrepotodev` | `api`, `worker` | **High** | Complete connection string used by SQLx in Rust services. |
| `API_PORT` | `8080` | `api` | Low | Port bound by the Axum REST API server. |
| `API_HOST` | `127.0.0.1` / `0.0.0.0` | `api` | Low | Network interface bound by the Axum API server. |
| `RUST_LOG` | `info,web_repo_to_ai=debug` | `api`, `worker` | Low | Tracing filter directive controlling log verbosity. |
| `STORAGE_BACKEND` | `local` | `api`, `worker` | Low | Active artifact storage engine (`local` filesystem or `s3`). |
| `STORAGE_LOCAL_PATH` | `./data/artifacts` | `api`, `worker` | Low | Host or container mount path for exported files and artifacts. |
| `BROWSER_WORKER_URL` | `http://127.0.0.1:3000` / `http://browser-worker:3000` | `worker` | Low | Endpoint URL for the isolated Chromium rendering service. |
| `PUBLIC_API_URL` | `http://127.0.0.1:8080/api/v1` | `web` | Low | Base URL for REST API endpoints consumed by SvelteKit client code. |
| `WEB_PORT` | `5173` | `web` | Low | Port bound by the SvelteKit frontend server. |
| `WEB_HOST` | `127.0.0.1` / `0.0.0.0` | `web` | Low | Host interface bound by the SvelteKit frontend server. |
| `GITHUB_APP_ID` | *(unset / empty)* | `api`, `worker` | Medium | Optional GitHub App ID for authenticated private repository access. |
| `GITHUB_APP_PRIVATE_KEY_PATH` | *(unset / empty)* | `api`, `worker` | **High** | Path to the RSA private key PEM file for GitHub App authentication. |
| `GITHUB_APP_CLIENT_ID` | *(unset / empty)* | `api`, `web` | Medium | OAuth client ID for user authorization flow. |
| `GITHUB_APP_CLIENT_SECRET` | *(unset / empty)* | `api` | **High** | OAuth client secret for exchanging temporary auth codes. |

> [!IMPORTANT]
> Never commit `deploy/.env` or any production secret keys to version control. The `.env` file is excluded in `.gitignore` and enforced by security gates in `scripts/check-all.*`.

---

## 4. Backup, Restore & Disaster Recovery Runbook

This runbook provides deterministic procedures for safeguarding data and recovering from cold outages in **under 5 minutes**.

### 4.1 PostgreSQL Database Backup (`pg_dump`)

#### Linux / macOS / Bash:
```bash
# Create backups directory if missing
mkdir -p ./backups

# Timestamped logical backup via running container
BACKUP_FILE="./backups/postgres_$(date +%Y%m%d_%H%M%S).dump"
podman exec web-repo-postgres pg_dump -U webrepouser -d webrepotodev -F c -b -v -f /tmp/backup.dump
podman cp web-repo-postgres:/tmp/backup.dump "$BACKUP_FILE"
podman exec web-repo-postgres rm -f /tmp/backup.dump

echo "Backup created successfully: $BACKUP_FILE"
```

#### Windows PowerShell:
```powershell
# Create backups directory
if (-not (Test-Path "backups")) { New-Item -ItemType Directory -Path "backups" | Out-Null }

# Timestamped logical backup
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupFile = "backups/postgres_$timestamp.dump"
podman exec web-repo-postgres pg_dump -U webrepouser -d webrepotodev -F c -b -v -f /tmp/backup.dump
podman cp web-repo-postgres:/tmp/backup.dump $backupFile
podman exec web-repo-postgres rm -f /tmp/backup.dump

Write-Host "Backup created successfully: $backupFile" -ForegroundColor Green
```

### 4.2 PostgreSQL Database Restore (`pg_restore`)

#### Linux / macOS / Bash:
```bash
RESTORE_FILE="./backups/latest.dump"

# Copy backup archive into database container
podman cp "$RESTORE_FILE" web-repo-postgres:/tmp/restore.dump

# Execute clean restore (drops and recreates tables within webrepotodev)
podman exec web-repo-postgres pg_restore -U webrepouser -d webrepotodev --clean --if-exists -v /tmp/restore.dump
podman exec web-repo-postgres rm -f /tmp/restore.dump

echo "Restore completed successfully from $RESTORE_FILE"
```

#### Windows PowerShell:
```powershell
$restoreFile = "backups/latest.dump"

# Copy backup archive into container
podman cp $restoreFile web-repo-postgres:/tmp/restore.dump

# Execute clean restore
podman exec web-repo-postgres pg_restore -U webrepouser -d webrepotodev --clean --if-exists -v /tmp/restore.dump
podman exec web-repo-postgres rm -f /tmp/restore.dump

Write-Host "Restore completed successfully from $restoreFile" -ForegroundColor Green
```

### 4.3 Artifact Volume Backup & Restore

Artifact volume `deploy_artifact_data` holds completed CDM conversion packages, token maps, and ZIP exports:

```bash
# Backup: Archive volume into compressed tarball
podman run --rm -v deploy_artifact_data:/data:ro -v $(pwd)/backups:/backup alpine tar czf /backup/artifacts_backup.tar.gz -C /data .

# Restore: Extract tarball into fresh or existing artifact volume
podman run --rm -v deploy_artifact_data:/data -v $(pwd)/backups:/backup alpine sh -c "rm -rf /data/* && tar xzf /backup/artifacts_backup.tar.gz -C /data"
```

### 4.4 Emergency Cold Disaster Recovery (< 5 Minutes)

In the event of corrupted host filesystems, container crashes, or catastrophic workstation restarts:

```
[Outage Occurs]
       │
       ▼
1. Stop Running Stack: `pwsh scripts/dev-down.ps1`
       │
       ▼
2. Prune Corrupted Caches: `pwsh scripts/clean-caches.ps1`
       │
       ▼
3. Re-provision Database: `pwsh scripts/dev-up.ps1 -Services postgres`
       │  (Wait for healthy status, ~10s)
       ▼
4. Restore Database Dump: `podman cp backups/latest.dump web-repo-postgres:/tmp/restore.dump ...`
       │  (pg_restore execution, ~15s)
       ▼
5. Restore Artifact Volume: `podman run --rm -v deploy_artifact_data:/data ...`
       │  (tar restore, ~20s)
       ▼
6. Launch Full Stack: `pwsh scripts/dev-up.ps1`
       │  (Services start & connect, ~20s)
       ▼
7. Validate Stack Health: `pwsh scripts/dev-status.ps1` && `pwsh scripts/check-resources.ps1`
       │
       ▼
[System Fully Restored in < 5 Minutes]
```

---

## 5. Production Hardening Guidelines

For production environments operating outside local workstations, apply the following hardening standards:

### 5.1 Reverse Proxy & Automatic HTTPS (Caddy / Nginx)

Expose only an encrypted reverse proxy to public networks. Keep API, database, and browser workers on internal networks:

```caddyfile
# Example Caddyfile for web-repo-to-ai production deployment
converter.example.com {
    encode gzip zstd

    # Route frontend web requests
    handle / {
        reverse_proxy web:5173
    }

    # Route backend REST API requests
    handle /api/* {
        reverse_proxy api:8080
    }

    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
        Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:;"
    }
}
```

### 5.2 Secrets & Credential Management
1. **Zero Hardcoded Secrets:** Never configure default passwords (`devpassword123`) in production.
2. **Environment Injection:** Use secure secret management (Podman Secrets, HashiCorp Vault, AWS Secrets Manager, or Kubernetes Secrets).
3. **Encryption at Rest:** Ensure database persistent storage and object storage volumes utilize block-level encryption (LUKS on Linux, BitLocker on Windows, KMS for S3).
4. **Token Encryption Key:** Generate a 256-bit cryptographically secure random key (`openssl rand -hex 32`) for `AES-256-GCM` token encryption at rest.

### 5.3 Container Sandbox & Network Isolation
1. **Enforce Read-Only Filesystems:** Set `read_only: true` on production containers with writable ephemeral mounts only at `/tmp` (`tmpfs`).
2. **Drop All Linux Capabilities:** Add `cap_drop: ["ALL"]` and re-add only essential capabilities (e.g., `cap_add: ["NET_BIND_SERVICE"]` if binding low ports).
3. **Preserve Network Segregation:** Keep `browser_net` strictly isolated from `internal_net`. The Chromium rendering engine must never have network access to PostgreSQL or internal administrative ports.
4. **No New Privileges:** Always enforce `security_opt: ["no-new-privileges:true"]` to block setuid/setgid privilege escalation.

### 5.4 Resource Quotas & Monitoring
1. **CPU & Memory Bounds:** Set explicit `deploy.resources.limits` in production Compose manifests (e.g., `cpus: "2.0"`, `memory: 2G` per worker).
2. **Storage Ceiling Alerting:** Run `scripts/check-resources.*` via cron / systemd timer every 15 minutes to guarantee storage remains strictly within the 30 GB ceiling.
