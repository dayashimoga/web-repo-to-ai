//! # Job Entity Schema & Lifecycle Models
//!
//! Strongly-typed Rust models for the `jobs` relational table defined in `docs/DATA_MODEL.md`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use web_repo_to_ai_core::error::CoreError;
use web_repo_to_ai_core::id::JobId;

/// Source extraction kind for a conversion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    /// Single web page extraction.
    WebSingle,
    /// Batch of independent web pages.
    WebBatch,
    /// Bounded crawl of a documentation site or web domain.
    WebCrawl,
    /// Public GitHub repository extraction.
    #[serde(rename = "github_public")]
    GitHubPublic,
    /// Private GitHub repository extraction (via GitHub App installation).
    #[serde(rename = "github_private")]
    GitHubPrivate,
}

impl JobKind {
    /// Returns the static database text identifier for this kind.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::WebSingle => "web_single",
            Self::WebBatch => "web_batch",
            Self::WebCrawl => "web_crawl",
            Self::GitHubPublic => "github_public",
            Self::GitHubPrivate => "github_private",
        }
    }

    /// Returns `true` if this job extracts from a GitHub repository.
    #[must_use]
    pub const fn is_github(&self) -> bool {
        matches!(self, Self::GitHubPublic | Self::GitHubPrivate)
    }

    /// Returns `true` if this job extracts from arbitrary web pages or crawls.
    #[must_use]
    pub const fn is_web(&self) -> bool {
        matches!(self, Self::WebSingle | Self::WebBatch | Self::WebCrawl)
    }
}

impl fmt::Display for JobKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for JobKind {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "web_single" => Ok(Self::WebSingle),
            "web_batch" => Ok(Self::WebBatch),
            "web_crawl" => Ok(Self::WebCrawl),
            "github_public" => Ok(Self::GitHubPublic),
            "github_private" => Ok(Self::GitHubPrivate),
            other => Err(CoreError::Validation {
                field: "kind".to_string(),
                reason: format!(
                    "Unknown job kind: '{other}'. Expected web_single, web_batch, web_crawl, github_public, or github_private"
                ),
            }),
        }
    }
}

/// Lifecycle status of a conversion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Enqueued and awaiting worker lease acquisition.
    Queued,
    /// Acquired by a worker instance, lease active.
    Leased,
    /// Actively processing through extraction/optimization pipeline.
    Running,
    /// Successfully finished with all requested artifacts generated.
    Completed,
    /// Partially completed with warnings/exclusions.
    Partial,
    /// Failed due to an unrecoverable error.
    Failed,
    /// Cancelled by user or administrative action.
    Cancelled,
}

impl JobStatus {
    /// Returns the database text representation for this status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased => "leased",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Returns `true` if this status is terminal (cannot transition further).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Partial | Self::Failed | Self::Cancelled
        )
    }

    /// Returns `true` if this status is active (queued, leased, or running).
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !self.is_terminal()
    }

    /// Validates whether a state transition from `self` to `target` is legally permitted.
    #[must_use]
    pub fn can_transition_to(&self, target: Self) -> bool {
        if *self == target {
            return true;
        }

        match self {
            Self::Queued => matches!(target, Self::Leased | Self::Cancelled),
            Self::Leased => matches!(
                target,
                Self::Running | Self::Queued | Self::Failed | Self::Cancelled
            ),
            Self::Running => matches!(
                target,
                Self::Completed
                    | Self::Partial
                    | Self::Failed
                    | Self::Cancelled
                    | Self::Queued
                    | Self::Leased
            ),
            Self::Completed | Self::Partial | Self::Failed | Self::Cancelled => false,
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for JobStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "leased" => Ok(Self::Leased),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "partial" => Ok(Self::Partial),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(CoreError::Validation {
                field: "status".to_string(),
                reason: format!(
                    "Unknown job status: '{other}'. Expected queued, leased, running, completed, partial, failed, or cancelled"
                ),
            }),
        }
    }
}

/// Active execution phase in the extraction and conversion pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobProgressPhase {
    /// Initial setup and parameter verification.
    Initializing,
    /// Network retrieval of target content.
    Fetching,
    /// Content parsing, DOM tree traversal, or repository extraction.
    Extracting,
    /// Deduplication, context optimization, and token budgeting.
    Optimizing,
    /// Multi-format artifact rendering and compression.
    Exporting,
    /// Terminal success state.
    Completed,
    /// Terminal failure state.
    Failed,
}

