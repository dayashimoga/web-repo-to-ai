//! # Relationships and Dependency Links
//!
//! Models representing semantic relationships (imports, call graphs, cross-references,
//! parent-child hierarchy, hyperlinks) between CDM sections and external resources.

use serde::{Deserialize, Serialize};

/// Categorization of relationships between components in a CDM document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    /// Source section imports or depends on target section/module.
    Imports,
    /// Source section is imported or depended on by target.
    ImportedBy,
    /// Source function/method calls target function/API.
    Calls,
    /// Source type/class inherits or extends target.
    Extends,
    /// Source links to target via internal or external hyperlink.
    LinksTo,
    /// Source references target symbol, documentation, or issue.
    References,
    /// Source is structural parent of target.
    ParentOf,
    /// Source is structural child of target.
    ChildOf,
    /// Source defines a declared symbol (class, struct, trait, function).
    DefinesSymbol,
    /// Source section is an exact or near duplicate of target section.
    DuplicateOf,
}

impl RelationshipKind {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Imports => "imports",
            Self::ImportedBy => "imported_by",
            Self::Calls => "calls",
            Self::Extends => "extends",
            Self::LinksTo => "links_to",
            Self::References => "references",
            Self::ParentOf => "parent_of",
            Self::ChildOf => "child_of",
            Self::DefinesSymbol => "defines_symbol",
            Self::DuplicateOf => "duplicate_of",
        }
    }
}

/// A directed edge connecting two entities in a CDM document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// Originating section, symbol, or file identifier.
    pub source_id: String,
    /// Destination section, symbol, or external URI.
    pub target_id: String,
    /// Semantic nature of the relationship.
    pub kind: RelationshipKind,
    /// Optional explanatory details (e.g. imported symbol name, HTTP link text).
    pub description: Option<String>,
}

impl Relationship {
    /// Constructs a new directed relationship.
    #[must_use]
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        kind: RelationshipKind,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            kind,
            description: None,
        }
    }

    /// Attaches an optional description to this relationship.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship::new("sec-main", "sec-lib", RelationshipKind::Imports)
            .with_description("use web_repo_to_ai_cdm::*;");

        assert_eq!(rel.source_id, "sec-main");
        assert_eq!(rel.target_id, "sec-lib");
        assert_eq!(rel.kind, RelationshipKind::Imports);
        assert_eq!(rel.kind.as_str(), "imports");
        assert_eq!(
            rel.description.as_deref(),
            Some("use web_repo_to_ai_cdm::*;")
        );
    }
}
