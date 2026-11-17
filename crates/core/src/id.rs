//! Strongly-typed domain identifiers.
//!
//! Provides distinct newtype wrappers around UUID v7 (time-ordered) to eliminate
//! identifier confusion between jobs, documents, tenants, requests, and artifacts.

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! define_domain_id {
    ($type_name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $type_name(Uuid);

        impl $type_name {
            /// Generates a new unique identifier using time-ordered UUID v7.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Creates an identifier wrapping an existing UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns a reference to the underlying UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Unwraps this identifier into the underlying UUID value.
            #[must_use]
            pub const fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $type_name {
            type Err = CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|err| CoreError::Validation {
                        field: stringify!($type_name).to_string(),
                        reason: format!("Invalid UUID string '{}': {}", s, err),
                    })
            }
        }

        impl From<Uuid> for $type_name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$type_name> for Uuid {
            fn from(id: $type_name) -> Self {
                id.0
            }
        }
    };
}

define_domain_id!(
    JobId,
    "Unique time-ordered identifier for a conversion job."
);
define_domain_id!(
    DocumentId,
    "Unique time-ordered identifier for a Canonical Context Document (CDM)."
);
define_domain_id!(
    TenantId,
    "Unique identifier for an authorized organization or anonymous session."
);
define_domain_id!(
    RequestId,
    "Unique identifier for tracing an HTTP or background worker request."
);
define_domain_id!(
    ArtifactId,
    "Unique identifier for an exported output file or archive artifact."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation_and_uniqueness() {
        let id1 = JobId::new();
        let id2 = JobId::new();
        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_id_from_str_roundtrip() {
        let original = DocumentId::new();
        let s = original.to_string();
        let parsed: DocumentId = s.parse().expect("Valid UUID string should parse");
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_id_invalid_string() {
        let result: Result<JobId> = "not-a-valid-uuid".parse();
        assert!(result.is_err());
        match result.unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "JobId"),
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }

    #[test]
    fn test_id_json_serialization() {
        let id = RequestId::new();
        let json = serde_json::to_string(&id).expect("Should serialize to JSON string");
        assert_eq!(json, format!("\"{}\"", id));

        let deserialized: RequestId = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_time_ordering_v7() {
        let id1 = JobId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = JobId::new();
        assert!(
            id1 < id2,
            "UUID v7 should be monotonically increasing over time"
        );
    }
}
