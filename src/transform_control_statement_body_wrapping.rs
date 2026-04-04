use crate::options::Options;
use crate::parser::{
    ControlStatementBodyCandidate, ControlStatementBodyWrappingContext,
    ControlStatementClosingKind, ControlStatementKind,
};
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

fn is_terminating_control_statement_body(text: &str) -> bool {
    let trimmed = text.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return true;
    }

    let uppercase = trimmed.to_ascii_uppercase();
    ["EXIT", "CONTINUE", "BREAK", "RAISE", "ABORT", "HALT"]
        .iter()
        .any(|keyword| {
            uppercase == *keyword
                || uppercase.strip_prefix(keyword).is_some_and(|rest| {
                    rest.starts_with('(') || rest.starts_with(char::is_whitespace)
                })
        })
}

fn is_body_wrapping_enabled(candidate: &ControlStatementBodyCandidate, options: &Options) -> bool {
    match candidate.kind {
        ControlStatementKind::For | ControlStatementKind::Foreach => {
            options.transformations.enable_for_body_wrapping
        }
        ControlStatementKind::While => options.transformations.enable_while_body_wrapping,
        ControlStatementKind::IfThen | ControlStatementKind::Else => {
            options.transformations.enable_if_body_wrapping
        }
    }
}

fn should_skip_terminating_body(
    candidate: &ControlStatementBodyCandidate,
    options: &Options,
) -> bool {
    match candidate.kind {
        ControlStatementKind::For | ControlStatementKind::Foreach => {
            options.transformations.skip_terminating_for_body_wrapping
        }
        ControlStatementKind::While => options.transformations.skip_terminating_while_body_wrapping,
        ControlStatementKind::IfThen | ControlStatementKind::Else => {
            options.transformations.skip_terminating_if_body_wrapping
        }
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

fn tail_replacement(
    source: &str,
    candidate: &ControlStatementBodyCandidate,
    options: &Options,
) -> TextReplacement {
    let owner_indent = owner_indent(source, candidate);
    let closing_text = match candidate.closing_kind {
        ControlStatementClosingKind::End => "end",
        ControlStatementClosingKind::EndSemicolon => "end;",
    };
    let mut text = String::new();
    if candidate.insert_body_semicolon {
        text.push(';');
    }
    text.push_str(&source[candidate.body_end_byte..candidate.body_suffix_end_byte]);
    text.push_str(&format!(
        "{}{}{}",
        options.line_ending, owner_indent, closing_text
    ));
    if candidate.tail_end_byte > candidate.body_suffix_end_byte {
        text.push_str(&format!("{}{}", options.line_ending, owner_indent));
    }

    TextReplacement {
        start: candidate.body_end_byte,
        end: candidate.tail_end_byte,
        text,
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
        if !is_body_wrapping_enabled(candidate, options) {
            continue;
        }

        let body_text = &source[candidate.body_start_byte..candidate.body_end_byte];
        if should_skip_terminating_body(candidate, options)
            && is_terminating_control_statement_body(body_text)
        {
            continue;
        }

        if let Some(replacement) = begin_replacement(source, candidate, options) {
            replacements.push((replacement, candidate.owner_start_byte));
        }
        replacements.push((
            tail_replacement(source, candidate, options),
            candidate.owner_start_byte,
        ));
    }

    replacements.sort_by(|(a, a_owner), (b, b_owner)| {
        a.start.cmp(&b.start).then_with(|| b_owner.cmp(a_owner))
    });

    replacements
        .into_iter()
        .map(|(replacement, _)| replacement)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::{
        ControlStatementBodyWrappingContext, ControlStatementClosingKind, ControlStatementKind,
    };
    use crate::replacements::apply_replacements_to_string;

    fn make_options() -> Options {
        Options {
            indentation: "  ".to_string(),
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    fn make_candidate_with_kind(
        kind: ControlStatementKind,
        source: &str,
        owner_text: &str,
        separator_text: &str,
        body_prefix_text: &str,
        body_text: &str,
    ) -> ControlStatementBodyCandidate {
        let owner_start_byte = source.find(owner_text).unwrap();
        let separator_start = source[owner_start_byte..]
            .find(separator_text)
            .map(|offset| owner_start_byte + offset)
            .unwrap();
        let separator_end_byte = separator_start + separator_text.len();
        let body_prefix_start_byte = source[owner_start_byte..]
            .find(body_prefix_text)
            .map(|offset| owner_start_byte + offset)
            .unwrap();
        let body_start_byte = source[owner_start_byte..]
            .find(body_text)
            .map(|offset| owner_start_byte + offset)
            .unwrap();
        let body_end_byte = body_start_byte + body_text.len();

        ControlStatementBodyCandidate {
            kind,
            owner_start_byte,
            separator_end_byte,
            body_prefix_start_byte,
            body_start_byte,
            body_end_byte,
            body_suffix_end_byte: body_end_byte,
            tail_end_byte: body_end_byte,
            closing_kind: ControlStatementClosingKind::EndSemicolon,
            insert_body_semicolon: false,
        }
    }

    fn make_candidate(
        source: &str,
        owner_text: &str,
        separator_text: &str,
        body_prefix_text: &str,
        body_text: &str,
    ) -> ControlStatementBodyCandidate {
        make_candidate_with_kind(
            ControlStatementKind::For,
            source,
            owner_text,
            separator_text,
            body_prefix_text,
            body_text,
        )
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

    #[test]
    fn test_transform_control_statement_body_wrapping_wraps_if_then_with_semicolon_and_else_gap() {
        let source = "begin\n  if A then Foo\n  else\n    Bar;\nend.";
        let owner_start_byte = source.find("if A then").unwrap();
        let separator_start = source.find("then").unwrap();
        let body_start_byte = source.find("Foo").unwrap();
        let else_start_byte = source.find("else").unwrap();
        let candidate = ControlStatementBodyCandidate {
            kind: ControlStatementKind::IfThen,
            owner_start_byte,
            separator_end_byte: separator_start + "then".len(),
            body_prefix_start_byte: body_start_byte,
            body_start_byte,
            body_end_byte: body_start_byte + "Foo".len(),
            body_suffix_end_byte: body_start_byte + "Foo".len(),
            tail_end_byte: else_start_byte,
            closing_kind: ControlStatementClosingKind::End,
            insert_body_semicolon: true,
        };
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n  begin\n    ");
        assert_eq!(replacements[1].text, ";\n  end\n  ");
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_absorbs_comments_between_then_and_else() {
        let source = "begin\n  if A then\n    Foo\n    // note\n  else\n    Bar;\nend.";
        let owner_start_byte = source.find("if A then").unwrap();
        let separator_start = source.find("then").unwrap();
        let body_start_byte = source.find("Foo").unwrap();
        let body_end_byte = body_start_byte + "Foo".len();
        let comment_end_byte = source.find("// note").unwrap() + "// note".len();
        let else_start_byte = source.find("else").unwrap();
        let candidate = ControlStatementBodyCandidate {
            kind: ControlStatementKind::IfThen,
            owner_start_byte,
            separator_end_byte: separator_start + "then".len(),
            body_prefix_start_byte: body_start_byte,
            body_start_byte,
            body_end_byte,
            body_suffix_end_byte: comment_end_byte,
            tail_end_byte: else_start_byte,
            closing_kind: ControlStatementClosingKind::End,
            insert_body_semicolon: true,
        };
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[1].text, ";\n    // note\n  end\n  ");
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_preserves_same_line_trailing_comment() {
        let source = "begin\n  if A then Foo; // tail\nend.";
        let owner_start_byte = source.find("if A then").unwrap();
        let separator_start = source.find("then").unwrap();
        let body_start_byte = source.find("Foo;").unwrap();
        let comment_start_byte = source.find("// tail").unwrap();
        let comment_end_byte = comment_start_byte + "// tail".len();
        let candidate = ControlStatementBodyCandidate {
            kind: ControlStatementKind::IfThen,
            owner_start_byte,
            separator_end_byte: separator_start + "then".len(),
            body_prefix_start_byte: body_start_byte,
            body_start_byte,
            body_end_byte: body_start_byte + "Foo;".len(),
            body_suffix_end_byte: comment_end_byte,
            tail_end_byte: comment_end_byte,
            closing_kind: ControlStatementClosingKind::EndSemicolon,
            insert_body_semicolon: false,
        };
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![candidate],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert_eq!(replacements.len(), 2);
        assert_eq!(replacements[0].text, "\n  begin\n    ");
        assert_eq!(replacements[1].text, " // tail\n  end;");
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_respects_per_statement_enable_flags() {
        let source = "begin\n  for I := 1 to 3 do Foo;\n  while Ready do Step;\nend.";
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![
                make_candidate_with_kind(
                    ControlStatementKind::For,
                    source,
                    "for I := 1 to 3 do Foo;",
                    "do",
                    "Foo;",
                    "Foo;",
                ),
                make_candidate_with_kind(
                    ControlStatementKind::While,
                    source,
                    "while Ready do Step;",
                    "do",
                    "Step;",
                    "Step;",
                ),
            ],
        };

        let mut options = make_options();
        options.transformations.enable_for_body_wrapping = false;

        let replacements = transform_control_statement_body_wrapping(source, &context, &options);
        let result = apply_replacements_to_string(source, &replacements);

        assert!(result.contains("for I := 1 to 3 do Foo;"));
        assert!(result.contains("while Ready do\n  begin\n    Step;\n  end;"));
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_skips_terminating_if_when_configured() {
        let source = "begin\n  if A then Exit;\nend.";
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![make_candidate_with_kind(
                ControlStatementKind::IfThen,
                source,
                "if A then Exit;",
                "then",
                "Exit;",
                "Exit;",
            )],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());

        assert!(replacements.is_empty());
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_wraps_terminating_if_when_skip_disabled() {
        let source = "begin\n  if A then Exit;\nend.";
        let context = ControlStatementBodyWrappingContext {
            candidates: vec![make_candidate_with_kind(
                ControlStatementKind::IfThen,
                source,
                "if A then Exit;",
                "then",
                "Exit;",
                "Exit;",
            )],
        };

        let mut options = make_options();
        options.transformations.skip_terminating_if_body_wrapping = false;

        let replacements = transform_control_statement_body_wrapping(source, &context, &options);
        let result = apply_replacements_to_string(source, &replacements);

        assert!(result.contains("if A then\n  begin\n    Exit;\n  end;"));
    }

    #[test]
    fn test_transform_control_statement_body_wrapping_orders_nested_tail_insertions_inside_out() {
        let source = "begin\n  if A then\n    if B then\n      Foo\n    else\n      Bar\n  else\n    Baz;\nend.";

        let outer_owner_start = source.find("if A then").unwrap();
        let outer_then_start = source.find("then").unwrap() + "then".len();
        let outer_body_start = source.find("if B then").unwrap();
        let outer_body_end = source.find("Bar").unwrap() + "Bar".len();
        let outer_else_start = source.rfind("else").unwrap();

        let inner_owner_start = source.find("if B then").unwrap();
        let inner_then_start = source[inner_owner_start..]
            .find("then")
            .map(|offset| inner_owner_start + offset + "then".len())
            .unwrap();
        let inner_then_body_start = source.find("Foo").unwrap();
        let inner_else_start = source[inner_owner_start..]
            .find("else")
            .map(|offset| inner_owner_start + offset)
            .unwrap();
        let inner_final_else_start = source.rfind("Bar").unwrap();

        let context = ControlStatementBodyWrappingContext {
            candidates: vec![
                ControlStatementBodyCandidate {
                    kind: ControlStatementKind::IfThen,
                    owner_start_byte: outer_owner_start,
                    separator_end_byte: outer_then_start,
                    body_prefix_start_byte: outer_body_start,
                    body_start_byte: outer_body_start,
                    body_end_byte: outer_body_end,
                    body_suffix_end_byte: outer_body_end,
                    tail_end_byte: outer_else_start,
                    closing_kind: ControlStatementClosingKind::End,
                    insert_body_semicolon: false,
                },
                ControlStatementBodyCandidate {
                    kind: ControlStatementKind::IfThen,
                    owner_start_byte: inner_owner_start,
                    separator_end_byte: inner_then_start,
                    body_prefix_start_byte: inner_then_body_start,
                    body_start_byte: inner_then_body_start,
                    body_end_byte: inner_then_body_start + "Foo".len(),
                    body_suffix_end_byte: inner_then_body_start + "Foo".len(),
                    tail_end_byte: inner_else_start,
                    closing_kind: ControlStatementClosingKind::End,
                    insert_body_semicolon: true,
                },
                ControlStatementBodyCandidate {
                    kind: ControlStatementKind::Else,
                    owner_start_byte: inner_owner_start,
                    separator_end_byte: inner_else_start + "else".len(),
                    body_prefix_start_byte: inner_final_else_start,
                    body_start_byte: inner_final_else_start,
                    body_end_byte: inner_final_else_start + "Bar".len(),
                    body_suffix_end_byte: inner_final_else_start + "Bar".len(),
                    tail_end_byte: inner_final_else_start + "Bar".len(),
                    closing_kind: ControlStatementClosingKind::EndSemicolon,
                    insert_body_semicolon: true,
                },
                ControlStatementBodyCandidate {
                    kind: ControlStatementKind::Else,
                    owner_start_byte: outer_owner_start,
                    separator_end_byte: outer_else_start + "else".len(),
                    body_prefix_start_byte: source.find("Baz;").unwrap(),
                    body_start_byte: source.find("Baz;").unwrap(),
                    body_end_byte: source.find("Baz;").unwrap() + "Baz;".len(),
                    body_suffix_end_byte: source.find("Baz;").unwrap() + "Baz;".len(),
                    tail_end_byte: source.find("Baz;").unwrap() + "Baz;".len(),
                    closing_kind: ControlStatementClosingKind::EndSemicolon,
                    insert_body_semicolon: false,
                },
            ],
        };

        let replacements =
            transform_control_statement_body_wrapping(source, &context, &make_options());
        let result = apply_replacements_to_string(source, &replacements);

        assert!(result.contains("      Foo;\n    end\n    else\n    begin"));
        assert!(result.contains("      Bar;\n    end;\n  end\n  else"));
    }
}
