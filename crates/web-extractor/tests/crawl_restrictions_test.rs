//! # Crawl Restrictions and Depth Traversal Integration Tests
//!
//! Validates crawl depth limits, exact host and subdomain scoping, path prefix enforcement,
//! custom domain allowlists, path include/exclude glob and regex patterns, query parameter
//! policies, asset extension blocking, port and scheme validation, page budget exhaustion,
//! and SSRF security defense.

use std::collections::HashSet;

use url::Url;
use web_repo_to_ai_url_security::UrlSecurityPolicy;
use web_repo_to_ai_web_extractor::{
    CrawlDecision, CrawlRejectionReason, CrawlRestrictions, CrawlRestrictionsEvaluator, CrawlScope,
    QueryParamPolicy,
};

// ============================================================================
// TEST 1: Crawl Depth Enforcement
// ============================================================================

#[test]
fn test_crawl_depth_enforcement() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default().with_max_depth(3);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Depth 0 -> 1: Allowed
    let u1 = Url::parse("https://example.com/page1").unwrap();
    let d1 = evaluator.evaluate_candidate(&u1, 0);
    assert!(d1.is_allowed());
    assert_eq!(d1.allowed_depth(), Some(1));

    // Depth 1 -> 2: Allowed
    let u2 = Url::parse("https://example.com/page2").unwrap();
    let d2 = evaluator.evaluate_candidate(&u2, 1);
    assert!(d2.is_allowed());
    assert_eq!(d2.allowed_depth(), Some(2));

    // Depth 2 -> 3: Allowed (at limit)
    let u3 = Url::parse("https://example.com/page3").unwrap();
    let d3 = evaluator.evaluate_candidate(&u3, 2);
    assert!(d3.is_allowed());
    assert_eq!(d3.allowed_depth(), Some(3));

    // Depth 3 -> 4: Rejected (exceeds max_depth 3)
    let u4 = Url::parse("https://example.com/page4").unwrap();
    let d4 = evaluator.evaluate_candidate(&u4, 3);
    assert!(d4.is_rejected());
    match d4 {
        CrawlDecision::Rejected { reason, .. } => {
            assert_eq!(
                reason,
                CrawlRejectionReason::ExceedsMaxDepth {
                    depth: 4,
                    max_depth: 3
                }
            );
        }
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 2: Exact Host Scope Enforcement
// ============================================================================

#[test]
fn test_exact_host_scope_enforcement() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default().with_scope(CrawlScope::ExactHost);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Exact host allowed
    let u1 = Url::parse("https://example.com/about").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    // Subdomain rejected under ExactHost
    let u2 = Url::parse("https://docs.example.com/guide").unwrap();
    let d2 = evaluator.evaluate_candidate(&u2, 0);
    assert!(d2.is_rejected());
    match d2 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::SubdomainDisallowed { host, seed_host } => {
                assert_eq!(host, "docs.example.com");
                assert_eq!(seed_host, "example.com");
            }
            _ => panic!("Expected SubdomainDisallowed, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }

    // External domain rejected
    let u3 = Url::parse("https://other-domain.org/").unwrap();
    let d3 = evaluator.evaluate_candidate(&u3, 0);
    assert!(d3.is_rejected());
    match d3 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::ExternalHost { host, seed_host } => {
                assert_eq!(host, "other-domain.org");
                assert_eq!(seed_host, "example.com");
            }
            _ => panic!("Expected ExternalHost, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 3: Subdomain Scope Allowance
// ============================================================================

#[test]
fn test_subdomain_scope_allowance() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default().with_scope(CrawlScope::Subdomains);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Primary host allowed
    let u1 = Url::parse("https://example.com/about").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    // Subdomains allowed
    let u2 = Url::parse("https://docs.example.com/api").unwrap();
    assert!(evaluator.evaluate_candidate(&u2, 0).is_allowed());

    let u3 = Url::parse("https://api.v2.example.com/v1").unwrap();
    assert!(evaluator.evaluate_candidate(&u3, 0).is_allowed());

    // Completely external domain rejected
    let u4 = Url::parse("https://not-example.com/").unwrap();
    assert!(evaluator.evaluate_candidate(&u4, 0).is_rejected());
}

// ============================================================================
// TEST 4: Path Prefix Scope Enforcement
// ============================================================================

#[test]
fn test_path_prefix_scope_enforcement() {
    let seed_url = Url::parse("https://example.com/docs/v2/index.html").unwrap();
    let restrictions = CrawlRestrictions::default().with_scope(CrawlScope::PathPrefix);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // URL under /docs/v2/ allowed
    let u1 = Url::parse("https://example.com/docs/v2/guides/quickstart").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    let u2 = Url::parse("https://example.com/docs/v2/api").unwrap();
    assert!(evaluator.evaluate_candidate(&u2, 0).is_allowed());

    // URL outside /docs/v2/ rejected
    let u3 = Url::parse("https://example.com/docs/v1/legacy").unwrap();
    let d3 = evaluator.evaluate_candidate(&u3, 0);
    assert!(d3.is_rejected());
    match d3 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::PathPrefixMismatch {
                path,
                required_prefix,
            } => {
                assert_eq!(path, "/docs/v1/legacy");
                assert_eq!(required_prefix, "/docs/v2/");
            }
            _ => panic!("Expected PathPrefixMismatch, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }

    let u4 = Url::parse("https://example.com/blog/article").unwrap();
    assert!(evaluator.evaluate_candidate(&u4, 0).is_rejected());
}

// ============================================================================
// TEST 5: Custom Domain Allowlist Scope
// ============================================================================

#[test]
fn test_custom_domain_allowlist_scope() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let scope = CrawlScope::DomainAllowlist(vec![
        "example.com".to_string(),
        "*.partner.org".to_string(),
        "cdn.assets.net".to_string(),
    ]);
    let restrictions = CrawlRestrictions::default().with_scope(scope);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Primary host allowed
    let u1 = Url::parse("https://example.com/home").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    // Wildcard partner allowed
    let u2 = Url::parse("https://api.partner.org/data").unwrap();
    assert!(evaluator.evaluate_candidate(&u2, 0).is_allowed());

    let u3 = Url::parse("https://docs.partner.org/").unwrap();
    assert!(evaluator.evaluate_candidate(&u3, 0).is_allowed());

    // Explicit cdn host allowed
    let u4_page = Url::parse("https://cdn.assets.net/terms").unwrap();
    assert!(evaluator.evaluate_candidate(&u4_page, 0).is_allowed());

    // Rogue domain rejected
    let u5 = Url::parse("https://malicious.com/").unwrap();
    let d5 = evaluator.evaluate_candidate(&u5, 0);
    assert!(d5.is_rejected());
    match d5 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::DomainNotInAllowlist { host } => {
                assert_eq!(host, "malicious.com");
            }
            _ => panic!("Expected DomainNotInAllowlist, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 6: Path Include Patterns (Globs and Wildcards)
// ============================================================================

#[test]
fn test_path_include_patterns() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default()
        .with_include_pattern("/docs/**")
        .with_include_pattern("/guides/*");
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // /docs/** matches subpaths
    let u1 = Url::parse("https://example.com/docs/intro").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    let u2 = Url::parse("https://example.com/docs/v1/deep/nested").unwrap();
    assert!(evaluator.evaluate_candidate(&u2, 0).is_allowed());

    // /guides/* matches single segment
    let u3 = Url::parse("https://example.com/guides/setup").unwrap();
    assert!(evaluator.evaluate_candidate(&u3, 0).is_allowed());

    // /guides/* does NOT match nested segments
    let u4 = Url::parse("https://example.com/guides/setup/advanced").unwrap();
    assert!(evaluator.evaluate_candidate(&u4, 0).is_rejected());

    // Non-matching path rejected
    let u5 = Url::parse("https://example.com/blog/post-1").unwrap();
    let d5 = evaluator.evaluate_candidate(&u5, 0);
    assert!(d5.is_rejected());
    match d5 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::NotMatchedByIncludePatterns { path } => {
                assert_eq!(path, "/blog/post-1");
            }
            _ => panic!("Expected NotMatchedByIncludePatterns, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 7: Path Exclude Patterns (Globs and Regex)
// ============================================================================

#[test]
fn test_path_exclude_patterns() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default()
        .with_exclude_pattern("/admin/**")
        .with_exclude_pattern("/cart/*")
        .with_exclude_pattern("*/logout")
        .with_exclude_pattern("regex:^/temp/[0-9]+$");
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Legitimate path allowed
    let u1 = Url::parse("https://example.com/products/widget").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    // /admin/** excluded
    let u2 = Url::parse("https://example.com/admin/dashboard/users").unwrap();
    let d2 = evaluator.evaluate_candidate(&u2, 0);
    assert!(d2.is_rejected());
    match d2 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::ExcludedByPattern { pattern, path } => {
                assert_eq!(pattern, "/admin/**");
                assert_eq!(path, "/admin/dashboard/users");
            }
            _ => panic!("Expected ExcludedByPattern, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }

    // /cart/* excluded
    let u3 = Url::parse("https://example.com/cart/checkout").unwrap();
    assert!(evaluator.evaluate_candidate(&u3, 0).is_rejected());

    // */logout excluded
    let u4 = Url::parse("https://example.com/user/session/logout").unwrap();
    assert!(evaluator.evaluate_candidate(&u4, 0).is_rejected());

    // regex:^/temp/[0-9]+$ excluded
    let u5 = Url::parse("https://example.com/temp/98765").unwrap();
    assert!(evaluator.evaluate_candidate(&u5, 0).is_rejected());

    // Non-numeric temp allowed
    let u6 = Url::parse("https://example.com/temp/docs").unwrap();
    assert!(evaluator.evaluate_candidate(&u6, 0).is_allowed());
}

// ============================================================================
// TEST 8: Query Parameter Policies
// ============================================================================

#[test]
fn test_query_param_policies() {
    let seed_url = Url::parse("https://example.com/").unwrap();

    // 1. StripAll policy
    let r1 = CrawlRestrictions::default().with_query_policy(QueryParamPolicy::StripAll);
    let mut ev1 = CrawlRestrictionsEvaluator::new(seed_url.clone(), r1).unwrap();
    let u1 = Url::parse("https://example.com/items?sort=price&page=2#heading").unwrap();
    let d1 = ev1.evaluate_candidate(&u1, 0);
    assert!(d1.is_allowed());
    assert_eq!(
        d1.allowed_url().unwrap().as_str(),
        "https://example.com/items"
    );

    // 2. AllowList policy
    let mut allow_keys = HashSet::new();
    allow_keys.insert("page".to_string());
    allow_keys.insert("category".to_string());
    let r2 =
        CrawlRestrictions::default().with_query_policy(QueryParamPolicy::AllowList(allow_keys));
    let mut ev2 = CrawlRestrictionsEvaluator::new(seed_url.clone(), r2).unwrap();
    let u2 =
        Url::parse("https://example.com/store?page=3&sort=asc&category=electronics&session=123")
            .unwrap();
    let d2 = ev2.evaluate_candidate(&u2, 0);
    assert!(d2.is_allowed());
    let normalized_url = d2.allowed_url().unwrap();
    assert!(normalized_url.as_str().contains("page=3"));
    assert!(normalized_url.as_str().contains("category=electronics"));
    assert!(!normalized_url.as_str().contains("sort=asc"));
    assert!(!normalized_url.as_str().contains("session=123"));

    // 3. BlockList policy
    let mut block_keys = HashSet::new();
    block_keys.insert("session".to_string());
    block_keys.insert("utm_source".to_string());
    let r3 =
        CrawlRestrictions::default().with_query_policy(QueryParamPolicy::BlockList(block_keys));
    let mut ev3 = CrawlRestrictionsEvaluator::new(seed_url.clone(), r3).unwrap();
    let u3 = Url::parse("https://example.com/page?id=42&session=abc&utm_source=twitter").unwrap();
    let d3 = ev3.evaluate_candidate(&u3, 0);
    assert!(d3.is_allowed());
    assert_eq!(
        d3.allowed_url().unwrap().as_str(),
        "https://example.com/page?id=42"
    );

    // 4. AllowAll policy
    let r4 = CrawlRestrictions::default().with_query_policy(QueryParamPolicy::AllowAll);
    let mut ev4 = CrawlRestrictionsEvaluator::new(seed_url, r4).unwrap();
    let u4 = Url::parse("https://example.com/data?x=1&y=2#fragment").unwrap();
    let d4 = ev4.evaluate_candidate(&u4, 0);
    assert!(d4.is_allowed());
    assert_eq!(
        d4.allowed_url().unwrap().as_str(),
        "https://example.com/data?x=1&y=2"
    );
}

// ============================================================================
// TEST 9: Asset Extension Rejection
// ============================================================================

#[test]
fn test_asset_extension_rejection() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default();
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    for ext in &[".pdf", ".zip", ".tar.gz", ".png", ".jpg", ".exe", ".iso"] {
        let u = Url::parse(&format!("https://example.com/files/resource{ext}")).unwrap();
        let d = evaluator.evaluate_candidate(&u, 0);
        assert!(d.is_rejected());
        match d {
            CrawlDecision::Rejected { reason, .. } => match reason {
                CrawlRejectionReason::DisallowedAssetExtension { extension } => {
                    assert!(ext.ends_with(&extension));
                }
                _ => panic!("Expected DisallowedAssetExtension, got {reason:?}"),
            },
            _ => panic!("Expected rejection"),
        }
    }

    // HTML or standard paths allowed
    let valid = Url::parse("https://example.com/docs/manual.html").unwrap();
    assert!(evaluator.evaluate_candidate(&valid, 0).is_allowed());
}

// ============================================================================
// TEST 10: Scheme and Port Restrictions
// ============================================================================

#[test]
fn test_scheme_and_port_restrictions() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default()
        .with_enforce_same_scheme(true)
        .with_enforce_same_port(true);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Matching https allowed
    let u1 = Url::parse("https://example.com/about").unwrap();
    assert!(evaluator.evaluate_candidate(&u1, 0).is_allowed());

    // Insecure HTTP rejected when seed is HTTPS and enforce_same_scheme is true
    let u2 = Url::parse("http://example.com/about").unwrap();
    let d2 = evaluator.evaluate_candidate(&u2, 0);
    assert!(d2.is_rejected());
    match d2 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::DisallowedScheme { scheme } => {
                assert_eq!(scheme, "http");
            }
            _ => panic!("Expected DisallowedScheme, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }

    // Non-standard port rejected
    let u3 = Url::parse("https://example.com:8443/about").unwrap();
    let d3 = evaluator.evaluate_candidate(&u3, 0);
    assert!(d3.is_rejected());
    match d3 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::DisallowedPort { port, .. } => {
                assert_eq!(port, Some(8443));
            }
            _ => panic!("Expected DisallowedPort, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 11: Total Page Budget Exhaustion
// ============================================================================

#[test]
fn test_total_page_budget_exhaustion() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default().with_max_pages(3);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    let p1 = Url::parse("https://example.com/p1").unwrap();
    let p2 = Url::parse("https://example.com/p2").unwrap();
    let p3 = Url::parse("https://example.com/p3").unwrap();

    assert!(evaluator.record_page_crawled(&p1, 1).is_ok());
    assert!(evaluator.record_page_crawled(&p2, 1).is_ok());
    assert!(evaluator.record_page_crawled(&p3, 2).is_ok());

    let stats = evaluator.stats();
    assert_eq!(stats.total_crawled, 3);
    assert_eq!(stats.remaining_pages, 0);
    assert!(stats.is_exhausted);

    // 4th candidate evaluated after exhaustion
    let p4 = Url::parse("https://example.com/p4").unwrap();
    let d4 = evaluator.evaluate_candidate(&p4, 1);
    assert!(d4.is_rejected());
    match d4 {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::TotalPageBudgetExceeded { count, limit } => {
                assert_eq!(count, 3);
                assert_eq!(limit, 3);
            }
            _ => panic!("Expected TotalPageBudgetExceeded, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 12: Per-Depth and Per-Host Page Budgets
// ============================================================================

#[test]
fn test_per_depth_and_per_host_page_budgets() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default()
        .with_scope(CrawlScope::Subdomains)
        .with_max_pages_per_depth(2)
        .with_max_pages_per_host(2);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // 1. Depth budget test
    let d1_a = Url::parse("https://example.com/d1_a").unwrap();
    let d1_b = Url::parse("https://example.com/d1_b").unwrap();
    evaluator.record_page_crawled(&d1_a, 1).unwrap();
    evaluator.record_page_crawled(&d1_b, 1).unwrap();

    // 3rd candidate at depth 1 rejected
    let d1_c = Url::parse("https://example.com/d1_c").unwrap();
    let dec_depth = evaluator.evaluate_candidate(&d1_c, 0); // parent 0 -> candidate depth 1
    assert!(dec_depth.is_rejected());
    match dec_depth {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::DepthBudgetExceeded {
                depth,
                count,
                limit,
            } => {
                assert_eq!(depth, 1);
                assert_eq!(count, 2);
                assert_eq!(limit, 2);
            }
            _ => panic!("Expected DepthBudgetExceeded, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }

    // But candidate at depth 2 allowed!
    let d2_a = Url::parse("https://sub.example.com/d2_a").unwrap();
    let dec_d2 = evaluator.evaluate_candidate(&d2_a, 1); // parent 1 -> candidate depth 2
    assert!(dec_d2.is_allowed());

    // 2. Host budget test
    evaluator.record_page_crawled(&d2_a, 2).unwrap();
    let d2_b = Url::parse("https://sub.example.com/d2_b").unwrap();
    evaluator.record_page_crawled(&d2_b, 2).unwrap();

    // 3rd page for sub.example.com rejected due to host budget (candidate at depth 3, where depth quota is not exceeded)
    let d2_c = Url::parse("https://sub.example.com/d2_c").unwrap();
    let dec_host = evaluator.evaluate_candidate(&d2_c, 2);
    assert!(dec_host.is_rejected());
    match dec_host {
        CrawlDecision::Rejected { reason, .. } => match reason {
            CrawlRejectionReason::HostBudgetExceeded { host, count, limit } => {
                assert_eq!(host, "sub.example.com");
                assert_eq!(count, 2);
                assert_eq!(limit, 2);
            }
            _ => panic!("Expected HostBudgetExceeded, got {reason:?}"),
        },
        _ => panic!("Expected rejection"),
    }
}

// ============================================================================
// TEST 13: Zero-Trust SSRF Security Rejection
// ============================================================================

#[test]
fn test_ssrf_zero_trust_rejection() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default()
        .with_scope(CrawlScope::DomainAllowlist(vec![
            "127.0.0.1".into(),
            "example.com".into(),
        ]))
        .with_ssrf_policy(UrlSecurityPolicy::default()); // default blocks loopback/private
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    // Loopback candidate blocked by SSRF
    let loopback = Url::parse("http://127.0.0.1/admin").unwrap();
    let dec_loop = evaluator.evaluate_candidate(&loopback, 0);
    assert!(dec_loop.is_rejected());
    match dec_loop {
        CrawlDecision::Rejected { reason, .. } => {
            assert!(matches!(
                reason,
                CrawlRejectionReason::SsrfViolation { .. }
                    | CrawlRejectionReason::DisallowedScheme { .. }
            ));
        }
        _ => panic!("Expected SSRF violation"),
    }

    // Cloud metadata IP blocked by SSRF
    let meta = Url::parse("http://169.254.169.254/latest").unwrap();
    let dec_meta = evaluator.evaluate_candidate(&meta, 0);
    assert!(dec_meta.is_rejected());
}

// ============================================================================
// TEST 14: URL Depth Deduplication & Minimum Seen Depth
// ============================================================================

#[test]
fn test_url_depth_deduplication_and_minimum_seen_depth() {
    let seed_url = Url::parse("https://example.com/").unwrap();
    let restrictions = CrawlRestrictions::default().with_max_depth(5);
    let mut evaluator = CrawlRestrictionsEvaluator::new(seed_url, restrictions).unwrap();

    let target_page = Url::parse("https://example.com/guide/installation").unwrap();

    // First discovered via deep path at parent_depth = 2 -> candidate depth = 3
    let d1 = evaluator.evaluate_candidate(&target_page, 2);
    assert!(d1.is_allowed());
    assert_eq!(d1.allowed_depth(), Some(3));

    // Later discovered via direct home link at parent_depth = 0 -> candidate depth = 1!
    let d2 = evaluator.evaluate_candidate(&target_page, 0);
    assert!(d2.is_allowed());
    assert_eq!(d2.allowed_depth(), Some(1)); // Updated to minimum depth!
}
