//! # Sitemap Discovery Orchestrator
//!
//! Orchestrates automated sitemap discovery through `robots.txt` inspections, well-known endpoint
//! probing, recursive `<sitemapindex>` processing, SSRF security validation, cross-domain
//! filtering, and deterministic URL deduplication.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use url::Url;
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy, ValidatedUrl};

use super::model::{DiscoveredSitemapReport, SitemapDiscoveryConfig, SitemapEntry, SitemapType};
use super::parser::parse_sitemap;
use super::robots::parse_robots_txt;
use crate::error::WebExtractorError;
use crate::http::WebFetcher;
use crate::url::WebTarget;

/// Common well-known candidate paths checked when `robots.txt` does not declare any sitemaps.
pub const WELL_KNOWN_SITEMAP_PATHS: &[&str] = &[
    "/sitemap.xml",
    "/sitemap_index.xml",
    "/sitemap/sitemap.xml",
    "/sitemap.xml.gz",
    "/sitemap.txt",
];

/// Automated sitemap discovery engine.
#[derive(Debug, Clone)]
pub struct SitemapDiscoverer {
    /// URL security policy enforcing scheme and SSRF constraints.
    policy: UrlSecurityPolicy,
    /// Secure web content fetcher.
    fetcher: Arc<WebFetcher>,
    /// Discovery limits and behavior configuration.
    config: SitemapDiscoveryConfig,
}

impl SitemapDiscoverer {
    /// Constructs a new `SitemapDiscoverer` with default configuration and standard `WebFetcher`.
    #[must_use]
    pub fn new(policy: UrlSecurityPolicy, dns_resolver: SecureDnsResolver) -> Self {
        let fetcher = Arc::new(WebFetcher::new(policy.clone(), dns_resolver));
        Self {
            policy,
            fetcher,
            config: SitemapDiscoveryConfig::default(),
        }
    }

    /// Sets a custom `WebFetcher` (e.g. configured with `MockHttpTransport` in tests).
    #[must_use]
    pub fn with_fetcher(mut self, fetcher: Arc<WebFetcher>) -> Self {
        self.fetcher = fetcher;
        self
    }

