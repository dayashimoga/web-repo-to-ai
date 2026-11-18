//! # Private Repository Tree & Source Extraction Engine
//!
//! Coordinates authenticated Git Trees API traversal, recursive subtree recovery,
//! file classification, bounded concurrency source and metadata extraction, mathematical
//! completeness accounting, multi-algorithm cryptographic hash validation, and
//! architecture context generation for private GitHub repositories using ephemeral GitHub App
//! installation tokens.
//!
//! ## Rule 05 Compliance & Architectural Invariants
//!
//! 1. **Authenticated Ingestion:** Ingestion uses short-lived installation access tokens (`Bearer ghs_*`)
//!    injected into [`GitHubApiClient`] headers.
//! 2. **Least Privilege & Read-Only Guarantee:** Extraction only requests read-only endpoints
//!    (`GET /repos/{owner}/{repo}/commits`, `GET /git/trees`, `GET /git/blobs`), never modifying remote state.
//! 3. **Mathematical Conservation Identity:** Completeness accounting strictly validates that
//!    $\text{Discovered} = \text{Extracted} + \text{Excluded} + \text{Failed} + \text{Unsupported} + \text{Missing}$.
//! 4. **Proactive & Resilient Token Renewal:** Interacts with [`InstallationTokenRefresher`] when available
//!    to renew tokens expiring during long-running extractions.
//! 5. **Zero Credential Leakage:** Tokens are excluded from result structures, debug strings, and error traces.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};
use uuid::Uuid;

use crate::accounting::{FileAccountingAuditor, FileAccountingBalanceSheet};
use crate::architecture::RepoArchitectureContext;
use crate::authorization::AuthorizedInstallationContext;
use crate::classification::{
    ClassifiedFile, FileCategory, FileClassificationConfig, TreeClassificationReport,
};
use crate::client::GitHubApiClient;
use crate::commit::GitHubCommitMetadata;
use crate::config_test::ConfigTestExtractionReport;
use crate::dependency::DependencyExtractionReport;
use crate::documentation::DocExtractionReport;
use crate::error::GitHubExtractorError;
use crate::hash_validator::{ContentHashValidator, HashValidationConfig, HashValidationReport};
use crate::manifest::RepoFileManifest;
use crate::metadata::GitHubRepositoryMetadata;
use crate::recovery::SubtreeRecoveryConfig;
use crate::source::{SourceExtractionConfig, SourceExtractionReport};
use crate::target::GitHubTarget;
use crate::token_refresh::{InstallationTokenRefresher, InstallationTokenStore};
use crate::tree::GitTreeResponse;

/// Configuration governing private repository tree and content extraction.
#[derive(Debug, Clone)]
pub struct PrivateExtractionConfig {
    /// Configuration for source file extraction (concurrency, size bounds).
    pub source_config: SourceExtractionConfig,
    /// Configuration for classifying discovered tree files.
    pub classification_config: FileClassificationConfig,
    /// Configuration for automated subtree recovery on truncation.
    pub recovery_config: SubtreeRecoveryConfig,
    /// Configuration for multi-algorithm cryptographic hash validation.
    pub hash_config: HashValidationConfig,
    /// Optional git ref to extract (defaults to default branch or `"HEAD"`).
    pub target_ref: Option<String>,
    /// Whether to fetch repository metadata (description, license, topics).
    pub fetch_metadata: bool,
    /// Safety margin in seconds for proactive token renewal.
    pub proactive_token_renewal_margin_secs: u64,
}

impl Default for PrivateExtractionConfig {
    fn default() -> Self {
        Self {
            source_config: SourceExtractionConfig::default(),
            classification_config: FileClassificationConfig::default(),
            recovery_config: SubtreeRecoveryConfig::default(),
            hash_config: HashValidationConfig::default(),
            target_ref: None,
            fetch_metadata: true,
            proactive_token_renewal_margin_secs: 300,
        }
    }
}

impl PrivateExtractionConfig {
    /// Overrides the source extraction configuration.
    #[must_use]
    pub fn with_source_config(mut self, config: SourceExtractionConfig) -> Self {
        self.source_config = config;
        self
    }

    /// Overrides the file classification configuration.
    #[must_use]
    pub fn with_classification_config(mut self, config: FileClassificationConfig) -> Self {
        self.classification_config = config;
        self
    }

