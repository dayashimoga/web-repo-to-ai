//! # Crawler Deduplication and Loop Prevention Integration Tests
//!
//! Validates canonical URL normalization, crawler trap and loop detection (path repetition,
//! cyclic sequences, excessive depth, query parameter explosion, redirect cycles),
//! frontier priority queue management, and multi-tier duplicate prevention (URL canonicalization,
//! BLAKE3 exact content deduplication, and SimHash near-duplicate detection).

use url::Url;
use web_repo_to_ai_web_extractor::{
    CrawlFrontier, CrawlLoopDetector, CrawlTrapReason, CrawlerUrlNormalizer, DedupOptions,
    DuplicateClassification, FrontierEnqueueDecision, LoopDetectorConfig,
};

#[test]
fn test_crawler_url_normalizer_canonicalization() {
    let normalizer = CrawlerUrlNormalizer::new();

    // 1. Fragment stripping
    let u1 = Url::parse("https://example.com/docs#installation-guide").unwrap();
    assert_eq!(
        normalizer.normalize(&u1).as_str(),
        "https://example.com/docs"
    );

    // 2. Default port stripping (80 for http, 443 for https, keep custom 8080)
    let u2 = Url::parse("http://example.com:80/api").unwrap();
    assert_eq!(normalizer.normalize(&u2).as_str(), "http://example.com/api");

    let u3 = Url::parse("https://example.com:443/api").unwrap();
    assert_eq!(
        normalizer.normalize(&u3).as_str(),
        "https://example.com/api"
    );

    let u4 = Url::parse("https://example.com:8080/api").unwrap();
    assert_eq!(
        normalizer.normalize(&u4).as_str(),
        "https://example.com:8080/api"
    );

    // 3. Trailing slash stripping on non-root paths; root "/" preserved
    let u5 = Url::parse("https://example.com/guide/").unwrap();
    assert_eq!(
        normalizer.normalize(&u5).as_str(),
        "https://example.com/guide"
    );

    let u6 = Url::parse("https://example.com/").unwrap();
    assert_eq!(normalizer.normalize(&u6).as_str(), "https://example.com/");

    // 4. Default index filenames normalized
    let u7 = Url::parse("https://example.com/docs/index.html").unwrap();
    assert_eq!(
        normalizer.normalize(&u7).as_str(),
        "https://example.com/docs"
    );

    let u8 = Url::parse("https://example.com/index.php").unwrap();
    assert_eq!(normalizer.normalize(&u8).as_str(), "https://example.com/");

    // 5. Tracking parameter stripping
    let u9 = Url::parse(
        "https://example.com/page?utm_source=twitter&utm_medium=social&fbclid=xyz123&page=2",
    )
    .unwrap();
    assert_eq!(
        normalizer.normalize(&u9).as_str(),
        "https://example.com/page?page=2"
    );

    // 6. Query parameter sorting
    let u10 = Url::parse("https://example.com/search?z=3&a=1&m=2").unwrap();
    assert_eq!(
        normalizer.normalize(&u10).as_str(),
        "https://example.com/search?a=1&m=2&z=3"
    );

    // 7. Combined canonicalization
    let u11 =
        Url::parse("https://example.com:443/docs/index.html?utm_campaign=winter&b=2&a=1#section-3")
            .unwrap();
    assert_eq!(
        normalizer.normalize(&u11).as_str(),
        "https://example.com/docs?a=1&b=2"
    );
}

#[test]
fn test_loop_detector_path_segment_repetition() {
    let detector = CrawlLoopDetector::new();

    // Normal path passes
    let valid = Url::parse("https://example.com/docs/api/v1/endpoints").unwrap();
    assert!(detector.evaluate_url(&valid).is_ok());

    // Repeating path segment beyond default threshold of 3
    let trap = Url::parse("https://example.com/tag/rust/tag/rust/tag/rust/tag/rust").unwrap();
    let result = detector.evaluate_url(&trap);
    assert!(result.is_err());
    match result.unwrap_err() {
        CrawlTrapReason::PathSegmentRepetition {
            segment,
            count,
            threshold,
        } => {
            assert_eq!(segment, "tag");
            assert_eq!(count, 4);
            assert_eq!(threshold, 3);
        }
        other => panic!("Expected PathSegmentRepetition, got: {other:?}"),
    }

    // Custom threshold configuration
    let custom_detector =
        CrawlLoopDetector::with_config(LoopDetectorConfig::default().with_max_segment_repeats(1));
    let twice = Url::parse("https://example.com/category/books/category/clothing").unwrap();
    assert!(custom_detector.evaluate_url(&twice).is_err());
}