impl JobProgressPhase {
    /// Returns the database text representation for this progress phase.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initializing => "initializing",
            Self::Fetching => "fetching",
            Self::Extracting => "extracting",
            Self::Optimizing => "optimizing",
            Self::Exporting => "exporting",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for JobProgressPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for JobProgressPhase {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "initializing" => Ok(Self::Initializing),
            "fetching" => Ok(Self::Fetching),
            "extracting" => Ok(Self::Extracting),
            "optimizing" => Ok(Self::Optimizing),
            "exporting" => Ok(Self::Exporting),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(CoreError::Validation {
                field: "progress_phase".to_string(),
                reason: format!("Unknown progress phase: '{other}'"),
            }),
        }
    }
}

/// Relational record representing a conversion job in the `jobs` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobRow {
    /// Unique UUID v7 identifier for the job.
    pub id: Uuid,
    /// Source extraction category (`web_single`, `github_public`, etc.).
    pub kind: String,
    /// Active lifecycle status (`queued`, `running`, `completed`, etc.).
    pub status: String,
    /// Initial target URL or repository address.
    pub source_url: String,
    /// Job configuration payload stored as JSONB.
    pub config_json: serde_json::Value,
    /// Pipeline progress phase (`initializing`, `extracting`, etc.).
    pub progress_phase: String,
    /// Integer progress percentage (0 to 100).
    pub progress_pct: i16,
    /// Identifier of the worker instance holding the lease.
    pub leased_by: Option<String>,
    /// UTC timestamp after which the lease is considered expired.
    pub lease_expires_at: Option<DateTime<Utc>>,
    /// Number of processing attempts executed so far.
    pub attempt_count: i16,
    /// Machine-readable error code if processing failed.
    pub error_code: Option<String>,
    /// Human-readable error message if processing failed.
    pub error_message: Option<String>,
    /// Web session or user identifier owning this job.
    pub owner_session_id: Option<Uuid>,
    /// Record creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Record last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Terminal completion timestamp.
    pub completed_at: Option<DateTime<Utc>>,
}

impl JobRow {
    /// Constructs a new `JobRow` in `Queued` state with UUID v7 identifier.
    #[must_use]
    pub fn new(kind: JobKind, source_url: impl Into<String>, config: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: JobId::new().into_inner(),
            kind: kind.as_str().to_string(),
            status: JobStatus::Queued.as_str().to_string(),
            source_url: source_url.into(),
            config_json: config,
            progress_phase: JobProgressPhase::Initializing.as_str().to_string(),
            progress_pct: 0,
            leased_by: None,
            lease_expires_at: None,
            attempt_count: 0,
            error_code: None,
            error_message: None,
            owner_session_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Returns the strongly-typed `JobId` domain identifier.
    #[must_use]
    pub fn job_id(&self) -> JobId {
        JobId::from(self.id)
    }

    /// Parses and returns the typed `JobKind`.
    pub fn kind_enum(&self) -> Result<JobKind, CoreError> {
        self.kind.parse()
    }

    /// Parses and returns the typed `JobStatus`.
    pub fn status_enum(&self) -> Result<JobStatus, CoreError> {
        self.status.parse()
    }

    /// Parses and returns the typed `JobProgressPhase`.
    pub fn progress_phase_enum(&self) -> Result<JobProgressPhase, CoreError> {
        self.progress_phase.parse()
    }

    /// Attaches an owner session ID.
    #[must_use]
    pub fn with_owner_session(mut self, session_id: Uuid) -> Self {
        self.owner_session_id = Some(session_id);
        self
    }

    /// Validates table constraints and invariants on this row.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.kind_enum()?;
        self.status_enum()?;
        self.progress_phase_enum()?;

        if self.source_url.trim().is_empty() {
            return Err(CoreError::Validation {
                field: "source_url".to_string(),
                reason: "source_url cannot be empty".to_string(),
            });
        }

        if self.source_url.len() > 2048 {
            return Err(CoreError::Validation {
                field: "source_url".to_string(),
                reason: format!(
                    "source_url exceeds 2048 characters (actual: {})",
                    self.source_url.len()
                ),
            });
        }

        if !(0..=100).contains(&self.progress_pct) {
            return Err(CoreError::Validation {
                field: "progress_pct".to_string(),
                reason: format!(
                    "progress_pct must be between 0 and 100 (actual: {})",
                    self.progress_pct
                ),
            });
        }

        if self.attempt_count < 0 {
            return Err(CoreError::Validation {
                field: "attempt_count".to_string(),
                reason: format!(
                    "attempt_count cannot be negative (actual: {})",
                    self.attempt_count
                ),
            });
        }

        Ok(())
    }
}
