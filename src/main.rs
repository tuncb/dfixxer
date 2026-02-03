mod dfixxer_error;
use dfixxer_error::DFixxerError;
mod arguments;
use arguments::{Command, expand_filename_pattern, parse_args};
mod options;
use options::{Options, find_custom_config_for_file, should_exclude_file};
mod replacements;
mod transform_procedure_section;
mod transform_single_keyword_sections;
mod transform_text;
mod transform_unit_program_section;
mod transform_uses_section;
mod transformer_utility;
use replacements::{
    TextReplacement, compute_source_sections, merge_replacements, print_replacements,
};
mod parser;
use parser::{parse, parse_with_spacing_context};

use crate::transform_procedure_section::transform_procedure_section;
use crate::transform_single_keyword_sections::transform_single_keyword_section;
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
    let initial_options: Options = Options::load_or_default(config_path);

    // Check if there's a custom config for this specific file
    let final_config_path = find_custom_config_for_file(
        &initial_options.custom_config_patterns,
        filename,
        Some(config_path),
    )
    .unwrap_or_else(|| config_path.to_string());

    let options: Options = if final_config_path != config_path {
        log::info!("Loading custom configuration from: {}", final_config_path);
        Options::load_or_default(&final_config_path)
    } else {
        initial_options
    };

    // Time file loading
    let source = timing.time_operation_result("File loading", || load_file(filename))?;

    // Time parsing
    let (parse_result, spacing_context) =
        timing.time_operation_result("Parsing", || parse_with_spacing_context(&source))?;

    // Helper function to apply text transformations to a replacement if enabled
    let apply_text_transformation_if_enabled =
        |replacement: TextReplacement| -> Option<TextReplacement> {
            if options.transformations.enable_text_transformations {
                let text = replacement.text.as_str();
                transform_text::apply_text_transformation_with_context(
                    replacement.start,
                    replacement.end,
                    text,
                    &options.text_changes,
                    Some(&spacing_context),
                )
                .or(Some(replacement)) // Return original if no changes needed
            } else {
                Some(replacement)
            }
        };

    // Time transformation
    let mut replacements: Vec<TextReplacement> = timing.time_operation("Transformation", || {
        parse_result
            .code_sections
            .iter()
            .filter_map(|code_section| {
                let transformation = match code_section.keyword.kind {
                    parser::Kind::Uses if options.transformations.enable_uses_section => {
                        transform_uses_section(code_section, &options, &source)
                    }
                    parser::Kind::Unit | parser::Kind::Program
                        if options.transformations.enable_unit_program_section =>
                    {
                        transform_unit_program_section(code_section, &options, &source)
                    }
                    parser::Kind::Interface
                    | parser::Kind::Implementation
                    | parser::Kind::Initialization
                    | parser::Kind::Finalization
                        if options.transformations.enable_single_keyword_sections =>
                    {
                        transform_single_keyword_section(&source, code_section, &options)
                    }
                    parser::Kind::ProcedureDeclaration | parser::Kind::FunctionDeclaration
                        if options.transformations.enable_procedure_section =>
                    {
                        transform_procedure_section(code_section, &options, &source)
                            .and_then(apply_text_transformation_if_enabled)
                    }
                    _ => None,
                };
                transformation
            })
            .collect()
    });

    // Apply text transformations if enabled
    if options.transformations.enable_text_transformations {
        timing.time_operation("Text transformations", || {
            // Calculate sections (gaps + existing replacements)
            let sections = compute_source_sections(&source, &replacements);

            // Apply text transformation to each section and add to replacements if there's a change
            for section in sections {
                let text = &source[section.start..section.end];
                if let Some(transformation) = transform_text::apply_text_transformation_with_context(
                    section.start,
                    section.end,
                    text,
                    &options.text_changes,
                    Some(&spacing_context),
                ) {
                    replacements.push(transformation);
                }
            }
        });
    }

    Ok((source, replacements))
}

