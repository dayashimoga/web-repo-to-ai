# Third-Party License Policy & Inventory (`docs/LICENSING.md`)

**Project:** `web-repo-to-ai`  
**Repository License:** MIT License (`LICENSE`)

---

## 1. Dependency Adoption & License Governance

Before adopting any Rust crate (`Cargo.toml`) or Node package (`package.json`), its actual license, transitive dependency licenses, maintenance status, and security posture must be verified.

### 1.1 Approved Permissive Licenses
- `MIT`
- `Apache-2.0`
- `BSD-2-Clause`
- `BSD-3-Clause`
- `ISC`
- `Zlib`
- `Unicode-3.0` / `Unicode-DFS-2016`
- `PostgreSQL`
- `CC0-1.0` / `Unlicense`

### 1.2 Restricted / Flagged Licenses (Require Explicit Review & Isolation)
- Copyleft licenses (`GPL-2.0`, `GPL-3.0`, `AGPL-3.0`, `LGPL-*`, `MPL-2.0`)
- Source-available or non-commercial licenses (`SSPL`, `BSL`, `CC-BY-NC-*`)
- Proprietary or unlicensed packages

---

## 2. Third-Party Inventory & SBOM Status

| Stage | Ecosystem | Verification Tooling | Status |
| :--- | :--- | :--- | :--- |
| **Stage 0 (0.1.3)** | Governance Baseline | Policy & `cargo-deny` / SBOM workflow specification established | Baseline Recorded |
| **Stage 1** | Frontend (`apps/web`) | `npm` package license audit (SvelteKit, Svelte 5, Tailwind CSS, TypeScript — all MIT/Apache-2.0) | Scheduled in Stage 1 |
| **Stage 2** | Rust Workspace (`Cargo.toml`) | `cargo deny check licenses` + CycloneDX/SPDX SBOM generation | Scheduled in Stage 0.2 / Stage 2 |
