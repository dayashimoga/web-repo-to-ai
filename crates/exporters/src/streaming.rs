//! # Streaming Exporters for Scalable Large Archives
//!
//! Provides bounded-memory streaming export pipelines for large repositories and crawling jobs:
//! - Multi-file streaming ZIP packages (`StreamingZipPackageWriter`)
//! - Line-delimited JSON Lines stream writer (`StreamingJsonlWriter`)
//! - Multi-document text/markdown stream writer (`StreamingTextWriter`)
//! - Bounded chunk buffering with disk-exhaustion guards and cryptographic checksums.

use std::io::{Read, Seek, Write};
use std::time::Instant;

use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256};
use web_repo_to_ai_cdm::CdmDocument;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub use zip::ZipArchive;

use crate::error::{ExporterError, Result};
use crate::json::export_json;
use crate::markdown::export_markdown;
use crate::options::{JsonExportOptions, MarkdownOptions, PlainTextOptions};
use crate::plain_text::export_plain_text;

/// Default streaming buffer capacity (64 KiB).
pub const DEFAULT_STREAMING_BUFFER_CAPACITY: usize = 64 * 1024;

/// Configuration governing streaming export execution and memory bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingExportConfig {
    /// Internal chunk buffer capacity in bytes (default 64 KiB).
    pub buffer_capacity: usize,
    /// Optional hard ceiling on total uncompressed bytes written (guards disk capacity).
    pub max_package_bytes: Option<u64>,
    /// Whether to generate an embedded `checksums.sha256` manifest upon package completion.
    pub include_checksums: bool,
    /// Whether to generate an embedded `manifest.json` package manifest upon completion.
    pub include_manifest: bool,
    /// ZIP compression method (Deflated by default).
    pub compress: bool,
}

impl Default for StreamingExportConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: DEFAULT_STREAMING_BUFFER_CAPACITY,
            max_package_bytes: None,
            include_checksums: true,
            include_manifest: true,
            compress: true,
        }
    }
}

