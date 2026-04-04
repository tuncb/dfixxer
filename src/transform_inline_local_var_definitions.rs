use crate::options::Options;
use crate::parser::{
    InlineLocalVarDefinitionActionKind, InlineLocalVarDefinitionContext,
    InlineLocalVarDefinitionRoutine,
};
use crate::replacements::TextReplacement;
use crate::transformer_utility::find_line_start;

fn line_prefix_if_indentation(source: &str, position: usize) -> Option<&str> {
    let line_start = find_line_start(source, position);
    let prefix = &source[line_start..position];
    prefix
        .chars()
        .all(|ch| ch.is_whitespace() && ch != '\n' && ch != '\r')
        .then_some(prefix)
}

fn owner_indent<'a>(source: &'a str, routine: &InlineLocalVarDefinitionRoutine) -> &'a str {
    line_prefix_if_indentation(source, routine.local_end_byte).unwrap_or("")
}

fn inserted_block_start_text(
    source: &str,
    routine: &InlineLocalVarDefinitionRoutine,
    options: &Options,
) -> Option<String> {
    let indent = format!("{}{}", owner_indent(source, routine), options.indentation);
    let mut actions: Vec<_> = routine
        .actions
        .iter()
        .filter(|action| action.kind == InlineLocalVarDefinitionActionKind::VarAtBlockStart)
        .collect();
    actions.sort_by_key(|action| action.declaration_order);

    if actions.is_empty() {
        return None;
    }

    let mut text = String::new();
    for action in actions {
        text.push_str(&format!(
            "{}{}var {}: {};",
            options.line_ending, indent, action.name, action.type_text
        ));
    }

    Some(text)
}

fn assignment_replacement_text(
    source: &str,
    action_kind: &InlineLocalVarDefinitionActionKind,
    name: &str,
    type_text: &str,
    expr_start_byte: usize,
    expr_end_byte: usize,
) -> String {
    let expr_text = &source[expr_start_byte..expr_end_byte];
    match action_kind {
        InlineLocalVarDefinitionActionKind::ConstFromAssignment => {
            format!("const {}: {} = {};", name, type_text, expr_text)
        }
        InlineLocalVarDefinitionActionKind::VarFromAssignment => {
            format!("var {}: {} := {};", name, type_text, expr_text)
        }
        InlineLocalVarDefinitionActionKind::VarAtBlockStart => unreachable!(),
    }
}

pub fn transform_inline_local_var_definitions(
    source: &str,
    context: &InlineLocalVarDefinitionContext,
    options: &Options,
) -> Vec<TextReplacement> {
    let mut replacements = Vec::new();

    for routine in &context.routines {
        replacements.push(TextReplacement {
            start: routine.local_start_byte,
            end: routine.local_end_byte,
            text: options.line_ending.to_string(),
        });

        if let Some(text) = inserted_block_start_text(source, routine, options) {
            replacements.push(TextReplacement {
                start: routine.begin_insert_at,
                end: routine.begin_insert_at,
                text,
            });
        }

        for action in &routine.actions {
            if action.kind == InlineLocalVarDefinitionActionKind::VarAtBlockStart {
                continue;
            }

            let (
                Some(statement_start_byte),
                Some(statement_end_byte),
                Some(expr_start_byte),
                Some(expr_end_byte),
            ) = (
                action.statement_start_byte,
                action.statement_end_byte,
                action.expr_start_byte,
                action.expr_end_byte,
            )
            else {
                continue;
            };

            replacements.push(TextReplacement {
                start: statement_start_byte,
                end: statement_end_byte,
                text: assignment_replacement_text(
                    source,
                    &action.kind,
                    &action.name,
                    &action.type_text,
                    expr_start_byte,
                    expr_end_byte,
                ),
            });
        }
    }

    replacements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::parse_with_contexts;
    use crate::replacements::apply_replacements_to_string;

    fn make_options() -> Options {
        Options {
            indentation: "  ".to_string(),
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    #[test]
    fn test_transform_inline_local_var_definitions_rewrites_safe_routine() {
        let source = r#"procedure Test;
var
  A: Integer;
  B: Integer;
  C: Integer;
begin
  A := 1;
  B := 2;
  B := B + 1;
end;"#;

        let (_, _, _, _, _, inline_context) = parse_with_contexts(source).expect("Failed to parse");
        let replacements =
            transform_inline_local_var_definitions(source, &inline_context, &make_options());
        let result = apply_replacements_to_string(source, &replacements);

        assert_eq!(
            result,
            r#"procedure Test;
begin
  var C: Integer;
  const A: Integer = 1;
  var B: Integer := 2;
  B := B + 1;
end;"#
        );
    }

    #[test]
    fn test_transform_inline_local_var_definitions_skips_read_before_write() {
        let source = r#"procedure Test;
var
  A: Integer;
begin
  Foo(A);
  A := 1;
end;"#;

        let (_, _, _, _, _, inline_context) = parse_with_contexts(source).expect("Failed to parse");
        let replacements =
            transform_inline_local_var_definitions(source, &inline_context, &make_options());

        assert!(replacements.is_empty());
    }

    #[test]
    fn test_transform_inline_local_var_definitions_skips_comments_inside_var_block() {
        let source = r#"procedure Test;
var
  A: Integer;
  // note
  B: Integer;
begin
  A := 1;
  B := 2;
end;"#;

        let (_, _, _, _, _, inline_context) = parse_with_contexts(source).expect("Failed to parse");
        let replacements =
            transform_inline_local_var_definitions(source, &inline_context, &make_options());

        assert!(replacements.is_empty());
    }
}
