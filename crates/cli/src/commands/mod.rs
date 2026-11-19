//! # CLI Subcommand Execution Handlers

pub mod batch;
pub mod completions;
pub mod config;
pub mod repo;
pub mod url;
pub mod verify;

pub use batch::{execute_batch, execute_batch_with_options, BatchItemResult, BatchJsonEnvelope};
pub use completions::execute_completions;
pub use config::execute_config;
pub use repo::{execute_repo, execute_repo_with_client};
pub use url::execute_url;
pub use verify::{execute_verify, VerifyItemResult, VerifyJsonEnvelope};
