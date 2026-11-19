//! # Crawler Loop and Trap Detection Engine
//!
//! Protects website crawlers against infinite URL traps, repeated directory patterns,
//! cyclic path sequences, query parameter combinatorial explosion, and redirect cycles.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

/// Default maximum number of times an identical path segment may appear anywhere in a single URL path.
pub const DEFAULT_MAX_SEGMENT_REPEATS: usize = 3;

/// Default maximum number of path segments allowable in a candidate URL.
pub const DEFAULT_MAX_PATH_SEGMENTS: usize = 16;

/// Default maximum allowable query parameter key-value pairs before flagging a query trap.
pub const DEFAULT_MAX_QUERY_PARAMS: usize = 10;

/// Default maximum occurrences of the exact same query parameter key in a single URL.
pub const DEFAULT_MAX_QUERY_KEY_REPEATS: usize = 2;

/// Default maximum allowable redirect transitions in a crawler redirect chain.
pub const DEFAULT_MAX_REDIRECT_CHAIN_LEN: usize = 10;

/// Reason identifying an identified crawler trap or loop pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrawlTrapReason {
    /// A single path segment appeared repeatedly in the path beyond the allowed threshold.
    PathSegmentRepetition {
        /// The repeating path segment.
        segment: String,
        /// Number of times the segment appeared.
        count: usize,
        /// Maximum allowable threshold.
        threshold: usize,
    },
    /// A multi-segment cyclic pattern repeated multiple times (e.g. `/cat/books/cat/books`).
    CyclicPathSequence {
        /// Repeating sequence pattern.
        pattern: Vec<String>,
        /// Number of contiguous repetitions detected.
        repetitions: usize,
    },
    /// Path exceeds allowable segment depth count.
    ExcessivePathSegments {
        /// Total segments in path.
        count: usize,
        /// Maximum allowed depth.
        limit: usize,
    },
    /// URL query string has too many key-value pairs.
    ExcessiveQueryParameters {
        /// Observed parameter count.
        count: usize,
        /// Allowed parameter limit.
        limit: usize,
    },
    /// A single query key appears too many times in the URL query string.
    RepeatedQueryKey {
        /// Offending query parameter key.
        key: String,
        /// Observed occurrences.
        count: usize,
        /// Allowed limit.
        limit: usize,
    },
    /// A redirect cycle was detected where a target redirects back to an earlier URL in the chain.
    RedirectLoop {
        /// Chain of URLs leading to the circular reference.
        cycle: Vec<String>,
    },
    /// Redirect chain exceeded the maximum permissible number of redirect hops.
    RedirectChainTooLong {
        /// Observed chain length.
        length: usize,
        /// Maximum allowable length.
        max: usize,
    },
}

impl std::fmt::Display for CrawlTrapReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathSegmentRepetition {
                segment,
                count,
                threshold,
            } => {
                write!(
                    f,
                    "Path segment '{segment}' repeated {count} times (threshold: {threshold})"
                )
            }
            Self::CyclicPathSequence {
                pattern,
                repetitions,
            } => {
                write!(
                    f,
                    "Cyclic path pattern [{}] repeated {repetitions} times",
                    pattern.join("/")
                )
            }
            Self::ExcessivePathSegments { count, limit } => {
                write!(f, "Excessive path segment depth ({count} > {limit})")
            }
            Self::ExcessiveQueryParameters { count, limit } => {
                write!(f, "Excessive query parameter count ({count} > {limit})")
            }
            Self::RepeatedQueryKey { key, count, limit } => {
                write!(
                    f,
                    "Query key '{key}' repeated {count} times (limit: {limit})"
                )
            }
            Self::RedirectLoop { cycle } => {
                write!(f, "Redirect loop detected: {}", cycle.join(" -> "))
            }
            Self::RedirectChainTooLong { length, max } => {
                write!(f, "Redirect chain exceeded maximum hops ({length} > {max})")
            }
        }
    }
}

