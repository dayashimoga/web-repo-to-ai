//! Prometheus-compatible in-memory metrics registry.
//!
//! Provides thread-safe atomic counters, gauges, and histograms with standard
//! Prometheus text exposition format generation.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::RwLock;

/// Thread-safe atomic monotonic counter.
#[derive(Debug, Default)]
pub struct Counter {
    val: AtomicU64,
}

impl Counter {
    /// Creates a new counter initialized to 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            val: AtomicU64::new(0),
        }
    }

    /// Increments the counter by 1.
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Increments the counter by an arbitrary positive amount.
    pub fn inc_by(&self, amount: u64) {
        self.val.fetch_add(amount, Ordering::Relaxed);
    }

    /// Reads the current value of the counter.
    #[must_use]
    pub fn get(&self) -> u64 {
        self.val.load(Ordering::Relaxed)
    }
}

/// Thread-safe atomic gauge that can increase or decrease.
#[derive(Debug, Default)]
pub struct Gauge {
    val: AtomicI64,
}

impl Gauge {
    /// Creates a new gauge initialized to 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            val: AtomicI64::new(0),
        }
    }

    /// Sets the gauge to a specific value.
    pub fn set(&self, val: i64) {
        self.val.store(val, Ordering::Relaxed);
    }

    /// Increments the gauge by 1.
    pub fn inc(&self) {
        self.val.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the gauge by 1.
    pub fn dec(&self) {
        self.val.fetch_sub(1, Ordering::Relaxed);
    }

    /// Reads the current value of the gauge.
    #[must_use]
    pub fn get(&self) -> i64 {
        self.val.load(Ordering::Relaxed)
    }
}

/// Thread-safe bucketed histogram for tracking latency or size distributions.
#[derive(Debug)]
pub struct Histogram {
    buckets: Vec<f64>,
    bucket_counts: Vec<AtomicU64>,
    count: AtomicU64,
    sum: AtomicU64, // Stored as scaled micro-units or integer representation
}

impl Histogram {
    /// Creates a new histogram with custom bucket thresholds.
    #[must_use]
    pub fn new(buckets: Vec<f64>) -> Self {
        let mut sorted = buckets;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = sorted.len();
        let mut counts = Vec::with_capacity(len);
        for _ in 0..len {
            counts.push(AtomicU64::new(0));
        }

        Self {
            buckets: sorted,
            bucket_counts: counts,
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
        }
    }

    /// Observes a value, incrementing appropriate cumulative buckets.
    pub fn observe(&self, value: f64) {
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
        self.count.fetch_add(1, Ordering::Relaxed);
        let as_scaled = (value.max(0.0) * 1000.0) as u64;
        self.sum.fetch_add(as_scaled, Ordering::Relaxed);
    }

    /// Reads the total observation count.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Reads the total observation sum.
    #[must_use]
    pub fn sum(&self) -> f64 {
        (self.sum.load(Ordering::Relaxed) as f64) / 1000.0
    }
}

/// Labeled collection of counters.
#[derive(Debug, Default)]
pub struct CounterVec {
    metrics: RwLock<BTreeMap<String, Counter>>,
}

impl CounterVec {
    /// Increments the counter associated with the given label key.
    pub fn inc(&self, label_val: &str) {
        self.inc_by(label_val, 1);
    }

    /// Increments the counter associated with the given label key by an amount.
    pub fn inc_by(&self, label_val: &str, amount: u64) {
        {
            let read = self.metrics.read().unwrap();
            if let Some(counter) = read.get(label_val) {
                counter.inc_by(amount);
                return;
            }
        }
        let mut write = self.metrics.write().unwrap();
        let counter = write.entry(label_val.to_string()).or_default();
        counter.inc_by(amount);
    }

    /// Returns a snapshot of all label-value pairs and their current counts.
    #[must_use]
    pub fn snapshot(&self) -> BTreeMap<String, u64> {
        let read = self.metrics.read().unwrap();
        read.iter().map(|(k, v)| (k.clone(), v.get())).collect()
    }
}

/// Standard latency buckets in milliseconds.
pub const LATENCY_BUCKETS_MS: &[f64] = &[
    10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 30000.0,
];

