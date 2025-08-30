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

pub fn apply_replacements(
    filename: &str,
    original_source: &str,
    mut replacements: Vec<TextReplacement>,
) -> Result<(), DFixxerError> {
    if replacements.is_empty() {
        return Ok(());
    }

    // Sort replacements by start position in reverse order
    // This allows us to apply them from end to beginning to avoid offset issues
    replacements.sort_by(|a, b| b.start.cmp(&a.start));

    let mut modified_source = original_source.to_string();

    // Apply each replacement
    for replacement in replacements {
        // Ensure the replacement range is valid
        if replacement.start <= replacement.end && replacement.end <= modified_source.len() {
            modified_source.replace_range(replacement.start..replacement.end, &replacement.text);
        }
    }

    // Write the modified source back to the file
    std::fs::write(filename, modified_source)?;

    Ok(())
}
