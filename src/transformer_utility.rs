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

    // Preserve a UTF-8 BOM at the beginning of the file. It should not be treated
    // as inline content when deciding whether to prepend a newline.
    let mut logical_prefix = prefix;
    let mut protected_prefix_len = 0usize;
    if line_start == 0
        && let Some(stripped_prefix) = logical_prefix.strip_prefix('\u{feff}')
    {
        logical_prefix = stripped_prefix;
        protected_prefix_len = '\u{feff}'.len_utf8();
    }

    let replacement_start = if logical_prefix
        .chars()
        .all(|c| c.is_whitespace() && c != '\n' && c != '\r')
    {
        // Only whitespace characters before section - remove them by extending replacement start.
        // If the prefix is only a BOM, keep the section start unchanged to preserve BOM bytes.
        if logical_prefix.is_empty() {
            section_start_byte
        } else {
            line_start + protected_prefix_len
        }
    } else if !logical_prefix.is_empty() {
        // Non-whitespace characters before section - add a newline before the section
        replacement_text = format!("{}{}", options.line_ending, replacement_text);
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
        text: replacement_text,
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
    fn test_adjust_replacement_with_bom_only_prefix() {
        let source = "\u{feff}keyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();
        let section_start = '\u{feff}'.len_utf8();

        let (start, text) =
            adjust_replacement_for_line_position(source, section_start, replacement_text, &options);

        assert_eq!(start, section_start); // Keep BOM intact
        assert_eq!(text, "keyword formatted;"); // No prepended newline
    }

    #[test]
    fn test_adjust_replacement_with_bom_and_whitespace_prefix() {
        let source = "\u{feff}  keyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();
        let section_start = '\u{feff}'.len_utf8() + 2;

        let (start, text) =
            adjust_replacement_for_line_position(source, section_start, replacement_text, &options);

        assert_eq!(start, '\u{feff}'.len_utf8()); // Trim spaces, preserve BOM
        assert_eq!(text, "keyword formatted;"); // No prepended newline
    }

    #[test]
    fn test_adjust_replacement_with_bom_and_non_whitespace_prefix() {
        let source = "\u{feff}abckeyword something;";
        let options = make_options(LineEnding::Lf);
        let replacement_text = "keyword formatted;".to_string();
        let section_start = '\u{feff}'.len_utf8() + 3;

        let (start, text) =
            adjust_replacement_for_line_position(source, section_start, replacement_text, &options);

        assert_eq!(start, section_start); // Non-whitespace prefix remains inline
        assert_eq!(text, "\nkeyword formatted;"); // Newline still prepended
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
        let result =
            create_text_replacement_if_different(source, 0, source.len(), "new text".to_string());
        assert!(result.is_some());
        let replacement = result.unwrap();
        assert_eq!(replacement.start, 0);
        assert_eq!(replacement.end, 13);
        assert_eq!(replacement.text, "new text".to_string());
    }
}
