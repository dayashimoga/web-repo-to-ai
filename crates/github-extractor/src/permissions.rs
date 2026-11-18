//! # GitHub App Minimum Permissions & Least-Privilege Verification
//!
//! Provides runtime verification, policy enforcement, excessive/forbidden scope detection,
//! and audit reporting for GitHub App installations and short-lived installation access tokens.
//!
//! ## Rule 05 Compliance & Least-Privilege Invariants
//!
//! Under Rule 05 ("Network, GitHub & Application Security"), repository conversion must
//! operate under the principle of least privilege:
//! 1. **Mandatory Read-Only Scopes:** The engine requires strictly `Contents: Read-only`
//!    and `Metadata: Read-only` to read directory trees, commit metadata, and blobs.
//! 2. **Zero Write/Admin Scopes:** Any write or administrative permission (e.g. `contents: write`,
//!    `issues: write`, `pull_requests: write`, `administration: write`) is strictly forbidden
//!    and triggers immediate fail-closed validation rejection.
//! 3. **Prohibited Sensitive Categories:** High-risk permission categories that expose
//!    credentials, CI/CD secrets, deployment environments, or administrative controls
//!    (e.g. `secrets`, `actions`, `administration`, `deployments`, `packages`, `security_events`,
//!    `members`, `organization_administration`, `workflows`) are strictly forbidden even if
//!    requested with read-only level.
//! 4. **Installation Suspension Check:** Suspended installations are flagged as non-compliant
//!    and rejected before initiating any API interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::error::GitHubExtractorError;
use crate::installation::{GitHubInstallation, GitHubInstallationToken};

/// Default forbidden permission categories presenting credential exposure, CI/CD, or administrative risk.
pub const DEFAULT_FORBIDDEN_CATEGORIES: &[&str] = &[
    "administration",
    "actions",
    "secrets",
    "deployments",
    "packages",
    "security_events",
    "members",
    "organization_administration",
    "organization_secrets",
    "organization_actions",
    "workflows",
    "blocking_keys",
];

/// Default forbidden permission levels violating read-only least-privilege.
pub const DEFAULT_FORBIDDEN_LEVELS: &[&str] = &["write", "admin"];

/// Specification of a granular permission violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PermissionViolation {
    /// A mandatory permission scope is missing.
    MissingRequired {
        /// Name of the missing scope (e.g. `"contents"`).
        scope: String,
        /// Expected minimum level (e.g. `"read"`).
        expected: String,
    },
    /// A granted permission exceeds the maximum allowed level (e.g. `"write"` when only `"read"` is permitted).
    ExcessiveLevel {
        /// Name of the scope with excessive access.
        scope: String,
        /// The granted permission level (e.g. `"write"` or `"admin"`).
        granted: String,
        /// The maximum permitted level (e.g. `"read"`).
        max_allowed: String,
    },
    /// A permission category is forbidden by policy (e.g. `"secrets"`, `"actions"`, `"administration"`).
    ForbiddenScope {
        /// Name of the forbidden scope.
        scope: String,
        /// The granted level.
        granted: String,
        /// Reason why this scope is forbidden.
        reason: String,
    },
    /// An unexpected additional scope was detected when strict minimal scoping is enforced.
    UnexpectedScope {
        /// Name of the unapproved scope.
        scope: String,
        /// The granted level.
        granted: String,
    },
    /// The GitHub App installation is currently suspended by GitHub or the administrator.
    SuspendedInstallation {
        /// Numeric ID of the suspended installation.
        installation_id: u64,
    },
}

impl fmt::Display for PermissionViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequired { scope, expected } => {
                write!(
                    f,
                    "Missing required permission scope '{scope}' (expected: '{expected}')"
                )
            }
            Self::ExcessiveLevel {
                scope,
                granted,
                max_allowed,
            } => {
                write!(
                    f,
                    "Excessive permission level for scope '{scope}': granted '{granted}', max permitted '{max_allowed}'"
                )
            }
            Self::ForbiddenScope {
                scope,
                granted,
                reason,
            } => {
                write!(
                    f,
                    "Forbidden permission scope '{scope}' ('{granted}') detected: {reason}"
                )
            }
            Self::UnexpectedScope { scope, granted } => {
                write!(
                    f,
                    "Unexpected permission scope '{scope}' ('{granted}') not permitted by strict minimal policy"
                )
            }
            Self::SuspendedInstallation { installation_id } => {
                write!(
                    f,
                    "Installation {installation_id} is currently suspended and cannot be accessed"
                )
            }
        }
    }
}

