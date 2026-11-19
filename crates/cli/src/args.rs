//! # CLI Command-Line Argument Definitions

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root command-line parser for `web-repo-to-ai`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "web-repo-to-ai",
    author,
    version,
    about = "Universal URL & GitHub Repository to AI-Ready Context Converter",
    long_about = "High-performance CLI that extracts, normalizes, deduplicates, and converts web URLs and GitHub repositories into token-optimized Markdown, JSON, XML, and other LLM context formats."
)]
pub struct Cli {
    /// Enable verbose diagnostic logging to stderr
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress informational and progress messages on stderr
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output structured execution metadata envelope in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Optional path to custom configuration file
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Convert a single web documentation or article URL into AI-ready format
    Url(UrlArgs),
    /// Convert a GitHub repository or local directory into AI-ready format
    #[command(alias = "repository")]
    Repo(RepoArgs),
    /// View, initialize, or validate CLI configuration files
    Config(ConfigArgs),
    /// Batch convert multiple URLs, repositories, or local directories
    Batch(BatchArgs),
    /// Verify cryptographic checksums or schema compliance of converted artifacts
    Verify(VerifyArgs),
    /// Generate shell completion scripts for supported shells
    Completions(CompletionsArgs),
}

/// Supported output artifact formats.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Concatenated clean Markdown (.md)
    Markdown,
    /// Clean plain text (.txt)
    Txt,
    /// Canonical JSON CDM schema v1.0 (.json)
    Json,
    /// Standalone sanitized HTML (.html)
    Html,
    /// Fine-tuning & RAG JSON Lines (.jsonl)
    Jsonl,
    /// Streaming ZIP bundle (.zip)
    Zip,
    /// Structural JSON package manifest (.json)
    Manifest,
    /// Cryptographic SHA-256 / BLAKE3 checksums (.txt)
    Checksums,
}

impl OutputFormat {
    /// Returns the canonical string representation for this format.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Txt => "txt",
            Self::Json => "json",
            Self::Html => "html",
            Self::Jsonl => "jsonl",
            Self::Zip => "zip",
            Self::Manifest => "manifest",
            Self::Checksums => "checksums",
        }
    }

    /// Returns standard file extension for this format.
    #[must_use]
    pub const fn default_extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Txt => "txt",
            Self::Json | Self::Manifest => "json",
            Self::Html => "html",
            Self::Jsonl => "jsonl",
            Self::Zip => "zip",
            Self::Checksums => "txt",
        }
    }

    /// Returns whether this format is binary.
    #[must_use]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Zip)
    }
}

/// Arguments for the `url` command.
#[derive(Args, Debug, Clone)]
pub struct UrlArgs {
    /// Target web URL to extract and convert
    #[arg(value_name = "URL")]
    pub url: String,

    /// Output file path, or '-' for stdout
    #[arg(short, long, default_value = "-", value_name = "PATH")]
    pub output: String,

    /// Output artifact format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Target context token budget ceiling
    #[arg(short, long, default_value_t = 160_000, value_name = "TOKENS")]
    pub budget: usize,

    /// Target model identifier for token counting
    #[arg(short, long, default_value = "cl100k_base", value_name = "MODEL")]
    pub model: String,

    /// Disable context optimization (deduplication & compression)
    #[arg(long)]
    pub no_optimize: bool,

    /// HTTP request timeout in seconds
    #[arg(long, default_value_t = 30, value_name = "SECONDS")]
    pub timeout: u64,

    /// Custom HTTP User-Agent header string
    #[arg(long, value_name = "UA")]
    pub user_agent: Option<String>,

    /// Remote web-repo-to-ai API base URL (e.g. http://localhost:8080)
    #[arg(long, env = "WEB_REPO_TO_AI_API_URL", value_name = "URL")]
    pub api_url: Option<String>,

    /// Remote API key for authentication
    #[arg(long, env = "WEB_REPO_TO_AI_API_KEY", value_name = "KEY")]
    pub api_key: Option<String>,
}

/// Arguments for the `repo` subcommand.
#[derive(Args, Debug, Clone)]
pub struct RepoArgs {
    /// Target repository identifier (e.g. 'owner/repo', GitHub URL, or local filesystem directory)
    #[arg(value_name = "REPO")]
    pub repo: String,

    /// Output file path, or '-' for stdout
    #[arg(short, long, default_value = "-", value_name = "PATH")]
    pub output: String,

    /// Output artifact format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Target context token budget ceiling
    #[arg(short, long, default_value_t = 160_000, value_name = "TOKENS")]
    pub budget: usize,

    /// Target model identifier for token counting
    #[arg(short, long, default_value = "cl100k_base", value_name = "MODEL")]
    pub model: String,

    /// Git branch, tag, or commit reference to extract
    #[arg(short, long, value_name = "REF")]
    pub r#ref: Option<String>,

    /// Subdirectory or subtree path within the repository
    #[arg(short, long, value_name = "SUBPATH")]
    pub subpath: Option<String>,

    /// Disable context optimization (deduplication & compression)
    #[arg(long)]
    pub no_optimize: bool,

    /// HTTP request timeout in seconds
    #[arg(long, default_value_t = 60, value_name = "SECONDS")]
    pub timeout: u64,

    /// GitHub access token for private repositories or increased rate limits
    #[arg(long, env = "GITHUB_TOKEN", value_name = "TOKEN")]
    pub github_token: Option<String>,

