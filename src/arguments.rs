// Handles CLI argument parsing and related types for dfixxer
use crate::dfixxer_error::DFixxerError;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Command {
    UpdateFile,
    InitConfig,
}

pub struct Arguments {
    pub command: Command,
    pub filename: String,
    pub config_path: Option<String>,
}

/// Find a configuration file named 'dfixxer.toml' starting from the
/// directory of the provided filename and walking up parent directories.
/// Returns the first matching absolute or relative path as a String if found.
pub fn find_config_for_filename(filename: &str) -> Option<String> {
    let file_path = Path::new(filename);
    // Start from the file's directory if available, else current working directory
    let mut dir: PathBuf = file_path
        .parent()
        .map(|p| p.to_path_buf())
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    loop {
        let candidate = dir.join("dfixxer.toml");
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
        // Walk up to parent; stop if at filesystem root or no parent
        if let Some(parent) = dir.parent() {
            // If parent is the same as current (possible at root), break to avoid infinite loop
            if parent == dir {
                break;
            }
            dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

pub fn parse_args(args: Vec<String>) -> Result<Arguments, DFixxerError> {
    if args.len() < 2 {
        return Err(DFixxerError::InvalidArgs(format!(
            "Usage: {} <command> [<args>]\n\nCommands:\n  update <filename> [--config <path>]\n  init-config <filename>",
            args[0]
        )));
    }

    match args[1].as_str() {
        "update" => {
            if args.len() < 3 {
                return Err(DFixxerError::InvalidArgs(format!(
                    "Usage: {} update <filename> [--config <path>]",
                    args[0]
                )));
            }

            let filename = args[2].clone();
            let mut config_path: Option<String> = None;

            // Parse optional flags after the filename
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--config" => {
                        if i + 1 >= args.len() {
                            return Err(DFixxerError::InvalidArgs(
                                "Missing value for --config".to_string(),
                            ));
                        }
                        config_path = Some(args[i + 1].clone());
                        i += 2;
                    }
                    unknown => {
                        return Err(DFixxerError::InvalidArgs(format!(
                            "Unknown argument '{}' for 'update'. Usage: {} update <filename> [--config <path>]",
                            unknown, args[0]
                        )));
                    }
                }
            }

            // If --config was not provided, try to find dfixxer.toml upward from the file's directory
            if config_path.is_none() {
                config_path = find_config_for_filename(&filename);
            }

            Ok(Arguments {
                command: Command::UpdateFile,
                filename,
                config_path,
            })
        }
        "init-config" => {
            if args.len() < 3 {
                return Err(DFixxerError::InvalidArgs(format!(
                    "Usage: {} init-config <filename>",
                    args[0]
                )));
            }

            // Disallow any additional arguments, especially --config
            if args.len() > 3 {
                if args[3] == "--config" {
                    return Err(DFixxerError::InvalidArgs(
                        "The --config option can only be used with the 'update' command"
                            .to_string(),
                    ));
                } else {
                    return Err(DFixxerError::InvalidArgs(format!(
                        "Unknown argument '{}' for 'init-config'. Usage: {} init-config <filename>",
                        args[3], args[0]
                    )));
                }
            }

            Ok(Arguments {
                command: Command::InitConfig,
                filename: args[2].clone(),
                config_path: None,
            })
        }
        _ => Err(DFixxerError::InvalidArgs(format!(
            "Unknown command '{}'. Available commands: update, init-config",
            args[1]
        ))),
    }
}