#[test]
fn test_loop_detector_cyclic_path_sequence() {
    let detector = CrawlLoopDetector::new();

    // Cyclic pattern of length 2 repeating 2 times: [category, books, category, books]
    let cycle_url =
        Url::parse("https://example.com/shop/category/books/category/books/item1").unwrap();
    let res = detector.evaluate_url(&cycle_url);
    assert!(res.is_err());
    match res.unwrap_err() {
        CrawlTrapReason::CyclicPathSequence {
            pattern,
            repetitions,
        } => {
            assert_eq!(pattern, vec!["category".to_string(), "books".to_string()]);
            assert_eq!(repetitions, 2);
        }
        other => panic!("Expected CyclicPathSequence, got: {other:?}"),
    }

    // Cyclic pattern of length 3: [a, b, c, a, b, c]
    let cycle3_url = Url::parse("https://example.com/a/b/c/a/b/c").unwrap();
    let res3 = detector.evaluate_url(&cycle3_url);
    assert!(res3.is_err());
    match res3.unwrap_err() {
        CrawlTrapReason::CyclicPathSequence {
            pattern,
            repetitions,
        } => {
            assert_eq!(pattern, vec!["a", "b", "c"]);
            assert_eq!(repetitions, 2);
        }
        other => panic!("Expected CyclicPathSequence, got: {other:?}"),
    }

    // Disabled cyclic detection
    let permissive = CrawlLoopDetector::with_config(
        LoopDetectorConfig::default().with_detect_cyclic_sequences(false),
    );
    assert!(permissive.evaluate_url(&cycle3_url).is_ok());
}

#[test]
fn test_loop_detector_excessive_path_segments() {
    let detector = CrawlLoopDetector::new();

    // 16 segments is within limit
    let p16 = (1..=16)
        .map(|i| format!("seg{i}"))
        .collect::<Vec<_>>()
        .join("/");
    let u16 = Url::parse(&format!("https://example.com/{p16}")).unwrap();
    assert!(detector.evaluate_url(&u16).is_ok());

    // 17 segments exceeds DEFAULT_MAX_PATH_SEGMENTS (16)
    let p17 = (1..=17)
        .map(|i| format!("seg{i}"))
        .collect::<Vec<_>>()
        .join("/");
    let u17 = Url::parse(&format!("https://example.com/{p17}")).unwrap();
    let res = detector.evaluate_url(&u17);
    assert!(res.is_err());
    match res.unwrap_err() {
        CrawlTrapReason::ExcessivePathSegments { count, limit } => {
            assert_eq!(count, 17);
            assert_eq!(limit, 16);
        }
        other => panic!("Expected ExcessivePathSegments, got: {other:?}"),
    }
}

#[test]
fn test_loop_detector_excessive_query_parameters() {
    let detector = CrawlLoopDetector::new();

    // 10 params is within DEFAULT_MAX_QUERY_PARAMS (10)
    let q10 = (1..=10)
        .map(|i| format!("k{i}=v{i}"))
        .collect::<Vec<_>>()
        .join("&");
    let u10 = Url::parse(&format!("https://example.com/items?{q10}")).unwrap();
    assert!(detector.evaluate_url(&u10).is_ok());

    // 11 params exceeds default limit
    let q11 = (1..=11)
        .map(|i| format!("k{i}=v{i}"))
        .collect::<Vec<_>>()
        .join("&");
    let u11 = Url::parse(&format!("https://example.com/items?{q11}")).unwrap();
    let res = detector.evaluate_url(&u11);
    assert!(res.is_err());
    match res.unwrap_err() {
        CrawlTrapReason::ExcessiveQueryParameters { count, limit } => {
            assert_eq!(count, 11);
            assert_eq!(limit, 10);
        }
        other => panic!("Expected ExcessiveQueryParameters, got: {other:?}"),
    }
}

