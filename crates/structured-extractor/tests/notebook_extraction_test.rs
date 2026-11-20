//! # Integration Tests: Jupyter Notebook Extraction
//!
//! Validates parsing of nbformat v4 notebooks containing Markdown, Code, and Raw cells,
//! hierarchical outline trees derived from Markdown headings, metadata extraction,
//! and CDM synthesis.

use web_repo_to_ai_cdm::hierarchy::HierarchyKind;
use web_repo_to_ai_cdm::model::{ExtractionStatus, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_cdm::validator::CdmValidator;
use web_repo_to_ai_structured_extractor::{
    detect_format, parse_notebook, NotebookCellKind, StructuredDocumentKind, StructuredExtractor,
    StructuredInput, StructuredLimits, StructuredOptions,
};

const SAMPLE_NOTEBOOK_JSON: &str = r#####"{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Data Science Workflow\n",
        "An end-to-end exploratory analysis notebook."
      ]
    },
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": "## 1. Environment Setup"
    },
    {
      "cell_type": "code",
      "execution_count": 1,
      "metadata": {},
      "source": [
        "import numpy as np\n",
        "import pandas as pd\n",
        "print('Libraries loaded successfully')"
      ],
      "outputs": [
        {
          "output_type": "stream",
          "name": "stdout",
          "text": ["Libraries loaded successfully\n"]
        }
      ]
    },
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": "## 2. Data Ingestion"
    },
    {
      "cell_type": "code",
      "execution_count": 2,
      "metadata": {},
      "source": "df = pd.DataFrame({'a': [1, 2, 3], 'b': [10.0, 20.0, 30.0]})\ndf",
      "outputs": [
        {
          "output_type": "execute_result",
          "execution_count": 2,
          "data": {
            "text/plain": ["   a     b\n0  1  10.0\n1  2  20.0\n2  3  30.0\n"],
            "text/html": ["<div><p>DataFrame (3x2)</p></div>"]
          },
          "metadata": {}
        }
      ]
    },
    {
      "cell_type": "raw",
      "metadata": {},
      "source": [
        "RAW CONFIGURATION:\n",
        "version=1.0.0\n"
      ]
    }
  ],
  "metadata": {
    "kernelspec": {
      "display_name": "Python 3 (ipykernel)",
      "language": "python",
      "name": "python3"
    },
    "language_info": {
      "name": "python",
      "version": "3.11.4"
    }
  },
  "nbformat": 4,
  "nbformat_minor": 5
}"#####;

#[test]
fn test_notebook_format_detection() {
    let input_by_name = StructuredInput::from_str("{}").with_filename("analysis.ipynb");
    assert_eq!(
        detect_format(&input_by_name).unwrap(),
        StructuredDocumentKind::JupyterNotebook
    );

    let input_by_content = StructuredInput::from_str(SAMPLE_NOTEBOOK_JSON);
    assert_eq!(
        detect_format(&input_by_content).unwrap(),
        StructuredDocumentKind::JupyterNotebook
    );
}

#[test]
fn test_notebook_parsing_domain_model() {
    let options = StructuredOptions::default();
    let limits = StructuredLimits::default();
    let nb = parse_notebook(SAMPLE_NOTEBOOK_JSON, &options, &limits).expect("parse notebook");

    assert_eq!(nb.metadata.nbformat, 4);
    assert_eq!(nb.metadata.nbformat_minor, 5);
    assert_eq!(
        nb.metadata.kernel_display_name.as_deref(),
        Some("Python 3 (ipykernel)")
    );
    assert_eq!(nb.metadata.kernel_name.as_deref(), Some("python3"));
    assert_eq!(nb.metadata.language_name.as_deref(), Some("python"));
    assert_eq!(nb.metadata.language_version.as_deref(), Some("3.11.4"));

    assert_eq!(nb.total_cells_discovered, 6);
    assert_eq!(nb.cells.len(), 6);
    assert_eq!(nb.markdown_cells_count(), 3);
    assert_eq!(nb.code_cells_count(), 2);
    assert_eq!(nb.raw_cells_count(), 1);

    // Verify cell 0: Markdown
    assert_eq!(nb.cells[0].cell_type, NotebookCellKind::Markdown);
    assert!(nb.cells[0].source.contains("# Data Science Workflow"));

    // Verify cell 2: Code with execution count 1 and stdout stream
    assert_eq!(nb.cells[2].cell_type, NotebookCellKind::Code);
    assert_eq!(nb.cells[2].execution_count, Some(1));
    assert_eq!(nb.cells[2].outputs.len(), 1);
    assert_eq!(
        nb.cells[2].outputs[0].text.as_deref(),
        Some("Libraries loaded successfully\n")
    );

    // Verify cell 4: Code with execution count 2 and execute_result HTML + plain text
    assert_eq!(nb.cells[4].cell_type, NotebookCellKind::Code);
    assert_eq!(nb.cells[4].execution_count, Some(2));
    assert_eq!(nb.cells[4].outputs.len(), 1);
    assert!(nb.cells[4].outputs[0]
        .text
        .as_ref()
        .unwrap()
        .contains("10.0"));
    assert!(nb.cells[4].outputs[0]
        .html
        .as_ref()
        .unwrap()
        .contains("DataFrame (3x2)"));

    // Verify cell 5: Raw
    assert_eq!(nb.cells[5].cell_type, NotebookCellKind::Raw);
    assert!(nb.cells[5].source.contains("RAW CONFIGURATION:"));
}

