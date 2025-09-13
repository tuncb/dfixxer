mod dfixxer_error;
use dfixxer_error::DFixxerError;
mod arguments;
use arguments::{Command, parse_args};
mod options;
use options::Options;
mod replacements;
mod transform_procedure_section;
mod transform_single_keyword_sections;
mod transform_text;
mod transform_unit_program_section;
mod transform_uses_section;
mod transformer_utility;
use replacements::{
    TextReplacement, fill_gaps_with_identity_replacements, merge_replacements, print_replacements,
};
mod parser;
use parser::parse;

use crate::transform_procedure_section::transform_procedure_section;
use crate::transform_single_keyword_sections::transform_single_keyword_section;
use crate::transform_text::apply_text_transformations;
use crate::transform_unit_program_section::transform_unit_program_section;
use crate::transform_uses_section::transform_uses_section;
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
    let mut replacements: Vec<TextReplacement> = timing.time_operation("Transformation", || {
        parse_result
            .code_sections
            .iter()
            .filter_map(|code_section| match code_section.keyword.kind {
                parser::Kind::Uses => {
                    if options.transformations.enable_uses_section {
                        transform_uses_section(code_section, &options, &source)
                    } else {
                        None
                    }
                }
                parser::Kind::Unit | parser::Kind::Program => {
                    if options.transformations.enable_unit_program_section {
                        transform_unit_program_section(code_section, &options, &source)
                    } else {
                        None
                    }
                }
                parser::Kind::Interface
                | parser::Kind::Implementation
                | parser::Kind::Initialization
                | parser::Kind::Finalization => {
                    if options.transformations.enable_single_keyword_sections {
                        transform_single_keyword_section(&source, code_section, &options)
                    } else {
                        None
                    }
                }
                parser::Kind::ProcedureDeclaration | parser::Kind::FunctionDeclaration => {
                    if options.transformations.enable_procedure_section {
                        transform_procedure_section(code_section, &options, &source)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    });

    // Apply text transformations if any are enabled
    if options.transformations.enable_text_transformations {
        replacements = timing.time_operation("Text transformations", || {
            // Fill gaps to get all text as replacements, then apply text transformations
            let all_replacements = fill_gaps_with_identity_replacements(&source, replacements);
            apply_text_transformations(&source, all_replacements, &options.text_changes)
        });
    }

    Ok((source, replacements))
}

fn run() -> Result<i32, DFixxerError> {
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
                    merge_replacements(&arguments.filename, &source, replacements)
                })?;
            }

            // Log the timing summary
            timing.log_summary();
            Ok(0)
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

            // Return the number of non-identity replacements as exit code
            let non_identity_count = replacements.iter().filter(|r| r.text.is_some()).count();
            Ok(non_identity_count as i32)
        }
        Command::InitConfig => {
            println!("Initializing configuration...");
            match Options::create_default_config(&arguments.filename) {
                Ok(()) => {
                    println!("Created default configuration file: {}", arguments.filename);
                    Ok(0)
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Command::Parse => {
            // Parse the file and print each node's kind and text using parse_raw
            let source = std::fs::read_to_string(&arguments.filename)?;
            parser::parse_raw(&source)?;
            Ok(0)
        }
        Command::ParseDebug => {
            // Parse the file and print the ParseResult structure
            let source = std::fs::read_to_string(&arguments.filename)?;
            let parse_result = parse(&source)?;
            println!("{:#?}", parse_result);
            Ok(0)
        }
    }
}

fn main() {
    // Parse arguments first to get log level
    let args: Vec<String> = std::env::args().collect();
    if let Ok(arguments) = parse_args(args.clone()) {
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
    match run() {
        Ok(exit_code) => {
            let total_duration = start_total.elapsed();
            log::info!("Total execution time: {:?}", total_duration);
            std::process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
