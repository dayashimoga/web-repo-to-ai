# Rule 08: Containerization, CI/CD & Resource Budget

1. **30 GB Podman Ceiling:** Keep total Podman images, build caches, and volumes strictly within 30 GB. Use slim multi-stage container builds.
2. **No Host Pollution:** Prefer containerized services (`postgres`, `browser-worker`) over host-wide software installations.
3. **Least-Privilege CI Workflows:** Pin GitHub Actions versions, declare minimal `permissions:` blocks, and separate frontend, backend, container, SBOM, and checksum artifacts without unnecessary nested ZIPs.
4. **Safe Cleanup Scripts:** Cleanup scripts must only prune build caches and expired temporary artifacts—never user data or tracked project files.
