//! # High-Performance JSON Lines Exporter (`crates/exporters/src/jsonl.rs`)
//!
//! Generates newline-delimited JSON (`.jsonl`) streams from CDM documents for:
//! - RAG vector database chunk ingestion with rich coordinate metadata
//! - Chat conversation fine-tuning datasets (`{"messages": [...]}`)
//! - Prompt / Completion fine-tuning datasets (`{"prompt": "...", "completion": "..."}`)
//! - Canonical CDM section streaming (lossless single-line JSON per section)

use std::io::Write;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection};

use crate::error::Result;
use crate::options::{ExportMetrics, JsonlFlavor, JsonlOptions};
use crate::util::CountingWriter;

/// RAG chunk retrieval record optimized for embedding models and vector databases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RagChunkRecord {
    /// Section unique identifier.
    pub id: String,
    /// Parent document unique identifier.
    pub document_id: String,
    /// Parent document title if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_title: Option<String>,
    /// Source provenance canonical URL.
    pub source_url: String,
    /// Source file path or identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Section heading or title.
    pub title: String,
    /// Section kind (e.g. Code, Markdown, Documentation).
    pub kind: String,
    /// Programming or markup language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Line range coordinates within source file `[start_line, end_line]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<[u32; 2]>,
    /// Byte range coordinates within source file `[start_byte, end_byte]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_range: Option<[u64; 2]>,
    /// Estimated token count.
    pub token_count: usize,
    /// Section content payload.
    pub content: String,
    /// Cryptographic BLAKE3 hex hash of content body.
    pub content_hash: String,
}

/// Chat message entry within a conversational fine-tuning record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Participant role: `system`, `user`, or `assistant`.
    pub role: String,
    /// Message textual content.
    pub content: String,
}

/// Chat conversation dataset record for instruction fine-tuning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FineTuningChatRecord {
    /// Chronological messages in the conversation turn.
    pub messages: Vec<ChatMessage>,
}

/// Traditional prompt/completion pair for LLM fine-tuning datasets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FineTuningPromptCompletionRecord {
    /// Input prompt or context prefix.
    pub prompt: String,
    /// Target completion content.
    pub completion: String,
}

/// High-performance JSON Lines (`.jsonl`) exporter.
#[derive(Debug, Clone, Default)]
pub struct JsonlExporter {
    options: JsonlOptions,
}

impl JsonlExporter {
    /// Creates a new JSON Lines exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a JSON Lines exporter with the specified configuration options.
    #[must_use]
    pub const fn with_options(options: JsonlOptions) -> Self {
        Self { options }
    }

    /// Exports the CDM document into an in-memory JSON Lines string.
    ///
    /// # Errors
    /// Returns `ExporterError` on rendering failure.
    pub fn export_to_string(&self, doc: &CdmDocument) -> Result<(String, ExportMetrics)> {
        export_jsonl(doc, &self.options)
    }

    /// Streams the JSON Lines export directly to any `std::io::Write` sink.
    ///
    /// # Errors
    /// Returns `ExporterError` on write failure.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        writer: &mut W,
    ) -> Result<ExportMetrics> {
        export_jsonl_to_writer(doc, &self.options, writer)
    }
}

/// Convenience function exporting a CDM document to an in-memory JSON Lines string.
///
/// # Errors
/// Returns `ExporterError` on formatting failure.
pub fn export_jsonl(doc: &CdmDocument, options: &JsonlOptions) -> Result<(String, ExportMetrics)> {
    let mut buffer = Vec::with_capacity(16 * 1024);
    let metrics = export_jsonl_to_writer(doc, options, &mut buffer)?;
    let jsonl_string = String::from_utf8(buffer)
        .map_err(|e| crate::error::ExporterError::Formatting(e.to_string()))?;
    Ok((jsonl_string, metrics))
}

