//! # Schema Versioning and SemVer Negotiation
//!
//! Strongly-typed semantic versioning representation for [`CdmDocument`] schemas,
//! providing ordering, compatibility checks, and string serialization.

use crate::error::{CdmError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// A strongly-typed semantic version for CDM specifications.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SchemaVersion {
    /// Major breaking version component.
    pub major: u32,
    /// Minor backward-compatible feature version component.
    pub minor: u32,
    /// Patch backward-compatible bugfix version component.
    pub patch: u32,
    /// Optional pre-release identifier (e.g. `alpha.1`, `beta`).
    pub pre_release: Option<String>,
}

impl SchemaVersion {
    /// Constructs a new semantic version without pre-release tags.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Constructs a semantic version with a pre-release identifier.
    #[must_use]
    pub fn with_pre_release(
        major: u32,
        minor: u32,
        patch: u32,
        pre_release: impl Into<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release.into()),
        }
    }

    /// The current authoritative schema version (`1.0.0`).
    #[must_use]
    pub const fn current() -> Self {
        Self::new(1, 0, 0)
    }

    /// Historical prototype schema version `0.8.0`.
    #[must_use]
    pub const fn v0_8_0() -> Self {
        Self::new(0, 8, 0)
    }

    /// Historical preview schema version `0.9.0`.
    #[must_use]
    pub const fn v0_9_0() -> Self {
        Self::new(0, 9, 0)
    }

    /// Production release schema version `1.0.0`.
    #[must_use]
    pub const fn v1_0_0() -> Self {
        Self::new(1, 0, 0)
    }

    /// Parses a semantic version string (e.g. `"1.0.0"` or `"0.9.1-beta"`).
    ///
    /// # Errors
    /// Returns [`CdmError::UnsupportedSchemaVersion`] if parsing fails.
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }

    /// Returns true if this version is backward-compatible with the target version.
    ///
    /// In Semantic Versioning:
    /// - For major >= 1: matching major version indicates compatibility.
    /// - For major == 0: zero-major releases indicate unstable prototypes where matching minor is required.
    #[must_use]
    pub fn is_compatible_with(&self, target: &Self) -> bool {
        if self.major >= 1 && target.major >= 1 {
            self.major == target.major
        } else {
            self.major == target.major && self.minor == target.minor
        }
    }

    /// Returns true if `self` is strictly newer than `other`.
    #[must_use]
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self > other
    }

    /// Returns true if `self` is strictly older than `other`.
    #[must_use]
    pub fn is_older_than(&self, other: &Self) -> bool {
        self < other
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref pre) = self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

impl FromStr for SchemaVersion {
    type Err = CdmError;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        let (core, pre_release) = match trimmed.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (trimmed, None),
        };

        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(CdmError::UnsupportedSchemaVersion {
                found: s.to_string(),
                supported: "X.Y.Z".to_string(),
            });
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| CdmError::UnsupportedSchemaVersion {
                found: s.to_string(),
                supported: "X.Y.Z with integer components".to_string(),
            })?;

        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| CdmError::UnsupportedSchemaVersion {
                found: s.to_string(),
                supported: "X.Y.Z with integer components".to_string(),
            })?;

        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| CdmError::UnsupportedSchemaVersion {
                found: s.to_string(),
                supported: "X.Y.Z with integer components".to_string(),
            })?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => match (&self.pre_release, &other.pre_release) {
                        (None, None) => Ordering::Equal,
                        (Some(_), None) => Ordering::Less, // Normal version has higher precedence than pre-release
                        (None, Some(_)) => Ordering::Greater,
                        (Some(a), Some(b)) => a.cmp(b),
                    },
                    other_ord => other_ord,
                },
                other_ord => other_ord,
            },
            other_ord => other_ord,
        }
    }
}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_display() {
        let v1 = SchemaVersion::parse("1.0.0").unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 0);
        assert_eq!(v1.patch, 0);
        assert_eq!(v1.pre_release, None);
        assert_eq!(v1.to_string(), "1.0.0");

        let v2 = SchemaVersion::parse("0.9.1-beta.2").unwrap();
        assert_eq!(v2.major, 0);
        assert_eq!(v2.minor, 9);
        assert_eq!(v2.patch, 1);
        assert_eq!(v2.pre_release, Some("beta.2".to_string()));
        assert_eq!(v2.to_string(), "0.9.1-beta.2");

        assert!(SchemaVersion::parse("invalid").is_err());
        assert!(SchemaVersion::parse("1.0").is_err());
        assert!(SchemaVersion::parse("1.0.a").is_err());
    }

    #[test]
    fn test_version_ordering() {
        let v0_8 = SchemaVersion::new(0, 8, 0);
        let v0_9 = SchemaVersion::new(0, 9, 0);
        let v1_0_pre = SchemaVersion::with_pre_release(1, 0, 0, "rc1");
        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);

        assert!(v0_8 < v0_9);
        assert!(v0_9 < v1_0_pre);
        assert!(v1_0_pre < v1_0);
        assert!(v1_0 < v1_1);
        assert!(v1_1.is_newer_than(&v1_0));
        assert!(v0_8.is_older_than(&v1_0));
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);
        let v2_0 = SchemaVersion::new(2, 0, 0);
        let v0_8 = SchemaVersion::new(0, 8, 0);
        let v0_9 = SchemaVersion::new(0, 9, 0);

        assert!(v1_0.is_compatible_with(&v1_1));
        assert!(!v1_0.is_compatible_with(&v2_0));
        assert!(!v0_8.is_compatible_with(&v0_9));
    }

    #[test]
    fn test_version_serde_roundtrip() {
        let version = SchemaVersion::with_pre_release(1, 2, 3, "alpha.1");
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"1.2.3-alpha.1\"");

        let deserialized: SchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, version);
    }
}
