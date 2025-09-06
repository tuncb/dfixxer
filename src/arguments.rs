// Handles CLI argument parsing and related types for dfixxer
use crate::dfixxer_error::DFixxerError;
use clap::{Parser, Subcommand, ValueEnum};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    /// No logging output
    Off,
    /// Only error messages
    Error,
    /// Error and warning messages
    Warn,
    /// Error, warning, and info messages
    Info,
    /// Error, warning, info, and debug messages
    Debug,
    /// All log messages including trace
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(Debug)]
pub enum Command {
    UpdateFile,
    CheckFile,
    InitConfig,
    Parse,
}

pub struct Arguments {
    pub command: Command,
    pub filename: String,
    pub config_path: Option<String>,
    pub log_level: Option<LogLevel>,
}

#[derive(Parser, Debug)]
#[command(name = "dfixxer", about = "Fix Delphi/Pascal files", version)]
struct Cli {
    /// Set the logging level
    #[arg(long = "log-level", short = 'l', value_enum, global = true)]
    log_level: Option<LogLevel>,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Update a file using configuration rules
    Update {
        /// The filename to update
        filename: String,
        /// Path to the configuration file
        #[arg(long = "config")]
        config: Option<String>,
    },
    /// Check a file and show what would be changed without modifying it
    Check {
        /// The filename to check
        filename: String,
        /// Path to the configuration file
        #[arg(long = "config")]
        config: Option<String>,
    },
    /// Initialize configuration for a file
    InitConfig {
        /// The filename to initialize configuration for
        filename: String,
    },
    /// Parse a file and print its AST
    Parse {
        /// The filename to parse
        filename: String,
    },
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
    // Parse arguments using clap
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) => {
            // Check if this is a help or version request (which should exit with code 0)
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                // Print the help/version and exit successfully
                print!("{}", e);
                std::process::exit(0);
            } else {
                // For other errors, return as DFixxerError
                return Err(DFixxerError::InvalidArgs(e.to_string()));
            }
        }
    };

    match cli.command {
        CliCommand::Update { filename, config } => {
            // If --config was not provided, try to find dfixxer.toml upward from the file's directory
            let config_path = match config {
                Some(path) => Some(path),
                None => find_config_for_filename(&filename),
            };

            Ok(Arguments {
                command: Command::UpdateFile,
                filename,
                config_path,
                log_level: cli.log_level,
            })
        }
        CliCommand::Check { filename, config } => {
            // If --config was not provided, try to find dfixxer.toml upward from the file's directory
            let config_path = match config {
                Some(path) => Some(path),
                None => find_config_for_filename(&filename),
            };

            Ok(Arguments {
                command: Command::CheckFile,
                filename,
                config_path,
                log_level: cli.log_level,
            })
        }
        CliCommand::InitConfig { filename } => Ok(Arguments {
            command: Command::InitConfig,
            filename,
            config_path: None,
            log_level: cli.log_level,
        }),
        CliCommand::Parse { filename } => Ok(Arguments {
            command: Command::Parse,
            filename,
            config_path: None,
            log_level: cli.log_level,
        }),
    }
}
