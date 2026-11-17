//! # API Server (`web-repo-to-ai-api`)
//!
//! Provides the entry point for the Axum REST and SSE server.

#![warn(missing_docs)]

#[tokio::main]
async fn main() {
    println!(
        "web-repo-to-ai-api v{} initialized",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_service_metadata() {
        assert_eq!(env!("CARGO_PKG_NAME"), "web-repo-to-ai-api");
    }
}
