use crate::options::Options;
use crate::parser::{LocalRoutineBlock, LocalRoutineSpacingContext};
use crate::replacements::TextReplacement;
use crate::transformer_utility::{create_text_replacement_if_different, find_line_start};

fn leading_whitespace(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace() || *ch == '\n' || *ch == '\r')
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    &text[..end]
}

fn next_line_slice(text: &str, start: usize) -> (&str, &str, usize) {
    let bytes = text.as_bytes();
    let mut idx = start;
    while idx < bytes.len() && bytes[idx] != b'\n' && bytes[idx] != b'\r' {
        idx += 1;
    }

    let line = &text[start..idx];
    if idx >= bytes.len() {
        return (line, "", idx);
    }

    if bytes[idx] == b'\r' && idx + 1 < bytes.len() && bytes[idx + 1] == b'\n' {
        (line, "\r\n", idx + 2)
    } else {
        (line, &text[idx..idx + 1], idx + 1)
    }
}

fn normalize_line_indent(
    line_indent: &str,
    current_header_indent: &str,
    target_indent: &str,
) -> Option<String> {
    if current_header_indent.is_empty() {
        return Some(format!("{}{}", target_indent, line_indent));
    }

    if let Some(relative_indent) = line_indent.strip_prefix(current_header_indent) {
        return Some(format!("{}{}", target_indent, relative_indent));
    }

    if line_indent.chars().all(char::is_whitespace)
        && line_indent.chars().count() < current_header_indent.chars().count()
    {
        return Some(target_indent.to_string());
    }

    None
}

fn normalize_block_text(
    block_text: &str,
    current_header_indent: &str,
    target_indent: &str,
) -> Option<String> {
    let mut output = String::with_capacity(block_text.len());
    let mut idx = 0usize;

    while idx < block_text.len() {
        let (line, line_ending, next_idx) = next_line_slice(block_text, idx);
        if line.trim().is_empty() {
            output.push_str(line_ending);
            idx = next_idx;
            continue;
        }

        let line_indent = leading_whitespace(line);
        let content = &line[line_indent.len()..];
        let normalized_indent =
            normalize_line_indent(line_indent, current_header_indent, target_indent)?;
        output.push_str(&normalized_indent);
        output.push_str(content);
        output.push_str(line_ending);
        idx = next_idx;
    }

    Some(output)
}

fn block_replacement(
    source: &str,
    block: &LocalRoutineBlock,
    options: &Options,
) -> Option<TextReplacement> {
    let replacement_start = find_line_start(source, block.start_byte);
    let replacement_end = block.end_byte;
    let block_text = &source[replacement_start..replacement_end];

    let anchor_line_start = find_line_start(source, block.anchor_start_byte);
    let current_header_indent = &source[anchor_line_start..block.anchor_start_byte];

    let owner_line_start = find_line_start(source, block.owner_header_start_byte);
    let owner_indent = &source[owner_line_start..block.owner_header_start_byte];
    let target_indent = format!("{}{}", owner_indent, options.indentation);

    let replacement_text = normalize_block_text(block_text, current_header_indent, &target_indent)?;
    create_text_replacement_if_different(
        source,
        replacement_start,
        replacement_end,
        replacement_text,
    )
}

pub fn transform_local_routine_indentation(
    source: &str,
    context: &LocalRoutineSpacingContext,
    options: &Options,
) -> Vec<TextReplacement> {
    context
        .blocks
        .iter()
        .filter_map(|block| block_replacement(source, block, options))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LineEnding, Options};
    use crate::parser::{LocalRoutineBlock, LocalRoutineSpacingContext};

    fn make_options() -> Options {
        Options {
            indentation: "  ".to_string(),
            line_ending: LineEnding::Lf,
            ..Default::default()
        }
    }

    fn make_context(
        source: &str,
        block_text: &str,
        anchor_text: &str,
        owner_text: &str,
    ) -> LocalRoutineSpacingContext {
        let block_start = source.find(block_text).unwrap();
        let anchor_start = source.find(anchor_text).unwrap();
        let owner_start = source.find(owner_text).unwrap();
        LocalRoutineSpacingContext {
            gaps: Vec::new(),
            blocks: vec![LocalRoutineBlock {
                start_byte: block_start,
                end_byte: block_start + block_text.len(),
                anchor_start_byte: anchor_start,
                owner_header_start_byte: owner_start,
            }],
        }
    }

    #[test]
    fn test_transform_local_routine_indentation_indents_unindented_block() {
        let source = "procedure Outer;\nprocedure Inner;\nbegin\n  DoWork;\nend;\nbegin\nend;";
        let context = make_context(
            source,
            "procedure Inner;\nbegin\n  DoWork;\nend;",
            "procedure Inner;",
            "procedure Outer;",
        );

        let replacements = transform_local_routine_indentation(source, &context, &make_options());
        assert_eq!(replacements.len(), 1);
        assert_eq!(
            replacements[0].text,
            "  procedure Inner;\n  begin\n    DoWork;\n  end;".to_string()
        );
    }

    #[test]
    fn test_transform_local_routine_indentation_is_idempotent_when_already_aligned() {
        let source =
            "procedure Outer;\n  procedure Inner;\n  begin\n    DoWork;\n  end;\nbegin\nend;";
        let context = make_context(
            source,
            "  procedure Inner;\n  begin\n    DoWork;\n  end;",
            "procedure Inner;",
            "procedure Outer;",
        );

        let replacements = transform_local_routine_indentation(source, &context, &make_options());
        assert!(replacements.is_empty());
    }

    #[test]
    fn test_transform_local_routine_indentation_moves_attached_comments_and_directives() {
        let source = "procedure Outer;\n// helper\n{$IFDEF DEBUG}\nprocedure Inner;\nbegin\nend;\n{$ENDIF}\nbegin\nend;";
        let context = make_context(
            source,
            "// helper\n{$IFDEF DEBUG}\nprocedure Inner;\nbegin\nend;\n{$ENDIF}",
            "procedure Inner;",
            "procedure Outer;",
        );

        let replacements = transform_local_routine_indentation(source, &context, &make_options());
        assert_eq!(replacements.len(), 1);
        assert_eq!(
            replacements[0].text,
            "  // helper\n  {$IFDEF DEBUG}\n  procedure Inner;\n  begin\n  end;\n  {$ENDIF}"
                .to_string()
        );
    }

    #[test]
    fn test_transform_local_routine_indentation_keeps_empty_lines_empty() {
        let source = "procedure Outer;\nprocedure Inner;\nbegin\n\n  DoWork;\nend;\nbegin\nend;";
        let context = make_context(
            source,
            "procedure Inner;\nbegin\n\n  DoWork;\nend;",
            "procedure Inner;",
            "procedure Outer;",
        );

        let replacements = transform_local_routine_indentation(source, &context, &make_options());
        assert_eq!(replacements.len(), 1);
        assert_eq!(
            replacements[0].text,
            "  procedure Inner;\n  begin\n\n    DoWork;\n  end;".to_string()
        );
    }

    #[test]
    fn test_transform_local_routine_indentation_skips_ambiguous_mixed_indent_block() {
        let source = "procedure Outer;\n  procedure Inner;\n \tbegin\n  end;\nbegin\nend;";
        let context = make_context(
            source,
            "  procedure Inner;\n \tbegin\n  end;",
            "procedure Inner;",
            "procedure Outer;",
        );

        let replacements = transform_local_routine_indentation(source, &context, &make_options());
        assert!(replacements.is_empty());
    }
}
