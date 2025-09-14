use crate::options::Options;
use crate::replacements::TextReplacement;

/// Find the start of the line containing the given byte position
pub fn find_line_start(source: &str, position: usize) -> usize {
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

/// Helper to determine the actual replacement start position and adjust replacement text
/// based on what appears before the section on the same line
pub fn adjust_replacement_for_line_position(
    source: &str,
    section_start_byte: usize,
    mut replacement_text: String,
    options: &Options,
) -> (usize, String) {
    // Find the beginning of the line containing the section
    let line_start = find_line_start(source, section_start_byte);

    // Check what's between line start and section start
    let prefix = &source[line_start..section_start_byte];

    let replacement_start = if prefix
        .chars()
        .all(|c| c.is_whitespace() && c != '\n' && c != '\r')
    {
        // Only whitespace characters before section - remove them by extending replacement start
        line_start
    } else if !prefix.is_empty() {
        // Non-whitespace characters before section - add a newline before the section
        replacement_text = format!("{}{}", options.line_ending.to_string(), replacement_text);
        section_start_byte
    } else {
        // If prefix is empty, section is already at start of line, no adjustment needed
        section_start_byte
    };

    (replacement_start, replacement_text)
}

/// Create a TextReplacement if the replacement text differs from the original
pub fn create_text_replacement_if_different(
    source: &str,
    replacement_start: usize,
    replacement_end: usize,
    replacement_text: String,
) -> Option<TextReplacement> {
    let original_text = &source[replacement_start..replacement_end];
    if replacement_text == original_text {
        return None;
    }

    Some(TextReplacement {
        start: replacement_start,
        end: replacement_end,
        text: Some(replacement_text),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};

    fn make_options(line_ending: LineEnding) -> Options {
        Options {
            line_ending,
            ..Default::default()
        }
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
    fn test_adjust_replacement_with_whitespace_prefix() {
        let source = "  keyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();

        let (start, text) = adjust_replacement_for_line_position(
            source,
            2, // keyword starts at position 2
            replacement_text,
            &options,
        );

        assert_eq!(start, 0); // Should start at beginning of line
        assert_eq!(text, "keyword formatted;"); // Text unchanged
    }

    #[test]
    fn test_adjust_replacement_with_non_whitespace_prefix() {
        let source = "otherkeyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();

        let (start, text) = adjust_replacement_for_line_position(
            source,
            5, // section starts at position 5
            replacement_text,
            &options,
        );

        assert_eq!(start, 5); // Should start at original position
        assert_eq!(text, "\nkeyword formatted;"); // Text should have newline prepended
    }

    #[test]
    fn test_adjust_replacement_at_line_start() {
        let source = "keyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();

        let (start, text) = adjust_replacement_for_line_position(
            source,
            0, // keyword starts at beginning
            replacement_text,
            &options,
        );

        assert_eq!(start, 0); // Should start at beginning
        assert_eq!(text, "keyword formatted;"); // Text unchanged
    }

    #[test]
    fn test_create_text_replacement_if_different_same_text() {
        let source = "original text";
        let result = create_text_replacement_if_different(
            source,
            0,
            source.len(),
            "original text".to_string(),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_create_text_replacement_if_different_different_text() {
        let source = "original text";
        let result = create_text_replacement_if_different(
            source,
            0,
            source.len(),
            "new text".to_string(),
        );
        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 13);
        assert_eq!(replacement.text, Some("new text".to_string()));
    }
}
