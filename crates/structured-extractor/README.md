# Structured Extractor Crate (`web-repo-to-ai-structured-extractor`)

Provides structured document parsing, schema extraction, and Canonical Context Document Model (`CdmDocument`) synthesis for:
- **OpenAPI 2.0 / 3.0 / 3.1 & Swagger** (JSON and YAML)
- **JSON Schema** (Draft-04, Draft-07, Draft 2020-12)
- **Generic YAML** documents and multi-document streams (`---`)
- **CSV / TSV & Tabular Data** (RFC 4180 parsing, automated delimiter sniffing for comma/tab/semicolon/pipe, header detection, column profiling and type inference, ragged row alignment, null normalization, and multi-format serializers)
- **Jupyter Notebooks (`.ipynb`)** (Phase 10.3.3)

## Features

- **Format Detection (`detector`):** Sniffs OpenAPI JSON/YAML, JSON Schema JSON/YAML, YAML streams, and delimited tabular text (CSV/TSV/PSV).
- **Tabular Engine (`tabular`):**
  - Streaming RFC 4180 state machine parser supporting quotes, commas, escaped quotes (`""`), and multiline field content across CRLF/LF.
  - Automated delimiter sniffer (`sniff_delimiter`) evaluating candidate delimiters (comma, tab, semicolon, pipe) by column variance and consistency.
  - Automated header detection (`sniff_has_headers`) using type disparity heuristics, uniqueness checks, and semantic keyword matching.
  - Column type inference (`infer_column_type`) classifying columns into `Integer`, `Float`, `Boolean`, `DateTime`, or `String`.
  - Column statistics profiling (`ColumnStats`) computing non-empty counts, null counts, unique distinct counts, sample values, and min/max ranges.
  - Multi-format serializers (`to_markdown`, `to_html`, `to_csv`, `to_json_records`, `to_summary_markdown`).
  - Resource & DoS ceilings (`max_tabular_rows`, `max_tabular_columns`, `max_cell_bytes`, `truncate_on_row_limit`).
- **CDM Synthesis (`extractor`):** Synthesizes `CdmDocument` (`SourceType::StructuredDocument`) with summary prose, GFM pipe data tables, hierarchical navigation trees, and mathematical `CompletenessReport` accounting (`discovered == extracted + excluded`).
- **High Performance:** Sub-50ms execution on 5,000+ tabular rows with bit-for-bit hash determinism.