impl StreamingExportConfig {
    /// Creates a validated streaming export configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::Formatting`] if buffer capacity is below 1 KiB.
    pub fn new(buffer_capacity: usize) -> Result<Self> {
        let config = Self {
            buffer_capacity,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Configures a maximum uncompressed package byte limit.
    #[must_use]
    pub fn with_max_package_bytes(mut self, max_bytes: u64) -> Self {
        self.max_package_bytes = Some(max_bytes);
        self
    }

    /// Configures manifest and checksum generation.
    #[must_use]
    pub fn with_manifests(mut self, include_manifest: bool, include_checksums: bool) -> Self {
        self.include_manifest = include_manifest;
        self.include_checksums = include_checksums;
        self
    }

    /// Configures whether ZIP compression is applied.
    #[must_use]
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Validates configuration parameters.
    pub fn validate(&self) -> Result<()> {
        if self.buffer_capacity < 1024 {
            return Err(ExporterError::Formatting(
                "Streaming buffer capacity must be at least 1024 bytes (1 KiB)".to_string(),
            ));
        }
        if let Some(max) = self.max_package_bytes {
            if max == 0 {
                return Err(ExporterError::Formatting(
                    "max_package_bytes must be greater than zero".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Comprehensive summary report returned upon completion of a streaming export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingExportSummary {
    /// Total number of distinct files or records streamed.
    pub total_items_written: usize,
    /// Total uncompressed payload bytes written.
    pub total_uncompressed_bytes: u64,
    /// SHA-256 cryptographic digest of the primary exported artifact or package.
    pub sha256_checksum: String,
    /// BLAKE3 cryptographic digest of the primary exported artifact or package.
    pub blake3_checksum: String,
    /// Elapsed execution wall-clock time in milliseconds.
    pub duration_ms: u64,
}

/// Metadata entry for a single file written to a streaming ZIP archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingFileEntry {
    /// Canonical relative archive path.
    pub path: String,
    /// Uncompressed byte size.
    pub byte_size: u64,
    /// Hex-encoded SHA-256 digest of the uncompressed file bytes.
    pub sha256_hex: String,
    /// Hex-encoded BLAKE3 digest of the uncompressed file bytes.
    pub blake3_hex: String,
}

/// Bounded-memory streaming ZIP package writer.
///
/// Writes files directly to an underlying `Write + Seek` destination without
/// holding the full archive payload in memory.
pub struct StreamingZipPackageWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    config: StreamingExportConfig,
    file_entries: Vec<StreamingFileEntry>,
    total_uncompressed_bytes: u64,
    start_time: Instant,
}

impl<W: Write + Seek> StreamingZipPackageWriter<W> {
    /// Initializes a new streaming ZIP package writer.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] if configuration validation fails.
    pub fn new(writer: W, config: StreamingExportConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            zip: ZipWriter::new(writer),
            config,
            file_entries: Vec::new(),
            total_uncompressed_bytes: 0,
            start_time: Instant::now(),
        })
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &StreamingExportConfig {
        &self.config
    }

    /// Returns the current total uncompressed bytes written.
    #[must_use]
    pub fn total_uncompressed_bytes(&self) -> u64 {
        self.total_uncompressed_bytes
    }

    /// Returns the number of files written so far.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.file_entries.len()
    }

    /// Sanitizes an archive relative path to prevent directory traversal vulnerabilities.
    fn sanitize_path(path: &str) -> String {
        let normalized = path.replace('\\', "/");
        let segments: Vec<&str> = normalized
            .split('/')
            .filter(|seg| !seg.is_empty() && *seg != "." && *seg != "..")
            .collect();
        if segments.is_empty() {
            "unnamed_file".to_string()
        } else {
            segments.join("/")
        }
    }

    /// Streams an individual file from a reader directly into the ZIP archive in bounded chunks.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::PackageSizeExceeded`] if `max_package_bytes` is breached.
    /// Returns [`ExporterError::Io`] on write failure.
    pub fn append_file_from_reader<R: Read>(
        &mut self,
        zip_path: &str,
        mut reader: R,
    ) -> Result<StreamingFileEntry> {
        let clean_path = Self::sanitize_path(zip_path);

        let compression = if self.config.compress {
            CompressionMethod::Deflated
        } else {
            CompressionMethod::Stored
        };

        let file_options = SimpleFileOptions::default()
            .compression_method(compression)
            .unix_permissions(0o644);

        self.zip
            .start_file(&clean_path, file_options)
            .map_err(|e| ExporterError::Io(std::io::Error::other(e)))?;

        let mut chunk_buf = vec![0u8; self.config.buffer_capacity];
        let mut file_bytes: u64 = 0;
        let mut sha256_hasher = Sha256::new();
        let mut blake3_hasher = Blake3Hasher::new();

        loop {
            let n = reader.read(&mut chunk_buf)?;
            if n == 0 {
                break;
            }

            let slice = &chunk_buf[..n];
            sha256_hasher.update(slice);
            blake3_hasher.update(slice);

            self.zip.write_all(slice)?;

            file_bytes += n as u64;
            self.total_uncompressed_bytes += n as u64;

            if let Some(max_allowed) = self.config.max_package_bytes {
                if self.total_uncompressed_bytes > max_allowed {
                    return Err(ExporterError::PackageSizeExceeded {
                        max_bytes: max_allowed,
                        actual_bytes: self.total_uncompressed_bytes,
                    });
                }
            }
        }

        let sha256_hex = format!("{:x}", sha256_hasher.finalize());
        let blake3_hex = blake3_hasher.finalize().to_hex().to_string();

        let entry = StreamingFileEntry {
            path: clean_path,
            byte_size: file_bytes,
            sha256_hex,
            blake3_hex,
        };

        self.file_entries.push(entry.clone());
        Ok(entry)
    }

    /// Appends an in-memory byte slice as a file in the archive.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] on write or capacity failure.
    pub fn append_file_bytes(
        &mut self,
        zip_path: &str,
        bytes: &[u8],
    ) -> Result<StreamingFileEntry> {
        self.append_file_from_reader(zip_path, bytes)
    }

    /// Appends a CDM document and its multi-format representations directly into the archive.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] on serialization or writing failure.
    pub fn append_cdm_document(
        &mut self,
        base_dir: &str,
        doc: &CdmDocument,
    ) -> Result<Vec<StreamingFileEntry>> {
        let prefix = Self::sanitize_path(base_dir);
        let mut entries = Vec::new();

        // 1. Markdown representation
        let md_content = export_markdown(doc, &MarkdownOptions::default())?;
        let md_entry =
            self.append_file_bytes(&format!("{prefix}/context.md"), md_content.as_bytes())?;
        entries.push(md_entry);

        // 2. Plaintext representation
        let txt_content = export_plain_text(doc, &PlainTextOptions::default())?;
        let txt_entry =
            self.append_file_bytes(&format!("{prefix}/context.txt"), txt_content.as_bytes())?;
        entries.push(txt_entry);

        // 3. JSON CDM representation
        let json_content = export_json(doc, &JsonExportOptions::default())?;
        let json_entry =
            self.append_file_bytes(&format!("{prefix}/document.json"), json_content.as_bytes())?;
        entries.push(json_entry);

        // 4. Source sections / files
        for (sec_idx, sec) in doc.sections.iter().enumerate() {
            let relative_name = if let Some(path) = &sec.original_path {
                path.clone()
            } else if let Some(title) = &sec.title {
                title.clone()
            } else {
                format!("section_{sec_idx}.txt")
            };
            let file_path = format!("{prefix}/sources/{}", Self::sanitize_path(&relative_name));
            let src_entry = self.append_file_bytes(&file_path, sec.body.as_bytes())?;
            entries.push(src_entry);
        }

        Ok(entries)
    }

    /// Finalizes the archive, optionally injecting checksums and manifest, and returns summary.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] on archive finalization failure.
    pub fn finish(mut self) -> Result<StreamingExportSummary> {
        // 1. Inject checksums.sha256 if enabled
        if self.config.include_checksums && !self.file_entries.is_empty() {
            let mut checksum_lines = String::with_capacity(self.file_entries.len() * 80);
            for entry in &self.file_entries {
                checksum_lines.push_str(&format!("{}  {}\n", entry.sha256_hex, entry.path));
            }
            self.append_file_bytes("checksums.sha256", checksum_lines.as_bytes())?;
        }

        // 2. Inject manifest.json if enabled
        if self.config.include_manifest && !self.file_entries.is_empty() {
            let manifest_json = serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1.0.0",
                "total_files": self.file_entries.len(),
                "total_uncompressed_bytes": self.total_uncompressed_bytes,
                "files": self.file_entries.iter().map(|f| serde_json::json!({
                    "path": f.path,
                    "byte_size": f.byte_size,
                    "sha256": f.sha256_hex,
                    "blake3": f.blake3_hex,
                })).collect::<Vec<_>>()
            }))?;
            self.append_file_bytes("manifest.json", manifest_json.as_bytes())?;
        }

        // 3. Finalize ZipWriter
        let _writer = self
            .zip
            .finish()
            .map_err(|e| ExporterError::Io(std::io::Error::other(e)))?;

        // 4. Compute overall package checksum deterministically from entries
        let mut archive_sha256 = Sha256::new();
        let mut archive_blake3 = Blake3Hasher::new();
        for entry in &self.file_entries {
            archive_sha256.update(entry.sha256_hex.as_bytes());
            archive_sha256.update(b":");
            archive_sha256.update(entry.path.as_bytes());
            archive_sha256.update(b"\n");

            archive_blake3.update(entry.blake3_hex.as_bytes());
            archive_blake3.update(b":");
            archive_blake3.update(entry.path.as_bytes());
            archive_blake3.update(b"\n");
        }

        let sha256_checksum = format!("{:x}", archive_sha256.finalize());
        let blake3_checksum = archive_blake3.finalize().to_hex().to_string();

        Ok(StreamingExportSummary {
            total_items_written: self.file_entries.len(),
            total_uncompressed_bytes: self.total_uncompressed_bytes,
            sha256_checksum,
            blake3_checksum,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
        })
    }
}

/// Bounded-memory streaming JSON Lines (`.jsonl`) writer.
pub struct StreamingJsonlWriter<W: Write> {
    writer: W,
    config: StreamingExportConfig,
    records_written: usize,
    total_bytes: u64,
    sha256: Sha256,
    blake3: Blake3Hasher,
    start_time: Instant,
}

impl<W: Write> StreamingJsonlWriter<W> {
    /// Constructs a new streaming JSONL writer.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] if configuration validation fails.
    pub fn new(writer: W, config: StreamingExportConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            writer,
            config,
            records_written: 0,
            total_bytes: 0,
            sha256: Sha256::new(),
            blake3: Blake3Hasher::new(),
            start_time: Instant::now(),
        })
    }

    /// Serializes and writes an individual record as a line of JSON to the stream.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::PackageSizeExceeded`] if max bytes ceiling is breached.
    /// Returns [`ExporterError::Json`] or [`ExporterError::Io`] on write failure.
    pub fn write_record<T: serde::Serialize>(&mut self, record: &T) -> Result<u64> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');

        let line_len = line.len() as u64;
        if let Some(max_allowed) = self.config.max_package_bytes {
            if self.total_bytes + line_len > max_allowed {
                return Err(ExporterError::PackageSizeExceeded {
                    max_bytes: max_allowed,
                    actual_bytes: self.total_bytes + line_len,
                });
            }
        }

        self.sha256.update(&line);
        self.blake3.update(&line);
        self.writer.write_all(&line)?;

        self.records_written += 1;
        self.total_bytes += line_len;
        Ok(line_len)
    }

    /// Flushes the underlying writer.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::Io`] on flush failure.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Finalizes the stream and returns export summary.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::Io`] on flush failure.
    pub fn finish(mut self) -> Result<StreamingExportSummary> {
        self.writer.flush()?;

        let sha256_checksum = format!("{:x}", self.sha256.finalize());
        let blake3_checksum = self.blake3.finalize().to_hex().to_string();

        Ok(StreamingExportSummary {
            total_items_written: self.records_written,
            total_uncompressed_bytes: self.total_bytes,
            sha256_checksum,
            blake3_checksum,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
        })
    }
}

