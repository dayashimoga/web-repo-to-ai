//! # Crawl Frontier and Deduplication Manager
//!
//! Provides frontier queue management, crawler-specific URL canonicalization,
//! crawler trap filtering, and content deduplication (exact BLAKE3 and near-duplicate SimHash).

use std::collections::{BinaryHeap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use url::Url;

use super::loop_detector::{CrawlLoopDetector, CrawlTrapReason};
use crate::dedup::model::{DedupOptions, DuplicateClassification};
use crate::dedup::registry::DuplicateRegistry;

/// Index filenames normalized to directory root if found at the end of a URL path.
pub const DEFAULT_INDEX_FILENAMES: &[&str] = &[
    "index.html",
    "index.htm",
    "index.php",
    "index.asp",
    "index.aspx",
    "default.html",
    "default.htm",
    "default.asp",
    "default.aspx",
];

/// Configuration governing crawler canonical URL normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalUrlNormalizerConfig {
    /// Strip default network ports (`80` for http, `443` for https).
    pub strip_default_ports: bool,
    /// Sort query parameter keys alphabetically for deterministic ordering.
    pub sort_query_params: bool,
    /// Strip marketing and analytics tracking query parameters (`utm_*`, `fbclid`, etc.).
    pub strip_tracking_params: bool,
    /// Normalize default index filenames (e.g. `/docs/index.html` -> `/docs/`).
    pub normalize_index_files: bool,
    /// Strip trailing slashes on non-root paths (e.g. `/about/` -> `/about`).
    pub strip_trailing_slashes: bool,
    /// Strip URL anchor fragment (`#heading`).
    pub strip_fragments: bool,
}

impl Default for CanonicalUrlNormalizerConfig {
    fn default() -> Self {
        Self {
            strip_default_ports: true,
            sort_query_params: true,
            strip_tracking_params: true,
            normalize_index_files: true,
            strip_trailing_slashes: true,
            strip_fragments: true,
        }
    }
}

/// Normalizes URLs into deterministic canonical forms specifically tailored for crawling deduplication.
#[derive(Debug, Clone, Default)]
pub struct CrawlerUrlNormalizer {
    config: CanonicalUrlNormalizerConfig,
}

impl CrawlerUrlNormalizer {
    /// Constructs a normalizer with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a normalizer with custom settings.
    #[must_use]
    pub fn with_config(config: CanonicalUrlNormalizerConfig) -> Self {
        Self { config }
    }

    /// Normalizes a candidate URL into its canonical deduplicated representation.
    #[must_use]
    pub fn normalize(&self, raw: &Url) -> Url {
        let mut url = raw.clone();

        // 1. Strip in-page section fragment
        if self.config.strip_fragments {
            url.set_fragment(None);
        }

        // 2. Strip default ports
        if self.config.strip_default_ports {
            if let Some(port) = url.port() {
                let scheme = url.scheme();
                if (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
                    let _ = url.set_port(None);
                }
            }
        }

        // 3. Normalize path: index files and trailing slashes
        let mut path = url.path().to_string();

        if self.config.normalize_index_files {
            for &idx_file in DEFAULT_INDEX_FILENAMES {
                if path.ends_with(idx_file) {
                    let prefix_len = path.len() - idx_file.len();
                    if prefix_len > 0 && path.as_bytes()[prefix_len - 1] == b'/' {
                        path.truncate(prefix_len);
                        break;
                    }
                }
            }
        }

        if self.config.strip_trailing_slashes && path.len() > 1 && path.ends_with('/') {
            path.pop();
        }

        url.set_path(&path);

        // 4. Normalize query parameters: strip tracking and sort keys
        if url.query().is_some() {
            let mut pairs: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(k, _)| {
                    if self.config.strip_tracking_params {
                        !is_tracking_param(k)
                    } else {
                        true
                    }
                })
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if pairs.is_empty() {
                url.set_query(None);
            } else {
                if self.config.sort_query_params {
                    pairs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                }
                let mut ser = url::form_urlencoded::Serializer::new(String::new());
                for (k, v) in pairs {
                    ser.append_pair(&k, &v);
                }
                url.set_query(Some(&ser.finish()));
            }
        }

        url
    }
}

