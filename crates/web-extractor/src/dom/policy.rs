//! # Boilerplate Removal Policy
//!
//! Configurable rules for filtering non-content DOM elements, structural chrome, and advertising widgets.

use serde::{Deserialize, Serialize};

/// Policy configuration governing DOM tree sanitization and boilerplate removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoilerplatePolicy {
    /// Strip `<script>`, `<style>`, `<noscript>`, `<iframe>`, `<object>`, `<embed>`, `<svg>`, `<canvas>`.
    pub strip_scripts_and_styles: bool,
    /// Strip HTML comments (`<!-- ... -->`).
    pub strip_comments: bool,
    /// Strip navigation elements (`<nav>`, `.nav`, `.navbar`, `.menu`, `.breadcrumb`).
    pub strip_nav: bool,
    /// Strip page footers (`<footer>`, `.footer`, `.site-footer`).
    pub strip_footers: bool,
    /// Strip auxiliary sidebars (`<aside>`, `.sidebar`, `.widget`).
    pub strip_sidebars: bool,
    /// Strip interactive forms (`<form>`, `.search-form`, `.login-form`).
    pub strip_forms: bool,
    /// Strip advertising and sponsored containers (`.ad`, `.ads`, `.advertisement`, `.sponsor`).
    pub strip_ads: bool,
    /// Strip cookie and GDPR consent banners (`.cookie`, `.consent`, `.gdpr`, `.cookie-banner`).
    pub strip_cookie_banners: bool,
    /// Strip social sharing buttons (`.share`, `.social-share`, `.share-buttons`).
    pub strip_social_sharing: bool,
    /// Strictly preserve code blocks (`<pre>`, `<code>`) with exact formatting and indentation.
    pub preserve_code_blocks: bool,
    /// Strictly preserve table structures (`<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>`).
    pub preserve_tables: bool,
    /// User-defined CSS class substrings to always strip.
    pub custom_blocked_classes: Vec<String>,
    /// User-defined CSS ID substrings to always strip.
    pub custom_blocked_ids: Vec<String>,
    /// User-defined CSS class substrings to always preserve (overrides blocklists).
    pub custom_allowed_classes: Vec<String>,
}

impl Default for BoilerplatePolicy {
    fn default() -> Self {
        Self {
            strip_scripts_and_styles: true,
            strip_comments: true,
            strip_nav: true,
            strip_footers: true,
            strip_sidebars: true,
            strip_forms: true,
            strip_ads: true,
            strip_cookie_banners: true,
            strip_social_sharing: true,
            preserve_code_blocks: true,
            preserve_tables: true,
            custom_blocked_classes: Vec::new(),
            custom_blocked_ids: Vec::new(),
            custom_allowed_classes: Vec::new(),
        }
    }
}

impl BoilerplatePolicy {
    /// Configures whether to strip navigation elements.
    #[must_use]
    pub fn with_strip_nav(mut self, strip: bool) -> Self {
        self.strip_nav = strip;
        self
    }

    /// Configures whether to strip footers.
    #[must_use]
    pub fn with_strip_footers(mut self, strip: bool) -> Self {
        self.strip_footers = strip;
        self
    }

    /// Configures whether to strip sidebars.
    #[must_use]
    pub fn with_strip_sidebars(mut self, strip: bool) -> Self {
        self.strip_sidebars = strip;
        self
    }

    /// Configures whether to strip advertisements.
    #[must_use]
    pub fn with_strip_ads(mut self, strip: bool) -> Self {
        self.strip_ads = strip;
        self
    }

    /// Adds custom CSS class names or substrings to strip.
    #[must_use]
    pub fn with_blocked_class(mut self, class_name: impl Into<String>) -> Self {
        self.custom_blocked_classes.push(class_name.into());
        self
    }

    /// Adds custom CSS ID names or substrings to strip.
    #[must_use]
    pub fn with_blocked_id(mut self, id_name: impl Into<String>) -> Self {
        self.custom_blocked_ids.push(id_name.into());
        self
    }

    /// Adds custom CSS class names to unconditionally preserve.
    #[must_use]
    pub fn with_allowed_class(mut self, class_name: impl Into<String>) -> Self {
        self.custom_allowed_classes.push(class_name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boilerplate_policy_defaults() {
        let policy = BoilerplatePolicy::default();
        assert!(policy.strip_scripts_and_styles);
        assert!(policy.strip_comments);
        assert!(policy.strip_nav);
        assert!(policy.strip_footers);
        assert!(policy.strip_ads);
        assert!(policy.preserve_code_blocks);
        assert!(policy.preserve_tables);
    }

    #[test]
    fn test_boilerplate_policy_builder() {
        let policy = BoilerplatePolicy::default()
            .with_strip_nav(false)
            .with_blocked_class("promo-box")
            .with_blocked_id("newsletter-signup")
            .with_allowed_class("preserve-me");

        assert!(!policy.strip_nav);
        assert_eq!(policy.custom_blocked_classes, vec!["promo-box"]);
        assert_eq!(policy.custom_blocked_ids, vec!["newsletter-signup"]);
        assert_eq!(policy.custom_allowed_classes, vec!["preserve-me"]);
    }
}
