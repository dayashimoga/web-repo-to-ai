# web-repo-to-ai-ocr-extractor

Isolated Optical Character Recognition (OCR) adapter and stream parser for `web-repo-to-ai`.

## Capabilities

- **Isolated Process Execution:** Sandboxed, async process spawning (`tokio::process::Command`) with `kill_on_drop(true)` and strict timeout controls.
- **Tesseract TSV Parsing:** High-fidelity parser extracting word tokens, bounding boxes, confidence scores ($0.0 \dots 1.0$), line numbers, and paragraph hierarchies.
- **DoS & Resource Safety:** Enforces maximum image byte limits, dimension constraints, execution timeouts, and maximum word counts.
- **Canonical Context Document Model (CDM) Integration:** Transforms structured OCR results into `CdmDocument` with `SourceType::DocumentOcr`, `SectionKind::Prose`, line ranges, confidence metadata, and completeness metrics.
- **Deterministic Pipeline Interoperability:** Output documents seamlessly integrate with downstream tokenizers, context optimizers, and multi-format exporters (Markdown, JSONL, XML).
