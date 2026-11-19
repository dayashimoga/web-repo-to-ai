//! Token bucket and sliding window rate limiting engine.
//!
//! Provides [`RateLimitDecision`], [`RateLimitTier`], [`TokenBucket`],
//! [`SlidingWindowCounter`], [`RateLimiter`] trait, and [`InMemoryTieredRateLimiter`].

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::config::RateLimitConfig;

/// Outcome of a rate limiting evaluation on an incoming request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitDecision {
    /// Request is allowed within the configured quota.
    Allowed {
        /// Maximum permitted quota in the current window or burst capacity.
        limit: u32,
        /// Remaining tokens or requests available in the bucket.
        remaining: u32,
        /// Seconds until the quota completely refills or window resets.
        reset_seconds: u64,
    },
    /// Request exceeds allowable quota; client should be throttled.
    Exceeded {
        /// Maximum permitted quota.
        limit: u32,
        /// Number of seconds client must wait before retrying (for `Retry-After`).
        retry_after_seconds: u64,
        /// Seconds until the next window reset.
        reset_seconds: u64,
    },
}

impl RateLimitDecision {
    /// Returns `true` if the request is allowed to proceed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    /// Returns the maximum allowable quota limit.
    #[must_use]
    pub const fn limit(&self) -> u32 {
        match self {
            Self::Allowed { limit, .. } | Self::Exceeded { limit, .. } => *limit,
        }
    }

    /// Returns the number of remaining requests or tokens.
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        match self {
            Self::Allowed { remaining, .. } => *remaining,
            Self::Exceeded { .. } => 0,
        }
    }

    /// Returns the mandatory wait time in seconds if throttled.
    #[must_use]
    pub const fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            Self::Allowed { .. } => None,
            Self::Exceeded {
                retry_after_seconds,
                ..
            } => Some(*retry_after_seconds),
        }
    }

    /// Returns the duration until full replenishment or reset.
    #[must_use]
    pub const fn reset_seconds(&self) -> u64 {
        match self {
            Self::Allowed { reset_seconds, .. } | Self::Exceeded { reset_seconds, .. } => {
                *reset_seconds
            }
        }
    }
}

/// Client tier used to assign differentiated rate limiting ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RateLimitTier {
    /// Authenticated tenant (API key Bearer token or valid session).
    #[default]
    Authenticated,
    /// Unauthenticated caller identified solely by IP address.
    Anonymous,
    /// Administrative principal with unrestricted access or elevated quota.
    Admin,
}

/// Token bucket algorithm maintaining sub-second fractional replenishment.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum burst capacity.
    pub capacity: f64,
    /// Refill rate in tokens per second.
    pub refill_rate_per_sec: f64,
    /// Current available fractional tokens.
    pub tokens: f64,
    /// Timestamp when tokens were last replenished.
    pub last_refill: Instant,
}

impl TokenBucket {
    /// Creates a new token bucket with given burst capacity and requests per minute refill rate.
    #[must_use]
    pub fn new(capacity: u32, requests_per_minute: u32, now: Instant) -> Self {
        let cap = f64::from(capacity.max(1));
        let refill_rate = f64::from(requests_per_minute.max(1)) / 60.0;
        Self {
            capacity: cap,
            refill_rate_per_sec: refill_rate,
            tokens: cap,
            last_refill: now,
        }
    }

    /// Evaluates and consumes tokens for a request.
    pub fn consume(&mut self, now: Instant, cost: f64) -> RateLimitDecision {
        // Replenish tokens based on elapsed duration
        let elapsed = now
            .saturating_duration_since(self.last_refill)
            .as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate_per_sec).min(self.capacity);
        self.last_refill = now;

        let limit = self.capacity.round() as u32;

        if self.tokens >= cost {
            self.tokens -= cost;
            let remaining = self.tokens.floor().max(0.0) as u32;
            let time_to_full = ((self.capacity - self.tokens) / self.refill_rate_per_sec).ceil();
            let reset_seconds = time_to_full.max(1.0) as u64;

            RateLimitDecision::Allowed {
                limit,
                remaining,
                reset_seconds,
            }
        } else {
            let needed = cost - self.tokens;
            let retry_after = (needed / self.refill_rate_per_sec).ceil();
            let retry_after_seconds = (retry_after as u64).max(1);
            let time_to_full = ((self.capacity - self.tokens) / self.refill_rate_per_sec).ceil();
            let reset_seconds = (time_to_full as u64).max(retry_after_seconds);

            RateLimitDecision::Exceeded {
                limit,
                retry_after_seconds,
                reset_seconds,
            }
        }
    }
}

/// Sliding window counter tracking request frequency across continuous window boundaries.
#[derive(Debug, Clone)]
pub struct SlidingWindowCounter {
    /// Window duration in seconds.
    pub window_seconds: u64,
    /// Maximum allowable requests per window.
    pub max_requests: u32,
    /// Prior window bucket index.
    pub prev_bucket: u64,
    /// Prior window request count.
    pub prev_count: u32,
    /// Current window bucket index.
    pub curr_bucket: u64,
    /// Current window request count.
    pub curr_count: u32,
}

