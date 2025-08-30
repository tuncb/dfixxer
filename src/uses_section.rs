use crate::options::Options;
use crate::{dfixxer_error::DFixxerError, replacements::TextReplacement};
use tree_sitter::{Node, Tree};

#[derive(Debug)]
pub enum UsesSection<'a> {
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

pub fn find_kuses_nodes<'a>(tree: &'a Tree, _source: &str) -> Vec<Node<'a>> {
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

pub fn transform_uses_section<'a>(
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

// Formats the replacement text for a uses section given the modules and options.
fn format_uses_replacement(modules: &Vec<String>, options: &Options) -> String {
    use crate::options::UsesSectionStyle;
    match options.uses_section_style {
        UsesSectionStyle::CommaAtTheBeginning => {
            let mut lines = Vec::new();
            if let Some(first) = modules.get(0) {
                lines.push(format!("{}{}", options.indentation, first));
                for module in modules.iter().skip(1) {
                    lines.push(format!("{}, {}", options.indentation, module));
                }
            }
            lines.push(format!("{};", options.indentation));
            format!("uses\n{}", lines.join("\n"))
        }
        _ => {
            let modules_text = modules.join(&format!(",\n{}", options.indentation));
            format!("uses\n{}{};", options.indentation, modules_text)
        }
    }
}

pub fn transform_to_replacement(
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

            let mut sorted_modules = modules.clone();
            sorted_modules.sort();

            let replacement_text = format_uses_replacement(&sorted_modules, options);
            Some(TextReplacement {
                start,
                end,
                text: replacement_text,
            })
        }
        _ => None, // Only handle parsed sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{Options, UsesSectionStyle};

    fn make_options(style: UsesSectionStyle, indentation: &str) -> Options {
        Options {
            uses_section_style: style,
            indentation: indentation.to_string(),
            // ...other fields with default values...
            ..Default::default()
        }
    }

    #[test]
    fn test_format_uses_replacement_comma_at_the_beginning() {
        let modules = vec![
            "UnitA".to_string(),
            "UnitB".to_string(),
            "UnitC".to_string(),
        ];
        let options = make_options(UsesSectionStyle::CommaAtTheBeginning, "    ");
        let expected = "uses\n    UnitA\n    , UnitB\n    , UnitC\n    ;";
        let result = format_uses_replacement(&modules, &options);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_uses_replacement_comma_at_the_end() {
        let modules = vec![
            "UnitA".to_string(),
            "UnitB".to_string(),
            "UnitC".to_string(),
        ];
        let options = make_options(UsesSectionStyle::CommaAtTheEnd, "    ");
        let expected = "uses\n    UnitA,\n    UnitB,\n    UnitC;";
        let result = format_uses_replacement(&modules, &options);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_uses_replacement_empty_modules() {
        let modules: Vec<String> = vec![];
        let options = make_options(UsesSectionStyle::CommaAtTheBeginning, "  ");
        let expected = "uses\n  ;";
        let result = format_uses_replacement(&modules, &options);
        assert_eq!(result, expected);
    }
}
