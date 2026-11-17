//! # Context Optimizer (`web-repo-to-ai-context-optimizer`)
//!
//! Fits canonical documents into strict target LLM context budgets:
//! - Importance scoring and structural file prioritization
//! - Intelligent boilerplate and comment stripping
//! - Dynamic context headroom allocation

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
    fn test_context_optimizer_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-context-optimizer");
    }
}
