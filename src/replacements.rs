use crate::dfixxer_error::DFixxerError;

#[derive(Debug)]
pub struct TextReplacement {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl TextReplacement {
    /// Get the line and column numbers for a given position in the source text
    fn get_line_column(source: &str, position: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;

        for (i, ch) in source.char_indices() {
            if i >= position {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column)
    }

    /// Get the original text that would be replaced
    fn get_original_text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

pub fn print_replacements(original_source: &str, replacements: &[TextReplacement]) {
    if replacements.is_empty() {
        return;
    }

    for (i, replacement) in replacements.iter().enumerate() {
        let (start_line, start_col) =
            TextReplacement::get_line_column(original_source, replacement.start);
        let (end_line, end_col) =
            TextReplacement::get_line_column(original_source, replacement.end);
        let original_text = replacement.get_original_text(original_source);

        println!("Replacement {}:", i + 1);
        println!(
            "  Location: {}:{}-{}:{}",
            start_line, start_col, end_line, end_col
        );
        println!("  Original:");
        for line in original_text.lines() {
            println!("    - {}", line);
        }
        println!("  Replacement:");
        for line in replacement.text.lines() {
            println!("    + {}", line);
        }
        println!();
    }
}

/// Generate identity replacements for the gaps between existing replacements
fn fill_gaps_with_identity_replacements(
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Vec<TextReplacement> {
    if replacements.is_empty() {
        // If no replacements, create one identity replacement for the entire source
        return vec![TextReplacement {
            start: 0,
            end: original_source.len(),
            text: original_source.to_string(),
        }];
    }

    // Sort replacements by start position
    replacements.sort_by_key(|r| r.start);

    let mut all_replacements = Vec::new();
    let mut last_end = 0;

    for replacement in replacements {
        // Add identity replacement for gap before this replacement
        if last_end < replacement.start {
            all_replacements.push(TextReplacement {
                start: last_end,
                end: replacement.start,
                text: original_source[last_end..replacement.start].to_string(),
            });
        }

        // Capture the end position before moving the replacement
        let replacement_end = replacement.end;

        // Add the actual replacement
        all_replacements.push(replacement);
        last_end = replacement_end;
    }

    // Add identity replacement for any remaining text after the last replacement
    if last_end < original_source.len() {
        all_replacements.push(TextReplacement {
            start: last_end,
            end: original_source.len(),
            text: original_source[last_end..].to_string(),
        });
    }

    all_replacements
}

pub fn apply_replacements(
    filename: &str,
    original_source: &str,
    replacements: Vec<TextReplacement>,
) -> Result<(), DFixxerError> {
    if replacements.is_empty() {
        return Ok(());
    }

    // Generate all replacements including identity replacements for unchanged parts
    let all_replacements = fill_gaps_with_identity_replacements(original_source, replacements);

    // Construct the final text by concatenating all replacement texts
    let modified_source: String = all_replacements
        .into_iter()
        .map(|r| r.text)
        .collect();

    // Write the modified source back to the file
    std::fs::write(filename, modified_source)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_gaps_empty_replacements() {
        let source = "Hello, world!";
        let replacements = vec![];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, source.len());
        assert_eq!(result[0].text, source);
    }

    #[test]
    fn test_fill_gaps_single_replacement() {
        let source = "Hello, world!";
        let replacements = vec![
            TextReplacement {
                start: 7,
                end: 12,
                text: "Rust".to_string(),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 3);
        // First gap: "Hello, "
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 7);
        assert_eq!(result[0].text, "Hello, ");
        // Replacement: "Rust"
        assert_eq!(result[1].start, 7);
        assert_eq!(result[1].end, 12);
        assert_eq!(result[1].text, "Rust");
        // Last gap: "!"
        assert_eq!(result[2].start, 12);
        assert_eq!(result[2].end, 13);
        assert_eq!(result[2].text, "!");
    }

    #[test]
    fn test_fill_gaps_multiple_replacements() {
        let source = "The quick brown fox";
        let replacements = vec![
            TextReplacement {
                start: 4,
                end: 9,
                text: "slow".to_string(),
            },
            TextReplacement {
                start: 10,
                end: 15,
                text: "green".to_string(),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 5);
        // Check that all parts concatenate to form expected result
        let final_text: String = result.iter().map(|r| r.text.clone()).collect();
        assert_eq!(final_text, "The slow green fox");
    }

    #[test]
    fn test_fill_gaps_adjacent_replacements() {
        let source = "abcdef";
        let replacements = vec![
            TextReplacement {
                start: 1,
                end: 3,
                text: "XX".to_string(),
            },
            TextReplacement {
                start: 3,
                end: 5,
                text: "YY".to_string(),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 4);
        let final_text: String = result.iter().map(|r| r.text.clone()).collect();
        assert_eq!(final_text, "aXXYYf");
    }

    #[test]
    fn test_fill_gaps_entire_file_replacement() {
        let source = "original";
        let replacements = vec![
            TextReplacement {
                start: 0,
                end: source.len(),
                text: "replaced".to_string(),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "replaced");
    }
}