/// Identifies if a query parameter key is marketing tracking noise.
fn is_tracking_param(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.starts_with("utm_")
        || lower == "fbclid"
        || lower == "gclid"
        || lower == "msclkid"
        || lower == "mc_cid"
        || lower == "mc_eid"
        || lower == "_hsenc"
        || lower == "_hsmi"
        || lower == "ref"
        || lower == "ref_src"
}

/// A prioritized crawl item queued in the frontier.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrawlQueueItem {
    /// Canonical normalized URL ready for acquisition.
    pub url: Url,
    /// Canonical string identifier for duplicate index lookup.
    pub canonical_url: String,
    /// Crawl depth from the seed URL.
    pub depth: usize,
    /// Priority level (lower numeric value = higher dequeuing priority).
    pub priority: usize,
}

impl Ord for CrawlQueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap: lowest priority number, lowest depth, then alphabetical canonical URL
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.depth.cmp(&self.depth))
            .then_with(|| other.canonical_url.cmp(&self.canonical_url))
    }
}

impl PartialOrd for CrawlQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Decision outcome when evaluating a candidate URL for inclusion in the frontier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrontierEnqueueDecision {
    /// Candidate is novel and successfully enqueued.
    Enqueued {
        /// Canonical normalized URL.
        canonical_url: Url,
        /// Assigned depth.
        depth: usize,
        /// Assigned priority.
        priority: usize,
    },
    /// Candidate was already enqueued or visited at an equal or lower depth.
    AlreadySeen {
        /// Canonical URL string.
        canonical_url: String,
        /// Original depth recorded.
        original_depth: usize,
    },
    /// Candidate was rejected because a crawler trap was identified.
    TrapBlocked {
        /// Offending URL.
        url: Url,
        /// Specific trap detected.
        reason: CrawlTrapReason,
    },
}

impl FrontierEnqueueDecision {
    /// Returns `true` if the candidate was enqueued.
    #[must_use]
    pub fn is_enqueued(&self) -> bool {
        matches!(self, Self::Enqueued { .. })
    }
}

/// Real-time metrics detailing crawler frontier status and deduplication efficiency.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontierStats {
    /// Current pending items queued in the frontier awaiting crawl.
    pub pending: usize,
    /// Total unique URLs marked as successfully visited.
    pub visited: usize,
    /// Total URLs marked as failed.
    pub failed: usize,
    /// Number of candidate URLs dropped as duplicate URLs.
    pub duplicate_urls_skipped: usize,
    /// Number of visited pages identified as exact or near duplicate content.
    pub duplicate_content_skipped: usize,
    /// Number of candidate URLs rejected as crawler traps or loops.
    pub traps_blocked: usize,
}

/// Stateful frontier engine orchestrating queue priorities, deduplication, and trap prevention.
#[derive(Debug, Clone, Default)]
pub struct CrawlFrontier {
    normalizer: CrawlerUrlNormalizer,
    loop_detector: CrawlLoopDetector,
    duplicate_registry: DuplicateRegistry,
    seen_canonical_urls: HashMap<String, usize>,
    visited_canonical_urls: HashSet<String>,
    failed_canonical_urls: HashMap<String, String>,
    queue: BinaryHeap<CrawlQueueItem>,
    redirect_traces: HashMap<String, Vec<String>>,
    stats: FrontierStats,
}

impl CrawlFrontier {
    /// Constructs a new `CrawlFrontier` with default normalizer and loop detector settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a `CrawlFrontier` with custom components.
    #[must_use]
    pub fn with_options(
        normalizer: CrawlerUrlNormalizer,
        loop_detector: CrawlLoopDetector,
        dedup_options: DedupOptions,
    ) -> Self {
        Self {
            normalizer,
            loop_detector,
            duplicate_registry: DuplicateRegistry::with_options(dedup_options),
            seen_canonical_urls: HashMap::new(),
            visited_canonical_urls: HashSet::new(),
            failed_canonical_urls: HashMap::new(),
            queue: BinaryHeap::new(),
            redirect_traces: HashMap::new(),
            stats: FrontierStats::default(),
        }
    }

