//! # Readability Heuristic Weights and Patterns
//!
//! Provides scoring weights and pattern classifiers based on readability algorithms.

use super::model::ReadabilityOptions;

/// Class/ID patterns indicating high-confidence article and editorial content.
pub const POSITIVE_PATTERNS: &[&str] = &[
    "article",
    "article-body",
    "article-content",
    "blog",
    "body",
    "content",
    "entry",
    "entry-content",
    "hentry",
    "instapaper_body",
    "main",
    "markdown-body",
    "page",
    "post",
    "post-body",
    "post-content",
    "prose",
    "story",
    "text",
];

/// Class/ID patterns indicating navigation, comments, footers, widgets, and noise.
pub const NEGATIVE_PATTERNS: &[&str] = &[
    "ad",
    "agegate",
    "archive",
    "combx",
    "comment",
    "community",
    "disqus",
    "extra",
    "foot",
    "footer",
    "header",
    "menu",
    "nav",
    "pager",
    "pagination",
    "popup",
    "print",
    "recommend",
    "related",
    "remark",
    "rss",
    "share",
    "shoutbox",
    "sidebar",
    "similar",
    "social",
    "sponsor",
    "widget",
];

/// Class/ID patterns identifying unlikely candidates for primary article containers.
pub const UNLIKELY_PATTERNS: &[&str] = &[
    "ad-box",
    "advertisement",
    "banner",
    "cookie",
    "footer",
    "header",
    "menu",
    "nav",
    "navbar",
    "navigation",
    "promo",
    "sidebar",
    "widget",
];

/// Structural base weight given to an HTML element based on its semantic tag.
#[must_use]
pub fn calculate_tag_weight(tag_lower: &str) -> f64 {
    match tag_lower {
        "article" => 30.0,
        "main" => 25.0,
        "section" => 10.0,
        "div" => 5.0,
        "nav" | "footer" | "header" | "aside" => -50.0,
        _ => 0.0,
    }
}

/// Evaluates class and ID attributes against positive and negative readability patterns.
#[must_use]
pub fn calculate_class_weight(
    id_opt: Option<&str>,
    class_opt: Option<&str>,
    options: &ReadabilityOptions,
) -> f64 {
    if !options.weight_classes_and_ids {
        return 0.0;
    }

    let mut weight = 0.0;

    let id_lower = id_opt.map(str::to_ascii_lowercase);
    let class_lower = class_opt.map(str::to_ascii_lowercase);

    let check_string = |s: &str| -> f64 {
        let mut score = 0.0;
        for pos in POSITIVE_PATTERNS {
            if s.contains(pos) {
                score += 25.0;
                break;
            }
        }
        for custom_pos in &options.custom_positive_classes {
            if s.contains(custom_pos.as_str()) {
                score += 25.0;
                break;
            }
        }
        for neg in NEGATIVE_PATTERNS {
            if s.contains(neg) {
                score -= 25.0;
                break;
            }
        }
        for custom_neg in &options.custom_negative_classes {
            if s.contains(custom_neg.as_str()) {
                score -= 25.0;
                break;
            }
        }
        score
    };

    if let Some(ref id) = id_lower {
        weight += check_string(id);
    }
    if let Some(ref class) = class_lower {
        weight += check_string(class);
    }

    weight
}

/// Determines whether a container is an unlikely candidate for the primary article body.
#[must_use]
pub fn is_unlikely_candidate(
    tag_lower: &str,
    id_opt: Option<&str>,
    class_opt: Option<&str>,
) -> bool {
    // Structural negative tags are always unlikely candidates
    if matches!(tag_lower, "nav" | "footer" | "header" | "aside") {
        return true;
    }

    // Never reject semantic article or main elements
    if matches!(tag_lower, "article" | "main") {
        return false;
    }

    let id_lower = id_opt.map(str::to_ascii_lowercase);
    let class_lower = class_opt.map(str::to_ascii_lowercase);

    let check_unlikely = |s: &str| -> bool {
        // If it contains a positive pattern, do not consider it unlikely
        for pos in POSITIVE_PATTERNS {
            if s.contains(pos) {
                return false;
            }
        }
        // Otherwise, check for unlikely patterns
        for unlikely in UNLIKELY_PATTERNS {
            if s.contains(unlikely) {
                return true;
            }
        }
        false
    };

    if let Some(ref id) = id_lower {
        if check_unlikely(id) {
            return true;
        }
    }
    if let Some(ref class) = class_lower {
        if check_unlikely(class) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_tag_weight() {
        assert_eq!(calculate_tag_weight("article"), 30.0);
        assert_eq!(calculate_tag_weight("main"), 25.0);
        assert_eq!(calculate_tag_weight("section"), 10.0);
        assert_eq!(calculate_tag_weight("div"), 5.0);
        assert_eq!(calculate_tag_weight("nav"), -50.0);
    }

    #[test]
    fn test_calculate_class_weight_positive_and_negative() {
        let opts = ReadabilityOptions::default();
        let positive = calculate_class_weight(Some("main-content"), Some("post-body"), &opts);
        assert!(positive >= 50.0);

        let negative = calculate_class_weight(Some("sidebar-box"), Some("widget-comments"), &opts);
        assert!(negative <= -50.0);
    }

    #[test]
    fn test_is_unlikely_candidate() {
        assert!(is_unlikely_candidate("nav", None, None));
        assert!(is_unlikely_candidate("div", Some("sidebar"), None));
        assert!(is_unlikely_candidate("div", None, Some("footer-menu")));
        // Article tag is never unlikely
        assert!(!is_unlikely_candidate("article", Some("sidebar"), None));
        // Positive pattern overrides unlikely pattern
        assert!(!is_unlikely_candidate(
            "div",
            Some("sidebar-post-content"),
            None
        ));
    }
}
