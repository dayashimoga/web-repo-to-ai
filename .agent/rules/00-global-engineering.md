# Rule 00: Global Engineering Contract

1. **Single Subphase Execution:** Execute exactly ONE subphase (`Stage → Phase → Subphase`) per user authorization. After implementation, testing, documentation update, and commit, STOP and wait for explicit user approval.
2. **Zero External Paid Dependencies:** Core extraction, normalization, CDM construction, deduplication, and token optimization must execute without paid AI APIs or commercial scraping services.
3. **Hardware & Storage Budget:** Design all workloads for CPU-only execution on 16 GB RAM and keep total Podman storage strictly <= 30 GB.
4. **Permissive Licensing Only:** Verify every dependency's actual license before adoption (`MIT`, `Apache-2.0`, `BSD`, `ISC`, `PostgreSQL`, `Unicode-3.0`). Block unreviewed copyleft or proprietary dependencies.
5. **No Silent Failures:** Never swallow errors, silently truncate content, or silently skip eligible files.
6. **Append-Only Ledgers:** Never overwrite historical entries in `CHANGELOG.md`, `TODO.md`, or `IMPLEMENTATION_LEDGER.md`.
