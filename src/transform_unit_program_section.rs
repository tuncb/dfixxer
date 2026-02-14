use crate::options::Options;
use crate::parser::{CodeSection, Kind};
use crate::replacements::TextReplacement;
use crate::transformer_utility::{
    adjust_replacement_for_line_position, create_text_replacement_if_different,
};

/// Transform a parser::CodeSection to TextReplacement (only for unit and program sections)
/// Expects exactly two siblings: module name followed by semicolon
/// Format: "unit module_name;" or "program module_name;" (single line)
pub fn transform_unit_program_section(
    code_section: &CodeSection,
    options: &Options,
    source: &str,
) -> Option<TextReplacement> {
    // Only process unit and program sections
    if code_section.keyword.kind != Kind::Unit && code_section.keyword.kind != Kind::Program {
        return None;
    }

    // Must have exactly two siblings: module name and semicolon
    if code_section.siblings.len() != 2 {
        return None;
    }

    // First sibling must be module name
    let module_sibling = &code_section.siblings[0];
    if module_sibling.kind != Kind::Module {
        return None;
    }

    // Second sibling must be semicolon
    let semicolon_sibling = &code_section.siblings[1];
    if semicolon_sibling.kind != Kind::Semicolon {
        return None;
    }

    // Extract the module text from the source using byte positions
    let module_name = &source[module_sibling.start_byte..module_sibling.end_byte];
    let semicolon_end_byte = semicolon_sibling.end_byte;

    // Format the replacement text as single line: "keyword module_name;"
    let keyword_text = match code_section.keyword.kind {
        Kind::Unit => "unit",
        Kind::Program => "program",
        _ => return None, // This shouldn't happen due to the check at the top
    };

    let replacement_text = format!("{} {};", keyword_text, module_name);

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
    use crate::options::{LineEnding, Options};
    use crate::parser::ParsedNode;

    fn make_options(line_ending: LineEnding) -> Options {
        Options {
            line_ending,
            ..Default::default()
        }
    }

    fn make_parsed_node(kind: Kind, start_byte: usize, end_byte: usize) -> ParsedNode {
        ParsedNode {
            kind,
            start_byte,
            end_byte,
            start_row: 0,
            start_column: 0,
            end_row: 0,
            end_column: 0,
        }
    }

    #[test]
    fn test_transform_unit_section() {
        let source = "unit MyUnit;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 5, 11),
                make_parsed_node(Kind::Semicolon, 11, 12),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should be None because original text is already formatted correctly
    }

    #[test]
    fn test_transform_program_section() {
        let source = "program MyProgram;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Program, 0, 7),
            siblings: vec![
                make_parsed_node(Kind::Module, 8, 17),
                make_parsed_node(Kind::Semicolon, 17, 18),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should be None because original text is already formatted correctly
    }

    #[test]
    fn test_transform_unit_section_with_whitespace() {
        let source = "unit\n  MyUnit\n  ;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 7, 13),
                make_parsed_node(Kind::Semicolon, 16, 17),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.text, "unit MyUnit;".to_string());
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 17);
    }

    #[test]
    fn test_transform_unit_section_with_bom_no_extra_leading_newline() {
        let source = "\u{feff}unit MyUnit;";
        let bom_len = '\u{feff}'.len_utf8();
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, bom_len, bom_len + 4),
            siblings: vec![
                make_parsed_node(Kind::Module, bom_len + 5, bom_len + 11),
                make_parsed_node(Kind::Semicolon, bom_len + 11, bom_len + 12),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should not insert a leading newline after BOM
    }

    #[test]
    fn test_skip_section_with_comment() {
        let source = "unit MyUnit; // comment";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 5, 11),
                make_parsed_node(Kind::Semicolon, 11, 12),
                make_parsed_node(Kind::Comment, 13, 23),
            ],
        };
        let options = make_options(LineEnding::Lf);
        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should skip due to extra sibling (comment)
    }

    #[test]
    fn test_skip_uses_section() {
        let source = "uses MyUnit;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Uses, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 5, 11),
                make_parsed_node(Kind::Semicolon, 11, 12),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should skip because it's not unit/program
    }

    #[test]
    fn test_skip_section_with_wrong_sibling_count() {
        let source = "unit MyUnit;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 5, 11),
                // Missing semicolon
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should skip due to having only 1 sibling instead of 2
    }

    #[test]
    fn test_skip_section_with_wrong_first_sibling() {
        let source = "unit MyUnit;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Semicolon, 5, 6), // Wrong order
                make_parsed_node(Kind::Module, 7, 13),
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should skip because first sibling is not a module
    }

    #[test]
    fn test_skip_section_with_wrong_second_sibling() {
        let source = "unit MyUnit;";
        let code_section = CodeSection {
            keyword: make_parsed_node(Kind::Unit, 0, 4),
            siblings: vec![
                make_parsed_node(Kind::Module, 5, 11),
                make_parsed_node(Kind::Module, 12, 18), // Should be semicolon
            ],
        };
        let options = make_options(LineEnding::Lf);

        let result = transform_unit_program_section(&code_section, &options, source);
        assert!(result.is_none()); // Should skip because second sibling is not a semicolon
    }
}
