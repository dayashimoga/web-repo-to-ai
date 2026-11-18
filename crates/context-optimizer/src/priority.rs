//! # Section Priority and Importance Scoring
//!
//! Evaluates content significance, architectural prominence, and information density
//! to compute deterministic priority scores and pinning states across [`CdmSection`]s.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmSection, SectionKind};

/// Explicit pinning or priority override classification for a content section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinStatus {
    /// Mandatory section; always included unless total available budget is smaller than its size.
    MustInclude,
    /// Standard section; prioritized according to calculated composite importance score.
    Normal,
    /// Low-priority section; included only if sufficient surplus budget remains.
    Deprioritized,
    /// Explicitly excluded from final context package regardless of available budget.
    MustExclude,
}

impl PinStatus {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MustInclude => "must_include",
            Self::Normal => "normal",
            Self::Deprioritized => "deprioritized",
            Self::MustExclude => "must_exclude",
        }
    }
}

/// Explainable breakdown of factors contributing to a section's priority score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriorityScoreFactors {
    /// Base weight derived from [`SectionKind`] (0.0 to 1.0).
    pub kind_weight: f64,
    /// Prominence bonus based on filename and directory location.
    pub prominence_bonus: f64,
    /// Modifier based on information density classification.
    pub density_modifier: f64,
    /// Upstream content classification signal score from Phase 7.1.4 (if present).
    pub signal_score: Option<f64>,
    /// Architectural code role weight (e.g. CoreImplementation vs TestOrFixture).
    pub nature_modifier: f64,
    /// Cumulative penalty for boilerplate, test fixtures, or generated code.
    pub penalty: f64,
}

/// Computed priority score and inclusion policy for a section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionPriority {
    /// Pinning status for budget allocation.
    pub pin_status: PinStatus,
    /// Normalized priority score in range `[0.0, 1.0]`.
    pub score: f64,
    /// Diagnostic factors explaining how the score was calculated.
    pub score_factors: PriorityScoreFactors,
}

/// Section importance and priority scoring engine.
#[derive(Debug, Clone, Default)]
pub struct SectionScorer;

impl SectionScorer {
    /// Creates a new default section scorer.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Computes the priority score and pinning status for a given [`CdmSection`].
    #[must_use]
    pub fn score(&self, section: &CdmSection) -> SectionPriority {
        // 1. Check explicit pinning attribute overrides
        if let Some(pin_attr) = section.attributes.get("pin") {
            match pin_attr.to_ascii_lowercase().as_str() {
                "must_include" | "pin" | "pinned" => {
                    return SectionPriority {
                        pin_status: PinStatus::MustInclude,
                        score: 1.0,
                        score_factors: PriorityScoreFactors {
                            kind_weight: 1.0,
                            prominence_bonus: 0.0,
                            density_modifier: 0.0,
                            signal_score: None,
                            nature_modifier: 0.0,
                            penalty: 0.0,
                        },
                    };
                }
                "must_exclude" | "exclude" | "forbidden" => {
                    return SectionPriority {
                        pin_status: PinStatus::MustExclude,
                        score: 0.0,
                        score_factors: PriorityScoreFactors {
                            kind_weight: 0.0,
                            prominence_bonus: 0.0,
                            density_modifier: 0.0,
                            signal_score: None,
                            nature_modifier: 0.0,
                            penalty: 1.0,
                        },
                    };
                }
                "deprioritized" | "low" => {
                    let mut priority = self.calculate_composite(section);
                    priority.pin_status = PinStatus::Deprioritized;
                    priority.score = (priority.score * 0.5).max(0.05);
                    return priority;
                }
                _ => {}
            }
        }

        self.calculate_composite(section)
    }