fn run() -> Result<i32, DFixxerError> {
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_args(args)?;

    // Handle version command immediately
    if matches!(arguments.command, Command::Version) {
        println!("dfixxer {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }

    // Expand filename pattern if multi flag is set, but only for commands that support it
    let filenames = match &arguments.command {
        Command::UpdateFile | Command::CheckFile | Command::Parse | Command::ParseDebug => {
            expand_filename_pattern(&arguments.filename, arguments.multi)?
        }
        Command::InitConfig => {
            // InitConfig doesn't use multi mode
            vec![arguments.filename.clone()]
        }
        Command::Version => {
            // Version doesn't need filenames, but this is unreachable due to early return
            vec![]
        }
    };

    // For commands that process files, check if files should be excluded
    let filtered_filenames: Vec<String> = match &arguments.command {
        Command::UpdateFile | Command::CheckFile => {
            // Load options to check exclusion patterns
            let config_path = arguments.config_path.as_deref().unwrap_or("dfixxer.toml");
            let options = Options::load_or_default(config_path);

            // Filter out excluded files
            filenames
                .into_iter()
                .filter(|filename| {
                    if should_exclude_file(&options.exclude_files, filename, Some(config_path)) {
                        log::info!("File '{}' is excluded by configuration, skipping", filename);
                        false
                    } else {
                        true
                    }
                })
                .collect()
        }
        _ => filenames,
    };

    if filtered_filenames.is_empty() {
        if arguments.multi {
            log::info!("No files to process after filtering");
        }
        return Ok(0);
    }

    let mut total_exit_code = 0i32;

    // Process each file
    for filename in &filtered_filenames {
        // For multi mode, show filename for check, parse, parse-debug commands
        if arguments.multi {
            match &arguments.command {
                Command::CheckFile | Command::Parse | Command::ParseDebug => {
                    let absolute_path =
                        std::fs::canonicalize(filename).unwrap_or_else(|_| filename.into());
                    println!("Processing file: {}", absolute_path.display());
                }
                Command::UpdateFile => {
                    log::info!("Processing file: {}", filename);
                }
                _ => {}
            }
        }

        let exit_code = match arguments.command {
            Command::UpdateFile => {
                let mut timing = TimingCollector::new();

                let (source, replacements) =
                    process_file(filename, arguments.config_path.as_deref(), &mut timing)?;

                // Time applying replacements
                if !replacements.is_empty() {
                    timing.time_operation_result("Applying replacements", || {
                        merge_replacements(filename, &source, replacements)
                    })?;
                }

                // Log the timing summary
                timing.log_summary();
                0
            }
            Command::CheckFile => {
                let mut timing = TimingCollector::new();

                let (source, replacements) =
                    process_file(filename, arguments.config_path.as_deref(), &mut timing)?;

                // Print replacements instead of applying them
                print_replacements(&source, &replacements);

                // Log the timing summary
                timing.log_summary();

                // Return the number of replacements as exit code
                replacements.len() as i32
            }
            Command::InitConfig => {
                // InitConfig doesn't use multi mode, so just process first file
                if filename == &filtered_filenames[0] {
                    println!("Initializing configuration...");
                    match Options::create_default_config(filename) {
                        Ok(()) => {
                            println!("Created default configuration file: {}", filename);
                            0
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else {
                    0 // Skip other files for init-config
                }
            }
            Command::Parse => {
                // Parse the file and print each node's kind and text using parse_raw
                let source = std::fs::read_to_string(filename)?;
                parser::parse_raw(&source)?;
                0
            }
            Command::ParseDebug => {
                // Parse the file and print the ParseResult structure
                let source = std::fs::read_to_string(filename)?;
                let parse_result = parse(&source)?;
                println!("{:#?}", parse_result);
                0
            }
            Command::Version => {
                // This is unreachable due to early return above, but included for completeness
                0
            }
        };

        total_exit_code += exit_code;
    }

    Ok(total_exit_code)
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