    /// Overrides the subtree recovery configuration.
    #[must_use]
    pub fn with_recovery_config(mut self, config: SubtreeRecoveryConfig) -> Self {
        self.recovery_config = config;
        self
    }

    /// Overrides the hash validation configuration.
    #[must_use]
    pub fn with_hash_config(mut self, config: HashValidationConfig) -> Self {
        self.hash_config = config;
        self
    }

    /// Sets an explicit target git ref (commit SHA, branch, or tag).
    #[must_use]
    pub fn with_target_ref(mut self, target_ref: impl Into<String>) -> Self {
        self.target_ref = Some(target_ref.into());
        self
    }

    /// Toggles fetching repository metadata.
    #[must_use]
    pub fn with_fetch_metadata(mut self, fetch: bool) -> Self {
        self.fetch_metadata = fetch;
        self
    }

    /// Sets the proactive token renewal safety margin in seconds.
    #[must_use]
    pub fn with_proactive_token_renewal_margin_secs(mut self, secs: u64) -> Self {
        self.proactive_token_renewal_margin_secs = secs;
        self
    }
}

/// Comprehensive result of extracting a private repository.
#[derive(Debug, Clone)]
pub struct PrivateExtractionResult {
    /// GitHub target repository identifier (`owner/repo`).
    pub target: GitHubTarget,
    /// Installation ID that authorized the extraction.
    pub installation_id: i64,
    /// Account login owning the installation.
    pub account_login: String,
    /// Verified commit metadata.
    pub commit: GitHubCommitMetadata,
    /// Optional repository metadata.
    pub metadata: Option<GitHubRepositoryMetadata>,
    /// Recovered full Git tree.
    pub tree: GitTreeResponse,
    /// File classification report.
    pub classification: TreeClassificationReport,
    /// Extracted source code files.
    pub source_report: SourceExtractionReport,
    /// Extracted documentation files.
    pub doc_report: DocExtractionReport,
    /// Extracted dependency manifests.
    pub dependency_report: DependencyExtractionReport,
    /// Extracted configuration and test files.
    pub config_test_report: ConfigTestExtractionReport,
    /// Discovered = Extracted + Excluded + Failed accounting balance sheet.
    pub accounting: FileAccountingBalanceSheet,
    /// Multi-algorithm cryptographic hash validation report.
    pub hash_validation: HashValidationReport,
    /// Consolidated file manifest.
    pub manifest: RepoFileManifest,
    /// Architecture context.
    pub architecture: RepoArchitectureContext,
    /// Total wall-clock duration of extraction.
    pub duration: Duration,
}

impl PrivateExtractionResult {
    /// Returns true if the mathematical conservation identity strictly holds with zero discrepancies.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.accounting.is_balanced()
    }

    /// Returns the total number of successfully extracted content files across all categories.
    #[must_use]
    pub fn total_extracted_files(&self) -> usize {
        self.source_report.extracted.len()
            + self.doc_report.extracted.len()
            + self.dependency_report.extracted.len()
            + self.config_test_report.extracted_configs.len()
            + self.config_test_report.extracted_tests.len()
    }

    /// Returns the total number of discovered tree entries.
    #[must_use]
    pub fn total_discovered_entries(&self) -> usize {
        self.tree.tree.len()
    }

    /// Returns the total bytes extracted across all file contents.
    #[must_use]
    pub fn total_extracted_bytes(&self) -> u64 {
        let src_bytes: u64 = self
            .source_report
            .extracted
            .iter()
            .map(|f| f.byte_size as u64)
            .sum();
        let doc_bytes: u64 = self
            .doc_report
            .extracted
            .iter()
            .map(|f| f.byte_size as u64)
            .sum();
        let dep_bytes: u64 = self
            .dependency_report
            .extracted
            .iter()
            .map(|f| f.byte_size as u64)
            .sum();
        let cfg_bytes: u64 = self
            .config_test_report
            .extracted_configs
            .iter()
            .map(|f| f.byte_size as u64)
            .sum();
        let tst_bytes: u64 = self
            .config_test_report
            .extracted_tests
            .iter()
            .map(|f| f.byte_size as u64)
            .sum();
        src_bytes + doc_bytes + dep_bytes + cfg_bytes + tst_bytes
    }

    /// Returns the 40-character commit SHA extracted.
    #[must_use]
    pub fn commit_sha(&self) -> &str {
        &self.commit.sha
    }

    /// Returns the 40-character root tree SHA extracted.
    #[must_use]
    pub fn root_tree_sha(&self) -> &str {
        &self.commit.tree_sha
    }
}

