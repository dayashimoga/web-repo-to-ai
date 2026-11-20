//! # Jupyter Notebook (.ipynb) Parser and Normalizer
//!
//! Parses nbformat v4 interactive notebooks into strongly-typed cells, execution outputs,
//! ANSI-stripped tracebacks, and comprehensive metadata.

use crate::error::{Result, StructuredError};
use crate::models::{
    JupyterNotebook, NotebookCell, NotebookCellKind, NotebookMetadata, NotebookOutput,
    NotebookOutputType, StructuredLimits, StructuredOptions,
};
use std::collections::BTreeMap;

/// Strips ANSI terminal escape sequences (e.g. SGR color codes, cursor controls)
/// from the provided string using a zero-allocation/fast character scanner.
#[must_use]
pub fn strip_ansi_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '[' {
                    // CSI sequence: ESC [ ... <final_byte>
                    chars.next(); // consume '['
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                } else if next_ch == ']' {
                    // OSC sequence: ESC ] ... (BEL or ST)
                    chars.next(); // consume ']'
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' {
                            if let Some(&after) = chars.peek() {
                                if after == '\\' {
                                    chars.next();
                                    break;
                                }
                            }
                        }
                    }
                } else if ('@'..='_').contains(&next_ch) {
                    // 2-character escape sequence (e.g. ESC N, ESC O)
                    chars.next();
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}

/// Normalizes source lines or text fields from either a JSON string array or a single string.
fn extract_multiline_string(val: Option<&serde_json::Value>) -> String {
    match val {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => {
            let mut res = String::new();
            for item in arr {
                if let serde_json::Value::String(s) = item {
                    res.push_str(s);
                }
            }
            res
        }
        _ => String::new(),
    }
}

