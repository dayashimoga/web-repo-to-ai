//! # GitHub Extractor (`web-repo-to-ai-github-extractor`)
//!
//! Provides deep repository ingestion for public and private GitHub sources:
//! - GitHub URL parsing, target decomposition, and validation (`GitHubTarget`, `GitHubUrlParser`)
//! - Branch and release tag discovery with pagination and rate limit tracking (`GitHubApiClient`, `RefDiscoveryReport`)
//! - Git Trees API traversal with recursive subtree recovery
//! - Completeness manifest accounting (Discovered = Extracted + Excluded + Failed)
//! - Ephemeral GitHub App installation token lifecycle management

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accounting;
pub mod app_manifest;
pub mod architecture;
pub mod authorization;
pub mod callback;
pub mod classification;
pub mod client;
pub mod code_review;
pub mod commit;
pub mod config_test;
pub mod dependency;
pub mod documentation;
pub mod error;
pub mod exclusion;
pub mod hash_validator;
pub mod installation;
pub mod manifest;
pub mod metadata;
pub mod models;
pub mod oauth;
pub mod permissions;
pub mod private_extractor;
pub mod recovery;
pub mod revocation;
pub mod selected_repositories;
pub mod source;
pub mod target;
pub mod token_refresh;
pub mod tree;
pub mod tree_generator;

