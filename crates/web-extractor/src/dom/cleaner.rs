//! # Boilerplate Cleaner & Node Classifier
//!
//! Evaluates DOM nodes against structural, semantic, and class/ID heuristics to identify and prune boilerplate.

use super::policy::BoilerplatePolicy;

/// Content indicators that protect an element and its subtree from being classified as boilerplate.
pub const CONTENT_INDICATORS: &[&str] = &[
    "article",
    "content",
    "main",
    "post",
    "entry-content",
    "markdown-body",
    "doc-content",
    "text-content",
    "post-body",
    "page-content",
    "documentation",
    "body-text",
    "prose",
];

/// Boilerplate class/ID keywords representing advertising, monetization, and sponsored widgets.
pub const AD_PATTERNS: &[&str] = &[
    "ad",
    "ads",
    "advert",
    "advertisement",
    "sponsored",
    "promo",
    "banner-ad",
    "sidebar-ad",
    "google-ad",
    "dfp",
    "adsbygoogle",
    "taboola",
    "outbrain",
    "sponsor",
];

/// Boilerplate class/ID keywords representing cookie consent, GDPR, and privacy modals.
pub const COOKIE_PATTERNS: &[&str] = &[
    "cookie",
    "consent",
    "gdpr",
    "privacy-banner",
    "privacy-modal",
    "cookie-banner",
    "cookie-notice",
    "cc-window",
    "cmpbox",
    "notice-banner",
];

/// Boilerplate class/ID keywords representing social media sharing toolbars and widgets.
pub const SOCIAL_PATTERNS: &[&str] = &[
    "social",
    "share",
    "sharing",
    "share-buttons",
    "tweet",
    "facebook-share",
    "share-bar",
    "social-icons",
    "social-links",
];

/// Boilerplate class/ID keywords representing non-content chrome, menus, breadcrumbs, and pagination.
pub const NAV_PATTERNS: &[&str] = &[
    "menu",
    "navbar",
    "nav-bar",
    "breadcrumb",
    "site-header",
    "page-footer",
    "site-footer",
    "pagination",
    "pager",
    "nav-menu",
    "top-nav",
];

/// Boilerplate class/ID keywords representing comment threads and reply forms.
pub const COMMENT_PATTERNS: &[&str] = &[
    "disqus",
    "comment",
    "comments",
    "comment-list",
    "reply-form",
    "respond",
    "feedback-form",
];

/// Boilerplate class/ID keywords representing promotional sidebars, newsletters, and signups.
pub const WIDGET_PATTERNS: &[&str] = &[
    "newsletter",
    "subscribe",
    "widget",
    "sidebar-widget",
    "author-bio-popup",
    "popup",
    "interstitial",
];

/// Evaluates HTML elements for boilerplate characteristics.
pub struct BoilerplateCleaner;

