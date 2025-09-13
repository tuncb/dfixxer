use crate::options::TextChangeOptions;
use crate::replacements::TextReplacement;

/// Apply text transformations based on the given options to the replacements
pub fn apply_text_transformations(
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
    options: &TextChangeOptions,
) -> Vec<TextReplacement> {
    for replacement in &mut replacements {
        // Skip final replacements that shouldn't be modified further
        if replacement.is_final {
            continue;
        }

        if let Some(ref mut text) = replacement.text {
            *text = apply_text_changes(text, options);
        } else {
            // For identity replacements, we need to get the original text,
            // apply changes, and if changed, convert to a regular replacement
            let original_text = &original_source[replacement.start..replacement.end];
            let modified_text = apply_text_changes(original_text, options);
            if modified_text != original_text {
                replacement.text = Some(modified_text);
            }
        }
    }
    replacements
}

/// Apply all text changes to a text string based on the given options
fn apply_text_changes(text: &str, options: &TextChangeOptions) -> String {
    let mut result = text.to_string();

    if options.space_after_comma {
        result = add_spaces_after_character(&result, ',');
    }

    if options.space_after_semi_colon {
        result = add_spaces_after_character(&result, ';');
    }

    result
}

/// Add spaces after a specific character in a text string where needed
fn add_spaces_after_character(text: &str, target_char: char) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        result.push(ch);

        // If we found the target character, check what follows
        if ch == target_char {
            // Look at the next character without consuming it
            if let Some(&next_ch) = chars.peek() {
                // Add space if the next character is not already a space, newline, or another target character
                if !next_ch.is_whitespace() && next_ch != target_char {
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
    fn test_add_spaces_after_character_no_commas() {
        let text = "Hello World";
        assert_eq!(add_spaces_after_character(text, ','), "Hello World");
    }

    #[test]
    fn test_add_spaces_after_character_comma_with_space() {
        let text = "Hello, World";
        assert_eq!(add_spaces_after_character(text, ','), "Hello, World");
    }

    #[test]
    fn test_add_spaces_after_character_comma_without_space() {
        let text = "Hello,World";
        assert_eq!(add_spaces_after_character(text, ','), "Hello, World");
    }

    #[test]
    fn test_add_spaces_after_character_multiple_commas() {
        let text = "A,B,C,D";
        assert_eq!(add_spaces_after_character(text, ','), "A, B, C, D");
    }

    #[test]
    fn test_add_spaces_after_character_mixed_commas() {
        let text = "A, B,C, D,E";
        assert_eq!(add_spaces_after_character(text, ','), "A, B, C, D, E");
    }

    #[test]
    fn test_add_spaces_after_character_comma_at_end() {
        let text = "Hello,";
        assert_eq!(add_spaces_after_character(text, ','), "Hello,");
    }

    #[test]
    fn test_add_spaces_after_character_comma_before_newline() {
        let text = "Hello,\nWorld";
        assert_eq!(add_spaces_after_character(text, ','), "Hello,\nWorld");
    }

    #[test]
    fn test_add_spaces_after_character_consecutive_commas() {
        let text = "A,,B";
        assert_eq!(add_spaces_after_character(text, ','), "A,, B");
    }

    #[test]
    fn test_apply_text_transformations_comma_only_with_identity_replacement() {
        let source = "Hello,World";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 11,
            text: None, // Identity replacement
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("Hello, World".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_comma_only_with_regular_replacement() {
        let source = "Original";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 8,
            text: Some("A,B,C".to_string()),
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("A, B, C".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_comma_only_mixed_replacements() {
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
            },
        ];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, Some("Hello, World".to_string()));
        assert_eq!(result[1].text, Some(" and ".to_string()));
        assert_eq!(result[2].text, Some("Baz, Qux".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_comma_only_skips_final_replacements() {
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
            },
        ];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 2);
        // Final replacement should be unchanged
        assert_eq!(result[0].text, Some("uses,System".to_string()));
        assert_eq!(result[0].is_final, true);
        // Regular replacement should have spaces added
        assert_eq!(result[1].text, Some(" test, code".to_string()));
        assert_eq!(result[1].is_final, false);
    }

    #[test]
    fn test_add_spaces_after_character_semicolon() {
        let text = "a;b;c";
        assert_eq!(add_spaces_after_character(text, ';'), "a; b; c");
    }

    #[test]
    fn test_add_spaces_after_character_semicolon_with_space() {
        let text = "a; b;c";
        assert_eq!(add_spaces_after_character(text, ';'), "a; b; c");
    }

    #[test]
    fn test_add_spaces_after_character_semicolon_before_newline() {
        let text = "a;\nb";
        assert_eq!(add_spaces_after_character(text, ';'), "a;\nb");
    }

    #[test]
    fn test_newline_behavior_comprehensive() {
        // Test various whitespace scenarios with commas
        let comma_tests = vec![
            ("a,b", "a, b"),        // No space after comma -> add space
            ("a, b", "a, b"),       // Already has space -> no change
            ("a,\nb", "a,\nb"),     // Newline after comma -> no space added
            ("a,\tb", "a,\tb"),     // Tab after comma -> no space added
            ("a,\r\nb", "a,\r\nb"), // CRLF after comma -> no space added
        ];

        for (input, expected) in comma_tests {
            assert_eq!(
                add_spaces_after_character(input, ','),
                expected,
                "Failed for comma test: {}",
                input
            );
        }

        // Test various whitespace scenarios with semicolons
        let semicolon_tests = vec![
            ("a;b", "a; b"),        // No space after semicolon -> add space
            ("a; b", "a; b"),       // Already has space -> no change
            ("a;\nb", "a;\nb"),     // Newline after semicolon -> no space added
            ("a;\tb", "a;\tb"),     // Tab after semicolon -> no space added
            ("a;\r\nb", "a;\r\nb"), // CRLF after semicolon -> no space added
        ];

        for (input, expected) in semicolon_tests {
            assert_eq!(
                add_spaces_after_character(input, ';'),
                expected,
                "Failed for semicolon test: {}",
                input
            );
        }
    }

    #[test]
    fn test_apply_text_changes_comma_only() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a, b;c, d");
    }

    #[test]
    fn test_apply_text_changes_semicolon_only() {
        let options = TextChangeOptions {
            space_after_comma: false,
            space_after_semi_colon: true,
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a,b; c,d");
    }

    #[test]
    fn test_apply_text_changes_both_comma_and_semicolon() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a, b; c, d");
    }

    #[test]
    fn test_apply_text_changes_neither() {
        let options = TextChangeOptions {
            space_after_comma: false,
            space_after_semi_colon: false,
        };
        let text = "a,b;c,d";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a,b;c,d");
    }

    #[test]
    fn test_apply_text_transformations_with_options() {
        let source = "Original";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 8,
            text: Some("a,b;c".to_string()),
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("a, b; c".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_identity_replacement() {
        let source = "a,b;c";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 5,
            text: None, // Identity replacement
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("a, b; c".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_skips_final_replacements() {
        let source = "Original";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 8,
            text: Some("a,b;c".to_string()),
            is_final: true, // Final replacement should not be modified
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("a,b;c".to_string())); // Unchanged
        assert_eq!(result[0].is_final, true);
    }
}