impl fmt::Display for PrivateExtractionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PrivateExtractionResult(target={}/{}, commit={}, extracted_files={}, balanced={})",
            self.target.owner,
            self.target.repo,
            self.commit.short_sha(),
            self.total_extracted_files(),
            self.is_balanced()
        )
    }
}

/// Orchestrator executing end-to-end tree and content extraction for private GitHub repositories.
pub struct PrivateRepositoryExtractor {
    client: Arc<GitHubApiClient>,
    token_refresher: Option<Arc<InstallationTokenRefresher>>,
    token_store: Option<Arc<dyn InstallationTokenStore>>,
    session_id: Option<Uuid>,
    default_config: PrivateExtractionConfig,
}

impl PrivateRepositoryExtractor {
    /// Creates a new `PrivateRepositoryExtractor` with default settings.
    pub fn new(client: Arc<GitHubApiClient>) -> Self {
        Self {
            client,
            token_refresher: None,
            token_store: None,
            session_id: None,
            default_config: PrivateExtractionConfig::default(),
        }
    }

    /// Injects an optional [`InstallationTokenRefresher`] for automated renewal of expiring tokens.
    #[must_use]
    pub fn with_token_refresher(
        mut self,
        refresher: Arc<InstallationTokenRefresher>,
        store: Arc<dyn InstallationTokenStore>,
        session_id: Uuid,
    ) -> Self {
        self.token_refresher = Some(refresher);
        self.token_store = Some(store);
        self.session_id = Some(session_id);
        self
    }

    /// Configures default extraction settings.
    #[must_use]
    pub fn with_config(mut self, config: PrivateExtractionConfig) -> Self {
        self.default_config = config;
        self
    }

