//! # Integration Tests: Jupyter Notebook ANSI Stripping and Output Handling
//!
//! Validates extraction of execution outputs (streams, execute results, display data,
//! error tracebacks), ANSI escape sequence stripping, and output filtering controls.

use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_structured_extractor::{
    parse_notebook, strip_ansi_escapes, NotebookOutputType, StructuredExtractor, StructuredInput,
    StructuredLimits, StructuredOptions,
};

const NOTEBOOK_WITH_OUTPUTS_AND_ERRORS: &str = r##"{
  "cells": [
    {
      "cell_type": "code",
      "execution_count": 1,
      "metadata": {},
      "source": "import sys\nprint('standard output')\nprint('standard error', file=sys.stderr)",
      "outputs": [
        {
          "output_type": "stream",
          "name": "stdout",
          "text": ["standard output\n"]
        },
        {
          "output_type": "stream",
          "name": "stderr",
          "text": ["standard error\n"]
        }
      ]
    },
    {
      "cell_type": "code",
      "execution_count": 2,
      "metadata": {},
      "source": "raise ValueError('Invalid numerical parameter provided')",
      "outputs": [
        {
          "output_type": "error",
          "ename": "ValueError",
          "evalue": "Invalid numerical parameter provided",
          "traceback": [
            "\u001b[0;31m---------------------------------------------------------------------------\u001b[0m",
            "\u001b[0;31mValueError\u001b[0m                                Traceback (most recent call last)",
            "\u001b[0;32m<ipython-input-2-abc123456789>\u001b[0m in \u001b[0;36m<module>\u001b[0;34m\u001b[0m\n\u001b[1;32m----> 1\u001b[0m \u001b[0;34mraise\u001b[0m \u001b[0mValueError\u001b[0m\u001b[0;34m(\u001b[0m\u001b[0;34m'Invalid numerical parameter provided'\u001b[0m\u001b[0;34m)\u001b[0m\n",
            "\u001b[0;31mValueError\u001b[0m: Invalid numerical parameter provided"
          ]
        }
      ]
    },
    {
      "cell_type": "code",
      "execution_count": 3,
      "metadata": {},
      "source": "display(plot)",
      "outputs": [
        {
          "output_type": "display_data",
          "data": {
            "text/plain": ["<Figure size 640x480 with 1 Axes>\n"],
            "image/png": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
          },
          "metadata": {}
        }
      ]
    }
  ],
  "metadata": {
    "language_info": { "name": "python", "version": "3.11" },
    "kernelspec": { "name": "python3", "display_name": "Python 3" }
  },
  "nbformat": 4,
  "nbformat_minor": 4
}"##;

#[test]
fn test_strip_ansi_escapes_edge_cases() {
    // SGR color codes
    let colored = "\u{1b}[0;31mError\u{1b}[0m and \u{1b}[1;32mSuccess\u{1b}[0m";
    assert_eq!(strip_ansi_escapes(colored), "Error and Success");

    // 256-color and 24-bit truecolor codes
    let truecolor = "\u{1b}[38;2;255;100;50mTrueColor\u{1b}[0m";
    assert_eq!(strip_ansi_escapes(truecolor), "TrueColor");

    // Clear line and cursor positioning
    let terminal_ctrl = "Line 1\u{1b}[2K\rLine 2\u{1b}[?25l";
    assert_eq!(strip_ansi_escapes(terminal_ctrl), "Line 1\rLine 2");

    // Empty and plain strings
    assert_eq!(strip_ansi_escapes(""), "");
    assert_eq!(strip_ansi_escapes("Clean"), "Clean");
}

#[test]
fn test_error_traceback_ansi_stripping_enabled() {
    let options = StructuredOptions::default().with_notebook_strip_ansi_escapes(true);
    let limits = StructuredLimits::default();
    let nb = parse_notebook(NOTEBOOK_WITH_OUTPUTS_AND_ERRORS, &options, &limits)
        .expect("parse notebook");

    let err_cell = &nb.cells[1];
    assert_eq!(err_cell.outputs.len(), 1);
    let err_output = &err_cell.outputs[0];
    assert_eq!(err_output.output_type, NotebookOutputType::Error);
    assert_eq!(err_output.ename.as_deref(), Some("ValueError"));
    assert_eq!(
        err_output.evalue.as_deref(),
        Some("Invalid numerical parameter provided")
    );

    // Verify ANSI sequences stripped from traceback
    for line in &err_output.traceback {
        assert!(
            !line.contains("\u{1b}"),
            "Traceback line should not contain ANSI escape bytes: {line}"
        );
    }
    assert!(err_output
        .traceback
        .iter()
        .any(|line| line.contains("ValueError: Invalid numerical parameter provided")));

    let plaintext = err_output.to_plaintext();
    assert!(plaintext.contains("ValueError: Invalid numerical parameter provided"));
    assert!(!plaintext.contains("\u{1b}"));
}