/// Centralized application core telemetry metrics.
#[derive(Debug)]
pub struct AppMetrics {
    /// Total incoming HTTP requests by status or route label.
    pub http_requests_total: CounterVec,
    /// Total incoming HTTP request execution duration in milliseconds.
    pub http_request_duration_ms: Histogram,
    /// Total conversion jobs processed by status (`submitted`, `processing`, `completed`, `failed`, `cancelled`).
    pub jobs_total: CounterVec,
    /// Total conversion job duration in milliseconds.
    pub job_duration_ms: Histogram,
    /// Extracted files total by source type (`web`, `github`).
    pub extracted_files_total: CounterVec,
    /// Total tokens evaluated or budgeted.
    pub tokens_processed_total: Counter,
    /// Current storage footprint in bytes.
    pub storage_bytes_current: Gauge,
    /// Currently leased active worker jobs.
    pub active_workers: Gauge,
    /// Total SSRF blocked requests.
    pub ssrf_blocks_total: Counter,
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl AppMetrics {
    /// Creates a new metrics registry with zeroed initial metrics.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http_requests_total: CounterVec::default(),
            http_request_duration_ms: Histogram::new(LATENCY_BUCKETS_MS.to_vec()),
            jobs_total: CounterVec::default(),
            job_duration_ms: Histogram::new(LATENCY_BUCKETS_MS.to_vec()),
            extracted_files_total: CounterVec::default(),
            tokens_processed_total: Counter::new(),
            storage_bytes_current: Gauge::new(),
            active_workers: Gauge::new(),
            ssrf_blocks_total: Counter::new(),
        }
    }

    /// Renders all metrics in standard Prometheus exposition text format.
    #[must_use]
    pub fn render_prometheus_text(&self) -> String {
        let mut out = String::new();

        // http_requests_total
        out.push_str(
            "# HELP web_repo_to_ai_http_requests_total Total HTTP requests processed by status.\n",
        );
        out.push_str("# TYPE web_repo_to_ai_http_requests_total counter\n");
        let http_reqs = self.http_requests_total.snapshot();
        if http_reqs.is_empty() {
            out.push_str("web_repo_to_ai_http_requests_total{status=\"none\"} 0\n");
        } else {
            for (label, count) in http_reqs {
                out.push_str(&format!(
                    "web_repo_to_ai_http_requests_total{{{label}}} {count}\n"
                ));
            }
        }

        // http_request_duration_ms
        out.push_str(
            "# HELP web_repo_to_ai_http_request_duration_ms HTTP request latency in milliseconds.\n",
        );
        out.push_str("# TYPE web_repo_to_ai_http_request_duration_ms histogram\n");
        for (i, bucket) in self.http_request_duration_ms.buckets.iter().enumerate() {
            let count = self.http_request_duration_ms.bucket_counts[i].load(Ordering::Relaxed);
            out.push_str(&format!(
                "web_repo_to_ai_http_request_duration_ms_bucket{{le=\"{bucket}\"}} {count}\n"
            ));
        }
        out.push_str(&format!(
            "web_repo_to_ai_http_request_duration_ms_bucket{{le=\"+Inf\"}} {}\n",
            self.http_request_duration_ms.count()
        ));
        out.push_str(&format!(
            "web_repo_to_ai_http_request_duration_ms_sum {:.3}\n",
            self.http_request_duration_ms.sum()
        ));
        out.push_str(&format!(
            "web_repo_to_ai_http_request_duration_ms_count {}\n",
            self.http_request_duration_ms.count()
        ));

        // jobs_total
        out.push_str(
            "# HELP web_repo_to_ai_jobs_total Total conversion jobs processed by status.\n",
        );
        out.push_str("# TYPE web_repo_to_ai_jobs_total counter\n");
        let jobs = self.jobs_total.snapshot();
        if jobs.is_empty() {
            out.push_str("web_repo_to_ai_jobs_total{status=\"none\"} 0\n");
        } else {
            for (status, count) in jobs {
                out.push_str(&format!(
                    "web_repo_to_ai_jobs_total{{status=\"{}\"}} {}\n",
                    status, count
                ));
            }
        }

        // job_duration_ms
        out.push_str("# HELP web_repo_to_ai_job_duration_ms Job duration in milliseconds.\n");
        out.push_str("# TYPE web_repo_to_ai_job_duration_ms histogram\n");
        for (i, bucket) in self.job_duration_ms.buckets.iter().enumerate() {
            let count = self.job_duration_ms.bucket_counts[i].load(Ordering::Relaxed);
            out.push_str(&format!(
                "web_repo_to_ai_job_duration_ms_bucket{{le=\"{}\"}} {}\n",
                bucket, count
            ));
        }
        out.push_str(&format!(
            "web_repo_to_ai_job_duration_ms_bucket{{le=\"+Inf\"}} {}\n",
            self.job_duration_ms.count()
        ));
        out.push_str(&format!(
            "web_repo_to_ai_job_duration_ms_sum {:.3}\n",
            self.job_duration_ms.sum()
        ));
        out.push_str(&format!(
            "web_repo_to_ai_job_duration_ms_count {}\n",
            self.job_duration_ms.count()
        ));

        // extracted_files_total
        out.push_str(
            "# HELP web_repo_to_ai_extracted_files_total Total files extracted by source type.\n",
        );
        out.push_str("# TYPE web_repo_to_ai_extracted_files_total counter\n");
        let files = self.extracted_files_total.snapshot();
        if files.is_empty() {
            out.push_str("web_repo_to_ai_extracted_files_total{source_type=\"none\"} 0\n");
        } else {
            for (src, count) in files {
                out.push_str(&format!(
                    "web_repo_to_ai_extracted_files_total{{source_type=\"{}\"}} {}\n",
                    src, count
                ));
            }
        }

        // tokens_processed_total
        out.push_str("# HELP web_repo_to_ai_tokens_processed_total Total tokens evaluated.\n");
        out.push_str("# TYPE web_repo_to_ai_tokens_processed_total counter\n");
        out.push_str(&format!(
            "web_repo_to_ai_tokens_processed_total {}\n",
            self.tokens_processed_total.get()
        ));

        // storage_bytes_current
        out.push_str("# HELP web_repo_to_ai_storage_bytes_current Current artifact storage usage in bytes.\n");
        out.push_str("# TYPE web_repo_to_ai_storage_bytes_current gauge\n");
        out.push_str(&format!(
            "web_repo_to_ai_storage_bytes_current {}\n",
            self.storage_bytes_current.get()
        ));

        // active_workers
        out.push_str(
            "# HELP web_repo_to_ai_active_workers Currently active worker lease executions.\n",
        );
        out.push_str("# TYPE web_repo_to_ai_active_workers gauge\n");
        out.push_str(&format!(
            "web_repo_to_ai_active_workers {}\n",
            self.active_workers.get()
        ));

        // ssrf_blocks_total
        out.push_str("# HELP web_repo_to_ai_ssrf_blocks_total Total SSRF blocked requests.\n");
        out.push_str("# TYPE web_repo_to_ai_ssrf_blocks_total counter\n");
        out.push_str(&format!(
            "web_repo_to_ai_ssrf_blocks_total {}\n",
            self.ssrf_blocks_total.get()
        ));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_and_gauge() {
        let counter = Counter::new();
        counter.inc();
        counter.inc_by(4);
        assert_eq!(counter.get(), 5);

        let gauge = Gauge::new();
        gauge.set(10);
        gauge.inc();
        gauge.dec();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram_observation() {
        let hist = Histogram::new(vec![10.0, 50.0, 100.0]);
        hist.observe(25.0);
        hist.observe(75.0);
        assert_eq!(hist.count(), 2);
        assert_eq!(hist.sum(), 100.0);
    }

    #[test]
    fn test_prometheus_exposition_rendering() {
        let metrics = AppMetrics::new();
        metrics.jobs_total.inc("completed");
        metrics.jobs_total.inc("completed");
        metrics.jobs_total.inc("failed");
        metrics.storage_bytes_current.set(1024 * 1024);
        metrics.tokens_processed_total.inc_by(50_000);

        let rendered = metrics.render_prometheus_text();
        assert!(rendered.contains("web_repo_to_ai_jobs_total{status=\"completed\"} 2"));
        assert!(rendered.contains("web_repo_to_ai_jobs_total{status=\"failed\"} 1"));
        assert!(rendered.contains("web_repo_to_ai_storage_bytes_current 1048576"));
        assert!(rendered.contains("web_repo_to_ai_tokens_processed_total 50000"));
    }
}
