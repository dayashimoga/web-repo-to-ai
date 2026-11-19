//! # Crawl-Depth and Domain Restrictions Engine
//!
//! Provides strict boundary controls, traversal depth limits, path pattern filters,
//! query string controls, and budget quotas during automated website crawls.

use std::collections::{HashMap, HashSet};

use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use crate::elements::link::ASSET_EXTENSIONS;
use crate::error::WebExtractorError;

/// Default maximum allowable traversal depth from seed URL (seed = 0).
pub const DEFAULT_MAX_CRAWL_DEPTH: usize = 3;

/// Default global ceiling on total pages crawled across a single session.
pub const DEFAULT_MAX_CRAWL_PAGES: usize = 500;

/// Absolute maximum depth permissible to protect against unbounded stack or queue overflow.
pub const ABSOLUTE_MAX_CRAWL_DEPTH: usize = 20;

/// Domain and hostname boundary restrictions governing a website crawl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CrawlScope {
    /// Restricts crawl strictly to the exact host of the seed URL (e.g. `example.com` only;
    /// subdomains like `docs.example.com` are rejected).
    #[default]
    ExactHost,
    /// Permits the seed host and any subordinate subdomain (e.g. `example.com`,
    /// `docs.example.com`, `api.example.com`).
    Subdomains,
    /// Restricts crawl strictly to the seed host AND paths starting with the seed URL's path prefix
    /// (e.g. `https://example.com/docs/v2/` only crawls paths beginning with `/docs/v2/`).
    PathPrefix,
    /// Permits the seed host plus an explicit allowlist of domain patterns or exact hosts
    /// (e.g. `["example.com", "*.example.com", "cdn.mycompany.org"]`).
    DomainAllowlist(Vec<String>),
}

/// Rules governing query string parameters during website crawls to prevent crawler traps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QueryParamPolicy {
    /// Strips all query parameters from candidate URLs; pages are crawled without query strings.
    #[default]
    StripAll,
    /// Allows only query parameters matching approved keys in the allowlist; unapproved keys are stripped.
    AllowList(HashSet<String>),
    /// Disallows specified query parameters; blacklisted keys are stripped.
    BlockList(HashSet<String>),
    /// Permits all query strings intact (caution: may cause query parameter combinatorial explosion).
    AllowAll,
}

/// Complete policy configuration enforcing depth, domain boundaries, path patterns, and budgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRestrictions {
    /// Domain/hostname scoping rules.
    pub scope: CrawlScope,
    /// Maximum traversal depth from seed URL (seed = 0).
    pub max_depth: usize,
    /// Global limit on total pages crawled across the entire crawl session.
    pub max_pages: usize,
    /// Optional limit on pages crawled at any individual depth tier.
    pub max_pages_per_depth: Option<usize>,
    /// Optional limit on pages crawled per unique hostname.
    pub max_pages_per_host: Option<usize>,
    /// Path patterns that candidate URLs MUST match to be crawled (if non-empty).
    pub include_patterns: Vec<String>,
    /// Path patterns that immediately disqualify a URL from being crawled.
    pub exclude_patterns: Vec<String>,
    /// Disallowed file extensions (default: media/binary assets like `.pdf`, `.zip`, etc.).
    pub exclude_extensions: HashSet<String>,
    /// Policy governing query strings to prevent crawler loops.
    pub query_policy: QueryParamPolicy,
    /// Whether to require candidate links to match the seed URL's network port.
    pub enforce_same_port: bool,
    /// Whether to require candidate links to match the seed URL's scheme (`http`/`https`).
    pub enforce_same_scheme: bool,
    /// Zero-trust SSRF policy.
    pub ssrf_policy: UrlSecurityPolicy,
}

impl Default for CrawlRestrictions {
    fn default() -> Self {
        let mut extensions = HashSet::new();
        for ext in ASSET_EXTENSIONS {
            extensions.insert(ext.to_string());
        }

        Self {
            scope: CrawlScope::default(),
            max_depth: DEFAULT_MAX_CRAWL_DEPTH,
            max_pages: DEFAULT_MAX_CRAWL_PAGES,
            max_pages_per_depth: None,
            max_pages_per_host: None,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            exclude_extensions: extensions,
            query_policy: QueryParamPolicy::default(),
            enforce_same_port: true,
            enforce_same_scheme: true,
            ssrf_policy: UrlSecurityPolicy::default(),
        }
    }
}

