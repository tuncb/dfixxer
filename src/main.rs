mod dfixxer_error;
use dfixxer_error::DFixxerError;
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;
mod arguments;
use arguments::{Command, parse_args};
mod options;
use options::Options;

#[derive(Debug)]
enum UsesSection<'a> {
    UsesSectionWithError {
        node: Node<'a>,
    },
    UsesSectionWithUnsupportedComment {
        node: Node<'a>,
    },
    UsesSectionParsed {
        node: Node<'a>,
        modules: Vec<String>,
        k_semicolon: Node<'a>,
    },
}

#[derive(Debug)]
struct TextReplacement {
    start: usize,
    end: usize,
    text: String,
}

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

fn find_kuses_nodes<'a>(tree: &'a Tree, _source: &str) -> Vec<Node<'a>> {
    fn traverse<'b>(node: Node<'b>, nodes: &mut Vec<Node<'b>>) {
        if node.kind() == "kUses" {
            nodes.push(node);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                traverse(child, nodes);
            }
        }
    }
    let mut nodes = Vec::new();
    traverse(tree.root_node(), &mut nodes);
    nodes
}

fn transform_uses_section<'a>(
    kuses_node: Node<'a>,
    source: &str,
) -> Result<UsesSection<'a>, DFixxerError> {
    // Check if the starting node has an error
    if kuses_node.has_error() {
        return Ok(UsesSection::UsesSectionWithError { node: kuses_node });
    }

    let mut modules = Vec::new();
    let mut section_end_node = None;

    // Get the parent node (should be declUses)
    let parent = match kuses_node.parent() {
        Some(p) => p,
        None => return Ok(UsesSection::UsesSectionWithError { node: kuses_node }),
    };

    // Check parent for errors
    if parent.has_error() {
        return Ok(UsesSection::UsesSectionWithError { node: kuses_node });
    }

    // Examine all children of the parent (siblings of kuses_node)
    for i in 0..parent.child_count() {
        if let Some(child) = parent.child(i) {
            // Check each sibling for errors
            if child.has_error() {
                return Ok(UsesSection::UsesSectionWithError { node: kuses_node });
            }

            // Check if any sibling is pp (preprocessor) or comment
            if child.kind() == "pp" || child.kind() == "comment" {
                return Ok(UsesSection::UsesSectionWithUnsupportedComment { node: kuses_node });
            }

            // Look for the section terminator (could be semicolon or kEnd)
            if child.kind() == ";" || child.kind() == "kEnd" {
                section_end_node = Some(child);
            }

            // Extract module names
            if child.kind() == "moduleName" || child.kind() == "identifier" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    modules.push(text.to_string());
                }
            }
        }
    }

    // Return parsed section if we found a terminator
    if let Some(end_node) = section_end_node {
        Ok(UsesSection::UsesSectionParsed {
            node: kuses_node,
            modules,
            k_semicolon: end_node,
        })
    } else {
        // No terminator found - treat as error
        Ok(UsesSection::UsesSectionWithError { node: kuses_node })
    }
}

fn transform_to_replacement(
    uses_section: &UsesSection,
    options: &Options,
) -> Option<TextReplacement> {
    match uses_section {
        UsesSection::UsesSectionParsed {
            node,
            modules,
            k_semicolon,
        } => {
            let start = node.start_byte();
            let end = k_semicolon.end_byte();

            // Sort modules alphabetically
            let mut sorted_modules = modules.clone();
            sorted_modules.sort();

            // Create the replacement text with proper formatting using configured indentation
            let modules_text = sorted_modules.join(&format!(",\n{}", options.indentation));
            let replacement_text = format!("uses\n{}{};", options.indentation, modules_text);

            Some(TextReplacement {
                start,
                end,
                text: replacement_text,
            })
        }
        _ => None, // Only handle parsed sections
    }
}

fn apply_replacements(
    filename: &str,
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Result<(), DFixxerError> {
    if replacements.is_empty() {
        return Ok(());
    }

    // Sort replacements by start position in reverse order
    // This allows us to apply them from end to beginning to avoid offset issues
    replacements.sort_by(|a, b| b.start.cmp(&a.start));

    let mut modified_source = original_source.to_string();

    // Apply each replacement
    for replacement in replacements {
        // Ensure the replacement range is valid
        if replacement.start <= replacement.end && replacement.end <= modified_source.len() {
            modified_source.replace_range(replacement.start..replacement.end, &replacement.text);
        }
    }

    // Write the modified source back to the file
    std::fs::write(filename, modified_source)?;

    Ok(())
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
