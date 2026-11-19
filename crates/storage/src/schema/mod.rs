//! # Relational Database Schema & Entities
//!
//! Provides production PostgreSQL schema DDL definitions, table constraints,
//! and strongly-typed Rust entity models for `jobs`, `job_artifacts`,
//! `github_installations`, and `content_cache_index`.

pub mod artifacts;
pub mod cache;
pub mod github;
pub mod jobs;

pub use artifacts::*;
pub use cache::*;
pub use github::*;
pub use jobs::*;

/// Embedded production-grade PostgreSQL 16+ DDL script.
pub const POSTGRES_SCHEMA_DDL: &str = include_str!("ddl.sql");

/// Expected relational tables defined in the schema.
pub const EXPECTED_TABLES: &[&str] = &[
    "jobs",
    "job_artifacts",
    "github_installations",
    "content_cache_index",
    "job_state_transitions",
    "batch_jobs",
    "batch_items",
];

/// Expected indexes defined in the schema.
pub const EXPECTED_INDEXES: &[&str] = &[
    "idx_jobs_status_lease",
    "idx_jobs_owner_session",
    "idx_jobs_created_at",
    "idx_job_artifacts_job_id",
    "idx_job_artifacts_expires_at",
    "uq_job_artifacts_job_format",
    "uq_github_installations_owner_gh_id",
    "idx_github_installations_owner",
    "idx_content_cache_last_accessed",
    "idx_job_state_transitions_job_id",
    "idx_batch_jobs_owner_session",
    "idx_batch_jobs_created_at",
    "idx_batch_items_job_id",
];

