use crate::options::Options;
use crate::parser::{Kind, UsesSection as ParserUsesSection};
use crate::replacements::TextReplacement;

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

fn sort_modules(modules: &Vec<String>, options: &Options) -> Vec<String> {
    let mut modules = modules.clone();

    // Apply modules_names_to_update: e.g. "System:Classes" means replace "Classes" with "System.Classes"
    for mapping in &options.modules_names_to_update {
        if let Some((prefix, name)) = mapping.split_once(':') {
            for module in modules.iter_mut() {
                if module == name {
                    *module = format!("{}.{}", prefix, name);
                }
            }
        }
    }

    let override_namespaces = &options.override_sorting_order;
    if override_namespaces.is_empty() {
        modules.sort();
        return modules;
    }

    // Partition modules into those that start with any override namespace and have a '.' after the namespace, and the rest
    let mut prioritized = Vec::new();
    let mut rest = Vec::new();
    for m in modules {
        let mut is_prioritized = false;
        for ns in override_namespaces {
            if m.starts_with(ns) {
                let ns_len = ns.len();
                if m.len() > ns_len && m.chars().nth(ns_len) == Some('.') {
                    is_prioritized = true;
                    break;
                }
            }
        }
        if is_prioritized {
            prioritized.push(m);
        } else {
            rest.push(m);
        }
    }
    prioritized.sort();
    rest.sort();
    prioritized.into_iter().chain(rest.into_iter()).collect()
}

/// Transform a parser::UsesSection to TextReplacement
/// Skips uses sections that contain comments or preprocessor nodes
pub fn transform_parser_uses_section_to_replacement(
    uses_section: &ParserUsesSection,
    options: &Options,
    source: &str,
) -> Option<TextReplacement> {
    // Check if any sibling contains comments or preprocessor nodes
    for sibling in &uses_section.siblings {
        match sibling.kind {
            Kind::Comment | Kind::Preprocessor => {
                // Skip this uses section if it contains comments or preprocessor directives
                return None;
            }
            _ => continue,
        }
    }

    // Extract module names from siblings
    let mut modules = Vec::new();
    for sibling in &uses_section.siblings {
        if matches!(sibling.kind, Kind::Module) {
            // Extract the module text from the source using byte positions
            let module_text = &source[sibling.start_byte..sibling.end_byte];
            modules.push(module_text.to_string());
        }
    }

    // Sort modules according to options
    let sorted_modules = sort_modules(&modules, options);

    // Format the replacement text
    let replacement_text = format_uses_replacement(&sorted_modules, options);

    // Create the text replacement
    Some(TextReplacement {
        start: uses_section.uses.start_byte,
        end: uses_section.semicolon.end_byte,
        text: replacement_text,
    })
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

    #[test]
    fn test_sort_modules_with_override_namespaces() {
        let modules = vec![
            "A".to_string(),
            "B".to_string(),
            "System.A".to_string(),
            "Abc.B".to_string(),
            "SystemA".to_string(),
            "AbcB".to_string(),
        ];
        let mut options = make_options(UsesSectionStyle::CommaAtTheBeginning, "    ");
        options.override_sorting_order = vec!["System".to_string(), "Abc".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["Abc.B", "System.A", "A", "AbcB", "B", "SystemA"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_without_override_namespaces() {
        let modules = vec!["B".to_string(), "A".to_string(), "C".to_string()];
        let mut options = make_options(UsesSectionStyle::CommaAtTheBeginning, "    ");
        options.override_sorting_order = vec![];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["A", "B", "C"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_with_dot_but_not_namespace() {
        let modules = vec![
            "X.Y".to_string(),
            "A.B".to_string(),
            "SystemA.B".to_string(),
        ];
        let mut options = make_options(UsesSectionStyle::CommaAtTheBeginning, "    ");
        options.override_sorting_order = vec!["System".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["A.B", "SystemA.B", "X.Y"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }
}
