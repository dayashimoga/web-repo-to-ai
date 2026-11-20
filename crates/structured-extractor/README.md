# Structured Extractor Crate (`web-repo-to-ai-structured-extractor`)

Provides structured document parsing, schema extraction, and Canonical Context Document Model (`CdmDocument`) synthesis for:
- **OpenAPI 2.0 / 3.0 / 3.1 & Swagger** (JSON and YAML)
- **JSON Schema** (Draft-04, Draft-07, Draft 2020-12)
- **Generic YAML** documents and multi-document streams (`---`)
- **CSV / TSV & Tabular Data** (RFC 4180 parsing, automated delimiter sniffing for comma/tab/semicolon/pipe, header detection, column profiling and type inference, ragged row alignment, null normalization, and multi-format serializers)
- **Jupyter Notebooks (`.ipynb`)** (nbformat v4 cell parsing, execution output extraction, ANSI terminal escape stripping, heading-based hierarchy trees, and multi-format export fidelity)

## Features

- **Format Detection (`detector`):** Sniffs OpenAPI JSON/YAML, JSON Schema JSON/YAML, YAML streams, delimited tabular text (CSV/TSV/PSV), and Jupyter Notebooks (`.ipynb`).
- **Tabular Engine (`tabular`):**
  - Streaming RFC 4180 state machine parser supporting quotes, commas, escaped quotes (`""`), and multiline field content across CRLF/LF.
  - Automated delimiter sniffer (`sniff_delimiter`) evaluating candidate delimiters (comma, tab, semicolon, pipe) by column variance and consistency.
  - Automated header detection (`sniff_has_headers`) using type disparity heuristics, uniqueness checks, and semantic keyword matching.
  - Column type inference (`infer_column_type`) classifying columns into `Integer`, `Float`, `Boolean`, `DateTime`, or `String`.
  - Column statistics profiling (`ColumnStats`) computing non-empty counts, null counts, unique distinct counts, sample values, and min/max ranges.
  - Multi-format serializers (`to_markdown`, `to_html`, `to_csv`, `to_json_records`, `to_summary_markdown`).
  - Resource & DoS ceilings (`max_tabular_rows`, `max_tabular_columns`, `max_cell_bytes`, `truncate_on_row_limit`).
- **Jupyter Notebook Engine (`notebook`):**
  - High-performance nbformat v4 JSON stream parser classifying cells into `Markdown`, `Code`, and `Raw` archetypes.
  - Source line consolidation supporting both `Vec<String>` and single `String` representations.
  - Execution output extraction: `Stream` (stdout/stderr), `ExecuteResult` (with plaintext, HTML, and base64 PNG/JPEG MIME dictionaries), `DisplayData`, and `Error` tracebacks.
  - Fast, regex-free ANSI terminal escape sequence stripper removing SGR color codes and cursor movement bytes from tracebacks and output streams.
  - Automatic language tag inference from notebook metadata `kernelspec` or `language_info` (defaulting to `python`).
  - Hierarchical outline tree construction mirroring Markdown heading levels (`#`, `##`, `###`) with nested code and output cells.
  - Resource limits: `max_notebook_cells`, `max_output_bytes`, `truncate_on_cell_limit`, and output line capping (`notebook_max_output_lines`).
- **CDM Synthesis (`extractor`):** Synthesizes `CdmDocument` (`SourceType::StructuredDocument`) with summary prose, GFM pipe data tables, code snippets with syntax highlighting tags, hierarchical navigation trees, and mathematical `CompletenessReport` accounting (`discovered == extracted + excluded`).
- **High Performance:** Sub-50ms execution on 5,000+ tabular rows and 100+ notebook iterations with bit-for-bit hash determinism.