    /// Internal calculation of the composite priority score.
    fn calculate_composite(&self, section: &CdmSection) -> SectionPriority {
        // Base weight by SectionKind
        let kind_weight = match section.kind {
            SectionKind::Manifest => 1.0,
            SectionKind::Frontmatter => 0.85,
            SectionKind::Admonition => 0.80,
            SectionKind::Prose => 0.75,
            SectionKind::CodeFile => 0.70,
            SectionKind::CodeSnippet => 0.65,
            SectionKind::Table => 0.60,
            SectionKind::Navigation => 0.20,
        };

        // File and path prominence bonus
        let path = section.original_path.as_deref().unwrap_or("");
        let title = section.title.as_deref().unwrap_or("");
        let filename = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();

        let mut prominence_bonus = 0.0;
        let mut is_root_readme = false;
        let mut is_root_manifest = false;

        if filename == "readme.md" || filename == "readme.txt" || filename == "readme" {
            prominence_bonus += 0.35;
            if !path.contains('/') || path.starts_with("./") {
                is_root_readme = true;
            }
        } else if filename == "architecture.md"
            || filename == "design.md"
            || filename == "spec.md"
            || filename == "data_model.md"
        {
            prominence_bonus += 0.30;
        } else if filename == "main.rs"
            || filename == "lib.rs"
            || filename == "index.ts"
            || filename == "index.js"
            || filename == "main.py"
            || filename == "app.py"
            || filename == "mod.rs"
        {
            prominence_bonus += 0.25;
        } else if filename == "cargo.toml"
            || filename == "package.json"
            || filename == "go.mod"
            || filename == "pyproject.toml"
        {
            prominence_bonus += 0.20;
            if !path.contains('/') || path.starts_with("./") {
                is_root_manifest = true;
            }
        } else if filename == "dockerfile"
            || filename == "containerfile"
            || filename == "tsconfig.json"
        {
            prominence_bonus += 0.10;
        }

        // Deep nesting penalty (files nested > 4 directories deep)
        let dir_depth = path.split('/').count().saturating_sub(1);
        let mut depth_penalty = 0.0;
        if dir_depth >= 4 {
            depth_penalty = 0.05 * (dir_depth - 3).min(3) as f64;
        }

        // Information density modifier
        let mut density_modifier = 0.0;
        if let Some(density_str) = section.attributes.get("information_density") {
            match density_str.to_ascii_lowercase().as_str() {
                "high" => density_modifier += 0.10,
                "medium" => {}
                "low" => density_modifier -= 0.10,
                "sparse" => density_modifier -= 0.25,
                _ => {}
            }
        }

        // Upstream signal score from Phase 7.1.4
        let signal_score = section
            .attributes
            .get("signal_score")
            .and_then(|s| s.parse::<f64>().ok());

        // Architectural code nature modifier
        let mut nature_modifier = 0.0;
        if let Some(nature_str) = section.attributes.get("code_nature") {
            match nature_str.to_ascii_lowercase().as_str() {
                "core_implementation" => nature_modifier += 0.15,
                "interface_or_type_definition" => nature_modifier += 0.10,
                "data_or_schema" => nature_modifier += 0.05,
                "build_or_script" => nature_modifier += 0.0,
                "test_or_fixture" => nature_modifier -= 0.20,
                "generated_or_boilerplate" => nature_modifier -= 0.35,
                _ => {}
            }
        }

        // Penalties for tests, generated code, and boilerplate
        let mut penalty = depth_penalty;
        let is_test = section
            .attributes
            .get("is_test")
            .is_some_and(|v| v == "true")
            || path.contains("/tests/")
            || path.contains("/test/")
            || filename.contains("test")
            || filename.contains("spec");

        if is_test {
            penalty += 0.25;
        }

        let is_generated = section
            .attributes
            .get("is_generated")
            .is_some_and(|v| v == "true")
            || filename.ends_with(".min.js")
            || filename.ends_with(".min.css")
            || filename.contains(".generated.");

        if is_generated {
            penalty += 0.40;
        }

        let is_boilerplate = section
            .attributes
            .get("is_boilerplate")
            .is_some_and(|v| v == "true");

        if is_boilerplate {
            penalty += 0.30;
        }

        // Compute baseline raw score
        let mut raw_score =
            kind_weight + prominence_bonus + density_modifier + nature_modifier - penalty;

        // Blend with upstream signal score if present
        if let Some(sig) = signal_score {
            raw_score = raw_score * 0.75 + sig * 0.25;
        }

        // Normalize strictly to [0.01, 1.00]
        let score = raw_score.clamp(0.01, 1.00);

        // Determine default PinStatus
        let pin_status = if is_root_readme
            || is_root_manifest
            || section.kind == SectionKind::Manifest
            || title.eq_ignore_ascii_case("overview")
        {
            PinStatus::MustInclude
        } else if is_generated || is_boilerplate || (is_test && score < 0.35) {
            PinStatus::Deprioritized
        } else {
            PinStatus::Normal
        };

        SectionPriority {
            pin_status,
            score,
            score_factors: PriorityScoreFactors {
                kind_weight,
                prominence_bonus,
                density_modifier,
                signal_score,
                nature_modifier,
                penalty,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_test_section(
        id: &str,
        path: &str,
        kind: SectionKind,
        attrs: Vec<(&str, &str)>,
    ) -> CdmSection {
        let mut attributes = BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }

        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(path.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!("https://github.com/repo/blob/main/{path}"),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: "hash".to_string(),
            body: "content".to_string(),
            token_metrics: None,
            attributes,
        }
    }

    #[test]
    fn test_readme_and_manifest_must_include() {
        let scorer = SectionScorer::new();

        let readme = make_test_section("s1", "README.md", SectionKind::Prose, vec![]);
        let p_readme = scorer.score(&readme);
        assert_eq!(p_readme.pin_status, PinStatus::MustInclude);
        assert!(p_readme.score >= 0.9);

        let manifest = make_test_section("s2", "Cargo.toml", SectionKind::Manifest, vec![]);
        let p_manifest = scorer.score(&manifest);
        assert_eq!(p_manifest.pin_status, PinStatus::MustInclude);
        assert!(p_manifest.score >= 0.95);
    }

    #[test]
    fn test_core_implementation_vs_test_fixture() {
        let scorer = SectionScorer::new();

        let core_impl = make_test_section(
            "core",
            "src/engine.rs",
            SectionKind::CodeFile,
            vec![
                ("code_nature", "core_implementation"),
                ("information_density", "high"),
                ("signal_score", "0.95"),
            ],
        );
        let p_core = scorer.score(&core_impl);

        let test_sec = make_test_section(
            "test",
            "tests/fixture_test.rs",
            SectionKind::CodeFile,
            vec![
                ("code_nature", "test_or_fixture"),
                ("is_test", "true"),
                ("information_density", "low"),
            ],
        );
        let p_test = scorer.score(&test_sec);

        assert!(
            p_core.score > p_test.score,
            "Core implementation ({}) must score significantly higher than test fixture ({})",
            p_core.score,
            p_test.score
        );
        assert_eq!(p_core.pin_status, PinStatus::Normal);
        assert_eq!(p_test.pin_status, PinStatus::Deprioritized);
    }

    #[test]
    fn test_explicit_pin_overrides() {
        let scorer = SectionScorer::new();

        let pinned = make_test_section(
            "pinned",
            "tests/important_test.rs",
            SectionKind::CodeFile,
            vec![("pin", "must_include")],
        );
        let p_pinned = scorer.score(&pinned);
        assert_eq!(p_pinned.pin_status, PinStatus::MustInclude);
        assert_eq!(p_pinned.score, 1.0);

        let excluded = make_test_section(
            "excluded",
            "src/unwanted.rs",
            SectionKind::CodeFile,
            vec![("pin", "must_exclude")],
        );
        let p_excluded = scorer.score(&excluded);
        assert_eq!(p_excluded.pin_status, PinStatus::MustExclude);
        assert_eq!(p_excluded.score, 0.0);
    }
}
