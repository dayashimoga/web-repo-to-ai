---
name: devops-cicd
description: >-
  Use this skill when configuring or maintaining Podman containers, Podman
  Compose, GitHub Actions CI/CD workflows, 30 GB storage budget scripts, SBOM
  generation, dependency license checks, or deployment configurations.
---

# DevOps, Containers & CI/CD Engineering Skill

## 1. Purpose
Provide reproducible containerized development, automated GitHub Actions quality/security pipelines, SBOM and checksum generation, and resource-bounded self-hosting within the strict 30 GB Podman storage ceiling.

## 2. When to Use
- Creating or updating `.github/workflows/*.yml` (CI, security, dependency review, CodeQL, build, release, container).
- Authoring `Containerfile`s, `deploy/podman-compose.yml`, or Caddy reverse-proxy configs.
- Writing resource inspection, cache cleanup, and container recovery scripts in `scripts/`.

## 3. Required Procedures
1. Enforce least-privilege `permissions:` in every GitHub Actions workflow and pin actions to verified versions.
2. Structure multi-stage `Containerfile` builds (`rust` builder -> slim runtime) and prune intermediate build layers so total Podman storage remains <= 30 GB.
3. Publish separate, directly usable artifacts (frontend build, backend binaries/containers, CycloneDX/SPDX SBOMs, `checksums.sha256`) without unnecessary nested ZIP wrappers.
4. Provide safe host/Podman inspection and cleanup scripts in `scripts/` that never touch user data or tracked repository files.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [08-deployment.md](../../rules/08-deployment.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- All workflow YAML files validate syntactically and execute real lint/test/build commands.
- Podman disk usage verified `< 30 GB` via `podman system df`.

## 6. Expected Outputs
- Executable `.github/workflows/*.yml`, `deploy/*` container configurations, `scripts/*` operational utilities, and updated `docs/DEPLOYMENT.md`.

## 7. Common Failure Modes
- **Storage Ceiling Breach:** Leaving unpruned Rust `target/` layers across multiple container builds until the 30 GB Podman VM disk fills up.
- **Over-Privileged Workflows:** Using `permissions: write-all` in GitHub Actions workflows.
- **Destructive Cleanup Scripts:** Running `podman system prune --volumes` unconditionally without warning, wiping local dev database state when only build cache pruning was requested.