impl BoilerplateCleaner {
    /// Determines whether a DOM element should be pruned based on tag, attributes, and policy.
    pub fn is_boilerplate(
        tag_name: &str,
        id_opt: Option<&str>,
        class_opt: Option<&str>,
        role_opt: Option<&str>,
        inside_main_or_article: bool,
        policy: &BoilerplatePolicy,
    ) -> bool {
        let tag_lower = tag_name.to_ascii_lowercase();

        // 1. Unconditionally stripped tags (scripts, styles, embeds, canvases, templates)
        if policy.strip_scripts_and_styles {
            match tag_lower.as_str() {
                "script" | "style" | "noscript" | "iframe" | "object" | "embed" | "svg"
                | "canvas" | "template" | "applet" => return true,
                _ => {}
            }
        }

        // 2. Strict protection for code blocks and tables (Rule 04)
        if policy.preserve_code_blocks && (tag_lower == "pre" || tag_lower == "code") {
            return false;
        }

        if policy.preserve_tables {
            match tag_lower.as_str() {
                "table" | "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" | "caption" => {
                    return false
                }
                _ => {}
            }
        }

        // 3. Protection for primary semantic containers
        if tag_lower == "main" || tag_lower == "article" {
            return false;
        }

        if let Some(role) = role_opt {
            if role.eq_ignore_ascii_case("main") || role.eq_ignore_ascii_case("article") {
                return false;
            }
        }

        let id_lower = id_opt.map(|s| s.to_ascii_lowercase());
        let class_lower = class_opt.map(|s| s.to_ascii_lowercase());

        // 4. Custom allowed classes check (highest priority user override)
        if let Some(classes) = &class_lower {
            for allowed in &policy.custom_allowed_classes {
                if classes.contains(&allowed.to_ascii_lowercase()) {
                    return false;
                }
            }
        }

        // 5. Content indicator protection (classes or IDs indicating article body)
        if Self::matches_any(
            class_lower.as_deref(),
            id_lower.as_deref(),
            CONTENT_INDICATORS,
        ) {
            return false;
        }

        // 6. Structural chrome tags
        if policy.strip_nav && tag_lower == "nav" {
            return true;
        }

        if policy.strip_footers && tag_lower == "footer" {
            return true;
        }

        if policy.strip_sidebars && tag_lower == "aside" && !inside_main_or_article {
            return true;
        }

        if policy.strip_forms && tag_lower == "form" {
            return true;
        }

        if tag_lower == "header" && !inside_main_or_article {
            return true;
        }

        // 7. Custom blocked classes and IDs
        for blocked_class in &policy.custom_blocked_classes {
            if let Some(classes) = &class_lower {
                if classes.contains(&blocked_class.to_ascii_lowercase()) {
                    return true;
                }
            }
        }

        for blocked_id in &policy.custom_blocked_ids {
            if let Some(id) = &id_lower {
                if id.contains(&blocked_id.to_ascii_lowercase()) {
                    return true;
                }
            }
        }

        // 8. Heuristic pattern checks
        if policy.strip_ads
            && Self::matches_any(class_lower.as_deref(), id_lower.as_deref(), AD_PATTERNS)
        {
            return true;
        }

        if policy.strip_cookie_banners
            && Self::matches_any(class_lower.as_deref(), id_lower.as_deref(), COOKIE_PATTERNS)
        {
            return true;
        }

        if policy.strip_social_sharing
            && Self::matches_any(class_lower.as_deref(), id_lower.as_deref(), SOCIAL_PATTERNS)
        {
            return true;
        }

        if policy.strip_nav
            && Self::matches_any(class_lower.as_deref(), id_lower.as_deref(), NAV_PATTERNS)
        {
            return true;
        }

        if Self::matches_any(
            class_lower.as_deref(),
            id_lower.as_deref(),
            COMMENT_PATTERNS,
        ) {
            return true;
        }

        if Self::matches_any(class_lower.as_deref(), id_lower.as_deref(), WIDGET_PATTERNS) {
            return true;
        }

        false
    }

    /// Checks if either class or id contains any of the target patterns as a substring or token.
    fn matches_any(class_opt: Option<&str>, id_opt: Option<&str>, patterns: &[&str]) -> bool {
        if let Some(class) = class_opt {
            for pattern in patterns {
                if class.contains(pattern) {
                    return true;
                }
            }
        }

        if let Some(id) = id_opt {
            for pattern in patterns {
                if id.contains(pattern) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_style_always_boilerplate() {
        let policy = BoilerplatePolicy::default();
        assert!(BoilerplateCleaner::is_boilerplate(
            "script", None, None, None, false, &policy
        ));
        assert!(BoilerplateCleaner::is_boilerplate(
            "style", None, None, None, false, &policy
        ));
        assert!(BoilerplateCleaner::is_boilerplate(
            "iframe", None, None, None, false, &policy
        ));
    }

    #[test]
    fn test_code_blocks_and_tables_preserved() {
        let policy = BoilerplatePolicy::default();
        assert!(!BoilerplateCleaner::is_boilerplate(
            "pre",
            None,
            Some("highlight"),
            None,
            false,
            &policy
        ));
        assert!(!BoilerplateCleaner::is_boilerplate(
            "code", None, None, None, false, &policy
        ));
        assert!(!BoilerplateCleaner::is_boilerplate(
            "table", None, None, None, false, &policy
        ));
        assert!(!BoilerplateCleaner::is_boilerplate(
            "tr", None, None, None, false, &policy
        ));
    }

    #[test]
    fn test_content_indicators_protect_element() {
        let policy = BoilerplatePolicy::default();
        // Even if class has 'ad' substring, 'entry-content' protects it
        assert!(!BoilerplateCleaner::is_boilerplate(
            "div",
            Some("main-post"),
            Some("entry-content"),
            None,
            false,
            &policy
        ));
    }

    #[test]
    fn test_ads_and_cookies_classified_as_boilerplate() {
        let policy = BoilerplatePolicy::default();
        assert!(BoilerplateCleaner::is_boilerplate(
            "div",
            None,
            Some("sidebar-ad banner"),
            None,
            false,
            &policy
        ));
        assert!(BoilerplateCleaner::is_boilerplate(
            "div",
            Some("cookie-banner"),
            None,
            None,
            false,
            &policy
        ));
        assert!(BoilerplateCleaner::is_boilerplate(
            "div",
            None,
            Some("social-share-buttons"),
            None,
            false,
            &policy
        ));
    }
}
