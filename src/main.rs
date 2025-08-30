mod dfixxer_error;
use dfixxer_error::DFixxerError;
use tree_sitter::{Parser, Tree};
use tree_sitter_pascal::LANGUAGE;
mod arguments;
use arguments::{Command, parse_args};
mod options;
use options::Options;
mod uses_section;
use uses_section::find_kuses_nodes;
mod replacements;
use replacements::{TextReplacement, apply_replacements};

use crate::uses_section::{UsesSection, transform_to_replacement, transform_uses_section};

fn load_file(filename: &str) -> Result<String, DFixxerError> {
    Ok(std::fs::read_to_string(filename)?)
}

fn parse_to_tree(source: &str) -> Result<Tree, DFixxerError> {
    let mut parser = Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .map_err(|_| DFixxerError::ParseError("Failed to set language".to_string()))?;
    parser
        .parse(source, None)
        .ok_or_else(|| DFixxerError::ParseError("Failed to parse source".to_string()))
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
            let tree = parse_to_tree(&source)?;

            let kuses_nodes = find_kuses_nodes(&tree, &source);

            let uses_sections: Vec<UsesSection> = kuses_nodes
                .into_iter()
                .map(|node| transform_uses_section(node, &source))
                .collect::<Result<Vec<_>, _>>()?;

            // Print warnings for error cases and filter out UsesSectionParsed sections
            let replacements: Vec<TextReplacement> = uses_sections
                .iter()
                .filter_map(|section| match section {
                    UsesSection::UsesSectionWithError { node } => {
                        println!(
                            "Uses section with grammar error found at line {}. Ignoring the section.",
                            node.start_position().row + 1
                        );
                        None
                    }
                    UsesSection::UsesSectionWithUnsupportedComment { node } => {
                        println!(
                            "Uses section with unsupported comment found at line {}. Ignoring the section.",
                            node.start_position().row + 1
                        );
                        None
                    }
                    UsesSection::UsesSectionParsed { .. } => transform_to_replacement(section, &options),
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
