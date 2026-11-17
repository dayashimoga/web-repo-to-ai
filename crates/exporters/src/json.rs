//! # JSON Exporter
//!
//! Production-grade, deterministic JSON generator transforming [`CdmDocument`]
//! into versioned Canonical CDM schema v1.0 or token-optimized structured context envelopes.

use crate::error::{ExporterError, Result};
use crate::options::{ExportMetrics, JsonExportOptions, JsonFormatFlavor};
use crate::util::CountingWriter;
use serde::{Deserialize, Serialize};
use std::io::Write;
use web_repo_to_ai_cdm::hierarchy::HierarchyNode;
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_cdm::section::CdmSection;

/// High-fidelity JSON export engine with fluent configuration.
#[derive(Debug, Clone, Default)]
pub struct JsonExporter {
    options: JsonExportOptions,
}

impl JsonExporter {
    /// Creates a new JSON exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: JsonExportOptions::default(),
        }
    }

    /// Sets the configuration options for this exporter.
    #[must_use]
    pub fn with_options(mut self, options: JsonExportOptions) -> Self {
        self.options = options;
        self
    }

    /// Exports a [`CdmDocument`] to an in-memory JSON string.
    pub fn export_to_string(&self, doc: &CdmDocument) -> Result<String> {
        export_json(doc, &self.options)
    }

    /// Exports a [`CdmDocument`] streaming directly into a writer.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        writer: &mut W,
    ) -> Result<ExportMetrics> {
        export_json_to_writer(doc, &self.options, writer)
    }
}

/// Exports a [`CdmDocument`] to a JSON string using the provided options.
pub fn export_json(doc: &CdmDocument, options: &JsonExportOptions) -> Result<String> {
    let mut buffer = Vec::with_capacity(estimate_initial_capacity(doc));
    export_json_to_writer(doc, options, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| ExporterError::Formatting(e.to_string()))
}

/// Exports a [`CdmDocument`] directly to any [`Write`] destination as JSON.
pub fn export_json_to_writer<W: Write>(
    doc: &CdmDocument,
    options: &JsonExportOptions,
    writer: &mut W,
) -> Result<ExportMetrics> {
    let mut counting = CountingWriter::new(writer);
    let mut metrics = ExportMetrics::default();

    match options.flavor {
        JsonFormatFlavor::CanonicalCdm => {
            export_canonical_cdm(doc, options, &mut counting, &mut metrics)?;
        }
        JsonFormatFlavor::StructuredContext => {
            export_structured_context(doc, options, &mut counting, &mut metrics)?;
        }
        JsonFormatFlavor::Minified => {
            export_minified(doc, options, &mut counting, &mut metrics)?;
        }
    }

    counting.flush()?;
    metrics.bytes_written = counting.bytes_written;
    metrics.lines_written = counting.lines_written;

    Ok(metrics)
}

fn estimate_initial_capacity(doc: &CdmDocument) -> usize {
    let body_bytes: usize = doc.sections.iter().map(|s| s.body.len()).sum();
    body_bytes.saturating_add(4096)
}

fn filter_sections(
    sections: &[CdmSection],
    max_token_budget: Option<usize>,
) -> (Vec<&CdmSection>, bool, usize) {
    let mut filtered = Vec::new();
    let mut cumulative_tokens: usize = 0;
    let mut truncated = false;

    for s in sections {
        let est = s.token_metrics.as_ref().map_or_else(
            || s.body.chars().count().div_ceil(4),
            |m| m.estimated_tokens,
        );

        if let Some(budget) = max_token_budget {
            if cumulative_tokens.saturating_add(est) > budget {
                truncated = true;
                break;
            }
        }
        cumulative_tokens += est;
        filtered.push(s);
    }

    (filtered, truncated, cumulative_tokens)
}

fn export_canonical_cdm<W: Write>(
    doc: &CdmDocument,
    options: &JsonExportOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let (filtered_sections, truncated, total_tokens) =
        filter_sections(&doc.sections, options.max_token_budget);

    metrics.sections_exported = filtered_sections.len();
    metrics.exclusions_exported = doc.exclusions.len();
    metrics.citations_exported = doc.citations.len();
    metrics.estimated_tokens = total_tokens;
    metrics.truncated = truncated;

    if truncated {
        // Build a pruned clone with only budgeted sections
        let mut pruned = doc.clone();
        pruned.sections = filtered_sections.into_iter().cloned().collect();
        if options.pretty {
            serde_json::to_writer_pretty(&mut *writer, &pruned)
                .map_err(|e| ExporterError::Formatting(e.to_string()))?;
        } else {
            serde_json::to_writer(&mut *writer, &pruned)
                .map_err(|e| ExporterError::Formatting(e.to_string()))?;
        }
    } else if options.pretty {
        serde_json::to_writer_pretty(&mut *writer, doc)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    } else {
        serde_json::to_writer(&mut *writer, doc)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    }

    writeln!(writer)?;
    Ok(())
}

