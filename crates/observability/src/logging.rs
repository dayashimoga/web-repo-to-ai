//! Structured logging and tracing subscriber initialization.
//!
//! Provides [`LoggingConfig`], [`LogFormat`], and [`init_tracing`] supporting
//! JSON, pretty, and compact output formats with dynamic `EnvFilter` control.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use web_repo_to_ai_core::{CoreError, Result};

/// Formatting style for log events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Single-line compact text formatting (ideal for local CLI and test runs).
    #[default]
    Compact,
    /// Multi-line human-readable pretty formatting with colors.
    Pretty,
    /// Structured JSON output (standard for production and Podman/Docker containers).
    Json,
}

impl FromStr for LogFormat {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "compact" => Ok(Self::Compact),
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            _ => Err(CoreError::Config(format!(
                "Invalid log format '{}': expected compact, pretty, or json",
                s
            ))),
        }
    }
}

/// Structured logging configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Desired output formatting style.
    pub format: LogFormat,
    /// Default log filter directive (e.g. "info", "debug,web_repo_to_ai=trace").
    pub level: String,
    /// Whether to include the module path/target in log records.
    pub with_target: bool,
    /// Whether to enforce automatic credential and secret redaction.
    pub redact_secrets: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Compact,
            level: "info".to_string(),
            with_target: true,
            redact_secrets: true,
        }
    }
}

impl LoggingConfig {
    /// Loads logging configuration from environment variables (`LOG_FORMAT`, `RUST_LOG`).
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(format_val) = std::env::var("LOG_FORMAT") {
            if let Ok(fmt) = format_val.parse() {
                config.format = fmt;
            }
        }

        if let Ok(level_val) = std::env::var("RUST_LOG") {
            if !level_val.trim().is_empty() {
                config.level = level_val;
            }
        }

        config
    }
}

/// Initializes the global tracing subscriber based on the provided configuration.
///
/// If a subscriber has already been installed (e.g. in test suites), this function
/// safely succeeds without panicking.
///
/// # Errors
/// Returns [`CoreError::Config`] if the log level directives are syntactically invalid.
pub fn init_tracing(config: &LoggingConfig) -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(filter);

    if config.redact_secrets {
        let make_writer = crate::redaction::RedactingMakeWriter::new(std::io::stdout);
        match config.format {
            LogFormat::Json => {
                let json_layer = tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(make_writer)
                    .with_target(config.with_target);
                let _ = registry.with(json_layer).try_init();
            }
            LogFormat::Pretty => {
                let pretty_layer = tracing_subscriber::fmt::layer()
                    .with_writer(make_writer)
                    .with_target(config.with_target);
                let _ = registry.with(pretty_layer).try_init();
            }
            LogFormat::Compact => {
                let compact_layer = tracing_subscriber::fmt::layer()
                    .compact()
                    .with_writer(make_writer)
                    .with_target(config.with_target);
                let _ = registry.with(compact_layer).try_init();
            }
        }
    } else {
        match config.format {
            LogFormat::Json => {
                let json_layer = tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(config.with_target);
                let _ = registry.with(json_layer).try_init();
            }
            LogFormat::Pretty => {
                let pretty_layer = tracing_subscriber::fmt::layer().with_target(config.with_target);
                let _ = registry.with(pretty_layer).try_init();
            }
            LogFormat::Compact => {
                let compact_layer = tracing_subscriber::fmt::layer()
                    .compact()
                    .with_target(config.with_target);
                let _ = registry.with(compact_layer).try_init();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Pretty);
        assert_eq!("compact".parse::<LogFormat>().unwrap(), LogFormat::Compact);
        assert!("invalid".parse::<LogFormat>().is_err());
    }

    #[test]
    fn test_init_tracing_idempotent() {
        let config = LoggingConfig::default();
        assert!(init_tracing(&config).is_ok());
        // Second invocation in the same process must not panic
        assert!(init_tracing(&config).is_ok());
    }
}
