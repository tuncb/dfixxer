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

/// Generate sections for the gaps between existing replacements (not including the replacements themselves)
pub fn compute_source_sections(
    original_source: &str,
    replacements: &[TextReplacement],
    excluded_ranges: &[(usize, usize)],
) -> Vec<SourceSection> {
    if replacements.is_empty() && excluded_ranges.is_empty() {
        return vec![SourceSection {
            start: 0,
            end: original_source.len(),
        }];
    }

    let mut occupied_ranges: Vec<(usize, usize)> = replacements
        .iter()
        .map(|replacement| (replacement.start, replacement.end))
        .collect();
    occupied_ranges.extend_from_slice(excluded_ranges);
    occupied_ranges.sort_unstable_by_key(|(start, end)| (*start, *end));

    let mut sections: Vec<SourceSection> = Vec::new();
    let mut last_end = 0usize;

    for (start, end) in occupied_ranges {
        if last_end < start {
            sections.push(SourceSection {
                start: last_end,
                end: start,
            });
        }
        last_end = last_end.max(end);
    }

    if last_end < original_source.len() {
        sections.push(SourceSection {
            start: last_end,
            end: original_source.len(),
        });
    }

    sections
}

pub fn apply_replacements_to_string(
    original_source: &str,
    replacements: &[TextReplacement],
) -> String {
    if replacements.is_empty() {
        return original_source.to_string();
    }

    // Sort replacements by start position without mutating caller slice.
    let mut order: Vec<_> = replacements.iter().collect();
    order.sort_by_key(|r| r.start);

    // Build final text by processing original text and applying replacements.
    let mut out = String::new();
    let mut current_pos = 0usize;

    for replacement in order {
        if current_pos < replacement.start {
            out.push_str(&original_source[current_pos..replacement.start]);
        }
        out.push_str(&replacement.text);
        current_pos = replacement.end;
    }

    if current_pos < original_source.len() {
        out.push_str(&original_source[current_pos..]);
    }

    out
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
        let result = compute_source_sections(source, &replacements, &[]);
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
        let result = compute_source_sections(source, &replacements, &[]);
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
        let result = compute_source_sections(source, &replacements, &[]);
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
        let result = compute_source_sections(source, &replacements, &[]);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_fill_gaps_with_excluded_ranges() {
        let source = "abcdefghij";
        let replacements = vec![TextReplacement {
            start: 2,
            end: 4,
            text: "XX".to_string(),
        }];
        let excluded_ranges = vec![(6, 8)];

        let result = compute_source_sections(source, &replacements, &excluded_ranges);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 2 },
                SourceSection { start: 4, end: 6 },
                SourceSection { start: 8, end: 10 },
            ]
        );
    }

    #[test]
    fn test_fill_gaps_handles_zero_length_replacement_and_exclusion_same_start() {
        let source = "abcdefghij";
        let replacements = vec![TextReplacement {
            start: 4,
            end: 4,
            text: "()".to_string(),
        }];
        let excluded_ranges = vec![(4, 6)];

        let result = compute_source_sections(source, &replacements, &excluded_ranges);
        assert_eq!(
            result,
            vec![
                SourceSection { start: 0, end: 4 },
                SourceSection { start: 6, end: 10 },
            ]
        );
    }

    #[test]
    fn test_apply_replacements_to_string() {
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

        let result = apply_replacements_to_string(source, &replacements);
        assert_eq!(result, "The slow green fox");
    }
}
