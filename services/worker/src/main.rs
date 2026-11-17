//! # Pipeline Worker Daemon (`web-repo-to-ai-worker`)
//!
//! Provides the entry point for the background asynchronous conversion pipeline daemon.

#![warn(missing_docs)]

#[tokio::main]
async fn main() {
    println!(
        "web-repo-to-ai-worker v{} initialized",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_worker_service_metadata() {
        assert_eq!(env!("CARGO_PKG_NAME"), "web-repo-to-ai-worker");
    }
}