    /// Evaluates, canonicalizes, and enqueues a candidate URL into the frontier queue.
    pub fn enqueue(
        &mut self,
        raw_url: &Url,
        depth: usize,
        priority: usize,
    ) -> FrontierEnqueueDecision {
        // 1. Loop and crawler trap evaluation
        if let Err(trap_reason) = self.loop_detector.evaluate_url(raw_url) {
            self.stats.traps_blocked += 1;
            return FrontierEnqueueDecision::TrapBlocked {
                url: raw_url.clone(),
                reason: trap_reason,
            };
        }

        // 2. Canonical URL normalization
        let canonical_url = self.normalizer.normalize(raw_url);
        let canonical_key = canonical_url.as_str().to_string();

        // 3. Duplicate check against seen URLs
        if let Some(&existing_depth) = self.seen_canonical_urls.get(&canonical_key) {
            if depth >= existing_depth {
                self.stats.duplicate_urls_skipped += 1;
                return FrontierEnqueueDecision::AlreadySeen {
                    canonical_url: canonical_key,
                    original_depth: existing_depth,
                };
            }
            // Candidate was reached via a shorter path; update recorded depth and re-queue
            self.seen_canonical_urls
                .insert(canonical_key.clone(), depth);
        } else {
            self.seen_canonical_urls
                .insert(canonical_key.clone(), depth);
        }

        // 4. Push item into priority queue
        let item = CrawlQueueItem {
            url: canonical_url.clone(),
            canonical_url: canonical_key,
            depth,
            priority,
        };
        self.queue.push(item);
        self.stats.pending = self.queue.len();

        FrontierEnqueueDecision::Enqueued {
            canonical_url,
            depth,
            priority,
        }
    }

    /// Dequeues the next highest-priority URL from the frontier.
    pub fn pop_next(&mut self) -> Option<CrawlQueueItem> {
        let item = self.queue.pop();
        self.stats.pending = self.queue.len();
        item
    }

    /// Marks a URL as successfully acquired, registering its canonical URL and inspecting content duplicates.
    pub fn mark_visited(
        &mut self,
        url: &Url,
        text_content: Option<&str>,
    ) -> DuplicateClassification {
        let canonical_url = self.normalizer.normalize(url);
        let canonical_key = canonical_url.as_str().to_string();

        self.visited_canonical_urls.insert(canonical_key.clone());
        self.stats.visited = self.visited_canonical_urls.len();

        if let Some(text) = text_content {
            let mut hasher = blake3::Hasher::new();
            hasher.update(text.as_bytes());
            let blake3_hex = hasher.finalize().to_hex().to_string();

            let classification =
                self.duplicate_registry
                    .check_and_register(&canonical_key, &blake3_hex, text);

            if !classification.is_unique() {
                self.stats.duplicate_content_skipped += 1;
            }

            classification
        } else {
            DuplicateClassification::Unique
        }
    }

    /// Marks a URL as failed to prevent re-enqueue loops.
    pub fn mark_failed(&mut self, url: &Url, error_message: String) {
        let canonical_url = self.normalizer.normalize(url);
        let canonical_key = canonical_url.as_str().to_string();
        self.failed_canonical_urls
            .insert(canonical_key, error_message);
        self.stats.failed = self.failed_canonical_urls.len();
    }

    /// Records a redirect hop from one URL to another and asserts circular redirect loops.
    pub fn record_redirect_hop(&mut self, from: &Url, to: &Url) -> Result<(), CrawlTrapReason> {
        let from_key = self.normalizer.normalize(from).as_str().to_string();
        let to_key = self.normalizer.normalize(to).as_str().to_string();

        let mut chain = self
            .redirect_traces
            .remove(&from_key)
            .unwrap_or_else(|| vec![from_key.clone()]);
        chain.push(to_key.clone());

        // Validate redirect chain against loop detector
        self.loop_detector.check_redirect_chain(&chain)?;

        self.redirect_traces.insert(to_key, chain);
        Ok(())
    }

    /// Returns `true` if the queue has no pending items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of items remaining in the frontier queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns a reference to the active frontier statistics.
    #[must_use]
    pub fn stats(&self) -> &FrontierStats {
        &self.stats
    }

    /// Clears the frontier queue and resets tracking state.
    pub fn reset(&mut self) {
        self.seen_canonical_urls.clear();
        self.visited_canonical_urls.clear();
        self.failed_canonical_urls.clear();
        self.queue.clear();
        self.redirect_traces.clear();
        self.stats = FrontierStats::default();
    }
}
