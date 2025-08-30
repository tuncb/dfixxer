mod dfixxer_error;
use dfixxer_error::DFixxerError;
mod arguments;
use arguments::{Command, parse_args};
mod options;
use options::Options;
mod replacements;
mod uses_section;
use replacements::{TextReplacement, apply_replacements};
mod parser;
use parser::parse;

use crate::uses_section::transform_parser_uses_section_to_replacement;

fn load_file(filename: &str) -> Result<String, DFixxerError> {
    Ok(std::fs::read_to_string(filename)?)
}

fn run() -> Result<(), DFixxerError> {
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_args(args)?;

    match arguments.command {
        Command::UpdateFile => {
            // Load options from config file, or use defaults if not found
            let config_path = arguments.config_path.as_deref().unwrap_or("dfixxer.toml");
            let options: Options = Options::load_or_default(config_path);

            let source = load_file(&arguments.filename)?;
            let parse_result = parse(&source)?;

            // Convert uses sections to text replacements, skipping ones with comments/preprocessor
            let replacements: Vec<TextReplacement> = parse_result
                .uses_sections
                .iter()
                .filter_map(|uses_section| {
                    transform_parser_uses_section_to_replacement(uses_section, &options, &source)
                })
                .collect();

            // Apply replacements to the original file
            if !replacements.is_empty() {
                apply_replacements(&arguments.filename, &source, replacements)?;
            }
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
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
