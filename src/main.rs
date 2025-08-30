mod dfixxer_error;
use dfixxer_error::DFixxerError;
mod arguments;
use arguments::{Command, parse_args};
mod options;
use options::Options;
mod replacements;
mod uses_section;
use replacements::{TextReplacement, apply_replacements, print_replacements};
mod parser;
use parser::parse;

use crate::uses_section::transform_parser_uses_section_to_replacement;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A timing collector that tracks multiple operations and can provide summaries
struct TimingCollector {
    timings: HashMap<String, Duration>,
}

impl TimingCollector {
    fn new() -> Self {
        Self {
            timings: HashMap::new(),
        }
    }

    /// Time an operation and store the result
    fn time_operation<T, F>(&mut self, operation_name: &str, operation: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        log::debug!("{} took: {:?}", operation_name, duration);
        self.timings.insert(operation_name.to_string(), duration);
        result
    }

    /// Time a fallible operation and store the result
    fn time_operation_result<T, E, F>(&mut self, operation_name: &str, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        log::debug!("{} took: {:?}", operation_name, duration);
        self.timings.insert(operation_name.to_string(), duration);
        result
    }

    /// Log a summary of all collected timings
    fn log_summary(&self) {
        let total_processing: Duration = self.timings.values().sum();

        log::info!("Performance summary:");
        for (operation, duration) in &self.timings {
            log::info!("  {}: {:?}", operation, duration);
        }
        log::info!("  Total processing: {:?}", total_processing);
    }
}

fn load_file(filename: &str) -> Result<String, DFixxerError> {
    Ok(std::fs::read_to_string(filename)?)
}

/// Process a file and return the replacements that would be made
fn process_file(
    filename: &str,
    config_path: Option<&str>,
    timing: &mut TimingCollector,
) -> Result<(String, Vec<TextReplacement>), DFixxerError> {
    // Load options from config file, or use defaults if not found
    let config_path = config_path.unwrap_or("dfixxer.toml");
    let options: Options = Options::load_or_default(config_path);

    // Time file loading
    let source = timing.time_operation_result("File loading", || load_file(filename))?;

    // Time parsing
    let parse_result = timing.time_operation_result("Parsing", || parse(&source))?;

    // Time transformation
    let replacements: Vec<TextReplacement> = timing.time_operation("Transformation", || {
        parse_result
            .uses_sections
            .iter()
            .filter_map(|uses_section| {
                transform_parser_uses_section_to_replacement(uses_section, &options, &source)
            })
            .collect()
    });

    Ok((source, replacements))
}

fn run() -> Result<(), DFixxerError> {
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_args(args)?;

    match arguments.command {
        Command::UpdateFile => {
            let mut timing = TimingCollector::new();

            let (source, replacements) = process_file(
                &arguments.filename,
                arguments.config_path.as_deref(),
                &mut timing,
            )?;

            // Time applying replacements
            if !replacements.is_empty() {
                timing.time_operation_result("Applying replacements", || {
                    apply_replacements(&arguments.filename, &source, replacements)
                })?;
            }

            // Log the timing summary
            timing.log_summary();
        }
        Command::CheckFile => {
            let mut timing = TimingCollector::new();

            let (source, replacements) = process_file(
                &arguments.filename,
                arguments.config_path.as_deref(),
                &mut timing,
            )?;

            // Print replacements instead of applying them
            print_replacements(&source, &replacements);

            // Log the timing summary
            timing.log_summary();
        }
        Command::InitConfig => {
            println!("Initializing configuration...");
            match Options::create_default_config(&arguments.filename) {
                Ok(()) => println!("Created default configuration file: {}", arguments.filename),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    // Parse arguments first to get log level
    let args: Vec<String> = std::env::args().collect();
    if let Ok(arguments) = parse_args(args) {
        // Set log level from command line arguments if provided
        if let Some(log_level) = &arguments.log_level {
            unsafe {
                std::env::set_var("RUST_LOG", log_level.as_str());
            }
        }
    }

    env_logger::init();

    // Time the entire run function
    let start_total = Instant::now();
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    let total_duration = start_total.elapsed();
    log::info!("Total execution time: {:?}", total_duration);
}
