mod dfixxer_error;
use dfixxer_error::DFixxerError;
mod arguments;
use arguments::{Command, expand_filename_pattern, parse_args};
use diffy::create_patch;
mod options;
use options::{Options, find_custom_config_for_file, should_exclude_file};
mod replacements;
mod transform_control_statement_body_wrapping;
mod transform_inherited_calls;
mod transform_inline_local_var_definitions;
mod transform_local_routine_indentation;
mod transform_local_routine_spacing;
mod transform_procedure_section;
mod transform_single_keyword_sections;
mod transform_text;
mod transform_unit_program_section;
mod transform_uses_section;
mod transformer_utility;
use replacements::{TextReplacement, apply_replacements_to_string, compute_source_sections};
mod parser;
use parser::{
    ControlStatementBodyWrappingContext, ControlStatementKind, ParseContextTimings, parse,
    parse_with_contexts_and_timings,
};
mod suppression;

use crate::suppression::collect_suppression_context;
use crate::transform_control_statement_body_wrapping::transform_control_statement_body_wrapping;
use crate::transform_inherited_calls::transform_inherited_calls;
use crate::transform_inline_local_var_definitions::transform_inline_local_var_definitions;
use crate::transform_local_routine_indentation::transform_local_routine_indentation;
use crate::transform_local_routine_spacing::transform_local_routine_spacing;
use crate::transform_procedure_section::transform_procedure_section;
use crate::transform_single_keyword_sections::transform_single_keyword_section;
use crate::transform_unit_program_section::transform_unit_program_section;
use crate::transform_uses_section::transform_uses_section;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
struct RulePerformanceSummary {
    candidates: usize,
    replacements: usize,
    duration: Duration,
}

/// Collects top-level timings plus fine-grained parser, rule, and text-rule metrics.
struct PerformanceCollector {
    stage_timings: BTreeMap<String, Duration>,
    parse_timings: BTreeMap<String, Duration>,
    rule_timings: BTreeMap<String, RulePerformanceSummary>,
    text_stats: transform_text::TextTransformationStats,
}

impl PerformanceCollector {
    fn new() -> Self {
        Self {
            stage_timings: BTreeMap::new(),
            parse_timings: BTreeMap::new(),
            rule_timings: BTreeMap::new(),
            text_stats: transform_text::TextTransformationStats::default(),
        }
    }

    fn record_stage_duration(&mut self, operation_name: &str, duration: Duration) {
        self.stage_timings
            .entry(operation_name.to_string())
            .and_modify(|total| *total += duration)
            .or_insert(duration);
    }

    fn record_parse_timings(&mut self, parse_timings: &ParseContextTimings) {
        self.parse_timings
            .insert("build tree".to_string(), parse_timings.build_tree);
        self.parse_timings.insert(
            "collect code sections".to_string(),
            parse_timings.collect_code_sections,
        );
        self.parse_timings.insert(
            "collect spacing context".to_string(),
            parse_timings.collect_spacing_context,
        );
        self.parse_timings.insert(
            "collect inherited call expansion context".to_string(),
            parse_timings.collect_inherited_call_expansion_context,
        );
        self.parse_timings.insert(
            "collect local routine spacing context".to_string(),
            parse_timings.collect_local_routine_spacing_context,
        );
        self.parse_timings.insert(
            "collect control body wrapping context".to_string(),
            parse_timings.collect_control_statement_body_wrapping_context,
        );
        self.parse_timings.insert(
            "collect inline local var definition context".to_string(),
            parse_timings.collect_inline_local_var_definition_context,
        );
    }

    fn record_rule_timing(
        &mut self,
        rule_name: &str,
        candidates: usize,
        replacements: usize,
        duration: Duration,
    ) {
        if candidates == 0 && replacements == 0 {
            return;
        }
        let stats = self.rule_timings.entry(rule_name.to_string()).or_default();
        stats.candidates += candidates;
        stats.replacements += replacements;
        stats.duration += duration;
    }

    fn record_text_stats(&mut self, stats: transform_text::TextTransformationStats) {
        self.text_stats.merge(stats);
    }

