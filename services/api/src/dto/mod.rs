//! Data Transfer Objects (DTOs) and request/response models.

pub mod batch;
pub mod github;
pub mod jobs;
pub mod params;
pub mod query;
pub mod sources;

pub use batch::{
    BatchItemPathParams, BatchItemRequest, BatchItemStatus, BatchItemsListResponse,
    BatchJobResponse, BatchPathParams, BatchProgress, BatchStatus, BatchSummaryItem,
    CreateBatchJobRequest, CreateBatchJobResponse, ListBatchesResponse, DEFAULT_BATCH_PARALLELISM,
    MAX_BATCH_PARALLELISM, MAX_BATCH_URLS, MIN_BATCH_URLS, VALID_CONTEXT_PROFILES,
};
pub use github::{
    InstallationPathParams, InstallationSummaryResponse, RevokeInstallationResponse,
    WebhookReceiptResponse,
};
pub use jobs::{
    CreateJobRequest, JobPreviewResponse, JobResponse, JobStatus, PreviewFileItem, PreviewQuery,
    VALID_EXPORT_FORMATS, VALID_PREVIEW_FORMATS,
};
pub use params::{ArtifactPathParams, JobPathParams};
pub use query::{ListJobsQuery, PaginatedResponse, VALID_SORT_ORDERS, VALID_STATUS_FILTERS};
pub use sources::{
    classify_source_url, DetectSourcesRequest, DetectSourcesResponse, DetectedSource,
    SourceClassification,
};

#[cfg(test)]
mod tests {
    use super::*;
    use web_repo_to_ai_core::{JobId, Validatable};

    #[test]
    fn test_detect_sources_request_validation() {
        // Valid
        let req = DetectSourcesRequest {
            urls: vec!["https://github.com/rust-lang/rust".to_string()],
        };
        assert!(req.validate().is_ok());

        // Empty URLs
        let req_empty = DetectSourcesRequest { urls: vec![] };
        assert!(req_empty.validate().is_err());

        // Invalid URL scheme
        let req_scheme = DetectSourcesRequest {
            urls: vec!["ftp://files.example.com".to_string()],
        };
        assert!(req_scheme.validate().is_err());

        // Malformed URL
        let req_malformed = DetectSourcesRequest {
            urls: vec!["not a url".to_string()],
        };
        assert!(req_malformed.validate().is_err());

        // > 50 URLs
        let too_many = (0..51)
            .map(|i| format!("https://example.com/{i}"))
            .collect();
        let req_overflow = DetectSourcesRequest { urls: too_many };
        assert!(req_overflow.validate().is_err());
    }

    #[test]
    fn test_source_classification() {
        let gh = classify_source_url("https://github.com/tokio-rs/tokio/tree/master/tokio");
        assert_eq!(gh.classification, SourceClassification::PublicGitHub);
        assert_eq!(gh.repo_owner.as_deref(), Some("tokio-rs"));
        assert_eq!(gh.repo_name.as_deref(), Some("tokio"));
        assert_eq!(gh.git_ref.as_deref(), Some("master"));
        assert_eq!(gh.subpath.as_deref(), Some("tokio"));

        let web = classify_source_url("https://docs.rs/axum/latest/axum/");
        assert_eq!(web.classification, SourceClassification::WebPage);
        assert_eq!(web.repo_owner, None);
    }

    #[test]
    fn test_create_job_request_validation() {
        let valid_job = CreateJobRequest {
            sources: vec!["https://github.com/owner/repo".to_string()],
            git_ref: Some("main".to_string()),
            path_patterns: Some(vec!["src/**/*.rs".to_string()]),
            token_budget: Some(100_000),
            target_model: Some("claude-3-5-sonnet".to_string()),
            export_formats: Some(vec!["markdown".to_string(), "json".to_string()]),
            deduplicate_exact: Some(true),
            deduplicate_semantic: Some(true),
            remove_comments: Some(false),
        };
        assert!(valid_job.validate().is_ok());

        // Invalid git_ref
        let bad_ref = CreateJobRequest {
            git_ref: Some("feature/../bad".to_string()),
            ..valid_job.clone()
        };
        assert!(bad_ref.validate().is_err());

        // Token budget too small
        let bad_budget_low = CreateJobRequest {
            token_budget: Some(500),
            ..valid_job.clone()
        };
        assert!(bad_budget_low.validate().is_err());

        // Token budget too high
        let bad_budget_high = CreateJobRequest {
            token_budget: Some(5_000_000),
            ..valid_job.clone()
        };
        assert!(bad_budget_high.validate().is_err());

        // Invalid export format
        let bad_format = CreateJobRequest {
            export_formats: Some(vec!["exe".to_string()]),
            ..valid_job
        };
        assert!(bad_format.validate().is_err());
    }

