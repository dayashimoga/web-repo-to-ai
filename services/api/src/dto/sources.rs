//! Data Transfer Objects for source URL detection and classification.

use serde::{Deserialize, Serialize};
use url::Url;
use web_repo_to_ai_core::{CoreError, Result, Validatable};

/// Category of source detected from input URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceClassification {
    /// Standard HTML webpage, article, or documentation site.
    WebPage,
    /// Public GitHub repository or repository subtree.
    #[serde(rename = "public_github")]
    PublicGitHub,
    /// Private GitHub repository requiring OAuth/App installation authorization.
    #[serde(rename = "private_github")]
    PrivateGitHub,
    /// Batch URL list or documentation sitemap.
    BatchList,
}

/// Metadata and classification for an inspected URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedSource {
    /// The original input URL as submitted.
    pub url: String,
    /// Categorized source classification.
    pub classification: SourceClassification,
    /// Standardized, canonical URL representation.
    pub normalized_url: String,
    /// GitHub repository owner or organization, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_owner: Option<String>,
    /// GitHub repository name, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    /// Target Git branch, tag, or commit reference, if detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,
    /// Subdirectory or path prefix within repository, if detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
}

/// Request body for classifying one or more source URLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectSourcesRequest {
    /// List of candidate URLs to detect and classify.
    pub urls: Vec<String>,
}

impl Validatable for DetectSourcesRequest {
    fn validate(&self) -> Result<()> {
        if self.urls.is_empty() {
            return Err(CoreError::Validation {
                field: "urls".to_string(),
                reason: "Must provide at least one URL (received empty array)".to_string(),
            });
        }

        if self.urls.len() > 50 {
            return Err(CoreError::Validation {
                field: "urls".to_string(),
                reason: format!(
                    "Maximum allowable batch size is 50 URLs (received {})",
                    self.urls.len()
                ),
            });
        }

        for (index, raw_url) in self.urls.iter().enumerate() {
            let trimmed = raw_url.trim();
            if trimmed.is_empty() {
                return Err(CoreError::Validation {
                    field: "urls".to_string(),
                    reason: format!("URL at index {index} cannot be empty or whitespace"),
                });
            }

            if trimmed.len() > 2048 {
                return Err(CoreError::Validation {
                    field: "urls".to_string(),
                    reason: format!(
                        "URL at index {index} exceeds maximum allowable length of 2048 characters (length: {})",
                        trimmed.len()
                    ),
                });
            }

            let parsed = Url::parse(trimmed).map_err(|err| CoreError::Validation {
                field: "urls".to_string(),
                reason: format!("URL at index {index} ('{trimmed}') is malformed: {err}"),
            })?;

            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err(CoreError::UnsupportedUrlScheme(format!(
                    "URL at index {index} has unsupported scheme '{}': only http and https are permitted",
                    parsed.scheme()
                )));
            }

            if parsed.host_str().is_none() {
                return Err(CoreError::Validation {
                    field: "urls".to_string(),
                    reason: format!("URL at index {index} ('{trimmed}') is missing a host name"),
                });
            }
        }

        Ok(())
    }
}

/// Response payload containing classified sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectSourcesResponse {
    /// Detected source details for each submitted URL.
    pub sources: Vec<DetectedSource>,
    /// Total count of inspected sources.
    pub total: usize,
}

/// Inspects and classifies a validated URL string.
#[must_use]
pub fn classify_source_url(raw_url: &str) -> DetectedSource {
    let trimmed = raw_url.trim();
    let Ok(parsed) = Url::parse(trimmed) else {
        return DetectedSource {
            url: trimmed.to_string(),
            classification: SourceClassification::WebPage,
            normalized_url: trimmed.to_string(),
            repo_owner: None,
            repo_name: None,
            git_ref: None,
            subpath: None,
        };
    };

    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();

    if host == "github.com" || host.ends_with(".github.com") {
        let segments: Vec<&str> = parsed
            .path_segments()
            .map(|s| s.filter(|seg| !seg.is_empty()).collect())
            .unwrap_or_default();

        if segments.len() >= 2 {
            let owner = segments[0].to_string();
            let repo = segments[1].trim_end_matches(".git").to_string();

            let (git_ref, subpath) =
                if segments.len() >= 4 && (segments[2] == "tree" || segments[2] == "blob") {
                    let r = segments[3].to_string();
                    let sp = if segments.len() > 4 {
                        Some(segments[4..].join("/"))
                    } else {
                        None
                    };
                    (Some(r), sp)
                } else {
                    (None, None)
                };

            let normalized = format!("https://github.com/{owner}/{repo}");

            return DetectedSource {
                url: trimmed.to_string(),
                classification: SourceClassification::PublicGitHub,
                normalized_url: normalized,
                repo_owner: Some(owner),
                repo_name: Some(repo),
                git_ref,
                subpath,
            };
        }
    }

    DetectedSource {
        url: trimmed.to_string(),
        classification: SourceClassification::WebPage,
        normalized_url: parsed.to_string(),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    }
}