impl SlidingWindowCounter {
    /// Creates a new sliding window counter with given request limit and window duration.
    #[must_use]
    pub const fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            window_seconds: if window_seconds == 0 {
                60
            } else {
                window_seconds
            },
            max_requests: if max_requests == 0 { 1 } else { max_requests },
            prev_bucket: 0,
            prev_count: 0,
            curr_bucket: 0,
            curr_count: 0,
        }
    }

    /// Evaluates and increments requests against the sliding window.
    pub fn consume(&mut self, now_unix_secs: u64, cost: u32) -> RateLimitDecision {
        let current_bucket = now_unix_secs / self.window_seconds;

        if current_bucket == self.curr_bucket {
            // Same window
        } else if current_bucket == self.curr_bucket + 1 {
            // Advanced by exactly 1 window: slide current into previous
            self.prev_bucket = self.curr_bucket;
            self.prev_count = self.curr_count;
            self.curr_bucket = current_bucket;
            self.curr_count = 0;
        } else {
            // Advanced by more than 1 window: reset both
            self.prev_bucket = current_bucket.saturating_sub(1);
            self.prev_count = 0;
            self.curr_bucket = current_bucket;
            self.curr_count = 0;
        }

        let time_in_current_window = now_unix_secs % self.window_seconds;
        let prev_weight =
            (self.window_seconds - time_in_current_window) as f64 / self.window_seconds as f64;
        let estimated_count = f64::from(self.prev_count) * prev_weight + f64::from(self.curr_count);
        let reset_seconds = self
            .window_seconds
            .saturating_sub(time_in_current_window)
            .max(1);

        if estimated_count + f64::from(cost) <= f64::from(self.max_requests) {
            self.curr_count = self.curr_count.saturating_add(cost);
            let remaining = (f64::from(self.max_requests) - (estimated_count + f64::from(cost)))
                .floor()
                .max(0.0) as u32;

            RateLimitDecision::Allowed {
                limit: self.max_requests,
                remaining,
                reset_seconds,
            }
        } else {
            // Calculate retry-after based on how long until the weight decays sufficiently
            let excess = (estimated_count + f64::from(cost)) - f64::from(self.max_requests);
            let retry_after = if self.prev_count > 0 {
                let decay_needed = excess / f64::from(self.prev_count);
                (decay_needed * self.window_seconds as f64).ceil() as u64
            } else {
                reset_seconds
            };

            RateLimitDecision::Exceeded {
                limit: self.max_requests,
                retry_after_seconds: retry_after.clamp(1, reset_seconds),
                reset_seconds,
            }
        }
    }
}

/// Abstract rate limiter contract for request throttling.
pub trait RateLimiter: Send + Sync {
    /// Evaluates rate limits for the specified client key, tier, and request cost.
    fn check(&self, key: &str, tier: RateLimitTier, cost: u32) -> RateLimitDecision;

    /// Resets rate limit quota for the given key (useful in testing or manual intervention).
    fn reset(&self, key: &str);

    /// Prunes stale buckets to ensure memory remains bounded (Rule 02).
    fn prune(&self);
}

/// In-memory tiered token-bucket rate limiter.
#[derive(Debug)]
pub struct InMemoryTieredRateLimiter {
    config: RateLimitConfig,
    buckets: RwLock<HashMap<String, (TokenBucket, Instant)>>,
}

impl InMemoryTieredRateLimiter {
    /// Creates a new in-memory tiered rate limiter from configuration.
    #[must_use]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: RwLock::new(HashMap::new()),
        }
    }
}

impl RateLimiter for InMemoryTieredRateLimiter {
    fn check(&self, key: &str, tier: RateLimitTier, cost: u32) -> RateLimitDecision {
        if !self.config.enabled {
            return RateLimitDecision::Allowed {
                limit: u32::MAX,
                remaining: u32::MAX,
                reset_seconds: 0,
            };
        }

        if tier == RateLimitTier::Admin {
            return RateLimitDecision::Allowed {
                limit: 10_000,
                remaining: 9_999,
                reset_seconds: 1,
            };
        }

        let (capacity, rpm) = match tier {
            RateLimitTier::Authenticated | RateLimitTier::Admin => {
                (self.config.burst_capacity, self.config.requests_per_minute)
            }
            RateLimitTier::Anonymous => (
                self.config.anonymous_burst_capacity,
                self.config.anonymous_requests_per_minute,
            ),
        };

        let now = Instant::now();

        let mut map = self.buckets.write().expect("rate limit lock poisoned");
        let entry = map
            .entry(key.to_string())
            .or_insert_with(|| (TokenBucket::new(capacity, rpm, now), now));

        entry.1 = now;
        entry.0.consume(now, f64::from(cost))
    }

