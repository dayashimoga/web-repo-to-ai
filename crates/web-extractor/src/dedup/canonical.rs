//! # Canonical URL Resolver
//!
//! Evaluates competing canonical URL signals (`<link rel="canonical">`, OpenGraph `og:url`,
//! Schema.org JSON-LD, HTTP redirects, and requested URL) under strict zero-trust SSRF validation.

use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::model::{CanonicalResolution, CanonicalSource, DedupOptions};

/// Resolver for determining the authoritative canonical URL of a webpage.
#[derive(Debug, Clone, Default)]
pub struct CanonicalUrlResolver;

impl CanonicalUrlResolver {
    /// Resolves the definitive canonical URL among competing metadata and transport signals.
    #[must_use]
    pub fn resolve(
        requested_url: &ValidatedUrl,
        response_url: &ValidatedUrl,
        meta_canonical: Option<&str>,
        og_url: Option<&str>,
        json_ld_url: Option<&str>,
        options: &DedupOptions,
        policy: &UrlSecurityPolicy,
    ) -> CanonicalResolution {
        let mut warnings = Vec::new();

        // 1. Evaluate HTML <link rel="canonical">
        if let Some(raw) = meta_canonical {
            if let Some(res) = Self::evaluate_candidate(
                raw,
                CanonicalSource::HtmlLinkRelCanonical,
                100,
                response_url,
                options,
                policy,
                &mut warnings,
            ) {
                return res;
            }
        }

        // 2. Evaluate OpenGraph og:url
        if let Some(raw) = og_url {
            if let Some(res) = Self::evaluate_candidate(
                raw,
                CanonicalSource::OpenGraph,
                85,
                response_url,
                options,
                policy,
                &mut warnings,
            ) {
                return res;
            }
        }

        // 3. Evaluate Schema.org JSON-LD url
        if let Some(raw) = json_ld_url {
            if let Some(res) = Self::evaluate_candidate(
                raw,
                CanonicalSource::JsonLd,
                80,
                response_url,
                options,
                policy,
                &mut warnings,
            ) {
                return res;
            }
        }

        // 4. Fall back to HTTP response URL (if redirected) or original requested URL
        let is_redirected = response_url.as_str().trim_end_matches('/')
            != requested_url.as_str().trim_end_matches('/');

        if is_redirected {
            CanonicalResolution {
                canonical_url: response_url.clone(),
                source: CanonicalSource::HttpRedirect,
                raw_candidate: None,
                is_cross_domain: false,
                confidence: 75,
                warnings,
            }
        } else {
            CanonicalResolution {
                canonical_url: response_url.clone(),
                source: CanonicalSource::OriginalRequest,
                raw_candidate: None,
                is_cross_domain: false,
                confidence: 70,
                warnings,
            }
        }
    }