/// Configurable permission policy for evaluating GitHub App installations and tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionPolicy {
    /// Human-readable policy name (e.g. `"StrictMinimal"`, `"Standard"`).
    pub name: String,
    /// Map of required scopes and their exact required level (e.g. `{"contents": "read", "metadata": "read"}`).
    pub required_scopes: HashMap<String, String>,
    /// List of permission category names that are strictly forbidden.
    pub forbidden_categories: Vec<String>,
    /// List of permission levels that are forbidden across all scopes (e.g. `["write", "admin"]`).
    pub forbidden_levels: Vec<String>,
    /// Whether additional non-required read-only scopes are permitted (e.g. `pull_requests: read`).
    pub allow_additional_read_scopes: bool,
    /// Whitelist of additional permitted read-only scopes if `allow_additional_read_scopes` is true.
    /// If empty, any benign non-forbidden read-only scope is allowed.
    pub allowed_additional_scopes: Vec<String>,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self::strict_minimal()
    }
}

impl PermissionPolicy {
    /// Creates a strict minimal policy requiring exactly `contents: read` and `metadata: read`.
    ///
    /// Any additional scope or non-read level is flagged as a violation.
    #[must_use]
    pub fn strict_minimal() -> Self {
        let mut required = HashMap::new();
        required.insert("contents".to_string(), "read".to_string());
        required.insert("metadata".to_string(), "read".to_string());

        Self {
            name: "StrictMinimal".to_string(),
            required_scopes: required,
            forbidden_categories: DEFAULT_FORBIDDEN_CATEGORIES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            forbidden_levels: DEFAULT_FORBIDDEN_LEVELS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            allow_additional_read_scopes: false,
            allowed_additional_scopes: Vec::new(),
        }
    }

    /// Creates a standard policy requiring `contents: read` and `metadata: read`,
    /// permitting benign read-only scopes (e.g. `pull_requests: read`), while
    /// strictly forbidding all write/admin levels and dangerous categories.
    #[must_use]
    pub fn standard() -> Self {
        let mut required = HashMap::new();
        required.insert("contents".to_string(), "read".to_string());
        required.insert("metadata".to_string(), "read".to_string());

        Self {
            name: "Standard".to_string(),
            required_scopes: required,
            forbidden_categories: DEFAULT_FORBIDDEN_CATEGORIES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            forbidden_levels: DEFAULT_FORBIDDEN_LEVELS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            allow_additional_read_scopes: true,
            allowed_additional_scopes: Vec::new(),
        }
    }

    /// Creates a fluent builder for constructing a custom [`PermissionPolicy`].
    #[must_use]
    pub fn builder(name: impl Into<String>) -> PermissionPolicyBuilder {
        PermissionPolicyBuilder::new(name)
    }
}

/// Fluent builder for [`PermissionPolicy`].
#[derive(Debug, Clone)]
pub struct PermissionPolicyBuilder {
    name: String,
    required_scopes: HashMap<String, String>,
    forbidden_categories: Vec<String>,
    forbidden_levels: Vec<String>,
    allow_additional_read_scopes: bool,
    allowed_additional_scopes: Vec<String>,
}

impl PermissionPolicyBuilder {
    /// Creates a new builder initialized with default forbidden categories and levels.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required_scopes: HashMap::new(),
            forbidden_categories: DEFAULT_FORBIDDEN_CATEGORIES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            forbidden_levels: DEFAULT_FORBIDDEN_LEVELS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            allow_additional_read_scopes: false,
            allowed_additional_scopes: Vec::new(),
        }
    }

    /// Adds a required permission scope and expected level.
    #[must_use]
    pub fn require_scope(mut self, scope: impl Into<String>, level: impl Into<String>) -> Self {
        self.required_scopes.insert(
            scope.into().to_ascii_lowercase(),
            level.into().to_ascii_lowercase(),
        );
        self
    }

    /// Adds a forbidden permission category.
    #[must_use]
    pub fn forbid_category(mut self, category: impl Into<String>) -> Self {
        let cat = category.into().to_ascii_lowercase();
        if !self.forbidden_categories.contains(&cat) {
            self.forbidden_categories.push(cat);
        }
        self
    }

    /// Adds a forbidden permission level.
    #[must_use]
    pub fn forbid_level(mut self, level: impl Into<String>) -> Self {
        let lvl = level.into().to_ascii_lowercase();
        if !self.forbidden_levels.contains(&lvl) {
            self.forbidden_levels.push(lvl);
        }
        self
    }

    /// Sets whether additional non-required read-only scopes are permitted.
    #[must_use]
    pub fn allow_additional_read_scopes(mut self, allow: bool) -> Self {
        self.allow_additional_read_scopes = allow;
        self
    }

    /// Adds a specific permitted additional read-only scope.
    #[must_use]
    pub fn allow_additional_scope(mut self, scope: impl Into<String>) -> Self {
        let s = scope.into().to_ascii_lowercase();
        if !self.allowed_additional_scopes.contains(&s) {
            self.allowed_additional_scopes.push(s);
        }
        self
    }

    /// Builds the configured [`PermissionPolicy`].
    #[must_use]
    pub fn build(self) -> PermissionPolicy {
        PermissionPolicy {
            name: self.name,
            required_scopes: self.required_scopes,
            forbidden_categories: self.forbidden_categories,
            forbidden_levels: self.forbidden_levels,
            allow_additional_read_scopes: self.allow_additional_read_scopes,
            allowed_additional_scopes: self.allowed_additional_scopes,
        }
    }
}

