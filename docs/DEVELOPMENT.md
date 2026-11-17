# Local Development Guide (`docs/DEVELOPMENT.md`)

**Project:** `web-repo-to-ai`

---

## 1. Workstation Prerequisites & Constraints

- **Target Hardware Budget:** CPU-only execution, 16 GB system RAM, <= 30 GB Podman storage ceiling.
- **Required Tools:**
  - `git` (>= 2.40)
  - `rustc` & `cargo` (stable Rust toolchain)
  - `node` (>= 20 LTS) & `npm`
  - `podman` (>= 5.x) & `podman-compose`

---

## 2. Verified Environment Inventory (Bootstrap Baseline)

Recorded during Stage 0 Subphase 0.1.1 inspection on Windows workstation:
- **OS:** Windows (14 logical processors, 16.64 GB physical RAM)
- **Git:** `git version 2.55.0.windows.5`
- **Rust:** `rustc 1.97.1` / `cargo 1.97.1`
- **Node / NPM:** `v20.18.0` / `npm 10.8.2`
- **Podman:** `podman version 5.8.3` (`podman-machine-default` WSL VM configured with 7 CPUs, 2 GiB RAM, 30 GiB disk ceiling)
- **Podman Compose:** Installed (`podman-compose.exe`)

---

## 3. Repository Layout & Commands

- **Rust Workspace Checks (Stage 2+):**
  ```powershell
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```
- **Frontend Checks (Stage 1+):**
  ```powershell
  cd apps/web
  npm run lint
  npm run check
  npm test
  ```
- **Container & Resource Budget Management (Stage 0.3+):**
  - Inspect Podman storage and host disk usage via `scripts/check-resources.ps1`.
  - Clean unused build caches safely via `scripts/clean-caches.ps1`.

---

## 4. GitHub Repository Settings & Branch Protection Policy

The GitHub remote repository (`dayashimoga/web-repo-to-ai`) is configured with the following baseline governance rules (established in Subphase `0.2.2`):

- **Default Branch:** `main`
- **Linear History Enforced:**
  - `allow_merge_commit: false` (traditional merge commits are disabled to ensure linear Git history)
  - `allow_squash_merge: true` (PR squash merge enabled with PR title as commit message)
  - `allow_rebase_merge: true` (rebase merging permitted for atomic subphase integration)
- **Automatic Head Branch Deletion:** `delete_branch_on_merge: true` (PR feature branches are pruned automatically upon merge)
- **Branch Protection on `main`:**
  - `allow_force_pushes: false` (force pushes to `main` are strictly blocked)
  - `allow_deletions: false` (deletion of `main` is strictly blocked)
  - `required_conversation_resolution: true` (all review comments and discussions must be resolved before merging)
- **Vulnerability & Security Alerts:**
  - Dependabot vulnerability alerts: `active`
  - Dependabot automated security fixes: `active`
