//! # CLI Config Command Execution Handler
//!
//! Handles `web-repo-to-ai config` subactions:
//! - `show`: display active configuration and resolved settings
//! - `init`: initialize a starter template config file
//! - `validate`: syntax and schema verification of config file
//! - `path`: print the discovered active config file location

use std::fs;
use std::path::{Path, PathBuf};

use crate::args::{ConfigAction, ConfigArgs};
use crate::config::{
    find_config_file, generate_template_json, generate_template_toml, load_config, mask_secret,
    CliConfigFile, CONFIG_FILE_NAME_JSON, CONFIG_FILE_NAME_TOML,
};
use crate::error::{CliError, ExitCode};
use crate::output::OutputWriter;

/// Executes the `config` command.
///
/// # Errors
///
/// Returns [`CliError`] on file I/O failure, validation failure, or schema invalidity.
pub fn execute_config(
    args: &ConfigArgs,
    global_config_path: Option<&Path>,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let action = args.action.clone().unwrap_or(ConfigAction::Show {
        path: None,
        reveal_secrets: false,
        json: false,
    });

    match action {
        ConfigAction::Show {
            path,
            reveal_secrets,
            json,
        } => execute_config_show(
            path.as_deref(),
            reveal_secrets,
            json || writer.json,
            global_config_path,
            writer,
        ),
        ConfigAction::Init { path, force, json } => {
            execute_config_init(path.as_deref(), force, json || writer.json, writer)
        }
        ConfigAction::Validate { path } => {
            execute_config_validate(path.as_deref(), global_config_path, writer)
        }
        ConfigAction::Path { path } => {
            execute_config_path(path.as_deref(), global_config_path, writer)
        }
    }
}

fn execute_config_show(
    path_opt: Option<&Path>,
    reveal_secrets: bool,
    json: bool,
    global_config_path: Option<&Path>,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let explicit = path_opt.or(global_config_path);
    let discovered = find_config_file(explicit);

    let config = if let Some(ref path) = discovered {
        writer.status(&format!("Discovered configuration at: {}", path.display()));
        load_config(Some(path))?.unwrap_or_default()
    } else {
        writer.status("No configuration file found; showing default settings.");
        CliConfigFile::default()
    };

    if json {
        let mut json_val = serde_json::to_value(&config).map_err(|e| {
            CliError::Validation(format!("Failed to serialize config to JSON: {e}"))
        })?;
        if !reveal_secrets {
            if let Some(obj) = json_val.as_object_mut() {
                if let Some(api) = obj.get_mut("api").and_then(|v| v.as_object_mut()) {
                    if let Some(key) = api.get_mut("api_key").and_then(|v| v.as_str()) {
                        let masked = mask_secret(key);
                        api.insert("api_key".to_string(), serde_json::Value::String(masked));
                    }
                }
                if let Some(gh) = obj.get_mut("github").and_then(|v| v.as_object_mut()) {
                    if let Some(tok) = gh.get_mut("token").and_then(|v| v.as_str()) {
                        let masked = mask_secret(tok);
                        gh.insert("token".to_string(), serde_json::Value::String(masked));
                    }
                }
            }
        }
        let formatted = serde_json::to_string_pretty(&json_val)
            .map_err(|e| CliError::Validation(format!("Failed to format config JSON: {e}")))?;
        println!("{formatted}");
    } else {
        let toml_str = config.to_toml_string(!reveal_secrets);
        println!("{toml_str}");
    }

    Ok(ExitCode::Success as i32)
}

fn execute_config_init(
    path_opt: Option<&Path>,
    force: bool,
    json: bool,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let target_path = if let Some(p) = path_opt {
        p.to_path_buf()
    } else if json {
        PathBuf::from(CONFIG_FILE_NAME_JSON)
    } else {
        PathBuf::from(CONFIG_FILE_NAME_TOML)
    };

    if target_path.exists() && !force {
        return Err(CliError::Validation(format!(
            "Configuration file '{}' already exists. Use --force to overwrite.",
            target_path.display()
        )));
    }

    let is_json = json || target_path.extension().and_then(|e| e.to_str()) == Some("json");
    let content = if is_json {
        generate_template_json()
    } else {
        generate_template_toml()
    };

    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::Validation(format!(
                    "Failed to create directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }
    }

    fs::write(&target_path, content).map_err(|e| {
        CliError::Validation(format!(
            "Failed to write configuration file '{}': {e}",
            target_path.display()
        ))
    })?;

    writer.status(&format!(
        "Initialized configuration file: {}",
        target_path.display()
    ));

    println!("Initialized configuration file: {}", target_path.display());
    Ok(ExitCode::Success as i32)
}

fn execute_config_validate(
    path_opt: Option<&Path>,
    global_config_path: Option<&Path>,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let explicit = path_opt.or(global_config_path);
    let file_path = match (explicit, find_config_file(explicit)) {
        (Some(p), None) => {
            return Err(CliError::Validation(format!(
                "Specified configuration file not found: {}",
                p.display()
            )));
        }
        (_, Some(path)) => path,
        (None, None) => {
            return Err(CliError::Validation(
                "No configuration file found to validate.".to_string(),
            ));
        }
    };

    load_config(Some(&file_path))?;

    writer.status(&format!(
        "Configuration file '{}' is valid.",
        file_path.display()
    ));
    println!("Configuration file '{}' is valid.", file_path.display());

    Ok(ExitCode::Success as i32)
}

fn execute_config_path(
    path_opt: Option<&Path>,
    global_config_path: Option<&Path>,
    writer: &OutputWriter,
) -> Result<i32, CliError> {
    let explicit = path_opt.or(global_config_path);
    if let Some(path) = find_config_file(explicit) {
        println!("{}", path.display());
        Ok(ExitCode::Success as i32)
    } else {
        writer.error("No configuration file found.");
        eprintln!("No configuration file found.");
        Ok(ExitCode::Validation as i32)
    }
}
