//! # Sandboxed Tesseract CLI OCR Adapter
//!
//! Executes Tesseract as an isolated child process with process-kill-on-drop,
//! RAII temporary file management, timeout enforcement, and TSV output parsing.

use crate::adapter::OcrAdapter;
use crate::error::{OcrError, Result};
use crate::models::{OcrEngineInfo, OcrInput, OcrOptions, OcrResult};
use crate::tsv::parse_tesseract_tsv;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, warn};

/// RAII guard ensuring temporary files are pruned upon completion or cancellation.
struct TempFileGuard {
    path: PathBuf,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(err) = std::fs::remove_file(&self.path) {
                warn!(path = ?self.path, error = %err, "Failed to remove temporary OCR file");
            }
        }
    }
}

/// Tesseract CLI engine adapter.
#[derive(Debug, Clone)]
pub struct TesseractAdapter {
    binary_path: PathBuf,
    default_options: OcrOptions,
}

impl Default for TesseractAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl TesseractAdapter {
    /// Creates a new `TesseractAdapter` discovering the binary path from environment or PATH.
    #[must_use]
    pub fn new() -> Self {
        let binary_path = Self::default_binary_path();
        Self {
            binary_path,
            default_options: OcrOptions::default(),
        }
    }

    /// Creates an adapter configured with an explicit path to the Tesseract binary.
    #[must_use]
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = path.into();
        self
    }

    /// Overrides default extraction options.
    #[must_use]
    pub fn with_default_options(mut self, options: OcrOptions) -> Self {
        self.default_options = options;
        self
    }

    /// Resolves the default Tesseract binary path.
    #[must_use]
    pub fn default_binary_path() -> PathBuf {
        if let Ok(env_path) = std::env::var("TESSERACT_PATH") {
            if !env_path.trim().is_empty() {
                return PathBuf::from(env_path.trim());
            }
        }
        PathBuf::from("tesseract")
    }

    /// Returns the configured binary path.
    #[must_use]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// Direct parser helper for TSV output strings using current engine info.
    pub async fn parse_tsv(&self, tsv: &str, duration_ms: u64) -> Result<OcrResult> {
        let info = self.engine_info().await.unwrap_or_else(|_| OcrEngineInfo {
            name: "Tesseract OCR".to_string(),
            version: "unknown".to_string(),
            available: false,
            path: Some(self.binary_path.to_string_lossy().to_string()),
            supported_languages: Vec::new(),
        });
        parse_tesseract_tsv(tsv, duration_ms, info)
    }

    /// Infers file extension from image magic bytes, filename, or mime type.
    fn infer_extension(input: &OcrInput) -> &'static str {
        let bytes = &input.bytes;
        if bytes.len() >= 8 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return "png";
        }
        if bytes.len() >= 3 && bytes.starts_with(b"\xFF\xD8\xFF") {
            return "jpg";
        }
        if bytes.len() >= 4 && (bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*")) {
            return "tiff";
        }
        if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            return "webp";
        }
        if bytes.len() >= 2 && bytes.starts_with(b"BM") {
            return "bmp";
        }

        if let Some(ref filename) = input.filename {
            if let Some(ext) = Path::new(filename).extension().and_then(|s| s.to_str()) {
                let lower = ext.to_ascii_lowercase();
                match lower.as_str() {
                    "png" => return "png",
                    "jpg" | "jpeg" => return "jpg",
                    "tiff" | "tif" => return "tiff",
                    "webp" => return "webp",
                    "bmp" => return "bmp",
                    _ => {}
                }
            }
        }

        if let Some(ref mime) = input.mime_type {
            match mime.to_ascii_lowercase().as_str() {
                "image/png" => return "png",
                "image/jpeg" => return "jpg",
                "image/tiff" => return "tiff",
                "image/webp" => return "webp",
                "image/bmp" => return "bmp",
                _ => {}
            }
        }

        "png"
    }
}