#[test]
fn test_error_traceback_ansi_stripping_disabled() {
    let options = StructuredOptions::default().with_notebook_strip_ansi_escapes(false);
    let limits = StructuredLimits::default();
    let nb = parse_notebook(NOTEBOOK_WITH_OUTPUTS_AND_ERRORS, &options, &limits)
        .expect("parse notebook");

    let err_cell = &nb.cells[1];
    let err_output = &err_cell.outputs[0];

    // Verify ANSI sequences preserved when disabled
    assert!(err_output
        .traceback
        .iter()
        .any(|line| line.contains("\u{1b}[0;31m")));
}

#[test]
fn test_stream_and_display_data_outputs() {
    let options = StructuredOptions::default();
    let limits = StructuredLimits::default();
    let nb = parse_notebook(NOTEBOOK_WITH_OUTPUTS_AND_ERRORS, &options, &limits).unwrap();

    // Cell 0: Stream stdout & stderr
    let stream_cell = &nb.cells[0];
    assert_eq!(stream_cell.outputs.len(), 2);
    assert_eq!(
        stream_cell.outputs[0].output_type,
        NotebookOutputType::Stream
    );
    assert_eq!(
        stream_cell.outputs[0].stream_name.as_deref(),
        Some("stdout")
    );
    assert_eq!(
        stream_cell.outputs[0].text.as_deref(),
        Some("standard output\n")
    );

    assert_eq!(
        stream_cell.outputs[1].output_type,
        NotebookOutputType::Stream
    );
    assert_eq!(
        stream_cell.outputs[1].stream_name.as_deref(),
        Some("stderr")
    );
    assert_eq!(
        stream_cell.outputs[1].text.as_deref(),
        Some("standard error\n")
    );

    // Cell 2: Display data with plain text and base64 image
    let disp_cell = &nb.cells[2];
    assert_eq!(disp_cell.outputs.len(), 1);
    let disp_out = &disp_cell.outputs[0];
    assert_eq!(disp_out.output_type, NotebookOutputType::DisplayData);
    assert!(disp_out
        .text
        .as_ref()
        .unwrap()
        .contains("<Figure size 640x480"));
    assert!(disp_out.image.is_some());
    assert!(disp_out.image.as_ref().unwrap().starts_with("iVBORw0KGgo"));
}

#[test]
fn test_notebook_exclude_outputs_toggle() {
    let extractor = StructuredExtractor::new();
    let input =
        StructuredInput::from_str(NOTEBOOK_WITH_OUTPUTS_AND_ERRORS).with_filename("script.ipynb");

    // With outputs disabled:
    let options = StructuredOptions::default().with_notebook_include_outputs(false);
    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("synthesize CDM");

    // Output sections should NOT be generated
    let output_sections: Vec<_> = doc
        .sections
        .iter()
        .filter(|s| s.attributes.get("cell_type").map(|s| s.as_str()) == Some("output"))
        .collect();
    assert_eq!(output_sections.len(), 0);

    // Code sections should still be present
    let code_sections: Vec<_> = doc
        .sections
        .iter()
        .filter(|s| s.kind == SectionKind::CodeSnippet)
        .collect();
    assert_eq!(code_sections.len(), 3);
}

#[test]
fn test_notebook_max_output_lines_capping() {
    let mut long_lines = String::new();
    for i in 1..=150 {
        long_lines.push_str(&format!("Line {i}: data processing step\n"));
    }
    let json_escaped = long_lines.replace('\n', "\\n");

    let notebook_json = format!(
        r##"{{
          "cells": [
            {{
              "cell_type": "code",
              "execution_count": 1,
              "metadata": {{}},
              "source": "print(long_output)",
              "outputs": [
                {{
                  "output_type": "stream",
                  "name": "stdout",
                  "text": "{json_escaped}"
                }}
              ]
            }}
          ],
          "metadata": {{ "kernelspec": {{ "name": "python3" }} }},
          "nbformat": 4,
          "nbformat_minor": 0
        }}"##
    );

    let options = StructuredOptions::default().with_notebook_max_output_lines(20);
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(&notebook_json).with_filename("capped.ipynb");
    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract CDM");

    let out_section = doc
        .sections
        .iter()
        .find(|s| s.attributes.get("cell_type").map(|s| s.as_str()) == Some("output"))
        .expect("found output section");

    assert!(out_section.body.contains("Line 20:"));
    assert!(!out_section.body.contains("Line 21:"));
    assert!(out_section.body.contains("... (130 lines truncated)"));
}