    /// Remote web-repo-to-ai API base URL (e.g. http://localhost:8080)
    #[arg(long, env = "WEB_REPO_TO_AI_API_URL", value_name = "URL")]
    pub api_url: Option<String>,

    /// Remote API key for authentication
    #[arg(long, env = "WEB_REPO_TO_AI_API_KEY", value_name = "KEY")]
    pub api_key: Option<String>,
}

/// Arguments for the `config` subcommand.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct ConfigArgs {
    /// Subcommand action to execute on configuration
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

/// Subactions supported under the `config` command.
#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum ConfigAction {
    /// Display current active configuration and resolved values
    Show {
        /// Path to a specific configuration file to view
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Show unmasked secrets (tokens and API keys)
        #[arg(long, default_value_t = false)]
        reveal_secrets: bool,

        /// Format output as JSON instead of TOML
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Create a starter configuration file template
    Init {
        /// Destination path for the new configuration file
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Overwrite existing file when initializing
        #[arg(short, long, default_value_t = false)]
        force: bool,

        /// Format template as JSON instead of TOML
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Validate configuration file syntax and schema
    Validate {
        /// Path to configuration file to validate
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Print path of discovered active configuration file
    Path {
        /// Explicit path to check
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,
    },
}

/// Arguments for the `batch` subcommand.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct BatchArgs {
    /// File containing URLs or repository targets (one per line, '-' for stdin)
    #[arg(short = 'i', long = "input-file", value_name = "PATH")]
    pub input_file: Option<PathBuf>,

    /// Direct positional targets to convert in batch (URLs, GitHub repos, or local paths)
    #[arg(value_name = "TARGETS")]
    pub targets: Vec<String>,

    /// Output directory for converted artifacts
    #[arg(short, long, default_value = "./output", value_name = "DIR")]
    pub output_dir: PathBuf,

    /// Output format for generated context documents
    #[arg(short, long, value_enum, default_value = "markdown")]
    pub format: OutputFormat,

    /// Target context token budget per source
    #[arg(short, long, default_value_t = 160_000, value_name = "TOKENS")]
    pub budget: usize,

    /// Model name for token counting
    #[arg(short, long, default_value = "cl100k_base", value_name = "MODEL")]
    pub model: String,

    /// Disable context budget optimization
    #[arg(long, default_value_t = false)]
    pub no_optimize: bool,

    /// Concurrency limit for parallel batch processing
    #[arg(short = 'c', long, default_value_t = 4, value_name = "N")]
    pub concurrency: usize,

    /// Continue processing remaining items if an individual conversion fails
    #[arg(long, default_value_t = true)]
    pub continue_on_error: bool,

    /// Network timeout in seconds per conversion
    #[arg(short, long, default_value_t = 60, value_name = "SECONDS")]
    pub timeout: u64,

    /// GitHub personal access token for remote repositories
    #[arg(long, env = "GITHUB_TOKEN", value_name = "TOKEN")]
    pub github_token: Option<String>,

    /// Remote API base URL (e.g. http://localhost:8080)
    #[arg(long, env = "WEB_REPO_TO_AI_API_URL", value_name = "URL")]
    pub api_url: Option<String>,

    /// Remote API key for authentication
    #[arg(long, env = "WEB_REPO_TO_AI_API_KEY", value_name = "KEY")]
    pub api_key: Option<String>,
}

/// Shell target for autocompletion generation.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ShellType {
    /// Bourne Again SHell (bash)
    Bash,
    /// Elvish shell
    Elvish,
    /// Friendly Interactive SHell (fish)
    Fish,
    /// Microsoft PowerShell
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
    /// Z Shell (zsh)
    Zsh,
}

impl ShellType {
    /// Converts to clap_complete Shell enum.
    #[must_use]
    pub fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            Self::Bash => clap_complete::Shell::Bash,
            Self::Elvish => clap_complete::Shell::Elvish,
            Self::Fish => clap_complete::Shell::Fish,
            Self::PowerShell => clap_complete::Shell::PowerShell,
            Self::Zsh => clap_complete::Shell::Zsh,
        }
    }
}

/// Arguments for the `completions` subcommand.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct CompletionsArgs {
    /// Target shell to generate completion script for
    #[arg(value_enum)]
    pub shell: ShellType,
}

/// Arguments for the `verify` subcommand.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct VerifyArgs {
    /// Path to artifact file, manifest, checksums file, or directory to verify
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Expected SHA-256 hex digest to verify against
    #[arg(long, value_name = "HEX")]
    pub sha256: Option<String>,

    /// Expected BLAKE3 hex digest to verify against
    #[arg(long, value_name = "HEX")]
    pub blake3: Option<String>,

    /// Remote job ID to fetch and verify artifacts for from API
    #[arg(long, value_name = "JOB_ID")]
    pub job_id: Option<String>,

    /// Remote API base URL (e.g. http://localhost:8080)
    #[arg(long, env = "WEB_REPO_TO_AI_API_URL", value_name = "URL")]
    pub api_url: Option<String>,

    /// Remote API key for authentication
    #[arg(long, env = "WEB_REPO_TO_AI_API_KEY", value_name = "KEY")]
    pub api_key: Option<String>,

    /// Strict CDM structural validation for JSON and Markdown artifacts
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}