    fn time_operation<T, F>(&mut self, operation_name: &str, operation: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        log::debug!("{} took: {:?}", operation_name, duration);
        self.record_stage_duration(operation_name, duration);
        result
    }

    fn time_operation_result<T, E, F>(&mut self, operation_name: &str, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        log::debug!("{} took: {:?}", operation_name, duration);
        self.record_stage_duration(operation_name, duration);
        result
    }

    fn log_summary(&self) {
        let total_processing: Duration = self.stage_timings.values().copied().sum();

        log::info!("Performance summary:");
        if !self.stage_timings.is_empty() {
            log::info!("  Stage timings:");
            for (operation, duration) in &self.stage_timings {
                log::info!("    {}: {:?}", operation, duration);
            }
        }
        if !self.parse_timings.is_empty() {
            log::info!("  Parse substage timings:");
            for (operation, duration) in &self.parse_timings {
                log::info!("    {}: {:?}", operation, duration);
            }
        }
        if !self.rule_timings.is_empty() {
            log::info!("  Rule timings:");
            for (rule_name, stats) in &self.rule_timings {
                log::info!(
                    "    {}: candidates={} replacements={} total={:?}",
                    rule_name,
                    stats.candidates,
                    stats.replacements,
                    stats.duration
                );
            }
        }
        if !self.text_stats.is_empty() {
            log::info!(
                "  Text transformation counters: sections={} changed_sections={} bytes={} skipped_error_ranges={} file_level_runs={} file_level_changes={}",
                self.text_stats.sections_processed,
                self.text_stats.sections_changed,
                self.text_stats.bytes_processed,
                self.text_stats.skipped_error_ranges,
                self.text_stats.file_level_runs,
                self.text_stats.file_level_changes
            );
            for (rule_name, stats) in self.text_stats.rule_stats() {
                log::info!(
                    "    {}: hits={} changes={} skips={}",
                    rule_name,
                    stats.hits,
                    stats.changes,
                    stats.skips
                );
            }
        }
        log::info!("  Total processing: {:?}", total_processing);
    }
}

fn filtered_control_statement_context<F>(
    context: &ControlStatementBodyWrappingContext,
    predicate: F,
) -> ControlStatementBodyWrappingContext
where
    F: Fn(&ControlStatementKind) -> bool,
{
    ControlStatementBodyWrappingContext {
        candidates: context
            .candidates
            .iter()
            .filter(|candidate| predicate(&candidate.kind))
            .cloned()
            .collect(),
    }
}

fn load_file(filename: &str) -> Result<String, DFixxerError> {
    Ok(std::fs::read_to_string(filename)?)
}

