use crate::options::Options;
use crate::parser::{Kind, UsesSection as ParserUsesSection};
use crate::replacements::TextReplacement;
use log::warn;

// Formats the replacement text for a uses section given the modules and options.
fn format_uses_replacement(modules: &Vec<String>, options: &Options) -> String {
    use crate::options::UsesSectionStyle;
    match options.uses_section_style {
        UsesSectionStyle::CommaAtTheBeginning => {
            let mut lines = Vec::new();
            if let Some(first) = modules.get(0) {
                // First unit: {indentation}{two spaces}{unit}
                lines.push(format!("{}  {}", options.indentation, first));
                // Following units: {indentation}, {unit}
                for module in modules.iter().skip(1) {
                    lines.push(format!("{}, {}", options.indentation, module));
                }
            }
            lines.push(format!("{};", options.indentation));
            format!(
                "uses{}{}",
                options.line_ending.to_string(),
                lines.join(&options.line_ending.to_string())
            )
        }
        _ => {
            let modules_text = modules.join(&format!(
                ",{}{}",
                options.line_ending.to_string(),
                options.indentation
            ));
            format!(
                "uses{}{}{};",
                options.line_ending.to_string(),
                options.indentation,
                modules_text
            )
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
        modules.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
    prioritized.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    rest.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
                warn!(
                    "Skipping uses section at byte range {}-{} due to presence of {} node",
                    uses_section.uses.start_byte,
                    uses_section.semicolon.end_byte,
                    match sibling.kind {
                        Kind::Comment => "comment",
                        Kind::Preprocessor => "preprocessor",
                        _ => "unknown",
                    }
                );
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
    let mut replacement_text = format_uses_replacement(&sorted_modules, options);

    // Determine the actual start position for replacement
    let mut replacement_start = uses_section.uses.start_byte;

    // Find the beginning of the line containing the uses section
    let line_start = find_line_start(source, uses_section.uses.start_byte);

    // Check what's between line start and uses section start
    let prefix = &source[line_start..uses_section.uses.start_byte];

    if prefix
        .chars()
        .all(|c| c.is_whitespace() && c != '\n' && c != '\r')
    {
        // Only whitespace characters before uses - remove them by extending replacement start
        replacement_start = line_start;
    } else if !prefix.is_empty() {
        // Non-whitespace characters before uses - add a newline before the uses section
        replacement_text = format!("{}{}", options.line_ending.to_string(), replacement_text);
    }
    // If prefix is empty, uses is already at start of line, no adjustment needed

    // Create the text replacement
    Some(TextReplacement {
        start: replacement_start,
        end: uses_section.semicolon.end_byte,
        text: replacement_text,
    })
}

/// Find the start of the line containing the given byte position
fn find_line_start(source: &str, position: usize) -> usize {
    if position == 0 {
        return 0;
    }

    // Search backwards from position to find the start of the line
    let bytes = source.as_bytes();
    for i in (0..position).rev() {
        if bytes[i] == b'\n' {
            return i + 1; // Return position after the newline
        }
    }
    0 // Beginning of file
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{Options, UsesSectionStyle};

    fn make_options(
        style: UsesSectionStyle,
        indentation: &str,
        line_ending: crate::options::LineEnding,
    ) -> Options {
        Options {
            uses_section_style: style,
            indentation: indentation.to_string(),
            line_ending,
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
        let options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "  ",
            crate::options::LineEnding::Crlf,
        );
        // With the new style, the first unit has two extra spaces beyond indentation
        let expected = "uses\r\n    UnitA\r\n  , UnitB\r\n  , UnitC\r\n  ;";
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
        let options = make_options(
            UsesSectionStyle::CommaAtTheEnd,
            "    ",
            crate::options::LineEnding::Crlf,
        );
        let expected = "uses\r\n    UnitA,\r\n    UnitB,\r\n    UnitC;";
        let result = format_uses_replacement(&modules, &options);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_uses_replacement_empty_modules() {
        let modules: Vec<String> = vec![];
        let options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "  ",
            crate::options::LineEnding::Crlf,
        );
        let expected = "uses\r\n  ;";
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
        let mut options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "    ",
            crate::options::LineEnding::Crlf,
        );
        options.override_sorting_order = vec!["System".to_string(), "Abc".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["Abc.B", "System.A", "A", "AbcB", "B", "SystemA"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_without_override_namespaces() {
        let modules = vec!["B".to_string(), "A".to_string(), "C".to_string()];
        let mut options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "    ",
            crate::options::LineEnding::Crlf,
        );
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
        let mut options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "    ",
            crate::options::LineEnding::Crlf,
        );
        options.override_sorting_order = vec!["System".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["A.B", "SystemA.B", "X.Y"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_format_uses_replacement_with_custom_line_ending() {
        let modules = vec!["UnitA".to_string(), "UnitB".to_string()];
        let options = make_options(
            UsesSectionStyle::CommaAtTheEnd,
            "  ",
            crate::options::LineEnding::Lf,
        );
        let expected = "uses\n  UnitA,\n  UnitB;";
        let result = format_uses_replacement(&modules, &options);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_line_start() {
        let source = "line1\nline2\nline3";
        assert_eq!(find_line_start(source, 0), 0); // Beginning of file
        assert_eq!(find_line_start(source, 3), 0); // Middle of first line
        assert_eq!(find_line_start(source, 6), 6); // Beginning of second line
        assert_eq!(find_line_start(source, 9), 6); // Middle of second line
        assert_eq!(find_line_start(source, 12), 12); // Beginning of third line
    }

    #[test]
    fn test_find_line_start_single_line() {
        let source = "single line";
        assert_eq!(find_line_start(source, 0), 0);
        assert_eq!(find_line_start(source, 5), 0);
        assert_eq!(find_line_start(source, 10), 0);
    }
}
