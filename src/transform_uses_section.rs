use crate::options::Options;
use crate::parser::{CodeSection, Kind};
use crate::replacements::TextReplacement;
use crate::transformer_utility::{
    adjust_replacement_for_line_position, create_text_replacement_if_different,
};
use icu_collator::CollatorBorrowed;
use icu_collator::options::{CollatorOptions, Strength};
use log::warn;
use std::cmp::Ordering;

// Formats the replacement text for a uses section given the modules and options.
fn format_uses_replacement(modules: &[String], options: &Options) -> String {
    use crate::options::UsesSectionStyle;
    let line_ending = options.line_ending.to_string();
    match options.uses_section.uses_section_style {
        UsesSectionStyle::CommaAtTheBeginning => {
            let mut lines = Vec::new();
            if let Some(first) = modules.first() {
                // First unit: {indentation}{two spaces}{unit}
                lines.push(format!("{}  {}", options.indentation, first));
                // Following units: {indentation}, {unit}
                for module in modules.iter().skip(1) {
                    lines.push(format!("{}, {}", options.indentation, module));
                }
            }
            lines.push(format!("{};", options.indentation));
            let joined_lines = lines.join(&line_ending);
            format!("uses{}{}", line_ending, joined_lines)
        }
        UsesSectionStyle::CommaAtTheEnd => {
            let separator = format!(",{}{}", line_ending, options.indentation);
            let modules_text = modules.join(&separator);
            format!(
                "uses{}{}{};",
                line_ending, options.indentation, modules_text
            )
        }
    }
}

fn build_base_collator() -> Option<CollatorBorrowed<'static>> {
    let mut options = CollatorOptions::default();
    options.strength = Some(Strength::Primary);
    CollatorBorrowed::try_new(Default::default(), options).ok()
}

fn fallback_module_compare(
    a: &str,
    b: &str,
    collator: Option<&CollatorBorrowed<'static>>,
) -> Ordering {
    if let Some(collator) = collator {
        return collator.compare(a, b);
    }

    a.to_lowercase().cmp(&b.to_lowercase())
}

fn sort_modules(modules: &[String], options: &Options) -> Vec<String> {
    let mut modules = modules.to_owned();

    // Apply module_names_to_update: e.g. "System:Classes" means replace "Classes" with "System.Classes"
    for mapping in &options.uses_section.module_names_to_update {
        if let Some((prefix, name)) = mapping.split_once(':') {
            for module in modules.iter_mut() {
                if module == name {
                    *module = format!("{}.{}", prefix, name);
                }
            }
        }
    }

    // Match pascal-uses-formatter behavior:
    // - override prefixes are applied in configured order
    // - prefix matching is case-insensitive and does not require a dot boundary
    // - fallback ordering uses locale-style base collation
    let override_namespaces: Vec<String> = options
        .uses_section
        .override_sorting_order
        .iter()
        .map(|ns| ns.to_lowercase())
        .collect();
    let collator = build_base_collator();

    modules.sort_by(|a, b| {
        let normalized_a = a.trim().to_lowercase();
        let normalized_b = b.trim().to_lowercase();

        for ns in &override_namespaces {
            let a_matches = normalized_a.starts_with(ns);
            let b_matches = normalized_b.starts_with(ns);

            if a_matches && !b_matches {
                return Ordering::Less;
            }
            if !a_matches && b_matches {
                return Ordering::Greater;
            }
        }

        fallback_module_compare(a, b, collator.as_ref())
    });

    modules
}

/// Transform a parser::CodeSection to TextReplacement (only for uses sections)
/// Skips code sections that are not uses sections or contain comments or preprocessor nodes
pub fn transform_uses_section(
    code_section: &CodeSection,
    options: &Options,
    source: &str,
) -> Option<TextReplacement> {
    // Only process uses sections
    if code_section.keyword.kind != Kind::Uses {
        return None;
    }

    // Check if any sibling contains comments or preprocessor nodes
    for sibling in &code_section.siblings {
        match sibling.kind {
            Kind::Comment | Kind::Preprocessor => {
                // Skip this uses section if it contains comments or preprocessor directives
                warn!(
                    "Skipping uses section at byte range {}-{} due to presence of {} node",
                    code_section.keyword.start_byte,
                    sibling.end_byte,
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

    // Extract module names from siblings (excluding semicolon)
    let mut modules = Vec::new();
    let mut semicolon_end_byte = code_section.keyword.end_byte; // default to keyword end if no semicolon found

    for sibling in &code_section.siblings {
        match sibling.kind {
            Kind::Module => {
                // Extract the module text from the source using byte positions
                let module_text = &source[sibling.start_byte..sibling.end_byte];
                modules.push(module_text.to_string());
            }
            Kind::Semicolon => {
                // Remember the semicolon's end position for replacement range
                semicolon_end_byte = sibling.end_byte;
            }
            _ => continue,
        }
    }

    // Sort modules according to options
    let sorted_modules = sort_modules(&modules, options);

    // Format the replacement text
    let replacement_text = format_uses_replacement(&sorted_modules, options);

    // Determine the actual start position for replacement and adjust text if needed
    let (replacement_start, replacement_text) = adjust_replacement_for_line_position(
        source,
        code_section.keyword.start_byte,
        replacement_text,
        options,
    );

    // Create the text replacement if different from original
    create_text_replacement_if_different(
        source,
        replacement_start,
        semicolon_end_byte,
        replacement_text,
    )
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
            uses_section: crate::options::UsesSectionOptions {
                uses_section_style: style,
                override_sorting_order: Vec::new(),
                module_names_to_update: Vec::new(),
            },
            indentation: indentation.to_string(),
            line_ending,
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
        options.uses_section.override_sorting_order = vec!["System".to_string(), "Abc".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["System.A", "SystemA", "Abc.B", "AbcB", "A", "B"];
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
        options.uses_section.override_sorting_order = vec![];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["A", "B", "C"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_without_dot_boundary_for_override_namespace() {
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
        options.uses_section.override_sorting_order = vec!["System".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["SystemA.B", "A.B", "X.Y"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_override_match_is_case_insensitive() {
        let modules = vec![
            "misc.A".to_string(),
            "SYSTEM.Z".to_string(),
            "system.A".to_string(),
            "B".to_string(),
        ];
        let mut options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "    ",
            crate::options::LineEnding::Crlf,
        );
        options.uses_section.override_sorting_order = vec!["system".to_string()];
        let sorted = sort_modules(&modules, &options);
        let expected = vec!["system.A", "SYSTEM.Z", "B", "misc.A"];
        let expected: Vec<String> = expected.into_iter().map(|s| s.to_string()).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_modules_uses_locale_base_collation_for_fallback() {
        let modules = vec![
            "ProjectDB.DelphiFacade.FMOPhase".to_string(),
            "ProjectDB.DelphiFacade_Abstract".to_string(),
            "ProjectDB.DelphiFacade.FMOStep".to_string(),
            "ProjectDB.DelphiFacade_Abstract".to_string(),
        ];
        let mut options = make_options(
            UsesSectionStyle::CommaAtTheBeginning,
            "    ",
            crate::options::LineEnding::Crlf,
        );
        options.uses_section.override_sorting_order = vec![];
        let sorted = sort_modules(&modules, &options);
        let expected = vec![
            "ProjectDB.DelphiFacade_Abstract",
            "ProjectDB.DelphiFacade_Abstract",
            "ProjectDB.DelphiFacade.FMOPhase",
            "ProjectDB.DelphiFacade.FMOStep",
        ];
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
}