#[test]
fn test_loop_detector_repeated_query_keys() {
    let detector = CrawlLoopDetector::new();

    // 2 repeats of a query key is acceptable
    let u2 = Url::parse("https://example.com/items?tag=rust&tag=tokio").unwrap();
    assert!(detector.evaluate_url(&u2).is_ok());

    // 3 repeats exceeds DEFAULT_MAX_QUERY_KEY_REPEATS (2)
    let u3 = Url::parse("https://example.com/items?tag=rust&tag=tokio&tag=axum").unwrap();
    let res = detector.evaluate_url(&u3);
    assert!(res.is_err());
    match res.unwrap_err() {
        CrawlTrapReason::RepeatedQueryKey { key, count, limit } => {
            assert_eq!(key, "tag");
            assert_eq!(count, 3);
            assert_eq!(limit, 2);
        }
        other => panic!("Expected RepeatedQueryKey, got: {other:?}"),
    }
}

#[test]
fn test_loop_detector_redirect_cycles() {
    let detector = CrawlLoopDetector::new();

    // Valid chain without cycles
    let legitimate_chain = vec![
        "https://example.com/start".to_string(),
        "https://example.com/login".to_string(),
        "https://example.com/dashboard".to_string(),
    ];
    assert!(detector.check_redirect_chain(&legitimate_chain).is_ok());

    // Direct circular loop: A -> B -> A
    let direct_cycle = vec![
        "https://example.com/page-a".to_string(),
        "https://example.com/page-b".to_string(),
        "https://example.com/page-a".to_string(),
    ];
    let res = detector.check_redirect_chain(&direct_cycle);
    assert!(res.is_err());
    match res.unwrap_err() {
        CrawlTrapReason::RedirectLoop { cycle } => {
            assert_eq!(cycle.len(), 3);
            assert_eq!(cycle[0], cycle[2]);
        }
        other => panic!("Expected RedirectLoop, got: {other:?}"),
    }

    // Multi-hop cycle: A -> B -> C -> D -> B
    let multihop_cycle = vec![
        "https://example.com/a".to_string(),
        "https://example.com/b".to_string(),
        "https://example.com/c".to_string(),
        "https://example.com/d".to_string(),
        "https://example.com/b".to_string(),
    ];
    let res_multi = detector.check_redirect_chain(&multihop_cycle);
    assert!(res_multi.is_err());
    match res_multi.unwrap_err() {
        CrawlTrapReason::RedirectLoop { cycle } => {
            assert_eq!(
                cycle,
                vec![
                    "https://example.com/b",
                    "https://example.com/c",
                    "https://example.com/d",
                    "https://example.com/b"
                ]
            );
        }
        other => panic!("Expected RedirectLoop, got: {other:?}"),
    }

    // Redirect chain too long (> 10 hops)
    let long_chain: Vec<String> = (0..=11)
        .map(|i| format!("https://example.com/step-{i}"))
        .collect();
    let res_long = detector.check_redirect_chain(&long_chain);
    assert!(res_long.is_err());
    match res_long.unwrap_err() {
        CrawlTrapReason::RedirectChainTooLong { length, max } => {
            assert_eq!(length, 12);
            assert_eq!(max, 10);
        }
        other => panic!("Expected RedirectChainTooLong, got: {other:?}"),
    }
}

#[test]
fn test_crawl_frontier_enqueue_and_deduplication() {
    let mut frontier = CrawlFrontier::new();

    let u1 = Url::parse("https://example.com/docs/guide").unwrap();
    let d1 = frontier.enqueue(&u1, 1, 0);
    assert!(d1.is_enqueued());
    assert_eq!(frontier.stats().pending, 1);
    assert_eq!(frontier.stats().duplicate_urls_skipped, 0);

    // Equivalent candidate URL with fragment and tracking parameters
    let u1_dup = Url::parse("https://example.com/docs/guide#overview?utm_source=twitter").unwrap();
    let d2 = frontier.enqueue(&u1_dup, 1, 0);
    assert!(!d2.is_enqueued());
    match d2 {
        FrontierEnqueueDecision::AlreadySeen {
            canonical_url,
            original_depth,
        } => {
            assert_eq!(canonical_url, "https://example.com/docs/guide");
            assert_eq!(original_depth, 1);
        }
        other => panic!("Expected AlreadySeen, got: {other:?}"),
    }
    assert_eq!(frontier.stats().pending, 1);
    assert_eq!(frontier.stats().duplicate_urls_skipped, 1);

    // Candidate seen at deeper depth is dropped
    let d3 = frontier.enqueue(&u1, 3, 0);
    assert!(matches!(d3, FrontierEnqueueDecision::AlreadySeen { .. }));
    assert_eq!(frontier.stats().duplicate_urls_skipped, 2);

    // Candidate seen at shorter depth is re-enqueued with improved depth
    let d4 = frontier.enqueue(&u1, 0, 0);
    assert!(d4.is_enqueued());
    assert_eq!(frontier.stats().pending, 2);
}

