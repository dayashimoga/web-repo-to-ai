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