    /// Overrides discovery configuration.
    #[must_use]
    pub fn with_config(mut self, config: SitemapDiscoveryConfig) -> Self {
        self.config = config;
        self
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &SitemapDiscoveryConfig {
        &self.config
    }

    /// Discovers sitemaps and extracts URLs for a target website.
    ///
    /// 1. Determines site origin/root from `target_url`.
    /// 2. If target URL is an explicit sitemap file, processes it directly.
    /// 3. Otherwise queries `robots.txt` for `Sitemap:` directives.
    /// 4. If none found, probes well-known candidate paths (`/sitemap.xml`, etc.).
    /// 5. Recursively expands child sitemaps up to `max_index_depth`.
    /// 6. Enforces domain/host boundaries, SSRF rules, and maximum URL limits.
    pub async fn discover(
        &self,
        target_url_str: &str,
    ) -> Result<DiscoveredSitemapReport, WebExtractorError> {
        let validated_target =
            ValidatedUrl::parse(target_url_str, &self.policy).map_err(WebExtractorError::from)?;
        let target_url = validated_target.url().clone();
        let target_host = target_url
            .host_str()
            .ok_or_else(|| WebExtractorError::Sitemap("Target URL has no host".to_string()))?
            .to_string();

        let mut sitemap_candidates = Vec::new();
        let mut warnings = Vec::new();

        // 1. Check if the target URL is already a direct sitemap URL
        if is_likely_sitemap_url(&target_url) {
            sitemap_candidates.push(target_url.clone());
        } else {
            let root_url = get_origin_root(&target_url)?;

            // 2. Query robots.txt if enabled
            if self.config.check_robots_txt {
                if let Ok(robots_url) = root_url.join("/robots.txt") {
                    if let Ok(val_robots) = ValidatedUrl::parse(robots_url.as_str(), &self.policy) {
                        let target = WebTarget::new(val_robots);
                        match self.fetcher.fetch(&target).await {
                            Ok(resp) if resp.status == 200 => {
                                let body_text = String::from_utf8_lossy(&resp.body);
                                let declared = parse_robots_txt(&body_text, &root_url);
                                for s in declared {
                                    if !sitemap_candidates.contains(&s) {
                                        sitemap_candidates.push(s);
                                    }
                                }
                            }
                            Ok(resp) => {
                                warnings.push(format!(
                                    "robots.txt returned HTTP {} from {}",
                                    resp.status, robots_url
                                ));
                            }
                            Err(e) => {
                                warnings.push(format!("Failed to fetch robots.txt: {e}"));
                            }
                        }
                    }
                }
            }

            // 3. If no sitemaps found via robots.txt, probe well-known candidate paths
            if sitemap_candidates.is_empty() && self.config.probe_well_known {
                for path in WELL_KNOWN_SITEMAP_PATHS {
                    if let Ok(candidate_url) = root_url.join(path) {
                        if let Ok(val_cand) =
                            ValidatedUrl::parse(candidate_url.as_str(), &self.policy)
                        {
                            let target = WebTarget::new(val_cand);
                            match self.fetcher.fetch(&target).await {
                                Ok(resp) if resp.status == 200 => {
                                    sitemap_candidates.push(candidate_url);
                                    break; // Found primary well-known sitemap
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if sitemap_candidates.is_empty() {
            warnings.push(format!(
                "No sitemaps discovered for target domain '{}'",
                target_host
            ));
            return Ok(DiscoveredSitemapReport {
                target_url,
                target_domain: target_host,
                sitemaps_inspected: Vec::new(),
                total_urls_discovered: 0,
                unique_urls: Vec::new(),
                filtered_cross_domain_count: 0,
                warning_messages: warnings,
            });
        }

        // 4. Process discovered sitemaps queue with recursion controls
        let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
        for candidate in sitemap_candidates {
            queue.push_back((candidate, 0));
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut sitemaps_inspected: Vec<Url> = Vec::new();
        let mut raw_discovered_count = 0;
        let mut filtered_cross_domain_count = 0;
        let mut unique_entries_map: std::collections::HashMap<String, SitemapEntry> =
            std::collections::HashMap::new();

        while let Some((current_sitemap_url, depth)) = queue.pop_front() {
            let url_str = current_sitemap_url.as_str().to_string();
            if visited.contains(&url_str) {
                continue;
            }
            visited.insert(url_str);

            // Validate sitemap URL security
            let validated_sitemap_url =
                match ValidatedUrl::parse(current_sitemap_url.as_str(), &self.policy) {
                    Ok(val) => val,
                    Err(e) => {
                        warnings.push(format!(
                            "Skipping sitemap '{}' due to URL security policy: {e}",
                            current_sitemap_url
                        ));
                        continue;
                    }
                };

            let target = WebTarget::new(validated_sitemap_url);
            let resp = match self.fetcher.fetch(&target).await {
                Ok(r) if r.status == 200 => r,
                Ok(r) => {
                    warnings.push(format!(
                        "Sitemap '{}' returned HTTP {}",
                        current_sitemap_url, r.status
                    ));
                    continue;
                }
                Err(e) => {
                    warnings.push(format!(
                        "Failed to fetch sitemap '{}': {e}",
                        current_sitemap_url
                    ));
                    continue;
                }
            };

            sitemaps_inspected.push(current_sitemap_url.clone());

            let parsed = match parse_sitemap(&current_sitemap_url, &resp.body) {
                Ok(p) => p,
                Err(e) => {
                    warnings.push(format!(
                        "Failed to parse sitemap '{}': {e}",
                        current_sitemap_url
                    ));
                    continue;
                }
            };

            warnings.extend(parsed.warnings);

            // Enqueue child sitemaps if this was a <sitemapindex>
            if parsed.sitemap_type == SitemapType::SitemapIndex {
                if depth < self.config.max_index_depth {
                    let mut child_count = 0;
                    for child in parsed.child_sitemaps {
                        if child_count >= self.config.max_child_sitemaps {
                            warnings.push(format!(
                                "Sitemap index '{}' exceeded max child sitemap limit ({})",
                                current_sitemap_url, self.config.max_child_sitemaps
                            ));
                            break;
                        }
                        if !visited.contains(child.sitemap_url.as_str()) {
                            queue.push_back((child.sitemap_url, depth + 1));
                            child_count += 1;
                        }
                    }
                } else {
                    warnings.push(format!(
                        "Max sitemap index recursion depth ({}) reached at '{}'",
                        self.config.max_index_depth, current_sitemap_url
                    ));
                }
            }

            // Ingest discovered URL entries
            for entry in parsed.entries {
                raw_discovered_count += 1;

                // Host and cross-domain policy check
                if !self.is_allowed_host(&entry.url, &target_host) {
                    filtered_cross_domain_count += 1;
                    continue;
                }

                // SSRF policy check on entry URL
                if ValidatedUrl::parse(entry.url.as_str(), &self.policy).is_err() {
                    filtered_cross_domain_count += 1;
                    continue;
                }

                // Deduplicate and retain entry with higher priority or newer lastmod
                let entry_key = entry.url.as_str().to_string();
                if let Some(existing) = unique_entries_map.get_mut(&entry_key) {
                    if entry.priority > existing.priority {
                        existing.priority = entry.priority;
                    }
                    if entry.last_modified > existing.last_modified {
                        existing.last_modified = entry.last_modified;
                    }
                } else {
                    if unique_entries_map.len() >= self.config.max_urls {
                        warnings.push(format!(
                            "Max URLs limit ({}) reached during discovery",
                            self.config.max_urls
                        ));
                        break;
                    }
                    unique_entries_map.insert(entry_key, entry);
                }
            }

            if unique_entries_map.len() >= self.config.max_urls {
                break;
            }
        }

        // Sort unique URLs by priority descending, then URL lexicographically
        let mut unique_urls: Vec<SitemapEntry> = unique_entries_map.into_values().collect();
        unique_urls.sort_by(|a, b| {
            let p_a = a.priority.unwrap_or(0.5);
            let p_b = b.priority.unwrap_or(0.5);
            p_b.partial_cmp(&p_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.url.as_str().cmp(b.url.as_str()))
        });

        Ok(DiscoveredSitemapReport {
            target_url,
            target_domain: target_host,
            sitemaps_inspected,
            total_urls_discovered: raw_discovered_count,
            unique_urls,
            filtered_cross_domain_count,
            warning_messages: warnings,
        })
    }

    /// Verifies if a candidate entry's host satisfies the configured domain policy.
    fn is_allowed_host(&self, entry_url: &Url, target_host: &str) -> bool {
        if !self.config.enforce_same_host {
            return true;
        }

        let entry_host = match entry_url.host_str() {
            Some(h) => h,
            None => return false,
        };

        if entry_host.eq_ignore_ascii_case(target_host) {
            return true;
        }

        if self.config.allow_subdomains {
            // Check if entry_host is a subdomain: e.g. api.example.com vs example.com
            let target_with_dot = format!(".{target_host}");
            if entry_host.to_ascii_lowercase().ends_with(&target_with_dot) {
                return true;
            }
        }

        false
    }
}

/// Heuristically identifies if a URL directly targets a sitemap file.
fn is_likely_sitemap_url(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    path.ends_with(".xml")
        || path.ends_with(".xml.gz")
        || path.ends_with(".txt")
        || path.contains("sitemap")
}

/// Extracts the scheme + host + optional port root URL.
fn get_origin_root(url: &Url) -> Result<Url, WebExtractorError> {
    let root_str = format!("{}://{}/", url.scheme(), url.authority());
    Url::parse(&root_str).map_err(|e| {
        WebExtractorError::Sitemap(format!(
            "Failed to determine site origin root for '{url}': {e}"
        ))
    })
}