#[test]
fn test_crawl_frontier_trap_blocking() {
    let mut frontier = CrawlFrontier::new();

    // Trap URL with segment repeats
    let trap_url = Url::parse("https://example.com/docs/docs/docs/docs/docs").unwrap();
    let decision = frontier.enqueue(&trap_url, 0, 0);
    assert!(!decision.is_enqueued());
    assert!(matches!(
        decision,
        FrontierEnqueueDecision::TrapBlocked {
            reason: CrawlTrapReason::PathSegmentRepetition { .. },
            ..
        }
    ));
    assert_eq!(frontier.stats().traps_blocked, 1);
    assert_eq!(frontier.stats().pending, 0);
}

#[test]
fn test_crawl_frontier_priority_queue_order() {
    let mut frontier = CrawlFrontier::new();

    let u_low_pri = Url::parse("https://example.com/low-pri").unwrap();
    let u_high_pri = Url::parse("https://example.com/high-pri").unwrap();
    let u_deep = Url::parse("https://example.com/deep").unwrap();
    let u_shallow = Url::parse("https://example.com/shallow").unwrap();

    frontier.enqueue(&u_low_pri, 1, 2);
    frontier.enqueue(&u_high_pri, 1, 0);
    frontier.enqueue(&u_deep, 3, 1);
    frontier.enqueue(&u_shallow, 1, 1);

    // 1. High priority (priority = 0) dequeued first
    let item1 = frontier.pop_next().unwrap();
    assert_eq!(item1.canonical_url, "https://example.com/high-pri");
    assert_eq!(item1.priority, 0);

    // 2. Medium priority, shallow depth (priority = 1, depth = 1) dequeued before deep
    let item2 = frontier.pop_next().unwrap();
    assert_eq!(item2.canonical_url, "https://example.com/shallow");
    assert_eq!(item2.depth, 1);

    // 3. Medium priority, deeper depth (priority = 1, depth = 3)
    let item3 = frontier.pop_next().unwrap();
    assert_eq!(item3.canonical_url, "https://example.com/deep");
    assert_eq!(item3.depth, 3);

    // 4. Low priority (priority = 2) dequeued last
    let item4 = frontier.pop_next().unwrap();
    assert_eq!(item4.canonical_url, "https://example.com/low-pri");
    assert_eq!(item4.priority, 2);

    assert!(frontier.is_empty());
}

#[test]
fn test_crawl_frontier_content_deduplication_exact_blake3() {
    let mut frontier = CrawlFrontier::new();

    let text = "This is a substantial unique page document detailing the architecture of the web-extractor crawler. It has comprehensive explanations of links, restrictions, and duplicate detection.";

    let u1 = Url::parse("https://example.com/docs/intro").unwrap();
    let class1 = frontier.mark_visited(&u1, Some(text));
    assert_eq!(class1, DuplicateClassification::Unique);
    assert_eq!(frontier.stats().visited, 1);
    assert_eq!(frontier.stats().duplicate_content_skipped, 0);

    // Second URL serving identical content
    let u2 = Url::parse("https://example.com/docs/mirror-intro").unwrap();
    let class2 = frontier.mark_visited(&u2, Some(text));
    assert!(matches!(
        class2,
        DuplicateClassification::ExactContentDuplicate { .. }
    ));
    assert_eq!(frontier.stats().visited, 2);
    assert_eq!(frontier.stats().duplicate_content_skipped, 1);
}

