//! # web-repo-to-ai-mcp Binary Entry Point
//!
//! Executable daemon running the Model Context Protocol (MCP) server over standard I/O (stdio).
//! Directs all diagnostics and tracing strictly to standard error (`stderr`), keeping `stdout`
//! reserved 100% for JSON-RPC 2.0 communication with AI agent clients (Claude Desktop, Antigravity, Cursor).

use clap::Parser;
use tracing::info;
use web_repo_to_ai_mcp::{McpServer, StdioTransport};

/// Command-line arguments for web-repo-to-ai-mcp.
#[derive(Debug, Parser)]
#[command(
    name = "web-repo-to-ai-mcp",
    about = "Model Context Protocol (MCP) server for web-repo-to-ai Universal URL & GitHub to AI converter",
    version
)]
struct Args {
    /// Enable verbose debug logging to stderr
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Override the reported server name
    #[arg(long, default_value = "web-repo-to-ai-mcp")]
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing to stderr
    let filter = if args.verbose { "debug" } else { "info" };
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!(
        server = %args.name,
        version = env!("CARGO_PKG_VERSION"),
        "Starting web-repo-to-ai Model Context Protocol (MCP) server on stdio"
    );

    let server = McpServer::builder()
        .name(args.name)
        .version(env!("CARGO_PKG_VERSION"))
        .instructions(
            "Universal URL & GitHub Repository to AI-Ready Context Converter. \
             Converts live webpages and GitHub repositories into optimized context for LLMs.",
        )
        .build();

    let transport = StdioTransport::new();
    if let Err(e) = server.run(transport).await {
        eprintln!("web-repo-to-ai-mcp fatal error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