/// Process a file and return the replacements that would be made
fn process_file(
    filename: &str,
    config_path: Option<&str>,
    timing: &mut PerformanceCollector,
) -> Result<(String, String, usize), DFixxerError> {
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
    let suppression_context = timing.time_operation("Inline suppression scan", || {
        collect_suppression_context(&source)
    });
    for warning in &suppression_context.warnings {
        let message = format!("{}:{}: {}", filename, warning.line, warning.message());
        log::warn!("{}", message);
        eprintln!("Warning: {}", message);
    }

    // Time parsing
    let (
        parse_result,
        spacing_context,
        inherited_expansion_context,
        local_routine_spacing_context,
        control_statement_body_wrapping_context,
        inline_local_var_definition_context,
        parse_context_timings,
    ) = timing.time_operation_result("Parsing", || parse_with_contexts_and_timings(&source))?;
    timing.record_parse_timings(&parse_context_timings);
    if !spacing_context.error_ranges.is_empty() {
        let message = format!(
            "Parser recovered with {} error span(s) in '{}'; text changes are skipped inside error spans.",
            spacing_context.error_ranges.len(),
            filename
        );
        log::warn!("{}", message);
        eprintln!("Warning: {}", message);
    }

    // Helper function to apply text transformations to a replacement if enabled
    let mut text_stats = transform_text::TextTransformationStats::default();
    let apply_text_transformation_if_enabled =
        |replacement: TextReplacement,
         text_stats: &mut transform_text::TextTransformationStats|
         -> Option<TextReplacement> {
            if options.transformations.enable_text_transformations {
                let text = replacement.text.as_str();
                transform_text::apply_text_transformation_with_context_and_stats(
                    replacement.start,
                    replacement.end,
                    text,
                    &options.text_changes,
                    Some(&spacing_context),
                    text_stats,
                )
                .or(Some(replacement))
            } else {
                Some(replacement)
            }
        };

    let transformation_start = Instant::now();
    let mut replacements: Vec<TextReplacement> = Vec::new();

    if options.transformations.enable_uses_section {
        let uses_sections: Vec<_> = parse_result
            .code_sections
            .iter()
            .filter(|code_section| code_section.keyword.kind == parser::Kind::Uses)
            .collect();
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = uses_sections
            .iter()
            .filter_map(|code_section| transform_uses_section(code_section, &options, &source))
            .collect();
        timing.record_rule_timing(
            "uses_section",
            uses_sections.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_unit_program_section {
        let unit_program_sections: Vec<_> = parse_result
            .code_sections
            .iter()
            .filter(|code_section| {
                matches!(
                    code_section.keyword.kind,
                    parser::Kind::Unit | parser::Kind::Program
                )
            })
            .collect();
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = unit_program_sections
            .iter()
            .filter_map(|code_section| {
                transform_unit_program_section(code_section, &options, &source)
            })
            .collect();
        timing.record_rule_timing(
            "unit_program_section",
            unit_program_sections.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_single_keyword_sections {
        let single_keyword_sections: Vec<_> = parse_result
            .code_sections
            .iter()
            .filter(|code_section| {
                matches!(
                    code_section.keyword.kind,
                    parser::Kind::Interface
                        | parser::Kind::Implementation
                        | parser::Kind::Initialization
                        | parser::Kind::Finalization
                )
            })
            .collect();
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = single_keyword_sections
            .iter()
            .filter_map(|code_section| {
                transform_single_keyword_section(&source, code_section, &options)
            })
            .collect();
        timing.record_rule_timing(
            "single_keyword_sections",
            single_keyword_sections.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_procedure_section {
        let procedure_sections: Vec<_> = parse_result
            .code_sections
            .iter()
            .filter(|code_section| {
                matches!(
                    code_section.keyword.kind,
                    parser::Kind::ProcedureDeclaration | parser::Kind::FunctionDeclaration
                )
            })
            .collect();
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = procedure_sections
            .iter()
            .filter_map(|code_section| transform_procedure_section(code_section, &options, &source))
            .filter_map(|replacement| {
                apply_text_transformation_if_enabled(replacement, &mut text_stats)
            })
            .collect();
        timing.record_rule_timing(
            "procedure_section",
            procedure_sections.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_inherited_call_expansion {
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = transform_inherited_calls(&inherited_expansion_context)
            .into_iter()
            .filter_map(|replacement| {
                apply_text_transformation_if_enabled(replacement, &mut text_stats)
            })
            .collect();
        timing.record_rule_timing(
            "inherited_call_expansion",
            inherited_expansion_context.candidates.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_local_routine_indentation {
        let rule_start = Instant::now();
        let rule_replacements =
            transform_local_routine_indentation(&source, &local_routine_spacing_context, &options);
        timing.record_rule_timing(
            "local_routine_indentation",
            local_routine_spacing_context.blocks.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_local_routine_spacing {
        let rule_start = Instant::now();
        let rule_replacements =
            transform_local_routine_spacing(&source, &local_routine_spacing_context, &options);
        timing.record_rule_timing(
            "local_routine_spacing",
            local_routine_spacing_context.gaps.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_inline_local_var_definitions {
        let rule_start = Instant::now();
        let rule_replacements: Vec<_> = transform_inline_local_var_definitions(
            &source,
            &inline_local_var_definition_context,
            &options,
        )
        .into_iter()
        .filter_map(|replacement| {
            apply_text_transformation_if_enabled(replacement, &mut text_stats)
        })
        .collect();
        timing.record_rule_timing(
            "inline_local_var_definitions",
            inline_local_var_definition_context.routines.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_for_body_wrapping {
        let for_context =
            filtered_control_statement_context(&control_statement_body_wrapping_context, |kind| {
                matches!(
                    kind,
                    ControlStatementKind::For | ControlStatementKind::Foreach
                )
            });
        let rule_start = Instant::now();
        let rule_replacements =
            transform_control_statement_body_wrapping(&source, &for_context, &options);
        timing.record_rule_timing(
            "control_body_wrapping.for_foreach",
            for_context.candidates.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_while_body_wrapping {
        let while_context =
            filtered_control_statement_context(&control_statement_body_wrapping_context, |kind| {
                matches!(kind, ControlStatementKind::While)
            });
        let rule_start = Instant::now();
        let rule_replacements =
            transform_control_statement_body_wrapping(&source, &while_context, &options);
        timing.record_rule_timing(
            "control_body_wrapping.while",
            while_context.candidates.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    if options.transformations.enable_if_body_wrapping {
        let if_else_context =
            filtered_control_statement_context(&control_statement_body_wrapping_context, |kind| {
                matches!(
                    kind,
                    ControlStatementKind::IfThen | ControlStatementKind::Else
                )
            });
        let rule_start = Instant::now();
        let rule_replacements =
            transform_control_statement_body_wrapping(&source, &if_else_context, &options);
        timing.record_rule_timing(
            "control_body_wrapping.if_else",
            if_else_context.candidates.len(),
            rule_replacements.len(),
            rule_start.elapsed(),
        );
        replacements.extend(rule_replacements);
    }

    timing.record_stage_duration("Transformation", transformation_start.elapsed());
    replacements.retain(|replacement| {
        !suppression_context.suppresses_replacement(replacement.start, replacement.end)
    });

    // Apply text transformations if enabled
    if options.transformations.enable_text_transformations {
        timing.time_operation("Text transformations", || {
            // Calculate sections (gaps + existing replacements)
            let sections = compute_source_sections(
                &source,
                &replacements,
                &suppression_context.text_exclusion_ranges(),
            );

            // Apply text transformation to each section and add to replacements if there's a change
            for section in sections {
                let text = &source[section.start..section.end];
                if let Some(transformation) =
                    transform_text::apply_text_transformation_with_context_and_stats(
                        section.start,
                        section.end,
                        text,
                        &options.text_changes,
                        Some(&spacing_context),
                        &mut text_stats,
                    )
                {
                    replacements.push(transformation);
                }
            }
        });
    }
    replacements.retain(|replacement| {
        !suppression_context.suppresses_replacement(replacement.start, replacement.end)
    });

    let mut replacement_count = replacements.len();
    let mut updated_source = if replacements.is_empty() {
        source.clone()
    } else {
        timing.time_operation("Applying replacements (in-memory)", || {
            apply_replacements_to_string(&source, &replacements)
        })
    };

    if options.transformations.enable_text_transformations
        && let Some(file_level_update) =
            timing.time_operation("File-level text transformations", || {
                transform_text::apply_file_level_text_changes_with_stats(
                    &updated_source,
                    &options.text_changes,
                    &options.line_ending,
                    &mut text_stats,
                )
            })
    {
        updated_source = file_level_update;
        replacement_count += 1;
    }

    timing.record_text_stats(text_stats);

    Ok((source, updated_source, replacement_count))
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
                let mut timing = PerformanceCollector::new();

                let (source, updated_source, _) =
                    process_file(filename, arguments.config_path.as_deref(), &mut timing)?;

                if source != updated_source {
                    timing.time_operation_result("Writing updated file", || {
                        std::fs::write(filename, &updated_source).map_err(DFixxerError::from)
                    })?;
                }

                // Log the timing summary
                timing.log_summary();
                0
            }
            Command::CheckFile => {
                let mut timing = PerformanceCollector::new();

                let (source, updated_source, replacement_count) =
                    process_file(filename, arguments.config_path.as_deref(), &mut timing)?;

                if source != updated_source {
                    let patch = timing.time_operation("Diff generation", || {
                        create_patch(&source, &updated_source)
                    });
                    println!("{}", patch);
                }

                // Log the timing summary
                timing.log_summary();

                // Return the number of replacements as exit code
                replacement_count as i32
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
