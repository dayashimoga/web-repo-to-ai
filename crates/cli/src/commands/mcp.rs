//! # MCP Command Handler
//!
//! Spawns the Model Context Protocol (MCP) server daemon over standard I/O (stdio).

use web_repo_to_ai_mcp::{McpServer, StdioTransport};

use crate::args::McpArgs;
use crate::error::CliError;

/// Executes the `mcp` subcommand.
pub async fn execute_mcp(args: &McpArgs) -> Result<i32, CliError> {
    StdioTransport::init_stdio_tracing();

    let server = McpServer::builder()
        .name(args.name.clone())
        .version(env!("CARGO_PKG_VERSION"))
        .instructions(
            "Universal URL & GitHub Repository to AI-Ready Context Converter. \
             Converts live webpages and GitHub repositories into optimized context for LLMs.",
        )
        .with_default_tools()
        .build();

    let transport = StdioTransport::new();
    server
        .run(transport)
        .await
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

    Ok(0)
}
