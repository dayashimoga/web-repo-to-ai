//! # CLI Shell Autocompletions Integration Tests
//!
//! Validates `web-repo-to-ai completions <shell>` generation across:
//! - Bash
//! - Zsh
//! - Fish
//! - PowerShell
//! - Elvish

use web_repo_to_ai_cli::{run_cli_with_args, ExitCode};

#[tokio::test]
async fn test_completions_cli_help() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "--help"]).await;
    assert_eq!(res.unwrap(), 0);
}

#[tokio::test]
async fn test_completions_bash_generation() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "bash"]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_completions_zsh_generation() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "zsh"]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_completions_fish_generation() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "fish"]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_completions_powershell_generation() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "powershell"]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_completions_elvish_generation() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "elvish"]).await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_completions_invalid_shell_fails() {
    let res = run_cli_with_args(["web-repo-to-ai", "completions", "cmd.exe"]).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.exit_code(), ExitCode::Validation as i32);
}