impl CrawlRestrictions {
    /// Sets the domain scope policy.
    #[must_use]
    pub fn with_scope(mut self, scope: CrawlScope) -> Self {
        self.scope = scope;
        self
    }

    /// Sets the maximum allowable crawl depth (clamped to `ABSOLUTE_MAX_CRAWL_DEPTH`).
    #[must_use]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth.min(ABSOLUTE_MAX_CRAWL_DEPTH);
        self
    }

    /// Sets the maximum allowable total pages to crawl.
    #[must_use]
    pub fn with_max_pages(mut self, max: usize) -> Self {
        self.max_pages = max.max(1);
        self
    }

    /// Sets a ceiling on pages crawled per depth tier.
    #[must_use]
    pub fn with_max_pages_per_depth(mut self, max: usize) -> Self {
        self.max_pages_per_depth = Some(max.max(1));
        self
    }

    /// Sets a ceiling on pages crawled per host.
    #[must_use]
    pub fn with_max_pages_per_host(mut self, max: usize) -> Self {
        self.max_pages_per_host = Some(max.max(1));
        self
    }

    /// Adds a path pattern that URLs must match to be crawled.
    #[must_use]
    pub fn with_include_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Adds a path pattern that immediately disqualifies a URL.
    #[must_use]
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Sets the query parameter policy.
    #[must_use]
    pub fn with_query_policy(mut self, policy: QueryParamPolicy) -> Self {
        self.query_policy = policy;
        self
    }

    /// Configures whether port matching is strictly enforced.
    #[must_use]
    pub fn with_enforce_same_port(mut self, enforce: bool) -> Self {
        self.enforce_same_port = enforce;
        self
    }

    /// Configures whether scheme matching is strictly enforced.
    #[must_use]
    pub fn with_enforce_same_scheme(mut self, enforce: bool) -> Self {
        self.enforce_same_scheme = enforce;
        self
    }

    /// Overrides the URL security SSRF policy.
    #[must_use]
    pub fn with_ssrf_policy(mut self, policy: UrlSecurityPolicy) -> Self {
        self.ssrf_policy = policy;
        self
    }
}

/// Specific reason explaining why a candidate URL was rejected by the crawler boundary engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrawlRejectionReason {
    /// Depth exceeds `max_depth`.
    ExceedsMaxDepth {
        /// Attempted depth.
        depth: usize,
        /// Maximum allowed depth.
        max_depth: usize,
    },
    /// Candidate hostname does not match the seed host under `ExactHost` scope.
    ExternalHost {
        /// Candidate host.
        host: String,
        /// Seed host.
        seed_host: String,
    },
    /// Candidate hostname is a subdomain, but `ExactHost` was required.
    SubdomainDisallowed {
        /// Candidate host.
        host: String,
        /// Seed host.
        seed_host: String,
    },
    /// Candidate domain not found in custom `DomainAllowlist`.
    DomainNotInAllowlist {
        /// Candidate host.
        host: String,
    },
    /// URL path does not begin with the seed path prefix under `PathPrefix` scope.
    PathPrefixMismatch {
        /// Candidate path.
        path: String,
        /// Required prefix.
        required_prefix: String,
    },
    /// URL path matched an explicit exclude pattern.
    ExcludedByPattern {
        /// Pattern that matched.
        pattern: String,
        /// Candidate path.
        path: String,
    },
    /// URL path failed to match any pattern in `include_patterns`.
    NotMatchedByIncludePatterns {
        /// Candidate path.
        path: String,
    },
    /// URL path ends with a disallowed static asset extension.
    DisallowedAssetExtension {
        /// Disallowed file extension.
        extension: String,
    },
    /// URL scheme does not match seed or is not http/https.
    DisallowedScheme {
        /// Offending scheme.
        scheme: String,
    },
    /// URL port differs from seed URL port.
    DisallowedPort {
        /// Candidate port.
        port: Option<u16>,
        /// Seed port.
        seed_port: Option<u16>,
    },
    /// Global total page budget has been exhausted.
    TotalPageBudgetExceeded {
        /// Number of crawled pages.
        count: usize,
        /// Total limit.
        limit: usize,
    },
    /// Page quota for the specific depth level has been exhausted.
    DepthBudgetExceeded {
        /// Depth level.
        depth: usize,
        /// Number of pages at this depth.
        count: usize,
        /// Limit for this depth.
        limit: usize,
    },
    /// Page quota for the specific host has been exhausted.
    HostBudgetExceeded {
        /// Hostname.
        host: String,
        /// Number of pages for this host.
        count: usize,
        /// Limit for this host.
        limit: usize,
    },
    /// Zero-Trust SSRF security policy violation.
    SsrfViolation {
        /// Reason for SSRF block.
        reason: String,
    },
}