/// Splits raw SQL DDL script into discrete executable SQL statements,
/// stripping leading/trailing whitespace and pure comment lines.
#[must_use]
pub fn split_ddl_statements(ddl: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_function_body = false;

    for line in ddl.lines() {
        let trimmed = line.trim();

        // Detect PL/pgSQL dollar-quoted function boundaries
        if trimmed.contains("$$") {
            in_function_body = !in_function_body;
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);

        // A statement terminates at ';' only outside function bodies
        if !in_function_body && trimmed.ends_with(';') {
            let stmt = current.trim().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        statements.push(current.trim().to_string());
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_ddl_contains_all_tables_and_indexes() {
        assert!(!POSTGRES_SCHEMA_DDL.is_empty());
        for table in EXPECTED_TABLES {
            assert!(
                POSTGRES_SCHEMA_DDL.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
                "Missing table declaration: {table}"
            );
        }

        for idx in EXPECTED_INDEXES {
            assert!(
                POSTGRES_SCHEMA_DDL.contains(idx),
                "Missing index declaration: {idx}"
            );
        }

        // Verify foreign key constraint with cascade deletion
        assert!(POSTGRES_SCHEMA_DDL.contains("REFERENCES jobs(id) ON DELETE CASCADE"));
        // Verify trigger definitions
        assert!(POSTGRES_SCHEMA_DDL.contains("fn_set_updated_at"));
        assert!(POSTGRES_SCHEMA_DDL.contains("trg_jobs_updated_at"));
        assert!(POSTGRES_SCHEMA_DDL.contains("trg_github_installations_updated_at"));
    }

    #[test]
    fn test_split_ddl_statements() {
        let statements = split_ddl_statements(POSTGRES_SCHEMA_DDL);
        assert!(
            statements.len() >= 15,
            "Expected at least 15 discrete SQL statements, got {}",
            statements.len()
        );

        // Ensure function declaration is preserved as a single block
        let func_stmt = statements
            .iter()
            .find(|s| s.contains("FUNCTION fn_set_updated_at"))
            .expect("Function statement not found");
        assert!(func_stmt.contains("RETURN NEW;"));
    }

    #[test]
    fn test_job_kind_roundtrip() {
        let kinds = [
            (JobKind::WebSingle, "web_single"),
            (JobKind::WebBatch, "web_batch"),
            (JobKind::WebCrawl, "web_crawl"),
            (JobKind::GitHubPublic, "github_public"),
            (JobKind::GitHubPrivate, "github_private"),
        ];

        for (kind, repr) in kinds {
            assert_eq!(kind.as_str(), repr);
            assert_eq!(kind.to_string(), repr);
            assert_eq!(JobKind::from_str(repr).unwrap(), kind);
        }

        assert!(JobKind::GitHubPublic.is_github());
        assert!(JobKind::GitHubPrivate.is_github());
        assert!(!JobKind::WebSingle.is_github());

        assert!(JobKind::WebSingle.is_web());
        assert!(JobKind::WebBatch.is_web());
        assert!(JobKind::WebCrawl.is_web());
        assert!(!JobKind::GitHubPublic.is_web());

        assert!(JobKind::from_str("invalid_kind").is_err());
    }

    #[test]
    fn test_job_status_transitions_and_predicates() {
        assert!(JobStatus::Queued.can_transition_to(JobStatus::Leased));
        assert!(JobStatus::Queued.can_transition_to(JobStatus::Cancelled));
        assert!(!JobStatus::Queued.can_transition_to(JobStatus::Running));

        assert!(JobStatus::Leased.can_transition_to(JobStatus::Running));
        assert!(JobStatus::Leased.can_transition_to(JobStatus::Queued));
        assert!(JobStatus::Leased.can_transition_to(JobStatus::Failed));

        assert!(JobStatus::Running.can_transition_to(JobStatus::Completed));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Partial));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Failed));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Leased));

        // Terminal states cannot transition further
        assert!(!JobStatus::Completed.can_transition_to(JobStatus::Running));
        assert!(!JobStatus::Failed.can_transition_to(JobStatus::Queued));
        assert!(!JobStatus::Cancelled.can_transition_to(JobStatus::Leased));

        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(!JobStatus::Queued.is_terminal());
        assert!(JobStatus::Queued.is_active());

        assert!(JobStatus::from_str("invalid_status").is_err());
    }

    #[test]
    fn test_job_row_lifecycle_and_validation() {
        let config = serde_json::json!({ "token_budget": 50000 });
        let row = JobRow::new(JobKind::WebSingle, "https://example.com/docs", config);

        assert_eq!(row.kind, "web_single");
        assert_eq!(row.status, "queued");
        assert_eq!(row.progress_pct, 0);
        assert_eq!(row.progress_phase, "initializing");
        assert!(row.validate().is_ok());

        assert_eq!(row.kind_enum().unwrap(), JobKind::WebSingle);
        assert_eq!(row.status_enum().unwrap(), JobStatus::Queued);
        assert_eq!(
            row.progress_phase_enum().unwrap(),
            JobProgressPhase::Initializing
        );

        let session_id = Uuid::new_v4();
        let with_session = row.with_owner_session(session_id);
        assert_eq!(with_session.owner_session_id, Some(session_id));

        // Invalid source URL
        let mut invalid_url = with_session.clone();
        invalid_url.source_url = "   ".to_string();
        assert!(invalid_url.validate().is_err());

        // Invalid progress pct
        let mut invalid_pct = with_session.clone();
        invalid_pct.progress_pct = 101;
        assert!(invalid_pct.validate().is_err());

        // Invalid attempt count
        let mut invalid_attempt = with_session;
        invalid_attempt.attempt_count = -1;
        assert!(invalid_attempt.validate().is_err());
    }

    #[test]
    fn test_artifact_format_properties_and_roundtrip() {
        let formats = [
            (
                ArtifactFormat::CdmJson,
                "cdm_json",
                "application/vnd.antigravity.cdm+json",
                "cdm.json",
            ),
            (
                ArtifactFormat::Markdown,
                "markdown",
                "text/markdown; charset=utf-8",
                "md",
            ),
            (
                ArtifactFormat::Txt,
                "txt",
                "text/plain; charset=utf-8",
                "txt",
            ),
            (ArtifactFormat::Json, "json", "application/json", "json"),
            (
                ArtifactFormat::Html,
                "html",
                "text/html; charset=utf-8",
                "html",
            ),
            (
                ArtifactFormat::Jsonl,
                "jsonl",
                "application/x-ndjson",
                "jsonl",
            ),
            (ArtifactFormat::Zip, "zip", "application/zip", "zip"),
            (
                ArtifactFormat::Manifest,
                "manifest",
                "application/json",
                "manifest.json",
            ),
            (
                ArtifactFormat::QualityReport,
                "quality_report",
                "application/json",
                "quality.json",
            ),
        ];

        for (format, repr, mime, ext) in formats {
            assert_eq!(format.as_str(), repr);
            assert_eq!(format.to_string(), repr);
            assert_eq!(format.mime_type(), mime);
            assert_eq!(format.file_extension(), ext);
            assert_eq!(ArtifactFormat::from_str(repr).unwrap(), format);
        }

        assert!(ArtifactFormat::from_str("unknown_format").is_err());
    }

    #[test]
    fn test_job_artifact_row_validation() {
        let job_id = Uuid::new_v4();
        let sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let blake3 = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

        let artifact = JobArtifactRow::new(
            job_id,
            ArtifactFormat::Markdown,
            "jobs/123/output.md",
            1024,
            sha256,
            blake3,
            Duration::hours(24),
        );

        assert_eq!(artifact.job_id, job_id);
        assert_eq!(artifact.format, "markdown");
        assert_eq!(artifact.byte_size, 1024);
        assert!(artifact.validate().is_ok());

        // Invalid sha256 length
        let mut invalid_sha = artifact.clone();
        invalid_sha.sha256_hex = "short_hash".to_string();
        assert!(invalid_sha.validate().is_err());

        // Invalid blake3 length
        let mut invalid_blake3 = artifact.clone();
        invalid_blake3.blake3_hex = "short_blake3".to_string();
        assert!(invalid_blake3.validate().is_err());

        // Negative byte size
        let mut invalid_bytes = artifact;
        invalid_bytes.byte_size = -5;
        assert!(invalid_bytes.validate().is_err());
    }

    #[test]
    fn test_github_installation_row_validation() {
        let owner_session = Uuid::new_v4();
        let mut install = GitHubInstallationRow::new(owner_session, 123456, "octocat");

        assert_eq!(install.github_installation_id, 123456);
        assert_eq!(install.account_login, "octocat");
        assert!(install.is_active());
        assert!(!install.is_revoked());
        assert!(install.validate().is_ok());

        // Encrypted token with valid 12-byte nonce
        let nonce = vec![0u8; 12];
        let ciphertext = vec![1, 2, 3, 4, 5];
        let expires_at = Utc::now() + Duration::hours(1);
        let with_creds =
            install
                .clone()
                .with_encrypted_token(ciphertext, nonce.clone(), expires_at);
        assert!(with_creds.is_active());
        assert!(with_creds.validate().is_ok());

        // Invalid nonce length (e.g. 16 instead of 12)
        let mut invalid_nonce = with_creds.clone();
        invalid_nonce.token_nonce = Some(vec![0u8; 16]);
        assert!(invalid_nonce.validate().is_err());

        // Revocation
        install.revoke();
        assert!(install.is_revoked());
        assert!(!install.is_active());
        assert_eq!(install.encrypted_token, None);
    }

    #[test]
    fn test_content_cache_index_validation() {
        let blake3 = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        let mut cache = ContentCacheIndexRow::new(
            "git:commit:abcdef123456",
            blake3,
            "cache/blobs/ab/cdef123456",
            4096,
        );

        assert_eq!(cache.cache_key, "git:commit:abcdef123456");
        assert_eq!(cache.byte_size, 4096);
        assert!(cache.validate().is_ok());

        cache.touch();

        // Empty cache key
        let mut invalid_key = cache.clone();
        invalid_key.cache_key = "  ".to_string();
        assert!(invalid_key.validate().is_err());

        // Negative byte size
        let mut invalid_size = cache;
        invalid_size.byte_size = -1;
        assert!(invalid_size.validate().is_err());
    }
}