/// Result of evaluating a permissions map or installation against a [`PermissionPolicy`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionAuditReport {
    /// True if the permissions comply with all invariants of the policy.
    pub compliant: bool,
    /// Name of the policy used for this audit.
    pub policy_name: String,
    /// The granted permissions map inspected.
    pub granted_permissions: HashMap<String, String>,
    /// List of detected policy violations (empty if compliant).
    pub violations: Vec<PermissionViolation>,
    /// UTC timestamp when the audit was performed.
    pub audited_at: DateTime<Utc>,
}

impl PermissionAuditReport {
    /// Returns true if the inspected permissions are fully compliant.
    #[must_use]
    pub fn is_compliant(&self) -> bool {
        self.compliant
    }

    /// Returns the slice of detected violations.
    #[must_use]
    pub fn violations(&self) -> &[PermissionViolation] {
        &self.violations
    }

    /// Returns a concise human-readable summary string of the audit outcome.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.compliant {
            format!(
                "Permission audit passed [{}]: all {} granted scopes comply with least privilege",
                self.policy_name,
                self.granted_permissions.len()
            )
        } else {
            let details = self
                .violations
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            format!(
                "Permission audit failed [{}]: {} violation(s) detected: {}",
                self.policy_name,
                self.violations.len(),
                details
            )
        }
    }

    /// Formats the audit report as a clean GitHub-Flavored Markdown table with status alerts.
    #[must_use]
    pub fn to_markdown_report(&self) -> String {
        let mut md = String::new();
        md.push_str("### GitHub App Permission Audit Report\n\n");
        md.push_str(&format!("- **Policy:** `{}`\n", self.policy_name));
        md.push_str(&format!(
            "- **Compliance Status:** {}\n",
            if self.compliant {
                "**COMPLIANT** (Least privilege verified)"
            } else {
                "**NON-COMPLIANT** (Rule 05 violation)"
            }
        ));
        md.push_str(&format!(
            "- **Audited At:** {}\n\n",
            self.audited_at.to_rfc3339()
        ));

        if !self.violations.is_empty() {
            md.push_str("> [!CAUTION]\n");
            md.push_str("> The following permission policy violations were detected:\n");
            for violation in &self.violations {
                md.push_str(&format!("> - {violation}\n"));
            }
            md.push('\n');
        } else {
            md.push_str("> [!NOTE]\n");
            md.push_str(
                "> Verified least-privilege permissions: strictly read-only scopes configured.\n\n",
            );
        }

        md.push_str("| Scope | Granted Level | Policy Status |\n");
        md.push_str("| :--- | :--- | :--- |\n");

        // Sort keys for deterministic output
        let mut sorted_keys: Vec<&String> = self.granted_permissions.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let val = &self.granted_permissions[key];
            let is_violating = self.violations.iter().any(|v| match v {
                PermissionViolation::ExcessiveLevel { scope, .. } => scope == key,
                PermissionViolation::ForbiddenScope { scope, .. } => scope == key,
                PermissionViolation::UnexpectedScope { scope, .. } => scope == key,
                _ => false,
            });

            let status_badge = if is_violating {
                "`REJECTED`"
            } else {
                "`PERMITTED`"
            };

            md.push_str(&format!("| `{key}` | `{val}` | {status_badge} |\n"));
        }

        md
    }

    /// Serializes the audit report to JSON.
    ///
    /// # Errors
    /// Returns error if JSON serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Permission verification and auditing engine for GitHub App installations and tokens.
pub struct PermissionAuditor;