impl std::fmt::Display for CrawlRejectionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExceedsMaxDepth { depth, max_depth } => {
                write!(f, "Exceeds max crawl depth ({depth} > {max_depth})")
            }
            Self::ExternalHost { host, seed_host } => {
                write!(
                    f,
                    "External host '{host}' does not match seed '{seed_host}'"
                )
            }
            Self::SubdomainDisallowed { host, seed_host } => {
                write!(
                    f,
                    "Subdomain '{host}' disallowed for exact host '{seed_host}'"
                )
            }
            Self::DomainNotInAllowlist { host } => {
                write!(f, "Host '{host}' not present in domain allowlist")
            }
            Self::PathPrefixMismatch {
                path,
                required_prefix,
            } => {
                write!(
                    f,
                    "Path '{path}' does not match required prefix '{required_prefix}'"
                )
            }
            Self::ExcludedByPattern { pattern, path } => {
                write!(f, "Path '{path}' excluded by pattern '{pattern}'")
            }
            Self::NotMatchedByIncludePatterns { path } => {
                write!(
                    f,
                    "Path '{path}' does not match any required include patterns"
                )
            }
            Self::DisallowedAssetExtension { extension } => {
                write!(f, "Path targets downloadable asset extension '{extension}'")
            }
            Self::DisallowedScheme { scheme } => {
                write!(f, "Scheme '{scheme}' is not allowed for crawling")
            }
            Self::DisallowedPort { port, seed_port } => {
                write!(f, "Port {port:?} does not match seed port {seed_port:?}")
            }
            Self::TotalPageBudgetExceeded { count, limit } => {
                write!(f, "Total page budget exceeded ({count} >= {limit})")
            }
            Self::DepthBudgetExceeded {
                depth,
                count,
                limit,
            } => {
                write!(f, "Depth {depth} budget exceeded ({count} >= {limit})")
            }
            Self::HostBudgetExceeded { host, count, limit } => {
                write!(f, "Host '{host}' budget exceeded ({count} >= {limit})")
            }
            Self::SsrfViolation { reason } => {
                write!(f, "SSRF security violation: {reason}")
            }
        }
    }
}

/// Outcome of evaluating a candidate URL against crawl restrictions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrawlDecision {
    /// URL is approved for crawling.
    Allowed {
        /// Normalized URL approved for crawling.
        url: Url,
        /// Traversal depth assigned to this URL.
        depth: usize,
    },
    /// URL is rejected by crawl restrictions.
    Rejected {
        /// Candidate URL that was inspected.
        url: Url,
        /// Reason for rejection.
        reason: CrawlRejectionReason,
    },
}

impl CrawlDecision {
    /// Returns `true` if the candidate was approved.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    /// Returns `true` if the candidate was rejected.
    #[must_use]
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected { .. })
    }

    /// Returns the approved [`Url`] if allowed.
    #[must_use]
    pub fn allowed_url(&self) -> Option<&Url> {
        match self {
            Self::Allowed { url, .. } => Some(url),
            Self::Rejected { .. } => None,
        }
    }

    /// Returns the assigned depth if allowed.
    #[must_use]
    pub fn allowed_depth(&self) -> Option<usize> {
        match self {
            Self::Allowed { depth, .. } => Some(*depth),
            Self::Rejected { .. } => None,
        }
    }
}

/// Metrics summarizing current crawl session consumption and remaining limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrawlBudgetStats {
    /// Total pages recorded as crawled.
    pub total_crawled: usize,
    /// Maximum global page quota.
    pub max_pages: usize,
    /// Remaining pages before session exhaustion.
    pub remaining_pages: usize,
    /// Breakdown of page counts by depth tier.
    pub pages_per_depth: HashMap<usize, usize>,
    /// Breakdown of page counts by hostname.
    pub pages_per_host: HashMap<String, usize>,
    /// Whether the global page limit has been reached.
    pub is_exhausted: bool,
}