/// Parses a Jupyter Notebook from raw JSON text.
pub fn parse_notebook(
    text: &str,
    options: &StructuredOptions,
    limits: &StructuredLimits,
) -> Result<JupyterNotebook> {
    limits.validate_input(text.as_bytes())?;

    let root: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| StructuredError::InvalidNotebook(format!("Failed to parse JSON: {e}")))?;

    let root_obj = root.as_object().ok_or_else(|| {
        StructuredError::InvalidNotebook("Notebook root must be a JSON object".to_string())
    })?;

    // Parse nbformat versions
    let nbformat = root_obj
        .get("nbformat")
        .and_then(|v| v.as_u64())
        .unwrap_or(4) as u32;

    let nbformat_minor = root_obj
        .get("nbformat_minor")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // Parse metadata
    let metadata = parse_notebook_metadata(root_obj.get("metadata"), nbformat, nbformat_minor);

    // Parse cells
    let raw_cells = root_obj
        .get("cells")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            StructuredError::InvalidNotebook("Notebook missing required 'cells' array".to_string())
        })?;

    let total_cells_discovered = raw_cells.len();
    let mut cells = Vec::new();
    let mut was_truncated = false;

    for (idx, raw_cell) in raw_cells.iter().enumerate() {
        if idx >= limits.max_notebook_cells {
            if limits.truncate_on_cell_limit {
                was_truncated = true;
                break;
            }
            return Err(StructuredError::LimitExceeded(format!(
                "Notebook cell count {} exceeds maximum allowed limit of {}",
                total_cells_discovered, limits.max_notebook_cells
            )));
        }

        if let Some(cell_obj) = raw_cell.as_object() {
            let cell_type_str = cell_obj
                .get("cell_type")
                .and_then(|v| v.as_str())
                .unwrap_or("code");

            let cell_type = match cell_type_str {
                "markdown" => NotebookCellKind::Markdown,
                "raw" => NotebookCellKind::Raw,
                _ => NotebookCellKind::Code,
            };

            let id = cell_obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            let execution_count = cell_obj
                .get("execution_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            let source = extract_multiline_string(cell_obj.get("source"));

            let cell_metadata = cell_obj
                .get("metadata")
                .and_then(|v| v.as_object())
                .map(|m| m.clone().into_iter().collect::<BTreeMap<_, _>>())
                .unwrap_or_default();

            let outputs = if cell_type == NotebookCellKind::Code && options.notebook_include_outputs
            {
                cell_obj
                    .get("outputs")
                    .and_then(|v| v.as_array())
                    .map(|outs| parse_cell_outputs(outs, options, limits))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            cells.push(NotebookCell {
                index: idx,
                cell_type,
                id,
                execution_count,
                source,
                outputs,
                metadata: cell_metadata,
            });
        }
    }

    Ok(JupyterNotebook {
        metadata,
        cells,
        total_cells_discovered,
        was_truncated,
    })
}

/// Parses notebook-level metadata object.
fn parse_notebook_metadata(
    meta_val: Option<&serde_json::Value>,
    nbformat: u32,
    nbformat_minor: u32,
) -> NotebookMetadata {
    let mut meta = NotebookMetadata {
        nbformat,
        nbformat_minor,
        ..Default::default()
    };

    if let Some(obj) = meta_val.and_then(|v| v.as_object()) {
        meta.raw_metadata = obj.clone().into_iter().collect();

        // Extract kernelspec
        if let Some(kernel) = obj.get("kernelspec").and_then(|v| v.as_object()) {
            meta.kernel_name = kernel
                .get("name")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            meta.kernel_display_name = kernel
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            if meta.language_name.is_none() {
                meta.language_name = kernel
                    .get("language")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
            }
        }

        // Extract language_info
        if let Some(lang) = obj.get("language_info").and_then(|v| v.as_object()) {
            if let Some(name) = lang.get("name").and_then(|v| v.as_str()) {
                meta.language_name = Some(name.to_string());
            }
            if let Some(ver) = lang.get("version").and_then(|v| v.as_str()) {
                meta.language_version = Some(ver.to_string());
            }
        }
    }

    meta
}

/// Parses execution outputs for a code cell.
fn parse_cell_outputs(
    raw_outputs: &[serde_json::Value],
    options: &StructuredOptions,
    limits: &StructuredLimits,
) -> Vec<NotebookOutput> {
    let mut outputs = Vec::new();

    for raw_out in raw_outputs {
        if let Some(obj) = raw_out.as_object() {
            let output_type_str = obj
                .get("output_type")
                .and_then(|v| v.as_str())
                .unwrap_or("stream");

            let output_type = match output_type_str {
                "execute_result" => NotebookOutputType::ExecuteResult,
                "display_data" => NotebookOutputType::DisplayData,
                "error" => NotebookOutputType::Error,
                _ => NotebookOutputType::Stream,
            };

            let stream_name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            let execution_count = obj
                .get("execution_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            let ename = obj
                .get("ename")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            let evalue = obj
                .get("evalue")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            let mut traceback = Vec::new();
            if let Some(tb_arr) = obj.get("traceback").and_then(|v| v.as_array()) {
                for line_val in tb_arr {
                    if let Some(line) = line_val.as_str() {
                        let cleaned = if options.notebook_strip_ansi_escapes {
                            strip_ansi_escapes(line)
                        } else {
                            line.to_string()
                        };
                        traceback.push(cleaned);
                    }
                }
            }

            let mut data = BTreeMap::new();
            let mut text = None;
            let mut html = None;
            let mut image = None;

            if output_type == NotebookOutputType::Stream {
                let mut raw_text = extract_multiline_string(obj.get("text"));
                if options.notebook_strip_ansi_escapes {
                    raw_text = strip_ansi_escapes(&raw_text);
                }
                if raw_text.len() > limits.max_output_bytes {
                    raw_text.truncate(limits.max_output_bytes);
                    raw_text.push_str("\n... [Output truncated by byte limit]");
                }
                text = Some(raw_text);
            } else if let Some(data_obj) = obj.get("data").and_then(|v| v.as_object()) {
                data = data_obj.clone().into_iter().collect();

                if let Some(t_val) = data_obj.get("text/plain") {
                    let mut t = extract_multiline_string(Some(t_val));
                    if options.notebook_strip_ansi_escapes {
                        t = strip_ansi_escapes(&t);
                    }
                    if t.len() > limits.max_output_bytes {
                        t.truncate(limits.max_output_bytes);
                        t.push_str("\n... [Output truncated by byte limit]");
                    }
                    text = Some(t);
                }

                if let Some(h_val) = data_obj.get("text/html") {
                    let mut h = extract_multiline_string(Some(h_val));
                    if h.len() > limits.max_output_bytes {
                        h.truncate(limits.max_output_bytes);
                        h.push_str("\n<!-- HTML truncated by byte limit -->");
                    }
                    html = Some(h);
                }

                if let Some(img_val) = data_obj
                    .get("image/png")
                    .or_else(|| data_obj.get("image/jpeg"))
                {
                    if let Some(s) = img_val.as_str() {
                        image = Some(s.to_string());
                    }
                }
            }

            outputs.push(NotebookOutput {
                output_type,
                stream_name,
                execution_count,
                text,
                html,
                image,
                ename,
                evalue,
                traceback,
                data,
            });
        }
    }

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_escapes() {
        let input = "\u{1b}[0;31mValueError\u{1b}[0m: \u{1b}[1mInvalid argument\u{1b}[0m";
        let cleaned = strip_ansi_escapes(input);
        assert_eq!(cleaned, "ValueError: Invalid argument");

        let clean_text = "Standard clean text without escape codes.";
        assert_eq!(strip_ansi_escapes(clean_text), clean_text);

        let cursor_escape = "\u{1b}[?25hVisible cursor";
        assert_eq!(strip_ansi_escapes(cursor_escape), "Visible cursor");
    }

    #[test]
    fn test_parse_simple_notebook() {
        let json = r##"{
            "cells": [
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": ["# Analysis Notebook\n", "Introductory notes."]
                },
                {
                    "cell_type": "code",
                    "execution_count": 1,
                    "metadata": {},
                    "source": "x = 42\nprint(x)",
                    "outputs": [
                        {
                            "output_type": "stream",
                            "name": "stdout",
                            "text": ["42\n"]
                        }
                    ]
                }
            ],
            "metadata": {
                "language_info": {
                    "name": "python",
                    "version": "3.11.0"
                },
                "kernelspec": {
                    "display_name": "Python 3",
                    "name": "python3"
                }
            },
            "nbformat": 4,
            "nbformat_minor": 2
        }"##;

        let options = StructuredOptions::default();
        let limits = StructuredLimits::default();
        let nb = parse_notebook(json, &options, &limits).expect("parse notebook");

        assert_eq!(nb.cells.len(), 2);
        assert_eq!(nb.markdown_cells_count(), 1);
        assert_eq!(nb.code_cells_count(), 1);
        assert_eq!(nb.metadata.nbformat, 4);
        assert_eq!(nb.primary_language(), "python");

        let code_cell = &nb.cells[1];
        assert_eq!(code_cell.execution_count, Some(1));
        assert_eq!(code_cell.outputs.len(), 1);
        assert_eq!(code_cell.outputs[0].text.as_deref(), Some("42\n"));
    }
}