    /// Executes end-to-end private repository extraction.
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError`] if authentication, commit resolution, tree traversal,
    /// or content acquisition fails.
    pub async fn extract(
        &self,
        ctx: &AuthorizedInstallationContext,
        config_override: Option<&PrivateExtractionConfig>,
    ) -> Result<PrivateExtractionResult, GitHubExtractorError> {
        let start = Instant::now();
        let effective_config = config_override.unwrap_or(&self.default_config);

        info!(
            "Starting private repository extraction for {}/{} (installation_id={})",
            ctx.owner, ctx.repo, ctx.installation_id
        );

        // 1. Target resolution
        let target = GitHubTarget::for_repo(&ctx.owner, &ctx.repo);

        // 2. Token freshness and authenticated client preparation
        let mut active_token = ctx.token.clone();
        if let (Some(refresher), Some(store), Some(session_id)) =
            (&self.token_refresher, &self.token_store, &self.session_id)
        {
            if let Ok(Some(record)) = store.get_token(session_id, ctx.installation_id).await {
                if record
                    .freshness(effective_config.proactive_token_renewal_margin_secs)
                    .needs_refresh()
                {
                    debug!(
                        "Token for installation {} needs refresh before extraction; renewing",
                        ctx.installation_id
                    );
                    if let Ok(new_token) = refresher
                        .get_or_refresh_token(session_id, ctx.installation_id, &ctx.account_login)
                        .await
                    {
                        active_token = new_token;
                    }
                }
            }
        }

        let auth_client = self.client.as_ref().clone().with_auth_token(&active_token);

        // 3. Ref discovery and commit resolution
        let git_ref = effective_config.target_ref.as_deref().unwrap_or("HEAD");
        debug!("Resolving commit for ref '{git_ref}'");
        let (commit, _) = auth_client
            .get_commit(&target, git_ref)
            .await
            .map_err(|e| e.sanitized())?;

        // 4. Optional repository metadata
        let metadata = if effective_config.fetch_metadata {
            auth_client
                .get_repo_metadata(&target)
                .await
                .map(|(m, _)| m)
                .ok()
        } else {
            None
        };

        // 5. Tree traversal and automated subtree recovery
        debug!("Fetching root tree for tree SHA '{}'", commit.tree_sha);
        let (base_tree, _) = auth_client
            .get_tree(&target, &commit.tree_sha, true)
            .await
            .map_err(|e| e.sanitized())?;

        let (recovered_tree, _) = auth_client
            .recover_tree(
                &target,
                base_tree,
                &effective_config.recovery_config,
                Some(&commit.sha),
            )
            .await
            .map_err(|e| e.sanitized())?;

        // 6. File classification
        debug!("Classifying {} tree entries", recovered_tree.tree.len());
        let classification = recovered_tree.classify(&effective_config.classification_config);

        // Partition classified files by category for downstream extraction
        let source_files: Vec<ClassifiedFile> = classification
            .files
            .iter()
            .filter(|f| f.category == FileCategory::SourceCode)
            .cloned()
            .collect();

        let doc_files: Vec<ClassifiedFile> = classification
            .files
            .iter()
            .filter(|f| f.category == FileCategory::Documentation)
            .cloned()
            .collect();

        let dep_files: Vec<ClassifiedFile> = classification
            .files
            .iter()
            .filter(|f| f.category == FileCategory::DependencyManifest)
            .cloned()
            .collect();

        let config_test_files: Vec<ClassifiedFile> = classification
            .files
            .iter()
            .filter(|f| f.category == FileCategory::Configuration || f.category.is_test())
            .cloned()
            .collect();

        // 7. Bounded concurrent content extraction across all categories
        debug!(
            "Extracting contents: {} sources, {} docs, {} dependencies, {} configs/tests",
            source_files.len(),
            doc_files.len(),
            dep_files.len(),
            config_test_files.len()
        );

        let (source_report, doc_report, dependency_report, config_test_report) = tokio::join!(
            auth_client.extract_source_files(
                &target,
                &source_files,
                &effective_config.source_config
            ),
            auth_client.extract_doc_files(&target, &doc_files, &effective_config.source_config),
            auth_client.extract_dependency_files(
                &target,
                &dep_files,
                &effective_config.source_config
            ),
            auth_client.extract_config_and_test_files(
                &target,
                &config_test_files,
                &effective_config.source_config
            ),
        );

        // 8. Completeness accounting (Discovered = Extracted + Excluded + Failed)
        let target_slug = format!("{}/{}", ctx.owner, ctx.repo);
        let accounting = FileAccountingAuditor::audit_reports(
            &recovered_tree,
            Some(&classification),
            Some(&source_report),
            Some(&doc_report),
            Some(&dependency_report),
            Some(&config_test_report),
            &target_slug,
            effective_config.target_ref.clone(),
            Some(commit.sha.clone()),
        );

        // 9. Multi-algorithm cryptographic hash validation
        let hash_validation = ContentHashValidator::validate_reports(
            Some(&source_report),
            Some(&doc_report),
            Some(&dependency_report),
            Some(&config_test_report),
            &effective_config.hash_config,
            &target_slug,
        );

        // 10. File manifest assembly
        let manifest = RepoFileManifest::from_classification_and_reports(
            &classification,
            Some(&source_report),
            Some(&doc_report),
            Some(&dependency_report),
            Some(&config_test_report),
        );

        // 11. Architecture context derivation
        let architecture = RepoArchitectureContext::from_repository_data(
            &classification,
            Some(&manifest),
            Some(&doc_report),
            Some(&dependency_report),
            Some(&config_test_report),
            metadata.as_ref(),
        );

        let elapsed = start.elapsed();
        info!(
            "Successfully extracted private repository {}/{} in {:.2?}: {} files extracted, {} tree entries, balanced={}",
            ctx.owner,
            ctx.repo,
            elapsed,
            source_report.extracted.len()
                + doc_report.extracted.len()
                + dependency_report.extracted.len()
                + config_test_report.extracted_configs.len()
                + config_test_report.extracted_tests.len(),
            recovered_tree.tree.len(),
            accounting.is_balanced()
        );

        Ok(PrivateExtractionResult {
            target,
            installation_id: ctx.installation_id,
            account_login: ctx.account_login.clone(),
            commit,
            metadata,
            tree: recovered_tree,
            classification,
            source_report,
            doc_report,
            dependency_report,
            config_test_report,
            accounting,
            hash_validation,
            manifest,
            architecture,
            duration: elapsed,
        })
    }
}
