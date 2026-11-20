//! # Format Sniffer and Detector for Structured Documents
//!
//! Inspects input bytes, filenames, and structural tokens to determine whether a document
//! represents OpenAPI (JSON or YAML), JSON Schema, generic YAML, CSV/TSV, or Jupyter Notebooks.

use crate::error::{Result, StructuredError};
use crate::models::{StructuredDocumentKind, StructuredInput};

/// Detects the format of a structured input.
pub fn detect_format(input: &StructuredInput) -> Result<StructuredDocumentKind> {
    // 1. Check filename extension hints if present
    if let Some(ref filename) = input.filename {
        let lower = filename.to_lowercase();
        if lower.ends_with(".ipynb") {
            return Ok(StructuredDocumentKind::JupyterNotebook);
        }
        if lower.ends_with(".csv")
            || lower.ends_with(".tsv")
            || lower.ends_with(".psv")
            || lower.ends_with(".tab")
        {
            return Ok(StructuredDocumentKind::Csv);
        }
    }

    let text = input.as_str()?.trim();
    if text.is_empty() {
        return Err(StructuredError::UnrecognizedFormat(
            "Input is empty".to_string(),
        ));
    }

    // 2. Try fast JSON parse
    if (text.starts_with('{') && text.ends_with('}'))
        || (text.starts_with('[') && text.ends_with(']'))
    {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(kind) = classify_json_value(&json_val) {
                return Ok(kind);
            }
        }
    }

    // 3. Try YAML parse
    if let Ok(yaml_val) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text) {
        if let Some(kind) = classify_yaml_value(&yaml_val) {
            return Ok(kind);
        }
        // If it's valid YAML mapping or sequence or contains multiple documents
        if yaml_val.is_mapping() || yaml_val.is_sequence() || text.contains("---") {
            return Ok(StructuredDocumentKind::GenericYaml);
        }
    }

    // 4. Multi-document YAML check
    if text.starts_with("---") || text.contains("\n---") {
        return Ok(StructuredDocumentKind::GenericYaml);
    }

    // 5. Delimited CSV/TSV sniff
    if is_likely_csv(text) {
        return Ok(StructuredDocumentKind::Csv);
    }

    Err(StructuredError::UnrecognizedFormat(
        "Could not detect structured format from input content".to_string(),
    ))
}

/// Classifies a parsed JSON value by inspecting signature keys.
fn classify_json_value(val: &serde_json::Value) -> Option<StructuredDocumentKind> {
    let obj = val.as_object()?;

    // OpenAPI 3.x
    if obj.contains_key("openapi") {
        return Some(StructuredDocumentKind::OpenApi);
    }

    // Swagger 2.0
    if obj.contains_key("swagger") {
        return Some(StructuredDocumentKind::OpenApi);
    }

    // Jupyter Notebook
    if obj.contains_key("cells") && obj.contains_key("nbformat") {
        return Some(StructuredDocumentKind::JupyterNotebook);
    }

    // JSON Schema
    if obj.contains_key("$schema") {
        return Some(StructuredDocumentKind::JsonSchema);
    }

    // Root schema heuristics: must have ("properties" or "$defs" or "definitions") AND "type"
    if (obj.contains_key("properties")
        || obj.contains_key("$defs")
        || obj.contains_key("definitions"))
        && (obj.contains_key("type") || obj.contains_key("title"))
    {
        return Some(StructuredDocumentKind::JsonSchema);
    }

    None
}

/// Classifies a parsed YAML value by inspecting signature keys.
fn classify_yaml_value(val: &serde_yaml_ng::Value) -> Option<StructuredDocumentKind> {
    let map = val.as_mapping()?;

    let key_matches = |target: &str| -> bool {
        map.keys().any(|k| {
            k.as_str()
                .map(|s| s.eq_ignore_ascii_case(target))
                .unwrap_or(false)
        })
    };

    if key_matches("openapi") || key_matches("swagger") {
        return Some(StructuredDocumentKind::OpenApi);
    }

    if key_matches("$schema") {
        return Some(StructuredDocumentKind::JsonSchema);
    }

    None
}

/// Heuristic check for CSV, TSV, or delimited tabular content.
fn is_likely_csv(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(5)
        .collect();

    if lines.len() < 2 {
        return false;
    }

    let candidates = [',', '\t', ';', '|'];
    for &delim in &candidates {
        let counts: Vec<usize> = lines.iter().map(|l| l.matches(delim).count()).collect();
        if counts[0] > 0 && counts.iter().all(|&c| c == counts[0]) {
            return true;
        }
    }

    false
}
