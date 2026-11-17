//! Shared domain traits and abstractions.
//!
//! Provides [`Identifiable`], [`Clock`], [`SystemClock`], and [`Validatable`].

use crate::error::Result;
use chrono::{DateTime, Utc};

/// Trait for domain models with a strongly-typed primary identifier.
pub trait Identifiable {
    /// The concrete identifier type.
    type Id: Copy + Eq + std::hash::Hash + std::fmt::Display;

    /// Returns the unique identifier of this entity.
    fn id(&self) -> Self::Id;
}

/// Abstract provider for timestamps, enabling deterministic time in tests.
pub trait Clock: Send + Sync {
    /// Returns the current UTC timestamp.
    fn now_utc(&self) -> DateTime<Utc>;
}

/// Default real-world system clock backed by standard UTC time.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    /// Creates a new instance of [`SystemClock`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Trait for domain entities that enforce self-validation rules.
pub trait Validatable {
    /// Validates internal consistency and domain invariants.
    ///
    /// # Errors
    /// Returns a [`crate::error::CoreError::Validation`] if any rule is violated.
    fn validate(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::JobId;

    struct TestEntity {
        job_id: JobId,
    }

    impl Identifiable for TestEntity {
        type Id = JobId;

        fn id(&self) -> Self::Id {
            self.job_id
        }
    }

    #[test]
    fn test_identifiable() {
        let expected_id = JobId::new();
        let entity = TestEntity {
            job_id: expected_id,
        };
        assert_eq!(entity.id(), expected_id);
    }

    #[test]
    fn test_system_clock() {
        let clock = SystemClock::new();
        let t1 = clock.now_utc();
        let t2 = clock.now_utc();
        assert!(t2 >= t1);
    }
}
