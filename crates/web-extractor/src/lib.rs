//! # Web Extractor (`web-repo-to-ai-web-extractor`)
//!
//! Extracts clean, structured content from web documentation and articles:
//! - HTML DOM parsing and boilerplate reduction
//! - Documentation hierarchy and code block extraction
//! - Dynamic browser fallback integration for SPA hydration

#![warn(missing_docs)]

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_extractor_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-web-extractor");
    }
}