    #[test]
    fn test_list_jobs_query_validation() {
        let valid_query = ListJobsQuery {
            page: Some(1),
            limit: Some(20),
            status: Some("completed".to_string()),
            sort_order: Some("desc".to_string()),
        };
        assert!(valid_query.validate().is_ok());

        let invalid_page = ListJobsQuery {
            page: Some(0),
            ..Default::default()
        };
        assert!(invalid_page.validate().is_err());

        let invalid_limit = ListJobsQuery {
            limit: Some(200),
            ..Default::default()
        };
        assert!(invalid_limit.validate().is_err());

        let invalid_status = ListJobsQuery {
            status: Some("unknown_status".to_string()),
            ..Default::default()
        };
        assert!(invalid_status.validate().is_err());
    }

    #[test]
    fn test_artifact_path_params_validation() {
        let job_id = JobId::new();
        let valid = ArtifactPathParams {
            job_id,
            format: "markdown".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = ArtifactPathParams {
            job_id,
            format: "unknown".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_preview_query_validation() {
        let valid = PreviewQuery {
            format: Some("markdown".to_string()),
            max_lines: Some(500),
            file_path: Some("src/main.rs".to_string()),
        };
        assert!(valid.validate().is_ok());

        let invalid_fmt = PreviewQuery {
            format: Some("exe".to_string()),
            ..Default::default()
        };
        assert!(invalid_fmt.validate().is_err());

        let invalid_max = PreviewQuery {
            max_lines: Some(0),
            ..Default::default()
        };
        assert!(invalid_max.validate().is_err());

        let too_high_max = PreviewQuery {
            max_lines: Some(100_000),
            ..Default::default()
        };
        assert!(too_high_max.validate().is_err());
    }

    #[test]
    fn test_create_batch_job_request_validation() {
        // Valid shorthand
        let valid_shorthand = CreateBatchJobRequest {
            urls: Some(vec![
                "https://github.com/rust-lang/rust".to_string(),
                "https://example.com/docs".to_string(),
            ]),
            ..Default::default()
        };
        assert!(valid_shorthand.validate().is_ok());
        assert_eq!(valid_shorthand.effective_items().len(), 2);

        // Valid structured items with overrides
        let valid_items = CreateBatchJobRequest {
            items: Some(vec![BatchItemRequest {
                url: "https://github.com/org/repo".to_string(),
                custom_id: Some("item-1".to_string()),
                git_ref: Some("main".to_string()),
                path_patterns: None,
                token_budget: Some(50_000),
                target_model: Some("gpt-4o".to_string()),
                export_formats: Some(vec!["markdown".to_string()]),
                context_profile: Some("code_review".to_string()),
            }]),
            ..Default::default()
        };
        assert!(valid_items.validate().is_ok());

        // Empty batch rejected
        let empty_batch = CreateBatchJobRequest::default();
        assert!(empty_batch.validate().is_err());

        // Batch > 100 rejected
        let too_many_urls = (0..101)
            .map(|i| format!("https://example.com/item/{i}"))
            .collect();
        let overflow_batch = CreateBatchJobRequest {
            urls: Some(too_many_urls),
            ..Default::default()
        };
        assert!(overflow_batch.validate().is_err());

        // Invalid URL scheme in batch item
        let invalid_scheme = CreateBatchJobRequest {
            urls: Some(vec!["ftp://files.example.com".to_string()]),
            ..Default::default()
        };
        assert!(invalid_scheme.validate().is_err());

        // Duplicate URLs within batch deduplicated
        let duplicates = CreateBatchJobRequest {
            urls: Some(vec![
                "https://example.com/item1".to_string(),
                "https://example.com/item1".to_string(),
                "https://example.com/item2".to_string(),
            ]),
            ..Default::default()
        };
        assert!(duplicates.validate().is_ok());
        assert_eq!(duplicates.effective_items().len(), 2);
    }
}