#[test]
fn test_crawl_frontier_content_deduplication_near_duplicate_simhash() {
    let mut frontier = CrawlFrontier::with_options(
        CrawlerUrlNormalizer::new(),
        CrawlLoopDetector::new(),
        DedupOptions {
            max_hamming_distance: 4, // 4-bit distance = ~93.75% similarity
            min_tokens_for_simhash: 10,
            allow_cross_domain_canonical: true,
        },
    );

    let base_text = "Universal URL and GitHub repository to AI context engine. Converts web articles, documentation sites, and repositories into clean structured Markdown and JSON formats optimized for LLM token budgets. Fully self-hostable with Tokio and Axum.";

    // Subtle modification (e.g. minor footer timestamp or session info)
    let modified_text = "Universal URL and GitHub repository to AI context engine. Converts web articles, documentation sites, and repositories into clean structured Markdown and JSON formats optimized for LLM token budgets. Fully self-hostable with Tokio and Axum. Copyright 2026.";

    let u1 = Url::parse("https://example.com/page-1").unwrap();
    let res1 = frontier.mark_visited(&u1, Some(base_text));
    assert_eq!(res1, DuplicateClassification::Unique);

    let u2 = Url::parse("https://example.com/page-2").unwrap();
    let res2 = frontier.mark_visited(&u2, Some(modified_text));
    assert!(
        matches!(res2, DuplicateClassification::NearDuplicate { .. }),
        "Expected NearDuplicate, got: {res2:?}"
    );
    assert_eq!(frontier.stats().duplicate_content_skipped, 1);
}

#[test]
fn test_crawl_frontier_redirect_trace_and_error_handling() {
    let mut frontier = CrawlFrontier::new();

    let u1 = Url::parse("https://example.com/a").unwrap();
    let u2 = Url::parse("https://example.com/b").unwrap();
    let u3 = Url::parse("https://example.com/c").unwrap();

    // Normal hops
    assert!(frontier.record_redirect_hop(&u1, &u2).is_ok());
    assert!(frontier.record_redirect_hop(&u2, &u3).is_ok());

    // Circular hop back to /a
    let cycle_res = frontier.record_redirect_hop(&u3, &u1);
    assert!(cycle_res.is_err());
    assert!(matches!(
        cycle_res.unwrap_err(),
        CrawlTrapReason::RedirectLoop { .. }
    ));

    // Error handling with mark_failed
    let failed_url = Url::parse("https://example.com/broken").unwrap();
    frontier.mark_failed(&failed_url, "HTTP 500 Internal Server Error".to_string());
    assert_eq!(frontier.stats().failed, 1);

    // Reset clears everything
    frontier.reset();
    assert_eq!(frontier.stats().failed, 0);
    assert_eq!(frontier.stats().visited, 0);
    assert_eq!(frontier.stats().pending, 0);
    assert!(frontier.is_empty());
}

#[test]
fn test_crawl_frontier_stats_integrity() {
    let mut frontier = CrawlFrontier::new();

    // 1. Enqueue 2 valid URLs
    let u1 = Url::parse("https://example.com/page1").unwrap();
    let u2 = Url::parse("https://example.com/page2").unwrap();
    frontier.enqueue(&u1, 0, 0);
    frontier.enqueue(&u2, 1, 1);
    assert_eq!(frontier.stats().pending, 2);

    // 2. Enqueue 1 duplicate URL
    frontier.enqueue(&u1, 0, 0);
    assert_eq!(frontier.stats().duplicate_urls_skipped, 1);

    // 3. Enqueue 1 trap URL
    let u_trap = Url::parse("https://example.com/a/a/a/a/a").unwrap();
    frontier.enqueue(&u_trap, 0, 0);
    assert_eq!(frontier.stats().traps_blocked, 1);

    // 4. Pop and mark visited
    let item1 = frontier.pop_next().unwrap();
    frontier.mark_visited(&item1.url, Some("Content A"));
    assert_eq!(frontier.stats().visited, 1);
    assert_eq!(frontier.stats().pending, 1);

    // 5. Mark duplicate content
    let u3 = Url::parse("https://example.com/page3").unwrap();
    frontier.mark_visited(&u3, Some("Content A"));
    assert_eq!(frontier.stats().visited, 2);
    assert_eq!(frontier.stats().duplicate_content_skipped, 1);

    // 6. Mark failed
    let u_fail = Url::parse("https://example.com/timeout").unwrap();
    frontier.mark_failed(&u_fail, "ConnectTimeout".to_string());
    assert_eq!(frontier.stats().failed, 1);

    let stats = frontier.stats();
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.visited, 2);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.duplicate_urls_skipped, 1);
    assert_eq!(stats.duplicate_content_skipped, 1);
    assert_eq!(stats.traps_blocked, 1);
}