#[async_trait]
impl OcrAdapter for TesseractAdapter {
    async fn is_available(&self) -> bool {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("--version");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        match cmd.output().await {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    async fn engine_info(&self) -> Result<OcrEngineInfo> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("--version");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let output = match cmd.output().await {
            Ok(out) => out,
            Err(_err) => {
                return Ok(OcrEngineInfo {
                    name: "Tesseract OCR".to_string(),
                    version: "unknown".to_string(),
                    available: false,
                    path: Some(self.binary_path.to_string_lossy().to_string()),
                    supported_languages: Vec::new(),
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next().unwrap_or("tesseract unknown");
        let version = first_line
            .strip_prefix("tesseract")
            .map(str::trim)
            .unwrap_or(first_line)
            .to_string();

        // Query supported languages
        let mut lang_cmd = Command::new(&self.binary_path);
        lang_cmd.arg("--list-langs");
        lang_cmd.stdout(Stdio::piped());
        lang_cmd.stderr(Stdio::piped());
        lang_cmd.kill_on_drop(true);

        let mut supported_languages = Vec::new();
        if let Ok(lang_out) = lang_cmd.output().await {
            if lang_out.status.success() {
                let lang_str = String::from_utf8_lossy(&lang_out.stdout);
                for line in lang_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with("List of available languages")
                        && !trimmed.contains(':')
                    {
                        supported_languages.push(trimmed.to_string());
                    }
                }
            }
        }

        Ok(OcrEngineInfo {
            name: "Tesseract OCR".to_string(),
            version,
            available: output.status.success(),
            path: Some(self.binary_path.to_string_lossy().to_string()),
            supported_languages,
        })
    }

    async fn extract_text(&self, input: &OcrInput, options: &OcrOptions) -> Result<OcrResult> {
        // Enforce input size limits
        options.limits.validate_input(&input.bytes)?;

        // Verify binary presence
        let engine_info = self.engine_info().await?;
        if !engine_info.available {
            return Err(OcrError::TesseractNotFound {
                path: self.binary_path.to_string_lossy().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Tesseract binary not available or not executable",
                ),
            });
        }

        let extension = Self::infer_extension(input);
        let temp_filename = format!(
            "ocr_{}_{}.{}",
            uuid::Uuid::new_v4(),
            std::process::id(),
            extension
        );
        let temp_path = std::env::temp_dir().join(temp_filename);

        // Write input bytes to temporary file with RAII cleanup guard
        {
            let mut file = tokio::fs::File::create(&temp_path)
                .await
                .map_err(OcrError::Io)?;
            file.write_all(&input.bytes).await.map_err(OcrError::Io)?;
            file.flush().await.map_err(OcrError::Io)?;
        }
        let _guard = TempFileGuard::new(temp_path.clone());

        let start_time = Instant::now();

        // Build command with isolation
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg(&temp_path);
        cmd.arg("stdout"); // Stream output to stdout
        cmd.arg("-l").arg(&options.language);
        cmd.arg("--psm").arg(options.psm.as_flag());
        cmd.arg("--oem").arg(options.oem.as_flag());

        if let Some(ref tessdata) = options.tessdata_dir {
            cmd.arg("--tessdata-dir").arg(tessdata);
        }

        for extra in &options.extra_args {
            cmd.arg(extra);
        }

        // Configuration file for TSV output
        cmd.arg("tsv");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        debug!(
            binary = ?self.binary_path,
            language = %options.language,
            psm = %options.psm.as_flag(),
            "Spawning sandboxed Tesseract OCR subprocess"
        );

        // Execute with timeout enforcement
        let timeout_duration = options.limits.timeout;
        let output_result = tokio::time::timeout(timeout_duration, cmd.output()).await;

        let output = match output_result {
            Ok(res) => res.map_err(OcrError::Io)?,
            Err(_) => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                let limit_ms = timeout_duration.as_millis() as u64;
                return Err(OcrError::ExecutionTimeout {
                    elapsed_ms,
                    limit_ms,
                });
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(OcrError::ProcessFailed {
                exit_code: output.status.code(),
                stderr,
            });
        }

        let tsv_stdout = String::from_utf8_lossy(&output.stdout);
        let ocr_result = parse_tesseract_tsv(&tsv_stdout, duration_ms, engine_info)?;

        // Validate extracted word count against limits
        options.limits.validate_word_count(ocr_result.total_words)?;

        Ok(ocr_result)
    }
}
