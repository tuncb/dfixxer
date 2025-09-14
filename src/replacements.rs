use crate::dfixxer_error::DFixxerError;

#[derive(Debug, Clone)]
pub struct TextReplacement {
    pub start: usize,
    pub end: usize,
    pub text: Option<String>, // None means use original text from source[start..end]
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
    if let Some(ref text) = replacement.text {
        for line in text.lines() {
            println!("    + {}", line);
        }
    }
    println!();
}

pub fn print_replacements(original_source: &str, replacements: &[TextReplacement]) {
    let non_identity_replacements: Vec<_> =
        replacements.iter().filter(|r| r.text.is_some()).collect();

    if non_identity_replacements.is_empty() {
        return;
    }

    for (i, replacement) in non_identity_replacements.iter().enumerate() {
        print_replacement(original_source, replacement, i + 1);
    }
}

/// Generate identity replacements for the gaps between existing replacements
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
        sections.push(SourceSection {
            start: r.start,
            end: r.end,
        });
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

// Original public API retained for external callers.
pub fn fill_gaps_with_identity_replacements(
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Vec<TextReplacement> {
    if replacements.is_empty() {
        return vec![TextReplacement {
            start: 0,
            end: original_source.len(),
            text: None,
        }];
    }
    replacements.sort_by_key(|r| r.start);
    let mut all: Vec<TextReplacement> = Vec::new();
    let mut last_end = 0usize;
    for r in replacements.into_iter() {
        if last_end < r.start {
            all.push(TextReplacement {
                start: last_end,
                end: r.start,
                text: None,
            });
        }
        last_end = r.end;
        all.push(r);
    }
    if last_end < original_source.len() {
        all.push(TextReplacement {
            start: last_end,
            end: original_source.len(),
            text: None,
        });
    }
    all
}

pub fn merge_replacements(
    filename: &str,
    original_source: &str,
    replacements: Vec<TextReplacement>,
) -> Result<(), DFixxerError> {
    if replacements.is_empty() {
        return Ok(());
    }

    let sections = compute_source_sections(original_source, &replacements);

    // Build final text by mapping each section to either replacement text or original slice
    let mut out = String::new();
    for section in sections {
        if let Some(r) = replacements
            .iter()
            .find(|tr| tr.start == section.start && tr.end == section.end && tr.text.is_some())
        {
            out.push_str(r.text.as_ref().unwrap());
        } else {
            out.push_str(&original_source[section.start..section.end]);
        }
    }

    std::fs::write(filename, out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_gaps_empty_replacements() {
        let source = "Hello, world!";
        let replacements: Vec<TextReplacement> = vec![];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![SourceSection {
                start: 0,
                end: source.len()
            }]
        );
    }

    #[test]
    fn test_fill_gaps_single_replacement() {
        let source = "Hello, world!";
        let replacements = vec![TextReplacement {
            start: 7,
            end: 12,
            text: Some("Rust".to_string()),
        }];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 7 },
                SourceSection { start: 7, end: 12 },
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
                text: Some("slow".to_string()),
            },
            TextReplacement {
                start: 10,
                end: 15,
                text: Some("green".to_string()),
            },
        ];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 4 },
                SourceSection { start: 4, end: 9 },
                SourceSection { start: 9, end: 10 },
                SourceSection { start: 10, end: 15 },
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
                text: Some("XX".to_string()),
            },
            TextReplacement {
                start: 3,
                end: 5,
                text: Some("YY".to_string()),
            },
        ];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 1 },
                SourceSection { start: 1, end: 3 },
                SourceSection { start: 3, end: 5 },
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
            text: Some("replaced".to_string()),
        }];
        let result = compute_source_sections(source, &replacements);
        assert_eq!(
            result,
            vec![SourceSection {
                start: 0,
                end: source.len()
            }]
        );
    }
}