    /// Evaluates, parses, normalizes, and security-validates a candidate URL string.
    fn evaluate_candidate(
        raw_candidate: &str,
        source: CanonicalSource,
        base_confidence: u8,
        response_url: &ValidatedUrl,
        options: &DedupOptions,
        policy: &UrlSecurityPolicy,
        warnings: &mut Vec<String>,
    ) -> Option<CanonicalResolution> {
        let trimmed = raw_candidate.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Strip fragment (#...)
        let no_fragment = match trimmed.split_once('#') {
            Some((before, _)) => before.trim(),
            None => trimmed,
        };

        if no_fragment.is_empty() {
            return None;
        }

        // Parse and resolve relative / absolute candidate
        let validated = if no_fragment.starts_with("//") {
            // Protocol-relative URL
            let scheme = response_url.scheme();
            let full_url = format!("{scheme}:{no_fragment}");
            ValidatedUrl::parse(&full_url, policy).ok()
        } else if no_fragment.starts_with("http://") || no_fragment.starts_with("https://") {
            ValidatedUrl::parse(no_fragment, policy).ok()
        } else {
            // Relative URL (e.g. /docs/page or page.html)
            response_url.join(no_fragment, policy).ok()
        };

        let candidate_url = match validated {
            Some(v) => v,
            None => {
                warnings.push(format!(
                    "Candidate canonical URL from {source:?} '{trimmed}' failed SSRF or security validation."
                ));
                return None;
            }
        };

        // Check cross-domain status
        let is_cross_domain = candidate_url.host_str() != response_url.host_str();
        let mut confidence = base_confidence;

        if is_cross_domain {
            if !options.allow_cross_domain_canonical {
                warnings.push(format!(
                    "Cross-domain canonical from {source:?} '{trimmed}' was rejected by policy."
                ));
                return None;
            }
            confidence = confidence.saturating_sub(15);
            warnings.push(format!(
                "Canonical candidate from {source:?} points to cross-domain host '{}'.",
                candidate_url.host_str()
            ));
        }

        Some(CanonicalResolution {
            canonical_url: candidate_url,
            source,
            raw_candidate: Some(trimmed.to_string()),
            is_cross_domain,
            confidence,
            warnings: warnings.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_resolver_prefers_link_rel_canonical() {
        let policy = UrlSecurityPolicy::default();
        let options = DedupOptions::default();

        let req =
            ValidatedUrl::parse("https://example.com/blog?utm_source=twitter", &policy).unwrap();
        let resp = ValidatedUrl::parse("https://example.com/blog", &policy).unwrap();

        let res = CanonicalUrlResolver::resolve(
            &req,
            &resp,
            Some("https://example.com/blog/canonical-target"),
            Some("https://example.com/blog/og-target"),
            None,
            &options,
            &policy,
        );

        assert_eq!(res.source, CanonicalSource::HtmlLinkRelCanonical);
        assert_eq!(
            res.canonical_url.as_str(),
            "https://example.com/blog/canonical-target"
        );
        assert!(res.is_high_confidence());
        assert!(!res.is_cross_domain);
    }

    #[test]
    fn test_canonical_resolver_relative_link_resolution() {
        let policy = UrlSecurityPolicy::default();
        let options = DedupOptions::default();

        let req = ValidatedUrl::parse("https://example.com/docs/intro", &policy).unwrap();
        let resp = req.clone();

        let res = CanonicalUrlResolver::resolve(
            &req,
            &resp,
            Some("/docs/getting-started"),
            None,
            None,
            &options,
            &policy,
        );

        assert_eq!(res.source, CanonicalSource::HtmlLinkRelCanonical);
        assert_eq!(
            res.canonical_url.as_str(),
            "https://example.com/docs/getting-started"
        );
    }

    #[test]
    fn test_canonical_resolver_rejects_ssrf_malicious_canonical() {
        let policy = UrlSecurityPolicy::default();
        let options = DedupOptions::default();

        let req = ValidatedUrl::parse("https://example.com/page", &policy).unwrap();
        let resp = req.clone();

        // Attacker attempts to trick canonicalization into cloud metadata
        let res = CanonicalUrlResolver::resolve(
            &req,
            &resp,
            Some("http://169.254.169.254/latest/meta-data"),
            Some("https://example.com/page/valid-og"),
            None,
            &options,
            &policy,
        );

        // Malicious candidate is rejected by SSRF filter, falls back to OpenGraph
        assert_eq!(res.source, CanonicalSource::OpenGraph);
        assert_eq!(
            res.canonical_url.as_str(),
            "https://example.com/page/valid-og"
        );
    }

    #[test]
    fn test_canonical_resolver_fallback_to_http_redirect() {
        let policy = UrlSecurityPolicy::default();
        let options = DedupOptions::default();

        let req = ValidatedUrl::parse("http://example.com/old-url", &policy).unwrap();
        let resp = ValidatedUrl::parse("https://example.com/new-url", &policy).unwrap();

        let res = CanonicalUrlResolver::resolve(&req, &resp, None, None, None, &options, &policy);

        assert_eq!(res.source, CanonicalSource::HttpRedirect);
        assert_eq!(res.canonical_url.as_str(), "https://example.com/new-url");
    }
}