/// Stateful evaluator that enforces crawl restrictions and tracks depth/page budget consumption.
#[derive(Debug, Clone)]
pub struct CrawlRestrictionsEvaluator {
    /// Seed URL that started the crawl session.
    seed_url: Url,
    /// Seed host in lowercase.
    seed_host: String,
    /// Seed path prefix (normalized).
    seed_path_prefix: String,
    /// Active policy restrictions.
    restrictions: CrawlRestrictions,
    /// Compiled inclusion regexes.
    include_regexes: Vec<(String, Regex)>,
    /// Compiled exclusion regexes.
    exclude_regexes: Vec<(String, Regex)>,
    /// Total pages crawled so far.
    total_crawled: usize,
    /// Count of crawled pages indexed by depth.
    pages_per_depth: HashMap<usize, usize>,
    /// Count of crawled pages indexed by hostname.
    pages_per_host: HashMap<String, usize>,
    /// Discovered URL registry tracking the minimum depth at which each URL has been seen.
    discovered_urls: HashMap<String, usize>,
}

impl CrawlRestrictionsEvaluator {
    /// Constructs a new evaluator bound to a seed URL and policy restrictions.
    pub fn new(seed_url: Url, restrictions: CrawlRestrictions) -> Result<Self, WebExtractorError> {
        let seed_host = match seed_url.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => {
                return Err(WebExtractorError::Sitemap(format!(
                    "Seed URL '{seed_url}' has no host"
                )))
            }
        };

        // Determine path prefix for PathPrefix scope: take directory of seed URL
        let seed_path_prefix = calculate_path_prefix(seed_url.path());

        let mut include_regexes = Vec::new();
        for pat in &restrictions.include_patterns {
            let re = compile_pattern_to_regex(pat)?;
            include_regexes.push((pat.clone(), re));
        }

        let mut exclude_regexes = Vec::new();
        for pat in &restrictions.exclude_patterns {
            let re = compile_pattern_to_regex(pat)?;
            exclude_regexes.push((pat.clone(), re));
        }

        let mut evaluator = Self {
            seed_url: seed_url.clone(),
            seed_host,
            seed_path_prefix,
            restrictions,
            include_regexes,
            exclude_regexes,
            total_crawled: 0,
            pages_per_depth: HashMap::new(),
            pages_per_host: HashMap::new(),
            discovered_urls: HashMap::new(),
        };

        // Seed URL is implicitly discovered at depth 0
        let seed_key = normalize_key_url(&seed_url);
        evaluator.discovered_urls.insert(seed_key, 0);

