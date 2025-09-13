use crate::replacements::TextReplacement;

/// Add spaces after commas in the given replacements
pub fn add_spaces_after_commas(
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Vec<TextReplacement> {
    for replacement in &mut replacements {
        // Skip final replacements that shouldn't be modified further
        if replacement.is_final {
            continue;
        }

        if let Some(ref mut text) = replacement.text {
            *text = add_spaces_to_text(text);
        } else {
            // For identity replacements, we need to get the original text,
            // add spaces, and if changed, convert to a regular replacement
            let original_text = &original_source[replacement.start..replacement.end];
            let modified_text = add_spaces_to_text(original_text);
            if modified_text != original_text {
                replacement.text = Some(modified_text);
            }
        }
    }
    replacements
}

/// Add spaces after commas in a text string where needed
fn add_spaces_to_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        result.push(ch);

        // If we found a comma, check what follows
        if ch == ',' {
            // Look at the next character without consuming it
            if let Some(&next_ch) = chars.peek() {
                // Add space if the next character is not already a space, newline, or another comma
                if !next_ch.is_whitespace() && next_ch != ',' {
                    result.push(' ');
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_spaces_to_text_no_commas() {
        let text = "Hello World";
        assert_eq!(add_spaces_to_text(text), "Hello World");
    }

    #[test]
    fn test_add_spaces_to_text_comma_with_space() {
        let text = "Hello, World";
        assert_eq!(add_spaces_to_text(text), "Hello, World");
    }

    #[test]
    fn test_add_spaces_to_text_comma_without_space() {
        let text = "Hello,World";
        assert_eq!(add_spaces_to_text(text), "Hello, World");
    }

    #[test]
    fn test_add_spaces_to_text_multiple_commas() {
        let text = "A,B,C,D";
        assert_eq!(add_spaces_to_text(text), "A, B, C, D");
    }

    #[test]
    fn test_add_spaces_to_text_mixed_commas() {
        let text = "A, B,C, D,E";
        assert_eq!(add_spaces_to_text(text), "A, B, C, D, E");
    }

    #[test]
    fn test_add_spaces_to_text_comma_at_end() {
        let text = "Hello,";
        assert_eq!(add_spaces_to_text(text), "Hello,");
    }

    #[test]
    fn test_add_spaces_to_text_comma_before_newline() {
        let text = "Hello,\nWorld";
        assert_eq!(add_spaces_to_text(text), "Hello,\nWorld");
    }

    #[test]
    fn test_add_spaces_to_text_consecutive_commas() {
        let text = "A,,B";
        assert_eq!(add_spaces_to_text(text), "A,, B");
    }

    #[test]
    fn test_add_spaces_after_commas_with_identity_replacement() {
        let source = "Hello,World";
        let replacements = vec![
            TextReplacement {
                start: 0,
                end: 11,
                text: None, // Identity replacement
                is_final: false,
            }
        ];

        let result = add_spaces_after_commas(source, replacements);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("Hello, World".to_string()));
    }

    #[test]
    fn test_add_spaces_after_commas_with_regular_replacement() {
        let source = "Original";
        let replacements = vec![
            TextReplacement {
                start: 0,
                end: 8,
                text: Some("A,B,C".to_string()),
                is_final: false,
            }
        ];

        let result = add_spaces_after_commas(source, replacements);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("A, B, C".to_string()));
    }

    #[test]
    fn test_add_spaces_after_commas_mixed_replacements() {
        let source = "Hello,World and Foo,Bar";
        let replacements = vec![
            TextReplacement {
                start: 0,
                end: 11,
                text: None, // Identity replacement that needs modification
                is_final: false,
            },
            TextReplacement {
                start: 11,
                end: 15,
                text: Some(" and ".to_string()), // Regular replacement, no commas
                is_final: false,
            },
            TextReplacement {
                start: 15,
                end: 23,
                text: Some("Baz,Qux".to_string()), // Regular replacement with comma
                is_final: false,
            }
        ];

        let result = add_spaces_after_commas(source, replacements);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, Some("Hello, World".to_string()));
        assert_eq!(result[1].text, Some(" and ".to_string()));
        assert_eq!(result[2].text, Some("Baz, Qux".to_string()));
    }

    #[test]
    fn test_add_spaces_after_commas_skips_final_replacements() {
        let source = "Hello,World and Foo,Bar";
        let replacements = vec![
            TextReplacement {
                start: 0,
                end: 11,
                text: Some("uses,System".to_string()), // Final replacement (uses section)
                is_final: true,
            },
            TextReplacement {
                start: 11,
                end: 23,
                text: Some(" test,code".to_string()), // Regular replacement
                is_final: false,
            }
        ];

        let result = add_spaces_after_commas(source, replacements);
        assert_eq!(result.len(), 2);
        // Final replacement should be unchanged
        assert_eq!(result[0].text, Some("uses,System".to_string()));
        assert_eq!(result[0].is_final, true);
        // Regular replacement should have spaces added
        assert_eq!(result[1].text, Some(" test, code".to_string()));
        assert_eq!(result[1].is_final, false);
    }
}