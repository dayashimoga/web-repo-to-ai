//! # Dual Binary Alias Execution Integration Tests
//!
//! Validates that both the primary binary (`web-repo-to-ai`) and the alias binary
//! (`web-repo-to-ai-cli`) compile, run, and provide identical command interfaces.

use web_repo_to_ai_cli::{run_cli_with_args, ExitCode};

#[tokio::test]
async fn test_primary_and_alias_binary_version() {
    let res_primary = run_cli_with_args(["web-repo-to-ai", "--version"]).await;
    assert_eq!(res_primary.unwrap(), ExitCode::Success as i32);

    let res_alias = run_cli_with_args(["web-repo-to-ai-cli", "--version"]).await;
    assert_eq!(res_alias.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_primary_and_alias_binary_help() {
    let res_primary = run_cli_with_args(["web-repo-to-ai", "--help"]).await;
    assert_eq!(res_primary.unwrap(), ExitCode::Success as i32);

    let res_alias = run_cli_with_args(["web-repo-to-ai-cli", "--help"]).await;
    assert_eq!(res_alias.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_primary_and_alias_subcommand_dispatch() {
    let subcommands = [
        "url",
        "repo",
        "config",
        "batch",
        "verify",
        "completions",
        "mcp",
    ];

    for sc in &subcommands {
        let code_prim = run_cli_with_args(["web-repo-to-ai", sc, "--help"]).await;
        assert_eq!(code_prim.unwrap(), ExitCode::Success as i32);

        let code_alias = run_cli_with_args(["web-repo-to-ai-cli", sc, "--help"]).await;
        assert_eq!(code_alias.unwrap(), ExitCode::Success as i32);
    }
}