pub use accounting::{
    AccountingDiscrepancy, AccountingDiscrepancyKind, DiscrepancySeverity, FileAccountingAuditor,
    FileAccountingBalanceSheet,
};
pub use app_manifest::{
    GitHubAppConfig, GitHubAppConfigBuilder, GitHubAppManifest, GitHubAppManifestGenerator,
    GitHubAppPermissions, GitHubAppPublicMetadata, GitHubHookAttributes, PermissionLevel,
};
pub use architecture::{
    ComponentInfo, ComponentRole, EntryPointInfo, EntryPointRole, RepoArchitectureContext,
    WorkspaceType,
};
pub use authorization::{
    AuthorizationOutcome, AuthorizedInstallationContext, InstallationAuthorizer,
};
pub use callback::{
    GitHubWebhookEvent, InMemoryWebhookDeliveryTracker, InstallationAction,
    InstallationCallbackParams, InstallationCallbackValidator, InstallationRepoAction,
    OAuthCallbackParams, OAuthCallbackValidator, OpenRedirectGuard, SetupAction,
    ValidatedInstallationCallback, ValidatedOAuthCallback, WebhookDeliveryTracker,
    WebhookEventParser, WebhookSignatureVerifier,
};
pub use classification::{
    detect_language, ClassifiedFile, FileCategory, FileClassificationConfig, FileClassifier,
    TreeClassificationReport, DEFAULT_MAX_FILE_SIZE_BYTES,
};
pub use client::{
    GitHubApiClient, DEFAULT_GITHUB_TIMEOUT, DEFAULT_MAX_PAGES, DEFAULT_PER_PAGE,
    DEFAULT_USER_AGENT, GITHUB_API_VERSION_HEADER,
};
pub use code_review::{
    CodeReviewPackage, CodeReviewPackageConfig, PrioritizedReviewFile, ReviewChecklistItem,
    ReviewFocus, ReviewPriorityTier,
};
pub use commit::{CommitAuthor, CommitSignatureVerification, GitHubCommitMetadata, PinnedRevision};
pub use config_test::{
    detect_config_category, detect_config_format, detect_test_category, ConfigCategory,
    ConfigTestExtractionReport, ExtractedConfigFile, ExtractedTestFile, TestCategory,
};
pub use dependency::{
    detect_ecosystem, detect_manifest_role, parse_cargo_toml, parse_go_mod, parse_manifest,
    parse_package_json, parse_pyproject_toml, parse_requirements_txt, DeclaredDependency,
    DependencyExtractionReport, DependencyType, ExtractedDependencyFile, ManifestRole,
    PackageEcosystem, ParsedManifest,
};
pub use documentation::{
    detect_doc_format, detect_doc_type, extract_title_and_frontmatter, DocExtractionReport,
    DocFormat, DocType, ExtractedDocFile,
};
pub use error::{redact_secrets, GitHubExtractorError};
pub use exclusion::{
    ExcludedFileRecord, ExclusionAuditReport, ExclusionCategory, ExclusionReportGenerator,
    ExtractionFailureCategory, FailedFileRecord, MissingFileRecord,
};
pub use hash_validator::{
    ContentHashValidator, HashAlgorithm, HashCheckMetric, HashValidationConfig,
    HashValidationFailure, HashValidationReport,
};
pub use installation::{
    GitHubAccountSummary, GitHubInstallation, GitHubInstallationToken,
    InstallationRepositoriesResponse, InstallationTokenRequest, InstalledRepositorySummary,
    RepositoryAccessDetails, RepositoryPermissions, RepositorySelection, UserInstallationsResponse,
};
pub use manifest::{FileExtractionStatus, ManifestFileEntry, RepoFileManifest};
pub use metadata::{GitHubLanguages, GitHubLicense, GitHubRepoContext, GitHubRepositoryMetadata};
pub use models::{GitHubBranch, GitHubRateLimit, GitHubTag, RefDiscoveryReport};
pub use oauth::{
    base64url_encode, build_app_installation_url, build_oauth_authorize_url, generate_app_jwt,
    generate_app_jwt_at_time, parse_rsa_pem_to_pkcs8_der, pkcs1_to_pkcs8_der,
    GitHubOAuthErrorResponse, GitHubOAuthTokenResponse, InMemoryOAuthStateStore, OAuthState,
    OAuthStateStore,
};
pub use permissions::{
    PermissionAuditReport, PermissionAuditor, PermissionPolicy, PermissionPolicyBuilder,
    PermissionViolation, DEFAULT_FORBIDDEN_CATEGORIES, DEFAULT_FORBIDDEN_LEVELS,
};
pub use private_extractor::{
    PrivateExtractionConfig, PrivateExtractionResult, PrivateRepositoryExtractor,
};
pub use recovery::{
    scan_zip_archive, stitch_child_path, ArchiveFormat, SubtreeRecoveryConfig,
    SubtreeRecoveryReport, TreeStitcher,
};
pub use revocation::{
    RevocationOutcome, RevocationSummary, TokenRevocationCoordinator, WebhookLifecycleAction,
};
pub use selected_repositories::{
    RepositoryAccessStatus, SelectedRepositoryListing, SelectedRepositoryManager, SyncSummary,
    DEFAULT_MAX_PAGINATION_PAGES, DEFAULT_REPOSITORIES_PER_PAGE,
};
pub use source::{
    compute_blake3_hash, compute_git_blob_sha, compute_sha256_hash, decode_base64,
    is_binary_content, ExtractedSourceFile, FailedSourceExtraction, GitBlob,
    SourceExtractionConfig, SourceExtractionReport, BINARY_SNIFF_WINDOW_BYTES,
    DEFAULT_EXTRACTION_CONCURRENCY,
};
pub use target::{
    is_default_host, is_reserved_github_path, validate_and_sanitize_subpath, validate_owner,
    validate_repo, GitHubTarget, GitHubTargetType, GitHubUrlParser, DEFAULT_GITHUB_API_BASE,
    DEFAULT_GITHUB_HOST, DEFAULT_GITHUB_WWW_HOST, DEFAULT_RAW_GITHUB_USERCONTENT_BASE,
};
pub use token_refresh::{
    InMemoryInstallationTokenStore, InstallationTokenRefresher, InstallationTokenStore,
    StoredTokenRecord, TokenFreshness, TokenLifecyclePolicy,
};
pub use tree::{
    GitFileMode, GitTreeEntry, GitTreeEntryType, GitTreeResponse, RecoveryStrategy,
    TreeCompleteness, TruncationReason, TruncationReport, GITHUB_TREE_ENTRY_LIMIT,
    SUBTREE_TRAVERSAL_LIMIT,
};
pub use tree_generator::{
    category_badge, format_byte_size, RepoTree, RepoTreeNode, TreeFormatConfig, TreeStyle,
};

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_extractor_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-github-extractor");
    }
}
