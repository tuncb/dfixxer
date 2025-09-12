use crate::options::Options;
use crate::parser::{CodeSection, Kind};
use crate::replacements::TextReplacement;
use crate::transformer_utility::{adjust_replacement_for_line_position, create_text_replacement_if_different};

/// Transform a single keyword section to lowercase if needed
pub fn transform_single_keyword_section(
    source: &str,
    code_section: &CodeSection,
    options: &Options,
) -> Option<TextReplacement> {
    // Only handle single-word keyword sections
    match code_section.keyword.kind {
        Kind::Interface | Kind::Implementation | Kind::Initialization | Kind::Finalization => {}
        _ => return None,
    }

    // Get the original text of the keyword
    let keyword_start = code_section.keyword.start_byte;
    let keyword_end = code_section.keyword.end_byte;
    let original_keyword = &source[keyword_start..keyword_end];

    // Check if the keyword is already lowercase
    let lowercase_keyword = original_keyword.to_lowercase();
    if original_keyword == lowercase_keyword {
        return None; // No transformation needed
    }

    // Use transformer utility to handle line positioning
    let (replacement_start, replacement_text) = adjust_replacement_for_line_position(
        source,
        keyword_start,
        lowercase_keyword,
        options,
    );

    // Create replacement if the text is different
    create_text_replacement_if_different(
        source,
        replacement_start,
        keyword_end,
        replacement_text,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::{Kind, ParsedNode};

    fn make_options() -> Options {
        Options {
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    fn make_code_section(kind: Kind, start_byte: usize, end_byte: usize) -> CodeSection {
        CodeSection {
            keyword: ParsedNode {
                kind,
                start_byte,
                end_byte,
                start_row: 0,
                start_column: start_byte,
                end_row: 0,
                end_column: end_byte,
            },
            siblings: Vec::new(),
        }
    }

    #[test]
    fn test_transform_interface_uppercase_to_lowercase() {
        let source = "INTERFACE";
        let code_section = make_code_section(Kind::Interface, 0, 9);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 9);
        assert_eq!(replacement.text, "interface");
    }

    #[test]
    fn test_transform_implementation_mixedcase_to_lowercase() {
        let source = "ImPlEmEnTaTiOn";
        let code_section = make_code_section(Kind::Implementation, 0, 14);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 14);
        assert_eq!(replacement.text, "implementation");
    }

    #[test]
    fn test_transform_initialization_already_lowercase() {
        let source = "initialization";
        let code_section = make_code_section(Kind::Initialization, 0, 14);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_none()); // No transformation needed
    }

    #[test]
    fn test_transform_finalization_with_whitespace_prefix() {
        let source = "  FINALIZATION";
        let code_section = make_code_section(Kind::Finalization, 2, 14);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 0); // Should start at beginning of line (removes whitespace)
        assert_eq!(replacement.end, 14);
        assert_eq!(replacement.text, "finalization");
    }

    #[test]
    fn test_transform_with_non_whitespace_prefix() {
        let source = "sometext INTERFACE";
        let code_section = make_code_section(Kind::Interface, 9, 18);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 9); // Should start at original position
        assert_eq!(replacement.end, 18);
        assert_eq!(replacement.text, "\ninterface"); // Should have newline prepended
    }

    #[test]
    fn test_skip_non_single_keyword_sections() {
        let source = "USES";
        let code_section = make_code_section(Kind::Uses, 0, 4);
        let options = make_options();

        let result = transform_single_keyword_section(source, &code_section, &options);

        assert!(result.is_none()); // Should skip non-single-keyword sections
    }
}