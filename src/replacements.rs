use crate::dfixxer_error::DFixxerError;

#[derive(Debug, Clone)]
pub struct TextReplacement {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSection {
    pub start: usize,
    pub end: usize,
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
    let (end_line, end_col) = TextReplacement::get_line_column(original_source, replacement.end);
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
    for line in replacement.text.lines() {
        println!("    + {}", line);
    }
    println!();
}

pub fn print_replacements(original_source: &str, replacements: &[TextReplacement]) {
    if replacements.is_empty() {
        return;
    }
    for (i, replacement) in replacements.iter().enumerate() {
        print_replacement(original_source, replacement, i + 1);
    }
}

/// Generate sections for the gaps between existing replacements (not including the replacements themselves)
pub fn compute_source_sections(
    original_source: &str,
    replacements: &[TextReplacement],
) -> Vec<SourceSection> {
    if replacements.is_empty() {
        return vec![SourceSection {
            start: 0,
            end: original_source.len(),
        }];
    }

    // Collect indices and sort (without mutating caller slice)
    let mut order: Vec<_> = replacements.iter().collect();
    order.sort_by_key(|r| r.start);

    let mut sections: Vec<SourceSection> = Vec::new();
    let mut last_end = 0usize;

    for r in order {
        if last_end < r.start {
            sections.push(SourceSection {
                start: last_end,
                end: r.start,
            });
        }
        last_end = r.end;
    }

    if last_end < original_source.len() {
        sections.push(SourceSection {
            start: last_end,
            end: original_source.len(),
        });
    }

    sections
}

pub fn merge_replacements(
    filename: &str,
    original_source: &str,
    replacements: Vec<TextReplacement>,
) -> Result<(), DFixxerError> {
    if replacements.is_empty() {
        return Ok(());
    }

    // Sort replacements by start position
    let mut sorted_replacements = replacements;
    sorted_replacements.sort_by_key(|r| r.start);

    // Build final text by processing original text and applying replacements
    let mut out = String::new();
    let mut current_pos = 0;

    for replacement in &sorted_replacements {
        // Add any original text before this replacement
        if current_pos < replacement.start {
            out.push_str(&original_source[current_pos..replacement.start]);
        }

        // Add replacement text
        out.push_str(&replacement.text);

        current_pos = replacement.end;
    }

    // Add any remaining original text after the last replacement
    if current_pos < original_source.len() {
        out.push_str(&original_source[current_pos..]);
    }

    std::fs::write(filename, out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_gaps_single_replacement() {
        let source = "Hello, world!";
        let replacements = vec![TextReplacement {
            start: 7,
            end: 12,
            text: "Rust".to_string(),
        }];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 7 },
                SourceSection { start: 12, end: 13 },
            ]
        );
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
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 4 },
                SourceSection { start: 9, end: 10 },
                SourceSection {
                    start: 15,
                    end: source.len()
                },
            ]
        );
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
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 1 },
                SourceSection { start: 5, end: 6 },
            ]
        );
    }

    #[test]
    fn test_fill_gaps_entire_file_replacement() {
        let source = "original";
        let replacements = vec![TextReplacement {
            start: 0,
            end: source.len(),
            text: "replaced".to_string(),
        }];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(result, vec![]);
    }
}