        Ok(evaluator)
    }

    /// Returns a reference to the active restrictions configuration.
    #[must_use]
    pub fn restrictions(&self) -> &CrawlRestrictions {
        &self.restrictions
    }

    /// Returns a reference to the seed URL.
    #[must_use]
    pub fn seed_url(&self) -> &Url {
        &self.seed_url
    }

    /// Evaluates a candidate URL discovered at `parent_depth` against all restrictions.
    pub fn evaluate_candidate(
        &mut self,
        raw_candidate: &Url,
        parent_depth: usize,
    ) -> CrawlDecision {
        let candidate_depth = parent_depth + 1;

        // 1. Depth limit check
        if candidate_depth > self.restrictions.max_depth {
            return CrawlDecision::Rejected {
                url: raw_candidate.clone(),
                reason: CrawlRejectionReason::ExceedsMaxDepth {
                    depth: candidate_depth,
                    max_depth: self.restrictions.max_depth,
                },
            };
        }

        // 2. Global total page budget check
        if self.total_crawled >= self.restrictions.max_pages {
            return CrawlDecision::Rejected {
                url: raw_candidate.clone(),
                reason: CrawlRejectionReason::TotalPageBudgetExceeded {
                    count: self.total_crawled,
                    limit: self.restrictions.max_pages,
                },
            };
        }

        // 3. Per-depth page budget check
        if let Some(depth_limit) = self.restrictions.max_pages_per_depth {
            let current_at_depth = self
                .pages_per_depth
                .get(&candidate_depth)
                .copied()
                .unwrap_or(0);
            if current_at_depth >= depth_limit {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::DepthBudgetExceeded {
                        depth: candidate_depth,
                        count: current_at_depth,
                        limit: depth_limit,
                    },
                };
            }
        }

        // 4. Scheme validation
        let scheme = raw_candidate.scheme().to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return CrawlDecision::Rejected {
                url: raw_candidate.clone(),
                reason: CrawlRejectionReason::DisallowedScheme { scheme },
            };
        }
        if self.restrictions.enforce_same_scheme
            && scheme != self.seed_url.scheme().to_ascii_lowercase()
        {
            return CrawlDecision::Rejected {
                url: raw_candidate.clone(),
                reason: CrawlRejectionReason::DisallowedScheme { scheme },
            };
        }

        // 5. Port validation
        if self.restrictions.enforce_same_port {
            let cand_port = raw_candidate.port_or_known_default();
            let seed_port = self.seed_url.port_or_known_default();
            if cand_port != seed_port {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::DisallowedPort {
                        port: raw_candidate.port(),
                        seed_port: self.seed_url.port(),
                    },
                };
            }
        }

        // 6. Host & Domain boundary evaluation
        let cand_host = match raw_candidate.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::ExternalHost {
                        host: String::new(),
                        seed_host: self.seed_host.clone(),
                    },
                };
            }
        };

        match &self.restrictions.scope {
            CrawlScope::ExactHost => {
                if cand_host != self.seed_host {
                    let dot_seed = format!(".{}", self.seed_host);
                    if cand_host.ends_with(&dot_seed) {
                        return CrawlDecision::Rejected {
                            url: raw_candidate.clone(),
                            reason: CrawlRejectionReason::SubdomainDisallowed {
                                host: cand_host,
                                seed_host: self.seed_host.clone(),
                            },
                        };
                    }
                    return CrawlDecision::Rejected {
                        url: raw_candidate.clone(),
                        reason: CrawlRejectionReason::ExternalHost {
                            host: cand_host,
                            seed_host: self.seed_host.clone(),
                        },
                    };
                }
            }
            CrawlScope::Subdomains => {
                let dot_seed = format!(".{}", self.seed_host);
                if cand_host != self.seed_host && !cand_host.ends_with(&dot_seed) {
                    return CrawlDecision::Rejected {
                        url: raw_candidate.clone(),
                        reason: CrawlRejectionReason::ExternalHost {
                            host: cand_host,
                            seed_host: self.seed_host.clone(),
                        },
                    };
                }
            }
            CrawlScope::PathPrefix => {
                if cand_host != self.seed_host {
                    return CrawlDecision::Rejected {
                        url: raw_candidate.clone(),
                        reason: CrawlRejectionReason::ExternalHost {
                            host: cand_host,
                            seed_host: self.seed_host.clone(),
                        },
                    };
                }
                let cand_path = raw_candidate.path();
                if !cand_path.starts_with(&self.seed_path_prefix) {
                    return CrawlDecision::Rejected {
                        url: raw_candidate.clone(),
                        reason: CrawlRejectionReason::PathPrefixMismatch {
                            path: cand_path.to_string(),
                            required_prefix: self.seed_path_prefix.clone(),
                        },
                    };
                }
            }
            CrawlScope::DomainAllowlist(allowed_domains) => {
                let is_allowed = allowed_domains
                    .iter()
                    .any(|pattern| match_domain_pattern(pattern, &cand_host));
                if !is_allowed && cand_host != self.seed_host {
                    return CrawlDecision::Rejected {
                        url: raw_candidate.clone(),
                        reason: CrawlRejectionReason::DomainNotInAllowlist { host: cand_host },
                    };
                }
            }
        }

        // 7. Per-host budget check
        if let Some(host_limit) = self.restrictions.max_pages_per_host {
            let current_for_host = self.pages_per_host.get(&cand_host).copied().unwrap_or(0);
            if current_for_host >= host_limit {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::HostBudgetExceeded {
                        host: cand_host,
                        count: current_for_host,
                        limit: host_limit,
                    },
                };
            }
        }

        // 8. Asset extension check
        let cand_path = raw_candidate.path();
        let cand_path_lower = cand_path.to_ascii_lowercase();
        for ext in &self.restrictions.exclude_extensions {
            if cand_path_lower.ends_with(ext) {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::DisallowedAssetExtension {
                        extension: ext.clone(),
                    },
                };
            }
        }

        // 9. Exclude pattern checks
        for (pattern_str, re) in &self.exclude_regexes {
            if re.is_match(cand_path) {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::ExcludedByPattern {
                        pattern: pattern_str.clone(),
                        path: cand_path.to_string(),
                    },
                };
            }
        }

        // 10. Include pattern checks (if any inclusion patterns configured)
        if !self.include_regexes.is_empty() {
            let matches_include = self
                .include_regexes
                .iter()
                .any(|(_, re)| re.is_match(cand_path));
            if !matches_include {
                return CrawlDecision::Rejected {
                    url: raw_candidate.clone(),
                    reason: CrawlRejectionReason::NotMatchedByIncludePatterns {
                        path: cand_path.to_string(),
                    },
                };
            }
        }

        // 11. Query parameter policy normalization
        let mut normalized_url = raw_candidate.clone();
        normalized_url.set_fragment(None);

        match &self.restrictions.query_policy {
            QueryParamPolicy::StripAll => {
                normalized_url.set_query(None);
            }
            QueryParamPolicy::AllowList(whitelist) => {
                if normalized_url.query().is_some() {
                    let filtered: Vec<(String, String)> = normalized_url
                        .query_pairs()
                        .filter(|(k, _)| whitelist.contains(k.as_ref()))
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();
                    if filtered.is_empty() {
                        normalized_url.set_query(None);
                    } else {
                        let mut ser = url::form_urlencoded::Serializer::new(String::new());
                        for (k, v) in filtered {
                            ser.append_pair(&k, &v);
                        }
                        normalized_url.set_query(Some(&ser.finish()));
                    }
                }
            }
            QueryParamPolicy::BlockList(blacklist) => {
                if normalized_url.query().is_some() {
                    let filtered: Vec<(String, String)> = normalized_url
                        .query_pairs()
                        .filter(|(k, _)| !blacklist.contains(k.as_ref()))
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();
                    if filtered.is_empty() {
                        normalized_url.set_query(None);
                    } else {
                        let mut ser = url::form_urlencoded::Serializer::new(String::new());
                        for (k, v) in filtered {
                            ser.append_pair(&k, &v);
                        }
                        normalized_url.set_query(Some(&ser.finish()));
                    }
                }
            }
            QueryParamPolicy::AllowAll => {}
        }

        // 12. SSRF Security Policy Validation
        if let Err(e) = ValidatedUrl::parse(normalized_url.as_str(), &self.restrictions.ssrf_policy)
        {
            return CrawlDecision::Rejected {
                url: normalized_url,
                reason: CrawlRejectionReason::SsrfViolation {
                    reason: e.to_string(),
                },
            };
        }

        // 13. Discovered URL tracking (record minimum seen depth)
        let key = normalize_key_url(&normalized_url);
        let effective_depth = if let Some(&existing_depth) = self.discovered_urls.get(&key) {
            if candidate_depth < existing_depth {
                self.discovered_urls.insert(key, candidate_depth);
                candidate_depth
            } else {
                existing_depth
            }
        } else {
            self.discovered_urls.insert(key, candidate_depth);
            candidate_depth
        };

        CrawlDecision::Allowed {
            url: normalized_url,
            depth: effective_depth,
        }
    }

    /// Records that a page was successfully acquired, updating depth, host, and total budget statistics.
    pub fn record_page_crawled(
        &mut self,
        url: &Url,
        depth: usize,
    ) -> Result<(), CrawlRejectionReason> {
        if self.total_crawled >= self.restrictions.max_pages {
            return Err(CrawlRejectionReason::TotalPageBudgetExceeded {
                count: self.total_crawled,
                limit: self.restrictions.max_pages,
            });
        }

        if let Some(depth_limit) = self.restrictions.max_pages_per_depth {
            let current = self.pages_per_depth.get(&depth).copied().unwrap_or(0);
            if current >= depth_limit {
                return Err(CrawlRejectionReason::DepthBudgetExceeded {
                    depth,
                    count: current,
                    limit: depth_limit,
                });
            }
        }

        let host = url
            .host_str()
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| self.seed_host.clone());

        if let Some(host_limit) = self.restrictions.max_pages_per_host {
            let current = self.pages_per_host.get(&host).copied().unwrap_or(0);
            if current >= host_limit {
                return Err(CrawlRejectionReason::HostBudgetExceeded {
                    host,
                    count: current,
                    limit: host_limit,
                });
            }
        }

        self.total_crawled += 1;
        *self.pages_per_depth.entry(depth).or_insert(0) += 1;
        *self.pages_per_host.entry(host).or_insert(0) += 1;

        let key = normalize_key_url(url);
        self.discovered_urls.entry(key).or_insert(depth);

        Ok(())
    }

    /// Returns aggregate metrics summarizing crawl consumption against configured quotas.
    #[must_use]
    pub fn stats(&self) -> CrawlBudgetStats {
        let remaining_pages = self
            .restrictions
            .max_pages
            .saturating_sub(self.total_crawled);
        let is_exhausted = self.total_crawled >= self.restrictions.max_pages;

        CrawlBudgetStats {
            total_crawled: self.total_crawled,
            max_pages: self.restrictions.max_pages,
            remaining_pages,
            pages_per_depth: self.pages_per_depth.clone(),
            pages_per_host: self.pages_per_host.clone(),
            is_exhausted,
        }
    }

    /// Resets runtime state while retaining restrictions and seed URL configuration.
    pub fn reset(&mut self) {
        self.total_crawled = 0;
        self.pages_per_depth.clear();
        self.pages_per_host.clear();
        self.discovered_urls.clear();
        let seed_key = normalize_key_url(&self.seed_url);
        self.discovered_urls.insert(seed_key, 0);
    }
}

