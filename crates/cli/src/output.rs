//! # CLI Output and Status Writing Mechanics

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

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