/// Streams a CDM document as newline-delimited JSON directly to an `io::Write` destination.
///
/// # Errors
/// Returns `ExporterError` if serialization or writing fails.
pub fn export_jsonl_to_writer<W: Write>(
    doc: &CdmDocument,
    options: &JsonlOptions,
    writer: &mut W,
) -> Result<ExportMetrics> {
    let mut cw = CountingWriter::new(writer);
    let mut metrics = ExportMetrics::default();

    let mut cumulative_tokens = 0usize;

    for sec in &doc.sections {
        let estimated = sec.token_metrics.as_ref().map_or_else(
            || sec.body.chars().count().div_ceil(4),
            |m| m.estimated_tokens,
        );

        if let Some(budget) = options.max_token_budget {
            if cumulative_tokens.saturating_add(estimated) > budget {
                metrics.truncated = true;
                break;
            }
        }
        cumulative_tokens = cumulative_tokens.saturating_add(estimated);
        metrics.sections_exported += 1;
        metrics.estimated_tokens += estimated;

        match options.flavor {
            JsonlFlavor::RagChunks => {
                let record = build_rag_chunk(doc, sec, estimated);
                serde_json::to_writer(&mut cw, &record)?;
                writeln!(cw)?;
            }
            JsonlFlavor::FineTuningChat => {
                let record = build_chat_record(doc, sec, options.system_prompt.as_deref());
                serde_json::to_writer(&mut cw, &record)?;
                writeln!(cw)?;
            }
            JsonlFlavor::FineTuningPromptCompletion => {
                let record = build_prompt_completion_record(doc, sec);
                serde_json::to_writer(&mut cw, &record)?;
                writeln!(cw)?;
            }
            JsonlFlavor::CanonicalSections => {
                serde_json::to_writer(&mut cw, sec)?;
                writeln!(cw)?;
            }
        }
    }

    cw.flush()?;
    metrics.bytes_written = cw.bytes_written;
    metrics.lines_written = cw.lines_written;

    Ok(metrics)
}

fn section_title_or_path(sec: &CdmSection) -> &str {
    if let Some(ref t) = sec.title {
        if !t.is_empty() {
            return t.as_str();
        }
    }
    if let Some(ref p) = sec.original_path {
        return p.as_str();
    }
    "Untitled Section"
}

fn build_rag_chunk(doc: &CdmDocument, sec: &CdmSection, estimated_tokens: usize) -> RagChunkRecord {
    RagChunkRecord {
        id: sec.section_id.clone(),
        document_id: doc.document_id.to_string(),
        document_title: doc.metadata.title.clone(),
        source_url: doc.canonical_url.clone(),
        path: sec.original_path.clone(),
        title: section_title_or_path(sec).to_string(),
        kind: sec.kind.as_str().to_string(),
        language: sec.language.clone(),
        line_range: sec.line_range.map(|r| [r.start_line, r.end_line]),
        byte_range: sec.byte_range.map(|r| [r.start_byte, r.end_byte]),
        token_count: estimated_tokens,
        content: sec.body.clone(),
        content_hash: sec.content_hash.clone(),
    }
}

fn build_chat_record(
    doc: &CdmDocument,
    sec: &CdmSection,
    custom_system_prompt: Option<&str>,
) -> FineTuningChatRecord {
    let mut messages = Vec::with_capacity(3);

    let system_content = custom_system_prompt.unwrap_or(
        "You are an expert AI software engineer. Provide clear, accurate code and documentation context.",
    );
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system_content.to_string(),
    });

    let doc_title = doc
        .metadata
        .title
        .as_deref()
        .unwrap_or("repository document");
    let target = section_title_or_path(sec);
    let user_prompt = format!(
        "Show and explain the contents of '{}' from '{}'.",
        target, doc_title
    );
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_prompt,
    });

    messages.push(ChatMessage {
        role: "assistant".to_string(),
        content: sec.body.clone(),
    });

    FineTuningChatRecord { messages }
}

fn build_prompt_completion_record(
    doc: &CdmDocument,
    sec: &CdmSection,
) -> FineTuningPromptCompletionRecord {
    let doc_title = doc.metadata.title.as_deref().unwrap_or("document");
    let path_line = sec
        .original_path
        .as_deref()
        .map(|p| format!("\nPath: {p}"))
        .unwrap_or_default();
    let lang_line = sec
        .language
        .as_deref()
        .map(|l| format!("\nLanguage: {l}"))
        .unwrap_or_default();
    let prompt = format!(
        "Document: {}\nSource: {}{}{}\nSection: {}\n\n### Content:\n",
        doc_title,
        doc.canonical_url,
        path_line,
        lang_line,
        section_title_or_path(sec)
    );

    FineTuningPromptCompletionRecord {
        prompt,
        completion: sec.body.clone(),
    }
}
