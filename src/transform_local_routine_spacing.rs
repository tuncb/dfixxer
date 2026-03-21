use crate::options::Options;
use crate::parser::{LocalRoutineSpacingContext, LocalRoutineSpacingGap};
use crate::replacements::TextReplacement;
use crate::transformer_utility::create_text_replacement_if_different;

fn trailing_indentation(text: &str) -> &str {
    let start = text.rfind(['\n', '\r']).map(|idx| idx + 1).unwrap_or(0);
    &text[start..]
}

fn normalize_gap_text(
    source: &str,
    gap: &LocalRoutineSpacingGap,
    options: &Options,
) -> Option<String> {
    let original = &source[gap.start..gap.end];
    if !original.chars().all(char::is_whitespace) {
        return None;
    }

    let indentation = trailing_indentation(original);
    Some(format!(
        "{}{}{}",
        options.line_ending, options.line_ending, indentation
    ))
}

pub fn transform_local_routine_spacing(
    source: &str,
    context: &LocalRoutineSpacingContext,
    options: &Options,
) -> Vec<TextReplacement> {
    context
        .gaps
        .iter()
        .filter_map(|gap| {
            let replacement_text = normalize_gap_text(source, gap, options)?;
            create_text_replacement_if_different(source, gap.start, gap.end, replacement_text)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::{LocalRoutineSpacingContext, LocalRoutineSpacingGap};

    fn make_options() -> Options {
        Options {
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    #[test]
    fn test_transform_local_routine_spacing_expands_single_line_gap() {
        let source = "procedure Outer;\nprocedure Inner;\nbegin\nend;\nbegin\nend;";
        let inner_start = source.find("procedure Inner;").unwrap();
        let inner_end = source.find("end;\nbegin").unwrap() + "end;".len();
        let body_start = source.rfind("begin").unwrap();
        let context = LocalRoutineSpacingContext {
            gaps: vec![
                LocalRoutineSpacingGap {
                    start: "procedure Outer;".len(),
                    end: inner_start,
                },
                LocalRoutineSpacingGap {
                    start: inner_end,
                    end: body_start,
                },
            ],
            blocks: Vec::new(),
        };

        let replacements = transform_local_routine_spacing(source, &context, &make_options());
        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n\n".to_string());
        assert_eq!(replacements[1].text, "\n\n".to_string());
    }

    #[test]
    fn test_transform_local_routine_spacing_preserves_existing_indentation() {
        let source = "procedure Outer;\n  procedure Inner;\n  begin\n  end;\n  begin\n  end;";
        let context = LocalRoutineSpacingContext {
            gaps: vec![LocalRoutineSpacingGap {
                start: "procedure Outer;".len(),
                end: source.find("procedure Inner;").unwrap(),
            }],
            blocks: Vec::new(),
        };

        let replacements = transform_local_routine_spacing(source, &context, &make_options());
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].text, "\n\n  ".to_string());
    }

    #[test]
    fn test_transform_local_routine_spacing_skips_non_whitespace_gap() {
        let source = "procedure Outer;//oops\nprocedure Inner;\nbegin\nend;\nbegin\nend;";
        let context = LocalRoutineSpacingContext {
            gaps: vec![LocalRoutineSpacingGap {
                start: "procedure Outer;".len(),
                end: source.find("procedure Inner;").unwrap(),
            }],
            blocks: Vec::new(),
        };

        let replacements = transform_local_routine_spacing(source, &context, &make_options());
        assert!(replacements.is_empty());
    }
}
