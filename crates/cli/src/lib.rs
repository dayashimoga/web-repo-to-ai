//! # CLI Library (`web-repo-to-ai-cli`)
//!
//! Provides the command-line interface entry points, argument parsers, execution
//! dispatchers, and output formatting for `web-repo-to-ai`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod args;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

use clap::Parser;
use std::sync::Arc;
use web_repo_to_ai_github_extractor::client::GitHubApiClient;
use web_repo_to_ai_url_security::SecureDnsResolver;
use web_repo_to_ai_web_extractor::http::HttpTransport;

pub use args::{
    BatchArgs, Cli, Commands, ConfigAction, ConfigArgs, OutputFormat, RepoArgs, UrlArgs,
};
pub use commands::batch::{
    execute_batch, execute_batch_with_options, BatchItemResult, BatchJsonEnvelope,
};
pub use commands::config::execute_config;
pub use commands::repo::{execute_repo, execute_repo_with_client};
pub use commands::url::{execute_url, execute_url_with_options, execute_url_with_transport};
pub use config::{find_config_file, load_config, parse_config_str, CliConfigFile};
pub use error::{CliError, ExitCode};
pub use output::{export_artifact, CliJsonEnvelope, OutputWriter};

/// Runs the CLI application using command-line arguments from `std::env::args_os`.
///
/// # Errors
///
/// Returns [`CliError`] on command failure, argument validation error, or I/O failure.
pub async fn run_cli() -> Result<i32, CliError> {
    let cli = Cli::parse();
    run_with_parsed(cli, None, None, None).await
}

/// Runs the CLI application using custom command-line arguments (useful for programmatic invocation).
///
/// # Errors
///
/// Returns [`CliError`] on argument parsing failure, validation error, or execution failure.
pub async fn run_cli_with_args<I, T>(args: I) -> Result<i32, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    run_cli_with_args_and_transport(args, None).await
}

/// Runs the CLI application using custom command-line arguments and an optional custom HTTP transport.
///
/// # Errors
///
/// Returns [`CliError`] on argument parsing failure, validation error, or execution failure.
pub async fn run_cli_with_args_and_transport<I, T>(
    args: I,
    transport: Option<Arc<dyn HttpTransport>>,
) -> Result<i32, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    run_cli_with_args_and_options(args, transport, None).await
}

/// Runs the CLI application using custom command-line arguments, optional transport, and optional DNS resolver.
///
/// # Errors
///
/// Returns [`CliError`] on argument parsing failure, validation error, or execution failure.
pub async fn run_cli_with_args_and_options<I, T>(
    args: I,
    transport: Option<Arc<dyn HttpTransport>>,
    resolver: Option<SecureDnsResolver>,
) -> Result<i32, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    run_cli_with_args_and_clients(args, transport, resolver, None).await
}

/// Runs the CLI application using custom command-line arguments, optional transport, optional DNS resolver, and optional GitHub client.
///
/// # Errors
///
/// Returns [`CliError`] on argument parsing failure, validation error, or execution failure.
pub async fn run_cli_with_args_and_clients<I, T>(
    args: I,
    transport: Option<Arc<dyn HttpTransport>>,
    resolver: Option<SecureDnsResolver>,
    github_client: Option<GitHubApiClient>,
) -> Result<i32, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let is_help_or_version = e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion;
            let msg = e.to_string();
            if is_help_or_version {
                print!("{msg}");
                return Ok(0);
            }
            eprintln!("{msg}");
            return Err(CliError::Validation(msg));
        }
    };
    run_with_parsed(cli, transport, resolver, github_client).await
}

async fn run_with_parsed(
    cli: Cli,
    transport: Option<Arc<dyn HttpTransport>>,
    resolver: Option<SecureDnsResolver>,
    github_client: Option<GitHubApiClient>,
) -> Result<i32, CliError> {
    let writer = OutputWriter::new(cli.verbose, cli.quiet, cli.json);
    let config_file = load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Url(mut url_args) => {
            if let Some(ref cfg) = config_file {
                cfg.apply_to_url_args(&mut url_args);
            }
            execute_url_with_options(&url_args, &writer, transport, resolver).await
        }
        Commands::Repo(mut repo_args) => {
            if let Some(ref cfg) = config_file {
                cfg.apply_to_repo_args(&mut repo_args);
            }
            execute_repo_with_client(&repo_args, &writer, github_client).await
        }
        Commands::Batch(mut batch_args) => {
            if let Some(ref cfg) = config_file {
                cfg.apply_to_batch_args(&mut batch_args);
            }
            execute_batch_with_options(&batch_args, &writer, transport, resolver, github_client)
                .await
        }
        Commands::Config(config_args) => {
            execute_config(&config_args, cli.config.as_deref(), &writer)
        }
    }
}