/// Configuration settings governing crawler loop and trap detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopDetectorConfig {
    /// Maximum allowable occurrences of an identical path segment.
    pub max_segment_repeats: usize,
    /// Maximum allowable path segment depth.
    pub max_path_segments: usize,
    /// Maximum query parameter count.
    pub max_query_params: usize,
    /// Maximum repetitions of an individual query key.
    pub max_query_key_repeats: usize,
    /// Maximum allowable redirect hops in a crawler trace.
    pub max_redirect_chain_len: usize,
    /// Whether to check for multi-segment cyclic patterns.
    pub detect_cyclic_sequences: bool,
    /// Minimum sequence length to consider for cyclic pattern detection.
    pub min_cycle_len: usize,
    /// Maximum sequence length to consider for cyclic pattern detection.
    pub max_cycle_len: usize,
    /// Contiguous repetitions of a sequence required to trigger a trap warning.
    pub cycle_repeat_threshold: usize,
}

impl Default for LoopDetectorConfig {
    fn default() -> Self {
        Self {
            max_segment_repeats: DEFAULT_MAX_SEGMENT_REPEATS,
            max_path_segments: DEFAULT_MAX_PATH_SEGMENTS,
            max_query_params: DEFAULT_MAX_QUERY_PARAMS,
            max_query_key_repeats: DEFAULT_MAX_QUERY_KEY_REPEATS,
            max_redirect_chain_len: DEFAULT_MAX_REDIRECT_CHAIN_LEN,
            detect_cyclic_sequences: true,
            min_cycle_len: 2,
            max_cycle_len: 4,
            cycle_repeat_threshold: 2,
        }
    }
}

impl LoopDetectorConfig {
    /// Sets the maximum allowable repeats of an identical path segment.
    #[must_use]
    pub fn with_max_segment_repeats(mut self, max: usize) -> Self {
        self.max_segment_repeats = max.max(1);
        self
    }

    /// Sets the maximum allowable path depth.
    #[must_use]
    pub fn with_max_path_segments(mut self, max: usize) -> Self {
        self.max_path_segments = max.max(1);
        self
    }

    /// Sets the maximum allowable query parameters.
    #[must_use]
    pub fn with_max_query_params(mut self, max: usize) -> Self {
        self.max_query_params = max;
        self
    }

    /// Sets the maximum allowable repetitions of a query key.
    #[must_use]
    pub fn with_max_query_key_repeats(mut self, max: usize) -> Self {
        self.max_query_key_repeats = max.max(1);
        self
    }

    /// Sets the maximum redirect chain length.
    #[must_use]
    pub fn with_max_redirect_chain_len(mut self, max: usize) -> Self {
        self.max_redirect_chain_len = max.max(1);
        self
    }

    /// Controls cyclic sequence detection.
    #[must_use]
    pub fn with_detect_cyclic_sequences(mut self, detect: bool) -> Self {
        self.detect_cyclic_sequences = detect;
        self
    }
}

/// Evaluator that inspects URLs, paths, queries, and redirect traces for crawler traps.
#[derive(Debug, Clone, Default)]
pub struct CrawlLoopDetector {
    config: LoopDetectorConfig,
}

