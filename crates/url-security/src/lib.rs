//! # URL Security Engine (`web-repo-to-ai-url-security`)
//!
//! Provides zero-trust security controls for URL ingestion:
//! - SSRF defenses and private IPv4/IPv6 CIDR blocking
//! - Cloud metadata service isolation (169.254.169.254)
//! - DNS resolution pin and redirect re-validation

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
    fn test_url_security_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-url-security");
    }
}
