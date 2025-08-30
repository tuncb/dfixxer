use crate::dfixxer_error::DFixxerError;

#[derive(Debug)]
pub struct TextReplacement {
    pub start: usize,
    pub end: usize,
    pub text: String,
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