/// Normalizes a URL to a stable lookup key.
fn normalize_key_url(url: &Url) -> String {
    let mut clean = url.clone();
    clean.set_fragment(None);
    clean.as_str().to_string()
}

/// Computes the directory path prefix from a URL path.
fn calculate_path_prefix(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    if path.ends_with('/') {
        return path.to_string();
    }

    match path.rfind('/') {
        Some(idx) => path[..=idx].to_string(),
        None => "/".to_string(),
    }
}

/// Compiles a path glob or regex pattern into a compiled [`Regex`].
fn compile_pattern_to_regex(pattern: &str) -> Result<Regex, WebExtractorError> {
    let trimmed = pattern.trim();
    if let Some(raw_re) = trimmed.strip_prefix("regex:") {
        Regex::new(raw_re).map_err(|e| {
            WebExtractorError::Sitemap(format!("Invalid regex pattern '{raw_re}': {e}"))
        })
    } else if trimmed.starts_with('^') {
        Regex::new(trimmed).map_err(|e| {
            WebExtractorError::Sitemap(format!("Invalid regex pattern '{trimmed}': {e}"))
        })
    } else {
        // Convert glob expression into standard regex
        let re_str = glob_to_regex_string(trimmed);
        Regex::new(&re_str).map_err(|e| {
            WebExtractorError::Sitemap(format!("Invalid glob pattern '{trimmed}': {e}"))
        })
    }
}

