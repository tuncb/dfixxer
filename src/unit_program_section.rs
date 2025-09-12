use crate::options::Options;
use crate::parser::{CodeSection, Kind};
use crate::replacements::TextReplacement;

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

    let mut replacement_text = format!("{} {};", keyword_text, module_name);

    // Determine the actual start position for replacement
    let mut replacement_start = code_section.keyword.start_byte;

    // Find the beginning of the line containing the unit/program section
    let line_start = find_line_start(source, code_section.keyword.start_byte);

    // Check what's between line start and unit/program section start
    let prefix = &source[line_start..code_section.keyword.start_byte];

    if prefix
        .chars()
        .all(|c| c.is_whitespace() && c != '\n' && c != '\r')
    {
        // Only whitespace characters before unit/program - remove them by extending replacement start
        replacement_start = line_start;
    } else if !prefix.is_empty() {
        // Non-whitespace characters before unit/program - add a newline before the unit/program section
        replacement_text = format!("{}{}", options.line_ending.to_string(), replacement_text);
    }
    // If prefix is empty, unit/program is already at start of line, no adjustment needed

    // Create the text replacement
    // If replacement_text is the same as the original unit/program section and starts at the same position, return None
    let original_text = &source[replacement_start..semicolon_end_byte];
    if replacement_text == original_text {
        return None;
    }

    Some(TextReplacement {
        start: replacement_start,
        end: semicolon_end_byte,
        text: replacement_text,
    })
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
        assert_eq!(replacement.text, "unit MyUnit;");
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 17);
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
        assert!(result.is_none()); // Should skip due to having 3 siblings instead of 2
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
