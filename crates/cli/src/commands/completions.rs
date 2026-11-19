//! # Shell Completion Generation Command Execution Engine
//!
//! Generates native shell autocompletion scripts for Bash, Zsh, Fish, PowerShell,
//! and Elvish shells via `clap_complete`.

use clap::CommandFactory;
use clap_complete::generate;
use std::io;

use crate::args::{Cli, CompletionsArgs};
use crate::error::{CliError, ExitCode};

/// Executes the `completions` command, writing the shell completion script to stdout.
///
/// # Errors
///
/// Returns [`CliError`] on standard output failure.
pub fn execute_completions(args: &CompletionsArgs) -> Result<i32, CliError> {
    let mut cmd = Cli::command();
    let shell = args.shell.to_clap_shell();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(ExitCode::Success as i32)
}