#[test]
fn test_notebook_to_markdown_formatting() {
    let options = StructuredOptions::default();
    let limits = StructuredLimits::default();
    let nb = parse_notebook(SAMPLE_NOTEBOOK_JSON, &options, &limits).unwrap();

    let summary = nb.to_summary_markdown();
    assert!(summary.contains("# Jupyter Notebook Summary"));
    assert!(summary.contains("Python 3 (ipykernel)"));
    assert!(summary.contains("**Code Cells**: 2"));
    assert!(summary.contains("**Markdown Cells**: 3"));
    assert!(summary.contains("**Raw Cells**: 1"));

    let md = nb.to_markdown(&options);
    assert!(md.contains("# Data Science Workflow"));
    assert!(md.contains("```python [1]"));
    assert!(md.contains("print('Libraries loaded successfully')"));
    assert!(md.contains("**Output (stdout):**"));
    assert!(md.contains("Libraries loaded successfully"));
    assert!(md.contains("```python [2]"));
    assert!(md.contains("**Result [2]:**"));
    assert!(md.contains("```raw"));
    assert!(md.contains("RAW CONFIGURATION:"));
}

#[test]
fn test_notebook_cdm_synthesis_and_validation() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(SAMPLE_NOTEBOOK_JSON)
        .with_filename("data_science.ipynb")
        .with_source_url("https://github.com/example/repo/notebooks/data_science.ipynb");
    let options = StructuredOptions::default();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("synthesize CDM");

    assert_eq!(doc.source_type, SourceType::StructuredDocument);
    assert_eq!(doc.extraction_status, ExtractionStatus::Completed);
    assert_eq!(doc.content_type, "application/x-ipynb+json");
    assert_eq!(doc.metadata.title.as_deref(), Some("data_science"));
    assert_eq!(doc.metadata.primary_language.as_deref(), Some("python"));

    // Section 1: Summary overview
    assert_eq!(doc.sections[0].kind, SectionKind::Prose);
    assert!(doc.sections[0]
        .title
        .as_deref()
        .unwrap()
        .contains("Summary & Overview"));

    // Verify cell sections
    let code_sections: Vec<_> = doc
        .sections
        .iter()
        .filter(|s| s.kind == SectionKind::CodeSnippet)
        .collect();
    assert_eq!(code_sections.len(), 2);
    assert_eq!(code_sections[0].language.as_deref(), Some("python"));
    assert_eq!(code_sections[1].language.as_deref(), Some("python"));

    // Output sections
    let output_sections: Vec<_> = doc
        .sections
        .iter()
        .filter(|s| s.attributes.get("cell_type").map(|s| s.as_str()) == Some("output"))
        .collect();
    assert_eq!(output_sections.len(), 2);

    // Verify mathematical completeness invariant
    let report = &doc.completeness;
    assert_eq!(report.discovered_count, 6);
    assert_eq!(report.extracted_count, 6);
    assert_eq!(report.excluded_count, 0);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.unsupported_count, 0);
    assert_eq!(
        report.discovered_count,
        report.extracted_count
            + report.excluded_count
            + report.failed_count
            + report.unsupported_count
    );

    // Validate CDM schema invariants
    let validator = CdmValidator::new();
    let val_report = validator.validate(&doc);
    assert!(
        val_report.is_valid(),
        "Validation errors: {:?}",
        val_report
            .issues
            .iter()
            .map(|i| &i.message)
            .collect::<Vec<_>>()
    );

    // Verify hierarchy tree outline
    assert_eq!(doc.hierarchy.kind, HierarchyKind::Root);
    // Under root: Summary section, and H1 heading (# Data Science Workflow)
    assert!(doc.hierarchy.children.len() >= 2);

    // Locate H1 node
    let h1_node = doc
        .hierarchy
        .children
        .iter()
        .find(|n| n.name.contains("Data Science Workflow"))
        .expect("found H1 node");
    assert_eq!(h1_node.kind, HierarchyKind::Heading);

    // Under H1 node: H2 nodes ("1. Environment Setup" and "2. Data Ingestion")
    let h2_setup = h1_node
        .children
        .iter()
        .find(|n| n.name.contains("Environment Setup"))
        .expect("found H2 setup node");
    assert_eq!(h2_setup.kind, HierarchyKind::Heading);

    // Code cell 1 should be under "1. Environment Setup"
    let code_cell_1 = h2_setup
        .children
        .iter()
        .find(|n| n.name.contains("Cell 3 [Code [1]]"))
        .expect("found code cell 1 under H2 setup");
    assert_eq!(code_cell_1.kind, HierarchyKind::Section);

    // Code cell 1 should contain its output node as child
    assert_eq!(code_cell_1.children.len(), 1);
    assert!(code_cell_1.children[0].name.contains("Cell 3 Output"));
}
