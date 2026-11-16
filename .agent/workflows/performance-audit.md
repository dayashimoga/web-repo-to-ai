---
description: Audit runtime performance, memory bounds, streaming I/O, CPU-only execution, and the 30 GB Podman storage budget.
---

# Workflow: Performance & Resource Audit (`performance-audit.md`)

## Purpose
Verify that the system operates within the 16 GB RAM CPU-only hardware budget and <= 30 GB Podman storage ceiling while meeting throughput targets.

## Audit Procedure
1. **Container & Disk Budget Check:**
   - Run `podman system df` and host drive checks to confirm total container storage is `< 30 GB`.
2. **Tiered Acquisition Verification:**
   - Confirm static HTTP extraction is always attempted before invoking `services/browser-worker`.
3. **Memory & Streaming Verification:**
   - Verify large downloads and ZIP/JSONL exports stream in bounded memory chunks rather than buffering entire repositories in RAM.
4. **Concurrency & Timeout Verification:**
   - Confirm all worker pools, HTTP fetches, and tree-sitter tasks use bounded semaphores and explicit timeouts.
5. **Benchmark Execution:**
   - Run extraction and tokenization benchmarks and record latency/memory metrics in `docs/PERFORMANCE.md`.
