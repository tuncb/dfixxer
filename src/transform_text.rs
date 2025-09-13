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
    // State machine to skip Delphi string literals and comments for spacing insertion.
    // We still may trim trailing whitespace (optionally) per line, but trimming is safe
    // inside comments / strings per spec given by user.
    #[derive(Copy, Clone, PartialEq)]
    enum State {
        Code,
        StringLiteral,    // Inside '...'
        LineComment,      // // until newline
        BraceComment,     // { ... }
        ParenStarComment, // (* ... *)
    }

    let mut result = String::with_capacity(text.len());
    let mut state = State::Code;
    let mut chars = text.chars().peekable();

    // For trimming we accumulate current line raw output, then on newline flush trimmed.
    let do_trim = options.trim_trailing_whitespace;
    let mut current_line = String::new();

    // Helper to push a character to either current line buffer (if trimming) or directly.
    let push_char = |c: char, current_line: &mut String, result: &mut String| {
        if do_trim {
            current_line.push(c);
        } else {
            result.push(c);
        }
    };

    // Helper to flush a newline (\n or \r) handling trimming.
    let flush_line_ending = |newline: char, current_line: &mut String, result: &mut String| {
        if do_trim {
            // Trim end whitespace of accumulated line, then push
            let trimmed = current_line.trim_end();
            result.push_str(trimmed);
            current_line.clear();
            result.push(newline);
        } else {
            result.push(newline);
        }
    };

    while let Some(ch) = chars.next() {
        match state {
            State::Code => {
                match ch {
                    '\'' => {
                        // Enter string literal
                        push_char(ch, &mut current_line, &mut result);
                        state = State::StringLiteral;
                    }
                    '{' => {
                        // Brace comment
                        push_char(ch, &mut current_line, &mut result);
                        state = State::BraceComment;
                    }
                    '(' => {
                        // Could start (* comment *)
                        if let Some('*') = chars.peek().copied() {
                            // consume '*'
                            let star = chars.next().unwrap();
                            push_char('(', &mut current_line, &mut result);
                            push_char(star, &mut current_line, &mut result);
                            state = State::ParenStarComment;
                        } else {
                            push_char('(', &mut current_line, &mut result);
                        }
                    }
                    '/' => {
                        if let Some('/') = chars.peek().copied() {
                            // line comment
                            let slash2 = chars.next().unwrap();
                            push_char('/', &mut current_line, &mut result);
                            push_char(slash2, &mut current_line, &mut result);
                            state = State::LineComment;
                        } else {
                            push_char('/', &mut current_line, &mut result);
                        }
                    }
                    ',' => {
                        // Potential spacing insertion (only in code state)
                        push_char(',', &mut current_line, &mut result);
                        if options.space_after_comma {
                            if let Some(&next_ch) = chars.peek() {
                                if !next_ch.is_whitespace() && next_ch != ',' {
                                    push_char(' ', &mut current_line, &mut result);
                                }
                            }
                        }
                    }
                    ';' => {
                        push_char(';', &mut current_line, &mut result);
                        if options.space_after_semi_colon {
                            if let Some(&next_ch) = chars.peek() {
                                if !next_ch.is_whitespace() && next_ch != ';' {
                                    push_char(' ', &mut current_line, &mut result);
                                }
                            }
                        }
                    }
                    '\n' | '\r' => {
                        flush_line_ending(ch, &mut current_line, &mut result);
                    }
                    _ => {
                        push_char(ch, &mut current_line, &mut result);
                    }
                }
            }
            State::StringLiteral => {
                if ch == '\n' || ch == '\r' {
                    // Unterminated string at line break: exit string state
                    flush_line_ending(ch, &mut current_line, &mut result);
                    state = State::Code;
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '\'' {
                        // Delphi/Pascal doubles '' inside a string to escape a single quote.
                        if let Some('\'') = chars.peek().copied() {
                            // This is an escaped quote, consume the second quote and stay in string
                            let escaped_quote = chars.next().unwrap();
                            push_char(escaped_quote, &mut current_line, &mut result);
                            // Stay in StringLiteral state - this is still part of the string
                        } else {
                            // End of string literal
                            state = State::Code;
                        }
                    }
                }
            }
            State::LineComment => {
                if ch == '\n' || ch == '\r' {
                    // End of line comment - use consistent flush_line_ending logic
                    flush_line_ending(ch, &mut current_line, &mut result);
                    state = State::Code;
                } else {
                    push_char(ch, &mut current_line, &mut result);
                }
            }
            State::BraceComment => {
                if ch == '\n' || ch == '\r' {
                    // Handle newlines in brace comments consistently
                    flush_line_ending(ch, &mut current_line, &mut result);
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '}' {
                        state = State::Code;
                    }
                }
            }
            State::ParenStarComment => {
                if ch == '\n' || ch == '\r' {
                    // Handle newlines in paren-star comments consistently
                    flush_line_ending(ch, &mut current_line, &mut result);
                } else {
                    push_char(ch, &mut current_line, &mut result);
                    if ch == '*' {
                        // Look ahead for ) to end comment
                        if let Some(')') = chars.peek().copied() {
                            let closing_paren = chars.next().unwrap();
                            push_char(closing_paren, &mut current_line, &mut result);
                            state = State::Code;
                        }
                    }
                }
            }
        }
    }

    if do_trim && !current_line.is_empty() {
        // flush last line (no newline present)
        let trimmed = current_line.trim_end();
        result.push_str(trimmed);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
    fn test_apply_text_changes_comma_only() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
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
            trim_trailing_whitespace: false,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("a,b;c".to_string())); // Unchanged
        assert_eq!(result[0].is_final, true);
    }

    #[test]
    fn test_apply_text_changes_with_trim_trailing_whitespace() {
        let options = TextChangeOptions {
            space_after_comma: false,
            space_after_semi_colon: false,
            trim_trailing_whitespace: true,
        };
        let text = "Line 1   \nLine 2\t\t\nLine 3 ";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_apply_text_changes_combined_comma_and_trim() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
            trim_trailing_whitespace: true,
        };
        let text = "a,b,c   \nd,e,f\t\t";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a, b, c\nd, e, f");
    }

    #[test]
    fn test_apply_text_changes_all_options_enabled() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: true,
        };
        let text = "a,b;c,d   \ne,f;g,h\t\t";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a, b; c, d\ne, f; g, h");
    }

    #[test]
    fn test_apply_text_transformations_with_trim_trailing_whitespace() {
        let source = "Original   ";
        let replacements = vec![TextReplacement {
            start: 0,
            end: 11,
            text: Some("a,b;c   \nd,e;f\t\t".to_string()),
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: true,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("a, b; c\nd, e; f".to_string()));
    }

    #[test]
    fn test_apply_text_transformations_identity_with_trim() {
        let source = "Hello,World   \nFoo;Bar\t\t";
        let replacements = vec![TextReplacement {
            start: 0,
            end: source.len(),
            text: None, // Identity replacement
            is_final: false,
        }];
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: true,
        };

        let result = apply_text_transformations(source, replacements, &options);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("Hello, World\nFoo; Bar".to_string()));
    }

    // --- Tests for edge cases and bug fixes ---

    #[test]
    fn test_escaped_quotes_in_string_literals() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        // Test escaped single quotes in Delphi/Pascal strings
        let text = "s := 'It''s a test',x;y";
        let result = apply_text_changes(text, &options);
        // The comma/semicolon inside the string should not be spaced
        assert_eq!(result, "s := 'It''s a test', x; y");
    }

    #[test]
    fn test_complex_escaped_quotes() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: false,
            trim_trailing_whitespace: false,
        };
        // Multiple escaped quotes and code after
        let text = "msg := 'Can''t say ''hello'', sorry',next";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "msg := 'Can''t say ''hello'', sorry', next");
    }

    #[test]
    fn test_unterminated_string_with_line_break() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        // Unterminated string that breaks at newline
        let text = "s := 'unterminated\ncode,after;break";
        let result = apply_text_changes(text, &options);
        // After line break, spacing should be applied
        assert_eq!(result, "s := 'unterminated\ncode, after; break");
    }

    #[test]
    fn test_multiline_comments_with_spacing() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        // Test multiline brace comments
        let text = "{ multi\nline,comment;here }\ncode,after";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "{ multi\nline,comment;here }\ncode, after");
    }

    #[test]
    fn test_multiline_paren_star_comments() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        // Test multiline (* *) comments
        let text = "(* multi\nline,comment;here *)\ncode,after";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "(* multi\nline,comment;here *)\ncode, after");
    }

    #[test]
    fn test_trim_with_different_line_endings() {
        let options = TextChangeOptions {
            space_after_comma: false,
            space_after_semi_colon: false,
            trim_trailing_whitespace: true,
        };
        // Test trimming with both LF and CRLF
        let text = "line1   \r\nline2\t\t\nline3   ";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "line1\r\nline2\nline3");
    }

    #[test]
    fn test_spacing_with_consecutive_punctuation() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        // Test that we don't add space before another comma/semicolon, but do add after
        let text = "a,,b;;c,;d";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "a,, b;; c, ; d"); // Spaces added after punctuation, not before
    }

    // --- Original tests ensuring spacing is skipped inside strings & comments ---
    #[test]
    fn test_skip_spacing_inside_string_literal() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        let text = "'a,b;c',x;y";
        // Only commas/semicolons outside the quotes should be spaced.
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "'a,b;c', x; y");
    }

    #[test]
    fn test_skip_spacing_inside_brace_comment() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        let text = "{a,b;c},x;y";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "{a,b;c}, x; y");
    }

    #[test]
    fn test_skip_spacing_inside_paren_star_comment() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        let text = "(*a,b;c*),x;y";
        let result = apply_text_changes(text, &options);
        assert_eq!(result, "(*a,b;c*), x; y");
    }

    #[test]
    fn test_skip_spacing_inside_line_comment() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        let text = "// a,b;c\nx,y;z";
        let result = apply_text_changes(text, &options);
        // Only second line is transformed.
        assert_eq!(result, "// a,b;c\nx, y; z");
    }

    #[test]
    fn test_mixed_code_and_comments_and_strings() {
        let options = TextChangeOptions {
            space_after_comma: true,
            space_after_semi_colon: true,
            trim_trailing_whitespace: false,
        };
        let text = "val:='a,b'; // c,d;e\n{ x,y;z } foo,bar;baz (* p,q;r *) qux,quux";
        let result = apply_text_changes(text, &options);
        assert_eq!(
            result,
            "val:='a,b'; // c,d;e\n{ x,y;z } foo, bar; baz (* p,q;r *) qux, quux"
        );
    }
}
