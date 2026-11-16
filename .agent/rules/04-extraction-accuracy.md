# Rule 04: Extraction Accuracy & Completeness

1. **Exact Code Preservation:** Never alter indentation, syntax, or character content of extracted source files or `<pre><code>` blocks in lossless mode.
2. **Structural Fidelity:** Preserve document heading hierarchy (`h1`–`h6`), tables, lists, and canonical links when converting webpages.
3. **Full Provenance Attribution:** Record canonical URL, relative file path, Git commit SHA/revision, line ranges, and `BLAKE3` content hash for every extracted section.
4. **Repository Completeness Gate:** Enforce $\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported}$. Never mark a repository job `COMPLETED` if any eligible file is unaccounted for.
5. **Honest Token Metrics:** Distinguish exact tokenizer counts from heuristic approximations; never label an estimate as exact.
