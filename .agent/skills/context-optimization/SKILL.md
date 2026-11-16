---
name: context-optimization
description: >-
  Use this skill when implementing structural normalization, exact/semantic
  deduplication, tokenizer adapters, context-budget prioritization, deterministic
  chunking, or multi-format exporters in crates/normalizer, crates/deduplication,
  crates/tokenizer, crates/context-optimizer, crates/chunking, and crates/exporters.
---

# AI Context & Token Optimization Engineering Skill

## 1. Purpose
Transform `CdmDocument` instances into high-signal, token-efficient, provenance-preserving context packages tailored to target AI models and specialized engineering profiles without requiring external LLM API calls.

## 2. When to Use
- Implementing structural cleanup (`crates/normalizer`) or exact/near-duplicate removal (`crates/deduplication`).
- Implementing model-specific BPE tokenizers or heuristic estimators (`crates/tokenizer`).
- Implementing context profiles (`lossless`, `compact`, `general-ai`, `code-review`, `security-review`, `debugging`, `documentation`, `rag`) and budget prioritization (`crates/context-optimizer`).
- Implementing AST/heading boundary chunking (`crates/chunking`) and exporters (`crates/exporters`).

## 3. Required Procedures
1. Maintain a strict separation between **lossless** extraction/normalization and **lossy** budget truncation or summarization.
2. Compute exact `BLAKE3` hashes on every section before and after normalization to track transformations.
3. Explicitly label every token count as either `Exact` (backed by the model's actual BPE vocabulary) or `Estimated` (with explicit uncertainty percentage).
4. When enforcing a maximum token budget, record every omitted section in `CdmDocument::exclusions` with `Reason::TokenBudgetExceeded` and its priority score.
5. Split chunks along semantic boundaries (AST declarations via `tree-sitter` or Markdown heading levels) rather than mid-line character offsets.

## 4. Engineering Rules
- Enforces [00-global-engineering.md](../../rules/00-global-engineering.md)
- Enforces [02-rust.md](../../rules/02-rust.md)
- Enforces [04-extraction-accuracy.md](../../rules/04-extraction-accuracy.md)
- Enforces [09-no-mock-no-placeholder.md](../../rules/09-no-mock-no-placeholder.md)

## 5. Validation Requirements
- Deterministic golden comparison tests verifying that identical inputs + settings always produce byte-identical outputs and chunks.
- Preservation metric tests verifying 0% code syntax corruption across all profiles.

## 6. Expected Outputs
- Optimized `CdmDocument`, token telemetry (`original_tokens`, `optimized_tokens`, `budget_utilization`), deterministic chunks, and verified Markdown/TXT/JSON/HTML/JSONL/ZIP exports.

## 7. Common Failure Modes
- **Dishonest Token Counts:** Labeling a `chars / 4` heuristic as an exact token count for a model.
- **Silent Budget Drops:** Dropping low-priority files to fit a 32k token budget without listing those dropped files in the exclusion report.
- **Mid-Function Chunk Splits:** Splitting source code chunks in the middle of a function body without preserving file/symbol headers.