/// Translates a glob pattern (`*`, `**`, `?`) into a regex string.
fn glob_to_regex_string(glob: &str) -> String {
    let glob_str = glob.trim();
    // If pattern does not start with '/' and does not start with '**', allow matching at any directory depth
    let normalized = if !glob_str.starts_with('/') && !glob_str.starts_with("**") {
        if glob_str.starts_with('*') {
            format!("*{glob_str}") // e.g. *.html -> **.html
        } else {
            format!("**/{glob_str}") // e.g. logout -> **/logout
        }
    } else {
        glob_str.to_string()
    };

    let mut regex = String::with_capacity(normalized.len() * 2 + 10);
    regex.push('^');

    let chars: Vec<char> = normalized.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    // `**` recursive wildcard
                    regex.push_str(".*");
                    i += 2;
                    // Skip optional trailing slash in `**/` to match directory root
                    if i < chars.len() && chars[i] == '/' {
                        regex.push_str("(?:/.*)?");
                        i += 1;
                    }
                } else {
                    // `*` single segment wildcard: matches characters except '/'
                    regex.push_str("[^/]*");
                    i += 1;
                }
            }
            '?' => {
                regex.push_str("[^/]");
                i += 1;
            }
            c @ ('.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '\\' | '|') => {
                regex.push('\\');
                regex.push(c);
                i += 1;
            }
            c => {
                regex.push(c);
                i += 1;
            }
        }
    }

    regex.push('$');
    regex
}

/// Matches a candidate hostname against an allowlist pattern (`example.com` or `*.example.com`).
fn match_domain_pattern(pattern: &str, candidate_host: &str) -> bool {
    let p_lower = pattern.trim().to_ascii_lowercase();
    let c_lower = candidate_host.trim().to_ascii_lowercase();

    if p_lower == c_lower {
        return true;
    }

    if let Some(base) = p_lower.strip_prefix("*.") {
        if c_lower == base || c_lower.ends_with(&format!(".{base}")) {
            return true;
        }
    }

    false
}
