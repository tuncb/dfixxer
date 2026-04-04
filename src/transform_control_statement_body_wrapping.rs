use crate::options::Options;
use crate::parser::{ControlStatementBodyCandidate, ControlStatementBodyWrappingContext};
use crate::replacements::TextReplacement;
use crate::transformer_utility::{create_text_replacement_if_different, find_line_start};

fn line_prefix_if_indentation(source: &str, position: usize) -> Option<&str> {
    let line_start = find_line_start(source, position);
    let prefix = &source[line_start..position];
    prefix
        .chars()
        .all(|ch| ch.is_whitespace() && ch != '\n' && ch != '\r')
        .then_some(prefix)
}

fn owner_indent<'a>(source: &'a str, candidate: &ControlStatementBodyCandidate) -> &'a str {
    let line_start = find_line_start(source, candidate.owner_start_byte);
    &source[line_start..candidate.owner_start_byte]
}

fn target_body_indent(
    source: &str,
    candidate: &ControlStatementBodyCandidate,
    options: &Options,
) -> String {
    let owner_indent = owner_indent(source, candidate);
    let default_indent = format!("{}{}", owner_indent, options.indentation);

    match line_prefix_if_indentation(source, candidate.body_prefix_start_byte) {
        Some(existing_indent)
            if existing_indent.starts_with(owner_indent) && existing_indent != owner_indent =>
        {
            existing_indent.to_string()
        }
        _ => default_indent,
    }
}

fn begin_replacement(
    source: &str,
    candidate: &ControlStatementBodyCandidate,
    options: &Options,
) -> Option<TextReplacement> {
    let owner_indent = owner_indent(source, candidate);
    let body_indent = target_body_indent(source, candidate, options);
    let replacement_text = format!(
        "{}{}begin{}{}",
        options.line_ending, owner_indent, options.line_ending, body_indent
    );

    create_text_replacement_if_different(
        source,
        candidate.separator_end_byte,
        candidate.body_prefix_start_byte,
        replacement_text,
    )
}

fn end_replacement(
    source: &str,
    candidate: &ControlStatementBodyCandidate,
    options: &Options,
) -> TextReplacement {
    let owner_indent = owner_indent(source, candidate);
    TextReplacement {
        start: candidate.body_end_byte,
        end: candidate.body_end_byte,
        text: format!("{}{}end;", options.line_ending, owner_indent),
    }
}

/// Convert control-statement body wrapping candidates into non-overlapping replacements.
pub fn transform_control_statement_body_wrapping(
    source: &str,
    context: &ControlStatementBodyWrappingContext,
    options: &Options,
) -> Vec<TextReplacement> {
    let mut replacements = Vec::with_capacity(context.candidates.len() * 2);

    for candidate in &context.candidates {
        if let Some(replacement) = begin_replacement(source, candidate, options) {
            replacements.push(replacement);
        }
        replacements.push(end_replacement(source, candidate, options));
    }

    replacements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::{ControlStatementBodyWrappingContext, ControlStatementKind};

    fn make_options() -> Options {
        Options {
            indentation: "  ".to_string(),
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    fn make_candidate(
        source: &str,
        owner_text: &str,
        separator_text: &str,
        body_prefix_text: &str,
        body_text: &str,
    ) -> ControlStatementBodyCandidate {
        let owner_start_byte = source.find(owner_text).unwrap();
        let separator_start = source.find(separator_text).unwrap();
        let separator_end_byte = separator_start + separator_text.len();
        let body_prefix_start_byte = source.find(body_prefix_text).unwrap();
        let body_start_byte = source.find(body_text).unwrap();
        let body_end_byte = body_start_byte + body_text.len();

        ControlStatementBodyCandidate {
            kind: ControlStatementKind::For,
            owner_start_byte,
            separator_end_byte,
            body_prefix_start_byte,
            body_start_byte,
            body_end_byte,
        }
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_wraps_multiline_body() {
        let source = "begin\n  for I := 1 to 3 do\n    Foo;\nend.";
        let candidate = make_candidate(source, "for I := 1 to 3 do", "do", "Foo;", "Foo;");
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n  begin\n    ");
        assert_eq!(replacements[1].text, "\n  end;");
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_wraps_same_line_body() {
        let source = "begin\n  for I := 1 to 3 do Foo;\nend.";
        let candidate = make_candidate(source, "for I := 1 to 3 do Foo;", "do", "Foo;", "Foo;");
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n  begin\n    ");
        assert_eq!(replacements[1].text, "\n  end;");
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_preserves_existing_body_indent() {
        let source = "begin\n  for I := 1 to 3 do\n      // note\n      Foo;\nend.";
        let candidate = make_candidate(source, "for I := 1 to 3 do", "do", "// note", "Foo;");
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n  begin\n      ");
        assert_eq!(replacements[1].text, "\n  end;");
    }
}