/// Bounded-memory streaming text/markdown writer.
pub struct StreamingTextWriter<W: Write> {
    writer: W,
    config: StreamingExportConfig,
    sections_written: usize,
    total_bytes: u64,
    sha256: Sha256,
    blake3: Blake3Hasher,
    start_time: Instant,
}

impl<W: Write> StreamingTextWriter<W> {
    /// Constructs a new streaming text writer.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] if configuration validation fails.
    pub fn new(writer: W, config: StreamingExportConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            writer,
            config,
            sections_written: 0,
            total_bytes: 0,
            sha256: Sha256::new(),
            blake3: Blake3Hasher::new(),
            start_time: Instant::now(),
        })
    }

    /// Appends a raw chunk or section to the stream.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::PackageSizeExceeded`] if max bytes ceiling is breached.
    /// Returns [`ExporterError::Io`] on write failure.
    pub fn append_chunk(&mut self, chunk: &[u8]) -> Result<u64> {
        let chunk_len = chunk.len() as u64;
        if let Some(max_allowed) = self.config.max_package_bytes {
            if self.total_bytes + chunk_len > max_allowed {
                return Err(ExporterError::PackageSizeExceeded {
                    max_bytes: max_allowed,
                    actual_bytes: self.total_bytes + chunk_len,
                });
            }
        }

        self.sha256.update(chunk);
        self.blake3.update(chunk);
        self.writer.write_all(chunk)?;

        self.sections_written += 1;
        self.total_bytes += chunk_len;
        Ok(chunk_len)
    }

    /// Appends a formatted Markdown section with a heading.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError`] on write or capacity failure.
    pub fn append_section(&mut self, heading: &str, content: &str) -> Result<u64> {
        let section = format!("\n\n## {heading}\n\n{content}");
        self.append_chunk(section.as_bytes())
    }

    /// Finalizes the stream and returns export summary.
    ///
    /// # Errors
    ///
    /// Returns [`ExporterError::Io`] on flush failure.
    pub fn finish(mut self) -> Result<StreamingExportSummary> {
        self.writer.flush()?;

        let sha256_checksum = format!("{:x}", self.sha256.finalize());
        let blake3_checksum = self.blake3.finalize().to_hex().to_string();

        Ok(StreamingExportSummary {
            total_items_written: self.sections_written,
            total_uncompressed_bytes: self.total_bytes,
            sha256_checksum,
            blake3_checksum,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
        })
    }
}
