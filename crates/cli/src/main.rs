//! # `web-repo-to-ai` CLI Binary Entry Point

use std::process;
use web_repo_to_ai_cli::run_cli;

#[tokio::main]
async fn main() {
    match run_cli().await {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("[web-repo-to-ai:error] {err}");
            process::exit(err.exit_code());
        }
    }
}