    fn reset(&self, key: &str) {
        let mut map = self.buckets.write().expect("rate limit lock poisoned");
        map.remove(key);
    }

    fn prune(&self) {
        let now = Instant::now();
        let ttl = Duration::from_secs(300); // 5 minutes idle TTL
        let mut map = self.buckets.write().expect("rate limit lock poisoned");
        map.retain(|_, (_, last_access)| now.saturating_duration_since(*last_access) < ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_initial_capacity_and_consumption() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(5, 60, now);

        // First request consumes 1 token
        let d1 = bucket.consume(now, 1.0);
        assert!(d1.is_allowed());
        assert_eq!(d1.remaining(), 4);
        assert_eq!(d1.limit(), 5);

        // Consume remaining 4 tokens
        for _ in 0..4 {
            let d = bucket.consume(now, 1.0);
            assert!(d.is_allowed());
        }

        // 6th request exhausts burst capacity -> 429
        let d6 = bucket.consume(now, 1.0);
        assert!(!d6.is_allowed());
        assert_eq!(d6.remaining(), 0);
        assert!(d6.retry_after_seconds().unwrap() >= 1);
    }

    #[test]
    fn test_token_bucket_replenishment_over_time() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(2, 60, now); // 60 RPM = 1 token/sec

        // Drain bucket
        assert!(bucket.consume(now, 1.0).is_allowed());
        assert!(bucket.consume(now, 1.0).is_allowed());
        assert!(!bucket.consume(now, 1.0).is_allowed());

        // Advance time by 1.1 seconds -> exactly 1 token refilled
        let later = now + Duration::from_millis(1100);
        let d = bucket.consume(later, 1.0);
        assert!(d.is_allowed());
    }

    #[test]
    fn test_sliding_window_counter_boundary_smoothness() {
        let mut window = SlidingWindowCounter::new(10, 60);

        // Consume 5 in window 0
        let d1 = window.consume(10, 5);
        assert!(d1.is_allowed());
        assert_eq!(d1.remaining(), 5);

        // Consume 5 in window 0 -> full
        let d2 = window.consume(20, 5);
        assert!(d2.is_allowed());

        // Next request in same window exceeds
        let d3 = window.consume(30, 1);
        assert!(!d3.is_allowed());

        // Advance to window 1 at t=75 (15 seconds into window 1)
        // Previous window (t=0..60) has 10 requests, weight = (60 - 15) / 60 = 0.75 -> 7.5 requests
        // Current window has 0 requests.
        // Effective count = 7.5. Limit is 10.
        // Consume 2 -> 7.5 + 2 = 9.5 <= 10 -> Allowed!
        let d4 = window.consume(75, 2);
        assert!(d4.is_allowed());

        // Consume another 2 -> 9.5 + 2 = 11.5 > 10 -> Exceeded!
        let d5 = window.consume(75, 2);
        assert!(!d5.is_allowed());
    }

    #[test]
    fn test_tiered_rate_limiter_anonymous_vs_authenticated_limits() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 60,
            burst_capacity: 10,
            anonymous_requests_per_minute: 10,
            anonymous_burst_capacity: 2,
            window_seconds: 60,
        };

        let limiter = InMemoryTieredRateLimiter::new(config);

        // Anonymous client gets 2 burst requests
        assert!(limiter
            .check("ip:1.2.3.4", RateLimitTier::Anonymous, 1)
            .is_allowed());
        assert!(limiter
            .check("ip:1.2.3.4", RateLimitTier::Anonymous, 1)
            .is_allowed());
        assert!(!limiter
            .check("ip:1.2.3.4", RateLimitTier::Anonymous, 1)
            .is_allowed());

        // Authenticated client gets 10 burst requests
        for _ in 0..10 {
            assert!(limiter
                .check("key:auth_tenant_1", RateLimitTier::Authenticated, 1)
                .is_allowed());
        }
        assert!(!limiter
            .check("key:auth_tenant_1", RateLimitTier::Authenticated, 1)
            .is_allowed());

        // Admin client is always allowed
        for _ in 0..50 {
            assert!(limiter
                .check("key:admin", RateLimitTier::Admin, 1)
                .is_allowed());
        }
    }

    #[test]
    fn test_rate_limiter_reset_and_pruning() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 10,
            burst_capacity: 1,
            anonymous_requests_per_minute: 10,
            anonymous_burst_capacity: 1,
            window_seconds: 60,
        };

        let limiter = InMemoryTieredRateLimiter::new(config);

        assert!(limiter
            .check("client_a", RateLimitTier::Anonymous, 1)
            .is_allowed());
        assert!(!limiter
            .check("client_a", RateLimitTier::Anonymous, 1)
            .is_allowed());

        // Reset client_a
        limiter.reset("client_a");
        assert!(limiter
            .check("client_a", RateLimitTier::Anonymous, 1)
            .is_allowed());

        // Pruning does not crash
        limiter.prune();
    }
}