fn export_minified<W: Write>(
    doc: &CdmDocument,
    options: &JsonExportOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let (filtered_sections, truncated, total_tokens) =
        filter_sections(&doc.sections, options.max_token_budget);

    metrics.sections_exported = filtered_sections.len();
    metrics.exclusions_exported = doc.exclusions.len();
    metrics.citations_exported = doc.citations.len();
    metrics.estimated_tokens = total_tokens;
    metrics.truncated = truncated;

    if truncated {
        let mut pruned = doc.clone();
        pruned.sections = filtered_sections.into_iter().cloned().collect();
        serde_json::to_writer(&mut *writer, &pruned)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    } else {
        serde_json::to_writer(&mut *writer, doc)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    }

    Ok(())
}

fn export_structured_context<W: Write>(
    doc: &CdmDocument,
    options: &JsonExportOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let (filtered_sections, truncated, total_tokens) =
        filter_sections(&doc.sections, options.max_token_budget);

    let structured_sections: Vec<StructuredSection> = if options.include_sections {
        filtered_sections
            .iter()
            .map(|s| StructuredSection {
                id: s.section_id.clone(),
                title: s.title.clone(),
                path: s.original_path.clone(),
                kind: s.kind.as_str().to_string(),
                language: s.language.clone(),
                lines: s.line_range.map(|r| [r.start_line, r.end_line]),
                bytes: s.byte_range.map(|b| b.byte_length()),
                estimated_tokens: s.token_metrics.as_ref().map_or_else(
                    || s.body.chars().count().div_ceil(4),
                    |m| m.estimated_tokens,
                ),
                content_hash: s.content_hash.clone(),
                content: s.body.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    let structured_citations: Vec<StructuredCitation> = if options.include_citations {
        doc.citations
            .iter()
            .map(|c| StructuredCitation {
                id: c.citation_id.clone(),
                url: c.url.clone(),
                title: c.title.clone(),
                concise: c.concise.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    let structured_exclusions: Vec<StructuredExclusion> = if options.include_exclusions {
        doc.exclusions
            .iter()
            .map(|e| StructuredExclusion {
                path: e.original_path.clone(),
                reason: e.reason.as_str().to_string(),
                pattern: e.pattern.clone(),
                size_bytes: e.byte_size,
                details: e.details.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    let hierarchy = if options.include_hierarchy {
        Some(convert_hierarchy_node(&doc.hierarchy))
    } else {
        None
    };

    let completeness = if options.include_completeness {
        Some(StructuredCompleteness {
            percentage: doc.completeness.extraction_ratio() * 100.0,
            discovered_count: doc.completeness.discovered_count,
            extracted_count: doc.completeness.extracted_count,
            excluded_count: doc.completeness.excluded_count,
            failed_count: doc.completeness.failed_count,
        })
    } else {
        None
    };

    let total_words: usize = filtered_sections
        .iter()
        .map(|s| {
            s.token_metrics
                .as_ref()
                .map_or_else(|| s.body.split_whitespace().count(), |m| m.word_count)
        })
        .sum();

    let total_chars: usize = filtered_sections
        .iter()
        .map(|s| s.body.chars().count())
        .sum();

    let envelope = StructuredContextEnvelope {
        title: if options.include_metadata {
            doc.metadata.title.clone()
        } else {
            None
        },
        canonical_url: doc.canonical_url.clone(),
        source_type: doc.source_type.as_str().to_string(),
        retrieved_at: doc.retrieved_at.to_rfc3339(),
        license: if options.include_metadata {
            doc.metadata.license.clone()
        } else {
            None
        },
        primary_language: if options.include_metadata {
            doc.metadata.primary_language.clone()
        } else {
            None
        },
        authors: if options.include_metadata {
            doc.metadata.authors.clone()
        } else {
            Vec::new()
        },
        description: if options.include_metadata {
            doc.metadata.description.clone()
        } else {
            None
        },
        metrics: StructuredMetrics {
            total_sections: filtered_sections.len(),
            total_words,
            total_chars,
            estimated_tokens: total_tokens,
        },
        completeness,
        hierarchy,
        sections: structured_sections,
        citations: structured_citations,
        exclusions: structured_exclusions,
        truncated,
    };

    metrics.sections_exported = filtered_sections.len();
    metrics.exclusions_exported = doc.exclusions.len();
    metrics.citations_exported = doc.citations.len();
    metrics.estimated_tokens = total_tokens;
    metrics.truncated = truncated;

    if options.pretty {
        serde_json::to_writer_pretty(&mut *writer, &envelope)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    } else {
        serde_json::to_writer(&mut *writer, &envelope)
            .map_err(|e| ExporterError::Formatting(e.to_string()))?;
    }

    writeln!(writer)?;
    Ok(())
}

fn convert_hierarchy_node(node: &HierarchyNode) -> StructuredHierarchyNode {
    StructuredHierarchyNode {
        name: node.name.clone(),
        path: node.path.clone(),
        kind: node.kind.as_str().to_string(),
        byte_size: node.byte_size,
        line_count: node.line_count,
        children: node.children.iter().map(convert_hierarchy_node).collect(),
    }
}

/// Token-optimized envelope containing high-signal context for AI model injection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredContextEnvelope {
    /// Document or repository title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Authoritative canonical URL.
    pub canonical_url: String,
    /// Origin source type identifier.
    pub source_type: String,
    /// Acquisition timestamp in RFC 3339 format.
    pub retrieved_at: String,
    /// Software license, if declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Primary programming or natural language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_language: Option<String>,
    /// Author names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// Description or abstract text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Volume and token consumption metrics.
    pub metrics: StructuredMetrics,
    /// Quantitative completeness accounting report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completeness: Option<StructuredCompleteness>,
    /// Document hierarchy or file directory tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<StructuredHierarchyNode>,
    /// Ordered collection of structured content sections.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<StructuredSection>,
    /// Citations and authoritative source links.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<StructuredCitation>,
    /// Auditable records of excluded files.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclusions: Vec<StructuredExclusion>,
    /// Whether the document was truncated due to a token budget ceiling.
    pub truncated: bool,
}

/// Summary volume and token telemetry for structured JSON context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredMetrics {
    /// Number of sections included.
    pub total_sections: usize,
    /// Total whitespace-delimited word count.
    pub total_words: usize,
    /// Total character count.
    pub total_chars: usize,
    /// Aggregate estimated token consumption.
    pub estimated_tokens: usize,
}

/// Completeness accounting report for structured JSON context.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredCompleteness {
    /// Extraction yield percentage in `[0.0, 100.0]`.
    pub percentage: f64,
    /// Total items discovered.
    pub discovered_count: u64,
    /// Items successfully extracted.
    pub extracted_count: u64,
    /// Items excluded with reason.
    pub excluded_count: u64,
    /// Items that failed extraction.
    pub failed_count: u64,
}

/// Hierarchical tree node for structured JSON context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredHierarchyNode {
    /// Item name (filename or directory name).
    pub name: String,
    /// Full normalized path from root.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Node kind (e.g. `file`, `directory`, `root`).
    pub kind: String,
    /// Optional byte size of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
    /// Optional line count spanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<u32>,
    /// Child nodes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<StructuredHierarchyNode>,
}

/// Individual content unit in structured JSON context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredSection {
    /// Unique section identifier.
    pub id: String,
    /// Section title or filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Relative source path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Content kind (e.g. `code_file`, `prose`, `table`).
    pub kind: String,
    /// Language identifier (e.g. `rust`, `python`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// 1-indexed line span `[start, end]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<[u32; 2]>,
    /// Byte length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// Estimated token count.
    pub estimated_tokens: usize,
    /// BLAKE3 hex hash over content.
    pub content_hash: String,
    /// Content string.
    pub content: String,
}

/// Authoritative citation in structured JSON context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredCitation {
    /// Citation identifier.
    pub id: String,
    /// Authoritative source URL.
    pub url: String,
    /// Title or label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Concise formatted citation.
    pub concise: String,
}

/// Exclusion record in structured JSON context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StructuredExclusion {
    /// Excluded file path.
    pub path: String,
    /// Primary exclusion reason category.
    pub reason: String,
    /// Glob or regex pattern matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// File size in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// Rationale details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
