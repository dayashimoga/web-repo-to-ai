//! # Job Artifact Entity Schema
//!
//! Strongly-typed Rust models for the `job_artifacts` relational table defined in `docs/DATA_MODEL.md`.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::{ArtifactId, JobId};

/// Format identifier for stored conversion artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormat {
    /// Canonical Context Document Model intermediate JSON representation.
    CdmJson,
    /// Cleaned, structured Markdown document.
    Markdown,
    /// Plain unformatted text.
    Txt,
    /// Structured JSON representation.
    Json,
    /// Standalone HTML rendering.
    Html,
    /// Line-delimited JSON for streaming chunk pipelines.
    Jsonl,
    /// Compressed ZIP archive containing complete extraction tree.
    Zip,
    /// Checksums and provenance manifest JSON.
    Manifest,
    /// Completeness, exclusion, and token quality audit report.
    QualityReport,
}

impl ArtifactFormat {
    /// Returns the database text representation for this format.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CdmJson => "cdm_json",
            Self::Markdown => "markdown",
            Self::Txt => "txt",
            Self::Json => "json",
            Self::Html => "html",
            Self::Jsonl => "jsonl",
            Self::Zip => "zip",
            Self::Manifest => "manifest",
            Self::QualityReport => "quality_report",
        }
    }

    /// Standard MIME content-type string for HTTP downloads.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::CdmJson => "application/vnd.antigravity.cdm+json",
            Self::Markdown => "text/markdown; charset=utf-8",
            Self::Txt => "text/plain; charset=utf-8",
            Self::Json => "application/json",
            Self::Html => "text/html; charset=utf-8",
            Self::Jsonl => "application/x-ndjson",
            Self::Zip => "application/zip",
            Self::Manifest | Self::QualityReport => "application/json",
        }
    }

    /// Default file extension for exported files.
    #[must_use]
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::CdmJson => "cdm.json",
            Self::Markdown => "md",
            Self::Txt => "txt",
            Self::Json => "json",
            Self::Html => "html",
            Self::Jsonl => "jsonl",
            Self::Zip => "zip",
            Self::Manifest => "manifest.json",
            Self::QualityReport => "quality.json",
        }
    }
}

impl fmt::Display for ArtifactFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ArtifactFormat {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cdm_json" | "cdm" => Ok(Self::CdmJson),
            "markdown" | "md" => Ok(Self::Markdown),
            "txt" | "text" => Ok(Self::Txt),
            "json" => Ok(Self::Json),
            "html" | "htm" => Ok(Self::Html),
            "jsonl" | "ndjson" => Ok(Self::Jsonl),
            "zip" => Ok(Self::Zip),
            "manifest" => Ok(Self::Manifest),
            "quality_report" | "quality" => Ok(Self::QualityReport),
            other => Err(CoreError::Validation {
                field: "format".to_string(),
                reason: format!(
                    "Unknown artifact format: '{other}'. Expected cdm_json, markdown, txt, json, html, jsonl, zip, manifest, or quality_report"
                ),
            }),
        }
    }
}

/// Relational record representing an exported artifact in the `job_artifacts` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobArtifactRow {
    /// Unique UUID v7 identifier for the artifact record.
    pub id: Uuid,
    /// Reference to parent `jobs.id`.
    pub job_id: Uuid,
    /// Export format identifier (`markdown`, `zip`, etc.).
    pub format: String,
    /// Relative storage path or object store key.
    pub storage_key: String,
    /// Exact file payload size in bytes.
    pub byte_size: i64,
    /// 64-character lowercase hexadecimal SHA-256 digest.
    pub sha256_hex: String,
    /// 64-character lowercase hexadecimal BLAKE3 digest.
    pub blake3_hex: String,
    /// Expiration deadline for automatic cache eviction.
    pub expires_at: DateTime<Utc>,
    /// Record creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl JobArtifactRow {
    /// Constructs a new artifact record with UUID v7 identifier and configurable TTL duration.
    #[must_use]
    pub fn new(
        job_id: Uuid,
        format: ArtifactFormat,
        storage_key: impl Into<String>,
        byte_size: i64,
        sha256_hex: impl Into<String>,
        blake3_hex: impl Into<String>,
        ttl_duration: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ArtifactId::new().into_inner(),
            job_id,
            format: format.as_str().to_string(),
            storage_key: storage_key.into(),
            byte_size,
            sha256_hex: sha256_hex.into().to_ascii_lowercase(),
            blake3_hex: blake3_hex.into().to_ascii_lowercase(),
            expires_at: now + ttl_duration,
            created_at: now,
        }
    }

    /// Returns the strongly-typed `ArtifactId` domain identifier.
    #[must_use]
    pub fn artifact_id(&self) -> ArtifactId {
        ArtifactId::from(self.id)
    }

    /// Returns the strongly-typed `JobId` domain identifier of the parent job.
    #[must_use]
    pub fn job_id(&self) -> JobId {
        JobId::from(self.job_id)
    }

    /// Parses and returns the typed `ArtifactFormat`.
    pub fn format_enum(&self) -> Result<ArtifactFormat, CoreError> {
        self.format.parse()
    }

    /// Validates table constraints and cryptographic hash lengths.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.format_enum()?;

        if self.storage_key.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "storage_key".to_string(),
                reason: "storage_key cannot be empty".to_string(),
            });
        }

        if self.byte_size < 0 {
            return Err(CoreError::Validation {
                field: "byte_size".to_string(),
                reason: format!("byte_size cannot be negative (actual: {})", self.byte_size),
            });
        }

        if self.sha256_hex.len() != 64 || !self.sha256_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "sha256_hex".to_string(),
                reason: format!(
                    "sha256_hex must be exactly 64 hexadecimal characters (actual: '{}')",
                    self.sha256_hex
                ),
            });
        }

        if self.blake3_hex.len() != 64 || !self.blake3_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "blake3_hex".to_string(),
                reason: format!(
                    "blake3_hex must be exactly 64 hexadecimal characters (actual: '{}')",
                    self.blake3_hex
                ),
            });
        }

        Ok(())
    }
}
