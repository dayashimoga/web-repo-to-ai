//! # CLI Output and Status Writing Mechanics

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_exporters::{
    build_manifest, export_html, export_json, export_jsonl, export_manifest, export_markdown,
    export_plain_text, export_zip_package, generate_checksums_manifest, ChecksumEntry, HtmlOptions,
    JsonExportOptions, JsonlOptions, MarkdownOptions, PlainTextOptions, ZipPackageOptions,
};

use crate::args::OutputFormat;
use crate::error::CliError;

/// Structured JSON execution metadata envelope.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CliJsonEnvelope {
    /// Whether the execution completed successfully.
    pub success: bool,
    /// Input URL that was processed.
    pub url: String,
    /// Output artifact format.
    pub format: String,
    /// Estimated token count.
    pub tokens: usize,
    /// Output byte size.
    pub bytes: usize,
    /// Execution duration in milliseconds.
    pub duration_ms: u128,
    /// Destination output path or '-' for stdout.
    pub output_path: String,
    /// Converted text content if writing to stdout with `--json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Error message if execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Process exit code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// Handles writing artifact payloads and progress reporting with strict channel separation.
#[derive(Debug, Clone)]
pub struct OutputWriter {
    /// Verbose diagnostic output flag.
    pub verbose: bool,
    /// Quiet suppression flag.
    pub quiet: bool,
    /// JSON execution envelope flag.
    pub json: bool,
}

impl OutputWriter {
    /// Creates a new `OutputWriter`.
    #[must_use]
    pub const fn new(verbose: bool, quiet: bool, json: bool) -> Self {
        Self {
            verbose,
            quiet,
            json,
        }
    }

    /// Emits a status or progress message to `stderr` unless quiet or json is set.
    pub fn status(&self, message: &str) {
        if !self.quiet && !self.json {
            eprintln!("[web-repo-to-ai] {message}");
        }
    }

    /// Emits a verbose diagnostic message to `stderr` if verbose is set and not json.
    pub fn debug(&self, message: &str) {
        if self.verbose && !self.quiet && !self.json {
            eprintln!("[web-repo-to-ai:debug] {message}");
        }
    }

    /// Emits an error message to `stderr` unless json is active.
    pub fn error(&self, message: &str) {
        if !self.json {
            eprintln!("[web-repo-to-ai:error] {message}");
        }
    }

    /// Writes raw payload bytes to destination (stdout or file path).
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] on disk write failure or standard output failure.
    pub fn write_payload(&self, destination: &str, data: &[u8]) -> io::Result<()> {
        if destination == "-" {
            let mut stdout = io::stdout().lock();
            stdout.write_all(data)?;
            stdout.flush()?;
        } else {
            let path = Path::new(destination);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut file = File::create(path)?;
            file.write_all(data)?;
            file.flush()?;
        }
        Ok(())
    }

    /// Emits structured JSON envelope to stdout.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if JSON serialization or stdout writing fails.
    pub fn write_json_envelope(&self, envelope: &CliJsonEnvelope) -> io::Result<()> {
        let serialized = serde_json::to_string_pretty(envelope).map_err(io::Error::other)?;
        println!("{serialized}");
        Ok(())
    }
}

/// Formats and serializes a CDM document into requested output format bytes.
///
/// # Errors
///
/// Returns [`CliError`] if exporter serialization fails.
pub fn export_artifact(
    doc: &CdmDocument,
    format: OutputFormat,
) -> std::result::Result<Vec<u8>, CliError> {
    match format {
        OutputFormat::Markdown => export_markdown(doc, &MarkdownOptions::default())
            .map(String::into_bytes)
            .map_err(CliError::from),
        OutputFormat::Txt => export_plain_text(doc, &PlainTextOptions::default())
            .map(String::into_bytes)
            .map_err(CliError::from),
        OutputFormat::Json => export_json(doc, &JsonExportOptions::canonical())
            .map(String::into_bytes)
            .map_err(CliError::from),
        OutputFormat::Html => export_html(doc, &HtmlOptions::default())
            .map(|(s, _)| s.into_bytes())
            .map_err(CliError::from),
        OutputFormat::Jsonl => export_jsonl(doc, &JsonlOptions::default())
            .map(|(s, _)| s.into_bytes())
            .map_err(CliError::from),
        OutputFormat::Zip => export_zip_package(doc, &ZipPackageOptions::complete())
            .map(|(b, _)| b)
            .map_err(CliError::from),
        OutputFormat::Manifest => export_manifest(&build_manifest(doc, vec![]), true)
            .map(String::into_bytes)
            .map_err(CliError::from),
        OutputFormat::Checksums => {
            let md = export_markdown(doc, &MarkdownOptions::default()).map_err(CliError::from)?;
            let entry = ChecksumEntry::from_sha256_bytes("document.md", md.as_bytes());
            Ok(generate_checksums_manifest(&[entry]).into_bytes())
        }
    }
}