impl CrawlLoopDetector {
    /// Constructs a new `CrawlLoopDetector` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new `CrawlLoopDetector` with custom configuration.
    #[must_use]
    pub fn with_config(config: LoopDetectorConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &LoopDetectorConfig {
        &self.config
    }

    /// Evaluates a candidate URL for path repetition, cyclic patterns, and query parameter traps.
    pub fn evaluate_url(&self, url: &Url) -> Result<(), CrawlTrapReason> {
        self.check_path(url.path())?;
        self.check_query(url)?;
        Ok(())
    }

    /// Checks a URL path string for segment repetition, cyclic patterns, and excessive depth.
    pub fn check_path(&self, path: &str) -> Result<(), CrawlTrapReason> {
        let segments: Vec<&str> = path
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        // 1. Check total path segment depth
        if segments.len() > self.config.max_path_segments {
            return Err(CrawlTrapReason::ExcessivePathSegments {
                count: segments.len(),
                limit: self.config.max_path_segments,
            });
        }

        // 2. Check individual segment frequency
        let mut freq_map: HashMap<&str, usize> = HashMap::new();
        for &seg in &segments {
            let count = freq_map.entry(seg).or_insert(0);
            *count += 1;
            if *count > self.config.max_segment_repeats {
                return Err(CrawlTrapReason::PathSegmentRepetition {
                    segment: seg.to_string(),
                    count: *count,
                    threshold: self.config.max_segment_repeats,
                });
            }
        }

        // 3. Check for contiguous cyclic sequence patterns (e.g. [a, b, a, b])
        if self.config.detect_cyclic_sequences && segments.len() >= 4 {
            let max_k = (segments.len() / 2).min(self.config.max_cycle_len);
            for k in self.config.min_cycle_len..=max_k {
                if let Some((pattern, reps)) = find_cyclic_sequence(&segments, k) {
                    if reps >= self.config.cycle_repeat_threshold {
                        return Err(CrawlTrapReason::CyclicPathSequence {
                            pattern,
                            repetitions: reps,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks a URL query string for parameter count and duplicate key traps.
    pub fn check_query(&self, url: &Url) -> Result<(), CrawlTrapReason> {
        let pairs: Vec<_> = url.query_pairs().collect();

        // Check total query parameters count
        if pairs.len() > self.config.max_query_params {
            return Err(CrawlTrapReason::ExcessiveQueryParameters {
                count: pairs.len(),
                limit: self.config.max_query_params,
            });
        }

        // Check duplicate query parameter keys
        let mut key_counts: HashMap<&str, usize> = HashMap::new();
        for (k, _) in &pairs {
            let count = key_counts.entry(k.as_ref()).or_insert(0);
            *count += 1;
            if *count > self.config.max_query_key_repeats {
                return Err(CrawlTrapReason::RepeatedQueryKey {
                    key: k.to_string(),
                    count: *count,
                    limit: self.config.max_query_key_repeats,
                });
            }
        }

        Ok(())
    }

    /// Inspects a sequence of redirect URLs, checking for circular loops and excessive hops.
    pub fn check_redirect_chain(&self, chain: &[String]) -> Result<(), CrawlTrapReason> {
        if chain.len() > self.config.max_redirect_chain_len {
            return Err(CrawlTrapReason::RedirectChainTooLong {
                length: chain.len(),
                max: self.config.max_redirect_chain_len,
            });
        }

        // Check for cycle: if any URL is seen more than once
        let mut seen_indices = HashMap::new();
        for (idx, url_str) in chain.iter().enumerate() {
            if let Some(&first_idx) = seen_indices.get(url_str) {
                let cycle_slice = chain[first_idx..=idx].to_vec();
                return Err(CrawlTrapReason::RedirectLoop { cycle: cycle_slice });
            }
            seen_indices.insert(url_str, idx);
        }

        Ok(())
    }
}

/// Identifies if a slice of path segments contains a contiguous repetition of length `k`.
fn find_cyclic_sequence(segments: &[&str], k: usize) -> Option<(Vec<String>, usize)> {
    if segments.len() < k * 2 {
        return None;
    }

    for start_idx in 0..=(segments.len() - k * 2) {
        let pattern = &segments[start_idx..start_idx + k];
        let mut contiguous_reps = 1;
        let mut offset = start_idx + k;

        while offset + k <= segments.len() {
            if &segments[offset..offset + k] == pattern {
                contiguous_reps += 1;
                offset += k;
            } else {
                break;
            }
        }

        if contiguous_reps >= 2 {
            let pattern_strings: Vec<String> = pattern.iter().map(ToString::to_string).collect();
            return Some((pattern_strings, contiguous_reps));
        }
    }

    None
}