impl PermissionAuditor {
    /// Audits a raw permissions map against the provided [`PermissionPolicy`].
    #[must_use]
    pub fn audit_permissions(
        permissions: &HashMap<String, String>,
        policy: &PermissionPolicy,
    ) -> PermissionAuditReport {
        let mut violations = Vec::new();

        // 1. Check for required scopes
        for (req_scope, req_level) in &policy.required_scopes {
            let req_scope_lower = req_scope.to_ascii_lowercase();
            let req_level_lower = req_level.to_ascii_lowercase();

            match permissions.get(&req_scope_lower) {
                None => {
                    violations.push(PermissionViolation::MissingRequired {
                        scope: req_scope.clone(),
                        expected: req_level.clone(),
                    });
                }
                Some(granted) => {
                    let granted_lower = granted.to_ascii_lowercase();
                    if granted_lower != req_level_lower {
                        violations.push(PermissionViolation::ExcessiveLevel {
                            scope: req_scope.clone(),
                            granted: granted.clone(),
                            max_allowed: req_level.clone(),
                        });
                    }
                }
            }
        }

        // 2. Inspect all granted permissions for forbidden categories, forbidden levels, or unexpected scopes
        for (scope, level) in permissions {
            let scope_lower = scope.to_ascii_lowercase();
            let level_lower = level.to_ascii_lowercase();

            // Check forbidden levels across all scopes (e.g. write, admin)
            if policy
                .forbidden_levels
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&level_lower))
            {
                // If not already flagged by required scope check
                if !violations.iter().any(|v| match v {
                    PermissionViolation::ExcessiveLevel { scope: s, .. } => s == scope,
                    _ => false,
                }) {
                    violations.push(PermissionViolation::ExcessiveLevel {
                        scope: scope.clone(),
                        granted: level.clone(),
                        max_allowed: "read".to_string(),
                    });
                }
            }

            // Check forbidden categories
            if policy
                .forbidden_categories
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&scope_lower))
            {
                violations.push(PermissionViolation::ForbiddenScope {
                    scope: scope.clone(),
                    granted: level.clone(),
                    reason: format!(
                        "Permission category '{scope}' presents security/operational risk and is prohibited under Rule 05"
                    ),
                });
            }

            // Check unexpected scopes if additional read scopes are not allowed
            let is_required = policy.required_scopes.contains_key(&scope_lower);
            if !is_required {
                let is_disallowed_additional = !policy.allow_additional_read_scopes
                    || (!policy.allowed_additional_scopes.is_empty()
                        && !policy
                            .allowed_additional_scopes
                            .iter()
                            .any(|a| a.eq_ignore_ascii_case(&scope_lower)));

                if is_disallowed_additional {
                    violations.push(PermissionViolation::UnexpectedScope {
                        scope: scope.clone(),
                        granted: level.clone(),
                    });
                }
            }
        }

        let compliant = violations.is_empty();

        PermissionAuditReport {
            compliant,
            policy_name: policy.name.clone(),
            granted_permissions: permissions.clone(),
            violations,
            audited_at: Utc::now(),
        }
    }

    /// Validates a raw permissions map against the provided [`PermissionPolicy`].
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::PermissionViolation`] if any policy violation is detected.
    pub fn validate_permissions(
        permissions: &HashMap<String, String>,
        policy: &PermissionPolicy,
    ) -> Result<(), GitHubExtractorError> {
        let report = Self::audit_permissions(permissions, policy);
        if report.is_compliant() {
            Ok(())
        } else {
            Err(GitHubExtractorError::PermissionViolation(report.summary()))
        }
    }

    /// Audits a [`GitHubInstallation`] against the provided [`PermissionPolicy`].
    ///
    /// Checks permissions map as well as installation suspension status.
    #[must_use]
    pub fn audit_installation(
        installation: &GitHubInstallation,
        policy: &PermissionPolicy,
    ) -> PermissionAuditReport {
        let mut report = Self::audit_permissions(&installation.permissions, policy);

        // Check if installation is suspended
        if installation.is_suspended() {
            report
                .violations
                .push(PermissionViolation::SuspendedInstallation {
                    installation_id: installation.id,
                });
            report.compliant = false;
        }

        report
    }

    /// Validates a [`GitHubInstallation`] against the provided [`PermissionPolicy`].
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::PermissionViolation`] if the installation is suspended
    /// or if permissions violate policy.
    pub fn validate_installation(
        installation: &GitHubInstallation,
        policy: &PermissionPolicy,
    ) -> Result<(), GitHubExtractorError> {
        let report = Self::audit_installation(installation, policy);
        if report.is_compliant() {
            Ok(())
        } else {
            Err(GitHubExtractorError::PermissionViolation(report.summary()))
        }
    }

    /// Audits a [`GitHubInstallationToken`] against the provided [`PermissionPolicy`].
    #[must_use]
    pub fn audit_installation_token(
        token: &GitHubInstallationToken,
        policy: &PermissionPolicy,
    ) -> PermissionAuditReport {
        Self::audit_permissions(&token.permissions, policy)
    }

    /// Validates a [`GitHubInstallationToken`] against the provided [`PermissionPolicy`].
    ///
    /// # Errors
    /// Returns [`GitHubExtractorError::PermissionViolation`] if token permissions violate policy.
    pub fn validate_installation_token(
        token: &GitHubInstallationToken,
        policy: &PermissionPolicy,
    ) -> Result<(), GitHubExtractorError> {
        let report = Self::audit_installation_token(token, policy);
        if report.is_compliant() {
            Ok(())
        } else {
            Err(GitHubExtractorError::PermissionViolation(report.summary()))
        }
    }
}
