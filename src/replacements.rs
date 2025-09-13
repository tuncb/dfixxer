use crate::dfixxer_error::DFixxerError;

#[derive(Debug)]
pub struct TextReplacement {
    pub start: usize,
    pub end: usize,
    pub text: Option<String>, // None means use original text from source[start..end]
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

pub fn print_replacement(original_source: &str, replacement: &TextReplacement, index: usize) {
    let (start_line, start_col) =
        TextReplacement::get_line_column(original_source, replacement.start);
    let (end_line, end_col) =
        TextReplacement::get_line_column(original_source, replacement.end);
    let original_text = replacement.get_original_text(original_source);

    println!("Replacement {}:", index);
    println!(
        "  Location: {}:{}-{}:{}",
        start_line, start_col, end_line, end_col
    );
    println!("  Original:");
    for line in original_text.lines() {
        println!("    - {}", line);
    }
    println!("  Replacement:");
    if let Some(ref text) = replacement.text {
        for line in text.lines() {
            println!("    + {}", line);
        }
    }
    println!();
}

pub fn print_replacements(original_source: &str, replacements: &[TextReplacement]) {
    let non_identity_replacements: Vec<_> = replacements
        .iter()
        .filter(|r| r.text.is_some())
        .collect();

    if non_identity_replacements.is_empty() {
        return;
    }

    for (i, replacement) in non_identity_replacements.iter().enumerate() {
        print_replacement(original_source, replacement, i + 1);
    }
}

/// Generate identity replacements for the gaps between existing replacements
pub fn fill_gaps_with_identity_replacements(
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Vec<TextReplacement> {
    if replacements.is_empty() {
        // If no replacements, create one identity replacement for the entire source
        return vec![TextReplacement {
            start: 0,
            end: original_source.len(),
            text: None, // Identity replacement - use original text
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
                text: None, // Identity replacement - use original text
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
            text: None, // Identity replacement - use original text
        });
    }

    all_replacements
}

pub fn merge_replacements(
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
        .map(|r| match r.text {
            Some(text) => text,
            None => original_source[r.start..r.end].to_string(),
        })
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
        assert_eq!(result[0].text, None);
    }

    #[test]
    fn test_fill_gaps_single_replacement() {
        let source = "Hello, world!";
        let replacements = vec![TextReplacement {
            start: 7,
            end: 12,
            text: Some("Rust".to_string()),
        }];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 3);
        // First gap: "Hello, "
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 7);
        assert_eq!(result[0].text, None);
        // Replacement: "Rust"
        assert_eq!(result[1].start, 7);
        assert_eq!(result[1].end, 12);
        assert_eq!(result[1].text, Some("Rust".to_string()));
        // Last gap: "!"
        assert_eq!(result[2].start, 12);
        assert_eq!(result[2].end, 13);
        assert_eq!(result[2].text, None);
    }

    #[test]
    fn test_fill_gaps_multiple_replacements() {
        let source = "The quick brown fox";
        let replacements = vec![
            TextReplacement {
                start: 4,
                end: 9,
                text: Some("slow".to_string()),
            },
            TextReplacement {
                start: 10,
                end: 15,
                text: Some("green".to_string()),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 5);
        // Check that all parts concatenate to form expected result
        let final_text: String = result
            .iter()
            .map(|r| match &r.text {
                Some(text) => text.clone(),
                None => source[r.start..r.end].to_string(),
            })
            .collect();
        assert_eq!(final_text, "The slow green fox");
    }

    #[test]
    fn test_fill_gaps_adjacent_replacements() {
        let source = "abcdef";
        let replacements = vec![
            TextReplacement {
                start: 1,
                end: 3,
                text: Some("XX".to_string()),
            },
            TextReplacement {
                start: 3,
                end: 5,
                text: Some("YY".to_string()),
            },
        ];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 4);
        let final_text: String = result
            .iter()
            .map(|r| match &r.text {
                Some(text) => text.clone(),
                None => source[r.start..r.end].to_string(),
            })
            .collect();
        assert_eq!(final_text, "aXXYYf");
    }

    #[test]
    fn test_fill_gaps_entire_file_replacement() {
        let source = "original";
        let replacements = vec![TextReplacement {
            start: 0,
            end: source.len(),
            text: Some("replaced".to_string()),
        }];
        let result = fill_gaps_with_identity_replacements(source, replacements);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, Some("replaced".to_string()));
    }
}
